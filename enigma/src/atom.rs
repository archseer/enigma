use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

/// Maximum character length of an atom.
pub const MAX_ATOM_CHARS: usize = 255;

#[derive(Debug, Clone, Copy)]
pub struct Atom(pub u32);

impl Atom {
    /// Construct a new atom from a raw string.
    pub fn new(val: &str) -> Self {
        assert!(val.len() <= MAX_ATOM_CHARS);
        Atom(ATOMS.write().from_str(val))
    }
}

// impl From<&str> for Atom {
//     #[inline]
//     fn from(value: &str) -> Self {
//         Atom::new(val)
//     }
// }

// impl From<String> for Atom {
//     #[inline]
//     fn from(value: String) -> Self {
//         Atom::new(val)
//     }
// }

#[derive(Debug)]
pub struct AtomTable {
    /// Direct mapping string to atom index
    ids: HashMap<&'static str, u32>,

    /// Reverse mapping atom index to string (sorted by index)
    names: Vec<&'static str>,
}

impl AtomTable {
    pub fn new() -> AtomTable {
        AtomTable {
            ids: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: &str) -> u32 {
        // this is to only have it allocated once (instead of twice, once for each index)
        let name = Box::leak(String::from(name).into_boxed_str());
        let index = self.names.len() as u32;
        self.ids.insert(name, index);
        self.names.push(name);
        index
    }

    pub fn lookup(&self, val: &str) -> Option<u32> {
        self.ids.get(val).copied()
    }

    pub fn cmp(&self, index1: u32, index2: u32) -> std::cmp::Ordering {
        let val1 = &self.names[index1 as usize];
        let val2 = &self.names[index2 as usize];
        val1.cmp(&val2)
    }

    /// Allocate new atom in the atom table or find existing.
    pub fn from_str(&mut self, val: &str) -> u32 {
        self.lookup(val).unwrap_or_else(|| self.insert(val))
    }

    pub fn to_str(&self, index: u32) -> Option<&'static str> {
        if index >= self.names.len() as u32 {
            return None;
        }
        self.names.get(index as usize).copied()
    }
}

