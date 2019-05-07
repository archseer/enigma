use crate::value::{Term, Variant, Tuple, TryFrom};
use crate::process::{PID, Ref};
use crate::exception::{Exception, Reason};
use crate::atom;
use crate::vm::Machine;
use crate::servo_arc::Arc;

use hashbrown::HashMap;
use parking_lot::{RwLock, Mutex, MutexGuard};

use tokio::prelude::*;

use futures::channel::mpsc;
use futures::select;
use futures::{
  compat::*,
  future::{FutureExt, TryFutureExt},
  io::AsyncWriteExt,
  stream::StreamExt,
  sink::SinkExt,
};
use futures::io::AsyncReadExt;

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
    _opts: Term
) -> Result<ID, Exception> {
    let tup = Tuple::try_from(&args)?;

    // TODO: opts 
    let (port, input) = mpsc::unbounded::<Signal>();
    // put the port (sender) in a ports table
    let pid = vm.port_table.write().insert(owner, port);

    match tup[0].into_variant() {
        Variant::Atom(atom::SPAWN) => {
            match tup[1].into_variant() {
                Variant::Atom(atom::TTY_SL) => vm.runtime.executor().spawn(tty(pid, owner, input).unit_error().boxed().compat()),
                _ => unimplemented!("port::spawn for {}", args),
            }
        }
        Variant::Atom(atom::FD) => {
            match (tup[1].into_variant(), tup[2].into_variant()) {
                (Variant::Integer(2), Variant::Integer(2)) => vm.runtime.executor().spawn(stderr(pid, owner, input).unit_error().boxed().compat()),
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
    // println!("sending from {} to port {} msg {}", from, port, msg);

    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        // TODO: error unhandled
        let tup = Tuple::try_from(&msg)?;
        if !tup.len() == 2 || !tup[0].is_pid() {
            return Err(Exception::new(Reason::EXC_BADARG));
        }
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
                        let future = async move { await!(chan.send(Signal::Command(bytes))); };
                        vm.runtime.executor().spawn(future.unit_error().boxed().compat());
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
    // unimplemented!();
    let res = vm.port_table.read().lookup(port).map(|port| port.chan.clone());
    if let Some(mut chan) = res {
        let reference = vm.next_ref();

        // TODO: error unhandled
        use futures::sink::SinkExt as FuturesSinkExt;
        let bytes = crate::bif::erlang::list_to_iodata(msg).unwrap();
        // let fut = chan
        //     .send(port::Signal::Command(bytes))
        //     .map_err(|_| ())
        //     .boxed()
        //     .compat();
        //     TODO: can probably do without await!, if we make sure we don't need 'static
        let future = async move {
            await!(chan.send(Signal::Control {
                from,
                reference,
                opcode,
                data: bytes,
            }));
        };
        vm.runtime.executor().spawn(future.unit_error().boxed().compat());

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

// use crate::keymap::{At, CharSearch, Movement, usize, Word};
use std::fmt;
use std::iter;
use std::ops::{Deref, Index, Range};
use std::string::Drain;
use unicode_segmentation::UnicodeSegmentation;

/// Maximum buffer size for the line read
pub(crate) static MAX_LINE: usize = 4096;

/// Delete (kill) direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Forward,
    Backward,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Forward
    }
}

/// Represents the current input (text and cursor position).
///
/// The methods do text manipulations or/and cursor movements.
pub struct LineBuffer {
    buf: String, // Edited line buffer (rl_line_buffer)
    pos: usize,  // Current cursor position (byte position) (rl_point)
}

impl fmt::Debug for LineBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineBuffer")
            .field("buf", &self.buf)
            .field("pos", &self.pos)
            .finish()
    }
}

impl LineBuffer {
    /// Create a new line buffer with the given maximum `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: String::with_capacity(capacity),
            pos: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn init(
        line: &str,
        pos: usize
    ) -> Self {
        let mut lb = Self::with_capacity(MAX_LINE);
        assert!(lb.insert_str(0, line));
        lb.set_pos(pos);
        lb
    }

    /// Extracts a string slice containing the entire buffer.
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Converts a buffer into a `String` without copying or allocating.
    pub fn into_string(self) -> String {
        self.buf
    }

    /// Current cursor position (byte position)
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Set cursor position (byte position)
    pub fn set_pos(&mut self, pos: usize) {
        assert!(pos <= self.buf.len());
        self.pos = pos;
    }

    /// Returns the length of this buffer, in bytes.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if this buffer has a length of zero.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Set line content (`buf`) and cursor position (`pos`).
    pub fn update(&mut self, buf: &str, pos: usize) {
        assert!(pos <= buf.len());
        let end = self.len();
        self.drain(0..end, Direction::default());
        let max = self.buf.capacity();
        if buf.len() > max {
            self.insert_str(0, &buf[..max]);
            if pos > max {
                self.pos = max;
            } else {
                self.pos = pos;
            }
        } else {
            self.insert_str(0, buf);
            self.pos = pos;
        }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        let end = self.len();
        self.drain(0..end, Direction::default());
        self.pos = 0;
    }

    /// Returns the position of the character just after the current cursor
    /// position.
    pub fn next_pos(&self, n: usize) -> Option<usize> {
        if self.pos == self.buf.len() {
            return None;
        }
        self.buf[self.pos..]
            .grapheme_indices(true)
            .take(n)
            .last()
            .map(|(i, s)| i + self.pos + s.len())
    }

    /// Returns the position of the character just before the current cursor
    /// position.
    fn prev_pos(&self, n: usize) -> Option<usize> {
        if self.pos == 0 {
            return None;
        }
        self.buf[..self.pos]
            .grapheme_indices(true)
            .rev()
            .take(n)
            .last()
            .map(|(i, _)| i)
    }

    /// Insert the character `ch` at current cursor position
    /// and advance cursor position accordingly.
    /// Return `None` when maximum buffer size has been reached,
    /// `true` when the character has been appended to the end of the line.
    pub fn insert(&mut self, ch: char, n: usize) -> Option<bool> {
        let shift = ch.len_utf8() * n;
        if self.buf.len() + shift > self.buf.capacity() {
            return None;
        }
        let push = self.pos == self.buf.len();
        if n == 1 {
            self.buf.insert(self.pos, ch);
        } else {
            let text = iter::repeat(ch).take(n).collect::<String>();
            let pos = self.pos;
            self.insert_str(pos, &text);
        }
        self.pos += shift;
        Some(push)
    }

    /// Move cursor on the left.
    pub fn move_backward(&mut self, n: usize) -> bool {
        match self.prev_pos(n) {
            Some(pos) => {
                self.pos = pos;
                true
            }
            None => false,
        }
    }

    /// Move cursor on the right.
    pub fn move_forward(&mut self, n: usize) -> bool {
        match self.next_pos(n) {
            Some(pos) => {
                self.pos = pos;
                true
            }
            None => false,
        }
    }

    /// Move cursor to the start of the line.
    pub fn move_home(&mut self) -> bool {
        if self.pos > 0 {
            self.pos = 0;
            true
        } else {
            false
        }
    }

    /// Move cursor to the end of the line.
    pub fn move_end(&mut self) -> bool {
        if self.pos == self.buf.len() {
            false
        } else {
            self.pos = self.buf.len();
            true
        }
    }

    /// Delete the character at the right of the cursor without altering the
    /// cursor position. Basically this is what happens with the "Delete"
    /// keyboard key.
    /// Return the number of characters deleted.
    pub fn delete(&mut self, n: usize) -> Option<String> {
        match self.next_pos(n) {
            Some(pos) => {
                let start = self.pos;
                let chars = self
                    .drain(start..pos, Direction::Forward)
                    .collect::<String>();
                Some(chars)
            }
            None => None,
        }
    }

    /// Delete the character at the left of the cursor.
    /// Basically that is what happens with the "Backspace" keyboard key.
    pub fn backspace(&mut self, n: usize) -> bool {
        match self.prev_pos(n) {
            Some(pos) => {
                let end = self.pos;
                self.drain(pos..end, Direction::Backward);
                self.pos = pos;
                true
            }
            None => false,
        }
    }

    /// Kill the text from point to the end of the line.
    pub fn kill_line(&mut self) -> bool {
        if !self.buf.is_empty() && self.pos < self.buf.len() {
            let start = self.pos;
            let end = self.buf.len();
            self.drain(start..end, Direction::Forward);
            true
        } else {
            false
        }
    }

    /// Kill backward from point to the beginning of the line.
    pub fn discard_line(&mut self) -> bool {
        if self.pos > 0 && !self.buf.is_empty() {
            let end = self.pos;
            self.drain(0..end, Direction::Backward);
            self.pos = 0;
            true
        } else {
            false
        }
    }

    /// Replaces the content between [`start`..`end`] with `text`
    /// and positions the cursor to the end of text.
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        let start = range.start;
        self.buf.drain(range);
        if start == self.buf.len() {
            self.buf.push_str(text);
        } else {
            self.buf.insert_str(start, text);
        }
        self.pos = start + text.len();
    }

    /// Insert the `s`tring at the specified position.
    /// Return `true` if the text has been inserted at the end of the line.
    pub fn insert_str(&mut self, idx: usize, s: &str) -> bool {
        if idx == self.buf.len() {
            self.buf.push_str(s);
            true
        } else {
            self.buf.insert_str(idx, s);
            false
        }
    }

    /// Remove the specified `range` in the line.
    pub fn delete_range(&mut self, range: Range<usize>) {
        self.set_pos(range.start);
        self.drain(range, Direction::default());
    }

    fn drain(&mut self, range: Range<usize>, dir: Direction) -> Drain<'_> {
        self.buf.drain(range)
    }
}

