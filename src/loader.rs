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
#[derive(Clone)]
pub enum LValue {
    /// Atom | Integer | Character | Nil (| Binary | BigInt in the future)
    Constant(Term),
    // TODO: these dupe LTerm values
    Atom(u32),
    Integer(i32),
    Character(u8),
    Nil,
    Str(Vec<u8>), // TODO this is so avoid constructing a full binary
    Bif(crate::bif::Fn),
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

// LValue as { X(), Y(), Literal(), Constant(integer | atom | nil term)

impl std::fmt::Debug for LValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LValue::X(i) => write!(f, "x{}", i),
            LValue::Y(i) => write!(f, "x{}", i),
            LValue::FloatReg(i) => write!(f, "f{}", i),
            LValue::Label(i) => write!(f, "l({})", i),
            LValue::Literal(i) => write!(f, "literal({})", i),
            LValue::Integer(i) => write!(f, "int({})", i),
            LValue::Atom(i) => write!(f, "atom({})", i),
            LValue::Character(i) => write!(f, "char({})", i),
            LValue::Constant(i) => write!(f, "const({})", i),
            LValue::Nil => write!(f, "nil()"),
            LValue::Literal(i) => write!(f, "{}", i),
            LValue::BigInt(i) => write!(f, "bigint({})", i),
            LValue::Bif(i) => write!(f, "<bif>"),
            LValue::Str(..) => write!(f, "<str>"),
            LValue::AllocList(..) => write!(f, "<alloclist>"),
            LValue::ExtendedList(..) => write!(f, "<ext list>"),
            LValue::ExtendedLiteral(..) => write!(f, "<ext lit>"),
            _ => write!(f, "lvalue<>"),
        }
    }
}

impl LValue {
    #[inline]
    pub fn to_u32(&self) -> u32 {
        match *self {
            LValue::Literal(i) => i,
            LValue::Atom(i) => i,
            LValue::Label(i) => i,
            LValue::Integer(i) => i as u32,
            LValue::Constant(i) => i.to_u32(),
            _ => unimplemented!("to_u32 for {:?}", self),
        }
    }

    #[inline]
    pub fn from_lit(&self) -> u32 {
        match *self {
            LValue::Literal(i) => i,
            _ => unreachable!(),
        }
    }

    pub fn into_term(&self) -> crate::value::Term {
        match *self {
            LValue::Constant(t) => t,
            _ => unreachable!("into_term for {:?}", self),
        }
    }

    pub fn into_value(&self) -> crate::value::Variant {
        match *self {
            LValue::Constant(t) => t.into_variant(),
            LValue::Integer(i) => crate::value::Variant::Integer(i),
            LValue::Atom(i) => crate::value::Variant::Atom(i),
            LValue::Nil => crate::value::Variant::Nil(crate::value::Special::Nil),
            _ => unreachable!("into_value for {:?}", self),
        }
    }