pub static ATOMS: Lazy<RwLock<AtomTable>> = Lazy::new(|| {
    let mut atoms = AtomTable::new();
    atoms.insert("nil"); // 0
    atoms.insert("true"); // 1
    atoms.insert("false"); // 2
    atoms.insert("undefined"); // 3
    atoms.insert("value"); // 4
    atoms.insert("all"); // 5

    atoms.insert("normal");
    atoms.insert("internal_error");
    atoms.insert("badarg");
    atoms.insert("badarith");
    atoms.insert("badmatch");
    atoms.insert("function_clause");
    atoms.insert("case_clause");
    atoms.insert("if_clause");
    atoms.insert("undef");
    atoms.insert("badfun");
    atoms.insert("badarity");
    atoms.insert("timeout_value");
    atoms.insert("no_proc");
    atoms.insert("not_alive");
    atoms.insert("system_limit");
    atoms.insert("try_clause");
    atoms.insert("not_sup");
    atoms.insert("badmap");
    atoms.insert("badkey");

    atoms.insert("nocatch");

    atoms.insert("exit");
    atoms.insert("error");
    atoms.insert("throw");

    atoms.insert("file");
    atoms.insert("line");

    atoms.insert("ok");

    atoms.insert("erlang");
    atoms.insert("apply");

    atoms.insert("trap_exit");
    atoms.insert("start");
    atoms.insert("kill");
    atoms.insert("killed");

    atoms.insert("not_loaded");
    atoms.insert("noproc");
    atoms.insert("file_info");

    atoms.insert("device");
    atoms.insert("directory");
    atoms.insert("regular");
    atoms.insert("symlink");
    atoms.insert("other");

    atoms.insert("read");
    atoms.insert("write");
    atoms.insert("read_write");
    atoms.insert("none");

    atoms.insert("down");
    atoms.insert("process");

    atoms.insert("link");
    atoms.insert("monitor");

    atoms.insert("current_function");
    atoms.insert("current_location");
    atoms.insert("current_stacktrace");
    atoms.insert("initial_call");
    atoms.insert("status");
    atoms.insert("messages");
    atoms.insert("message_queue_len");
    atoms.insert("message_queue_data");
    atoms.insert("links");
    atoms.insert("monitored_by");
    atoms.insert("dictionary");
    atoms.insert("error_handler");
    atoms.insert("heap_size");
    atoms.insert("stack_size");
    atoms.insert("memory");
    atoms.insert("garbage_collection");
    atoms.insert("garbage_collection_info");
    atoms.insert("group_leader");
    atoms.insert("reductions");
    atoms.insert("priority");
    atoms.insert("trace");
    atoms.insert("binary");
    atoms.insert("sequential_trace_token");
    atoms.insert("catch_level");
    atoms.insert("backtrace");
    atoms.insert("last_calls");
    atoms.insert("total_heap_size");
    atoms.insert("suspending");
    atoms.insert("min_heap_size");
    atoms.insert("min_bin_vheap_size");
    atoms.insert("max_heap_size");
    atoms.insert("magic_ref");
    atoms.insert("fullsweep_after");
    atoms.insert("registered_name");

    atoms.insert("enoent");

    atoms.insert("badfile");

    atoms.insert("max");
    atoms.insert("high");
    atoms.insert("medium");
    atoms.insert("low");

    atoms.insert("nonode@nohost");

    atoms.insert("port");
    atoms.insert("time_offset");

    atoms.insert("bag");
    atoms.insert("duplicate_bag");
    atoms.insert("set");
    atoms.insert("ordered_set");
    atoms.insert("keypos");
    atoms.insert("write_concurrency");
    atoms.insert("read_concurrency");
    atoms.insert("heir");
    atoms.insert("public");
    atoms.insert("private");
    atoms.insert("protected");
    atoms.insert("named_table");
    atoms.insert("compressed");

    atoms.insert("undefined_function");

    atoms.insert("os_type");
    atoms.insert("win32");
    atoms.insert("unix");

    atoms.insert("info");
    atoms.insert("flush");

    atoms.insert("erts_internal");
    atoms.insert("flush_monitor_messages");

    atoms.insert("$_");
    atoms.insert("$$");
    atoms.insert("_");

    atoms.insert("const");
    atoms.insert("and");
    atoms.insert("or");
    atoms.insert("andalso");
    atoms.insert("andthen");
    atoms.insert("orelse");
    atoms.insert("self");
    atoms.insert("is_seq_trace");
    atoms.insert("set_seq_token");
    atoms.insert("get_seq_token");
    atoms.insert("return_trace");
    atoms.insert("exception_trace");
    atoms.insert("display");
    atoms.insert("process_dump");
    atoms.insert("enable_trace");
    atoms.insert("disable_trace");
    atoms.insert("caller");
    atoms.insert("silent");
    atoms.insert("set_tcw");
    atoms.insert("set_tcw_fake");

    atoms.insert("is_atom");
    atoms.insert("is_float");
    atoms.insert("is_integer");
    atoms.insert("is_list");
    atoms.insert("is_number");
    atoms.insert("is_pid");
    atoms.insert("is_port");
    atoms.insert("is_reference");
    atoms.insert("is_tuple");
    atoms.insert("is_map");
    atoms.insert("is_binary");
    atoms.insert("is_function");
    atoms.insert("is_record");

    atoms.insert("abs");
    atoms.insert("element");
    atoms.insert("hd");
    atoms.insert("length");
    atoms.insert("node");
    atoms.insert("round");
    atoms.insert("size");
    atoms.insert("map_size");
    atoms.insert("map_get");
    atoms.insert("is_map_key");
    atoms.insert("bit_size");
    atoms.insert("tl");
    atoms.insert("trunc");
    atoms.insert("float");
    atoms.insert("+");
    atoms.insert("-");
    atoms.insert("*");
    atoms.insert("/");
    atoms.insert("div");
    atoms.insert("rem");
    atoms.insert("band");
    atoms.insert("bor");
    atoms.insert("bxor");
    atoms.insert("bnot");
    atoms.insert("bsl");
    atoms.insert("bsr");
    atoms.insert(">");
    atoms.insert(">=");
    atoms.insert("<");
    atoms.insert("=<");
    atoms.insert("=:=");
    atoms.insert("==");
    atoms.insert("=/=");
    atoms.insert("/=");
    atoms.insert("not");
    atoms.insert("xor");

    atoms.insert("native");
    atoms.insert("second");
    atoms.insert("millisecond");
    atoms.insert("microsecond");
    atoms.insert("perf_counter");

    atoms.insert("exiting");
    atoms.insert("garbage_collecting");
    atoms.insert("waiting");
    atoms.insert("running");
    atoms.insert("runnable");
    atoms.insert("suspended");

    atoms.insert("module");
    atoms.insert("md5");
    atoms.insert("exports");
    atoms.insert("functions");
    atoms.insert("nifs");
    atoms.insert("attributes");
    atoms.insert("compile");
    atoms.insert("native_addresses");

    atoms.insert("hipe_architecture");
    atoms.insert("new");
    atoms.insert("infinity");

    atoms.insert("enotdir");

    atoms.insert("spawn");
    atoms.insert("tty_sl -c -e");
    atoms.insert("fd");
    atoms.insert("system_version");

    atoms.insert("unicode");
    atoms.insert("utf8");
    atoms.insert("latin1");

    atoms.insert("command");
    atoms.insert("data");

    atoms.insert("DOWN");
    atoms.insert("UP");
    atoms.insert("EXIT");

    atoms.insert("on_heap");
    atoms.insert("off_heap");

    atoms.insert("system_logger");

    atoms.insert("$end_of_table");
    atoms.insert("iterator");

    atoms.insert("match");
    atoms.insert("nomatch");

    atoms.insert("exclusive");
    atoms.insert("append");
    atoms.insert("sync");
    atoms.insert("skip_type_check");

    atoms.insert("purify");

    atoms.insert("acquired");
    atoms.insert("busy");
    atoms.insert("lock_order_violation");

    atoms.insert("eof");

    atoms.insert("beam_lib");
    atoms.insert("version");

    atoms.insert("type");
    atoms.insert("pid");
    atoms.insert("new_index");
    atoms.insert("new_uniq");
    atoms.insert("index");
    atoms.insert("uniq");
    atoms.insert("env");
    atoms.insert("refc");
    atoms.insert("arity");
    atoms.insert("name");
    atoms.insert("local");
    atoms.insert("external");
    atoms.insert("machine");
    atoms.insert("otp_release");

    atoms.insert("bof");
    atoms.insert("cur");

    atoms.insert("no_integer");

    atoms.insert("endian");
    atoms.insert("little");
    atoms.insert("big");

    atoms.insert("protection");

    atoms.insert("trim");
    atoms.insert("trim_all");
    atoms.insert("global");
    atoms.insert("scope");

    RwLock::new(atoms)
});

