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

pub fn from_str(val: &str) -> u32 {
    ATOMS.from_str(val)
}

pub fn to_str(index: u32) -> Result<String, String> {
    if let Some(p) = ATOMS.to_str(index) {
        return Ok(unsafe { (*p).name.clone() });
    }
    Err(format!("Atom does not exist: {}", index))
}
