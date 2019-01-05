//use std::ptr;
use crate::value::Value;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::u16;

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
    index: RwLock<HashMap<String, usize>>,

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

    pub fn reserve(&self, _len: usize) {}

    pub fn register_atom(&self, s: &str) -> usize {
        let mut index_r = self.index_r.write();
        let index = index_r.len();
        self.index.write().insert(s.to_string(), index);
        index_r.push(Atom::new(s));
        index
    }

    /// Allocate new atom in the atom table or find existing.
    pub fn from_str(&self, val: &str) -> usize {
        {
            let atoms = self.index.read();

            if atoms.contains_key(val) {
                return atoms[val];
            }
        } // drop read lock

        self.register_atom(val)
    }

    pub fn to_str(&self, a: &Value) -> Result<String, String> {
        if let Value::Atom(index) = a {
            if let Some(p) = self.lookup(a) {
                return Ok(unsafe { (*p).name.clone() });
            }
            return Err(format!("Atom does not exist: {}", index));
        }
        panic!("Value is not an atom!")
    }

    pub fn lookup(&self, a: &Value) -> Option<*const Atom> {
        if let Value::Atom(index) = a {
            let index_r = self.index_r.read();
            if *index >= index_r.len() {
                return None;
            }
            return Some(&index_r[*index] as *const Atom);
        }
        panic!("Value is not an atom!")
    }
    pub fn lookup_index(&self, index: usize) -> Option<*const Atom> {
        let index_r = self.index_r.read();
        if index >= index_r.len() {
            return None;
        }
        Some(&index_r[index] as *const Atom)
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
    atoms.register_atom("bad_map");
    atoms.register_atom("bad_key");

    atoms.register_atom("nocatch");

    atoms.register_atom("exit");
    atoms.register_atom("error");
    atoms.register_atom("throw");

    atoms.register_atom("file");
    atoms.register_atom("line");
    atoms
};

pub const TRUE: usize = 1;
pub const FALSE: usize = 2;
pub const UNDEFINED: usize = 3;
pub const VALUE: usize = 4;
pub const ALL: usize = 5;
pub const NORMAL: usize = 6;
pub const INTERNAL_ERROR: usize = 7;
pub const BADARG: usize = 8;
pub const BADARITH: usize = 9;
pub const BADMATCH: usize = 10;
pub const FUNCTION_CLAUSE: usize = 11;
pub const CASE_CLAUSE: usize = 12;
pub const IF_CLAUSE: usize = 13;
pub const UNDEF: usize = 14;
pub const BADFUN: usize = 15;
pub const BADARITY: usize = 16;
pub const TIMEOUT_VALUE: usize = 17;
pub const NO_PROC: usize = 18;
pub const NOT_ALIVE: usize = 19;
pub const SYSTEM_LIMIT: usize = 20;
pub const TRY_CLAUSE: usize = 21;
pub const NOT_SUP: usize = 22;
pub const BAD_MAP: usize = 23;
pub const BAD_KEY: usize = 24;

pub const NOCATCH: usize = 25;

pub const EXIT: usize = 26;
pub const ERROR: usize = 27;
pub const THROW: usize = 28;

pub const FILE: usize = 29;
pub const LINE: usize = 30;

pub fn from_str(val: &str) -> usize {
    ATOMS.from_str(val)
}

pub fn to_str(a: &Value) -> Result<String, String> {
    ATOMS.to_str(a)
}

pub fn from_index(index: usize) -> Result<String, String> {
    if let Some(p) = ATOMS.lookup_index(index) {
        return Ok(unsafe { (*p).name.clone() });
    }
    Err(format!("Atom does not exist: {}", index))
}