    // conversion funs
    pub fn to_atom(&self) -> crate::instruction::Atom {
        match *self {
            LValue::Atom(i) => i,
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_arity(&self) -> crate::instruction::Arity {
        use std::convert::TryInto;
        match *self {
            LValue::Literal(i) => i.try_into().unwrap(),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_label(&self) -> crate::instruction::Label {
        match *self {
            LValue::Label(i) => i,
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_reg(&self) -> crate::instruction::Register {
        use crate::instruction::Register;
        use std::convert::TryInto;
        match *self {
            LValue::X(i) => Register::X(i.try_into().unwrap()),
            LValue::Y(i) => Register::Y(i.try_into().unwrap()),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_lit(&self) -> u32 {
        match *self {
            LValue::Literal(i) => i,
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_regs(&self) -> crate::instruction::Regs {
        use std::convert::TryInto;
        match *self {
            LValue::Literal(i) => i.try_into().unwrap(),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_freg(&self) -> crate::instruction::FRegister {
        use crate::instruction::FRegister;
        use std::convert::TryInto;
        match *self {
            LValue::FloatReg(i) => FRegister::FloatReg(i.try_into().unwrap()),
            LValue::ExtendedLiteral(i) => FRegister::ExtendedLiteral(i),
            LValue::X(i) => FRegister::X(i.try_into().unwrap()),
            LValue::Y(i) => FRegister::Y(i.try_into().unwrap()),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_fregs(&self) -> crate::instruction::FloatRegs {
        use std::convert::TryInto;
        match *self {
            LValue::FloatReg(i) => i.try_into().unwrap(),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_bif(&self) -> crate::bif::Fn {
        match *self {
            LValue::Bif(i) => i,
            _ => panic!(),
        }
    }
    pub fn to_val(&self, constants: &mut Vec<Term>, heap: &Heap) -> crate::instruction::Source {
        use crate::instruction::Source;
        use std::convert::TryInto;
        match self {
            LValue::Constant(c) => {
                let i = constants.len();
                constants.push(*c);
                Source::Constant(i.try_into().unwrap())
            }
            LValue::BigInt(num) => {
                // very unperformant, double indirection
                let i = constants.len();
                constants.push(Term::bigint(heap, num.clone()));
                Source::Constant(i.try_into().unwrap())
            }
            LValue::ExtendedLiteral(i) => Source::ExtendedLiteral(*i),
            LValue::X(i) => Source::X((*i).try_into().unwrap()),
            LValue::Y(i) => Source::Y((*i).try_into().unwrap()),
            _ => panic!("{:?} passed", self),
        }
    }

    // temporary
    pub fn to_size(&self, constants: &mut Vec<Term>) -> crate::instruction::Size {
        use crate::instruction::Size;
        use std::convert::TryInto;
        match self {
            LValue::Constant(c) => {
                let i = constants.len();
                constants.push(*c);
                Size::Constant(i.try_into().unwrap())
            }
            LValue::Literal(i) => Size::Literal(*i),
            LValue::X(i) => Size::X((*i).try_into().unwrap()),
            LValue::Y(i) => Size::Y((*i).try_into().unwrap()),
            _ => panic!("{:?} passed", self),
        }
    }

    pub fn to_ext_list(
        &self,
        constants: &mut Vec<Term>,
        heap: &Heap,
    ) -> crate::instruction::ExtendedList {
        match self {
            LValue::ExtendedList(l) => Box::new(
                l.iter()
                    .map(|i| match i {
                        LValue::Literal(i) => crate::instruction::Entry::Literal(*i),
                        LValue::ExtendedLiteral(i) => {
                            crate::instruction::Entry::ExtendedLiteral(*i)
                        }
                        LValue::Label(i) => crate::instruction::Entry::Label(*i),
                        _ => crate::instruction::Entry::Value(i.to_val(constants, heap)),
                    })
                    .collect(),
            ),
            _ => panic!(),
        }
    }

    pub fn to_str(&self) -> Vec<u8> {
        match self {
            LValue::Str(s) => s.clone(), // TODO: clone not good
            _ => panic!(),
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

        let mut constants = Vec::new();
        let instructions: Vec<_> = self
            .instructions
            .iter()
            .map(|ins| transform_instruction(ins.clone(), &mut constants, &self.literal_heap))
            .collect();
        // println!("ins {:?}", instructions);

        Ok(Module {
            imports: self.imports,
            exports: self.exports,
            constants,
            literals: self.literals,
            literal_heap: self.literal_heap,
            lambdas: self.lambdas,
            funs: self.funs,
            instructions: instructions,
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
        let _val = etf::decode(chunk, &self.literal_heap);
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
                        instruction.args = vec![LValue::Str(bytes.as_bytes().to_vec())];
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
                // postprocess Bif0, Bif1, Bif2, GcBif1, GcBif2, GcBif3 to contain Fn ptrs
                Opcode::Bif0 => {
                    let dest = instruction.args[0].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[0] = LValue::Bif(*fun);
                    instruction
                }
                Opcode::Bif1 => {
                    let dest = instruction.args[1].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[1] = LValue::Bif(*fun);
                    instruction
                }
                Opcode::Bif2 => {
                    let dest = instruction.args[1].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[1] = LValue::Bif(*fun);
                    instruction
                }
                Opcode::GcBif1 => {
                    let dest = instruction.args[2].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[2] = LValue::Bif(*fun);
                    instruction
                }
                Opcode::GcBif2 => {
                    let dest = instruction.args[2].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[2] = LValue::Bif(*fun);
                    instruction
                }
                Opcode::GcBif3 => {
                    let dest = instruction.args[2].from_lit();
                    let mfa = &self.imports[dest as usize];
                    let fun = match crate::bif::BIFS.get(mfa) {
                        Some(fun) => fun,
                        None => unimplemented!("BIF {} not implemented", mfa),
                    };
                    instruction.args[2] = LValue::Bif(*fun);
                    instruction
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
                        return LValue::Constant(Term::nil());
                    }
                    LValue::Constant(Term::atom(atom_map[&(*i - 1)]))
                }
                LValue::Integer(i) => LValue::Constant(Term::int(*i)),
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
        });
    }

    fn postprocess_lambdas(&mut self) {
        let labels = &self.labels;
        self.lambdas.iter_mut().for_each(|lambda| {
            lambda.offset = labels[&lambda.offset];
        });
    }
}

fn transform_instruction(
    ins: self::Instruction,
    constants: &mut Vec<Term>,
    literal_heap: &Heap,
) -> crate::instruction::Instruction {
    // use crate::opcodes::Opcode;
    use crate::instruction::Instruction;
    use std::convert::TryInto;

    match ins.op {
        Opcode::Label | Opcode::IntCodeEnd | Opcode::Line => unreachable!("removed at load time"),
        Opcode::PutTuple
        | Opcode::Put
        | Opcode::BsStartMatch2
        | Opcode::BsContextToBinary
        | Opcode::BsSave2
        | Opcode::BsRestore2 => unreachable!("compiler too old"),
        Opcode::FuncInfo => Instruction::FuncInfo {
            module: ins.args[0].to_val(constants, literal_heap),
            function: ins.args[1].to_val(constants, literal_heap),
            arity: ins.args[2].to_arity(),
        },
        Opcode::Call => Instruction::Call {
            arity: ins.args[0].to_arity(),
            label: ins.args[1].to_label(),
        },
        Opcode::CallLast => Instruction::CallLast {
            arity: ins.args[0].to_arity(),
            label: ins.args[1].to_label(),
            words: ins.args[2].to_regs(),
        },
        Opcode::CallOnly => Instruction::CallOnly {
            arity: ins.args[0].to_arity(),
            label: ins.args[1].to_label(),
        },
        Opcode::CallExt => Instruction::CallExt {
            arity: ins.args[0].to_arity(),
            destination: ins.args[1].to_lit() as usize,
        },
        Opcode::CallExtOnly => Instruction::CallExtOnly {
            arity: ins.args[0].to_arity(),
            destination: ins.args[1].to_lit() as usize,
        },
        Opcode::CallExtLast => Instruction::CallExtLast {
            arity: ins.args[0].to_arity(),
            destination: ins.args[1].to_lit() as usize,
            words: ins.args[2].to_regs(),
        },
        Opcode::Bif0 => Instruction::Bif0 {
            bif: ins.args[0].to_bif(),
            reg: ins.args[1].to_reg(),
        },
        Opcode::Bif1 => Instruction::Bif1 {
            fail: ins.args[0].to_label(),
            bif: ins.args[1].to_bif(),
            arg1: ins.args[2].to_val(constants, literal_heap),
            reg: ins.args[3].to_reg(),
        },
        Opcode::Bif2 => Instruction::Bif2 {
            fail: ins.args[0].to_label(),
            bif: ins.args[1].to_bif(),
            arg1: ins.args[2].to_val(constants, literal_heap),
            arg2: ins.args[3].to_val(constants, literal_heap),
            reg: ins.args[4].to_reg(),
        },
        Opcode::Allocate => Instruction::Allocate {
            stackneed: ins.args[0].to_regs(),
            live: ins.args[1].to_regs(),
        },
        Opcode::AllocateHeap => Instruction::AllocateHeap {
            stackneed: ins.args[0].to_regs(),
            heapneed: ins.args[1].to_regs(),
            live: ins.args[2].to_regs(),
        },
        Opcode::AllocateZero => Instruction::AllocateZero {
            stackneed: ins.args[0].to_regs(),
            live: ins.args[1].to_regs(),
        },
        Opcode::AllocateHeapZero => Instruction::AllocateHeapZero {
            stackneed: ins.args[0].to_regs(),
            heapneed: ins.args[1].to_regs(),
            live: ins.args[2].to_regs(),
        },
        Opcode::TestHeap => Instruction::TestHeap {
            // heapneed: ins.args[0].to_regs(),
            live: ins.args[1].to_regs(),
        },
        Opcode::Init => Instruction::Init {
            n: ins.args[0].to_reg(),
        },
        Opcode::Deallocate => Instruction::Deallocate {
            n: ins.args[0].to_regs(),
        },
        Opcode::Return => Instruction::Return,
        Opcode::Send => Instruction::Send,
        Opcode::RemoveMessage => Instruction::RemoveMessage,
        Opcode::Timeout => Instruction::Timeout,
        Opcode::LoopRec => Instruction::LoopRec {
            label: ins.args[0].to_label(),
            source: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::LoopRecEnd => Instruction::LoopRecEnd {
            label: ins.args[0].to_label(),
        },
        Opcode::Wait => Instruction::Wait {
            label: ins.args[0].to_label(),
        },
        Opcode::WaitTimeout => Instruction::WaitTimeout {
            label: ins.args[0].to_label(),
            time: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsLt => Instruction::IsLt {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsGe => Instruction::IsGe {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsEq => Instruction::IsEq {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsNe => Instruction::IsNe {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsEqExact => Instruction::IsEqExact {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsNeExact => Instruction::IsNeExact {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arg2: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::IsInteger => Instruction::IsInteger {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsFloat => Instruction::IsFloat {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsNumber => Instruction::IsNumber {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsAtom => Instruction::IsAtom {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsPid => Instruction::IsPid {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsReference => Instruction::IsReference {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsPort => Instruction::IsPort {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsNil => Instruction::IsNil {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsBinary => Instruction::IsBinary {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsList => Instruction::IsList {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsNonemptyList => Instruction::IsNonemptyList {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsTuple => Instruction::IsTuple {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::TestArity => Instruction::TestArity {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arity: ins.args[2].to_u32(),
        },
        Opcode::SelectVal => Instruction::SelectVal {
            arg: ins.args[0].to_val(constants, literal_heap),
            fail: ins.args[1].to_label(),
            destinations: ins.args[2].to_ext_list(constants, literal_heap),
        },
        Opcode::SelectTupleArity => Instruction::SelectTupleArity {
            arg: ins.args[0].to_val(constants, literal_heap),
            fail: ins.args[1].to_label(),
            arities: ins.args[2].to_ext_list(constants, literal_heap),
        },
        Opcode::Jump => Instruction::Jump(ins.args[0].to_label()),
        Opcode::Catch => Instruction::Catch {
            register: ins.args[0].to_reg(),
            fail: ins.args[1].to_label(),
        },
        Opcode::CatchEnd => Instruction::CatchEnd {
            register: ins.args[0].to_reg(),
        },
        Opcode::Move => Instruction::Move {
            source: ins.args[0].to_val(constants, literal_heap),
            destination: ins.args[1].to_reg(),
        },
        Opcode::GetList => Instruction::GetList {
            source: ins.args[0].to_reg(),
            head: ins.args[1].to_reg(),
            tail: ins.args[2].to_reg(),
        },
        Opcode::GetTupleElement => Instruction::GetTupleElement {
            source: ins.args[0].to_reg(),
            element: ins.args[1].to_u32(), // TODO: strict to_u32
            destination: ins.args[2].to_reg(),
        },
        Opcode::SetTupleElement => Instruction::SetTupleElement {
            new_element: ins.args[0].to_val(constants, literal_heap),
            tuple: ins.args[1].to_val(constants, literal_heap),
            position: ins.args[2].to_u32(), // TODO: strict to_u32
        },
        Opcode::PutList => Instruction::PutList {
            head: ins.args[0].to_val(constants, literal_heap),
            tail: ins.args[1].to_val(constants, literal_heap),
            destination: ins.args[2].to_reg(),
        },
        Opcode::PutTuple2 => Instruction::PutTuple2 {
            source: ins.args[0].to_reg(),
            list: ins.args[1].to_ext_list(constants, literal_heap),
        },
        Opcode::Badmatch => Instruction::Badmatch {
            value: ins.args[0].to_val(constants, literal_heap),
        },
        Opcode::IfEnd => Instruction::IfEnd,
        Opcode::CaseEnd => Instruction::CaseEnd {
            value: ins.args[0].to_val(constants, literal_heap),
        },
        Opcode::CallFun => Instruction::CallFun {
            arity: ins.args[0].to_arity(),
        },
        Opcode::IsFunction => Instruction::IsFunction {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::BsPutInteger => Instruction::BsPutInteger {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_val(constants, literal_heap),
            unit: ins.args[2].to_u32().try_into().unwrap(),
            flags: ins.args[3].to_u32().try_into().unwrap(),
            source: ins.args[4].to_val(constants, literal_heap),
        },
        Opcode::BsPutBinary => Instruction::BsPutBinary {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_val(constants, literal_heap),
            unit: ins.args[2].to_u32().try_into().unwrap(),
            flags: ins.args[3].to_u32().try_into().unwrap(),
            source: ins.args[4].to_val(constants, literal_heap),
        },
        Opcode::BsPutFloat => Instruction::BsPutFloat {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_val(constants, literal_heap),
            unit: ins.args[2].to_u32().try_into().unwrap(),
            flags: ins.args[3].to_u32().try_into().unwrap(),
            source: ins.args[4].to_val(constants, literal_heap),
        },
        Opcode::BsPutString => Instruction::BsPutString {
            binary: ins.args[0].to_str(),
        },
        Opcode::Fclearerror => Instruction::Fclearerror,
        Opcode::Fcheckerror => Instruction::Fcheckerror,
        Opcode::Fmove => Instruction::Fmove {
            source: ins.args[0].to_freg(),      // reg + fregs
            destination: ins.args[1].to_freg(), // reg + fregs
        },
        Opcode::Fconv => Instruction::Fconv {
            source: ins.args[0].to_freg(),
            destination: ins.args[1].to_fregs(),
        },
        Opcode::Fadd => Instruction::Fadd {
            fail: ins.args[0].to_label(),
            a: ins.args[1].to_fregs(),
            b: ins.args[2].to_fregs(),
            destination: ins.args[3].to_fregs(),
        },
        Opcode::Fsub => Instruction::Fsub {
            fail: ins.args[0].to_label(),
            a: ins.args[1].to_fregs(),
            b: ins.args[2].to_fregs(),
            destination: ins.args[3].to_fregs(),
        },
        Opcode::Fmul => Instruction::Fmul {
            fail: ins.args[0].to_label(),
            a: ins.args[1].to_fregs(),
            b: ins.args[2].to_fregs(),
            destination: ins.args[3].to_fregs(),
        },
        Opcode::Fdiv => Instruction::Fdiv {
            fail: ins.args[0].to_label(),
            a: ins.args[1].to_fregs(),
            b: ins.args[2].to_fregs(),
            destination: ins.args[3].to_fregs(),
        },
        Opcode::Fnegate => Instruction::Fnegate {
            source: ins.args[0].to_fregs(),
            destination: ins.args[1].to_fregs(),
        },
        Opcode::MakeFun2 => Instruction::MakeFun2 {
            i: ins.args[0].to_regs(),
        },
        Opcode::Try => Instruction::Try {
            register: ins.args[0].to_reg(),
            fail: ins.args[1].to_label(),
        },
        Opcode::TryEnd => Instruction::TryEnd {
            register: ins.args[0].to_reg(),
        },
        Opcode::TryCase => Instruction::TryCase {
            register: ins.args[0].to_reg(),
        },
        Opcode::TryCaseEnd => Instruction::TryCaseEnd {
            value: ins.args[0].to_val(constants, literal_heap),
        },
        Opcode::Raise => Instruction::Raise {
            trace: ins.args[0].to_reg(),
            value: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::BsInit2 => Instruction::BsInit2 {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_size(constants),
            words: ins.args[2].to_regs(),
            regs: ins.args[3].to_regs(),
            flags: ins.args[4].to_u32().try_into().unwrap(), // maybe skip flags?
            destination: ins.args[5].to_reg(),
        },
        Opcode::BsAdd => Instruction::BsAdd {
            fail: ins.args[0].to_label(),
            size1: ins.args[1].to_val(constants, literal_heap),
            size2: ins.args[2].to_val(constants, literal_heap),
            unit: ins.args[3].to_u32().try_into().unwrap(),
            destination: ins.args[4].to_reg(),
        },
        Opcode::Apply => Instruction::Apply {
            arity: ins.args[0].to_arity(),
        },
        Opcode::ApplyLast => Instruction::ApplyLast {
            arity: ins.args[0].to_arity(),
            nwords: ins.args[1].to_regs(),
        },
        Opcode::IsBoolean => Instruction::IsBoolean {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::IsFunction2 => Instruction::IsFunction2 {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
            arity: ins.args[2].to_val(constants, literal_heap),
        },
        Opcode::BsGetInteger2 => Instruction::BsGetInteger2 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
            size: ins.args[3].to_val(constants, literal_heap),
            unit: ins.args[4].to_u32().try_into().unwrap(),
            flags: ins.args[5].to_u32().try_into().unwrap(),
            destination: ins.args[6].to_reg(),
        },
        Opcode::BsGetFloat2 => Instruction::BsGetFloat2 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
            size: ins.args[3].to_val(constants, literal_heap),
            unit: ins.args[4].to_u32().try_into().unwrap(),
            flags: ins.args[5].to_u32().try_into().unwrap(),
            destination: ins.args[6].to_reg(),
        },
        Opcode::BsGetBinary2 => Instruction::BsGetBinary2 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
            size: ins.args[3].to_val(constants, literal_heap),
            unit: ins.args[4].to_u32().try_into().unwrap(),
            flags: ins.args[5].to_u32().try_into().unwrap(),
            destination: ins.args[6].to_reg(),
        },
        Opcode::BsSkipBits2 => Instruction::BsSkipBits2 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_val(constants, literal_heap),
            unit: ins.args[3].to_u32().try_into().unwrap(),
            flags: ins.args[4].to_u32().try_into().unwrap(),
        },
        Opcode::BsTestTail2 => Instruction::BsTestTail2 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            bits: ins.args[2].to_u32(),
        },
        Opcode::GcBif1 => Instruction::GcBif1 {
            label: ins.args[0].to_label(),
            live: ins.args[1].to_regs(),
            bif: ins.args[2].to_bif(),
            arg1: ins.args[3].to_val(constants, literal_heap),
            reg: ins.args[4].to_reg(),
        },
        Opcode::GcBif2 => Instruction::GcBif2 {
            label: ins.args[0].to_label(),
            live: ins.args[1].to_regs(),
            bif: ins.args[2].to_bif(),
            arg1: ins.args[3].to_val(constants, literal_heap),
            arg2: ins.args[4].to_val(constants, literal_heap),
            reg: ins.args[5].to_reg(),
        },
        Opcode::IsBitstr => Instruction::IsBitstr {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::BsTestUnit => Instruction::BsTestUnit {
            fail: ins.args[0].to_label(),
            context: ins.args[1].to_reg(),
            unit: ins.args[2].to_lit().try_into().unwrap(),
        },
        Opcode::BsMatchString => Instruction::BsMatchString {
            fail: ins.args[0].to_label(),
            context: ins.args[1].to_reg(),
            bits: ins.args[2].to_u32(),
            string: ins.args[3].to_str(),
        },
        Opcode::BsInitWritable => Instruction::BsInitWritable,
        Opcode::BsAppend => Instruction::BsAppend {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_val(constants, literal_heap),
            extra: ins.args[2].to_regs(),
            live: ins.args[3].to_regs(),
            unit: ins.args[4].to_lit().try_into().unwrap(),
            bin: ins.args[5].to_val(constants, literal_heap),
            // flags: ins.args[6].to_u32().try_into().unwrap(), skip flags
            destination: ins.args[7].to_reg(),
        },
        Opcode::BsPrivateAppend => Instruction::BsPrivateAppend {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_val(constants, literal_heap),
            unit: ins.args[2].to_lit().try_into().unwrap(),
            bin: ins.args[3].to_val(constants, literal_heap),
            // flags: ins.args[5].to_u32().try_into().unwrap(), skip flags
            destination: ins.args[5].to_reg(),
        },
        Opcode::Trim => Instruction::Trim {
            // TODO: use regs sizes
            n: ins.args[0].to_u32(),
            remaining: ins.args[1].to_u32(),
        },
        Opcode::BsInitBits => Instruction::BsInitBits {
            fail: ins.args[0].to_label(),
            size: ins.args[1].to_size(constants),
            words: ins.args[2].to_regs(),
            regs: ins.args[3].to_regs(),
            flags: ins.args[4].to_u32().try_into().unwrap(), // maybe skip flags?
            destination: ins.args[5].to_reg(),
        },
        Opcode::BsGetUtf8 => Instruction::BsGetUtf8 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
            destination: ins.args[4].to_reg(),
        },
        Opcode::BsGetUtf16 => Instruction::BsGetUtf16 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
            destination: ins.args[4].to_reg(),
        },
        Opcode::BsGetUtf32 => Instruction::BsGetUtf32 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
            destination: ins.args[4].to_reg(),
        },
        Opcode::BsSkipUtf8 => Instruction::BsSkipUtf8 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
        },
        Opcode::BsSkipUtf16 => Instruction::BsSkipUtf16 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
        },
        Opcode::BsSkipUtf32 => Instruction::BsSkipUtf32 {
            fail: ins.args[0].to_label(),
            ms: ins.args[1].to_reg(),
            size: ins.args[2].to_size(constants),
            flags: ins.args[3].to_lit().try_into().unwrap(),
        },
        Opcode::BsUtf8Size => Instruction::BsUtf8Size {
            fail: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            destination: ins.args[2].to_reg(),
        },
        Opcode::BsUtf16Size => Instruction::BsUtf16Size {
            fail: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            destination: ins.args[2].to_reg(),
        },
        Opcode::BsPutUtf8 => Instruction::BsPutUtf8 {
            fail: ins.args[0].to_label(),
            flags: ins.args[1].to_u32().try_into().unwrap(),
            source: ins.args[2].to_reg(),
        },
        Opcode::BsPutUtf16 => Instruction::BsPutUtf16 {
            fail: ins.args[0].to_label(),
            flags: ins.args[1].to_u32().try_into().unwrap(),
            source: ins.args[2].to_reg(),
        },
        Opcode::BsPutUtf32 => Instruction::BsPutUtf32 {
            fail: ins.args[0].to_label(),
            flags: ins.args[1].to_u32().try_into().unwrap(),
            source: ins.args[2].to_reg(),
        },
        Opcode::OnLoad => unimplemented!("on_load instruction!"),
        Opcode::RecvMark => Instruction::RecvMark,
        Opcode::RecvSet => Instruction::RecvSet,
        Opcode::GcBif3 => Instruction::GcBif3 {
            label: ins.args[0].to_label(),
            live: ins.args[1].to_regs(),
            bif: ins.args[2].to_bif(),
            arg1: ins.args[3].to_val(constants, literal_heap),
            arg2: ins.args[4].to_val(constants, literal_heap),
            arg3: ins.args[5].to_val(constants, literal_heap),
            reg: ins.args[6].to_reg(),
        },
        Opcode::PutMapAssoc => Instruction::PutMapAssoc {
            // fail: obsolete past OTP 20 since we know it's preceeded by is_map
            map: ins.args[1].to_val(constants, literal_heap),
            destination: ins.args[2].to_reg(),
            live: ins.args[3].to_regs(),
            rest: ins.args[4].to_ext_list(constants, literal_heap),
        },
        Opcode::PutMapExact => Instruction::PutMapExact {
            // fail: obsolete past OTP 20 since we know it's preceeded by is_map
            map: ins.args[1].to_val(constants, literal_heap),
            destination: ins.args[2].to_reg(),
            live: ins.args[3].to_regs(),
            rest: ins.args[4].to_ext_list(constants, literal_heap),
        },
        Opcode::IsMap => Instruction::IsMap {
            label: ins.args[0].to_label(),
            arg1: ins.args[1].to_val(constants, literal_heap),
        },
        Opcode::HasMapFields => Instruction::HasMapFields {
            label: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            rest: ins.args[2].to_ext_list(constants, literal_heap),
        },
        Opcode::GetMapElements => Instruction::GetMapElements {
            label: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            rest: ins.args[2].to_ext_list(constants, literal_heap),
        },
        Opcode::IsTaggedTuple => Instruction::IsTaggedTuple {
            fail: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            arity: ins.args[2].to_u32(),
            atom: ins.args[3].to_val(constants, literal_heap),
        },
        Opcode::BuildStacktrace => Instruction::BuildStacktrace,
        Opcode::RawRaise => Instruction::RawRaise,
        Opcode::GetHd => Instruction::GetHd {
            source: ins.args[0].to_reg(),
            head: ins.args[1].to_reg(),
        },
        Opcode::GetTl => Instruction::GetTl {
            source: ins.args[0].to_reg(),
            tail: ins.args[1].to_reg(),
        },
        Opcode::BsGetTail => Instruction::BsGetTail {
            context: ins.args[0].to_reg(),
            destination: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
        },
        Opcode::BsStartMatch3 => Instruction::BsStartMatch3 {
            fail: ins.args[0].to_label(),
            source: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
            destination: ins.args[3].to_reg(),
        },
        Opcode::BsGetPosition => Instruction::BsGetPosition {
            context: ins.args[0].to_reg(),
            destination: ins.args[1].to_reg(),
            live: ins.args[2].to_regs(),
        },
        Opcode::BsSetPosition => Instruction::BsSetPosition {
            context: ins.args[0].to_reg(),
            position: ins.args[1].to_val(constants, literal_heap),
        },
    }
}

fn decode_literals<'a>(rest: &'a [u8], heap: &Heap) -> IResult<&'a [u8], Vec<Term>> {
    do_parse!(
        rest,
        _count: be_u32
            >> literals:
                many0!(complete!(do_parse!(
                    size: be_u32 >> literal: take!(size) >> (etf::decode(literal, heap))
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
            LValue::Integer((((b as isize) & 0b1110_0000) << 3 | (r as isize)) as i32),
        )) // upcasting to i64 from usize not safe
    } else {
        // Bit 4 is 1 means that bits 5-6-7 contain amount of bytes+2 to store
        // the value
        let mut n_bytes = (b as i32 >> 5) + 2;
        let mut rest = rest;
        if n_bytes == 9 {
            // bytes=9 means upper 5 bits were set to 1, special case 0b11111xxx
            // which means that following nested tagged value encodes size,
            // followed by the bytes (Size+9)
            let (r, len) = be_u8(rest)?;
            let (r, size) = read_smallint(len, r)?;
            n_bytes = size as i32 + 9; // TODO: enforce unsigned
            rest = r;
        }

        // Read the remaining big endian bytes and convert to int
        let (rest, long_bytes) = take!(rest, n_bytes)?;

        let r = BigInt::from_signed_bytes_be(long_bytes);

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
                1 => Ok((rest, LValue::Integer(val))),
                2 => Ok((rest, LValue::Atom(val as u32))), // remapped into const at postprocess
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

    // TODO: make sure values inside this list are packed
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

#[derive(Debug, Clone)]
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
