use crate::value::{Term, Variant, Tuple, CastFrom};
use crate::process::{PID, Ref};
use crate::exception::{Exception};
use crate::atom;
use crate::value;
use crate::vm::Machine;
use crate::servo_arc::Arc;

use hashbrown::HashMap;
use parking_lot::{RwLock, Mutex, MutexGuard};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use futures::{
    select,
    channel::mpsc,
    prelude::*,
};

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
    //     self.chan.send_async(Signal::Command(msg)).await;
    // }

    // pub async fn control(&mut self, sig: usize) -> Result<(), mpsc::SendError> {
    //     self.chan.send_async(Signal::Control(sig)).await
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
    _opts: Term
) -> Result<ID, Exception> {
    let tup = Tuple::cast_from(&args)?;

    // TODO: opts
    let (port, input) = mpsc::unbounded::<Signal>();
    // put the port (sender) in a ports table
    let pid = vm.port_table.write().insert(owner, port);

    match tup[0].into_variant() {
        Variant::Atom(atom::SPAWN) => {
            match tup[1].into_variant() {
                Variant::Atom(atom::TTY_SL) => vm.runtime.executor().spawn(tty(pid, owner, input)),
                Variant::Cons(..) => {
                    let cons = value::Cons::cast_from(&tup[1]).unwrap();
                    match value::cons::unicode_list_to_buf(cons, 2048).unwrap().as_ref() {
                        "tty_sl -c -e" => vm.runtime.executor().spawn(tty(pid, owner, input)),
                        _ => unimplemented!("port::spawn for {}", args),
                    };
                }
                _ => unimplemented!("port::spawn for {}", args),
            }
        }
        Variant::Atom(atom::FD) => {
            match (tup[1].into_variant(), tup[2].into_variant()) {
                (Variant::Integer(2), Variant::Integer(2)) => vm.runtime.executor().spawn(stderr(pid, owner, input)),
                _ => unimplemented!("port::spawn for {}", args),
            }
        }
        _ => unimplemented!("port::spawn for {}", args),
    };

    Ok(pid)
}