pub const TRUE: u32 = 1;
pub const FALSE: u32 = 2;
pub const UNDEFINED: u32 = 3;
pub const VALUE: u32 = 4;
pub const ALL: u32 = 5;

pub const NORMAL: u32 = 6;
pub const INTERNAL_ERROR: u32 = 7;
pub const BADARG: u32 = 8;
pub const BADARITH: u32 = 9;
pub const BADMATCH: u32 = 10;
pub const FUNCTION_CLAUSE: u32 = 11;
pub const CASE_CLAUSE: u32 = 12;
pub const IF_CLAUSE: u32 = 13;
pub const UNDEF: u32 = 14;
pub const BADFUN: u32 = 15;
pub const BADARITY: u32 = 16;
pub const TIMEOUT_VALUE: u32 = 17;
pub const NO_PROC: u32 = 18;
pub const NOT_ALIVE: u32 = 19;
pub const SYSTEM_LIMIT: u32 = 20;
pub const TRY_CLAUSE: u32 = 21;
pub const NOT_SUP: u32 = 22;
pub const BADMAP: u32 = 23;
pub const BADKEY: u32 = 24;

pub const NOCATCH: u32 = 25;

pub const EXIT: u32 = 26;
pub const ERROR: u32 = 27;
pub const THROW: u32 = 28;

