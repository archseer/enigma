use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

#[inline]
pub fn cmp(a1: Atom, a2: Atom) -> std::cmp::Ordering {
    ATOMS.read().cmp(a1.0, a2.0)
}

/// Maximum character length of an atom.
pub const MAX_ATOM_CHARS: usize = 255;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Atom(pub u32);

impl Atom {
    /// Construct a new atom from a raw string.
    #[inline]
    pub fn new(val: &str) -> Self {
        assert!(val.len() <= MAX_ATOM_CHARS);
        ATOMS.write().from_str(val)
    }

    #[inline]
    pub fn to_str(&self) -> Option<&'static str> {
        ATOMS.read().to_str(self.0)
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Atom({})", self.0)
    }
}

impl From<&str> for Atom {
    #[inline]
    fn from(value: &str) -> Self {
        Atom::new(value)
    }
}

impl From<String> for Atom {
    #[inline]
    fn from(value: String) -> Self {
        Atom::new(value.as_str())
    }
}

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
    pub fn from_str(&mut self, val: &str) -> Atom {
        Atom(self.lookup(val).unwrap_or_else(|| self.insert(val)))
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

pub const TRUE: Atom = Atom(1);
pub const FALSE: Atom = Atom(2);
pub const UNDEFINED: Atom = Atom(3);
pub const VALUE: Atom = Atom(4);
pub const ALL: Atom = Atom(5);

pub const NORMAL: Atom = Atom(6);
pub const INTERNAL_ERROR: Atom = Atom(7);
pub const BADARG: Atom = Atom(8);
pub const BADARITH: Atom = Atom(9);
pub const BADMATCH: Atom = Atom(10);
pub const FUNCTION_CLAUSE: Atom = Atom(11);
pub const CASE_CLAUSE: Atom = Atom(12);
pub const IF_CLAUSE: Atom = Atom(13);
pub const UNDEF: Atom = Atom(14);
pub const BADFUN: Atom = Atom(15);
pub const BADARITY: Atom = Atom(16);
pub const TIMEOUT_VALUE: Atom = Atom(17);
pub const NO_PROC: Atom = Atom(18);
pub const NOT_ALIVE: Atom = Atom(19);
pub const SYSTEM_LIMIT: Atom = Atom(20);
pub const TRY_CLAUSE: Atom = Atom(21);
pub const NOT_SUP: Atom = Atom(22);
pub const BADMAP: Atom = Atom(23);
pub const BADKEY: Atom = Atom(24);

pub const NOCATCH: Atom = Atom(25);

pub const EXIT: Atom = Atom(26);
pub const ERROR: Atom = Atom(27);
pub const THROW: Atom = Atom(28);

pub const FILE: Atom = Atom(29);
pub const LINE: Atom = Atom(30);

pub const OK: Atom = Atom(31);

pub const ERLANG: Atom = Atom(32);
pub const APPLY: Atom = Atom(33);

pub const TRAP_EXIT: Atom = Atom(34);
pub const START: Atom = Atom(35);
pub const KILL: Atom = Atom(36);
pub const KILLED: Atom = Atom(37);

pub const NOT_LOADED: Atom = Atom(38);
pub const NOPROC: Atom = Atom(39);
pub const FILE_INFO: Atom = Atom(40);

pub const DEVICE: Atom = Atom(41);
pub const DIRECTORY: Atom = Atom(42);
pub const REGULAR: Atom = Atom(43);
pub const SYMLINK: Atom = Atom(44);
pub const OTHER: Atom = Atom(45);

pub const READ: Atom = Atom(46);
pub const WRITE: Atom = Atom(47);
pub const READ_WRITE: Atom = Atom(48);
pub const NONE: Atom = Atom(49);

pub const DOWN: Atom = Atom(50);
pub const PROCESS: Atom = Atom(51);

pub const LINK: Atom = Atom(52);
pub const MONITOR: Atom = Atom(53);

pub const CURRENT_FUNCTION: Atom = Atom(54);
pub const CURRENT_LOCATION: Atom = Atom(55);
pub const CURRENT_STACKTRACE: Atom = Atom(56);
pub const INITIAL_CALL: Atom = Atom(57);
pub const STATUS: Atom = Atom(58);
pub const MESSAGES: Atom = Atom(59);
pub const MESSAGE_QUEUE_LEN: Atom = Atom(60);
pub const MESSAGE_QUEUE_DATA: Atom = Atom(61);
pub const LINKS: Atom = Atom(62);
pub const MONITORED_BY: Atom = Atom(63);
pub const DICTIONARY: Atom = Atom(64);
pub const ERROR_HANDLER: Atom = Atom(65);
pub const HEAP_SIZE: Atom = Atom(66);
pub const STACK_SIZE: Atom = Atom(67);
pub const MEMORY: Atom = Atom(68);
pub const GARBAGE_COLLECTION: Atom = Atom(69);
pub const GARBAGE_COLLECTION_INFO: Atom = Atom(70);
pub const GROUP_LEADER: Atom = Atom(71);
pub const REDUCTIONS: Atom = Atom(72);
pub const PRIORITY: Atom = Atom(73);
pub const TRACE: Atom = Atom(74);
pub const BINARY: Atom = Atom(75);
pub const SEQUENTIAL_TRACE_TOKEN: Atom = Atom(76);
pub const CATCH_LEVEL: Atom = Atom(77);
pub const BACKTRACE: Atom = Atom(78);
pub const LAST_CALLS: Atom = Atom(79);
pub const TOTAL_HEAP_SIZE: Atom = Atom(80);
pub const SUSPENDING: Atom = Atom(81);
pub const MIN_HEAP_SIZE: Atom = Atom(82);
pub const MIN_BIN_VHEAP_SIZE: Atom = Atom(83);
pub const MAX_HEAP_SIZE: Atom = Atom(84);
pub const MAGIC_REF: Atom = Atom(85);
pub const FULLSWEEP_AFTER: Atom = Atom(86);
pub const REGISTERED_NAME: Atom = Atom(87);

pub const ENOENT: Atom = Atom(88);

pub const BADFILE: Atom = Atom(89);

pub const MAX: Atom = Atom(90);
pub const HIGH: Atom = Atom(91);
pub const MEDIUM: Atom = Atom(92);
pub const LOW: Atom = Atom(93);

pub const NO_NODE_NO_HOST: Atom = Atom(94);

pub const PORT: Atom = Atom(95);
pub const TIME_OFFSET: Atom = Atom(96);

pub const BAG: Atom = Atom(97);
pub const DUPLICATE_BAG: Atom = Atom(98);
pub const SET: Atom = Atom(99);
pub const ORDERED_SET: Atom = Atom(100);
pub const KEYPOS: Atom = Atom(101);
pub const WRITE_CONCURRENCY: Atom = Atom(102);
pub const READ_CONCURRENCY: Atom = Atom(103);
pub const HEIR: Atom = Atom(104);
pub const PUBLIC: Atom = Atom(105);
pub const PRIVATE: Atom = Atom(106);
pub const PROTECTED: Atom = Atom(107);
pub const NAMED_TABLE: Atom = Atom(108);
pub const COMPRESSED: Atom = Atom(109);

pub const UNDEFINED_FUNCTION: Atom = Atom(110);

pub const OS_TYPE: Atom = Atom(111);
pub const WIN32: Atom = Atom(112);
pub const UNIX: Atom = Atom(113);

pub const INFO: Atom = Atom(114);
pub const FLUSH: Atom = Atom(115);
pub const ERTS_INTERNAL: Atom = Atom(116);
pub const FLUSH_MONITOR_MESSAGES: Atom = Atom(117);

pub const DOLLAR_UNDERSCORE: Atom = Atom(118);
pub const DOLLAR_DOLLAR: Atom = Atom(119);
pub const UNDERSCORE: Atom = Atom(120);

pub const CONST: Atom = Atom(120);
pub const AND: Atom = Atom(121);
pub const OR: Atom = Atom(122);
pub const ANDALSO: Atom = Atom(123);
pub const ANDTHEN: Atom = Atom(124);
pub const ORELSE: Atom = Atom(125);
pub const SELF: Atom = Atom(126);
pub const MESSAGE: Atom = Atom(127);
pub const IS_SEQ_TRACE: Atom = Atom(128);
pub const SET_SEQ_TOKEN: Atom = Atom(129);
pub const GET_SEQ_TOKEN: Atom = Atom(130);
pub const RETURN_TRACE: Atom = Atom(131);
pub const EXCEPTION_TRACE: Atom = Atom(132);
pub const DISPLAY: Atom = Atom(133);
pub const PROCESS_DUMP: Atom = Atom(134);
pub const ENABLE_TRACE: Atom = Atom(135);
pub const DISABLE_TRACE: Atom = Atom(136);
pub const CALLER: Atom = Atom(137);
pub const SILENT: Atom = Atom(138);
pub const SET_TCW: Atom = Atom(139);
pub const SET_TCW_FAKE: Atom = Atom(140);

pub const IS_ATOM: Atom = Atom(141);
pub const IS_FLOAT: Atom = Atom(142);
pub const IS_INTEGER: Atom = Atom(143);
pub const IS_LIST: Atom = Atom(144);
pub const IS_NUMBER: Atom = Atom(145);
pub const IS_PID: Atom = Atom(146);
pub const IS_PORT: Atom = Atom(147);
pub const IS_REFERENCE: Atom = Atom(148);
pub const IS_TUPLE: Atom = Atom(149);
pub const IS_MAP: Atom = Atom(150);
pub const IS_BINARY: Atom = Atom(151);
pub const IS_FUNCTION: Atom = Atom(152);
pub const IS_RECORD: Atom = Atom(153);

pub const ABS: Atom = Atom(154);
pub const ELEMENT: Atom = Atom(155);
pub const HD: Atom = Atom(156);
pub const LENGTH: Atom = Atom(157);
pub const NODE: Atom = Atom(158);
pub const ROUND: Atom = Atom(159);
pub const SIZE: Atom = Atom(160);
pub const MAP_SIZE: Atom = Atom(161);
pub const MAP_GET: Atom = Atom(162);
pub const IS_MAP_KEY: Atom = Atom(163);
pub const BIT_SIZE: Atom = Atom(164);
pub const TL: Atom = Atom(165);
pub const TRUNC: Atom = Atom(166);
pub const FLOAT: Atom = Atom(167);
pub const PLUS: Atom = Atom(168);
pub const MINUS: Atom = Atom(169);
pub const TIMES: Atom = Atom(170);
pub const DIV: Atom = Atom(171);
pub const INTDIV: Atom = Atom(172);
pub const REM: Atom = Atom(173);
pub const BAND: Atom = Atom(174);
pub const BOR: Atom = Atom(175);
pub const BXOR: Atom = Atom(176);
pub const BNOT: Atom = Atom(177);
pub const BSL: Atom = Atom(178);
pub const BSR: Atom = Atom(179);
pub const GT: Atom = Atom(180);
pub const GE: Atom = Atom(181);
pub const LT: Atom = Atom(182);
pub const LE: Atom = Atom(183);
pub const EQ: Atom = Atom(184);
pub const EQEQ: Atom = Atom(185);
pub const NEQ: Atom = Atom(186);
pub const NEQEQ: Atom = Atom(187);
pub const NOT: Atom = Atom(188);
pub const XOR: Atom = Atom(189);

pub const NATIVE: Atom = Atom(190);
pub const SECOND: Atom = Atom(191);
pub const MILLISECOND: Atom = Atom(192);
pub const MICROSECOND: Atom = Atom(193);
pub const PERF_COUNTER: Atom = Atom(194);

pub const EXITING: Atom = Atom(195);
pub const GARBAGE_COLLECTING: Atom = Atom(196);
pub const WAITING: Atom = Atom(197);
pub const RUNNING: Atom = Atom(198);
pub const RUNNABLE: Atom = Atom(199);
pub const SUSPENDED: Atom = Atom(200);

pub const MODULE: Atom = Atom(201);
pub const MD5: Atom = Atom(202);
pub const EXPORTS: Atom = Atom(203);
pub const FUNCTIONS: Atom = Atom(204);
pub const NIFS: Atom = Atom(205);
pub const ATTRIBUTES: Atom = Atom(206);
pub const COMPILE: Atom = Atom(207);
pub const NATIVE_ADDRESSES: Atom = Atom(208);

pub const HIPE_ARCHITECTURE: Atom = Atom(209);
pub const NEW: Atom = Atom(210);
pub const INFINITY: Atom = Atom(211);

pub const ENOTDIR: Atom = Atom(212);
pub const SPAWN: Atom = Atom(213);
pub const TTY_SL: Atom = Atom(214);
pub const FD: Atom = Atom(215);
pub const SYSTEM_VERSION: Atom = Atom(216);

pub const UNICODE: Atom = Atom(217);
pub const UTF8: Atom = Atom(218);
pub const LATIN1: Atom = Atom(219);

pub const COMMAND: Atom = Atom(220);
pub const DATA: Atom = Atom(221);

pub const DOWN_U: Atom = Atom(222);
pub const UP_U: Atom = Atom(223);
pub const EXIT_U: Atom = Atom(224);

pub const ON_HEAP: Atom = Atom(225);
pub const OFF_HEAP: Atom = Atom(226);

pub const SYSTEM_LOGGER: Atom = Atom(227);

pub const DOLLAR_END_OF_TABLE: Atom = Atom(228);
pub const ITERATOR: Atom = Atom(229);

pub const MATCH: Atom = Atom(230);
pub const NOMATCH: Atom = Atom(231);

pub const EXCLUSIVE: Atom = Atom(232);
pub const APPEND: Atom = Atom(233);
pub const SYNC: Atom = Atom(234);
pub const SKIP_TYPE_CHECK: Atom = Atom(235);

pub const PURIFY: Atom = Atom(236);

pub const ACQUIRED: Atom = Atom(237);
pub const BUSY: Atom = Atom(238);
pub const LOCK_ORDER_VIOLATION: Atom = Atom(239);

pub const EOF: Atom = Atom(240);

pub const BEAM_LIB: Atom = Atom(241);
pub const VERSION: Atom = Atom(242);

pub const TYPE: Atom = Atom(243);
pub const PID: Atom = Atom(244);
pub const NEW_INDEX: Atom = Atom(245);
pub const NEW_UNIQ: Atom = Atom(246);
pub const INDEX: Atom = Atom(247);
pub const UNIQ: Atom = Atom(248);
pub const ENV: Atom = Atom(249);
pub const REFC: Atom = Atom(250);
pub const ARITY: Atom = Atom(251);
pub const NAME: Atom = Atom(252);
pub const LOCAL: Atom = Atom(253);
pub const EXTERNAL: Atom = Atom(254);
pub const MACHINE: Atom = Atom(255);
pub const OTP_RELEASE: Atom = Atom(256);

pub const BOF: Atom = Atom(257);
pub const CUR: Atom = Atom(258);

pub const NO_INTEGER: Atom = Atom(259);

pub const ENDIAN: Atom = Atom(260);
pub const LITTLE: Atom = Atom(261);
pub const BIG: Atom = Atom(262);

pub const PROTECTION: Atom = Atom(263);

pub const TRIM: Atom = Atom(264);
pub const TRIM_ALL: Atom = Atom(265);
pub const GLOBAL: Atom = Atom(266);
pub const SCOPE: Atom = Atom(267);
