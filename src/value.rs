use crate::atom;
use crate::bitstring;
use crate::exception;
use crate::immix::Heap;
use crate::module;
use crate::nanbox::TypedNanBox;
use crate::process::{self, InstrPtr};
use crate::servo_arc::Arc;
use allocator_api::Layout;
use num::bigint::BigInt;
use std::hash::{Hash, Hasher};

mod cons;
mod map;
mod tuple;
pub use cons::Cons;
pub use map::{Map, HAMT};
pub use tuple::Tuple;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
// annoying: we have to wrap Floats to be able to define hash
pub struct Float(pub f64);
impl Eq for Float {}
impl Hash for Float {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
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

const TERM_FLOAT: u8 = 0;
const TERM_NIL: u8 = 1;
const TERM_INTEGER: u8 = 2;
const TERM_ATOM: u8 = 3;
const TERM_PORT: u8 = 4;
const TERM_PID: u8 = 5;
const TERM_CONS: u8 = 6;
const TERM_POINTER: u8 = 7;

struct WrongBoxError;

/// A term is a nanboxed compact representation of a value in 64 bits. It can either be immediate,
/// in which case it embeds the data, or a boxed pointer, that points to more data.
//#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Hash)]
#[derive(Debug, Clone, Eq)] // TODO make it Copy
pub struct Term {
    value: TypedNanBox<Variant>,
}

unsafe impl Sync for Term {}
unsafe impl Send for Term {}

impl Hash for Term {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // maybe we could hash the repr directly
        self.into_variant().hash(state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Special {
    Nil,
    /// An internal placeholder signifying "THE_NON_VALUE".
    None,
    Literal(),
}

#[derive(Debug, Clone, Eq, PartialOrd, Hash)]
pub enum Variant {
    Float(f64),
    Nil(Special), // TODO: expand nil to be able to hold different types of empty (tuple, list, map)
    Integer(i32),
    Atom(u32),
    Port(u32),
    Pid(process::PID),
    Cons(*const self::Cons),
    Pointer(*const Header), // tuple, map, binary, ref
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

impl From<&mut Cons> for Term {
    fn from(value: &mut Cons) -> Term {
        Term::from(Variant::Cons(value))
    }
}

impl From<&mut Tuple> for Term {
    fn from(value: &mut Tuple) -> Term {
        Term::from(Variant::Pointer(value as *const Tuple as *const Header))
    }
}

impl From<Variant> for Term {
    fn from(value: Variant) -> Term {
        unsafe {
            match value {
                Variant::Float(value) => Term {
                    value: TypedNanBox::new(TERM_FLOAT, value),
                },
                Variant::Nil(..) => Term {
                    value: TypedNanBox::new(TERM_NIL, 0),
                },
                Variant::Integer(value) => Term {
                    value: TypedNanBox::new(TERM_INTEGER, value),
                },
                Variant::Atom(value) => Term {
                    value: TypedNanBox::new(TERM_ATOM, value),
                },
                Variant::Port(value) => Term {
                    value: TypedNanBox::new(TERM_PORT, value),
                },
                Variant::Pid(value) => Term {
                    value: TypedNanBox::new(TERM_PID, value),
                },
                Variant::Cons(value) => Term {
                    value: TypedNanBox::new(TERM_CONS, value),
                },
                Variant::Pointer(value) => Term {
                    value: TypedNanBox::new(TERM_POINTER, value),
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
                TERM_FLOAT => Variant::Float(value.unpack()),
                TERM_NIL => Variant::Nil(0),
                TERM_INTEGER => Variant::Integer(value.unpack()),
                TERM_ATOM => Variant::Atom(value.unpack()),
                TERM_PORT => Variant::Port(value.unpack()),
                TERM_PID => Variant::Pid(value.unpack()),
                TERM_CONS => Variant::Cons(value.unpack()),
                TERM_POINTER => Variant::Pointer(value.unpack()),
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

/// Represents the header of a boxed value on the heap. Is followed by value.
pub type Header = u8;

pub const BOXED_REF: u8 = 0;
pub const BOXED_TUPLE: u8 = 1;
pub const BOXED_BINARY: u8 = 2;
pub const BOXED_MAP: u8 = 3;
pub const BOXED_BIGINT: u8 = 4;
pub const BOXED_CLOSURE: u8 = 5;

// pub enum Value {
//     /// Special emulator values

//     /// continuation pointer
//     CP(Option<InstrPtr>),
//     /// Catch context
//     Catch(InstrPtr),
//     /// Stack trace
//     StackTrace(*const exception::StackTrace),
// }

/// Strings use an Arc so they can be sent to other processes without
/// requiring a full copy of the data.
#[derive(Debug)]
#[repr(C)]
pub struct Binary {
    pub header: Header,
    pub value: bitstring::Binary,
}

#[derive(Debug)]
#[repr(C)]
pub struct Bignum {
    pub header: Header,
    pub value: BigInt,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ref {
    pub header: Header,
    pub value: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Closure {
    pub header: Header,
    pub mfa: module::MFA,
    pub ptr: u32,
    pub binding: Option<Vec<Term>>,
}

// term order:
// number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string
#[derive(Eq, PartialEq)]
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
    pub fn nil() -> Self {
        Term {
            value: TypedNanBox::new(TERM_NIL, 0),
        }
    }

    #[inline]
    pub fn atom(value: u32) -> Self {
        Term {
            value: TypedNanBox::new(TERM_ATOM, value),
        }
    }

    // TODO: just use Term::from everywhere
    #[inline]
    pub fn int(value: i32) -> Self {
        Term::from(value as i32)
    }

    // immediates

    #[inline]
    pub fn is_none(&self) -> bool {
        self.value.tag() == 7
    }

    pub fn is_float(&self) -> bool {
        self.value.tag() == TERM_FLOAT
    }

    pub fn is_nil(&self) -> bool {
        self.value.tag() == TERM_NIL
    }

    pub fn is_smallint(&self) -> bool {
        self.value.tag() == TERM_INTEGER
    }

    pub fn is_atom(&self) -> bool {
        self.value.tag() == TERM_ATOM
    }

    pub fn is_port(&self) -> bool {
        self.value.tag() == TERM_PORT
    }

    pub fn is_pid(&self) -> bool {
        self.value.tag() == TERM_PID
    }

    pub fn is_pointer(&self) -> bool {
        self.value.tag() == TERM_POINTER
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        let tag = self.value.tag();
        tag == TERM_POINTER || tag == TERM_NIL
    }

    #[inline]
    pub fn get_type(&self) -> Type {
        match self.value.tag() {
            TERM_FLOAT => Type::Number,
            TERM_NIL => Type::Nil,
            TERM_INTEGER => Type::Number,
            TERM_ATOM => Type::Atom,
            TERM_PORT => Type::Port,
            TERM_PID => Type::Pid,
            TERM_CONS => Type::Pid,
            TERM_POINTER => match self.get_boxed_header() {
                BOXED_REF => Type::Ref,
                BOXED_TUPLE => Type::Tuple,
                BOXED_BINARY => Type::Binary,
                BOXED_MAP => Type::Map,
                BOXED_BIGINT => Type::Number,
                BOXED_CLOSURE => Type::Closure,
                _ => unimplemented!(),
            },
            _ => unreachable!(),
        }
    }

    pub fn get_boxed_header(&self) -> Header {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return *ptr }
        }
        panic!("Not a boxed type!")
    }

    pub fn get_boxed_value<T>(&self) -> &T {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return &*(ptr as *const T) }
        }
        panic!("Not a boxed type!")
    }

    pub fn get_boxed_value_mut<T>(&self) -> &mut T {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return &mut *(ptr as *mut T) }
        }
        panic!("Not a boxed type!")
    }

    // ------

    #[inline]
    pub fn is_integer(&self) -> bool {
        match *self {
            Variant::BigInt(..) => true,
            Variant::Integer(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        self.get_type() == Type::Number
    }

    #[inline]
    pub fn is_ref(&self) -> bool {
        self.get_type() == Type::Ref
    }

    pub fn is_binary(&self) -> bool {
        self.get_type() == Type::Binary
    }

    #[inline]
    pub fn is_non_empty_list(&self) -> bool {
        match self.into_variant() {
            Variant::Cons(ptr) => unsafe { !(*ptr).head.is_nil() },
            _ => false,
        }
    }

    #[inline]
    pub fn is_tuple(&self) -> bool {
        self.get_type() == Type::Tuple
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        self.get_type() == Type::Closure
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        match self.into_variant() {
            Variant::Atom(atom::TRUE) | Variant::Atom(atom::FALSE) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_map(&self) -> bool {
        self.get_type() == Type::Map
    }

    pub fn is_cp(&self) -> bool {
        match *self {
            Variant::CP(..) => true,
            _ => false,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self.into_variant() {
            Variant::Atom(i) => i,
            Variant::Pid(i) => i,
            Variant::Integer(i) => i as u32,
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            return Variant::Atom(atom::TRUE).into();
        }
        Variant::Atom(atom::FALSE).into()
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.into_variant().eq(other.into_variant())
    }
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Variant::Nil(..), Variant::Nil(..)) => true,
            (Variant::Integer(i1), Variant::Integer(i2)) => i1 == i2,
            (Variant::Float(f1), Variant::Float(f2)) => f1 == f2,

            (Variant::Atom(a1), Variant::Atom(a2)) => a1 == a2,
            (Variant::Pid(p1), Variant::Pid(p2)) => p1 == p2,
            (Variant::Port(p1), Variant::Port(p2)) => p1 == p2,

            (Variant::Cons(l1), Variant::Cons(l2)) => unsafe {
                (**l1)
                    .iter()
                    .zip((**l2).iter())
                    .all(|(e1, e2)| e1.erl_eq(e2))
            },

            (Variant::Pointer(p1), Variant::Pointer(p2)) => unsafe {

            (Variant::Ref(r1), Variant::Ref(r2)) => r1 == r2,
            (Variant::BigInt(b1), Variant::BigInt(b2)) => b1 == b2,
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
    // non strict comparisons need to handle these + bigint
    // (Variant::Integer(_), Variant::Float(_)) => unimplemented!(),
    // (Variant::Float(_), Variant::Integer(_)) => unimplemented!(),
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.into_variant())
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Nil(..) => write!(f, "nil"),
            Variant::Integer(i) => write!(f, "{}", i),
            Variant::Atom(i) => write!(f, ":{}", atom::to_str(*i).unwrap()),
            Variant::Port(i) => write!(f, "#Port<{}>", i),
            Variant::Pid(i) => write!(f, "#Pid<{}>", i),
            Variant::Cons(c) => unsafe {
                write!(f, "[")?;
                let mut cons = *c;
                loop {
                    write!(f, "{}", (*cons).head)?;
                    match (*cons).tail.into_variant() {
                        // Proper list ends here, do not show the tail
                        Variant::Nil(..) => break,
                        // List continues, print a comma and follow the tail
                        Variant::Cons(c) => {
                            write!(f, ", ")?;
                            cons = c;
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
            Variant::Pointer(ptr) => unsafe {
                match **ptr {
                    BOXED_TUPLE => {
                        let t = *(*ptr as *const Tuple);
                        
                        write!(f, "{{")?;
                        let mut iter = t.iter().peekable();
                        while let Some(val) = iter.next() {
                            write!(f, "{}", val)?;
                            if iter.peek().is_some() {
                                write!(f, ", ")?;
                            }
                        }
                        write!(f, "}}")
                    },
                    BOXED_REF => write!(f, "#Ref<>"),
                    BOXED_BINARY => write!(f, "#Binary<>"),
                    BOXED_MAP => write!(f, "#Map<>"),
                    BOXED_BIGINT => write!(f, "#BigInt<>"),
                    BOXED_CLOSURE => write!(f, "#Closure<>"),
                    _ => unimplemented!(),
                }
            }
        }
    }
}

#[allow(clippy::mut_from_ref)]
pub fn tuple(heap: &Heap, len: u32) -> &mut Tuple {
    let tuple = heap.alloc(self::Tuple {
        header: BOXED_TUPLE,
        len,
    });
    let layout = Layout::new::<Term>().repeat(len as usize).unwrap().0;
    heap.alloc_layout(layout); // TODO: do something with the ptr
    tuple
}

pub fn cons(heap: &Heap, head: Term, tail: Term) -> Term {
    Term::from(heap.alloc(self::Cons { head, tail }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value;

    #[test]
    fn test_list_equality() {
        let heap = &Heap::new();
        let v1 = cons!(heap, Term::int(1), cons!(heap, Term::int(2), Term::nil()));
        let v2 = cons!(heap, Term::int(1), cons!(heap, Term::int(2), Term::nil()));
        assert!(v1.erl_eq(&v2));

        let v3 = cons!(heap, Term::int(1), cons!(heap, Term::int(3), Term::nil()));
        assert!(!v1.erl_eq(&v3));
    }

    #[test]
    fn test_tuple_equality() {
        let heap = &Heap::new();
        let v1 = tup2!(heap, Term::int(1), Term::int(2));
        let v2 = tup2!(heap, Term::int(1), Term::int(2));
        assert!(v1.erl_eq(&v2));

        let v3 = tup3!(heap, Term::int(1), Term::int(1), Term::int(1));
        assert!(!v1.erl_eq(&v3));
    }
}