pub const FILE: u32 = 29;
pub const LINE: u32 = 30;

pub const OK: u32 = 31;

pub const ERLANG: u32 = 32;
pub const APPLY: u32 = 33;

pub const TRAP_EXIT: u32 = 34;
pub const START: u32 = 35;
pub const KILL: u32 = 36;
pub const KILLED: u32 = 37;

pub const NOT_LOADED: u32 = 38;
pub const NOPROC: u32 = 39;
pub const FILE_INFO: u32 = 40;

pub const DEVICE: u32 = 41;
pub const DIRECTORY: u32 = 42;
pub const REGULAR: u32 = 43;
pub const SYMLINK: u32 = 44;
pub const OTHER: u32 = 45;

pub const READ: u32 = 46;
pub const WRITE: u32 = 47;
pub const READ_WRITE: u32 = 48;
pub const NONE: u32 = 49;

pub const DOWN: u32 = 50;
pub const PROCESS: u32 = 51;

pub const LINK: u32 = 52;
pub const MONITOR: u32 = 53;

pub const CURRENT_FUNCTION: u32 = 54;
pub const CURRENT_LOCATION: u32 = 55;
pub const CURRENT_STACKTRACE: u32 = 56;
pub const INITIAL_CALL: u32 = 57;
pub const STATUS: u32 = 58;
pub const MESSAGES: u32 = 59;
pub const MESSAGE_QUEUE_LEN: u32 = 60;
pub const MESSAGE_QUEUE_DATA: u32 = 61;
pub const LINKS: u32 = 62;
pub const MONITORED_BY: u32 = 63;
pub const DICTIONARY: u32 = 64;
pub const ERROR_HANDLER: u32 = 65;
pub const HEAP_SIZE: u32 = 66;
pub const STACK_SIZE: u32 = 67;
pub const MEMORY: u32 = 68;
pub const GARBAGE_COLLECTION: u32 = 69;
pub const GARBAGE_COLLECTION_INFO: u32 = 70;
pub const GROUP_LEADER: u32 = 71;
pub const REDUCTIONS: u32 = 72;
pub const PRIORITY: u32 = 73;
pub const TRACE: u32 = 74;
pub const BINARY: u32 = 75;
pub const SEQUENTIAL_TRACE_TOKEN: u32 = 76;
pub const CATCH_LEVEL: u32 = 77;
pub const BACKTRACE: u32 = 78;
pub const LAST_CALLS: u32 = 79;
pub const TOTAL_HEAP_SIZE: u32 = 80;
pub const SUSPENDING: u32 = 81;
pub const MIN_HEAP_SIZE: u32 = 82;
pub const MIN_BIN_VHEAP_SIZE: u32 = 83;
pub const MAX_HEAP_SIZE: u32 = 84;
pub const MAGIC_REF: u32 = 85;
pub const FULLSWEEP_AFTER: u32 = 86;
pub const REGISTERED_NAME: u32 = 87;

pub const ENOENT: u32 = 88;

pub const BADFILE: u32 = 89;

pub const MAX: u32 = 90;
pub const HIGH: u32 = 91;
pub const MEDIUM: u32 = 92;
pub const LOW: u32 = 93;

pub const NO_NODE_NO_HOST: u32 = 94;

pub const PORT: u32 = 95;
pub const TIME_OFFSET: u32 = 96;

pub const BAG: u32 = 97;
pub const DUPLICATE_BAG: u32 = 98;
pub const SET: u32 = 99;
pub const ORDERED_SET: u32 = 100;
pub const KEYPOS: u32 = 101;
pub const WRITE_CONCURRENCY: u32 = 102;
pub const READ_CONCURRENCY: u32 = 103;
pub const HEIR: u32 = 104;
pub const PUBLIC: u32 = 105;
pub const PRIVATE: u32 = 106;
pub const PROTECTED: u32 = 107;
pub const NAMED_TABLE: u32 = 108;
pub const COMPRESSED: u32 = 109;

