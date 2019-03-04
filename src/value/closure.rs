use crate::module;
use crate::value::{Boxed, Term, TryFrom, Variant, WrongBoxError, BOXED_CLOSURE};

#[derive(Debug)]
#[repr(C)]
pub struct Closure {
    pub ptr: u32,
    pub mfa: module::MFA,
    pub binding: Option<Vec<Term>>,
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for Boxed<Closure> {
    type Error = WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_CLOSURE {
                    return Ok(&*(ptr as *const Boxed<Closure>));
                }
            }
        }
        Err(WrongBoxError)
    }
}
