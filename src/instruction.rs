// Label could be Option<NonZerou32> in some places?
use crate::bif;
//use crate::loader::Source;

// bs_start_match2, bs_save2, bs_restore2, bs_context_to_binary are deprecated on OTP22

// 184 bytes
// 56 bytes with shrunken Source, 48 with flags removed on BsInit
// 88 bytes with Term consts embedded

pub type Arity = u8;
pub type Label = u32;
pub type Atom = u32;

pub type FloatRegs = u8;
// shrink these from u16 to u8 once swap instruction is used
pub type Regs = u16;
pub type RegisterX = Regs;
pub type RegisterY = Regs;

pub type ExtendedList = Box<Vec<Entry>>;

#[derive(Debug, Clone)]
pub enum Entry {
    Literal(u32),
    Label(u32),
    ExtendedLiteral(u32),
    Value(Source),
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

// TODO: can't derive Copy yet since we have extended list which needs cloning

// TUPLE ELEMENTS: u24

// we could bitmask Source. In which case it would fit current loader size.
// type can be either const, x or y (needs two bits). Meaning we'd get a u30 for the payload.
// pub type Source = u32;

#[derive(Clone)]
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
        bif: bif::Fn,
        reg: Register,
    },
    Bif1 {
        fail: Label,
        bif: bif::Fn,
        arg1: Source, // const or reg
        reg: Register,
    },
    Bif2 {
        fail: Label,
        bif: bif::Fn,
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
    SelectTupleArity {
        arg: Source,
        fail: Label,
        arities: ExtendedList,
    },
    Jump(Label),
    Catch,
    CatchEnd,
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
        flags: u8,      // not used? // TODO flags as bitstring::Flag (u8)
        source: Source, // maybe always register
    },
    BsPutBinary {
        fail: Label,
        size: Source,
        unit: u16,
        flags: u8,      // not used?
        source: Source, // maybe always register
    },
    BsPutFloat {
        fail: Label,
        size: Source,
        unit: u16,
        flags: u8,      // not used?
        source: Source, // maybe always register
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
        source: FloatRegs,
        destination: FloatRegs,
    },
    Fsub {
        fail: Label,
        source: FloatRegs,
        destination: FloatRegs,
    },
    Fmul {
        fail: Label,
        source: FloatRegs,
        destination: FloatRegs,
    },
    Fdiv {
        fail: Label,
        source: FloatRegs,
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
        flags: u8,
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
        flags: u8,
        destination: Register,
    },
    BsGetFloat2 {
        fail: Label,
        ms: Register,
        live: Regs,
        size: Source,
        unit: u16,
        flags: u8,
        destination: Register,
    },
    BsGetBinary2 {
        fail: Label,
        ms: Register,
        live: Regs,
        size: Source,
        unit: u16,
        flags: u8,
        destination: Register,
    },
    BsSkipBits2 {
        fail: Label,
        ms: Register,
        size: Source,
        unit: u16,
        flags: u8,
    },
    BsTestTail2 {
        fail: Label,
        ms: Register,
        bits: u32,
    },
    GcBif1 {
        label: Label,
        live: Regs,
        bif: bif::Fn,
        arg1: Source,
        reg: Register,
    },
    GcBif2 {
        label: Label,
        live: Regs,
        bif: bif::Fn,
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
        // flags: u8,
        destination: Register,
    },
    BsPrivateAppend {
        // gets rewritten to no flags
        fail: Label,
        size: Source,
        // untagged integer (12 bits) -- can be packed
        unit: u16,
        bin: Source,
        // flags: u8,
        destination: Register,
    },
    Trim {
        n: u32,
        remaining: u32,
    },
    BsInitBits {
        // gets rewritten into a following move
        fail: Label,
        size: Size,
        words: Regs,
        regs: Regs,
        flags: u8,
        destination: Register,
    },
    BsGetUtf8 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
        destination: Register,
    },
    BsGetUtf16 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
        destination: Register,
    },
    BsGetUtf32 {
        // gets rewritten into 3 args
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
        destination: Register,
    },
    // gets rewritten onto gets
    BsSkipUtf8 {
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
    },
    BsSkipUtf16 {
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
    },
    BsSkipUtf32 {
        fail: Label,
        ms: Register,
        size: Size,
        unit: u16,
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
        flags: u8,
        source: Register,
    },
    BsPutUtf16 {
        fail: Label,
        flags: u8,
        source: Register,
    },
    BsPutUtf32 {
        // gets rewritten into 3 args
        fail: Label,
        flags: u8,
        source: Register,
    },
    OnLoad,
    RecvMark {
        label: Label,
    },
    RecvSet {
        label: Label,
    },
    GcBif3 {
        label: Label,
        live: Regs,
        bif: bif::Fn,
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
}
