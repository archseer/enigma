use crate::atom::{self, ATOMS};
use crate::bitstring;
use crate::etf;
use crate::immix::Heap;
use crate::module::{Lambda, Module, MFA};
use crate::opcodes::*;
use crate::servo_arc::Arc;
use crate::value::Term;
use hashbrown::HashMap;
use libflate::zlib;
use nom::*;
use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;
use std::io::{Cursor, Read};

// (filename_index, loc)
pub type FuncInfo = (u32, u32);

/// Declares a location as invalid.
pub const LINE_INVALID_LOCATION: usize = 0;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    /// position inside the bytecode
    pub pos: usize,
    /// line number inside the original file
    pub loc: usize,
}

pub struct Loader<'a> {
    atoms: Vec<&'a str>,
    imports: Vec<MFA>,
    exports: Vec<MFA>,
    literals: Vec<Term>,
    strings: Vec<u8>,
    lambdas: Vec<Lambda>,
    atom_map: HashMap<u32, u32>, // TODO: remove this; local id -> global id
    funs: HashMap<(u32, u32), u32>, // (fun name as atom, arity) -> offset
    labels: HashMap<u32, u32>,   // label -> offset
    line_items: Vec<FuncInfo>,
    lines: Vec<Line>,
    file_names: Vec<&'a str>,
    code: &'a [u8],
    literal_heap: Heap,
    instructions: Vec<Instruction>,
    on_load: Option<u32>,
}

/// Compact term encoding values. BEAM does some tricks to be able to share the memory layout with
/// regular values, but for the most part we don't need this (it also doesn't fit nanboxed values
/// well).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LValue {
    // TODO: these dupe LTerm values
    Atom(u32),
    Integer(i32),
    Character(u8),
    Nil,
    Binary(Arc<bitstring::Binary>),
    Str(Vec<u8>), // TODO this is so avoid constructing a full binary
    BigInt(BigInt),
    //
    Literal(u32),
    X(u32),
    Y(u32),
    Label(u32),
    ExtendedList(Box<Vec<LValue>>),
    FloatReg(u32),
    AllocList(Box<Vec<(u8, u32)>>),
    ExtendedLiteral(u32), // TODO; replace at load time
}

impl LValue {
    pub fn to_u32(&self) -> u32 {
        match *self {
            LValue::Literal(i) => i,
            LValue::Atom(i) => i,
            LValue::Label(i) => i,
            LValue::Integer(i) => i as u32,
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }
}

impl<'a> Loader<'a> {
    pub fn new() -> Loader<'a> {
        Loader {
            atoms: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            literals: Vec::new(),
            literal_heap: Heap::new(),
            strings: Vec::new(),
            lambdas: Vec::new(),
            atom_map: HashMap::new(),
            labels: HashMap::new(),
            funs: HashMap::new(),
            line_items: Vec::new(),
            lines: Vec::new(),
            file_names: Vec::new(),
            code: &[],
            instructions: Vec::new(),
            on_load: None,
        }
    }

