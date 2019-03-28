use crate::value::{Term, Variant, Tuple, TryFrom};
use crate::process::PID;
use crate::exception::Exception;
use crate::atom;
use crate::vm::Machine;

use crate::servo_arc::Arc;
use hashbrown::HashMap;
use parking_lot::{RwLock, Mutex, MutexGuard};

use std::future::Future;

use tokio::await;
use tokio::prelude::*;
use crate::tokio::prelude::SinkExt;

use futures::channel::mpsc;
use futures::select;
use futures::future::{FutureExt, TryFutureExt};
use futures::stream::{StreamExt, FusedStream};
use futures::sink::SinkExt as FuturesSinkExt;
use futures::compat::Compat;

/// The type of a PID.
pub type ID = u32;

/// The maximum PID value.
pub const MAX_ID: ID = std::u32::MAX;

pub struct Port {
    id: ID,
    parent: PID,
    // chan: mpsc::UnboundedSender<Signal>,
    pub chan: mpsc::UnboundedSender<Signal>,
}

impl Port {
    fn new(id: ID, parent: PID, chan: mpsc::UnboundedSender<Signal>) -> Self {
        Port {
            id,
            parent,
            chan
        }
    }

    // TODO: probably better to return the future here and await outside
    // pub async fn send_message(&mut self, msg: Vec<u8>) { // Result<(), mpsc::SendError> {
    //     await!(self.chan.send_async(Signal::Command(msg)));
    // }

    // pub async fn control(&mut self, sig: usize) -> Result<(), mpsc::SendError> {
    //     await!(self.chan.send_async(Signal::Control(sig)))
    // }
}

pub enum Signal {
    Command(Vec<u8>), // TODO: zero-copy passing via slice would be better
    Control(usize), // usize => a set of constant predefined values
    Close
}

pub type RcTable = RwLock<Table>; // TODO: I don't like this lock at all

pub struct Table {
    /// The PID to use for the next port.
    next_pid: ID,

    ports: HashMap<ID, Mutex<Port>>
}

impl Table {
    pub fn new() -> RcTable {
        RwLock::new(Table { next_pid: 0, ports: HashMap::new() })
    }

    pub fn insert(&mut self, parent: PID, chan: mpsc::UnboundedSender<Signal>) -> ID {
        let pid = self.next_pid();
        let port = Mutex::new(Port::new(pid, parent, chan));
        self.ports.insert(pid, port);
        pid
    }

    /// Looks up a Port. Will lock the port until the reference is dropped.
    pub fn lookup(&self, pid: ID) -> Option<MutexGuard<Port>> {
        self.ports.get(&pid).map(|port| port.lock())
    }

    fn next_pid(&mut self) -> ID {
        let pid = self.next_pid;

        if pid == MAX_ID {
            self.next_pid = 0;
            // self.recycle = true;
        } else {
            self.next_pid += 1;
        }

        pid
    }
}

pub fn spawn(
    vm: &Machine,
    parent: PID,
    args: Term,
    opts: Term
) -> Result<ID, Exception> {
    let tup = Tuple::try_from(&args)?;

    match tup[0].into_variant() {
        Variant::Atom(atom::SPAWN) => {
            match tup[1].into_variant() {
                Variant::Atom(atom::TTY_SL) => {
                    // TODO: opts 
                    let (port, input) = mpsc::unbounded::<Signal>();
                    // TODO: put the port (sender) in a ports table
                    let fut = tty(parent, input);
                    tokio::spawn_async(fut);

                    let pid = vm.port_table.write().insert(parent, port);
                    Ok(pid)
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

// TODO: needs type async fn
type Driver = fn(parent: PID, input: mpsc::UnboundedReceiver<Signal>); 

/// Port driver implementations.

async fn tty(parent: PID, mut input: mpsc::UnboundedReceiver<Signal>) {
    let mut buf = [0;1];
    let mut stdin = tokio::io::stdin();
    let mut input = input.fuse();

    // need to disable echo and canon

    loop {
        println!("Looping!");
        select! {
            msg = input.next() => {
                // process command
                // match msg {
                    // input says {command, write}, so write
                    // port_control stuff (op_get_winsize)
                    // if :close, break loop
                // }
            }
            _ = stdin.read_async(&mut buf).fuse() => {
                println!("read byte! {:?}", buf);
                // send {port, {:data, <bytes>}} back
            }
        }
    }
    ()
}

// async fn stderr(parent: PID, mut input: mpsc::UnboundedReceiver<Signal>) {
//     let mut buf = [0;1];
//     let mut stderr = tokio::io::stderr();

//     loop {
//         select! {
//             msg = input.next().fuse() => {
//                 // process command
//                 // match msg {
//                     // input says {command, write}, so write
//                     // port_control stuff (op_get_winsize)
//                     // if :close, break loop
//                 // }
//             }
//             _ = stderr.read_async(&mut buf).fuse() => {
//                 // send {port, {:data, <bytes>}} back
//             }
//         }
//     }
//     ()
// }
