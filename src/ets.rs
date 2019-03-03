use crate::value::Term;
use crate::servo_arc::Arc;
use crate::process::RcProcess;

pub mod bif;
pub mod error;
// use std::error::Error;
pub use error::Result;

/// Represents an interface to a single table.
pub trait Table {
    // first, next, last, prev could be iter? --> iter can't go backwards so we'll add a Cursor API
    // almost no code uses prev() outside of a few OTP tests so we could just start with Iter.
    // also, the db impl seems to equate the two
    fn first(&self, process: &RcProcess) -> Result<Term>;

    fn next(&self, process: &RcProcess, key: Term) -> Result<Term>;

    fn last(&self, process: &RcProcess) -> Result<Term>;

    fn prev(&self, process: &RcProcess, key: Term) -> Result<Term>;

    // put
    fn insert(&mut self, /* [in out] */ value: Term, key_clash_fail: bool); /* DB_ERROR_BADKEY if key exists */ 

    fn get(&self, process: &RcProcess,  key: Term) -> Result<Term>;

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term>;

    // contains_key ? why is result a Term, not bool
    fn member(&self, key: Term) -> Result<Term>;

    // erase  (remove_entry in rust)
    fn remove(&mut self, key: Term) -> Result<Term>;

    fn remove_object(&mut self, object: Term) -> Result<Term>;

    fn slot(&self, slot: Term) -> Result<Term>;

    // int (*db_select_chunk)(process: &RcProcess, 
			   // table: &Self, /* [in out] */
    //                        Eterm tid,
			   // Eterm pattern,
			   // Sint chunk_size,
			   // int reverse,
	 		   // Eterm* ret);
    
    // _continue is for when the main function traps, let's just use generators
    fn select(&self, process: &RcProcess, tid: Term, pattern: Term, reverse: bool) -> Result<Term>;

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
    // void (*db_print)(fmtfn_t to,
		     // void* to_arg, 
		     // int show, 
		     // table: Self /* [in out] */ );

    // void (*db_foreach_offheap)(DbTable* db,  /* [in out] */ 
			       // void (*func)(ErlOffHeap *, void *),
			       // void *arg);

    // TODO: replace both with get_mut or alter
    // Lookup a dbterm for updating. Return false if not found.
    // fn lookup_dbterm(&self, process: &RcProcess, key: Term, obj: Term) -> DbUpdateHandle;
    // Must be called for each db_lookup_dbterm that returned true, even if dbterm was not
    // updated. If the handle was of a new object and cret is not DB_ERROR_NONE, the object is
    // removed from the table.
    // fn finalize_dbterm(&self, cret: usize, handle: DbUpdateHandle);
}

// TODO: we want to avoid mutex for concurrent writes tho, maybe dyn Table + Sync
pub type RcTableRegistry = Arc<TableRegistry>;

pub struct TableRegistry {
    tables: Vec<Arc<dyn Table + Sync + Send>>
}

impl TableRegistry {
    pub fn with_rc() -> RcTableRegistry {
        Arc::new(Self {
            tables: Vec::new()
        })
    }

    pub fn new_table(process: &RcProcess) -> Result<Box<dyn Table>> {
        unimplemented!()
    }
}


