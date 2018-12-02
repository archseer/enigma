// use std::io;
// use std::fs;
use std::str;
use nom::*;
use nom::dbg;

fn main() {
    let bytes = include_bytes!("../hello.beam");
    let res = scan_beam(bytes).unwrap();
    println!("{:?}", res);
    //println!("{:?}", bytes);
}

// fn parse_beam() -> Result<Vec<u8>, io::Error> {
//     let s = fs::read("hello.beam")?;
//     Ok(s)
// }

#[derive(Debug, PartialEq)]

// BEAMChunk = <<
// ChunkName:4/unit:8,           // "Code", "Atom", "StrT", "LitT", ...
// ChunkSize:32/big,
// ChunkData:ChunkSize/unit:8,   // data format is defined by ChunkName
// Padding4:0..3/unit:8
// >>

pub struct Chunk<'a> {
    pub name: &'a str,//[u32; 4],
    pub data: &'a [u8],
}

named!( scan_beam<&[u8], Vec<Chunk>>,
    do_parse!(
        tag!("FOR1") >>
        _size: le_u32 >>
        tag!("BEAM") >>
        data: chunks >>
        (data)
    )
);

named!(
    chunks<&[u8], Vec<Chunk>>,
    many0!(chunk)
);

named!(
    chunk<&[u8], Chunk>,
     do_parse!(
        name: map_res!(take!(4), std::str::from_utf8) >>
        size: be_u32 >>
        bytes: take!(size) >>
        take!(align_bytes(size)) >>
        (Chunk { name, data: bytes })
    )
);

fn align_bytes(size: u32) -> u32 {
    let rem = size % 4;
    if rem == 0 {
        0
    } else {
        4 - rem
    }
}
