//use std::ptr;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::u16;

/// Maximum character length of an atom.
pub const MAX_ATOM_CHARS: usize = 255;

#[derive(Debug)]
pub struct Atom {
    /// Length of utf8-encoded atom name.
    pub len: u16,
    /// First 4 bytes used for comparisons
    pub ord0: u32, // TODO: use ord for comparison
    // TODO: Allocate these on atom heap or as a sequence of static blocks
    pub name: String,
}

impl Atom {
    /// Construct a new atom from a raw string.
    pub fn new(s: &str) -> Atom {
        let b = s.as_bytes();
        let mut ord0 = 0u32;

        // This might be particularly ugly. Erlang/OTP does this by preallocating
        // a minimum of 4 bytes and taking from them unconditionally.
        if !b.is_empty() {
            ord0 = u32::from(b[0]) << 24;
            if b.len() > 1 {
                ord0 |= u32::from(b[1]) << 16;
                if b.len() > 2 {
                    ord0 |= u32::from(b[2]) << 8;
                    if b.len() > 3 {
                        ord0 |= u32::from(b[3]);
                    }
                }
            }
        }

        assert!(s.len() <= u16::MAX as usize);
        Atom {
            len: s.len() as u16,
            ord0,
            name: s.to_string(),
        }
    }
}

/// Lookup table (generic for other types later)
#[derive(Debug)]
pub struct AtomTable {
    /// Direct mapping string to atom index
    index: RwLock<HashMap<String, u32>>,

    /// Reverse mapping atom index to string (sorted by index)
    index_r: RwLock<Vec<Atom>>,
}

/// Stores atom lookup tables.
impl AtomTable {
    pub fn new() -> AtomTable {
        AtomTable {
            index: RwLock::new(HashMap::new()),
            index_r: RwLock::new(Vec::new()),
        }
    }

    pub fn reserve(&self, _len: u32) {}

    pub fn register_atom(&self, val: &str) -> u32 {
        // TODO: probably keep a single lock for both
        let mut index_r = self.index_r.write();
        let index = index_r.len() as u32;
        self.index.write().insert(val.to_string(), index);
        index_r.push(Atom::new(val));
        index
    }

    pub fn lookup(&self, val: &str) -> Option<u32> {
        let atoms = self.index.read();

        if atoms.contains_key(val) {
            return Some(atoms[val]);
        }
        None
    }

    /// Allocate new atom in the atom table or find existing.
    pub fn from_str(&self, val: &str) -> u32 {
        // unfortunately, cache hits need the write lock to avoid race conditions
        let mut atoms = self.index.write();

        if atoms.contains_key(val) {
            return atoms[val];
        }

        let mut index_r = self.index_r.write();
        let index = index_r.len() as u32;
        atoms.insert(val.to_string(), index);
        index_r.push(Atom::new(val));
        index
    }

    pub fn to_str(&self, index: u32) -> Option<*const Atom> {
        let index_r = self.index_r.read();
        if index >= index_r.len() as u32 {
            return None;
        }
        Some(&index_r[index as usize] as *const Atom)
    }
}

