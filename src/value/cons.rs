use super::{Term, Variant, WrongBoxError, TryInto};
use std::ptr::NonNull;
use core::marker::PhantomData;

#[derive(Debug)]
#[repr(C)]
pub struct Cons {
    pub head: Term,
    pub tail: Term,
}

unsafe impl Sync for Cons {}

// TODO: to be TryFrom once rust stabilizes the trait
impl TryInto<Cons> for Term {
    type Error = WrongBoxError;

    #[inline]
    fn try_into(&self) -> Result<&Cons, WrongBoxError> {
        if let Variant::Cons(ptr) = self.into_variant() {
            unsafe { return Ok(&*(ptr as *const Cons)) }
        }
        Err(WrongBoxError)
    }
}

pub struct Iter<'a> {
    head: Option<NonNull<Cons>>,
    //len: usize,
    marker: PhantomData<&'a Cons>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Term;

    #[inline]
    fn next(&mut self) -> Option<&'a Term> {
        self.head.map(|node| unsafe {
            // Need an unbound lifetime to get 'a
            let node = &*node.as_ptr();
            if let Ok(cons) = node.tail.try_into() {
                self.head = Some(NonNull::new_unchecked(cons as *const Cons as *mut Cons));
            } else {
                // TODO match badly formed lists
                self.head = None;
            }
            &node.head
        })
    }
}

impl Cons {
    pub fn iter(&self) -> Iter {
        Iter {
            head: unsafe { Some(NonNull::new_unchecked(self as *const Cons as *mut Cons)) },
            //len: self.len,
            marker: PhantomData,
        }
    }
}

impl<'a> IntoIterator for &'a Cons {
    type Item = &'a Term;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