pub const UNDEFINED_FUNCTION: u32 = 110;

pub const OS_TYPE: u32 = 111;
pub const WIN32: u32 = 112;
pub const UNIX: u32 = 113;

pub const INFO: u32 = 114;
pub const FLUSH: u32 = 115;
pub const ERTS_INTERNAL: u32 = 116;
pub const FLUSH_MONITOR_MESSAGES: u32 = 117;

pub const DOLLAR_UNDERSCORE: u32 = 118;
pub const DOLLAR_DOLLAR: u32 = 119;
pub const UNDERSCORE: u32 = 120;

pub const CONST: u32 = 120;
pub const AND: u32 = 121;
pub const OR: u32 = 122;
pub const ANDALSO: u32 = 123;
pub const ANDTHEN: u32 = 124;
pub const ORELSE: u32 = 125;
pub const SELF: u32 = 126;
pub const MESSAGE: u32 = 127;
pub const IS_SEQ_TRACE: u32 = 128;
pub const SET_SEQ_TOKEN: u32 = 129;
pub const GET_SEQ_TOKEN: u32 = 130;
pub const RETURN_TRACE: u32 = 131;
pub const EXCEPTION_TRACE: u32 = 132;
pub const DISPLAY: u32 = 133;
pub const PROCESS_DUMP: u32 = 134;
pub const ENABLE_TRACE: u32 = 135;
pub const DISABLE_TRACE: u32 = 136;
pub const CALLER: u32 = 137;
pub const SILENT: u32 = 138;
pub const SET_TCW: u32 = 139;
pub const SET_TCW_FAKE: u32 = 140;

pub const IS_ATOM: u32 = 141;
pub const IS_FLOAT: u32 = 142;
pub const IS_INTEGER: u32 = 143;
pub const IS_LIST: u32 = 144;
pub const IS_NUMBER: u32 = 145;
pub const IS_PID: u32 = 146;
pub const IS_PORT: u32 = 147;
pub const IS_REFERENCE: u32 = 148;
pub const IS_TUPLE: u32 = 149;
pub const IS_MAP: u32 = 150;
pub const IS_BINARY: u32 = 151;
pub const IS_FUNCTION: u32 = 152;
pub const IS_RECORD: u32 = 153;

pub const ABS: u32 = 154;
pub const ELEMENT: u32 = 155;
pub const HD: u32 = 156;
pub const LENGTH: u32 = 157;
pub const NODE: u32 = 158;
pub const ROUND: u32 = 159;
pub const SIZE: u32 = 160;
pub const MAP_SIZE: u32 = 161;
pub const MAP_GET: u32 = 162;
pub const IS_MAP_KEY: u32 = 163;
pub const BIT_SIZE: u32 = 164;
pub const TL: u32 = 165;
pub const TRUNC: u32 = 166;
pub const FLOAT: u32 = 167;
pub const PLUS: u32 = 168;
pub const MINUS: u32 = 169;
pub const TIMES: u32 = 170;
pub const DIV: u32 = 171;
pub const INTDIV: u32 = 172;
pub const REM: u32 = 173;
pub const BAND: u32 = 174;
pub const BOR: u32 = 175;
pub const BXOR: u32 = 176;
pub const BNOT: u32 = 177;
pub const BSL: u32 = 178;
pub const BSR: u32 = 179;
pub const GT: u32 = 180;
pub const GE: u32 = 181;
pub const LT: u32 = 182;
pub const LE: u32 = 183;
pub const EQ: u32 = 184;
pub const EQEQ: u32 = 185;
pub const NEQ: u32 = 186;
pub const NEQEQ: u32 = 187;
pub const NOT: u32 = 188;
pub const XOR: u32 = 189;

