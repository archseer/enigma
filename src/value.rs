// We cast header -> boxed value a lot in this file.
#![allow(clippy::cast_ptr_alignment)]

use crate::atom;
use crate::bitstring;
use crate::exception;
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader;
use crate::module;
use crate::nanbox::TypedNanBox;
use crate::process;
use crate::servo_arc::Arc;
use allocator_api::Layout;
use num_bigint::BigInt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

mod closure;
pub mod cons;
mod map;
mod tuple;
pub use self::closure::Closure;
pub use self::cons::Cons;
pub use self::map::{Map, HAMT};
pub use self::tuple::Tuple;

pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(value: &T) -> Result<&Self, Self::Error>;
}

pub trait TryInto<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_into(&self) -> Result<&T, Self::Error>;
}

pub trait TryIntoMut<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_into_mut(&self) -> Result<&mut T, Self::Error>;
}

// TryFrom implies TryInto
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
{
    type Error = U::Error;

    fn try_into(&self) -> Result<&U, U::Error> {
        U::try_from(self)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
// annoying: we have to wrap Floats to be able to define hash
pub struct Float(pub f64);
impl Eq for Float {}
impl Hash for Float {
    // hash raw float
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bits = if self.0 == 0.0 {
            0 // this accounts for +0.0 and -0.0
        } else {
            unsafe { std::mem::transmute::<f64, u64>(self.0) }
        };
        bits.hash(state);
    }
}

pub const TERM_FLOAT: u8 = 0;
pub const TERM_NIL: u8 = 1;
pub const TERM_INTEGER: u8 = 2;
pub const TERM_ATOM: u8 = 3;
pub const TERM_PORT: u8 = 4;
pub const TERM_PID: u8 = 5;
pub const TERM_CONS: u8 = 6;
pub const TERM_POINTER: u8 = 7;

#[derive(Debug)]
pub struct WrongBoxError;

/// A term is a nanboxed compact representation of a value in 64 bits. It can either be immediate,
/// in which case it embeds the data, or a boxed pointer, that points to more data.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Term {
    value: TypedNanBox<Variant>,
}

unsafe impl Sync for Term {}
unsafe impl Send for Term {}

impl Hash for Term {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // maybe we could hash the f64 repr directly in some cases
        match self.into_variant() {
            Variant::Pointer(p) => match self.get_boxed_header().unwrap() {
                BOXED_BINARY => {
                    let value = &self.get_boxed_value::<bitstring::RcBinary>().unwrap();

                    BOXED_BINARY.hash(state);
                    value.data.hash(state)
                }
                BOXED_TUPLE => {
                    let value = unsafe { &*(p as *const Tuple) };

                    BOXED_TUPLE.hash(state);
                    value.as_slice().hash(state)
                }
                BOXED_REF => {
                    let value = &self.get_boxed_value::<process::Ref>().unwrap();
                    BOXED_REF.hash(state);
                }
                _ => unimplemented!("unimplemented Hash for {}", self),
            },
            Variant::Cons(..) => {
                // let value = unsafe { &*(ptr as *const Cons) };
                let mut list = self;

                // hash all values, including the tal.
                while let Ok(Cons { head, tail }) = list.try_into() {
                    head.hash(state);
                    list = tail;
                }
                list.hash(state)
            }
            variant => variant.hash(state),
        }
    }
}

// NEED hash for boxed types

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Special {
    Nil = 0,
    /// An internal placeholder signifying "THE_NON_VALUE".
    None = 1,
    //Literal,
}

#[derive(Debug, Copy, Clone, Eq, Hash)]
pub enum Variant {
    Float(self::Float),
    Nil(Special), // TODO: expand nil to be able to hold different types of empty (tuple, list, map)
    Integer(i32),
    Atom(u32),
    Port(u32),
    Pid(process::PID),
    Cons(*const self::Cons),
    Pointer(*const Header), // tuple, map, binary, ref
}
unsafe impl Send for Variant {}
// unsafe impl Sync for Variant {}

