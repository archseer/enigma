use crate::atom;
use crate::bitstring;
use crate::immix::Heap;
use crate::module;
use crate::value::{self, Term, HAMT};
use nom::*;
use num::bigint::{BigInt, Sign};
use num::traits::ToPrimitive;

/// External Term Format parser

#[allow(dead_code)]
#[derive(Debug)]
enum Tag {
    NewFloat = 70,
    BitBinary = 77,
    AtomCacheRef_ = 82,
    SmallInteger = 97,
    Integer = 98,
    Float = 99,
    Atom = 100, // deprecated latin-1 ? check orig source
    Reference = 101,
    Port = 102,
    Pid = 103,
    SmallTuple = 104,
    LargeTuple = 105,
    Nil = 106,
    String = 107,
    List = 108,
    Binary = 109,
    SmallBig = 110,
    LargeBig = 111,
    NewFun = 112,
    Export = 113,
    NewReference = 114,
    SmallAtom = 115, // deprecated latin-1
    Map = 116,
    Fun = 117,
    AtomU8 = 118,
    SmallAtomU8 = 119,
}

pub fn decode<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    // starts with  be_u8 that's 131
    let (rest, ver) = be_u8(rest)?;
    assert_eq!(ver, 131, "Expected ETF version number to be 131!");
    decode_value(rest, heap)
}

pub fn decode_value<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    // next be_u8 specifies the type tag
    let (rest, tag) = be_u8(rest)?;
    let tag: Tag = unsafe { ::std::mem::transmute(tag) };

    match tag {
        Tag::NewFloat => {
            let (rest, flt) = be_u64(rest)?;
            Ok((rest, Term::from(f64::from_bits(flt))))
        }
        // TODO:
        // BitBinary
        // AtomCacheRef_
        Tag::SmallInteger => {
            let (rest, int) = be_u8(rest)?;
            // TODO store inside the pointer once we no longer copy
            Ok((rest, Term::int(i32::from(int))))
        }
        Tag::Integer => {
            let (rest, int) = be_i32(rest)?;
            Ok((rest, Term::int(int)))
        }
        // Float: outdated? in favour of NewFloat
        // Reference
        // Port
        // Pid
        Tag::String => decode_string(rest, heap),
        Tag::Binary => decode_binary(rest, heap),
        // NewFun
        Tag::Export => decode_export(rest, heap),
        // NewReference
        // SmallAtom (deprecated?)
        Tag::Map => decode_map(rest, heap),
        // Fun
        // AtomU8
        // SmallAtomU8
        Tag::List => decode_list(rest, heap),
        Tag::Atom => decode_atom(rest),
        Tag::Nil => Ok((rest, Term::nil())),
        Tag::SmallTuple => {
            let (rest, size) = be_u8(rest)?;
            decode_tuple(rest, u32::from(size), heap)
        }
        Tag::LargeTuple => {
            let (rest, size) = be_u32(rest)?;
            decode_tuple(rest, size, heap)
        }
        Tag::SmallBig => {
            let (rest, size) = be_u8(rest)?;
            decode_bignum(rest, size.into(), heap)
        }
        Tag::LargeBig => {
            let (rest, size) = be_u32(rest)?;
            decode_bignum(rest, size, heap)
        }

        _ => unimplemented!("etf: {:?}", tag),
    }
}

pub fn decode_atom(rest: &[u8]) -> IResult<&[u8], Term> {
    let (rest, len) = be_u16(rest)?;
    let (rest, string) = take_str!(rest, len)?;

    // TODO: create atom &string
    Ok((rest, Term::atom(atom::from_str(string))))
}

pub fn decode_tuple<'a>(rest: &'a [u8], len: u32, heap: &Heap) -> IResult<&'a [u8], Term> {
    // alloc space for elements
    let tuple = value::tuple(heap, len);

    // TODO: nested tuples are dropped and then segfault, prevent that!
    let rest = (0..len).fold(rest, |rest, i| {
        let (rest, el) = decode_value(rest, heap).unwrap();
        // use ptr write to avoid dropping uninitialized values!
        unsafe {
            std::ptr::write(&mut tuple[i as usize], el);
        }
        rest
    });

    Ok((rest, tuple.into()))
}