impl Deref for LineBuffer {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

struct Renderer {
    line: LineBuffer,
    out: std::io::Stdout,
}

impl Renderer {
    pub fn new(out: std::io::Stdout) -> Self {
        Self {
            line: LineBuffer::with_capacity(512),
            out
        }
    }

    pub fn beep(&mut self) {
        self.out.write_all(b"\x07").unwrap();
        self.out.flush().unwrap();
    }

    pub fn move_rel(&mut self, bytes: &[u8]) {
        use std::convert::TryInto;
        let pos = i16::from_be_bytes(bytes[1..3].try_into().unwrap());
        if pos < 0 {
            write!(self.out, "{}", termion::cursor::Left(-pos as u16));
        } else {
            write!(self.out, "{}", termion::cursor::Right(pos as u16));
        }
        self.out.flush().unwrap();
    }

    pub fn put_chars(&mut self, chars: &[u8]) {
        self.line.move_end();

        for c in chars {
            if *c == b'\n' {
                self.out.write_all(b"\r\n").unwrap();
                self.line.clear();
            } else {
                assert!(self.line.insert(*c as char, 1).is_some());
                self.out.write_all(&[*c]).unwrap();
            }
            // move cursor
        }
        self.out.flush().unwrap();
    }

    pub fn insert_chars(&mut self, chars: &[u8]) {
        for c in chars {
            if *c == b'\n' {
                self.out.write_all(b"\r\n").unwrap();
                self.out.flush().unwrap();
                self.line.clear();
            } else {
                assert!(self.line.insert(*c as char, 1).is_some())
            }
            // move cursor
        }
        // TODO: need to redraw
    }

