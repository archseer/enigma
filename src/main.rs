use nom::*;
use std::str;

fn main() {
    let bytes = include_bytes!("../hello.beam");
    if let ([], res) = scan_beam(bytes).unwrap() {
        let chunk = res.iter().find(|chunk| chunk.name == "Code").unwrap();
        let code = code_chunk(chunk.data).unwrap();
        println!("{:?}", code);

        let chunk = res.iter().find(|chunk| chunk.name == "AtU8").unwrap();
        let code = atom_chunk(chunk.data).unwrap();
        println!("{:?}", code);
    }
}

#[derive(Debug, PartialEq)]
pub struct Chunk<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

named!(
    scan_beam<&[u8], Vec<Chunk>>,
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
    many0!(complete!(chunk))
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

#[derive(Debug)]
pub struct CodeChunk<'a> {
    pub sub_size: u32,
    pub version: u32,
    pub opcode_max: u32,
    pub labels: u32,
    pub functions: u32,
    pub code: &'a [u8],
}
// 4 bytes	sub-size	Length of information fields before code. This allow the header to extended, and an older loader will still know where the code starts.
// 4 bytes	set	Instruction set identifer. Will be 0 for OTP R5. Should be bumped if the instructions are changed in incompatible ways (if the semantics or argument types for an instruction are changed, or if instructions are deleted or renumbered).
// 4 bytes	opcode_max	The highest opcode used in the code section. This allows easy addition of new opcodes. The loader should refuse to load the file if opcode_max is greater than the maximum opcode it knows about.
// 4 bytes	labels	Number of labels (to help loader allocate label table).
// 4 bytes	functions	Number of functions.
// xx bytes	...	Code.

named!(
    code_chunk<&[u8], CodeChunk>,
    do_parse!(
        sub_size: be_u32 >>
        version: be_u32 >>
        opcode_max: be_u32 >>
        labels: be_u32 >>
        functions: be_u32 >>
        take!(sub_size) >>
        code: rest >>
        (CodeChunk { sub_size, version, opcode_max, labels, functions, code })
    )
);

#[derive(Debug)]
pub struct AtomChunk<'a> {
    pub count: u32,
    pub atoms: Vec<&'a str>,
}

named!(
    atom_chunk<&[u8], AtomChunk>,
    do_parse!(
        count: be_u32 >>
        atoms: count!(map_res!(length_bytes!(be_u8), std::str::from_utf8), count as usize) >>
        (AtomChunk { count, atoms })
    )
);

// --------------------

#[derive(Debug)]
pub struct Context {
    // atom table
// export table
// module table
// register table??

// registers
// program pointer/reference?
}
