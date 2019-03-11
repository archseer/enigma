use crate::value::Term;
use crate::vm;
//use crate::servo_arc::Arc;
use crate::process::{self, RcProcess};
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::sync::Arc; // servo_arc doesn't work with trait objects

#[macro_export]
macro_rules! table_kind {
    ($x:expr) => {
        Status::from_bits_truncate($x.bits & Status::KIND_MASK.bits)
    };
}

pub mod bif;
pub mod hash_table;
pub mod pam;

pub mod error;
// use std::error::Error;
pub use error::Result;

/// Represents an interface to a single table.
pub trait Table: Send + Sync {
    fn meta(&self) -> &Metadata;

    // first, next, last, prev could be iter? --> iter can't go backwards so we'll add a Cursor API
    // almost no code uses prev() outside of a few OTP tests so we could just start with Iter.
    // also, the db impl seems to equate the two
    fn first(&self, process: &RcProcess) -> Result<Term>;

    fn next(&self, process: &RcProcess, key: Term) -> Result<Term>;

    fn last(&self, process: &RcProcess) -> Result<Term>;

    fn prev(&self, process: &RcProcess, key: Term) -> Result<Term>;

    // put
    fn insert(&self, process: &RcProcess, value: Term, key_clash_fail: bool) -> Result<()>; /* DB_ERROR_BADKEY if key exists */

    fn get(&self, process: &RcProcess, key: Term) -> Result<Term>;

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term>;

    fn member(&self, key: Term) -> bool;

    // erase  (remove_entry in rust)
    fn remove(&mut self, key: Term) -> Result<Term>;

    fn remove_object(&mut self, object: Term) -> Result<Term>;

    fn slot(&self, slot: Term) -> Result<Term>;

    // int (*db_select_chunk)(process: &RcProcess, table: &Self, Eterm tid, Eterm pattern, Sint chunk_size, int reverse, Eterm* ret);

    // _continue is for when the main function traps, let's just use generators
    fn select(
        &self,
        vm: &vm::Machine,
        process: &RcProcess,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
        reverse: bool,
    ) -> Result<Term>;

    fn select_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term>;

    fn select_delete(&mut self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term>;

    fn select_delete_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term>;

    fn select_count(&self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term>;

    fn select_count_continue(&self, process: &RcProcess, continuation: Term) -> Result<Term>;

    fn select_replace(&mut self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term>;

    fn select_replace_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term>;

    fn take(&mut self, process: &RcProcess, key: Term) -> Result<Term>;

    /// takes reds, then returns new reds (equal to delete_all)
    fn clear(&mut self, process: &RcProcess, reds: usize) -> Result<usize>;

    // don't think we'll need these
    // int (*db_free_empty_table)(DbTable* db);
    // SWord (*db_free_table_continue)(DbTable* db, SWord reds);

    // this is just Derive(Debug) + Display
    // void (*db_print)(fmtfn_t to, void* to_arg, int show, table: Self);

    // void (*db_foreach_offheap)(DbTable* db, void (*func)(ErlOffHeap *, void *), void *arg);

    // TODO: replace both with get_mut or alter
    // Lookup a dbterm for updating. Return false if not found.
    // fn lookup_dbterm(&self, process: &RcProcess, key: Term, obj: Term) -> DbUpdateHandle;
    // Must be called for each db_lookup_dbterm that returned true, even if dbterm was not
    // updated. If the handle was of a new object and cret is not DB_ERROR_NONE, the object is
    // removed from the table.
    // fn finalize_dbterm(&self, cret: usize, handle: DbUpdateHandle);
}

bitflags! {
    pub struct Status: u32 {
        const INITIAL = 0;

        const DB_PRIVATE = (1 << 0);
        const DB_PROTECTED = (1 << 1);
        const DB_PUBLIC = (1 << 2);
        /// table is being deleted
        const DB_DELETE = (1 << 3);

        const KIND_OFFSET = 4;
        const KIND_BITS = 5;
        const KIND_MASK  = (((1<<(Self::KIND_BITS.bits + Self::KIND_OFFSET.bits))-1) & !((1<<(Self::KIND_OFFSET.bits))-1));

        const DB_SET = (1 << 4);
        const DB_BAG = (1 << 5);
        const DB_DUPLICATE_BAG = (1 << 6);
        const DB_ORDERED_SET = (1 << 7);
        const DB_CA_ORDERED_SET = (1 << 8);

        /// write_concurrency
        const DB_FINE_LOCKED = (1 << 9);
        /// read_concurrency
        const DB_FREQ_READ = (1 << 10);
        const DB_NAMED_TABLE = (1 << 11);
        const DB_BUSY = (1 << 12);
    }
}

// Shared metadata for all table types.
pub struct Metadata {
    ///
    tid: process::Ref,
    /// atom
    name: Option<usize>,
    status: Status,
    kind: Status,
    keypos: usize,
    owner: process::PID,
    compress: bool,
}

// TODO: we want to avoid mutex for concurrent writes tho, maybe dyn Table + Sync
pub type RcTableRegistry = Arc<Mutex<TableRegistry>>;

pub type RcTable = Arc<dyn Table>;

pub struct TableRegistry {
    tables: HashMap<process::Ref, RcTable>,
    named_tables: HashMap<usize, RcTable>,
}

impl TableRegistry {
    pub fn with_rc() -> RcTableRegistry {
        Arc::new(Mutex::new(Self {
            tables: HashMap::new(),
            named_tables: HashMap::new(),
        }))
    }

    pub fn get(&self, reference: process::Ref) -> Option<RcTable> {
        self.tables.get(&reference).cloned()
    }

    pub fn get_named(&self, name: usize) -> Option<RcTable> {
        self.named_tables.get(&name).cloned()
    }

    pub fn insert(&mut self, reference: process::Ref, table: RcTable) {
        self.tables.insert(reference, table);
    }

    pub fn insert_named(&mut self, name: usize, table: RcTable) -> bool {
        if !self.named_tables.contains_key(&name) {
            self.named_tables.insert(name, table);
            return true;
        }
        false
    }

    pub fn remove(&mut self, table: &RcTable) -> bool {
        let meta = table.meta();

        // remove table from index
        self.tables.remove(&meta.tid);

        // if named, remove from named index
        if let Some(name) = meta.name {
            self.named_tables.remove(&name);
        }

        true
    }

    pub fn whereis(&self, name: usize) -> Option<process::Ref> {
        self.named_tables.get(&name).map(|table| table.meta().tid)
    }
}