    pub fn load_file(mut self, bytes: &'a [u8]) -> Result<Module, nom::Err<&'a [u8]>> {
        let (_, data) = scan_beam(bytes).unwrap();
        let mut chunks = HashMap::new();
        for (name, chunk) in data {
            chunks.insert(name, chunk);
        }

        // parse all the chunks:

        // build atoms table first
        self.load_atoms(chunks.remove("AtU8").expect("Atom AtU8 chunk not found!"));

        if let Some(chunk) = chunks.remove("LocT") {
            self.load_local_fun_table(chunk); // can probably be ignored
        }
        self.load_imports_table(chunks.remove("ImpT").expect("ImpT chunk not found!"));
        self.load_exports_table(chunks.remove("ExpT").expect("ExpT chunk not found!"));
        if let Some(chunk) = chunks.remove("StrT") {
            self.load_strings_table(chunk);
        }
        if let Some(chunk) = chunks.remove("LitT") {
            self.load_literals_table(chunk);
        }
        if let Some(chunk) = chunks.remove("Line") {
            self.load_lines_table(chunk);
        }
        if let Some(chunk) = chunks.remove("FunT") {
            self.load_lambdas_table(chunk);
        }
        self.load_attributes(chunks.remove("Attr").expect("Attr chunk not found!"));
        self.load_code(chunks.remove("Code").expect("Code chunk not found!"));

        // parse the instructions, swapping for global vals
        // - swap load atoms with global atoms
        // - patch jump instructions to labels (store patches if label wasn't seen yet)
        // - make imports work via pointers..
        self.prepare();

        Ok(Module {
            imports: self.imports,
            exports: self.exports,
            literals: self.literals,
            literal_heap: self.literal_heap,
            lambdas: self.lambdas,
            funs: self.funs,
            instructions: self.instructions,
            lines: self.lines,
            name: self.atom_map[&0], // atom 0 is module name
            on_load: self.on_load,
        })
    }

    fn load_code(&mut self, chunk: Chunk<'a>) {
        let (_, data) = code_chunk(chunk).unwrap();
        self.code = data.code;
    }

    fn load_atoms(&mut self, chunk: Chunk<'a>) {
        let (_, atoms) = atom_chunk(chunk).unwrap();
        self.atoms = atoms;
        ATOMS.reserve(self.atoms.len() as u32);

        for (index, a) in self.atoms.iter().enumerate() {
            let g_index = atom::from_str(a);
            // keep a mapping of these to patch the instrs
            self.atom_map.insert(index as u32, g_index);
        }

        // Create a new version number for this module and fill self.mod_id
        // self.set_mod_id(code_server)
    }

    fn load_attributes(&mut self, chunk: Chunk) {
        // Contains two parts: a proplist of module attributes, encoded as External Term Format,
        // and a compiler info (options and version) encoded similarly.
        let (_rest, _val) = etf::decode(chunk, &self.literal_heap).unwrap();
    }

    fn load_local_fun_table(&mut self, chunk: Chunk) {
        let (_, _data) = loct_chunk(chunk).unwrap();
    }

    fn load_imports_table(&mut self, chunk: Chunk) {
        let (_, data) = loct_chunk(chunk).unwrap();
        self.imports = data
            .into_iter()
            .map(|mfa| {
                MFA(
                    self.atom_map[&(mfa.0 as u32 - 1)],
                    self.atom_map[&(mfa.1 as u32 - 1)],
                    mfa.2,
                )
            })
            .collect();
    }

    fn load_exports_table(&mut self, chunk: Chunk) {
        let (_, data) = loct_chunk(chunk).unwrap();
        self.exports = data
            .into_iter()
            .map(|mfa| {
                MFA(
                    self.atom_map[&(mfa.0 as u32 - 1)],
                    mfa.1,
                    mfa.2, // TODO: translate these offsets instead of using funs[]
                )
            })
            .collect();
    }

    fn load_strings_table(&mut self, chunk: Chunk) {
        self.strings = chunk.to_vec();
    }

    fn load_literals_table(&mut self, chunk: Chunk) {
        let (rest, size) = be_u32(chunk).unwrap();
        let mut data = Vec::with_capacity(size as usize);

        // Decompress deflated literal table
        let iocursor = Cursor::new(rest);
        zlib::Decoder::new(iocursor)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let buf = &data[..];

        assert_eq!(data.len(), size as usize, "LitT inflate failed");

        // self.literals.reserve(count as u32);

        // Decode literals into literal heap
        // pass in an allocator that allocates to a permanent non GC heap
        // TODO: probably GC'd when module is deallocated?
        // &self.literal_allocator
        let (_, literals) = decode_literals(buf, &self.literal_heap).unwrap();
        self.literals = literals;
    }

    fn load_lines_table(&mut self, chunk: Chunk<'a>) {
        // If the emulator flag ignoring the line information was given, return immediately.

        // if (erts_no_line_info) { return (); }

        let (_, (line_items, file_names)) = decode_lines(chunk).unwrap();

        self.line_items = line_items;
        self.file_names = file_names;
        // TODO: insert file names into atom table, remap line_items to point to atoms

        // Allocate the arrays to be filled while code is being loaded.
        // stp->line_instr = (LineInstr *) erts_alloc(ERTS_ALC_T_PREPARED_CODE,
        // stp->num_line_instrs *
        // sizeof(LineInstr));
        // stp->current_li = 0;
        // stp->func_line = (unsigned int *)  erts_alloc(ERTS_ALC_T_PREPARED_CODE,
        // stp->num_functions *
        // sizeof(int));
    }

    fn load_lambdas_table(&mut self, chunk: Chunk) {
        let (_, data) = funt_chunk(chunk).unwrap();
        // TODO: convert at load time, if nfree == 0, then allocate as a literal --> move instruction,
        // if captures env, then allocate a func --> make_fun2
        //
        self.lambdas = data;
    }

    // TODO: return a Module
    fn prepare(&mut self) {
        self.postprocess_raw_code();
        self.postprocess_lambdas();

        //self.postprocess_fix_labels()?;
        //self.postprocess_setup_imports()?;
    }

    /// Loops twice, once to parse annotations, once to remap the args.
    fn postprocess_raw_code(&mut self) {
        // scan over the Code chunk bits, but we'll need to know bit length of each instruction.
        let (_, code) = scan_instructions(self.code).unwrap();

        let mut code_iter = code.into_iter();
        let mut pos = 0;
        let mut last_func_start = 0;

        self.lines.push(Line {
            pos: LINE_INVALID_LOCATION,
            loc: 0,
        });

        while let Some(mut instruction) = code_iter.next() {
            let instruction = match &instruction.op {
                Opcode::Line => {
                    // args: [Literal(4)]
                    if !self.line_items.is_empty() {
                        if let LValue::Literal(item) = instruction.args[0] {
                            // if (item >= stp->num_line_items) {
                            // LoadError2(stp, "line instruction index overflow (%u/%u)",
                            // item, stp->num_line_items);
                            // }
                            // li = stp->current_li;
                            // if (li >= stp->num_line_instrs) {
                            // LoadError2(stp, "line instruction table overflow (%u/%u)",
                            // li, stp->num_line_instrs);
                            // }
                            let loc = self.line_items[item as usize].1 as usize;
                            // TODO: map as loc => offset

                            if pos > 0 && pos - 1 == last_func_start {
                                // This line instruction directly follows the func_info
                                // instruction. Its address must be adjusted to point to
                                // func_info instruction.
                                self.lines.push(Line {
                                    pos: last_func_start,
                                    loc,
                                })
                            } else {
                                //} else if (li <= stp->func_line[function_number-1] || stp->line_instr[li-1].loc != loc) {
                                // Only store the location if it is different from the previous
                                // location in the same function.
                                self.lines.push(Line { pos, loc })
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    continue;
                }
                Opcode::Label => {
                    // one operand, Integer
                    if let LValue::Literal(i) = instruction.args[0] {
                        self.labels.insert(i as u32, self.instructions.len() as u32);
                    } else {
                        panic!("Bad argument to {:?}", instruction.op)
                    }
                    continue;
                }
                Opcode::FuncInfo => {
                    // record function data M:F/A
                    // don't skip so we can apply tracing during runtime
                    if let [_module, LValue::Atom(f), LValue::Literal(a)] = &instruction.args[..] {
                        last_func_start = pos; // store pos for line number data
                        let f = self.atom_map[&(*f - 1)]; // necessary because atoms weren't remapped yet
                        self.funs
                            .insert((f, *a as u32), (self.instructions.len() as u32) + 1); // need to point after func_info
                    } else {
                        panic!("Bad argument to {:?}", instruction.op)
                    }
                    instruction
                }
                Opcode::IntCodeEnd => {
                    // push a pointer to the end of instructions
                    self.funs.insert(
                        (LINE_INVALID_LOCATION as u32, 0),
                        self.instructions.len() as u32,
                    );
                    break;
                }
                Opcode::OnLoad => unimplemented!("on_load instruction"),
                Opcode::BsPutString => {
                    if let [LValue::Literal(len), LValue::Literal(offset)] = instruction.args[..] {
                        // TODO: ideally use a single string that we slice into for these interned strings
                        // but need to tie them to the string heap lifetime
                        let offset = offset as usize;
                        let bytes = &self.strings[offset..offset + len as usize];
                        instruction.args = vec![LValue::Binary(Arc::new(bitstring::Binary::from(
                            bytes.as_bytes(),
                        )))];
                        instruction
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsMatchString => {
                    if let [fail, src, LValue::Literal(bits), LValue::Literal(offset)] =
                        &instruction.args[..]
                    {
                        // TODO: needs to work with byte unaligned stuff

                        // TODO: ideally use a single string that we slice into for these interned strings
                        // but need to tie them to the string heap lifetime
                        let offset = *offset as usize;
                        let len = bits >> 3;
                        let bytes = &self.strings[offset..offset + len as usize];
                        instruction.args = vec![
                            fail.clone(), // not happy about the clones
                            src.clone(),
                            LValue::Literal(*bits),
                            LValue::Str(bytes.as_bytes().to_vec()),
                        ];
                        instruction
                    } else {
                        unreachable!()
                    }
                }
                Opcode::PutTuple => {
                    if let [LValue::Literal(arity), destination] = &instruction.args[..] {
                        let mut i = *arity;
                        let mut list = Vec::new();
                        while i > 0 {
                            if let Some(Instruction {
                                op: Opcode::Put,
                                args,
                            }) = code_iter.next()
                            {
                                if let [arg] = &args[..] {
                                    list.push(arg.clone());
                                    i -= 1;
                                } else {
                                    unreachable!()
                                }
                            } else {
                                unreachable!()
                            }
                        }
                        instruction.op = Opcode::PutTuple2;
                        instruction.args =
                            vec![destination.clone(), LValue::ExtendedList(Box::new(list))];
                        instruction
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Put => {
                    // put should always have been caught by the put_tuple case and translated to put_tuple2
                    unreachable!()
                }
                _ => instruction,
            };

            pos += 1;
            self.instructions.push(instruction);
        }

        let atom_map = &self.atom_map;
        let labels = &self.labels;
        let postprocess_value = |arg: &LValue| -> LValue {
            match arg {
                LValue::Atom(i) => {
                    if *i == 0 {
                        return LValue::Nil;
                    }
                    LValue::Atom(atom_map[&(*i - 1)])
                }
                LValue::Label(l) => {
                    if *l == 0 {
                        return LValue::Label(0);
                    }
                    LValue::Label(labels[l])
                }
                val => val.clone(),
            }
        };

        self.instructions.iter_mut().for_each(|instruction| {
            // patch local atoms into global
            instruction.args = instruction
                .args
                .iter()
                .map(|arg| match arg {
                    LValue::ExtendedList(vec) => {
                        let vec = (*vec).iter().map(|arg| postprocess_value(arg)).collect();
                        LValue::ExtendedList(Box::new(vec))
                    }
                    _ => postprocess_value(arg),
                })
                .collect();
        })
    }

    fn postprocess_lambdas(&mut self) {
        let labels = &self.labels;
        self.lambdas.iter_mut().for_each(|lambda| {
            lambda.offset = labels[&lambda.offset];
        });
    }
}

fn decode_literals<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Vec<Term>> {
    do_parse!(
        rest,
        _count: be_u32
            >> literals:
                many0!(complete!(do_parse!(
                    _size: be_u32 >> literal: call!(etf::decode, heap) >> (literal)
                )))
            >> (literals)
    )
}

fn decode_lines<'a>(rest: &'a [u8]) -> IResult<&'a [u8], (Vec<FuncInfo>, Vec<&str>)> {
    // Check version of line table.
    let (rest, version) = be_u32(rest)?;
    if version != 0 {
        // Wrong version. Silently ignore the line number chunk.
        return Ok((rest, (Vec::new(), Vec::new()))); // TODO: use option maybe
    }

    do_parse!(
        rest,
        _flags: be_u32 >> // reserved for future use
        _num_line_instrs: be_u32 >>
        num_line_items: be_u32 >>
        num_fnames: be_u32 >>
        line_items: call!(decode_line_items, num_line_items) >>
        file_names: count!(map_res!(length_bytes!(be_u16), std::str::from_utf8), num_fnames as usize) >>
            ((line_items, file_names))
    )
}

fn decode_line_items<'a>(rest: &'a [u8], mut count: u32) -> IResult<&'a [u8], Vec<(u32, u32)>> {
    let mut vec = Vec::with_capacity(count as usize);
    vec.push((LINE_INVALID_LOCATION as u32, 0 as u32)); // 0th index = undefined location
    let mut fname_index = 0;

    let mut new_rest = rest;

    while count > 0 {
        count -= 1;
        let (rest, term) = compact_term(new_rest)?;
        new_rest = rest;
        match term {
            LValue::Integer(n) => {
                vec.push((fname_index, n as u32));
                // TODO: validate
                // Too many files or huge line number. Silently invalidate the location.
                // loc = LINE_INVALID_LOCATION;
            }
            LValue::Atom(i) => {
                fname_index = i;
                // if (val > stp->num_fnames) {
                // LoadError2(stp, "file index overflow (%u/%u)",
                // val, stp->num_fnames);
                // }
                count += 1;
            }
            _ => unreachable!(),
        }
    }
    Ok((new_rest, vec))
}

type Chunk<'a> = &'a [u8];

named!(
    pub scan_beam<&[u8], Vec<(&str, Chunk)>>,
    do_parse!(
        tag!("FOR1") >>
        _size: le_u32 >>
        tag!("BEAM") >>
        data: chunks >>
        (data)
    )
);

named!(
    chunks<&[u8], Vec<(&str, Chunk)>>,
    many0!(complete!(chunk))
);

named!(
    chunk<&[u8], (&str, Chunk)>,
    do_parse!(
        name: map_res!(take!(4), std::str::from_utf8) >>
        size: be_u32 >>
        bytes: take!(size) >>
        take!(align_bytes(size)) >>
        (( name, bytes ))
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
        //take!(sub_size) >> offset from the header
        code: rest >>
        (CodeChunk { sub_size, version, opcode_max, labels, functions, code })
    )
);

named!(
    atom_chunk<&[u8], Vec<&str>>,
    do_parse!(
        count: be_u32 >>
        // TODO figure out if we can prealloc the size
        atoms: count!(map_res!(length_bytes!(be_u8), std::str::from_utf8), count as usize) >>
        (atoms)
    )
);

named!(
    fun_entry<&[u8], MFA>,
    do_parse!(
        function: be_u32 >>
        arity: be_u32 >>
        label: be_u32 >>
        (MFA(function, arity, label))
    )
);

named!(
    loct_chunk<&[u8], Vec<MFA>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (entries)
    )
);

named!(
    litt_chunk<&[u8], Vec<MFA>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(fun_entry, count as usize) >>
        (entries)
    )
);

named!(
    funt_chunk<&[u8], Vec<Lambda>>,
    do_parse!(
        count: be_u32 >>
        entries: count!(do_parse!(
            name: be_u32 >>
            arity: be_u32 >>
            offset: be_u32 >>
            index: be_u32 >>
            nfree: be_u32 >>
            ouniq: be_u32 >>
            (Lambda { name, arity, offset: offset, index, nfree, ouniq })
            )
        , count as usize) >>
        (entries)
    )
);

// It can be Literal=0, Integer=1, Atom=2, XRegister=3, YRegister=4, Label=5, Character=6, Extended=7.
// If the base tag was Extended=7, then bits 4-5-6-7 PLUS 7 will become the extended tag.
// It can have values Float=8, List=9, FloatReg=10, AllocList=11, Literal=12.

// basically read_int, but returns an integer and never bignum to use elsewhere in loader.
// TODO: deduplicate.
fn read_smallint(b: u8, rest: &[u8]) -> IResult<&[u8], i32> {
    let (rest, val) = read_int(b, rest)?;

    if let LValue::Integer(i) = val {
        return Ok((rest, i));
    }
    unreachable!()
}

fn read_int(b: u8, rest: &[u8]) -> IResult<&[u8], LValue> {
    // it's not extended
    if 0 == (b & 0b1000) {
        // Bit 3 is 0 marks that 4 following bits contain the value
        return Ok((rest, LValue::Integer(i32::from(b >> 4))));
    }

    // Bit 3 is 1, but...
    if 0 == (b & 0b1_0000) {
        // Bit 4 is 0, marks that the following 3 bits (most significant) and
        // the following byte (least significant) will contain the 11-bit value
        let (rest, r) = be_u8(rest)?;
        Ok((
            rest,
            LValue::Integer((((b as usize) & 0b1110_0000) << 3 | (r as usize)) as i32),
        )) // upcasting to i64 from usize not safe
    } else {
        // Bit 4 is 1 means that bits 5-6-7 contain amount of bytes+2 to store
        // the value
        let mut n_bytes = (b as u32 >> 5) + 2;
        let mut rest = rest;
        if n_bytes == 9 {
            // bytes=9 means upper 5 bits were set to 1, special case 0b11111xxx
            // which means that following nested tagged value encodes size,
            // followed by the bytes (Size+9)
            let (r, len) = be_u8(rest)?;
            let (r, size) = read_smallint(len, r)?;
            n_bytes = size as u32 + 9; // TODO: enforce unsigned
            rest = r;
        }

        // Read the remaining big endian bytes and convert to int
        let (rest, long_bytes) = take!(rest, n_bytes)?;
        let sign = if long_bytes[0] & 0x80 == 0x80 {
            Sign::Minus
        } else {
            Sign::Plus
        };

        let r = BigInt::from_bytes_be(sign, long_bytes);

        if let Some(i) = r.to_i32() {
            // fits in a regular int
            return Ok((rest, LValue::Integer(i)));
        }

        Ok((rest, LValue::BigInt(r)))
    } // if larger than 11 bits
}

fn compact_term(i: &[u8]) -> IResult<&[u8], LValue> {
    let (rest, b) = be_u8(i)?;
    let tag = b & 0b111;

    // TODO: need to parse over few bytes, but not bigint too as smallint
    if b & 0b1111_1001 == 0b1111_1001 {
        // bigint scenario TODO: triple check
        let (rest, val) = read_int(b, rest)?;
        return Ok((rest, val));
    }

    if tag < 0b111 {
        //println!("b is {:?}, tag is {:?}", b, tag);
        let (rest, val) = read_int(b, rest)?;

        if let LValue::Integer(val) = val {
            return match tag {
                0 => Ok((rest, LValue::Literal(val as u32))),
                1 => Ok((rest, LValue::Integer(val))), // TODO: this cast is unsafe
                2 => Ok((rest, LValue::Atom(val as u32))),
                3 => Ok((rest, LValue::X(val as u32))),
                4 => Ok((rest, LValue::Y(val as u32))),
                5 => Ok((rest, LValue::Label(val as u32))),
                6 => Ok((rest, LValue::Character(val as u8))),
                _ => unreachable!(),
            };
        } else {
            return Ok((rest, val)); // bigint
        }
    }

    parse_extended_term(b, rest)
}

fn parse_extended_term(b: u8, rest: &[u8]) -> IResult<&[u8], LValue> {
    match b {
        0b0001_0111 => parse_list(rest),
        0b0010_0111 => parse_float_reg(rest),
        0b0011_0111 => parse_alloc_list(rest),
        0b0100_0111 => parse_extended_literal(rest),
        _ => unreachable!(),
    }
}

fn parse_list(rest: &[u8]) -> IResult<&[u8], LValue> {
    // The stream now contains a smallint size, then size/2 pairs of values
    let (rest, b) = be_u8(rest)?;
    let (mut rest, n) = read_smallint(b, rest)?;
    let mut els = Vec::with_capacity(n as usize);

    // TODO: create tuple of size n, then read n/2 pairs of key/label
    // sequentially into the tuple. (used for select_val ops)

    for _i in 0..n {
        let (new_rest, term) = compact_term(rest)?;
        els.push(term);
        rest = new_rest;
    }

    Ok((rest, LValue::ExtendedList(Box::new(els))))
}

fn parse_float_reg(rest: &[u8]) -> IResult<&[u8], LValue> {
    let (rest, b) = be_u8(rest)?;
    let (rest, n) = read_smallint(b, rest)?;

    Ok((rest, LValue::FloatReg(n as u32)))
}

fn parse_alloc_list(rest: &[u8]) -> IResult<&[u8], LValue> {
    let (rest, b) = be_u8(rest)?;
    let (mut rest, n) = read_smallint(b, rest)?;
    let mut els = Vec::with_capacity(n as usize);

    for _i in 0..n {
        // decode int Type (0 = words, 1 = floats, 2 = literal)
        let (new_rest, b) = be_u8(rest)?;
        let (new_rest, typ) = read_smallint(b, new_rest)?;
        // decode int Val as is, except  type = 2, get(literals, val)
        // TODO: decide how to handle literals 2
        let (new_rest, b) = be_u8(new_rest)?;
        let (new_rest, val) = read_smallint(b, new_rest)?;

        els.push((typ as u8, val as u32));
        rest = new_rest;
    }

    Ok((rest, LValue::AllocList(Box::new(els))))
}

fn parse_extended_literal(rest: &[u8]) -> IResult<&[u8], LValue> {
    let (rest, b) = be_u8(rest)?;
    let (rest, val) = read_smallint(b, rest)?;
    Ok((rest, LValue::ExtendedLiteral(val as u32)))
}

// ---------------------------------------------------

#[derive(Debug)]
pub struct Instruction {
    pub op: Opcode,
    pub args: Vec<LValue>,
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
            Ok((&[] as &[u8], LValue::Literal(9)))
        );
        assert_eq!(
            compact_term(&vec![0b11110000u8]),
            Ok((&[] as &[u8], LValue::Literal(15)))
        );
    }
    // TODO: test encoding/decoding 38374938373887374983978484
}
