use super::*;
use crate::immix::Heap;
use crate::value::{Cons, Term, TryFrom, TryInto, TryIntoMut, Tuple, Variant};
use error::*;
use parking_lot::RwLock;

pub(crate) struct OrderedSet {
    meta: Metadata,
    hashmap: RwLock<BTreeMap<Term, Term>>,
    heap: Heap,
}

unsafe impl Sync for OrderedSet {}
unsafe impl Send for OrderedSet {}

impl OrderedSet {
    pub fn new(meta: Metadata, _process: &RcProcess) -> Self {
        Self {
            meta,
            hashmap: RwLock::new(BTreeMap::new()),
            heap: Heap::new(),
        }
    }
}

fn get_key(pos: usize, value: Term) -> Term {
    let tuple = Tuple::try_from(&value).unwrap();
    tuple[pos]
}

impl Table for OrderedSet {
    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn first(&self, process: &RcProcess) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().iter().next() {
            Some((key, _value)) => Ok(key.deep_clone(heap)),
            None => Ok(atom!(DOLLAR_END_OF_TABLE)),
        }
    }

    fn next(&self, _process: &RcProcess, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn last(&self, process: &RcProcess) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().iter().rev().next() {
            Some((key, _value)) => Ok(key.deep_clone(heap)),
            None => Ok(atom!(DOLLAR_END_OF_TABLE)),
        }
    }

    fn prev(&self, _process: &RcProcess, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    // put
    fn insert(&self, _process: &RcProcess, value: Term, _key_clash_fail: bool) -> Result<()> {
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap.write().insert(key, value);
        Ok(())
    }

    fn get(&self, process: &RcProcess, key: Term) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().get(&key) {
            Some(val) => Ok(val.deep_clone(heap)),
            None => Ok(Term::nil()),
        }
    }

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().get(&key) {
            Some(value) => {
                let tup = Tuple::try_from(&*value).unwrap();
                assert!(tup.len() > index);
                Ok(tup[index].deep_clone(heap))
            }
            None => Ok(Term::nil()),
        }
    }

    // contains_key ? why is result a Term, not bool
    fn member(&self, key: Term) -> bool {
        self.hashmap.read().contains_key(&key)
    }

    fn update_element(&self, _process: &RcProcess, _key: Term, _list: Term) -> Result<Term> {
        unimplemented!();
    }

    // erase  (remove_entry in rust)
    fn remove(&self, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn remove_object(&mut self, _object: Term) -> Result<Term> {
        unimplemented!()
    }

    fn slot(&self, _slot: Term) -> Result<Term> {
        unimplemented!()
    }

    // int (*db_select_chunk)(process: &RcProcess,
    // table: &Self, /* [in out] */
    //                        Eterm tid,
    // Eterm pattern,
    // Sint chunk_size,
    // int reverse,
    // Eterm* ret);

    // _continue is for when the main function traps, let's just use generators
    fn select(
        &self,
        _vm: &vm::Machine,
        _process: &RcProcess,
        _pattern: &pam::Pattern,
        _flags: pam::r#match::Flag,
        _reverse: bool,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_delete(
        &self,
        _vm: &vm::Machine,
        _process: &RcProcess,
        _pattern: &pam::Pattern,
        _flags: pam::r#match::Flag,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_delete_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_count(&self, _process: &RcProcess, _tid: Term, _pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    // fn select_count_continue(&self, process: &RcProcess, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_replace(
        &mut self,
        _process: &RcProcess,
        _tid: Term,
        _pattern: Term,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_replace_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn take(&mut self, _process: &RcProcess, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    /// takes reds, then returns new reds (equal to delete_all)
    fn clear(&mut self, _process: &RcProcess, _reds: usize) -> Result<usize> {
        unimplemented!()
    }
}
