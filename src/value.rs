use crate::atom;
use crate::bitstring;
use crate::exception;
use crate::immix::Heap;
use crate::module;
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

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Hash)]
pub enum Value {
    // Immediate values
    Nil, // also known as nil
    Integer(i64),
    Character(u8),
    Atom(u32),
    Pid(process::PID),
    Port(u32),
    Ref(u32),
    Float(self::Float),
    // Extended values (on heap)
    List(*const self::Cons),
    Tuple(*const self::Tuple),
    /// Boxed values
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    Binary(Arc<bitstring::Binary>),
    Map(self::Map),

    /// An interned string is a string allocated on the permanent space. For
    /// every unique interned string there is only one object allocated.
    // InternedBinary(Arc<String>),
    BigInt(Box<BigInt>), // Arc<BigInt>
    Closure(*const self::Closure),

    /// Special loader values (invalid in user runtime)
    // Import(), Export(),
    Literal(u32),
    X(u32),
    Y(u32),
    Label(u32),
    ExtendedList(Box<Vec<Value>>),
    FloatReg(u32),
    AllocList(Box<Vec<(u8, u32)>>),
    ExtendedLiteral(u32), // TODO; replace at load time

    /// Special emulator values

    /// An internal placeholder signifying "THE_NON_VALUE".
    None,
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

// Stores data on the process heap. Small, but expensive to copy.
// HeapBin(len + ptr)
// Stores data off the process heap, in an Arc<>. Cheap to copy around.
// RefBin(Arc<String/Vec<u8?>>)
// ^^ start with just RefBin since Rust already will do the String management for us
// SubBin(len (original?), offset, bitsize,bitoffset,is_writable, orig_ptr -> Bin/RefBin)

// consider using an Arc<RwLock<>> to make the inner string mutable? is the overhead worth it?
// data is always append only, so maybe have an atomic bool for the writable bit and keep the
// normal structure lockless.

// bitstring is the base model, binary is an 8-bit aligned bitstring
// https://www.reddit.com/r/rust/comments/2d7rrj/bit_level_pattern_matching/
// https://docs.rs/bitstring/0.1.1/bitstring/bit_string/trait.BitString.html

impl Value {
    pub fn is_none(&self) -> bool {
        match *self {
            Value::None => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match *self {
            Value::BigInt(..) => true,
            Value::Integer(..) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match *self {
            Value::Float(..) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Value::Float(..) => true,
            Value::BigInt(..) => true,
            Value::Integer(..) => true,
            _ => false,
        }
    }

    pub fn is_atom(&self) -> bool {
        match *self {
            Value::Atom(..) => true,
            _ => false,
        }
    }

    pub fn is_pid(&self) -> bool {
        match *self {
            Value::Pid(..) => true,
            _ => false,
        }
    }

    pub fn is_ref(&self) -> bool {
        match *self {
            Value::Ref(..) => true,
            _ => false,
        }
    }

    pub fn is_port(&self) -> bool {
        match *self {
            Value::Port(..) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match *self {
            Value::Nil => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match *self {
            Value::Binary(..) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match *self {
            Value::List { .. } => true,
            Value::Nil => true, // apparently also valid
            _ => false,
        }
    }

    pub fn is_non_empty_list(&self) -> bool {
        match *self {
            Value::List(ptr) => unsafe { !(*ptr).head.is_nil() },
            _ => false,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match *self {
            Value::Tuple(..) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match *self {
            Value::Closure(..) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match *self {
            Value::Atom(atom::TRUE) | Value::Atom(atom::FALSE) => true,
            _ => false,
        }
    }

    pub fn is_map(&self) -> bool {
        match *self {
            Value::Map(..) => true,
            _ => false,
        }
    }

    pub fn is_cp(&self) -> bool {
        match *self {
            Value::CP(..) => true,
            _ => false,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match *self {
            Value::Literal(i) => i,
            Value::Atom(i) => i,
            Value::Label(i) => i,
            Value::Pid(i) => i,
            Value::Integer(i) => i as u32,
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            return Value::Atom(atom::TRUE);
        }
        Value::Atom(atom::FALSE)
    }

    pub fn erl_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
            (Value::Character(c1), Value::Character(c2)) => c1 == c2,
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::BigInt(b1), Value::BigInt(b2)) => b1 == b2,
            (Value::Integer(_), Value::Float(_)) => unimplemented!(),
            (Value::Float(_), Value::Integer(_)) => unimplemented!(),

            (Value::Atom(a1), Value::Atom(a2)) => a1 == a2,
            (Value::Pid(p1), Value::Pid(p2)) => p1 == p2,
            (Value::Port(p1), Value::Port(p2)) => p1 == p2,
            (Value::Ref(r1), Value::Ref(r2)) => r1 == r2,

            (Value::List(l1), Value::List(l2)) => unsafe {
                (**l1)
                    .iter()
                    .zip((**l2).iter())
                    .all(|(e1, e2)| e1.erl_eq(e2))
            },
            (Value::Tuple(v1), Value::Tuple(v2)) => unsafe {
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
            (Value::Binary(b1), Value::Binary(b2)) => b1 == b2,
            (Value::Literal(l1), Value::Literal(l2)) => l1 == l2,
            (Value::X(l1), Value::X(l2)) => l1 == l2,
            (Value::Y(l1), Value::Y(l2)) => l1 == l2,
            (Value::FloatReg(l1), Value::FloatReg(l2)) => l1 == l2,
            (Value::Label(l1), Value::Label(l2)) => l1 == l2,
            (Value::Closure(c1), Value::Closure(c2)) => unsafe { (**c1).mfa == (**c2).mfa },
            (Value::CP(l1), Value::CP(l2)) => l1 == l2,
            (Value::Catch(l1), Value::Catch(l2)) => l1 == l2,
            (Value::Closure { .. }, _) => unreachable!(), // There should never happen
            (_, Value::Closure { .. }) => unreachable!(),
            (Value::ExtendedList { .. }, _) => unreachable!(),
            (_, Value::ExtendedList { .. }) => unreachable!(),
            (Value::AllocList(..), _) => unreachable!(),
            (_, Value::AllocList(..)) => unreachable!(),
            (Value::ExtendedLiteral(..), _) => unreachable!(),
            (_, Value::ExtendedLiteral(..)) => unreachable!(),
            (Value::StackTrace(..), _) => unreachable!(),
            (_, Value::StackTrace(..)) => unreachable!(),
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Character(i) => write!(f, "{}", i),
            Value::Atom(i) => write!(f, ":{}", atom::to_str(&Value::Atom(*i)).unwrap()),
            Value::Tuple(t) => unsafe {
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
            Value::List(c) => unsafe {
                write!(f, "[")?;
                let mut cons = *c;
                loop {
                    write!(f, "{}", (*cons).head)?;
                    match &(*cons).tail {
                        // Proper list ends here, do not show the tail
                        Value::Nil => break,
                        // List continues, print a comma and follow the tail
                        Value::List(c) => {
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
            Value::Pid(pid) => write!(f, "#Pid<{}>", pid),
            Value::X(i) => write!(f, "x({})", i),
            Value::Y(i) => write!(f, "y({})", i),
            Value::Literal(..) => write!(f, "(literal)"),
            Value::Label(..) => write!(f, "(label)"),
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
