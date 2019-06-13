use crate::atom;
use crate::bitstring;
use crate::immix::Heap;
use crate::module;
use crate::value::{self, Term, Variant, HAMT};
use nom::*;
use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;

/// External Term Format parser

#[allow(dead_code)]
#[derive(Debug)]
enum Tag {
    NewFloat = 70,
    BitBinary = 77,
    AtomCacheRef_ = 82,
    NewPid = 88,
    NewPort = 89,
    NewerReferenceExt = 90,
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

pub fn decode<'a>(bytes: &'a [u8], heap: &Heap) -> Term {
    // starts with  be_u8 that's 131
    let (rest, ver) = be_u8(bytes).unwrap();
    assert_eq!(ver, 131, "Expected ETF version number to be 131!");

    // check if followed by 80+size, then decode
    // The compressed term format is as follows:
    // 1	1	4	N
    // 131	80	UncompressedSize	Zlib-compressedData

    if rest[0] == 80 {
        use libflate::zlib;
        use std::io::Read;
        let (rest, _) = be_u8(rest).unwrap();
        let (rest, size) = be_u32(rest).unwrap();

        let mut data = Vec::with_capacity(size as usize);

        // let iocursor = std::io::Cursor::new(rest);
        zlib::Decoder::new(rest)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();

        let (_, term) = decode_value(&data, heap).unwrap();
        term
    } else {
        let (_, term) = decode_value(rest, heap).unwrap();
        term
    }
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
        Tag::BitBinary => decode_bitstring(rest, heap),
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

        map.insert(key, val);
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

pub fn decode_bitstring<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, len) = be_u32(rest)?;
    let (rest, bits) = be_u8(rest)?;
    if len == 0 && bits == 0 {
        return Ok((rest, Term::binary(heap, bitstring::Binary::new())));
    }

    if bits > 8 {
        panic!("Tag::BitBinary with invalid len bits");
    }

    let (rest, bytes) = take!(rest, len)?;
    let bin = crate::servo_arc::Arc::new(bitstring::Binary::from(bytes));
    let num_bits = len as usize * 8 + bits as usize;
    Ok((
        rest,
        Term::subbinary(heap, bitstring::SubBinary::new(bin, num_bits, 0, false)),
    ))
}

pub fn decode_export<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Term> {
    let (rest, m) = decode_value(rest, heap)?;
    let (rest, f) = decode_value(rest, heap)?;
    let (rest, a) = decode_value(rest, heap)?;

    Ok((
        rest,
        Term::export(
            heap,
            module::MFA(
                m.to_atom().unwrap(),
                f.to_atom().unwrap(),
                a.to_uint().unwrap(),
            ),
        ),
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

// ----
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

pub fn encode(term: Term) -> std::io::Result<Vec<u8>> {
    let mut res = vec![131]; // version number
    encode_term(&mut res, term)?;
    // :debug_info_v1, :erl_abstract_code, {:none, []}}
    Ok(res)
}

pub fn encode_term(res: &mut Vec<u8>, term: Term) -> std::io::Result<()> {
    use value::{CastFrom, Cons, Tuple};

    match term.into_variant() {
        Variant::Integer(i) => {
            if 0 <= i && i <= std::u8::MAX as i32 {
                res.write_u8(Tag::SmallInteger as u8)?;
                res.write_u8(i as u8)?;
            } else {
                res.write_u8(Tag::Integer as u8)?;
                res.write_i32::<BigEndian>(i)?;
            }
        }
        Variant::Nil(..) => {
            res.write_u8(Tag::Nil as u8)?;
        }
        Variant::Atom(i) => {
            let atom = atom::to_str(i).unwrap();
            encode_atom(res, atom)?;
        }
        Variant::Float(value::Float(f)) => encode_float(res, f)?,
        Variant::Cons(..) => encode_list(res, Cons::cast_from(&term).unwrap())?,
        // encode list
        // encode improper list
        Variant::Pointer(_ptr) => match term.get_boxed_header().unwrap() {
            value::BOXED_TUPLE => encode_tuple(res, Tuple::cast_from(&term).unwrap())?,
            value::BOXED_BINARY => {
                encode_binary(res, bitstring::RcBinary::cast_from(&term).unwrap())?
            }
            value::BOXED_BIGINT => {
                let value = &term.get_boxed_value::<BigInt>().unwrap();
                encode_bigint(res, value)?
            }
            i => unimplemented!("etf::encode for boxed {}", i),
        },
        _ => unimplemented!("etf::encode for: {}", term),
    }
    Ok(())
}

fn encode_nil(res: &mut Vec<u8>) -> std::io::Result<()> {
    res.write_u8(Tag::Nil as u8)
}

fn encode_tuple(res: &mut Vec<u8>, tuple: &value::Tuple) -> std::io::Result<()> {
    if tuple.len() < 0x100 {
        res.write_u8(Tag::SmallTuple as u8)?;
        res.write_u8(tuple.len() as u8)?;
    } else {
        res.write_u8(Tag::LargeTuple as u8)?;
        res.write_u32::<BigEndian>(tuple.len() as u32)?;
    }
    for e in tuple.iter().copied() {
        encode_term(res, e)?;
    }
    Ok(())
}

fn encode_atom(res: &mut Vec<u8>, atom: String) -> std::io::Result<()> {
    if atom.len() > 0xFFFF {
        // return Err(EncodeError::TooLongAtomName(atom));
        panic!("Atom name too long!");
    }
    res.write_u8(Tag::AtomU8 as u8)?;
    res.write_u16::<BigEndian>(atom.len() as u16)?;
    res.write_all(atom.as_bytes())?;
    Ok(())
}

fn encode_float(res: &mut Vec<u8>, float: f64) -> std::io::Result<()> {
    res.write_u8(Tag::NewFloat as u8)?;
    res.write_f64::<BigEndian>(float)?;
    Ok(())
}

fn encode_binary(res: &mut Vec<u8>, binary: &bitstring::RcBinary) -> std::io::Result<()> {
    unimplemented!()
}

fn encode_list(res: &mut Vec<u8>, list: &value::Cons) -> std::io::Result<()> {
    let len = list.iter().count();

    // TODO FIXME: handle improper lists

    if len > 0
        && len <= std::u16::MAX as usize
        && list.iter().all(|e| match e.to_int() {
            Some(i) if i >= 0 && i <= 0x100 => true,
            _ => false,
        })
    {
        res.write_u8(Tag::String as u8)?;
        res.write_u16::<BigEndian>(len as u16)?;
        for b in list.iter().map(|e| e.to_int().unwrap()) {
            res.write_u8(b as u8)?;
        }
    } else {
        res.write_u8(Tag::List as u8)?;
        res.write_u32::<BigEndian>(len as u32)?;
        for e in list.iter().copied() {
            encode_term(res, e)?;
        }
        encode_nil(res)?;
    }
    Ok(())
}

fn encode_bigint(res: &mut Vec<u8>, bigint: &BigInt) -> std::io::Result<()> {
    let (sign, bytes) = bigint.to_bytes_le();
    if bytes.len() <= std::u8::MAX as usize {
        res.write_u8(Tag::SmallBig as u8)?;
        res.write_u8(bytes.len() as u8)?;
    } else if bytes.len() <= std::u32::MAX as usize {
        res.write_u8(Tag::LargeBig as u8)?;
        res.write_u32::<BigEndian>(bytes.len() as u32)?;
    } else {
        panic!("BigInt too large!");
    }
    let sign = if sign == Sign::Plus { 0 } else { 1 };
    res.write_u8(sign)?;
    res.write_all(&bytes)?;
    Ok(())
}
