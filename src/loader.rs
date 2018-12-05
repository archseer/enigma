use crate::opcodes::*;
use crate::vm::Machine;
use nom::*;
use num_bigint::{BigInt, Sign};

pub struct Loader<'a> {
    pub vm: &'a Machine,
}

impl<'a> Loader<'a> {
    pub fn load_file(&self, bytes: &'a [u8]) -> Result<CodeChunk, nom::Err<&[u8]>> {
        let (_, res) = scan_beam(bytes).unwrap();

        let names: Vec<_> = res.iter().map(|chunk| chunk.name).collect();
        println!("{:?}", names);

        let chunk = res.iter().find(|chunk| chunk.name == "Code").unwrap();
        let code = code_chunk(chunk.data)?;
        println!("{:?}", code);

        let chunk = res.iter().find(|chunk| chunk.name == "AtU8").unwrap();
        let data = atom_chunk(chunk.data)?;
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "LocT").unwrap();
        let data = loct_chunk(chunk.data)?;
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "ImpT").unwrap();
        let data = loct_chunk(chunk.data)?;
        println!("{:?}", data);

        let chunk = res.iter().find(|chunk| chunk.name == "ExpT").unwrap();
        let data = loct_chunk(chunk.data)?;
        println!("{:?}", data);

        // parse all the chunks
        // load all the atoms, lambda funcs and literals into the VM and store the vm vals
        // parse the instructions, swapping for global vals
        // - skip line
        // - store labels as offsets
        // - patch jump instructions to labels (store patches if label wasn't seen yet)
        // - make imports work via pointers..
        println!("{:?}", code.1);

        return Ok(code.1);
    }
}

#[derive(Debug, PartialEq)]
pub struct Chunk<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

named!(
    pub scan_beam<&[u8], Vec<Chunk>>,
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
        //take!(sub_size) >>
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
    Literal(u8),
    Integer(u64),
    Atom(u8),
    X(u8),
    Y(u8),
    Label(u8),
    Character(u8),
    // Extended,
    Float(),
    List(),
    FloatReg(),
    AllocList(),
    ExtendedLiteral(),
}

fn compact_term(i: &[u8]) -> IResult<&[u8], Term> {
    let (rest, b) = be_u8(i)?;
    let tag = b & 0b111;

    println!("b: {}, tag: {}, {}", b, tag, tag < 0b111);
    if tag < 0b111 {
        // it's not extended
        let (val, rest) = if 0 == (b & 0b1000) {
            // Bit 3 is 0 marks that 4 following bits contain the value
            (b >> 4, rest)
        } else {
            // Bit 3 is 1, but...
            if 0 == (b & 0b1_0000) {
                // Bit 4 is 0, marks that the following 3 bits (most significant) and
                // the following byte (least significant) will contain the 11-bit value
                let (rest, r) = be_u8(rest)?;
                (((b & 0b1110_0000) << 3 | r), rest)
            } else {
                // Bit 4 is 1 means that bits 5-6-7 contain amount of bytes+2 to store
                // the value
                let mut n_bytes = (b >> 5) + 2;
                if n_bytes == 9 {
                    println!("more than 9!")
                    //     // bytes=9 means upper 5 bits were set to 1, special case 0b11111xxx
                    //     // which means that following nested tagged value encodes size,
                    //     // followed by the bytes (Size+9)
                    //     let bnext = r.read_u8();
                    //     if let Integral::Small(tmp) = read_word(bnext, r) {
                    //       n_bytes = tmp as Word + 9;
                    //     } else {
                    //       panic!("{}read word encountered a wrong byte length", module())
                    //     }
                }

                // Read the remaining big endian bytes and convert to int
                let (rest, long_bytes) = take!(rest, n_bytes)?;
                let sign = if long_bytes[0] & 0x80 == 0x80 {
                    Sign::Minus
                } else {
                    Sign::Plus
                };

                let r = BigInt::from_bytes_be(sign, &long_bytes);
                println!("{}", r);
                //Integral::from_big(r)
                (23, rest)
            } // if larger than 11 bits
        };

        return match tag {
            0 => Ok((rest, Term::Literal(val))),
            1 => Ok((rest, Term::Integer(val as u64))),
            2 => Ok((rest, Term::Atom(val))),
            3 => Ok((rest, Term::X(val))),
            4 => Ok((rest, Term::Y(val))),
            5 => Ok((rest, Term::Label(val))),
            6 => Ok((rest, Term::Character(val))),
            _ => panic!("can't happen"),
        };
    }

    // extended_term()
    Ok((rest, Term::Integer(b as u64)))
}

#[derive(Debug)]
pub struct Instruction {
    pub op: Opcode,
    pub args: Vec<Term>,
}

named!(
    pub scan_instructions<&[u8], Vec<Instruction>>,
    many0!(complete!(scan_instruction))
);

// apply compact_term, arity times (keep an arity table)
named!(
    scan_instruction<&[u8], Instruction>,
    do_parse!(
        op: be_u8 >>
        args: count!(compact_term, opcode_arity(op)) >>
        (Instruction { op: to_opcode(op), args })
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
