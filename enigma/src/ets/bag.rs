use super::*;
use crate::immix::Heap;
use crate::value::{CastFrom, Cons, Term, Tuple};
use error::*;
use parking_lot::RwLock;
use std::collections::HashSet;

pub(crate) struct Bag {
    meta: Metadata,
    hashmap: RwLock<HashMap<Term, HashSet<Term>>>,
    heap: Heap,
}

unsafe impl Sync for Bag {}
unsafe impl Send for Bag {}

impl Bag {
    pub fn new(meta: Metadata, _process: &RcProcess) -> Self {
        Self {
            meta,
            hashmap: RwLock::new(HashMap::new()),
            heap: Heap::new(),
        }
    }
}

fn get_key(pos: usize, value: Term) -> Term {
    let tuple = Tuple::cast_from(&value).unwrap();
    tuple[pos]
}

impl Table for Bag {
    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn first(&self, _process: &RcProcess) -> Result<Term> {
        unimplemented!()
    }

    fn next(&self, _process: &RcProcess, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn last(&self, _process: &RcProcess) -> Result<Term> {
        unimplemented!()
    }

    fn prev(&self, _process: &RcProcess, _key: Term) -> Result<Term> {
        unimplemented!()
    }

    // put
    fn insert(&self, _process: &RcProcess, value: Term, _key_clash_fail: bool) -> Result<()> {
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap
            .write()
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(value);
        Ok(())
    }

    fn get(&self, process: &RcProcess, key: Term) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().get(&key) {
            Some(set) => Ok(set
                .iter()
                .fold(Term::nil(), |acc, v| cons!(heap, v.deep_clone(heap), acc))),
            None => Ok(Term::nil()),
        }
    }

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().get(&key) {
            Some(set) => Ok(set
                .iter()
                .map(|v| {
                    let tup = Tuple::cast_from(&*v).unwrap();
                    assert!(tup.len() > index);
                    tup[index]
                })
                .fold(Term::nil(), |acc, v| cons!(heap, v.deep_clone(heap), acc))),
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

    fn select_replace(&mut self, _process: &RcProcess, _tid: Term, _pattern: Term) -> Result<Term> {
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
