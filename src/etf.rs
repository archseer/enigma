use crate::arc_without_weak::ArcWithoutWeak;
use crate::atom;
use crate::immix::Heap;
use crate::value::{self, Value, HAMT};
use nom::*;
use num::traits::ToPrimitive;
use num_bigint::{BigInt, Sign};

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

pub fn decode<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    // starts with  be_u8 that's 131
    let (rest, ver) = be_u8(rest)?;
    assert_eq!(ver, 131, "Expected ETF version number to be 131!");
    decode_value(rest, heap)
}

pub fn decode_value<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    // next be_u8 specifies the type tag
    let (rest, tag) = be_u8(rest)?;
    let tag: Tag = unsafe { ::std::mem::transmute(tag) };

    match tag {
        Tag::NewFloat => {
            let (rest, flt) = be_u64(rest)?;
            Ok((rest, Value::Float(value::Float(f64::from_bits(flt)))))
        }
        // TODO:
        // BitBinary
        // AtomCacheRef_
        Tag::SmallInteger => {
            let (rest, int) = be_u8(rest)?;
            // TODO store inside the pointer once we no longer copy
            Ok((rest, Value::Integer(i64::from(int))))
        }
        Tag::Integer => {
            let (rest, int) = be_i32(rest)?;
            Ok((rest, Value::Integer(i64::from(int))))
        }
        // Float: outdated? in favour of NewFloat
        // Reference
        // Port
        // Pid
        Tag::String => decode_string(rest, heap),
        Tag::Binary => decode_binary(rest, heap),
        // NewFun
        // Export
        // NewReference
        // SmallAtom (deprecated?)
        Tag::Map => decode_map(rest, heap),
        // Fun
        // AtomU8
        // SmallAtomU8
        Tag::List => decode_list(rest, heap),
        Tag::Atom => decode_atom(rest),
        Tag::Nil => Ok((rest, Value::Nil)),
        Tag::SmallTuple => {
            let (rest, size) = be_u8(rest)?;
            decode_tuple(rest, size as usize, heap)
        }
        Tag::LargeTuple => {
            let (rest, size) = be_u32(rest)?;
            decode_tuple(rest, size as usize, heap)
        }
        Tag::SmallBig => {
            let (rest, size) = be_u8(rest)?;
            decode_bignum(rest, size as usize)
        }
        Tag::LargeBig => {
            let (rest, size) = be_u32(rest)?;
            decode_bignum(rest, size as usize)
        }

        _ => unimplemented!("etf: {:?}", tag),
    }
}

pub fn decode_atom(rest: &[u8]) -> IResult<&[u8], Value> {
    let (rest, len) = be_u16(rest)?;
    let (rest, string) = take_str!(rest, len)?;

    // TODO: create atom &string
    Ok((rest, Value::Atom(atom::from_str(string))))
}

pub fn decode_tuple<'a>(rest: &'a [u8], len: usize, heap: &Heap) -> IResult<&'a [u8], Value> {
    // alloc space for elements
    let tuple = value::tuple(heap, len);

    let rest = (0..len).fold(rest, |rest, i| {
        let (rest, el) = decode_value(rest, heap).unwrap();
        tuple[i] = el;
        rest
    });

    Ok((rest, Value::Tuple(tuple)))
}

pub fn decode_list<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    let (rest, len) = be_u32(rest)?;

    unsafe {
        let (rest, val) = decode_value(rest, heap)?;

        let start = heap.alloc(value::Cons {
            head: val,
            tail: Value::Nil,
        });

        let (tail, rest) =
            (0..len - 1).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
                let value::Cons { ref mut tail, .. } = *cons;
                let (rest, val) = decode_value(rest, heap).unwrap();
                let new_cons = heap.alloc(value::Cons {
                    head: val,
                    tail: Value::Nil,
                });
                std::mem::replace(&mut *tail, Value::List(new_cons));
                (new_cons as *mut value::Cons, rest)
            });

        // set the tail
        let (rest, val) = decode_value(rest, heap).unwrap();
        (*tail).tail = val;

        Ok((rest, Value::List(start)))
    }
}

pub fn decode_map<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    let (mut new_rest, len) = be_u32(rest)?;
    let mut map = HAMT::new();

    for _i in 0..len {
        let (rest, key) = decode_value(new_rest, heap)?;
        println!("{:?}", key);
        let (rest, val) = decode_value(rest, heap)?;
        println!("{:?}", val);

        map = map.plus(key, val);
        new_rest = rest;
    }
    Ok((new_rest, Value::Map(value::Map(ArcWithoutWeak::new(map)))))
}

/// A string of bytes encoded as tag 107 (String) with 16-bit length.
/// This is basically a list, but it's optimized to decode to char.
pub fn decode_string<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Value> {
    let (rest, len) = be_u16(rest)?;
    if len == 0 {
        return Ok((rest, Value::Nil));
    }

    unsafe {
        let (rest, elem) = be_u8(rest)?;

        let start = heap.alloc(value::Cons {
            head: Value::Character(elem),
            tail: Value::Nil,
        });

        let (tail, rest) =
            (0..len - 1).fold((start as *mut value::Cons, rest), |(cons, rest), _i| {
                let value::Cons { ref mut tail, .. } = *cons;
                let (rest, elem) = be_u8(rest).unwrap();

                let new_cons = heap.alloc(value::Cons {
                    head: Value::Character(elem),
                    tail: Value::Nil,
                });

                std::mem::replace(&mut *tail, Value::List(new_cons as *const value::Cons));
                (new_cons as *mut value::Cons, rest)
            });

        // set the tail
        (*tail).tail = Value::Nil;

        Ok((rest, Value::List(start)))
    }
}

pub fn decode_binary<'a>(rest: &'a [u8], _heap: &Heap) -> IResult<&'a [u8], Value> {
    let (rest, len) = be_u32(rest)?;
    if len == 0 {
        return Ok((rest, Value::Binary(ArcWithoutWeak::new(Vec::new()))));
    }

    let (rest, bytes) = take!(rest, len)?;
    Ok((rest, Value::Binary(ArcWithoutWeak::new(bytes.to_vec()))))
}

#[cfg(target_pointer_width = "32")]
pub const WORD_BITS: usize = 32;

#[cfg(target_pointer_width = "64")]
pub const WORD_BITS: usize = 64;

pub fn decode_bignum(rest: &[u8], size: usize) -> IResult<&[u8], Value> {
    let (rest, sign) = be_u8(rest)?;

    let sign = if sign == 0 { Sign::Plus } else { Sign::Minus };

    let (rest, digits) = take!(rest, size)?;
    let big = BigInt::from_bytes_le(sign, digits);

    // Assert that the number fits into small
    if big.bits() < WORD_BITS - 4 {
        let b_signed = big.to_isize().unwrap();
        return Ok((rest, Value::Integer(b_signed as i64)));
    }

    Ok((rest, Value::BigInt(Box::new(big))))
}
