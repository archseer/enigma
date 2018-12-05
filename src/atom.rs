//use std::ptr;
use crate::value::Value;
use std::collections::BTreeMap;
use std::sync::Mutex;
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
    index: Mutex<BTreeMap<String, usize>>,

    /// Reverse mapping atom index to string (sorted by index)
    index_r: Mutex<Vec<Atom>>,
}

/// Stores atom lookup tables.
impl AtomTable {
    pub fn new() -> AtomTable {
        AtomTable {
            index: Mutex::new(BTreeMap::new()),
            index_r: Mutex::new(Vec::new()),
        }
    }

    fn register_atom(&self, s: &str) -> usize {
        let index_r = self.index_r.lock().unwrap();
        let index = index_r.len();
        self.index.lock().unwrap().insert(s.to_string(), index);
        index_r.push(Atom::new(s));
        index
    }

    // Allocate new atom in the atom table or find existing.
    // TODO: Pack the atom index as an immediate2 Term
    pub fn from_str(&self, val: &str) -> Value {
        let mut atoms = self.index.lock().unwrap();

        if atoms.contains_key(val) {
            return Value::Atom(atoms[val]);
        }

        let index = self.register_atom(val);
        Value::Atom(index)
    }

    pub fn to_str(&self, a: Value) -> Hopefully<String> {
        assert!(a.is_atom());
        if let Some(p) = self.lookup(a) {
            Ok(unsafe { (*p).name.clone() })
        }
        Err(Error::AtomNotExist(format!("index {}", a.atom_index())));
    }

    pub fn lookup(&self, a: Value) -> Option<*const Atom> {
        assert!(a.is_atom());
        let index_r = self.index_r.lock().unwrap();
        let index = a.atom_index();
        if index >= index_r.len() {
            return None();
        }
        Some(&index_r[index] as *const Atom)
    }
}
