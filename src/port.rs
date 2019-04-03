use crate::value::{Term, Variant, Tuple, TryFrom};
use crate::process::{PID, Ref};
use crate::exception::{Exception, Reason};
use crate::atom;
use crate::vm::Machine;

use hashbrown::HashMap;
use parking_lot::{RwLock, Mutex, MutexGuard};

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
    owner: PID,
    // chan: mpsc::UnboundedSender<Signal>,
    pub chan: mpsc::UnboundedSender<Signal>,
}

impl Port {
    fn new(id: ID, owner: PID, chan: mpsc::UnboundedSender<Signal>) -> Self {
        Port {
            id,
            owner,
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
    Connect(PID), // new_owner
    Control {
        from: PID,
        reference: Ref,
        opcode: usize,
        data: Vec<u8>
    }, // usize => a set of constant predefined values
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

    pub fn insert(&mut self, owner: PID, chan: mpsc::UnboundedSender<Signal>) -> ID {
        let pid = self.next_pid();
        let port = Mutex::new(Port::new(pid, owner, chan));
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
    owner: PID,
    args: Term,
    opts: Term
) -> Result<ID, Exception> {
    let tup = Tuple::try_from(&args)?;

    // TODO: opts 
    let (port, input) = mpsc::unbounded::<Signal>();

    match tup[0].into_variant() {
        Variant::Atom(atom::SPAWN) => {
            match tup[1].into_variant() {
                Variant::Atom(atom::TTY_SL) => tokio::spawn_async(tty(owner, input)),
                _ => unimplemented!(),
            }
        }
        Variant::Atom(atom::FD) => {
            match (tup[1].into_variant(), tup[2].into_variant()) {
                (Variant::Integer(2), Variant::Integer(2)) => tokio::spawn_async(stderr(owner, input)),
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    };

    // put the port (sender) in a ports table
    let pid = vm.port_table.write().insert(owner, port);
    Ok(pid)
}

pub fn send_message(
    vm: &Machine,
    from: PID,
    port: ID,
    msg: Term
    ) -> Result<Term, Exception> {
    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        // TODO: error unhandled
        use futures::sink::SinkExt as FuturesSinkExt;
        use futures::future::{FutureExt, TryFutureExt};
        let tup = Tuple::try_from(&msg)?;
        if !tup.len() == 2 || !tup[0].is_pid() {
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        let mut chan = chan.compat();
        match Tuple::try_from(&tup[1]) {
            Ok(cmd) => {
                match cmd[0].into_variant() {
                    // TODO: some commands are [id | binary]
                    Variant::Atom(atom::COMMAND) => {
                        // TODO: validate tuple len 2
                        let bytes = crate::bif::erlang::list_to_iodata(cmd[1]).unwrap();
                        // let fut = chan
                        //     .send(port::Signal::Command(bytes))
                        //     .map_err(|_| ())
                        //     .boxed()
                        //     .compat();
                        // TODO: can probably do without await!, if we make sure we don't need 'static
                        tokio::spawn_async(async move { await!(chan.send(Signal::Command(bytes))); });
                    }
                    _ => unimplemented!("msg to port {}", msg),
                }
            }
            _ => unimplemented!()
        }
    } else {
        // TODO: handle errors properly
        println!("NOTFOUND");
    }
    Ok(msg)
}

/// Schedules a port operation and returns a ref. When we're done, need to reply to sender with
/// {ref, data}.
pub fn control(
    vm: &Machine,
    from: PID,
    port: ID,
    opcode: usize,
    msg: Term,
    ) -> Result<Ref, Exception> {
    unimplemented!();
    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        let reference = vm.next_ref();

        // TODO: error unhandled
        use futures::sink::SinkExt as FuturesSinkExt;
        let bytes = msg.to_bytes().unwrap().to_owned();
        // let fut = chan
        //     .send(port::Signal::Command(bytes))
        //     .map_err(|_| ())
        //     .boxed()
        //     .compat();
        //     TODO: can probably do without await!, if we make sure we don't need 'static
        tokio::spawn_async(async move {
            await!(chan.send(Signal::Control {
                from,
                reference,
                opcode,
                data: bytes,
            }));
        });

        Ok(reference)
    } else {
        // TODO: handle errors properly
        println!("NOTFOUND");
        Ok(1)
    }
}

// TODO: needs type async fn
type Driver = fn(owner: PID, input: mpsc::UnboundedReceiver<Signal>); 

/// Port driver implementations.

async fn tty(owner: PID, mut input: mpsc::UnboundedReceiver<Signal>) {
    let mut buf = [0;1024];
    let mut stdin = tokio::io::stdin();
    let mut input = input.fuse();

    let mut out = std::io::stdout();

    // need to disable echo and canon

    loop {
        select! {
            msg = input.next() => {
                // process command
                match msg {
                    // * Port ! {Owner, {command, Data}}
                    Some(Signal::Command(bytes)) => {
                        out.write_all(&bytes).unwrap();
                        out.flush().unwrap();
                    }
                    // * Port ! {Owner, {connect, NewOwner}}
                    Some(Signal::Connect(_new_owner)) => {
                        unimplemented!()
                    },
                    // * Port ! {Owner, close}
                    Some(Signal::Close) => {
                        break;
                        // TODO: any cleanup etc?
                    },

                    // port_control stuff (op_get_winsize)
                    Some(Signal::Control{..}) => {
                    }
  
                    // TODO, drop Signal::Close, just close sender
                    None => break,
                }
            },
            // // TODO: use a larger buffer and check ret val for how many bytes we've read
            // res = stdin.read_async(&mut buf).fuse() => {
            //     match res {
            //         Ok(bytes) => println!("read byte! num {}, {:?}", bytes, &buf[..bytes]),
            //         Err(err) => panic!(err)
            //     }
            //     // send {port, {:data, <bytes>}} back
            // },
        }
    }
    ()
}

async fn stderr(owner: PID, mut input: mpsc::UnboundedReceiver<Signal>) {
    let mut stderr = tokio::io::stderr();
    let mut input = input.fuse();

    loop {
        let msg = await!(input.next());
        // process command
        // match msg {
        // input says {command, write}, so write
        // port_control stuff (op_get_winsize)
        // if :close, break loop
        // }
    }
    ()
}
