use super::{Term, Variant, Header, WrongBoxError, BOXED_TUPLE, TryInto};
use std::ops::{Deref, DerefMut};
// use std::convert::TryFrom;

#[derive(Debug, Eq)]
#[repr(C)]
pub struct Tuple {
    /// Number of elements following the header.
    pub header: Header,
    pub len: u32,
}

const TUPLE_SIZE: usize = std::mem::size_of::<Tuple>();

impl Tuple {
    pub fn as_slice(&self) -> &[Term] {
        &self[..]
    }
}

impl Deref for Tuple {
    type Target = [Term];
    fn deref(&self) -> &[Term] {
        unsafe { ::std::slice::from_raw_parts((self as *const Tuple).add(TUPLE_SIZE) as *const Term, self.len as usize) }
    }
}

impl DerefMut for Tuple {
    fn deref_mut(&mut self) -> &mut [Term] {
        unsafe { ::std::slice::from_raw_parts_mut((self as *mut Tuple).add(TUPLE_SIZE) as *mut Term, self.len as usize) }
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<Tuple> for Term {
    type Error = WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&Tuple, WrongBoxError> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == BOXED_TUPLE {
                    return Ok(&*(ptr as *const Tuple))
                }
            }
        }
        Err(WrongBoxError)
    }
}
