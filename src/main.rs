mod opcodes;
use crate::opcodes::*;
use nom::*;
use std::str;

fn main() {
    let bytes = include_bytes!("../hello.beam");
    if let ([], res) = scan_beam(bytes).unwrap() {
        let names: Vec<_> = res.iter().map(|chunk| chunk.name).collect();
        println!("{:?}", names);

        let chunk = res.iter().find(|chunk| chunk.name == "Code").unwrap();
        let code = code_chunk(chunk.data).unwrap();
        println!("{:?}", code);

        let chunk = res.iter().find(|chunk| chunk.name == "AtU8").unwrap();
        let data = atom_chunk(chunk.data).unwrap();
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "LocT").unwrap();
        let data = loct_chunk(chunk.data).unwrap();
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "ImpT").unwrap();
        let data = loct_chunk(chunk.data).unwrap();
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "ExpT").unwrap();
        let data = loct_chunk(chunk.data).unwrap();
        println!("{:?}", data);

        let code = code.1.code;
        // TODO: scan over the Code chunk bits, but we'll need to know
        // bit length of each instruction.
        let mut pc = 0;
        loop {
            //let opcode = to_opcode(code[pc]);
            let (_, (opcode, args)) = scan_instruction(code).unwrap();
            pc = pc + 1;
            println!("op: {:?}, args: {:?}", opcode, args);

            // apply compact_term, arity times (keep an arity table)
            let (_, res) = scan_instructions(code).unwrap();
            println!("res: {:?}", res);

            match opcode {
                Opcode::Line => {
                    // one operand, Integer
                    break;
                }
                _ => println!("Unimplemented opcode {:?}", opcode),
            }
        }
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
    pub sub_size: u32,   // prefixed extra field data
    pub version: u32,    // should be 0
    pub opcode_max: u32, // highest opcode used for versioning
    pub labels: u32,     // number of labels
    pub functions: u32,  // number of functions
    pub code: &'a [u8],
}

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

type ErlFun = (u32, u32, u32);

named!(
    fun_entry<&[u8], ErlFun>,
    do_parse!(
        function: be_u32 >>
        arity: be_u32 >>
        label: be_u32 >>
        (function, arity, label)
    )
);

#[derive(Debug)]
pub struct FunTableChunk {
    pub count: u32,
    pub entries: Vec<ErlFun>,
}

named!(
    loct_chunk<&[u8], FunTableChunk>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (FunTableChunk { count, entries })
    )
);

// "StrT", "ImpT", "ExpT", "LocT", "Attr", "CInf", "Dbgi", "Line"

// It can be Literal=0, Integer=1, Atom=2, XRegister=3, YRegister=4, Label=5, Character=6, Extended=7.
// If the base tag was Extended=7, then bits 4-5-6-7 PLUS 7 will become the extended tag.
// It can have values Float=8, List=9, FloatReg=10, AllocList=11, Literal=12.

#[derive(Debug)]
pub enum Term {
    Literal(),
    Integer(),
    Atom(),
    X(),
    Y(),
    Label(),
    Character(),
    // Extended,
    Float(),
    List(),
    FloatReg(),
    AllocList(),
    ExtendedLiteral(),
}

fn compact_term(i: &[u8]) -> IResult<&[u8], u8> {
    let (rest, b) = be_u8(i)?;
    let tag = b & 0b111;

    if tag < 0b111 {
        // it's not extended
        if 0 == (b & 0b1000) {
            // Bit 3 is 0 marks that 4 following bits contain the value
            return Ok((rest, b >> 4));
        }
        // if bit 3 was 1,
        // Bit 4 is 0, marks that the following 3 bits (most significant) and
        // the following byte (least significant) will contain the 11-bit value
        //
        // Bit 4 is 1 means that bits 5-6-7 contain amount of bytes+2 to store
        // the value

        return Ok((rest, 1));
    }

    // extended_term()
    Ok((rest, b))
}

named!(
    scan_instructions<&[u8], Vec<(Opcode, Vec<u8>)>>,
    many0!(complete!(scan_instruction))
);

named!(
    scan_instruction<&[u8], (Opcode, Vec<u8>)>,
    do_parse!(
        op: be_u8 >>
        term: count!(compact_term, opcode_arity(op)) >>
        ((to_opcode(op), term))
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compact_term() {
        assert_eq!(
            compact_term(&vec![0b10010000u8]),
            Ok((&[] as &[u8], 9 as u8))
        );
        assert_eq!(
            compact_term(&vec![0b11110000u8]),
            Ok((&[] as &[u8], 15 as u8))
        );
    }
}

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
