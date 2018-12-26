use crate::arc_without_weak::ArcWithoutWeak;
use crate::atom;
use crate::immix::Heap;
use crate::module;
use crate::process;
use allocator_api::Layout;
use num::bigint::BigInt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
// annoying: we have to wrap Floats to be able to define hash
pub struct Float(pub f64);
impl Eq for Float {}
impl Hash for Float {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("Don't use floats as hash keys")
    }
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Hash)]
pub enum Value {
    // Immediate values
    Nil(), // also known as nil
    Integer(i64),
    Character(u8),
    Atom(usize),
    Catch(),
    Pid(process::PID),
    Port(),
    Ref(),
    Float(self::Float),
    // Extended values (on heap)
    List(*const self::Cons),
    Tuple(*const self::Tuple), // TODO: allocate on custom heap
    /// Boxed values
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    Binary(ArcWithoutWeak<String>),

    /// An interned string is a string allocated on the permanent space. For
    /// every unique interned string there is only one object allocated.
    //InternedBinary(ArcWithoutWeak<ImmutableString>),
    BigInt(Box<BigInt>), // ArcWithoutWeak<BigInt>
    Closure(*const self::Closure),
    /// Special values (invalid in runtime)
    // Import(), Export(),
    Literal(usize),
    X(usize),
    Y(usize),
    Label(usize),
    ExtendedList(Vec<Value>),
    FloatReg(usize),
    AllocList(u64),
    ExtendedLiteral(usize), // TODO; replace at load time
    CP(Option<usize>),      // continuation pointer
}

#[derive(Debug)]
pub struct Cons {
    pub head: Value,
    pub tail: Value,
}

#[derive(Debug)]
pub struct Tuple {
    /// Number of elements following the header.
    pub len: usize,
    pub ptr: NonNull<Value>,
}

#[derive(Debug)]
pub struct Closure {
    pub mfa: module::MFA,
    pub ptr: usize,
    pub binding: Option<Vec<Value>>,
}

impl Deref for Tuple {
    type Target = [Value];
    fn deref(&self) -> &[Value] {
        unsafe { ::std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl DerefMut for Tuple {
    fn deref_mut(&mut self) -> &mut [Value] {
        unsafe { ::std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

unsafe impl Sync for Value {}
unsafe impl Send for Value {}

unsafe impl Sync for Cons {}

// Stores data on the process heap. Small, but expensive to copy.
// HeapBin(len + ptr)
// Stores data off the process heap, in an ArcWithoutWeak<>. Cheap to copy around.
// RefBin(Arc<String/Vec<u8?>>)
// ^^ start with just RefBin since Rust already will do the String management for us
// SubBin(len (original?), offset, bitsize,bitoffset,is_writable, orig_ptr -> Bin/RefBin)

// bitstring is the base model, binary is an 8-bit aligned bitstring
// https://www.reddit.com/r/rust/comments/2d7rrj/bit_level_pattern_matching/
// https://docs.rs/bitstring/0.1.1/bitstring/bit_string/trait.BitString.html

impl Value {
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
            Value::Nil(..) => true,
            _ => false,
        }
    }

    // TODO: is_binary

    pub fn is_list(&self) -> bool {
        match *self {
            Value::List { .. } => true,
            Value::Nil(..) => true, // apparently also valid
            _ => false,
        }
    }

    pub fn is_non_empty_list(&self) -> bool {
        match *self {
            Value::List(ptr) => {
                // TODO: traverse the list recursively and check the last tail?
                // !ptr.is_nil()
                false
            }
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

    pub fn to_usize(&self) -> usize {
        match *self {
            Value::Literal(i) => i,
            Value::Atom(i) => i,
            Value::Label(i) => i,
            Value::Pid(i) => i,
            Value::Integer(i) => i as usize,
            _ => panic!("Unimplemented to_usize for {:?}", self),
        }
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            return Value::Atom(atom::TRUE);
        }
        Value::Atom(atom::FALSE)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil() => write!(f, "nil"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Character(i) => write!(f, "{}", i),
            Value::Atom(i) => write!(f, ":{}", atom::to_str(&Value::Atom(*i)).unwrap()),
            Value::Tuple(t) => unsafe {
                write!(f, "{{")?;
                let slice: &[Value] = &(**t);
                slice.iter().for_each(|val| write!(f, "{}, ", val).unwrap());
                write!(f, "}}")
            },
            Value::List(c) => unsafe {
                write!(f, "[")?;
                let mut cons = *c;
                loop {
                    write!(f, "{}", (*cons).head)?;
                    match &(*cons).tail {
                        // Proper list ends here, do not show the tail
                        Value::Nil() => break,
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

pub fn tuple(heap: &Heap, len: usize) -> &mut Tuple {
    let layout = Layout::new::<Value>().repeat(len).unwrap().0;
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

// /// A pointer to a value managed by the GC.
// #[derive(Clone, Copy)]
// pub struct ValuePointer {
//     pub raw: TaggedPointer<Value>,
// }

// unsafe impl Send for ValuePointer {}
// unsafe impl Sync for ValuePointer {}
