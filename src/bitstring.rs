use crate::process::RcProcess;
use crate::servo_arc::Arc;
use crate::value::{self, Term, TryInto};
use parking_lot::Mutex;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// make_mask(n) constructs a mask with n bits.
/// Example: make_mask!(3) returns the binary number 00000111.
macro_rules! make_mask {
    ($n:expr) => {
        ((1 as u8) << $n) - 1
    };
}

/// mask_bits assigns src to dst, but preserves the dst bits outside the mask.
macro_rules! mask_bits {
    ($src:expr, $dst:expr, $mask:expr) => {
        ($src & $mask) | ($dst & !$mask)
    };
}

/// nbytes!(x) returns the number of bytes needed to store x bits.
macro_rules! nbytes {
    ($x:expr) => {
        ($x as usize + 7 as usize) >> 3
    };
}

macro_rules! byte_offset {
    ($ofs:expr) => {
        $ofs as usize >> 3
    };
}

macro_rules! bit_offset {
    ($ofs:expr) => {
        $ofs & 7
    };
}

#[derive(Debug)]
pub struct Binary {
    pub flags: AtomicUsize,

    /// The actual underlying bits. Not wrapped with a lock so we can hash
    pub data: Vec<u8>,

    /// Used for synchronizing writes
    pub write_lock: Mutex<()>,
}

