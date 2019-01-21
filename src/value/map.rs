use super::{Header, Term, TryInto, Variant, WrongBoxError, BOXED_MAP};
use hamt_rs::HamtMap;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

pub type HAMT = HamtMap<Term, Term>;

#[derive(Eq, Clone)]
#[repr(C)]
pub struct Map {
    pub header: Header,
    pub map: HAMT,
}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<Map> for Term {
    type Error = WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&Map, WrongBoxError> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe {
                if *ptr == BOXED_MAP {
                    return Ok(&*(ptr as *const Map));
                }
            }
        }
        Err(WrongBoxError)
    }
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

impl PartialEq for Map {
    fn eq(&self, _other: &Self) -> bool {
        // Some(self.cmp(other))
        unimplemented!()
    }
}

impl PartialOrd for Map {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        // Some(self.cmp(other))
        unimplemented!()
    }
}

impl std::fmt::Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{{map}}")
    }
}