pub fn decode_list<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, len) = be_u32(rest)?;

    unsafe {
        let (rest, val) = decode_value(rest, heap)?;

        let start = heap.alloc(value::Cons {
            head: val,
            tail: Term::nil(),
        });

        let (tail, rest) =
            (0..len - 1).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
                let value::Cons { ref mut tail, .. } = *cons;
                let (rest, val) = decode_value(rest, heap).unwrap();
                let new_cons = heap.alloc(value::Cons {
                    head: val,
                    tail: Term::nil(),
                });
                let ptr = new_cons as *mut value::Cons;
                std::mem::replace(&mut *tail, Term::from(new_cons));
                (ptr, rest)
            });

        // set the tail
        let (rest, val) = decode_value(rest, heap).unwrap();
        (*tail).tail = val;

        Ok((rest, Term::from(start)))
    }
}

pub fn decode_map<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (mut new_rest, len) = be_u32(rest)?;
    let mut map = HAMT::new();

    for _i in 0..len {
        let (rest, key) = decode_value(new_rest, heap)?;
        let (rest, val) = decode_value(rest, heap)?;

        map = map.plus(key, val);
        new_rest = rest;
    }
    Ok((new_rest, Term::map(heap, map)))
}

/// A string of bytes encoded as tag 107 (String) with 16-bit length.
/// This is basically a list, but it's optimized to decode to char.
pub fn decode_string<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, len) = be_u16(rest)?;
    if len == 0 {
        return Ok((rest, Term::nil()));
    }

    unsafe {
        let (rest, elem) = be_u8(rest)?;

        let start = heap.alloc(value::Cons {
            head: Term::int(i32::from(elem)),
            tail: Term::nil(),
        });

        let (tail, rest) =
            (0..len - 1).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
                let value::Cons { ref mut tail, .. } = *cons;
                let (rest, elem) = be_u8(rest).unwrap();

                let new_cons = heap.alloc(value::Cons {
                    head: Term::int(i32::from(elem)),
                    tail: Term::nil(),
                });

                let ptr = new_cons as *mut value::Cons;
                std::mem::replace(&mut *tail, Term::from(new_cons));
                (ptr, rest)
            });

        // set the tail
        (*tail).tail = Term::nil();

        Ok((rest, Term::from(start)))
    }
}

pub fn decode_binary<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, len) = be_u32(rest)?;
    if len == 0 {
        return Ok((rest, Term::binary(heap, bitstring::Binary::new())));
    }

    let (rest, bytes) = take!(rest, len)?;
    Ok((rest, Term::binary(heap, bitstring::Binary::from(bytes))))
}

pub fn decode_export<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, m) = decode_value(rest, heap)?;
    let (rest, f) = decode_value(rest, heap)?;
    let (rest, a) = decode_value(rest, heap)?;

    Ok((
        rest,
        Term::export(heap, module::MFA(m.to_u32(), f.to_u32(), a.to_u32())),
    ))
}

#[cfg(target_pointer_width = "32")]
pub const WORD_BITS: usize = 32;

#[cfg(target_pointer_width = "64")]
pub const WORD_BITS: usize = 64;

pub fn decode_bignum<'a>(rest: &'a [u8], size: u32, heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, sign) = be_u8(rest)?;

    let sign = if sign == 0 { Sign::Plus } else { Sign::Minus };

    let (rest, digits) = take!(rest, size)?;
    let big = BigInt::from_bytes_le(sign, digits);

    // Assert that the number fits into small
    if let Some(b_signed) = big.to_i32() {
        return Ok((rest, Term::int(b_signed)));
    }

    Ok((rest, Term::bigint(heap, big)))
}
