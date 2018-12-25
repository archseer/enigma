//use std::ptr;
use crate::value::Value;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::BTreeMap; // TODO: maybe HashMap
use std::u16;

#[derive(Debug)]
pub struct Atom {
    /// Length of utf8-encoded atom name.
    pub len: u16,
    /// First 4 bytes used for comparisons
    pub ord0: u32,
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
    index: RwLock<BTreeMap<String, usize>>,

    /// Reverse mapping atom index to string (sorted by index)
    index_r: RwLock<Vec<Atom>>,
}

/// Stores atom lookup tables.
impl AtomTable {
    pub fn new() -> AtomTable {
        AtomTable {
            index: RwLock::new(BTreeMap::new()),
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
    pub fn from_str(&self, val: &str) -> Value {
        {
            let atoms = self.index.read();

            if atoms.contains_key(val) {
                return Value::Atom(atoms[val]);
            }
        } // drop read lock

        let index = self.register_atom(val);
        Value::Atom(index)
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
    atoms
};

pub const TRUE: usize = 1;
pub const FALSE: usize = 2;

pub fn from_str(val: &str) -> Value {
    ATOMS.from_str(val)
}

pub fn i_from_str(val: &str) -> usize {
    if let Value::Atom(i) = ATOMS.from_str(val) {
        return i;
    }
    panic!("unreachable")
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
