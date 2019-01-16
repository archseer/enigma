use crate::atom;
use crate::bitstring;
use crate::exception;
use crate::immix::Heap;
use crate::module;
use crate::nanbox::TypedNanBox;
use crate::process::{self, InstrPtr};
use crate::servo_arc::Arc;
use allocator_api::Layout;
use core::marker::PhantomData;
use hamt_rs::HamtMap;
use num::bigint::BigInt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
// annoying: we have to wrap Floats to be able to define hash
pub struct Float(pub f64);
impl Eq for Float {}
impl Hash for Float {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

pub type HAMT = HamtMap<Value, Value>;

#[derive(PartialEq, Eq, Clone)]
pub struct Map(pub Arc<HamtMap<Value, Value>>);

impl Hash for Map {
    fn hash<H: Hasher>(&self, _state: &mut H) {
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

// nanbox as:
// 1 float
// 2 nil
// 3 int32
// 4 atom -> could we represent nil as atom 0?
// 5 port --> or maybe dump port for now
// 6 pid
// 7 box ptr (list, tuple, map, binary, ref (it's 96 bits), bigint, closure, cp/catch/stacktrace)
// cons has a special type on BEAM
// 8 the_non_val?? --> maybe we could keep a constant NaN for that
//
// box data should have a header followed by value
//
// what about catch which is direct immediate in erlang, also CP is 00 on stack and means header on
// heap.

/// A term is a nanboxed compact representation of a value in 64 bits. It can either be immediate,
/// in which case it embeds the data, or a boxed pointer, that points to more data.
#[derive(Debug)]
pub struct Term {
    value: TypedNanBox<Variant>,
}

#[derive(Clone, Debug)]
pub enum Variant {
    Float(f64),
    Nil,
    Integer(i32),
    Atom(u32),
    Port(u32),
    Pid(process::PID),
    Pointer(*const Value), // list, tuple, map, bunary, ref
    /// An internal placeholder signifying "THE_NON_VALUE".
    None,
}

impl From<f64> for Term {
    fn from(value: f64) -> Term {
        Term::from(Variant::Float(value))
    }
}

impl From<i32> for Term {
    fn from(value: i32) -> Term {
        Term::from(Variant::Integer(value))
    }
}

impl From<process::PID> for Term {
    fn from(value: process::PID) -> Term {
        Term::from(Variant::Pid(value))
    }
}

impl From<*const Value> for Term {
    fn from(value: *const Value) -> Term {
        Term::from(Variant::Pointer(value))
    }
}

impl From<Variant> for Term {
    fn from(value: Variant) -> Term {
        unsafe {
            match value {
                Variant::Float(value) => Term {
                    value: TypedNanBox::new(0, value),
                },
                Variant::Nil => Term {
                    value: TypedNanBox::new(1, 0),
                },
                Variant::Integer(value) => Term {
                    value: TypedNanBox::new(2, value),
                },
                Variant::Atom(value) => Term {
                    value: TypedNanBox::new(3, value),
                },
                Variant::Port(value) => Term {
                    value: TypedNanBox::new(4, value),
                },
                Variant::Pid(value) => Term {
                    value: TypedNanBox::new(5, value),
                },
                Variant::Pointer(value) => Term {
                    value: TypedNanBox::new(6, value),
                },
                Variant::None => Term {
                    value: TypedNanBox::new(7, 0),
                },
            }
        }
    }
}

impl From<Term> for Variant {
    fn from(value: Term) -> Variant {
        value.value.into()
    }
}

impl From<TypedNanBox<Variant>> for Variant {
    fn from(value: TypedNanBox<Variant>) -> Variant {
        #[allow(unused_assignments)]
        unsafe {
            match value.tag() {
                0 => Variant::Float(value.unpack()),
                1 => Variant::Nil,
                2 => Variant::Integer(value.unpack()),
                3 => Variant::Atom(value.unpack()),
                4 => Variant::Port(value.unpack()),
                5 => Variant::Pid(value.unpack()),
                6 => Variant::Pointer(value.unpack()),
                7 => Variant::None,
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl Term {
    pub fn into_variant(self) -> Variant {
        self.into()
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Hash)]
pub enum Value {
    // TODO: figure out if these should be an enum or a header + ptr (typed ptr)
    /// Boxed values (on heap)
    Ref(u32),
    List(*const self::Cons),
    Tuple(*const self::Tuple),
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    Binary(Arc<bitstring::Binary>),
    Map(self::Map),
    BigInt(Box<BigInt>), // Arc<BigInt>
    Closure(*const self::Closure),

    /// Special emulator values

    /// continuation pointer
    CP(Box<Option<InstrPtr>>),
    /// Catch context
    Catch(Box<InstrPtr>),
    /// Stack trace
    StackTrace(*const exception::StackTrace),
}

#[derive(Debug)]
pub struct Cons {
    pub head: Value,
    pub tail: Value,
}

pub struct Iter<'a> {
    head: Option<NonNull<Cons>>,
    //len: usize,
    marker: PhantomData<&'a Cons>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Value;

    #[inline]
    fn next(&mut self) -> Option<&'a Value> {
        self.head.map(|node| unsafe {
            // Need an unbound lifetime to get 'a
            let node = &*node.as_ptr();
            if let Value::List(cons) = node.tail {
                self.head = Some(NonNull::new_unchecked(cons as *mut Cons));
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
    type Item = &'a Value;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Tuple {
    /// Number of elements following the header.
    pub len: u32,
    pub ptr: NonNull<Value>,
}

impl Tuple {
    pub fn as_slice(&self) -> &[Value] {
        &self[..]
    }
}

#[derive(Debug)]
pub struct Closure {
    pub mfa: module::MFA,
    pub ptr: u32,
    pub binding: Option<Vec<Value>>,
}

impl Deref for Tuple {
    type Target = [Value];
    fn deref(&self) -> &[Value] {
        unsafe { ::std::slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }
}

impl DerefMut for Tuple {
    fn deref_mut(&mut self) -> &mut [Value] {
        unsafe { ::std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len as usize) }
    }
}

unsafe impl Sync for Value {}
unsafe impl Send for Value {}

unsafe impl Sync for Cons {}

// term order:
// number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string
pub enum Type {
    Number,
    Atom,
    Ref,
    Closure,
    Port,
    Pid,
    Tuple,
    Map,
    Nil,
    List,
    Binary,
}

impl Term {
    #[inline]
    pub fn is_none(&self) -> bool {
        self.value.tag() == 7
    }

    pub fn is_float(&self) -> bool {
        self.value.tag() == 0
    }

    pub fn is_nil(&self) -> bool {
        self.value.tag() == 1
    }

    pub fn is_smallint(&self) -> bool {
        self.value.tag() == 2
    }

    pub fn is_atom(&self) -> bool {
        self.value.tag() == 3
    }

    pub fn is_port(&self) -> bool {
        self.value.tag() == 4
    }

    pub fn is_pid(&self) -> bool {
        self.value.tag() == 5
    }

    pub fn is_pointer(&self) -> bool {
        self.value.tag() == 6
    }

    pub fn get_type(&self) -> Type {
        match self.value.tag() {
            0 => Type::Number,
            1 => Type::Nil,
            2 => Type::Number,
            3 => Type::Atom,
            4 => Type::Port,
            5 => Type::Pid,
            6 => {
                match self.get_boxed_value() {
                    Value::Ref(..) => Type::Ref,
                    Value::List(..) => Type::List,
                    Value::Tuple(..) => Type::Tuple,
                    Value::Binary(..) => Type::Binary,
                    Value::Map(..) => Type::Map,
                    Value::BigInt(..) => Type::Number,
                    Value::Closure(..) => Type::Closure,
                    _ => unimplemented!()
                }
            },
            _ => unreachable!()
        }
    }

    pub fn get_boxed_value(&self) -> &Value {
       if let Variant::Pointer(ptr) = self.into_variant() {
           unsafe {
               return &*ptr
           }
       }
       panic!("Not a boxed type!")
    }

    pub fn get_boxed_value_mut(&self) -> &mut Value {
       if let Variant::Pointer(ptr) = self.into_variant() {
           unsafe {
               return &mut *ptr
           }
       }
       panic!("Not a boxed type!")
    }
}

impl Variant {
    #[inline]
    pub fn is_none(&self) -> bool {
        match *self {
            Variant::None => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_smallint(&self) -> bool {
        match *self {
            Variant::Integer(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        match *self {
            Variant::BigInt(..) => true,
            Variant::Integer(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        match *self {
            Variant::Float(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        match *self {
            Variant::Float(..) => true,
            Variant::BigInt(..) => true,
            Variant::Integer(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_atom(&self) -> bool {
        match *self {
            Variant::Atom(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_pid(&self) -> bool {
        match *self {
            Variant::Pid(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_ref(&self) -> bool {
        match *self {
            Variant::Ref(..) => true,
            _ => false,
        }
    }

    pub fn is_port(&self) -> bool {
        match *self {
            Variant::Port(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        match *self {
            Variant::Nil => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match *self {
            Variant::Binary(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        match *self {
            Variant::List { .. } => true,
            Variant::Nil => true, // apparently also valid
            _ => false,
        }
    }

    #[inline]
    pub fn is_non_empty_list(&self) -> bool {
        match *self {
            Variant::List(ptr) => unsafe { !(*ptr).head.is_nil() },
            _ => false,
        }
    }

    #[inline]
    pub fn is_tuple(&self) -> bool {
        match *self {
            Variant::Tuple(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        match *self {
            Variant::Closure(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        match *self {
            Variant::Atom(atom::TRUE) | Variant::Atom(atom::FALSE) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_map(&self) -> bool {
        match *self {
            Variant::Map(..) => true,
            _ => false,
        }
    }

    pub fn is_cp(&self) -> bool {
        match *self {
            Variant::CP(..) => true,
            _ => false,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match *self {
            Variant::Atom(i) => i,
            Variant::Pid(i) => i,
            Variant::Integer(i) => i as u32,
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            return Variant::Atom(atom::TRUE);
        }
        Variant::Atom(atom::FALSE)
    }

    pub fn erl_eq(&self, other: &Variant) -> bool {
        match (self, other) {
            (Variant::Nil, Variant::Nil) => true,
            (Variant::Integer(i1), Variant::Integer(i2)) => i1 == i2,
            (Variant::Float(f1), Variant::Float(f2)) => f1 == f2,
            (Variant::BigInt(b1), Variant::BigInt(b2)) => b1 == b2,
            (Variant::Integer(_), Variant::Float(_)) => unimplemented!(),
            (Variant::Float(_), Variant::Integer(_)) => unimplemented!(),

            (Variant::Atom(a1), Variant::Atom(a2)) => a1 == a2,
            (Variant::Pid(p1), Variant::Pid(p2)) => p1 == p2,
            (Variant::Port(p1), Variant::Port(p2)) => p1 == p2,
            (Variant::Ref(r1), Variant::Ref(r2)) => r1 == r2,

            (Variant::List(l1), Variant::List(l2)) => unsafe {
                (**l1)
                    .iter()
                    .zip((**l2).iter())
                    .all(|(e1, e2)| e1.erl_eq(e2))
            },
            (Variant::Tuple(v1), Variant::Tuple(v2)) => unsafe {
                if (**v1).len == (**v2).len {
                    (**v1)
                        .as_slice()
                        .iter()
                        .zip((**v2).as_slice())
                        .all(|(e1, e2)| e1.erl_eq(e2))
                } else {
                    false
                }
            },
            (Variant::Binary(b1), Variant::Binary(b2)) => b1 == b2,
            (Variant::Literal(l1), Variant::Literal(l2)) => l1 == l2,
            (Variant::Closure(c1), Variant::Closure(c2)) => unsafe { (**c1).mfa == (**c2).mfa },
            (Variant::CP(l1), Variant::CP(l2)) => l1 == l2,
            (Variant::Catch(l1), Variant::Catch(l2)) => l1 == l2,
            (Value::Closure { .. }, _) => unreachable!(), // There should never happen
            (_, Value::Closure { .. }) => unreachable!(),
            (Value::StackTrace(..), _) => unreachable!(),
            (_, Value::StackTrace(..)) => unreachable!(),
            _ => false,
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Nil => write!(f, "nil"),
            Variant::Integer(i) => write!(f, "{}", i),
            Variant::Character(i) => write!(f, "{}", i),
            Variant::Atom(i) => write!(f, ":{}", atom::to_str(&Variant::Atom(*i)).unwrap()),
            Variant::Tuple(t) => unsafe {
                write!(f, "{{")?;
                let slice: &[Value] = &(**t);
                let mut iter = slice.iter().peekable();
                while let Some(val) = iter.next() {
                    write!(f, "{}", val)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            },
            Variant::List(c) => unsafe {
                write!(f, "[")?;
                let mut cons = *c;
                loop {
                    write!(f, "{}", (*cons).head)?;
                    match &(*cons).tail {
                        // Proper list ends here, do not show the tail
                        Variant::Nil => break,
                        // List continues, print a comma and follow the tail
                        Variant::List(c) => {
                            write!(f, ", ")?;
                            cons = *c;
                        }
                        // Improper list, show tail
                        val => {
                            write!(f, "| {}", val)?;
                            break;
                        }
                    }
                }
                write!(f, "]")
            },
            Variant::Pid(pid) => write!(f, "#Pid<{}>", pid),
            v => write!(f, "({:?})", v),
        }
    }
}

#[allow(clippy::mut_from_ref)]
pub fn tuple(heap: &Heap, len: u32) -> &mut Tuple {
    let layout = Layout::new::<Value>().repeat(len as usize).unwrap().0;
    let tuple = heap.alloc(self::Tuple {
        len,
        ptr: NonNull::dangling(),
    });
    tuple.ptr = heap.alloc_layout(layout).cast();
    tuple
}

pub fn cons(heap: &Heap, head: Value, tail: Value) -> Value {
    Value::List(heap.alloc(self::Cons { head, tail }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value;

    #[test]
    fn test_list_equality() {
        let heap = &Heap::new();
        let v1 = cons!(
            heap,
            Value::Integer(1),
            cons!(heap, Value::Integer(2), Value::Nil)
        );
        let v2 = cons!(
            heap,
            Value::Integer(1),
            cons!(heap, Value::Integer(2), Value::Nil)
        );
        assert!(v1.erl_eq(&v2));

        let v3 = cons!(
            heap,
            Value::Integer(1),
            cons!(heap, Value::Integer(3), Value::Nil)
        );
        assert!(!v1.erl_eq(&v3));
    }

    #[test]
    fn test_tuple_equality() {
        let heap = &Heap::new();
        let v1 = tup2!(heap, Value::Integer(1), Value::Integer(2));
        let v2 = tup2!(heap, Value::Integer(1), Value::Integer(2));
        assert!(v1.erl_eq(&v2));

        let v3 = tup3!(
            heap,
            Value::Integer(1),
            Value::Integer(1),
            Value::Integer(1)
        );
        assert!(!v1.erl_eq(&v3));
    }
}

// /// A pointer to a value managed by the GC.
// #[derive(Clone, Copy)]
// pub struct ValuePointer {
//     pub raw: TaggedPointer<Value>,
// }

// unsafe impl Send for ValuePointer {}
// unsafe impl Sync for ValuePointer {}