    pub fn delete_chars(&mut self, bytes: &[u8]) {
        use std::convert::TryInto;
        let n = i16::from_be_bytes(bytes[1..3].try_into().unwrap());
        if n > 0 { // delete forwards
            self.line.delete(n as usize);
        } else { // delete backwards
            self.line.backspace((-n) as usize);
        }
        // TODO: need to redraw
    }
}

#[cfg(test)]
mod test {
    use super::{ChangeListener, DeleteListener, Direction, LineBuffer, MAX_LINE};
    use crate::keymap::{At, CharSearch, Word};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn next_pos() {
        let s = LineBuffer::init("ö̲g̈", 0);
        assert_eq!(7, s.len());
        let pos = s.next_pos(1);
        assert_eq!(Some(4), pos);

        let s = LineBuffer::init("ö̲g̈", 4);
        let pos = s.next_pos(1);
        assert_eq!(Some(7), pos);
    }

    #[test]
    fn prev_pos() {
        let s = LineBuffer::init("ö̲g̈", 4);
        assert_eq!(7, s.len());
        let pos = s.prev_pos(1);
        assert_eq!(Some(0), pos);

        let s = LineBuffer::init("ö̲g̈", 7);
        let pos = s.prev_pos(1);
        assert_eq!(Some(4), pos);
    }

