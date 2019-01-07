use crate::value::Value;
use parking_lot::Mutex;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug)]
pub struct Binary {
    pub flags: AtomicUsize,

    /// The actual underlying bits. Not wrapped with a lock so we can hash
    pub data: Vec<u8>,

    /// Used for synchronizing writes
    pub write_lock: Mutex<()>,
}

impl Binary {
    pub fn new() -> Self {
        Binary {
            flags: AtomicUsize::new(0),
            data: Vec::new(),
            write_lock: Mutex::new(()),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Binary {
            flags: AtomicUsize::new(0),
            data: Vec::with_capacity(cap),
            write_lock: Mutex::new(()),
        }
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Binary {
            flags: AtomicUsize::new(0),
            data: vec,
            write_lock: Mutex::new(()),
        }
    }
}

impl Ord for Binary {
    fn cmp(&self, other: &Binary) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for Binary {
    fn partial_cmp(&self, other: &Binary) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Binary {
    fn eq(&self, other: &Binary) -> bool {
        self.data == other.data
    }
}

impl Eq for Binary {}

impl Hash for Binary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

pub struct SubBinary {
    // TODO: wrap into value
    /// Binary size in bytes
    size: usize,
    /// Offset into binary
    offset: usize,
    /// Bit size
    bitsize: usize,
    /// Bit offset
    bit_offset: u8,
    /// Is the underlying binary writable?
    is_writable: bool,
    /// Original binary (refc or heap)
    original: Value,
}

// TODO: let's use nom to handle offsets & matches, and keep a reference to the binary
pub struct MatchBuffer {
    /// Original binary
    original: Value,
    /// Current position in binary
    base: usize,
    /// Offset in bits
    offset: u8,
    /// Size of binary in bits
    size: usize,
}

pub struct MatchState<'a> {
    // TODO: wrap into value
    mb: MatchBuffer,
    /// Saved offsets, only valid for contexts created through bs_start_match2.
    saved_offsets: &'a [Value],
}

bitflags! {
    /// Flags for bs_get_* / bs_put_* / bs_init* instructions.
    pub struct Flag: u8 {
        /// Field is guaranteed to be byte-aligned. TODO: seems unused?
        const BSF_ALIGNED = 1;
        /// Field is little-endian (otherwise big-endian).
        const BSF_LITTLE = 2;
        /// Field is signed (otherwise unsigned).
        const BSF_SIGNED = 4;
        /// Size in bs_init is exact. TODO: seems unused?
        const BSF_EXACT = 8;
        /// Native endian.
        const BSF_NATIVE = 16;
    }
}
