use crate::value::Value;
use nom::*;

/// External Term Format parser

#[derive(Debug)]
enum Tag {
    ETF = 131,
    NewFloat = 70,
    BitBinary = 77,
    AtomCacheRef_ = 82,
    SmallInteger = 97,
    Integer = 98,
    Float = 99,
    Atom = 100, // deprecated latin-1 ?
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

pub fn decode(rest: &[u8]) -> IResult<&[u8], Value> {
    // starts with  be_u8 that's 131
    let (rest, ver) = be_u8(rest)?;
    assert_eq!(ver, 131, "Expected ETF version number to be 131!");
    decode_value(rest)
}

pub fn decode_value(rest: &[u8]) -> IResult<&[u8], Value> {
    // next be_u8 specifies the type tag
    let (rest, tag) = be_u8(rest)?;
    println!("decode_value {:?}", tag);
    let tag: Tag = unsafe { ::std::mem::transmute(tag) };
    println!("here!");

    match tag {
        Tag::List => decode_list(rest),
        Tag::Atom => decode_atom(rest),
        Tag::Nil => Ok((rest, Value::None())),
        _ => panic!("Tag is {:?}", tag),
    }
}

pub fn decode_atom(rest: &[u8]) -> IResult<&[u8], Value> {
    println!("decode_atom");
    let (rest, size) = be_u16(rest)?;
    let (rest, string) = take_str!(rest, size)?;

    println!("{:?}", string);
    // TODO: create atom &string
    Ok((rest, Value::Atom(1)))
}

pub fn decode_list(rest: &[u8]) -> IResult<&[u8], Value> {
    println!("decode_list");
    let (rest, len) = be_u32(rest)?;

    // TODO: use alloc
    let mut start = Value::Cons {
        head: Box::new(Value::None()),
        tail: Box::new(Value::None()),
    };

    let (tail, rest) = (0..len).fold((&mut start, rest), |(cons, buf), i| {
        // TODO: probably doing something wrong here
        if let &mut Value::Cons {
            ref mut head,
            ref mut tail,
        } = cons
        {
            let (rest, val) = decode_value(buf).unwrap();
            let new_cons = Value::Cons {
                head: Box::new(Value::None()),
                tail: Box::new(Value::None()),
            };
            std::mem::replace(&mut *head, Box::new(val));
            std::mem::replace(&mut *tail, Box::new(new_cons));
            return (tail, rest);
        }
        panic!("Wrong value!")
    });

    // set the tail
    let (rest, val) = decode_value(rest).unwrap();
    std::mem::replace(&mut *tail, val);

    return Ok((rest, start));
}
