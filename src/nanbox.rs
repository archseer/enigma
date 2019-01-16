//! Defines the `unsafe_make_nanbox` macro which defines a type which packs values of different types
//! into the unused space of the NaN representation of `f64`.

use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::mem;

const TAG_SHIFT: u64 = 48;
const DOUBLE_MAX_TAG: u32 = 0b11111_11111_11000_0;
const SHIFTED_DOUBLE_MAX_TAG: u64 = ((DOUBLE_MAX_TAG as u64) << TAG_SHIFT) | 0xFFFFFFFF;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NanBox(u64);

impl fmt::Debug for NanBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NanBox {{ tag: {:?}, payload: {:?} }}",
            self.tag(),
            self.0 & ((1 << TAG_SHIFT) - 1)
        )
    }
}

pub trait NanBoxable: Sized {
    unsafe fn from_nan_box(n: NanBox) -> Self;

    fn into_nan_box(self) -> NanBox;

    fn pack_nan_box(self, tag: u8) -> NanBox {
        let mut b = self.into_nan_box();

        let shifted_tag = ((DOUBLE_MAX_TAG as u64) | (tag as u64)) << TAG_SHIFT;
        b.0 |= shifted_tag;
        debug_assert!(b.tag() == u32::from(tag), "{} == {}", b.tag(), tag);
        b
    }

    unsafe fn unpack_nan_box(value: NanBox) -> Self {
        let mask = (1 << TAG_SHIFT) - 1;
        let b = NanBox(value.0 & mask);
        Self::from_nan_box(b)
    }
}

impl NanBoxable for f64 {
    unsafe fn from_nan_box(n: NanBox) -> f64 {
        mem::transmute(n)
    }

    fn into_nan_box(self) -> NanBox {
        unsafe { NanBox(mem::transmute(self)) }
    }

    fn pack_nan_box(self, tag: u8) -> NanBox {
        debug_assert!(tag == 0);
        self.into_nan_box()
    }

    unsafe fn unpack_nan_box(value: NanBox) -> Self {
        Self::from_nan_box(value)
    }
}

macro_rules! impl_cast {
    ($($typ: ident)+) => {
        $(
        impl NanBoxable for $typ {
            unsafe fn from_nan_box(n: NanBox) -> $typ {
                n.0 as $typ
            }

            fn into_nan_box(self) -> NanBox {
                NanBox(self as u64)
            }
        }
        )*
    }
}

impl_cast! { u8 u16 u32 i8 i16 i32 }

impl NanBoxable for char {
    unsafe fn from_nan_box(n: NanBox) -> char {
        std::char::from_u32_unchecked(n.0 as u32)
    }

    fn into_nan_box(self) -> NanBox {
        NanBox(self as u64)
    }
}

impl<'a, T> NanBoxable for &'a T {
    unsafe fn from_nan_box(n: NanBox) -> Self {
        &*(n.0 as *const T)
    }

    fn into_nan_box(self) -> NanBox {
        NanBox(self as *const T as u64)
    }
}

impl<'a, T> NanBoxable for Option<&'a T> {
    unsafe fn from_nan_box(n: NanBox) -> Self {
        (n.0 as *const T).as_ref()
    }

    fn into_nan_box(self) -> NanBox {
        use std::ptr::null;
        (match self {
            Some(p) => p as *const T,
            None => null(),
        })
        .into_nan_box()
    }
}

macro_rules! impl_array {
    ($($typ: ty)+) => {
        $(
        impl NanBoxable for $typ {
            unsafe fn from_nan_box(n: NanBox) -> Self {
                use std::ptr::copy_nonoverlapping;
                use std::mem::size_of;
                debug_assert!(size_of::<Self>() <= 6);
                let mut result = Self::default();
                copy_nonoverlapping(
                    &n as *const NanBox as *const _,
                    result.as_mut_ptr(),
                    result.len());
                result
            }

            fn into_nan_box(self) -> NanBox {
                unsafe {
                    use std::ptr::copy_nonoverlapping;
                    use std::mem::size_of;
                    debug_assert!(size_of::<Self>() <= 6);
                    let mut result = NanBox(0);
                    copy_nonoverlapping(
                        self.as_ptr(),
                        &mut result as *mut NanBox as *mut _,
                        self.len());
                    result
                }
            }
        }
        )*
    }
}

impl_array! { [u8; 1] [u8; 2] [u8; 3] [u8; 4] [u8; 5] [u8; 6] }
impl_array! { [i8; 1] [i8; 2] [i8; 3] [i8; 4] [i8; 5] [i8; 6] }
impl_array! { [i16; 1] [i16; 2] [i16; 3] }
impl_array! { [u16; 1] [u16; 2] [u16; 3] }
impl_array! { [i32; 1] }
impl_array! { [u32; 1] }
impl_array! { [f32; 1] }

macro_rules! impl_cast_t {
    ($param: ident, $($typ: ty)+) => {
        $(
        impl<$param> NanBoxable for $typ {
            unsafe fn from_nan_box(n: NanBox) -> $typ {
                n.0 as $typ
            }

            fn into_nan_box(self) -> NanBox {
                debug_assert!((self as u64) >> TAG_SHIFT == 0);
                NanBox(self as u64)
            }
        }
        )*
    }
}

impl_cast_t! { T, *mut T *const T }

impl NanBox {
    pub unsafe fn new<T>(tag: u8, value: T) -> NanBox
    where
        T: NanBoxable,
    {
        debug_assert!(
            tag < 1 << 4,
            "Nanboxes must have tags smaller than {}",
            1 << 4
        );
        value.pack_nan_box(tag)
    }

