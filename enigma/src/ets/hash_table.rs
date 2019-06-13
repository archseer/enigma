use super::*;
use crate::immix::Heap;
use crate::value::{CastFrom, CastInto, CastIntoMut, Cons, Term, Tuple, Variant};
use error::*;
use hashbrown::HashMap;
use parking_lot::RwLock;

pub(crate) struct HashTable {
    meta: Metadata,
    hashmap: RwLock<HashMap<Term, Term>>,
    heap: Heap,
}

unsafe impl Sync for HashTable {}
unsafe impl Send for HashTable {}

impl HashTable {
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

impl Table for HashTable {
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
        // TODO deep copy that value
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap.write().insert(key, value);
        Ok(())
    }

    fn get(&self, process: &RcProcess, key: Term) -> Result<Term> {
        let heap = &process.context_mut().heap;

        // println!("debug: ----");
        // self.hashmap.clone().into_iter().for_each(|(key, value)| {
        //     println!("key {} value {}", key, value);
        // });
        // println!("debug: end----");
        Ok(self
            .hashmap
            .read()
            .get(&key)
            // TODO: bag types
            .map(|v| cons!(heap, v.deep_clone(heap), Term::nil()))
            .unwrap_or_else(Term::nil))
    }

    fn get_element(&self, process: &RcProcess, key: Term, index: usize) -> Result<Term> {
        let heap = &process.context_mut().heap;

        match self.hashmap.read().get(&key) {
            Some(value) => {
                let tup = Tuple::cast_from(&*value).unwrap();
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

    fn update_element(&self, _process: &RcProcess, key: Term, list: Term) -> Result<Term> {
        let mut table = self.hashmap.write();
        let item = match table.get_mut(&key) {
            Some(item) => item,
            None => return Ok(atom!(FALSE)), // return BadKey
        };
        // println!("item! {}", *item);
        // println!("values {}", list);
        // TODO verify that items are always tuples
        let item: &mut Tuple = match item.cast_into_mut() {
            Ok(t) => t,
            Err(_) => unreachable!(),
        };

        // First verify that list is ok to avoid nasty rollback scenarios
        let list = Cons::cast_from(&list).unwrap();
        let res: std::result::Result<Vec<_>, _> = list
            .iter()
            .map(|val| {
                // value contains tuple
                // tuple is arity 2 {pos, val}
                // and has pos as integer, >= 1 and isn't == to keypos and is in the db term tuple
                // arity range
                if let Ok(tup) = Tuple::cast_from(&val) {
                    if tup.len() == 2 && tup[0].is_smallint() {
                        let pos = (tup[0].to_int().unwrap() - 1) as usize; // 1 indexed
                        if pos != self.meta().keypos && pos < item.len() {
                            return Ok((pos, tup[1]));
                        }
                    }
                }
                // println!("update_element_3 failed!");
                Err(new_error(ErrorKind::BadItem))
            })
            .collect();

        // The point of no return, no failures from here on.
        res?.iter()
            .for_each(|(pos, val)| item[*pos] = val.deep_clone(&self.heap));
        // println!("Updated items!");
        Ok(atom!(TRUE))
    }

    // erase  (remove_entry in rust)
    fn remove(&self, key: Term) -> Result<Term> {
        Ok(Term::boolean(self.hashmap.write().remove(&key).is_some()))
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
        vm: &vm::Machine,
        process: &RcProcess,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
        _reverse: bool,
    ) -> Result<Term> {
        let heap = &process.context_mut().heap;
        let values: Vec<_> = self
            .hashmap
            .read()
            .iter()
            .filter_map(|(_key, val)| pam::r#match::run(vm, process, pattern, *val, flags))
            .collect();

        // TODO: need the intermediate because of reverse, in the future don't reverse
        let res = values
            .into_iter()
            .rev()
            .fold(Term::nil(), |acc, val| cons!(heap, val, acc));
        // println!("PAM res: {}", res);
        Ok(res)
    }

    // fn select_continue(&mut self, process: &RcProcess, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_delete(
        &self,
        vm: &vm::Machine,
        process: &RcProcess,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
    ) -> Result<Term> {
        let heap = &process.context_mut().heap;
        let mut count = 0;
        let am_true = atom!(TRUE);
        self.hashmap.write().retain(|_key, val| {
            // println!("running retain for {}", val);
            match pam::r#match::run(vm, process, pattern, *val, flags) {
                Some(res) if res == am_true => {
                    // println!("deleting {}", val);
                    count += 1;
                    false
                } // don't keep
                _ => true,
            }
        });
        Ok(Term::uint(heap, count as u32))
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
