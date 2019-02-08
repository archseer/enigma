use super::{Term, TryInto, Variant, WrongBoxError};
use core::marker::PhantomData;
use std::cmp::Ordering;
use std::ptr::NonNull;
use crate::immix::Heap;

#[derive(Debug, Eq)]
#[repr(C)]
pub struct Cons {
    pub head: Term,
    pub tail: Term,
}

unsafe impl Sync for Cons {}

impl PartialEq for Cons {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl PartialOrd for Cons {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cons {
    /// Lists are compared element by element.
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

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

impl Cons {
    // pub fn from_iter<I: IntoIterator<Item=Term>>(iter: I, heap: &Heap) -> Self
    //     where I::Item: DoubleEndedIterator {
    //     iter.rev().fold(Term::nil(), |res, val| value::cons(heap, val, res))
    // }

    // impl FromIterator<Term> for Cons { can't do this since we need heap

    pub fn from_iter<'a, I: IntoIterator<Item=&'a Term> + ExactSizeIterator>(iter: I, heap: &Heap) -> Term {
        let len = iter.len();
        let mut iter = iter.into_iter();
        let val = iter.next().unwrap();
        let c = heap.alloc(Cons {
            head: *val,
            tail: Term::nil(),
        });

        unsafe {
            (0..len - 1).fold(c as *mut Cons, |cons, _i| {
                let Cons { ref mut tail, .. } = *cons;
                let val = iter.next().unwrap();
                let new_cons = heap.alloc(Cons {
                    head: *val,
                    tail: Term::nil(),
                });
                let ptr = new_cons as *mut Cons;
                std::mem::replace(&mut *tail, Term::from(new_cons));
                ptr
            });
        }

        Term::from(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value;

    #[test]
    fn test_from_iter() {
        let heap = Heap::new();

        let tup = tup3!(&heap, Term::int(1), Term::int(2), Term::int(3));
        let t: &value::Tuple = tup.try_into().expect("wasn't a tuple");

        let res = Cons::from_iter(t.into_iter(), &heap);
        let cons: &Cons = res.try_into().expect("wasn't a cons");

        let mut iter = cons.iter();
        assert_eq!(Some(&Term::int(1)), iter.next());
        assert_eq!(Some(&Term::int(2)), iter.next());
        assert_eq!(Some(&Term::int(3)), iter.next());
        assert_eq!(None, iter.next());
    }
}
