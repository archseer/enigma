use super::{Header, Term, TryFrom, TryIntoMut, Variant, WrongBoxError, BOXED_TUPLE};
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
// use std::convert::TryFrom;

#[derive(Debug, Eq)]
#[repr(C)]
pub struct Tuple {
    /// Number of elements following the header.
    pub header: Header,
    pub len: u32,
}

impl Tuple {
    pub fn as_slice(&self) -> &[Term] {
        &self[..]
    }
}

impl Deref for Tuple {
    type Target = [Term];
    fn deref(&self) -> &[Term] {
        unsafe {
            ::std::slice::from_raw_parts(
                (self as *const Self).add(1) as *const Term,
                self.len as usize,
            )
        }
    }
}

impl DerefMut for Tuple {
    fn deref_mut(&mut self) -> &mut [Term] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                (self as *mut Self).add(1) as *mut Term,
                self.len as usize,
            )
        }
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        // fast path: different lengths means not equal
        if self.len != other.len {
            return false;
        }
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for Tuple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tuple {
    /// Tuples are ordered by size, two tuples with the same size are compared element by element.
    fn cmp(&self, other: &Self) -> Ordering {
        self.len
            .cmp(&other.len) // fast path: different lengths means not equal
            .then_with(|| self.as_slice().cmp(other.as_slice()))
    }
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for Tuple {
    type Error = WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_TUPLE {
                    return Ok(&*(ptr as *const Tuple));
                }
            }
        }
        Err(WrongBoxError)
    }
}

impl TryIntoMut<Tuple> for Term {
    type Error = WrongBoxError;

    #[inline]
    fn try_into_mut(&self) -> Result<&mut Tuple, WrongBoxError> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == BOXED_TUPLE {
                    return Ok(&mut *(ptr as *mut Tuple));
                }
            }
        }
        Err(WrongBoxError)
    }
}
