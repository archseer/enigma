// Label could be Option<NonZerou32> in some places?
use crate::bif::Fn as BifFn;
use crate::bitstring;
//pub type BifFn = fn() -> usize;
//use crate::loader::Source;

// 56 bytes with shrunken Source, 48 with flags removed on BsInit

#[derive(Clone, Copy)]

pub struct Bif(pub BifFn);

impl std::ops::Deref for Bif {
    type Target = BifFn;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for Bif {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<bif>")
    }
}

pub type Arity = u8;
pub type Label = u32;
pub type Atom = u32;

pub type FloatRegs = u8;
// shrink these from u16 to u8 once swap instruction is used
pub type Regs = u16;
pub type RegisterX = Regs;
pub type RegisterY = Regs;

pub type ExtendedList = Box<Vec<Entry>>;
pub type JumpTable = Box<Vec<Label>>;

#[derive(Debug, Clone)]
pub enum Entry {
    Literal(u32),
    Label(u32),
    ExtendedLiteral(u32),
    Value(Source),
}

impl Entry {
    pub fn to_label(&self) -> u32 {
        match *self {
            Entry::Label(i) => i,
            _ => panic!(),
        }
    }
    pub fn into_register(&self) -> Register {
        match self {
            Entry::Value(s) => match *s {
                Source::X(i) => Register::X(i),
                Source::Y(i) => Register::Y(i),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    pub fn into_value(&self) -> Source {
        match *self {
            Entry::ExtendedLiteral(i) => Source::ExtendedLiteral(i),
            Entry::Value(s) => s,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Register {
    X(RegisterX),
    Y(RegisterY),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FRegister {
    FloatReg(FloatRegs),
    ExtendedLiteral(u32),
    X(RegisterX),
    Y(RegisterY),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Source {
    Constant(u32), // index (max as u16? would shrink us down to 32)
    ExtendedLiteral(u32),
    X(RegisterX),
    Y(RegisterY),
}

// temporary because no specialization
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Size {
    Constant(u32),
    Literal(u32),
    X(RegisterX),
    Y(RegisterY),
}

impl Size {
    // FIXME: try to eliminate Size
    pub fn to_val(self, context: &crate::process::ExecutionContext) -> crate::value::Term {
        match self {
            Size::Literal(i) => crate::value::Term::uint(&context.heap, i),
            Size::Constant(i) => context.expand_arg(Source::Constant(i)),
            Size::X(i) => context.expand_arg(Source::X(i)),
            Size::Y(i) => context.expand_arg(Source::Y(i)),
        }
    }
}

// TODO: can't derive Copy yet since we have extended list which needs cloning
// TODO: is_ instructions can probably be coerced into Register instead of Source
// TODO: specialize bifs with f=0 as head vs body (ones jump, others don't)
// TODO: specialize arith into ops, and sub-specialize "increments" source + const

// TUPLE ELEMENTS: u24

// we could bitmask Source. In which case it would fit current loader size.
// type can be either const, x or y (needs two bits). Meaning we'd get a u30 for the payload.
// We could also embed it into Term, by abusing the Special type -- anything that would make
// Constant fit into the instruction and not require a separate table would be good.
// represent Vec<u8> string buffers as pointers into the string table (a slice would work?).

#[derive(Clone, Debug)]
pub enum Instruction {
    FuncInfo {
        // TODO: these are constants but could be Atom
        module: Source,
        function: Source,
        arity: Arity,
    },
    Call {
        arity: Arity,
        label: Label,
    },
    CallLast {
        arity: Arity,
        label: Label,
        words: Regs, // TODO: register count?
    },
    CallOnly {
        arity: Arity,
        label: Label,
    },
    CallExt {
        arity: Arity,
        destination: usize, // index in exports
    },
    CallExtOnly {
        arity: Arity,
        destination: usize,
    },
    CallExtLast {
        arity: Arity,
        destination: usize,
        words: Regs, // TODO: register count
    },
    Bif0 {
        bif: Bif,
        reg: Register,
    },
    Bif1 {
        fail: Label,
        bif: Bif,
        arg1: Source, // const or reg
        reg: Register,
    },
    Bif2 {
        fail: Label,
        bif: Bif,
        arg1: Source, // const or reg
        arg2: Source,
        reg: Register,
    },
    Allocate {
        stackneed: Regs,
        live: Regs,
    },
    AllocateHeap {
        stackneed: Regs,
        heapneed: Regs,
        live: Regs,
    },
    AllocateZero {
        stackneed: Regs,
        live: Regs,
    },
    AllocateHeapZero {
        stackneed: Regs,
        heapneed: Regs,
        live: Regs,
    },
    TestHeap {
        // TODO: temporarily skipped
        // heapneed: Regs,
        live: Regs,
    },
    Init {
        // maybe just Y?
        n: Register,
    },
    Deallocate {
        n: Regs,
    },
    Return,
    Send,
    RemoveMessage,
    Timeout,
    LoopRec {
        label: Label,
        source: Source, // TODO
    },
    LoopRecEnd {
        label: Label,
    },
    Wait {
        label: Label,
    },
    WaitTimeout {
        label: Label,
        time: Source,
    },
    IsLt {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    IsGe {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    IsEq {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    IsNe {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    IsEqExact {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    IsNeExact {
        label: Label,
        arg1: Source,
        arg2: Source,
    },
    // TODO: make all type checks use Register?
    IsInteger {
        label: Label,
        arg1: Source,
    },
    IsFloat {
        label: Label,
        arg1: Source,
    },
    IsNumber {
        label: Label,
        arg1: Source,
    },
    IsAtom {
        label: Label,
        arg1: Source,
    },
    IsPid {
        label: Label,
        arg1: Source,
    },
    IsReference {
        label: Label,
        arg1: Source,
    },
    IsPort {
        label: Label,
        arg1: Source,
    },
    IsNil {
        label: Label,
        arg1: Source,
    },
    IsBinary {
        label: Label,
        arg1: Source,
    },
    IsList {
        label: Label,
        arg1: Source,
    },
    IsNonemptyList {
        label: Label,
        arg1: Source,
    },
    IsTuple {
        label: Label,
        arg1: Source,
    },
    TestArity {
        label: Label,
        arg1: Source,
        arity: u32,
    },
    SelectVal {
        arg: Source,
        fail: Label,
        destinations: ExtendedList,
    },
    JumpOnVal {
        // SelectVal optimized with a jump table
        arg: Source,
        fail: Label,
        table: JumpTable,
        min: i32,
    },
    SelectTupleArity {
        arg: Source,
        fail: Label,
        arities: ExtendedList,
    },
    Jump(Label),
    Catch {
        // TODO: always y
        register: Register,
        fail: Label,
    },
    CatchEnd {
        // TODO: always y
        register: Register,
    },
    Move {
        source: Source, // register or constant
        destination: Register,
    },
    GetList {
        source: Register,
        head: Register,
        tail: Register,
    },
    GetTupleElement {
        source: Register, // maybe just register?
        element: u32,     // max tuple size
        destination: Register,
    },
    SetTupleElement {
        new_element: Source, // maybe just register?
        tuple: Source,
        position: u32, // max tuple size
    },
    PutList {
        head: Source,
        tail: Source,
        destination: Register,
    },
    //PutTuple and Put were rewritten into PutTuple2,
    // PutTuple,
    // Put,
    PutTuple2 {
        source: Register,
        list: ExtendedList,
    },
    // optimize badmatch and case_end to always move into X first? like makeops
    Badmatch {
        value: Source,
    },
    IfEnd,
    CaseEnd {
        value: Source,
    },
    CallFun {
        arity: Arity,
    },
    IsFunction {
        label: Label,
        arg1: Source,
    },
    BsPutInteger {
        fail: Label,
        size: Source,
        unit: u16,
        flags: bitstring::Flag, // not used?
        source: Source,         // maybe always register
    },
    BsPutBinary {
        fail: Label,
        size: Source,
        unit: u16,
        flags: bitstring::Flag, // not used?
        source: Source,         // maybe always register
    },
    BsPutFloat {
        fail: Label,
        size: Source,
        unit: u16,
        flags: bitstring::Flag, // not used?
        source: Source,         // maybe always register
    },
    BsPutString {
        binary: Vec<u8>,
    },
    Fclearerror,
    Fcheckerror,
    Fmove {
        source: FRegister,
        destination: FRegister,
    },
    Fconv {
        source: FRegister,
        destination: FloatRegs,
    },
    Fadd {
        fail: Label,
        a: FloatRegs,
        b: FloatRegs,
        destination: FloatRegs,
    },
    Fsub {
        fail: Label,
        a: FloatRegs,
        b: FloatRegs,
        destination: FloatRegs,
    },
    Fmul {
        fail: Label,
        a: FloatRegs,
        b: FloatRegs,
        destination: FloatRegs,
    },
    Fdiv {
        fail: Label,
        a: FloatRegs,
        b: FloatRegs,
        destination: FloatRegs,
    },
    Fnegate {
        source: FloatRegs,
        destination: FloatRegs,
    },
    MakeFun2 {
        i: Regs, // points to lambda
    },
    Try {
        // TODO: only y
        register: Register,
        fail: Label,
    },
    TryEnd {
        // TODO: only y
        register: Register,
    },
    TryCase {
        // TODO: only y
        register: Register,
    },
    TryCaseEnd {
        value: Source,
    },
    Raise {
        trace: Register, // xy
        value: Source,
    },
    BsInit2 {
        fail: Label,
        size: Size,
        words: Regs,
        regs: Regs,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsAdd {
        fail: Label,
        size1: Source,
        size2: Source,
        unit: u16,
        destination: Register,
    },
    Apply {
        arity: Arity,
    },
    ApplyLast {
        arity: Arity,
        nwords: Regs,
    },
    IsBoolean {
        label: Label,
        arg1: Source,
    },
    IsFunction2 {
        label: Label,
        arg1: Source,
        arity: Source,
    },
    /*
    BsStartMatch2 {
        fail: Label,
        source: Source,
        live: Regs,
        slots: u32,
        destination: Register,
    },
    BsContextToBinary {
        context: Register, // it reuses the same reg
    },
    BsSave2 {
        context: Register,
        slot: Source,
    },
    BsRestore2 {
        context: Register,
        slot: Source,
    },
    */
    BsGetInteger2 {
        fail: Label,
        ms: Register,
        live: Regs,
        size: Source,
        unit: u16,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsGetFloat2 {
        fail: Label,
        ms: Register,
        live: Regs,
        size: Source,
        unit: u16,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsGetBinary2 {
        fail: Label,
        ms: Register,
        live: Regs,
        size: Source,
        unit: u16,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsSkipBits2 {
        fail: Label,
        ms: Register,
        size: Source,
        unit: u16,
        flags: bitstring::Flag,
    },
    BsTestTail2 {
        fail: Label,
        ms: Register,
        bits: u32,
    },
    GcBif1 {
        label: Label,
        live: Regs,
        bif: Bif,
        arg1: Source,
        reg: Register,
    },
    GcBif2 {
        label: Label,
        live: Regs,
        bif: Bif,
        arg1: Source,
        arg2: Source,
        reg: Register,
    },
    IsBitstr {
        label: Label,
        arg1: Source,
    },
    BsTestUnit {
        fail: Label,
        context: Register,
        unit: u16,
    },
    BsMatchString {
        fail: Label,
        context: Register,
        bits: u32,
        string: Vec<u8>,
    },
    BsInitWritable,
    BsAppend {
        // 8 params, ugh, BEAM shrinks it
        // bs_append Fail Size Extra Live Unit Bin Flags Dst \
        // move Bin x | i_bs_append Fail Extra Live Unit Size Dst
        fail: Label,
        size: Source,
        extra: Regs,
        live: Regs,
        // untagged integer (12 bits) -- can be packed
        unit: u16,
        bin: Source,
        // flags: bitstring::Flag,
        destination: Register,
    },
    BsPrivateAppend {
        // gets rewritten to no flags
        fail: Label,
        size: Source,
        // untagged integer (12 bits) -- can be packed
        unit: u16,
        bin: Source,
        // flags: bitstring::Flag,
        destination: Register,
    },
    Trim {
        n: Regs,
        // remaining: Regs, it seems unused in BEAM
    },
    BsInitBits {
        // gets rewritten into a following move
        fail: Label,
        size: Size,
        words: Regs,
        regs: Regs,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsGetUtf8 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsGetUtf16 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
        destination: Register,
    },
    BsGetUtf32 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
        destination: Register,
    },
    // gets rewritten onto gets
    BsSkipUtf8 {
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
    },
    BsSkipUtf16 {
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
    },
    BsSkipUtf32 {
        fail: Label,
        ms: Register,
        size: Size,
        flags: bitstring::Flag,
    },
    BsUtf8Size {
        fail: Label,
        source: Register,
        destination: Register,
    },
    BsUtf16Size {
        fail: Label,
        source: Register,
        destination: Register,
    },
    BsPutUtf8 {
        fail: Label,
        flags: bitstring::Flag,
        source: Register,
    },
    BsPutUtf16 {
        fail: Label,
        flags: bitstring::Flag,
        source: Register,
    },
    BsPutUtf32 {
        // gets rewritten into 3 args
        fail: Label,
        flags: bitstring::Flag,
        source: Register,
    },
    // OnLoad, load time instr!
    // Modified in OTP 21 because it turns out that we don't need the label after all.
    RecvMark,
    // Modified in OTP 21 because it turns out that we don't need the label after all.
    RecvSet,
    GcBif3 {
        label: Label,
        live: Regs,
        bif: Bif,
        arg1: Source,
        arg2: Source,
        arg3: Source,
        reg: Register,
    },
    PutMapAssoc {
        // fail: Label // obsolete past OTP 20 since we know it's preceeded by is_map
        map: Source,
        destination: Register,
        live: Regs,
        rest: ExtendedList,
    },
    PutMapExact {
        // fail: Label // obsolete past OTP 20 since we know it's preceeded by is_map
        map: Source,
        destination: Register,
        live: Regs,
        rest: ExtendedList,
    },
    IsMap {
        label: Label,
        arg1: Source,
    },
    HasMapFields {
        label: Label,
        source: Register,
        rest: ExtendedList,
    },
    GetMapElements {
        label: Label,
        source: Register,
        rest: ExtendedList,
    },
    IsTaggedTuple {
        fail: Label,
        source: Register,
        arity: u32,
        atom: Source, // TODO: just use Atom, it's constant atm
    },
    BuildStacktrace,
    RawRaise,
    GetHd {
        source: Register,
        head: Register,
    },
    GetTl {
        source: Register,
        tail: Register,
    },
    BsGetTail {
        context: Register,
        destination: Register,
        live: Regs,
    },
    BsStartMatch3 {
        fail: Label,
        source: Register,
        live: Regs,
        destination: Register,
    },
    BsGetPosition {
        context: Register,
        destination: Register,
        live: Regs,
    },
    BsSetPosition {
        context: Register,
        position: Source,
    },
    Swap {
        a: Register,
        b: Register,
    },
}
