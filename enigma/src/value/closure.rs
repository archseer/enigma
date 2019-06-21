use crate::module;
use crate::value::{Boxed, CastFrom, Term, Variant, WrongBoxError, BOXED_CLOSURE};

#[derive(Debug)]
#[repr(C)]
pub struct Closure {
    pub ptr: u32,
    pub mfa: module::MFA,
    pub binding: Option<Vec<Term>>,
}

impl CastFrom<Term> for Closure {
    type Error = WrongBoxError;

    #[inline]
    fn cast_from(value: &Term) -> Result<&Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_CLOSURE {
                    return Ok(&(*(ptr as *const Boxed<Closure>)).value);
                }
            }
        }
        Err(WrongBoxError)
    }
}