pub type RcBinary = Arc<Binary>;

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

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<value::Boxed<RcBinary>> for Term {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&value::Boxed<RcBinary>, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == value::BOXED_BINARY {
                    return Ok(&*(ptr as *const value::Boxed<RcBinary>));
                }
            }
        }
        Err(value::WrongBoxError)
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
    original: RcBinary,
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<value::Boxed<SubBinary>> for Term {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&value::Boxed<SubBinary>, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == value::BOXED_SUBBINARY {
                    return Ok(&*(ptr as *const value::Boxed<SubBinary>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

// TODO: let's use nom to handle offsets & matches, and keep a reference to the binary
#[derive(Debug)]
pub struct MatchBuffer {
    /// Original binary
    original: RcBinary,
    /// Current position in binary
    // base: usize, // TODO: actually a ptr?
    /// Offset in bits
    pub offset: usize, // TODO: maybe don't make these pub, add a remainder method
    /// Size of binary in bits
    pub size: usize,
}

pub struct MatchState {
    // TODO: wrap into value
    pub mb: MatchBuffer,
    /// Saved offsets, only valid for contexts created through bs_start_match2.
    pub saved_offsets: Vec<usize>,
} // TODO: Dump start_match_2 support. use MatchBuffer directly

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<value::Boxed<MatchState>> for Term {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&value::Boxed<MatchState>, value::WrongBoxError> {
        if let value::Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == value::BOXED_MATCHSTATE {
                    return Ok(&*(ptr as *const value::Boxed<MatchState>));
                }
            }
        }
        Err(value::WrongBoxError)
    }
}

bitflags! {
    /// Flags for bs_get_* / bs_put_* / bs_init* instructions.
    pub struct Flag: u8 {
        const BSF_NONE = 0;
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

#[cfg(target_endian = "little")]
const NATIVE_ENDIAN: Flag = Flag::BSF_LITTLE;

#[cfg(target_endian = "big")]
const NATIVE_ENDIAN: Flag = Flag::BSF_NONE;

macro_rules! bit_is_machine_endian {
    ($x:expr) => {
        $x & Flag::BSF_LITTLE == NATIVE_ENDIAN
    };
}

#[cfg(target_endian = "little")]
macro_rules! native_endian {
    ($x:expr) => {
        if $x.contains(Flag::BSF_NATIVE) {
            $x.remove(Flag::BSF_NATIVE);
            $x.insert(Flag::BSF_LITTLE);
        }
    };
}

#[cfg(target_endian = "big")]
macro_rules! native_endian {
        if $x.contains(Flag::BSF_NATIVE) {
            $x.remove(Flag::BSF_NATIVE);
            $x.remove(Flag::BSF_LITTLE);
        }
}

macro_rules! binary_size {
    ($str:expr) => {
        $str.get_boxed_value::<value::Boxed<RcBinary>>()
            .value
            .data
            .len()
    };
}

pub fn start_match_2(process: &RcProcess, binary: Term, max: u32) -> Term {
    assert!(binary.is_binary());

    // TODO: BEAM allocates size on all binary types right after the header so we can grab it
    // without needing the binary subtype.
    let total_bin_size = binary_size!(binary);

    if (total_bin_size >> (8 * std::mem::size_of::<usize>() - 3)) != 0 {
        // Uint => maybe u8??
        return Term::none();
    }

    //     NeededSize = ERL_BIN_MATCHSTATE_SIZE(Max);
    //     hp = HeapOnlyAlloc(p, NeededSize);
    //     ms = (ErlBinMatchState *) hp;

    // let (orig, offs, bitoffs, bitsize) = if let SubBinary{original, offset: offs, bit_offset: bitoffs, bitsize} {
    //     (original, offs, bifoffs, bitsize)
    // } else {
    //     // TODO: extract RcBinary
    //     (original, 0, 0, 0)
    // }

    let original: RcBinary = match binary.try_into() {
        Ok(value::Boxed {
            value,
            header: value::BOXED_BINARY,
        }) => {
            let value: &RcBinary = value;
            value.clone()
        }
        _ => unreachable!(),
    };

    let (original, offs, bitoffs, bitsize) = (original, 0, 0, 0);

    //     pb = (ProcBin *) boxed_val(Orig);
    //     if (pb->thing_word == HEADER_PROC_BIN && pb->flags != 0) {
    // 	erts_emasculate_writable_binary(pb);
    //     }

    // ms->thing_word = HEADER_BIN_MATCHSTATE(Max); TODO use max once we have heap vecs

    let offset = 8 * offs + bitoffs;

    Term::matchstate(
        &process.context_mut().heap,
        MatchState {
            mb: MatchBuffer {
                original,
                //base: binary_bytes(original),
                offset,
                size: total_bin_size * 8 + offset + bitsize,
            },
            saved_offsets: vec![offset],
        },
    )
}

// #ifdef DEBUG
// # define CHECK_MATCH_BUFFER(MB) check_match_buffer(MB)

// static void check_match_buffer(ErlBinMatchBuffer* mb)
// {
//     Eterm realbin;
//     Uint byteoffs;
//     byte* bytes, bitoffs, bitsz;
//     ProcBin* pb;
//     ERTS_GET_REAL_BIN(mb->orig, realbin, byteoffs, bitoffs, bitsz);
//     bytes = binary_bytes(realbin) + byteoffs;
//     ERTS_ASSERT(mb->base >= bytes && mb->base <= (bytes + binary_size(mb->orig)));
//     pb = (ProcBin *) boxed_val(realbin);
//     if (pb->thing_word == HEADER_PROC_BIN)
//         ERTS_ASSERT(pb->flags == 0);
// }
// #else
// # define CHECK_MATCH_BUFFER(MB)
// #endif

impl MatchBuffer {
    pub fn get_float(
        &mut self,
        _process: &RcProcess,
        num_bits: usize,
        mut flags: Flag,
    ) -> Option<Term> {
        let mut fl32: f32 = 0.0;
        let mut fl64: f64 = 0.0;

        // TODO: preprocess flags for native endian in loader(remove native_endian and set bsf_little off or on)
        native_endian!(flags);

        // CHECK_MATCH_BUFFER(mb);

        if num_bits == 0 {
            return Some(Term::from(0.0));
        }

        if (self.size - self.offset) < num_bits {
            // Asked for too many bits.
            return None;
        }

        let fptr: *mut u8 = match num_bits {
            32 => &mut fl32 as *mut f32 as *mut u8,
            64 => &mut fl64 as *mut f64 as *mut u8,
            _ => return None,
        };

        if bit_is_machine_endian!(flags) {
            unsafe {
                copy_bits(
                    self.original.data.as_ptr(),
                    self.offset,
                    1,
                    fptr,
                    0,
                    1,
                    num_bits,
                )
            };
        } else {
            unsafe {
                copy_bits(
                    self.original.data.as_ptr(),
                    self.offset,
                    1,
                    fptr.add(nbytes!(num_bits) - 1),
                    0,
                    -1,
                    num_bits,
                )
            };
        }

        let f = if num_bits == 32 {
            if !fl32.is_finite() {
                return None;
            }
            Term::from(fl32 as f64)
        } else {
            //   #ifdef DOUBLE_MIDDLE_ENDIAN
            //   FloatDef ftmp;
            //   ftmp.fd = f64;
            //   f.fw[0] = ftmp.fw[1];
            //   f.fw[1] = ftmp.fw[0];
            //   ERTS_FP_ERROR_THOROUGH(p, f.fd, return THE_NON_VALUE);
            //   #else
            //   ...
            //   #endif
            if !fl64.is_finite() {
                return None;
            }
            Term::from(fl64)
        };
        self.offset += num_bits;
        Some(f)
    }

    pub fn get_binary(
        &mut self,
        process: &RcProcess,
        num_bits: usize,
        mut flags: Flag,
    ) -> Option<Term> {
        // CHECK_MATCH_BUFFER(mb);

        // Reduce the use of none by using Result.
        if (self.size - self.offset) < num_bits {
            // Asked for too many bits.
            return None;
        }

        // From now on, we can't fail.

        let binary = Term::subbinary(
            &process.context_mut().heap,
            SubBinary {
                original: self.original.clone(),
                size: byte_offset!(num_bits),
                bitsize: bit_offset!(num_bits),
                offset: byte_offset!(self.offset),
                bit_offset: bit_offset!(self.offset as u8), // TODO looks wrong
                is_writable: false,
            },
        );
        self.offset += num_bits;
        Some(binary)
    }
}

// Stores data on the process heap. Small, but expensive to copy.
// HeapBin(len + ptr)
// Stores data off the process heap, in an Arc<>. Cheap to copy around.
// RefBin(Arc<String/Vec<u8?>>)
// ^^ start with just RefBin since Rust already will do the String management for us
// SubBin(len (original?), offset, bitsize,bitoffset,is_writable, orig_ptr -> Bin/RefBin)

// consider using an Arc<RwLock<>> to make the inner string mutable? is the overhead worth it?
// data is always append only, so maybe have an atomic bool for the writable bit and keep the
// normal structure lockless.

// bitstring is the base model, binary is an 8-bit aligned bitstring
// https://www.reddit.com/r/rust/comments/2d7rrj/bit_level_pattern_matching/
// https://docs.rs/bitstring/0.1.1/bitstring/bit_string/trait.BitString.html

/// The basic bit copy operation. Copies n bits from the source buffer to
/// the destination buffer. Depending on the directions, it can reverse the
/// copied bits.
pub unsafe fn copy_bits(
    mut src: *const u8, // Base pointer to source.
    soffs: usize,       // Bit offset for source relative to src.
    sdir: isize,        // Direction: 1 (forward) or -1 (backward).
    mut dst: *mut u8,   // Base pointer to destination.
    doffs: usize,       // Bit offset for destination relative to dst.
    ddir: isize,        // Direction: 1 (forward) or -1 (backward).
    n: usize,
) // Number of bits to copy.
{
    if n == 0 {
        return;
    }

    src = src.offset(sdir * byte_offset!(soffs) as isize);
    dst = dst.offset(ddir * byte_offset!(doffs) as isize);
    let soffs = bit_offset!(soffs);
    let doffs = bit_offset!(doffs);
    let deoffs = bit_offset!(doffs + n);
    let mut lmask = if doffs > 0 { make_mask!(8 - doffs) } else { 0 };
    let rmask = if deoffs > 0 {
        (make_mask!(deoffs)) << (8 - deoffs)
    } else {
        0
    };

    // Take care of the case that all bits are in the same byte.

    if doffs + n < 8 {
        // All bits are in the same byte
        lmask = if (lmask & rmask) > 0 {
            lmask & rmask
        } else {
            lmask | rmask
        };

        if soffs == doffs {
            *dst = mask_bits!(*src, *dst, lmask);
        } else if soffs > doffs {
            let mut bits: u8 = *src << (soffs - doffs); // TODO: is it u8
            if soffs + n > 8 {
                src = src.offset(sdir);
                bits |= *src >> (8 - (soffs - doffs));
            }
            *dst = mask_bits!(bits, *dst, lmask);
        } else {
            *dst = mask_bits!((*src >> (doffs - soffs)), *dst, lmask);
        }
        return; // We are done!
    }

    // At this point, we know that the bits are in 2 or more bytes.

    let mut count = (if lmask > 0 { n - (8 - doffs) } else { n }) >> 3;

    if soffs == doffs {
        /*
         * The bits are aligned in the same way. We can just copy the bytes
         * (except for the first and last bytes). Note that the directions
         * might be different, so we can't just use memcpy().
         */

        if lmask > 0 {
            *dst = mask_bits!(*src, *dst, lmask);
            dst = dst.offset(ddir);
            src = src.offset(sdir);
        }

        while count > 0 {
            count -= 1;
            *dst = *src;
            dst = dst.offset(ddir);
            src = src.offset(sdir);
        }

        if rmask > 0 {
            *dst = mask_bits!(*src, *dst, rmask);
        }
    } else {
        let mut bits: u8 = 0;
        let mut bits1: u8 = 0;
        let mut rshift = 0;
        let mut lshift = 0;

        // The tricky case. The bits must be shifted into position.

        if soffs > doffs {
            lshift = soffs - doffs;
            rshift = 8 - lshift;
            bits = *src;
            if soffs + n > 8 {
                src = src.offset(sdir);
            }
        } else {
            rshift = doffs - soffs;
            lshift = 8 - rshift;
            bits = 0;
        }

        if lmask > 0 {
            bits1 = bits << lshift;
            bits = *src;
            src = src.offset(sdir);
            bits1 |= bits >> rshift;
            *dst = mask_bits!(bits1, *dst, lmask);
            dst = dst.offset(ddir);
        }

        while count > 0 {
            count -= 1;
            bits1 = bits << lshift;
            bits = *src;
            src = src.offset(sdir);
            *dst = bits1 | (bits >> rshift);
            dst = dst.offset(ddir);
        }

        if rmask > 0 {
            bits1 = bits << lshift;
            if (rmask << rshift) & 0xff > 0 {
                bits = *src;
                bits1 |= bits >> rshift;
            }
            *dst = mask_bits!(bits1, *dst, rmask);
        }
    }
}
