use super::*;
use crate::immix::Heap;
use crate::value::{Cons, Term, TryFrom, TryInto, Tuple, Variant};
use chashmap::CHashMap;

pub(crate) struct HashTable {
    meta: Metadata,
    hashmap: CHashMap<Term, Term>,
    heap: Heap,
}

unsafe impl Sync for HashTable {}
unsafe impl Send for HashTable {}

impl HashTable {
    pub fn new(meta: Metadata, process: &RcProcess) -> Self {
        Self {
            meta,
            hashmap: CHashMap::new(),
            heap: Heap::new(),
        }
    }
}

fn get_key(pos: usize, value: Term) -> Term {
    let tuple = Tuple::try_from(&value).unwrap();
    tuple[pos - 1]
}

impl Table for HashTable {
    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn first(&self, process: &RcProcess) -> Result<Term> {
        unimplemented!()
    }

    fn next(&self, process: &RcProcess, key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn last(&self, process: &RcProcess) -> Result<Term> {
        unimplemented!()
    }

    fn prev(&self, process: &RcProcess, key: Term) -> Result<Term> {
        unimplemented!()
    }

    // put
    fn insert(&self, process: &RcProcess, value: Term, key_clash_fail: bool) -> Result<()> {
        // TODO deep copy that value
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap.insert(key, value);
        Ok(())
    }

    fn get(&self, process: &RcProcess, key: Term) -> Result<Term> {
        let heap = &process.context_mut().heap;

        Ok(self
            .hashmap
            .get(&key)
            // TODO: bag types
            .map(|v| cons!(heap, v.deep_clone(heap), Term::nil()))
            .unwrap_or_else(|| Term::nil()))
    }

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term> {
        let heap = &process.context_mut().heap;

        Ok(self
            .hashmap
            .get(&key)
            // TODO: deep clone
            .map(|v| {
                let tup = Tuple::try_from(&*v).unwrap();
                assert!(tup.len() > index);
                tup[index]
            })
            .map(|v| cons!(heap, v.deep_clone(heap), Term::nil()))
            .unwrap_or_else(|| Term::nil()))
    }

    // contains_key ? why is result a Term, not bool
    fn member(&self, key: Term) -> bool {
        self.hashmap.contains_key(&key)
    }

    // erase  (remove_entry in rust)
    fn remove(&mut self, key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn remove_object(&mut self, object: Term) -> Result<Term> {
        unimplemented!()
    }

    fn slot(&self, slot: Term) -> Result<Term> {
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
        vm: &vm::Machine,
        process: &RcProcess,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
        reverse: bool,
    ) -> Result<Term> {
        let heap = &process.context_mut().heap;
        let res = self
            .hashmap
            .clone() // TODO: eww, temporary until I implement my own buckets
            .into_iter()
            .fold(Term::nil(), |acc, (_key, val)| {
                println!("running select for {}", val);
                match pam::r#match::run(vm, process, pattern, val, flags) {
                    Some(val) => cons!(heap, val, acc),
                    None => acc,
                }
            });
        println!("PAM res: {}", res);
        Ok(res)
    }

    fn select_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_delete(&mut self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_delete_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_count(&self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_count_continue(&self, process: &RcProcess, continuation: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_replace(&mut self, process: &RcProcess, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    fn select_replace_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
        unimplemented!()
    }

    fn take(&mut self, process: &RcProcess, key: Term) -> Result<Term> {
        unimplemented!()
    }

    /// takes reds, then returns new reds (equal to delete_all)
    fn clear(&mut self, process: &RcProcess, reds: usize) -> Result<usize> {
        unimplemented!()
    }
}