pub fn send_message(
    vm: &Machine,
    from: PID,
    port: ID,
    msg: Term
    ) -> Result<Term, Exception> {
     // info!("sending from {} to port {} msg {}", from, port, msg);

    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        // TODO: error unhandled
        let tup = Tuple::cast_from(&msg)?;
        if !tup.len() == 2 || !tup[0].is_pid() {
            return Err(badarg!());
        }
        match Tuple::cast_from(&tup[1]) {
            Ok(cmd) => {
                match cmd[0].into_variant() {
                    // TODO: some commands are [id | binary]
                    Variant::Atom(atom::COMMAND) => {
                        // TODO: validate tuple len 2
                        let bytes = crate::bif::erlang::list_to_iodata(cmd[1]).unwrap();
                        vm.runtime.block_on(chan.send(Signal::Command(bytes)));
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

pub fn close(vm: &Machine, from: PID, port: ID) -> Result<(), Exception> {
    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());

    if let Some(mut chan) = res {
        // TODO: error unhandled

        let future = async move {chan.send(Signal::Close).await; };
        vm.runtime.executor().spawn(future);
    }
    Ok(())
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
    // unimplemented!();
    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        let reference = vm.next_ref();

        // TODO: error unhandled
        let bytes = crate::bif::erlang::list_to_iodata(msg).unwrap();
        // TODO: can probably do without await, if we make sure we don't need 'static
        let future = async move {
            chan.send(Signal::Control {
                from,
                reference,
                opcode,
                data: bytes,
            }).await;
        };
        vm.runtime.executor().spawn(future);

        Ok(reference)
    } else {
        // TODO: handle errors properly
        println!("NOTFOUND");
        Ok(1)
    }
}

// TODO: needs type async fn
type Driver = fn(owner: PID, input: mpsc::UnboundedReceiver<Signal>);

// Port driver implementations.

mod renderer;
use renderer::Renderer;

// termios
use std::{io, mem};
use libc::c_int;

pub fn get_terminal_attr() -> io::Result<Termios> {
    extern "C" {
        pub fn tcgetattr(fd: c_int, termptr: *mut Termios) -> c_int;
    }
    unsafe {
        let mut termios = mem::zeroed();
        cvt(tcgetattr(0, &mut termios))?;
        Ok(termios)
    }
}

pub fn set_terminal_attr(termios: &Termios) -> io::Result<()> {
    extern "C" {
        pub fn tcsetattr(fd: c_int, opt: c_int, termptr: *const Termios) -> c_int;
    }
    cvt(unsafe { tcsetattr(0, 0, termios) }).and(Ok(()))
}

pub fn raw_terminal_attr(termios: &mut Termios) {
    extern "C" {
        pub fn cfmakeraw(termptr: *mut Termios);
    }
    unsafe { cfmakeraw(termios) }

}

use libc::termios as Termios;

// Support functions for converting libc return values to io errors {
trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
        ($($t:ident)*) => ($(impl IsMinusOne for $t {
            fn is_minus_one(&self) -> bool {
                *self == -1
            }
        })*)
    }

impl_is_minus_one! { i8 i16 i32 i64 isize }

fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
// } End of support functions
// termios

async fn tty(id: ID, owner: PID, input: mpsc::UnboundedReceiver<Signal>) {
    use termion::terminal_size;
    let mut buf: [u8;1024] = [0;1024];
    let mut stdin = tokio::io::stdin();
    let mut input = input.fuse();

    let out = std::io::stdout();

    // for raw mode
    use termion::raw::IntoRawMode;
    let mut stdout = std::io::stdout().into_raw_mode().unwrap();

    let mut ios = get_terminal_attr().unwrap();
    ios.c_lflag |= (libc::ISIG); // IEXTEN
    ios.c_cc[libc::VMIN] = 1;
    ios.c_cc[libc::VTIME] = 0;
    set_terminal_attr(&ios).unwrap();

    // need to disable echo and canon

    // TODO FIXME: this heap will get trashed if tty shuts down
    let heap = crate::immix::Heap::new();

    const TTYSL_DRV_CONTROL_MAGIC_NUMBER: usize = 0x018b_0900;

    let mut renderer = Renderer::new(out);

    loop {
        select! {
            msg = input.next() => {
                // process command
                match msg {
                    // * Port ! {Owner, {command, Data}}
                    Some(Signal::Command(bytes)) => {
                        match bytes[0] {
                            // PUTC
                            0 => {

                                // info!("put_chars: bytes={:?}", &bytes[1..]);
                                renderer.put_chars(&bytes[1..]);
                            }
                            // 1 MOVE
                            1 => {
                                renderer.move_rel(&bytes[1..]);
                            }
                            // 2 INSC
                            2 => {
                                // info!("insert_chars: bytes={:?}", &bytes[1..]);
                                renderer.insert_chars(&bytes[1..]);
                            }
                            // 3 DELC
                            3 => {
                                renderer.delete_chars(&bytes[1..]);
                            }
                            // BEEP
                            4 => {
                                // info!("beep");
                                renderer.beep();
                            }
                            // PUTC_SYNC
                            5 => {
                                // info!("putc_sync: bytes={:?}", &bytes[1..]);
                                renderer.put_chars(&bytes[1..]);

                                crate::process::send_signal(&Machine::current(), owner, crate::process::Signal::Message {
                                    from: id, // TODO: this was supposed to be port id, not pid
                                    value: tup2!(&heap, Term::port(id), atom!(OK))
                                });
                            }
                            n => unimplemented!("command {} for tty", n),
                        }
                        // POP first char as command
                        // on 5 == sync_putc, we need to send back an ack
                        // {port, :ok}
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
                    Some(Signal::Control{from, reference, opcode, ..}) => {
                        match opcode - TTYSL_DRV_CONTROL_MAGIC_NUMBER {
                            // WINSIZE
                            100 => {
                                let (w, h) = terminal_size().unwrap();
                                let w = u32::from(w).to_ne_bytes();
                                let h = u32::from(h).to_ne_bytes();
                                let bytes = &[w, h].concat();

                                // basically bitstring!
                                let mut list = Term::nil();
                                for char in bytes.iter().copied().rev() {
                                    list = cons!(&heap, Term::int(i32::from(char)), list);
                                }

                                crate::process::send_signal(&Machine::current(), from, crate::process::Signal::Message {
                                    from: id, // TODO: this was supposed to be port id, not pid
                                    value: tup2!(&heap, Term::reference(&heap, reference), list)
                                });
                            },
                            // GET_UNICODE_STATE
                            101 => {
                                crate::process::send_signal(&Machine::current(), from, crate::process::Signal::Message {
                                    from: id, // TODO: this was supposed to be port id, not pid
                                    value: tup2!(&heap, Term::reference(&heap, reference), cons!(&heap, Term::int(1), Term::nil()))
                                });
                            },
                            // SET_UNICODE_STATE
                            102 => {
                                crate::process::send_signal(&Machine::current(), from, crate::process::Signal::Message {
                                    from: id, // TODO: this was supposed to be port id, not pid
                                    value: tup2!(&heap, Term::reference(&heap, reference), cons!(&heap, Term::int(1), Term::nil()))
                                });
                            },
                            _ => {
                                crate::process::send_signal(&Machine::current(), from, crate::process::Signal::Message {
                                    from: id, // TODO: this was supposed to be port id, not pid
                                    value: tup2!(&heap, Term::reference(&heap, reference), atom!(BADARG))
                                });
                                break;
                            }
                        }
                    }

                    // TODO, drop Signal::Close, just close sender
                    None => break,
                }
            },
            // TODO: use a larger buffer and check ret val for how many bytes we've read
            res = stdin.read(&mut buf).fuse() => {
                match res {
                    Ok(bytes) => {
                        // info!("id: {} , owner {} recv! {:?}", id, owner, &buf[..bytes]);
                        let vm = Machine::current();
                        // need to return a tuple, but want to avoid heap alloc here..
                        let bin = Arc::new(crate::bitstring::Binary::from(&buf[..bytes]));
                        crate::process::send_signal(&vm, owner, crate::process::Signal::PortMessage {
                            from: id,
                            value: bin
                        });
                    },
                    Err(err) => panic!(err)
                }
                // send {port, {:data, <bytes>}} back
            },
        }
    }
}

async fn stderr(id: ID, _owner: PID, input: mpsc::UnboundedReceiver<Signal>) {
    let mut stderr = tokio::io::stderr();
    let mut input = input.fuse();

    loop {
        match input.next().await {
            // * Port ! {Owner, {command, Data}}
            Some(Signal::Command(bytes)) => {
                stderr.write_all(&bytes).await;
                // stderr.flush().unwrap();
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
                unimplemented!("unimplemented control");
            }

            // TODO, drop Signal::Close, just close sender
            None => break,
        }
        // port_control stuff (op_get_winsize)
    }
}
