use crate::module;
use crate::value::{Boxed, Term, TryInto, Variant, WrongBoxError, BOXED_CLOSURE};

#[derive(Debug)]
#[repr(C)]
pub struct Closure {
    pub ptr: u32,
    pub mfa: module::MFA,
    pub binding: Option<Vec<Term>>,
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<Boxed<Closure>> for Term {
    type Error = WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&Boxed<Closure>, WrongBoxError> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == BOXED_CLOSURE {
                    return Ok(&*(ptr as *const Boxed<Closure>));
                }
            }
        }
        Err(WrongBoxError)
    }
}
