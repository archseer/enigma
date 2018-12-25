// use crate::arc_without_weak::ArcWithoutWeak;
use crate::atom;
use crate::module;
use crate::process;
use num::bigint::BigInt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
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
    Float(f64),
    // Extended values (on heap)
    List(*const self::Cons),
    Tuple(*const self::Tuple), // TODO: allocate on custom heap
    /// Boxed values
    /// Strings use an Arc so they can be sent to other processes without
    /// requiring a full copy of the data.
    //Binary(ArcWithoutWeak<ImmutableString>),

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

// TODO: maybe box binaries further:
// // contains size, followed in memory by the data bytes
// ProcBin { nbytes: Word } ,
// // contains reference to heapbin
// RefBin,
// // stores data on a separate heap somewhere else with refcount
// HeapBin { nbytes: Word, refc: Word },

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

// /// A pointer to a value managed by the GC.
// #[derive(Clone, Copy)]
// pub struct ValuePointer {
//     pub raw: TaggedPointer<Value>,
// }

// unsafe impl Send for ValuePointer {}
// unsafe impl Sync for ValuePointer {}
