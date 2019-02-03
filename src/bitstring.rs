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

/// Binaries are bitstrings by default, byte aligned ones are binaries.
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

impl SubBinary {
    pub fn new(original: RcBinary, num_bits: usize, offset: usize) -> Self {
        SubBinary {
            original,
            size: byte_offset!(num_bits),
            bitsize: bit_offset!(num_bits),
            offset: byte_offset!(offset),
            bit_offset: bit_offset!(offset as u8), // TODO looks wrong
            is_writable: false,
        }
    }

    pub fn is_binary(&self) -> bool {
        self.bitsize != 0
    }
}

// TODO: let's use nom to handle offsets & matches, and keep a reference to the binary
#[derive(Debug)]
pub struct MatchBuffer {
    /// Original binary
    pub original: RcBinary,
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
            .unwrap()
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
            SubBinary::new(self.original.clone(), num_bits, self.offset),
        );
        self.offset += num_bits;
        Some(binary)
    }

    /// Copy up to 4 bytes into the supplied buffer.
    #[inline]
    fn align_utf8_bytes(&self, buf: *mut u8) {
        let bits = self.size - self.offset;

        let bits = match bits {
            0...7 => unreachable!(),
            8...15 => 8,
            16...23 => 24,
            24...31 => 24,
            _ => 32,
        };

        unsafe { copy_bits(self.original.data.as_ptr(), self.offset, 1, buf, 0, 1, bits) }
    }

    pub fn get_utf8(&mut self) -> Option<Term> {
        // Number of trailing bytes for each value of the first byte.
        const TRAILING_BYTES_FOR_UTF8: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 9, 9, 9, 9, 9, 9, 9, 9,
        ];

        // CHECK_MATCH_BUFFER(mb);
        let remaining_bits = self.size - self.offset;
        if remaining_bits < 8 {
            return None;
        }

        let mut tmp_buf: [u8; 4] = [0; 4];

        let pos: &[u8] = if bit_offset!(self.offset) == 0 {
            let offset = byte_offset!(self.offset);
            &self.original.data[offset..]
        } else {
            self.align_utf8_bytes(tmp_buf.as_mut_ptr());
            &tmp_buf[..]
        };

        let result = pos[0] as usize;
        let result = match TRAILING_BYTES_FOR_UTF8[result] {
            0 => {
                // One byte only
                self.offset += 8;
                result
            }
            1 => {
                // Two bytes
                if remaining_bits < 16 {
                    return None;
                }
                let a = pos[1] as usize;
                if (a & 0xC0) != 0x80 {
                    return None;
                }
                let result = (result << 6) + a - 0x00003080;
                self.offset += 16;
                result
            }
            2 => {
                // Three bytes
                if remaining_bits < 24 {
                    return None;
                }
                let a = pos[1] as usize;
                let b = pos[2] as usize;
                if (a & 0xC0) != 0x80 || (b & 0xC0) != 0x80 || (result == 0xE0 && a < 0xA0) {
                    return None;
                }
                let result = (((result << 6) + a) << 6) + b - 0x000E2080;
                if 0xD800 <= result && result <= 0xDFFF {
                    return None;
                }
                self.offset += 24;
                result
            }
            3 => {
                // Four bytes
                if remaining_bits < 32 {
                    return None;
                }
                let a = pos[1] as usize;
                let b = pos[2] as usize;
                let c = pos[3] as usize;
                if (a & 0xC0) != 0x80
                    || (b & 0xC0) != 0x80
                    || (c & 0xC0) != 0x80
                    || (result == 0xF0 && a < 0x90)
                {
                    return None;
                }
                let result = (((((result << 6) + a) << 6) + b) << 6) + c - 0x03C82080;
                if result > 0x10FFFF {
                    return None;
                }
                self.offset += 32;
                result
            }
            _ => unreachable!(),
        };

        Some(Term::int(result as i32)) // potentionally unsafe?
    }

    pub fn get_utf16(&mut self, flags: Flag) -> Option<Term> {
        let remaining_bits = self.size - self.offset;
        if remaining_bits < 16 {
            return None;
        }

        let mut tmp_buf: [u8; 4] = [0; 4];

        // CHECK_MATCH_BUFFER(mb);
        // Set up the pointer to the source bytes.
        let src: &[u8] = if bit_offset!(self.offset) == 0 {
            /* We can access the binary directly because the bytes are aligned. */
            let offset = byte_offset!(self.offset);
            &self.original.data[offset..]
        } else {
            /*
             * We must copy the data to a temporary buffer. If possible,
             * get 4 bytes, otherwise two bytes.
             */
            let n = if remaining_bits < 32 { 16 } else { 32 };
            unsafe {
                copy_bits(
                    self.original.data.as_ptr(),
                    self.offset,
                    1,
                    tmp_buf.as_mut_ptr(),
                    0,
                    1,
                    n,
                )
            };
            &tmp_buf[..]
        };

        // Get the first (and maybe only) 16-bit word. See if we are done.
        let w1: u16 = if flags.contains(Flag::BSF_LITTLE) {
            u16::from(src[0]) | (u16::from(src[1]) << 8)
        } else {
            (u16::from(src[0]) << 8) | u16::from(src[1])
        };
        if w1 < 0xD800 || w1 > 0xDFFF {
            self.offset += 16;
            return Some(Term::int(i32::from(w1)));
        } else if w1 > 0xDBFF {
            return None;
        }

        // Get the second 16-bit word and combine it with the first.
        let w2: u16 = if remaining_bits < 32 {
            return None;
        } else if flags.contains(Flag::BSF_LITTLE) {
            u16::from(src[2]) | (u16::from(src[3]) << 8)
        } else {
            (u16::from(src[2]) << 8) | u16::from(src[3])
        };
        if !(0xDC00 <= w2 && w2 <= 0xDFFF) {
            return None;
        }
        self.offset += 32;
        Some(Term::int(
            ((((w1 as u32 & 0x3FF) << 10) | (w2 as u32 & 0x3FF)) + 0x10000) as i32, // potentially unsafe
        ))
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

// https://docs.rs/bitstring/0.1.1/bitstring/bit_string/trait.BitString.html

/// Compare unaligned bitstrings. Much slower than memcmp, so only use when necessary.
// pub fn cmp_bits(a_ptr: *const u8, a_offs: usize, b_ptr: *const u8, b_offs: usize, size: usize) -> bool {
//     // byte a;
//     // byte b;
//     // byte a_bit;
//     // byte b_bit;
//     // Uint lshift;
//     // Uint rshift;
//     // int cmp;

//     assert!(a_offs < 8 && b_offs < 8);

//     if size == 0 {
//         return false;
//     }

//     if ((a_offs | b_offs | size) & 7) == 0 {
// 	let byte_size = size >> 3;
//         // compare as slices
// 	return sys_memcmp(a_ptr, b_ptr, byte_size);
//     }

//     // Compare bit by bit until a_ptr is aligned on byte boundary
//     a = *a_ptr++;
//     b = *b_ptr++;
//     if a_offs {
// 	loop {
// 	    let a_bit: u8 = get_bit(a, a_offs);
// 	    let b_bit: u8 = get_bit(b, b_offs);
// 	    if (cmp = (a_bit-b_bit)) != 0 {
// 		return cmp;
// 	    }
// 	    if --size == 0 {
// 		return false;
//             }

// 	    b_offs++;
// 	    if b_offs == 8 {
// 		b_offs = 0;
// 		b = *b_ptr++;
// 	    }
// 	    a_offs++;
// 	    if a_offs == 8 {
// 		a_offs = 0;
// 		a = *a_ptr++;
// 		break;
// 	    }
// 	}
//     }

//     // Compare byte by byte as long as at least 8 bits remain
//     if size >= 8 {
//         lshift = b_offs;
//         rshift = 8 - lshift;
//         loop {
//             let b_cmp: u8 = (b << lshift);
//             b = *b_ptr++;
//             b_cmp |= b >> rshift;
//             if (cmp = (a - b_cmp)) != 0 {
//                 return cmp;
//             }
//             size -= 8;
// 	    if size < 8 {
// 		break;
//             }
//             a = *a_ptr++;
//         }

// 	if size == 0 {
// 	    return 0;
//         }
// 	a = *a_ptr++;
//     }

//     // Compare the remaining bits bit by bit
//     if size > 0 {
//         loop {
//             let a_bit: u8 = get_bit(a, a_offs);
//             let b_bit: u8 = get_bit(b, b_offs);
//             if (cmp = (a_bit-b_bit)) != 0 {
//                 return cmp;
//             }
//             if --size == 0 {
//                 return false;
//             }

//             a_offs++;
// 	    assert!(a_offs < 8);

//             b_offs++;
//             if b_offs == 8 {
//                 b_offs = 0;
//                 b = *b_ptr++;
//             }
//         }
//     }

//     false
// }

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