    pub unsafe fn unpack<T>(self) -> T
    where
        T: NanBoxable,
    {
        T::unpack_nan_box(self)
    }

    #[inline]
    pub fn tag(self) -> u32 {
        if self.0 <= SHIFTED_DOUBLE_MAX_TAG {
            0
        } else {
            (self.0 >> TAG_SHIFT) as u32 & !DOUBLE_MAX_TAG
        }
    }
}

pub struct TypedNanBox<T> {
    nanbox: NanBox,
    _marker: PhantomData<T>,
}

impl<T> Copy for TypedNanBox<T> where T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + Copy {}

impl<T> Clone for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + Clone,
{
    fn clone(&self) -> Self {
        T::from(TypedNanBox {
            nanbox: self.nanbox,
            _marker: PhantomData,
        })
        //.clone()
        .into()
    }
}

impl<T> fmt::Debug for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", T::from(self.clone()))
    }
}

impl<T> fmt::Display for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + fmt::Display + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", T::from(self.clone()))
    }
}

impl<T> PartialEq for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + PartialEq<T> + Clone,
{
    fn eq(&self, other: &TypedNanBox<T>) -> bool {
        T::from(self.clone()) == T::from(other.clone())
    }
}

impl<T> Eq for TypedNanBox<T> where T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + Eq + Clone {}

impl<T> PartialOrd for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + PartialOrd<T> + Clone,
{
    fn partial_cmp(&self, other: &TypedNanBox<T>) -> Option<Ordering> {
        T::from(self.clone()).partial_cmp(&T::from(other.clone()))
    }
}

impl<T> Ord for TypedNanBox<T>
where
    T: From<TypedNanBox<T>> + Into<TypedNanBox<T>> + Ord + Clone,
{
    fn cmp(&self, other: &TypedNanBox<T>) -> Ordering {
        T::from(self.clone()).cmp(&T::from(other.clone()))
    }
}

impl<T> From<T> for TypedNanBox<T>
where
    T: From<TypedNanBox<T>>,
{
    fn from(value: T) -> TypedNanBox<T> {
        value.into()
    }
}

impl<T> TypedNanBox<T> {
    pub unsafe fn new<U>(tag: u8, value: U) -> TypedNanBox<T>
    where
        U: NanBoxable,
    {
        TypedNanBox {
            nanbox: NanBox::new(tag, value),
            _marker: PhantomData,
        }
    }

    pub unsafe fn unpack<U>(self) -> U
    where
        U: NanBoxable,
    {
        self.nanbox.unpack()
    }

    pub fn tag(&self) -> u32 {
        self.nanbox.tag()
    }
}

/// Creates an `enum` which is packed into the signaling NaN representation of `f64`.
///
/// Some limitations apply to make this work in a safe manner.
///
/// * The first and only the first variant must hold a `f64`.
/// * There must be 8 or fewer variants in the defined enum (this is only checked with
///   `debug_assert!`)
/// * Pointers stored in a nanbox must only use the lower 48 bits (checked via `debug_assert!` only).
///

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod tests {
    use super::*;

    use std::f64;
    use std::fmt;

    use quickcheck::TestResult;

    fn test_eq<T>(l: T, r: T) -> TestResult
    where
        T: PartialEq + fmt::Debug,
    {
        if l == r {
            TestResult::passed()
        } else {
            TestResult::error(format!("{:?} != {:?}", l, r))
        }
    }

    quickcheck! {
        fn nanbox_f64(f: f64) -> TestResult {
            unsafe {
                test_eq(NanBox::new(0, f).unpack(), f)
            }
        }

        fn nanbox_u32(tag: u8, v: u32) -> TestResult {
            if tag == 0 || tag >= 8 {
                return TestResult::discard();
            }
            unsafe {
                TestResult::from_bool(NanBox::new(tag, v).tag() == tag as u32)
            }
        }

        fn nanbox_ptr(tag: u8, v: u32) -> TestResult {
            if tag == 0 || tag >= 8 {
                return TestResult::discard();
            }
            unsafe {
                let nanbox = NanBox::new(tag, Box::into_raw(Box::new(v)));
                TestResult::from_bool(nanbox.tag() == tag as u32)
            }
        }
    }

    unsafe_make_nanbox! {
        #[derive(Clone, Debug, PartialEq)]
        pub enum Value, Variant {
            Float(f64),
            Int(i32),
            Pointer(*mut ()),
            Array([u8; 6])
        }
    }

    #[test]
    fn box_test() {
        assert_eq!(Value::from(123).into_variant(), Variant::Int(123));
        assert_eq!(
            Value::from(3000 as *mut ()).into_variant(),
            Variant::Pointer(3000 as *mut ())
        );
        assert_eq!(Value::from(3.14).into_variant(), Variant::Float(3.14));

        let array = [1, 2, 3, 4, 5, 6];
        assert_eq!(Value::from(array).into_variant(), Variant::Array(array));

        let array = [255, 255, 255, 255, 255, 255];
        assert_eq!(Value::from(array).into_variant(), Variant::Array(array));
    }

    #[test]
    fn nan_box_nan() {
        match Value::from(f64::NAN).into_variant() {
            Variant::Float(x) => assert!(x.is_nan()),
            x => panic!("Unexpected {:?}", x),
        }
    }

    #[should_panic]
    #[test]
    fn invalid_pointer() {
        ((1u64 << TAG_SHIFT) as *const ()).into_nan_box();
    }
}
