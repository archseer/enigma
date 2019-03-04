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

pub fn from_str(val: &str) -> u32 {
    ATOMS.from_str(val)
}

pub fn to_str(index: u32) -> Result<String, String> {
    if let Some(p) = ATOMS.to_str(index) {
        return Ok(unsafe { (*p).name.clone() });
    }
    Err(format!("Atom does not exist: {}", index))
}