pub const NATIVE: u32 = 190;
pub const SECOND: u32 = 191;
pub const MILLISECOND: u32 = 192;
pub const MICROSECOND: u32 = 193;
pub const PERF_COUNTER: u32 = 194;

pub const EXITING: u32 = 195;
pub const GARBAGE_COLLECTING: u32 = 196;
pub const WAITING: u32 = 197;
pub const RUNNING: u32 = 198;
pub const RUNNABLE: u32 = 199;
pub const SUSPENDED: u32 = 200;

pub const MODULE: u32 = 201;
pub const MD5: u32 = 202;
pub const EXPORTS: u32 = 203;
pub const FUNCTIONS: u32 = 204;
pub const NIFS: u32 = 205;
pub const ATTRIBUTES: u32 = 206;
pub const COMPILE: u32 = 207;
pub const NATIVE_ADDRESSES: u32 = 208;

pub const HIPE_ARCHITECTURE: u32 = 209;
pub const NEW: u32 = 210;
pub const INFINITY: u32 = 211;

pub const ENOTDIR: u32 = 212;
pub const SPAWN: u32 = 213;
pub const TTY_SL: u32 = 214;
pub const FD: u32 = 215;
pub const SYSTEM_VERSION: u32 = 216;

pub const UNICODE: u32 = 217;
pub const UTF8: u32 = 218;
pub const LATIN1: u32 = 219;

pub const COMMAND: u32 = 220;
pub const DATA: u32 = 221;

pub const DOWN_U: u32 = 222;
pub const UP_U: u32 = 223;
pub const EXIT_U: u32 = 224;

pub const ON_HEAP: u32 = 225;
pub const OFF_HEAP: u32 = 226;

pub const SYSTEM_LOGGER: u32 = 227;

pub const DOLLAR_END_OF_TABLE: u32 = 228;
pub const ITERATOR: u32 = 229;

pub const MATCH: u32 = 230;
pub const NOMATCH: u32 = 231;

pub const EXCLUSIVE: u32 = 232;
pub const APPEND: u32 = 233;
pub const SYNC: u32 = 234;
pub const SKIP_TYPE_CHECK: u32 = 235;

pub const PURIFY: u32 = 236;

pub const ACQUIRED: u32 = 237;
pub const BUSY: u32 = 238;
pub const LOCK_ORDER_VIOLATION: u32 = 239;

pub const EOF: u32 = 240;

pub const BEAM_LIB: u32 = 241;
pub const VERSION: u32 = 242;

pub const TYPE: u32 = 243;
pub const PID: u32 = 244;
pub const NEW_INDEX: u32 = 245;
pub const NEW_UNIQ: u32 = 246;
pub const INDEX: u32 = 247;
pub const UNIQ: u32 = 248;
pub const ENV: u32 = 249;
pub const REFC: u32 = 250;
pub const ARITY: u32 = 251;
pub const NAME: u32 = 252;
pub const LOCAL: u32 = 253;
pub const EXTERNAL: u32 = 254;
pub const MACHINE: u32 = 255;
pub const OTP_RELEASE: u32 = 256;

pub const BOF: u32 = 257;
pub const CUR: u32 = 258;

pub const NO_INTEGER: u32 = 259;

pub const ENDIAN: u32 = 260;
pub const LITTLE: u32 = 261;
pub const BIG: u32 = 262;

pub const PROTECTION: u32 = 263;

pub const TRIM: u32 = 264;
pub const TRIM_ALL: u32 = 265;
pub const GLOBAL: u32 = 266;
pub const SCOPE: u32 = 267;

#[inline]
pub fn cmp(index1: u32, index2: u32) -> std::cmp::Ordering {
    ATOMS.read().cmp(index1, index2)
}
#[inline]
pub fn from_str(val: &str) -> u32 {
    // unfortunately, cache hits need the write lock to avoid race conditions
    ATOMS.write().from_str(val)
}

#[inline]
pub fn to_str(index: u32) -> Option<&'static str> {
    ATOMS.read().to_str(index)
}