impl From<f64> for Term {
    fn from(value: f64) -> Term {
        Term::from(Variant::Float(self::Float(value)))
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

impl<T> From<&mut Boxed<T>> for Term {
    fn from(value: &mut Boxed<T>) -> Term {
        Term::from(Variant::Pointer(value as *const Boxed<T> as *const Header))
    }
}

impl From<Variant> for Term {
    fn from(value: Variant) -> Term {
        unsafe {
            match value {
                Variant::Float(self::Float(value)) => Term {
                    value: TypedNanBox::new(TERM_FLOAT, value),
                },
                Variant::Nil(value) => Term {
                    value: TypedNanBox::new(TERM_NIL, value),
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
            match value.tag() as u8 {
                TERM_FLOAT => Variant::Float(self::Float(value.unpack())),
                TERM_NIL => Variant::Nil(value.unpack()),
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
/// Any value allocated on the heap needs repr(C) to guarantee the ordering.
/// This is because we always point to Header, then we recast into the right type.
///
/// TODO: We could avoid this by having the value follow the header and offseting the pointer by
/// header, but that means we'd need to have the header be one full processor word wide to ensure
/// alignment. That means there would be some wasted space.
pub type Header = u8;

pub const BOXED_REF: u8 = 0;
pub const BOXED_TUPLE: u8 = 1;
pub const BOXED_BINARY: u8 = 2;
pub const BOXED_MAP: u8 = 3;
pub const BOXED_BIGINT: u8 = 4;
pub const BOXED_CLOSURE: u8 = 5;
// TODO: these should be direct pointers, no heap
pub const BOXED_CP: u8 = 6;
pub const BOXED_CATCH: u8 = 7;
pub const BOXED_STACKTRACE: u8 = 8;

pub const BOXED_MATCHSTATE: u8 = 9;
pub const BOXED_SUBBINARY: u8 = 10;

pub const BOXED_MODULE: u8 = 20;
pub const BOXED_EXPORT: u8 = 21;
pub const BOXED_FILE: u8 = 22;

#[derive(Debug)]
#[repr(C)]
pub struct Boxed<T> {
    pub header: Header,
    pub value: T,
}

// term order:
// number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
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

    // runtime values
    CP,
    Catch,
    MatchState,
}

pub enum Num {
    Float(f64),
    Integer(i32),
    Bignum(BigInt),
}

impl Term {
    #[inline(always)]
    pub fn tag(&self) -> u8 {
        self.value.tag() as u8
    }

    #[inline(always)]
    pub fn nil() -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_NIL, Special::Nil),
            }
        }
    }

    #[inline]
    pub fn none() -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_NIL, Special::None),
            }
        }
    }

    #[inline]
    pub fn atom(value: u32) -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_ATOM, value),
            }
        }
    }

    // TODO: just use Term::from everywhere
    #[inline]
    pub fn int(value: i32) -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_INTEGER, value),
            }
        }
    }

    #[inline]
    pub fn int64(heap: &Heap, value: i64) -> Self {
        if value > (i32::max_value() as i64) {
            Term::bigint(heap, BigInt::from(value))
        } else {
            unsafe {
                Term {
                    value: TypedNanBox::new(TERM_INTEGER, value as i32),
                }
            }
        }
    }

    #[inline]
    pub fn uint(heap: &Heap, value: u32) -> Self {
        if value > (i32::max_value() as u32) {
            Term::bigint(heap, BigInt::from(value))
        } else {
            unsafe {
                Term {
                    value: TypedNanBox::new(TERM_INTEGER, value as i32),
                }
            }
        }
    }

    #[inline]
    pub fn uint64(heap: &Heap, value: u64) -> Self {
        if value > (i32::max_value() as u64) {
            Term::bigint(heap, BigInt::from(value))
        } else {
            unsafe {
                Term {
                    value: TypedNanBox::new(TERM_INTEGER, value as i32),
                }
            }
        }
    }

    pub fn pid(value: process::PID) -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_PID, value),
            }
        }
    }

    pub fn port(value: u32) -> Self {
        unsafe {
            Term {
                value: TypedNanBox::new(TERM_PORT, value),
            }
        }
    }

    pub fn reference(heap: &Heap, value: process::Ref) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_REF,
            value,
        }))
    }

    pub fn map(heap: &Heap, value: HAMT) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_MAP,
            value: Map(value),
        }))
    }

    pub fn closure(heap: &Heap, value: Closure) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_CLOSURE,
            value,
        }))
    }

    pub fn bigint(heap: &Heap, value: BigInt) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_BIGINT,
            value,
        }))
    }

    pub fn binary(heap: &Heap, value: bitstring::Binary) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_BINARY,
            value: Arc::new(value),
        }))
    }

    pub fn subbinary(heap: &Heap, value: bitstring::SubBinary) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_SUBBINARY,
            value,
        }))
    }

    pub fn matchstate(heap: &Heap, value: bitstring::MatchState) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_MATCHSTATE,
            value,
        }))
    }

    pub fn cp(heap: &Heap, value: Option<InstrPtr>) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_CP,
            value,
        }))
    }

    pub fn catch(heap: &Heap, value: InstrPtr) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_CATCH,
            value,
        }))
    }

    pub fn stacktrace(heap: &Heap, value: exception::StackTrace) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_STACKTRACE,
            value,
        }))
    }

    pub fn export(heap: &Heap, value: module::MFA) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_EXPORT,
            value,
        }))
    }

    pub fn file(heap: &Heap, value: std::fs::File) -> Self {
        Term::from(heap.alloc(Boxed {
            header: BOXED_FILE,
            value,
        }))
    }

    pub fn boxed<T>(heap: &Heap, header: u8, value: T) -> Self {
        Term::from(heap.alloc(Boxed { header, value }))
    }

    // immediates

    #[inline(always)]
    pub fn is_none(self) -> bool {
        self.value.tag() as u8 == TERM_NIL // TODO
    }

    #[inline(always)]
    pub fn is_float(self) -> bool {
        self.value.tag() as u8 == TERM_FLOAT
    }

    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.value.tag() as u8 == TERM_NIL
    }

    #[inline(always)]
    pub fn is_smallint(self) -> bool {
        self.value.tag() as u8 == TERM_INTEGER
    }

    #[inline(always)]
    pub fn is_atom(self) -> bool {
        self.value.tag() as u8 == TERM_ATOM
    }

    #[inline(always)]
    pub fn is_port(self) -> bool {
        self.value.tag() as u8 == TERM_PORT
    }

    #[inline(always)]
    pub fn is_pid(self) -> bool {
        self.value.tag() as u8 == TERM_PID
    }

    #[inline(always)]
    pub fn is_pointer(self) -> bool {
        self.value.tag() as u8 == TERM_POINTER
    }

    #[inline(always)]
    pub fn is_list(self) -> bool {
        let tag = self.value.tag() as u8;
        tag == TERM_CONS || tag == TERM_NIL
        // TODO: is nil also ok?
    }

    #[inline(always)]
    pub fn is_immed(self) -> bool {
        let tag = self.value.tag() as u8;
        tag != TERM_CONS || tag != TERM_POINTER
    }

    #[inline]
    pub fn get_type(self) -> Type {
        match self.value.tag() as u8 {
            TERM_FLOAT => Type::Number,
            TERM_NIL => Type::Nil,
            TERM_INTEGER => Type::Number,
            TERM_ATOM => Type::Atom,
            TERM_PORT => Type::Port,
            TERM_PID => Type::Pid,
            TERM_CONS => Type::List,
            TERM_POINTER => match self.get_boxed_header().unwrap() {
                BOXED_REF => Type::Ref,
                BOXED_TUPLE => Type::Tuple,
                BOXED_BINARY => Type::Binary,
                BOXED_MAP => Type::Map,
                BOXED_BIGINT => Type::Number,
                BOXED_CLOSURE => Type::Closure,
                BOXED_CP => Type::CP,
                BOXED_CATCH => Type::Catch,
                BOXED_MATCHSTATE => Type::MatchState,
                BOXED_SUBBINARY => Type::Binary,
                BOXED_MODULE => Type::Ref, // init expects a module in progress as a ref
                BOXED_EXPORT => Type::Closure, // exports are a type of function
                BOXED_FILE => Type::Ref,   // files are stored as magic ref pointers in beam
                i => unimplemented!("get_type for {}", i),
            },
            _ => unreachable!(),
        }
    }

    // TODO: add unchecked variant
    pub fn get_boxed_header(self) -> Result<Header, String> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return Ok(*ptr) }
        }
        Err("Not a boxed type!".to_string())
    }

    // TODO: add unchecked variant
    pub fn get_boxed_value<T>(&self) -> Result<&T, &str> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return Ok(&(*(ptr as *const Boxed<T>)).value) }
        }
        Err("Not a boxed type!")
    }

    // TODO: add unchecked variant
    pub fn get_boxed_value_mut<T>(&self) -> Result<&mut T, &str> {
        if let Variant::Pointer(ptr) = self.into_variant() {
            unsafe { return Ok(&mut (*(ptr as *mut Boxed<T>)).value) }
        }
        Err("Not a boxed type!")
    }

    /// A method that's optimized for retrieving number types.
    pub fn into_number(self) -> Result<Num, ()> {
        match self.into_variant() {
            Variant::Integer(i) => Ok(Num::Integer(i)),
            Variant::Float(self::Float(i)) => Ok(Num::Float(i)),
            Variant::Pointer(ptr) => unsafe {
                match *ptr {
                    BOXED_BIGINT => {
                        let boxed = &*(ptr as *const Boxed<BigInt>);
                        Ok(Num::Bignum(boxed.value.clone()))
                    }
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }

    // TODO: ExtendedList should instead become a Term vec
    // TODO: might be atoms only??
    pub fn into_lvalue(self) -> Option<loader::LValue> {
        match self.into_variant() {
            Variant::Integer(i) => Some(loader::LValue::Integer(i)),
            Variant::Atom(i) => Some(loader::LValue::Atom(i)),
            Variant::Nil(..) => Some(loader::LValue::Nil),
            // TODO Variant::Float(self::Float(i)) => Num::Float(i),
            _ => None,
        }
    }

    // ------

    #[inline]
    pub fn is_integer(self) -> bool {
        match self.into_variant() {
            Variant::Integer(_) => true,
            Variant::Pointer(ptr) => unsafe {
                match *ptr {
                    BOXED_BIGINT => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }

    #[inline]
    pub fn is_number(self) -> bool {
        self.get_type() == Type::Number
    }

    #[inline]
    pub fn is_ref(self) -> bool {
        self.get_type() == Type::Ref
    }

    #[inline]
    pub fn is_bitstring(self) -> bool {
        self.get_type() == Type::Binary
    }

    #[inline]
    pub fn is_binary(self) -> bool {
        if let Variant::Pointer(ptr) = self.into_variant() {
            return match unsafe { *ptr } {
                BOXED_SUBBINARY => unsafe {
                    let Boxed { value: binary, .. } = &*(ptr as *const Boxed<bitstring::SubBinary>);
                    return binary.is_binary();
                },
                BOXED_BINARY => true,
                _ => false,
            };
        }
        false
    }

    #[inline]
    pub fn is_non_empty_list(self) -> bool {
        match self.into_variant() {
            Variant::Cons(_ptr) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_tuple(self) -> bool {
        self.get_type() == Type::Tuple
    }

    #[inline]
    pub fn is_function(self) -> bool {
        self.get_type() == Type::Closure
    }

    #[inline]
    pub fn is_boolean(self) -> bool {
        match self.into_variant() {
            Variant::Atom(atom::TRUE) | Variant::Atom(atom::FALSE) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_map(self) -> bool {
        self.get_type() == Type::Map
    }

    #[inline]
    pub fn is_cp(self) -> bool {
        self.get_type() == Type::CP
    }

    pub fn to_u32(self) -> u32 {
        match self.into_variant() {
            Variant::Atom(i) => i,
            Variant::Pid(i) => i,
            Variant::Integer(i) => i as u32,
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }

    // TODO: had to add this for the list_to_atom
    pub fn to_int(self) -> Option<u32> {
        match self.into_variant() {
            Variant::Integer(i) => Some(i as u32),
            _ => None,
        }
    }

    pub fn to_i32(self) -> Option<i32> {
        match self.into_variant() {
            Variant::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn to_bool(self) -> Option<bool> {
        match self.into_variant() {
            Variant::Atom(atom::TRUE) => Some(true),
            Variant::Atom(atom::FALSE) => Some(false),
            _ => None,
        }
    }

    pub fn to_ref(&self) -> Option<process::Ref> {
        match self.get_boxed_header() {
            Ok(BOXED_REF) => {
                // TODO use ok_or to cast to some, then use ?
                let value = &self.get_boxed_value::<process::Ref>().unwrap();
                Some(**value)
            }
            _ => None,
        }
    }

    // TODO: use std::borrow::Cow<[u8]> to return a copy_bits in the case of unalignment
    pub fn to_bytes(&self) -> Option<&[u8]> {
        match self.get_boxed_header() {
            Ok(BOXED_BINARY) => {
                // TODO use ok_or to cast to some, then use ?
                let value = &self.get_boxed_value::<bitstring::RcBinary>().unwrap();
                Some(&value.data)
            }
            Ok(BOXED_SUBBINARY) => {
                // TODO use ok_or to cast to some, then use ?
                let value = &self.get_boxed_value::<bitstring::SubBinary>().unwrap();

                if value.bit_offset & 7 != 0 {
                    panic!("to_str can't work with non-zero bit_offset");
                }

                let offset = value.offset; // byte_offset!
                Some(&value.original.data[offset..value.size + 1])
            }
            _ => None,
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        match self.get_boxed_header() {
            Ok(BOXED_BINARY) => {
                // TODO use ok_or to cast to some, then use ?
                let value = &self.get_boxed_value::<bitstring::RcBinary>().unwrap();
                // TODO: handle err
                std::str::from_utf8(&value.data).ok()
            }
            Ok(BOXED_SUBBINARY) => {
                // TODO use ok_or to cast to some, then use ?
                let value = &self.get_boxed_value::<bitstring::SubBinary>().unwrap();

                if value.bit_offset & 7 != 0 {
                    panic!("to_str can't work with non-zero bit_offset");
                }

                let offset = value.offset;
                std::str::from_utf8(&value.original.data[offset..value.size + 1]).ok()
            }
            _ => None,
        }
    }

    pub fn boolean(value: bool) -> Self {
        if value {
            return Variant::Atom(atom::TRUE).into();
        }
        Variant::Atom(atom::FALSE).into()
    }

    /// deeply clone the value, copying heap allocated structures onto the new heap.
    /// TODO: implementation without recursion
    pub fn deep_clone(&self, heap: &Heap) -> Self {
        match self.into_variant() {
            Variant::Float(..)
            | Variant::Nil(..)
            | Variant::Integer(..)
            | Variant::Atom(..)
            | Variant::Port(..)
            | Variant::Pid(..) => {
                // immediates
                *self
            }
            Variant::Cons(cons) => {
                let cons = unsafe { &*cons };
                // TODO: badly formed lists

                // TODO: the intermediary collect into vec is not great but can't map without
                // ExactSizeIterator
                let vec: Vec<_> = cons.iter().collect();
                Cons::from_iter(vec.iter().map(|v| v.deep_clone(heap)), heap)
            }
            Variant::Pointer(ptr) => unsafe {
                match *ptr {
                    BOXED_TUPLE => {
                        let tup = &*(ptr as *const Tuple);
                        let new_tuple = self::tuple(heap, tup.len() as u32);
                        for (i, val) in tup.iter().enumerate() {
                            std::ptr::write(&mut new_tuple[i], val.deep_clone(heap));
                        }
                        Term::from(new_tuple)
                    }
                    BOXED_MAP => {
                        let map = &(*(ptr as *const Boxed<map::Map>)).value;
                        // Term::map(heap, map.0.clone()) need to deep_clone kvs
                        let mut new_map = HAMT::new();
                        for (key, value) in map.0.iter() {
                            new_map = new_map.plus(key.deep_clone(heap), value.deep_clone(heap));
                        }
                        Term::map(heap, new_map)
                    }
                    BOXED_EXPORT => {
                        let export = &(*(ptr as *const Boxed<module::MFA>)).value;
                        Term::export(heap, *export)
                    }
                    BOXED_REF => {
                        let reference = &(*(ptr as *const Boxed<process::Ref>)).value;
                        Term::reference(heap, *reference)
                    }
                    BOXED_BIGINT => {
                        let bigint = &(*(ptr as *const Boxed<BigInt>)).value;
                        Term::bigint(heap, bigint.clone())
                    }
                    _ => unimplemented!("deep_clone for {}", self), // TODO: deep clone for Ref<>
                }
            },
        }
    }

    pub fn erl_partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // TODO: loosely compare int and floats
        // non strict comparisons need to handle these + bigint
        // (Variant::Integer(_), Variant::Float(_)) => unimplemented!(),
        // (Variant::Float(_), Variant::Integer(_)) => unimplemented!(),
        Some(self.cmp(other))
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.into_variant().eq(&other.into_variant())
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

            (Variant::Cons(l1), Variant::Cons(l2)) => unsafe { (**l1).eq(&(**l2)) },

            (Variant::Pointer(p1), Variant::Pointer(p2)) => unsafe {
                let header = **p1;
                if header == **p2 {
                    match header {
                        BOXED_TUPLE => {
                            let t1 = &*(*p1 as *const Tuple);
                            let t2 = &*(*p2 as *const Tuple);
                            t1.eq(t2)
                        }
                        BOXED_MAP => {
                            let m1 = &*(*p1 as *const Boxed<Map>);
                            let m2 = &*(*p2 as *const Boxed<Map>);
                            m1.value.eq(&m2.value)
                        }
                        BOXED_CLOSURE => unreachable!(),
                        // TODO: handle other boxed types
                        // ref, bigint, cp, catch, stacktrace, binary, subbinary
                        // TODO: binary and subbinary need to be compared
                        BOXED_BINARY => {
                            let b1 = &*(*p1 as *const Boxed<bitstring::RcBinary>);
                            let b2 = &*(*p2 as *const Boxed<bitstring::RcBinary>);
                            b1.value.data.eq(&b2.value.data)
                        }
                        BOXED_REF => {
                            let r1 = &*(*p1 as *const Boxed<process::Ref>);
                            let r2 = &*(*p2 as *const Boxed<process::Ref>);
                            r1.value.eq(&r2.value)
                        }
                        BOXED_EXPORT => {
                            let e1 = &*(*p1 as *const Boxed<module::MFA>);
                            let e2 = &*(*p2 as *const Boxed<module::MFA>);
                            e1.value.eq(&e2.value)
                        }
                        i => unimplemented!("boxed_value eq for {}", i),
                    }
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Term) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// TODO: make faster by not doing into_variant in some cases
impl Ord for Term {
    fn cmp(&self, other: &Term) -> Ordering {
        // TODO: prevent blowing out the stack from recursion in the future

        // TODO: atom, smallint and float have fastpaths here

        // compare types first, if not equal, we can compare them as raw Type casts
        // else, start comparing immediates
        // allow inexact number comparison

        let t1 = self.get_type();
        let t2 = other.get_type();

        if t1 != t2 {
            // types don't match, use term ordering
            return t1.cmp(&t2);
        }

        // types match, let's keep going
        self.into_variant().cmp(&other.into_variant())
    }
}

impl PartialOrd for Variant {
    fn partial_cmp(&self, other: &Variant) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variant {
    fn cmp(&self, other: &Variant) -> Ordering {
        match (self, other) {
            (Variant::Nil(..), Variant::Nil(..)) => Ordering::Equal,
            (Variant::Integer(i1), Variant::Integer(i2)) => i1.cmp(i2),
            (Variant::Float(f1), Variant::Float(f2)) => f1.partial_cmp(f2).unwrap(),

            (Variant::Atom(a1), Variant::Atom(a2)) => a1.cmp(a2),
            (Variant::Pid(p1), Variant::Pid(p2)) => p1.cmp(p2),
            (Variant::Port(p1), Variant::Port(p2)) => p1.cmp(p2),

            (Variant::Cons(l1), Variant::Cons(l2)) => unsafe { (**l1).cmp(&(**l2)) },

            (Variant::Pointer(p1), Variant::Pointer(p2)) => unsafe {
                let header = **p1;
                if header == **p2 {
                    match header {
                        BOXED_TUPLE => {
                            let t1 = &*(*p1 as *const Tuple);
                            let t2 = &*(*p2 as *const Tuple);
                            t1.cmp(t2)
                        }
                        BOXED_MAP => unimplemented!(),
                        BOXED_CLOSURE => unreachable!(),
                        // TODO: handle other boxed types
                        // ref, bigint, cp, catch, stacktrace, binary, subbinary
                        // TODO: binary and subbinary need to be compared
                        BOXED_BIGINT => {
                            // TODO: bigint with int compare?
                            let i1 = &(*(*p1 as *const Boxed<BigInt>)).value;
                            let i2 = &(*(*p2 as *const Boxed<BigInt>)).value;
                            i1.cmp(i2)
                        }
                        _ => unimplemented!("cmp for {}", header),
                    }
                } else {
                    unimplemented!()
                }
            },
            (Variant::Integer(i1), Variant::Pointer(p2)) => unsafe {
                if **p2 != BOXED_BIGINT {
                    unreachable!()
                }
                let i2 = &(*(*p2 as *const Boxed<BigInt>)).value;
                BigInt::from(*i1).cmp(i2)
            },
            (Variant::Pointer(p1), Variant::Integer(i2)) => unsafe {
                if **p1 != BOXED_BIGINT {
                    unreachable!()
                }
                let i1 = &(*(*p1 as *const Boxed<BigInt>)).value;
                i1.cmp(&BigInt::from(*i2))
            },
            // int and bigint
            _ => unimplemented!("cmp for {:?} and {:?}", self, other),
        }
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.into_variant())
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Nil(..) => write!(f, "[]"),
            Variant::Integer(i) => write!(f, "{}", i),
            Variant::Float(self::Float(i)) => write!(f, "{}", i),
            Variant::Atom(i) => write!(f, ":{}", atom::to_str(*i).unwrap()),
            Variant::Port(i) => write!(f, "#Port<{}>", i),
            Variant::Pid(i) => write!(f, "#Pid<{}>", i),
            Variant::Cons(c) => unsafe {
                let cons = &**c;
                let is_printable = cons.iter().all(|v| match v.into_variant() {
                    Variant::Integer(i) if i > 0 && i <= 255 => {
                        let i = i as u8;
                        // isn't a control char, or is space
                        !(i < b' ' || i >= 127)
                            || (i == b' ' || i == b'\n' || i == b'\t' || i == b'\r')
                    }
                    _ => false,
                });

                if is_printable {
                    write!(f, "\"")?;
                    let string = cons::unicode_list_to_buf(cons, 8096).unwrap();
                    write!(f, "{}", string)?;
                    write!(f, "\"")
                } else {
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
                }
            },
            Variant::Pointer(ptr) => unsafe {
                match **ptr {
                    BOXED_TUPLE => {
                        let t = &*(*ptr as *const Tuple);

                        write!(f, "{{")?;
                        let mut iter = t.iter().peekable();
                        while let Some(val) = iter.next() {
                            write!(f, "{}", val)?;
                            if iter.peek().is_some() {
                                write!(f, ", ")?;
                            }
                        }
                        write!(f, "}}")
                    }
                    BOXED_REF => {
                        let reference = &(*(*ptr as *const Boxed<process::Ref>)).value;
                        write!(f, "#Ref<0.0.0.{}>", reference)
                    }
                    BOXED_BINARY => {
                        // let binary = &(*(*ptr as *const Boxed<bitstring::RcBinary>)).value;
                        // write!(f, "#Binary<{:.40?}>", binary.data) // up to 40 chars
                        write!(f, "#Binary<>")
                    }
                    BOXED_SUBBINARY => write!(f, "#SubBinary<>"),
                    BOXED_MATCHSTATE => write!(f, "#MatchState<>"),
                    BOXED_MAP => {
                        let map = &(*(*ptr as *const Boxed<map::Map>)).value;
                        write!(f, "%{{")?;
                        let mut iter = map.0.iter().peekable();
                        while let Some((key, val)) = iter.next() {
                            write!(f, "{} => {}", key, val)?;
                            if iter.peek().is_some() {
                                write!(f, ", ")?;
                            }
                        }
                        write!(f, "}}")
                    }
                    BOXED_BIGINT => {
                        write!(f, "#BigInt<")?;
                        let ptr = &*(*ptr as *const Boxed<BigInt>);
                        ptr.value.fmt(f)?;
                        write!(f, ">")
                    }
                    BOXED_CLOSURE => {
                        let ptr = &*(*ptr as *const Boxed<Closure>);
                        write!(f, "#Fun<{}>", ptr.value.mfa)
                    }
                    BOXED_CP => {
                        let ptr = &*(*ptr as *const Boxed<Option<InstrPtr>>);
                        write!(f, "CP<{:?}>", ptr.value)
                    }
                    BOXED_CATCH => write!(f, "CATCH"),
                    BOXED_STACKTRACE => write!(f, "STRACE"),
                    BOXED_MODULE => write!(f, "MODULE<>"),
                    BOXED_EXPORT => {
                        let ptr = &*(*ptr as *const Boxed<module::MFA>);
                        write!(f, "&{}", ptr.value)
                    }
                    BOXED_FILE => write!(f, "#File<REF>"),
                    _ => unimplemented!(),
                }
            },
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
    heap.alloc_layout(layout);
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
        assert!(v1.eq(&v2));

        let v3 = cons!(heap, Term::int(1), cons!(heap, Term::int(3), Term::nil()));
        assert!(!v1.eq(&v3));
    }

    #[test]
    fn test_tuple_equality() {
        let heap = &Heap::new();
        let v1 = tup2!(heap, Term::int(1), Term::int(2));
        let v2 = tup2!(heap, Term::int(1), Term::int(2));
        assert!(v1.eq(&v2));

        let v3 = tup3!(heap, Term::int(1), Term::int(1), Term::int(1));
        assert!(!v1.eq(&v3));
    }
}