pub static ATOMS: Lazy<AtomTable> = sync_lazy! {
    let atoms = AtomTable::new();
    atoms.register_atom("nil");   // 0
    atoms.register_atom("true");  // 1
    atoms.register_atom("false"); // 2
    atoms.register_atom("undefined"); // 3
    atoms.register_atom("value"); // 4
    atoms.register_atom("all"); // 5

    atoms.register_atom("normal");
    atoms.register_atom("internal_error");
    atoms.register_atom("badarg");
    atoms.register_atom("badarith");
    atoms.register_atom("badmatch");
    atoms.register_atom("function_clause");
    atoms.register_atom("case_clause");
    atoms.register_atom("if_clause");
    atoms.register_atom("undef");
    atoms.register_atom("badfun");
    atoms.register_atom("badarity");
    atoms.register_atom("timeout_value");
    atoms.register_atom("no_proc");
    atoms.register_atom("not_alive");
    atoms.register_atom("system_limit");
    atoms.register_atom("try_clause");
    atoms.register_atom("not_sup");
    atoms.register_atom("badmap");
    atoms.register_atom("badkey");

    atoms.register_atom("nocatch");

    atoms.register_atom("exit");
    atoms.register_atom("error");
    atoms.register_atom("throw");

    atoms.register_atom("file");
    atoms.register_atom("line");

    atoms.register_atom("ok");

    atoms.register_atom("erlang");
    atoms.register_atom("apply");

    atoms.register_atom("trap_exit");
    atoms.register_atom("start");
    atoms.register_atom("kill");
    atoms.register_atom("killed");

    atoms.register_atom("not_loaded");
    atoms.register_atom("noproc");
    atoms.register_atom("file_info");

    atoms.register_atom("device");
    atoms.register_atom("directory");
    atoms.register_atom("regular");
    atoms.register_atom("symlink");
    atoms.register_atom("other");

    atoms.register_atom("read");
    atoms.register_atom("write");
    atoms.register_atom("read_write");
    atoms.register_atom("none");

    atoms.register_atom("down");
    atoms.register_atom("process");

    atoms.register_atom("link");
    atoms.register_atom("monitor");

    atoms.register_atom("current_function");
    atoms.register_atom("current_location");
    atoms.register_atom("current_stacktrace");
    atoms.register_atom("initial_call");
    atoms.register_atom("status");
    atoms.register_atom("messages");
    atoms.register_atom("message_queue_len");
    atoms.register_atom("message_queue_data");
    atoms.register_atom("links");
    atoms.register_atom("monitored_by");
    atoms.register_atom("dictionary");
    atoms.register_atom("error_handler");
    atoms.register_atom("heap_size");
    atoms.register_atom("stack_size");
    atoms.register_atom("memory");
    atoms.register_atom("garbage_collection");
    atoms.register_atom("garbage_collection_info");
    atoms.register_atom("group_leader");
    atoms.register_atom("reductions");
    atoms.register_atom("priority");
    atoms.register_atom("trace");
    atoms.register_atom("binary");
    atoms.register_atom("sequential_trace_token");
    atoms.register_atom("catch_level");
    atoms.register_atom("backtrace");
    atoms.register_atom("last_calls");
    atoms.register_atom("total_heap_size");
    atoms.register_atom("suspending");
    atoms.register_atom("min_heap_size");
    atoms.register_atom("min_bin_vheap_size");
    atoms.register_atom("max_heap_size");
    atoms.register_atom("magic_ref");
    atoms.register_atom("fullsweep_after");
    atoms.register_atom("registered_name");

    atoms.register_atom("enoent");

    atoms.register_atom("badfile");

    atoms.register_atom("max");
    atoms.register_atom("high");
    atoms.register_atom("medium");
    atoms.register_atom("low");

    atoms.register_atom("nonode@nohost");

    atoms.register_atom("port");
    atoms.register_atom("time_offset");

    atoms.register_atom("bag");
    atoms.register_atom("duplicate_bag");
    atoms.register_atom("set");
    atoms.register_atom("ordered_set");
    atoms.register_atom("keypos");
    atoms.register_atom("write_concurrency");
    atoms.register_atom("read_concurrency");
    atoms.register_atom("heir");
    atoms.register_atom("public");
    atoms.register_atom("private");
    atoms.register_atom("protected");
    atoms.register_atom("named_table");
    atoms.register_atom("compressed");

    atoms.register_atom("undefined_function");

    atoms.register_atom("os_type");
    atoms.register_atom("win32");
    atoms.register_atom("unix");

    atoms.register_atom("info");
    atoms.register_atom("flush");

    atoms.register_atom("erts_internal");
    atoms.register_atom("flush_monitor_messages");

    atoms.register_atom("$_");
    atoms.register_atom("$$");
    atoms.register_atom("_");

    atoms.register_atom("const");
    atoms.register_atom("and");
    atoms.register_atom("or");
    atoms.register_atom("andalso");
    atoms.register_atom("andthen");
    atoms.register_atom("orelse");
    atoms.register_atom("self");
    atoms.register_atom("is_seq_trace");
    atoms.register_atom("set_seq_token");
    atoms.register_atom("get_seq_token");
    atoms.register_atom("return_trace");
    atoms.register_atom("exception_trace");
    atoms.register_atom("display");
    atoms.register_atom("process_dump");
    atoms.register_atom("enable_trace");
    atoms.register_atom("disable_trace");
    atoms.register_atom("caller");
    atoms.register_atom("silent");
    atoms.register_atom("set_tcw");
    atoms.register_atom("set_tcw_fake");

    atoms.register_atom("is_atom");
    atoms.register_atom("is_float");
    atoms.register_atom("is_integer");
    atoms.register_atom("is_list");
    atoms.register_atom("is_number");
    atoms.register_atom("is_pid");
    atoms.register_atom("is_port");
    atoms.register_atom("is_reference");
    atoms.register_atom("is_tuple");
    atoms.register_atom("is_map");
    atoms.register_atom("is_binary");
    atoms.register_atom("is_function");
    atoms.register_atom("is_record");

    atoms.register_atom("abs");
    atoms.register_atom("element");
    atoms.register_atom("hd");
    atoms.register_atom("length");
    atoms.register_atom("node");
    atoms.register_atom("round");
    atoms.register_atom("size");
    atoms.register_atom("map_size");
    atoms.register_atom("map_get");
    atoms.register_atom("is_map_key");
    atoms.register_atom("bit_size");
    atoms.register_atom("tl");
    atoms.register_atom("trunc");
    atoms.register_atom("float");
    atoms.register_atom("+");
    atoms.register_atom("-");
    atoms.register_atom("*");
    atoms.register_atom("/");
    atoms.register_atom("div");
    atoms.register_atom("rem");
    atoms.register_atom("band");
    atoms.register_atom("bor");
    atoms.register_atom("bxor");
    atoms.register_atom("bnot");
    atoms.register_atom("bsl");
    atoms.register_atom("bsr");
    atoms.register_atom(">");
    atoms.register_atom(">=");
    atoms.register_atom("<");
    atoms.register_atom("=<");
    atoms.register_atom("=:=");
    atoms.register_atom("==");
    atoms.register_atom("=/=");
    atoms.register_atom("/=");
    atoms.register_atom("not");
    atoms.register_atom("xor");

    atoms.register_atom("native");
    atoms.register_atom("second");
    atoms.register_atom("millisecond");
    atoms.register_atom("microsecond");
    atoms.register_atom("perf_counter");

    atoms.register_atom("exiting");
    atoms.register_atom("garbage_collecting");
    atoms.register_atom("waiting");
    atoms.register_atom("running");
    atoms.register_atom("runnable");
    atoms.register_atom("suspended");

    atoms.register_atom("module");
    atoms.register_atom("md5");
    atoms.register_atom("exports");
    atoms.register_atom("functions");
    atoms.register_atom("nifs");
    atoms.register_atom("attributes");
    atoms.register_atom("compile");
    atoms.register_atom("native_addresses");

    atoms.register_atom("hipe_architecture");
    atoms.register_atom("new");
    atoms.register_atom("infinity");

    atoms.register_atom("enotdir");

    atoms.register_atom("spawn");
    atoms.register_atom("tty_sl -c -e");
    atoms.register_atom("fd");
    atoms.register_atom("system_version");

    atoms.register_atom("unicode");
    atoms.register_atom("utf8");
    atoms.register_atom("latin1");

    atoms.register_atom("command");
    atoms.register_atom("data");


    atoms.register_atom("DOWN");
    atoms.register_atom("UP");
    atoms.register_atom("EXIT");

    atoms.register_atom("on_heap");
    atoms.register_atom("off_heap");

    atoms.register_atom("system_logger");

    atoms.register_atom("$end_of_table");
    atoms.register_atom("iterator");

    atoms.register_atom("match");
    atoms.register_atom("nomatch");

    atoms.register_atom("exclusive");
    atoms.register_atom("append");
    atoms.register_atom("sync");
    atoms.register_atom("skip_type_check");

    atoms
};

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

pub fn from_str(val: &str) -> u32 {
    ATOMS.from_str(val)
}

pub fn to_str(index: u32) -> Result<String, String> {
    if let Some(p) = ATOMS.to_str(index) {
        return Ok(unsafe { (*p).name.clone() });
    }
    Err(format!("Atom does not exist: {}", index))
}
