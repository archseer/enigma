use crate::atom;
use crate::bitstring::{Binary, RcBinary};
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, TryFrom, Variant};
use crate::vm;
use std::fs;
use std::pin::Pin;
use std::sync::atomic::AtomicU32;

// it uses a queue (vecdeq? or just a slice of iovecs) of iovecs.
// vectored reads / writes on copying_read
// -- it uses an accumulator (ioqueue) just for the small writes it seems
// ACCUMULATOR_SIZE = 2 << 10 (2048 bytes)
// if accumulated_bytes + iovec >= ACCUMULATOR_SIZE
//   and iovec < ACCUMULATOR_SIZE / 2 --> then enqueue?
//
//   --------
//
//  buffer into accumulator up to a certain size, at which point accumulator is appended to queue
//  and replaced by a new empty one
//
//  VecDeque<Binary> + cursor for ioqueue --> TODO: should hold iovecs too
//  Vec<u8> for accumulator
//
//  TODO: a substring is technically a BytesMut into a string?

type IOQueue = BufDeque<Cursor<RcBinary>>;

pub struct Buffer {
    // accumulator: Vec<u8>,
    ioqueue: IOQueue,

    external_lock: AtomicU32,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            ioqueue: IOQueue::new(),
            external_lock: AtomicU32::new(0),
        }
    }

    pub fn size(&self) -> usize {
        self.ioqueue.remaining()
    }

    pub fn peek_head(&self) -> Result<Term, Exception> {
        unimplemented!()
    }

    pub fn copying_read(&self) -> usize {
        unimplemented!()
    }

    pub fn write(&mut self, iovec: RcBinary) {
        // TODO: combine_small_writes
        // TODO: enqueue accumulator if needed
        self.ioqueue.buffer(Cursor::new(iovec))
        // TODO: if tail isn't nil, write it too
    }

    pub fn skip(&mut self, block_size: usize) {
        self.ioqueue.advance(block_size)
    }

    pub fn find_byte_index(&self, block_size: usize) -> Option<usize> {
        unimplemented!()
    }

    pub fn try_lock(&self) -> bool {
        self.external_lock
            .compare_and_swap(0, 1, std::sync::atomic::Ordering::Acquire)
            == 0
    }

    pub fn unlock(&self) -> bool {
        self.external_lock
            .compare_and_swap(1, 0, std::sync::atomic::Ordering::Release)
            == 1
    }
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for Buffer {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_BUFFER {
                    return Ok(&(*(ptr as *const value::Boxed<Self>)).value);
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

pub mod bif {
    use super::*;
    use crate::bif::Result;

    pub fn new_0(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let heap = &process.context_mut().heap;
        let buf = Buffer::new();
        Ok(Term::buffer(heap, buf))
    }

    pub fn size_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let heap = &process.context_mut().heap;
        let buf = Buffer::try_from(&args[0])?;
        Ok(Term::uint64(heap, buf.size() as u64))
    }

    pub fn peek_head_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let buf = Buffer::try_from(&args[0])?;
        unimplemented!()
    }

    pub fn copying_read_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let buf = Buffer::try_from(&args[0])?;
        unimplemented!()
    }

    pub fn write_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        // let buf = Buffer::try_from(&args[0])?;
        // parse into iovec
        // buf.write(iovec);
        unimplemented!()
    }

    pub fn skip_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        // let buf = Buffer::try_from(&args[0])?;
        unimplemented!()
    }

    pub fn find_byte_index_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let buf = Buffer::try_from(&args[0])?;
        unimplemented!()
    }

    pub fn try_lock_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let buf = Buffer::try_from(&args[0])?;
        if !buf.try_lock() {
            return Ok(atom!(BUSY));
        }
        Ok(atom!(ACQUIRED))
    }

    pub fn unlock_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let buf = Buffer::try_from(&args[0])?;
        if !buf.unlock() {
            return Err(Exception::with_value(
                Reason::EXC_ERROR,
                atom!(LOCK_ORDER_VIOLATION),
            ));
        }
        Ok(atom!(OK))
    }
}

use std::collections::VecDeque;
use std::fmt;

use bytes::Buf;
use iovec::IoVec;

#[derive(Clone)]
pub struct Cursor<T> {
    bytes: T,
    pos: usize,
}

impl<T: AsRef<[u8]>> Cursor<T> {
    #[inline]
    pub(crate) fn new(bytes: T) -> Cursor<T> {
        Cursor { bytes, pos: 0 }
    }
}

impl Cursor<Vec<u8>> {
    fn reset(&mut self) {
        self.pos = 0;
        self.bytes.clear();
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for Cursor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Cursor")
            .field("pos", &self.pos)
            .field("len", &self.bytes.as_ref().len())
            .finish()
    }
}

impl<T: AsRef<[u8]>> Buf for Cursor<T> {
    #[inline]
    fn remaining(&self) -> usize {
        self.bytes.as_ref().len() - self.pos
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        &self.bytes.as_ref()[self.pos..]
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        debug_assert!(self.pos + cnt <= self.bytes.as_ref().len());
        self.pos += cnt;
    }
}

struct BufDeque<T> {
    bufs: VecDeque<T>,
}

impl<T> BufDeque<T> {
    fn new() -> BufDeque<T> {
        BufDeque {
            // TODO: use a cursor here instead?
            bufs: VecDeque::new(),
        }
    }

    pub(super) fn buffer<BB: Buf + Into<T>>(&mut self, mut buf: BB) {
        debug_assert!(buf.has_remaining());
        self.bufs.push_back(buf.into());
    }
}

impl<T: Buf> Buf for BufDeque<T> {
    #[inline]
    fn remaining(&self) -> usize {
        self.bufs.iter().map(bytes::buf::Buf::remaining).sum()
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        for buf in &self.bufs {
            return buf.bytes();
        }
        &[]
    }

    #[inline]
    fn advance(&mut self, mut cnt: usize) {
        while cnt > 0 {
            {
                let front = &mut self.bufs[0];
                let rem = front.remaining();
                if rem > cnt {
                    front.advance(cnt);
                    return;
                } else {
                    front.advance(rem);
                    cnt -= rem;
                }
            }
            self.bufs.pop_front();
        }
    }

    #[inline]
    fn bytes_vec<'t>(&'t self, dst: &mut [&'t IoVec]) -> usize {
        if dst.is_empty() {
            return 0;
        }
        let mut vecs = 0;
        for buf in &self.bufs {
            vecs += buf.bytes_vec(&mut dst[vecs..]);
            if vecs == dst.len() {
                break;
            }
        }
        vecs
    }
}