    #[test]
    fn insert() {
        let mut s = LineBuffer::with_capacity(MAX_LINE);
        let push = s.insert('α', 1).unwrap();
        assert_eq!("α", s.buf);
        assert_eq!(2, s.pos);
        assert_eq!(true, push);

        let push = s.insert('ß', 1).unwrap();
        assert_eq!("αß", s.buf);
        assert_eq!(4, s.pos);
        assert_eq!(true, push);

        s.pos = 0;
        let push = s.insert('γ', 1).unwrap();
        assert_eq!("γαß", s.buf);
        assert_eq!(2, s.pos);
        assert_eq!(false, push);
    }

    #[test]
    fn moves() {
        let mut s = LineBuffer::init("αß", 4);
        let ok = s.move_backward(1);
        assert_eq!("αß", s.buf);
        assert_eq!(2, s.pos);
        assert_eq!(true, ok);

        let ok = s.move_forward(1);
        assert_eq!("αß", s.buf);
        assert_eq!(4, s.pos);
        assert_eq!(true, ok);

        let ok = s.move_home();
        assert_eq!("αß", s.buf);
        assert_eq!(0, s.pos);
        assert_eq!(true, ok);

        let ok = s.move_end();
        assert_eq!("αß", s.buf);
        assert_eq!(4, s.pos);
        assert_eq!(true, ok);
    }

    #[test]
    fn move_grapheme() {
        let mut s = LineBuffer::init("ag̈", 4);
        assert_eq!(4, s.len());
        let ok = s.move_backward(1);
        assert_eq!(true, ok);
        assert_eq!(1, s.pos);

        let ok = s.move_forward(1);
        assert_eq!(true, ok);
        assert_eq!(4, s.pos);
    }

    #[test]
    fn delete() {
        let mut s = LineBuffer::init("αß", 2);
        let chars = s.delete(1);
        assert_eq!("α", s.buf);
        assert_eq!(2, s.pos);
        assert_eq!(Some("ß".to_owned()), chars);

        let ok = s.backspace(1);
        assert_eq!("", s.buf);
        assert_eq!(0, s.pos);
        assert_eq!(true, ok);
    }
}

async fn tty(id: ID, owner: PID, input: mpsc::UnboundedReceiver<Signal>) {
    use termion::terminal_size;
    let mut buf: [u8;1024] = [0;1024];
    // let mut stdin = tokio::io::stdin().compat();
    let mut stdin = tokio_stdin_stdout::stdin(0).compat();
    let mut input = input.fuse();

    let out = std::io::stdout();

    // for raw mode
    use termion::raw::IntoRawMode;
    let mut stdout = std::io::stdout().into_raw_mode().unwrap();

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
                                renderer.put_chars(&bytes[1..]);
                            }
                            // 1 MOVE
                            1 => {
                                renderer.move_rel(&bytes);
                            }
                            // 2 INSC
                            2 => {
                                renderer.insert_chars(&bytes);
                            }
                            // 3 DELC
                            3 => {
                                renderer.delete_chars(&bytes);
                            }
                            // BEEP
                            4 => {
                                renderer.beep();
                            }
                            // PUTC_SYNC
                            5 => {
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
                                println!("badarg yo");
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
        match await!(input.next()) {
            // * Port ! {Owner, {command, Data}}
            Some(Signal::Command(bytes)) => {
                stderr.write_all(&bytes).unwrap();
                stderr.flush().unwrap();
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
