use crate::bif;
use crate::module::MFA;
use crate::instr_ptr::InstrPtr;
use crate::servo_arc::Arc;
use hashbrown::HashMap;
use parking_lot::RwLock;
use std::fmt;

/// Reference counted ExportsTable.
pub type RcExportsTable = Arc<RwLock<ExportsTable>>; // TODO: I don't like this lock at all

pub enum Export {
    Fun(InstrPtr),
    Bif(bif::BifFn),
}

impl fmt::Debug for Export {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Export::Fun(..) => write!(f, "Export(fn)"),
            Export::Bif(..) => write!(f, "Export(bif)"),
        }
    }
}

#[derive(Debug)]
pub struct ExportsTable {
    exports: HashMap<MFA, Export>, // hashbrown is send & sync, so no locks?
}

impl ExportsTable {
    pub fn with_rc() -> RcExportsTable {
        let mut exports = HashMap::new();

        // load all the bif exports
        for (key, val) in bif::BIFS.iter() {
            exports.insert(*key, Export::Bif(*val));
        }

        Arc::new(RwLock::new(ExportsTable { exports }))
    }

    pub fn register(&mut self, mfa: MFA, ptr: InstrPtr) {
        self.exports.insert(mfa, Export::Fun(ptr));
    }

    pub fn lookup(&self, mfa: &MFA) -> Option<&Export> {
        self.exports.get(mfa)
    }

    // get or get stub
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_rc() {
        let table = ExportsTable::with_rc();
    }

    #[test]
    fn test_lookup() {
        // let mut table = ExportsTable::new();

        // assert!(table.lookup(0).is_none());

        // let pid = table.reserve().unwrap();

        // assert!(table.lookup(pid).is_none());

        // table.map(pid, 10);

        // assert!(table.lookup(pid).is_some());
        // assert_eq!(table.lookup(pid).unwrap(), 10);
    }
}
