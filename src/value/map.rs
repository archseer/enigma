use super::{Boxed, CastFrom, CastFromMut, Term, Variant, WrongBoxError, BOXED_MAP};
// use hamt_rs::HamtMap;
use im::hashmap::HashMap;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

// TODO: evaluate using im-rs or https://github.com/orium/rpds HashTrieMap
pub type HAMT = HashMap<Term, Term>;

#[derive(Eq, Clone)]
pub struct Map(pub HAMT);

impl CastFrom<Term> for Map {
    type Error = WrongBoxError;

    #[inline]
    fn cast_from(value: &Term) -> Result<&Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_MAP {
                    return Ok(&(*(ptr as *const Boxed<Self>)).value);
                }
            }
        }
        Err(WrongBoxError)
    }
}

impl CastFromMut<Term> for Map {
    type Error = WrongBoxError;

    #[inline]
    fn cast_from_mut(value: &Term) -> Result<&mut Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_MAP {
                    return Ok(&mut (*(ptr as *const Boxed<Self> as *mut Boxed<Self>)).value);
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
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
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
