use super::*;
use crate::chashmap::CHashMap;
use crate::immix::Heap;
use crate::value::{Cons, Term, TryFrom, TryInto, TryIntoMut, Tuple, Variant};
use error::*;

pub(crate) struct HashTable {
    meta: Metadata,
    hashmap: CHashMap<Term, Term>,
    heap: Heap,
}

unsafe impl Sync for HashTable {}
unsafe impl Send for HashTable {}

impl HashTable {
    pub fn new(meta: Metadata, process: &Pin<&mut Process>) -> Self {
        Self {
            meta,
            hashmap: CHashMap::new(),
            heap: Heap::new(),
        }
    }
}

fn get_key(pos: usize, value: Term) -> Term {
    let tuple = Tuple::try_from(&value).unwrap();
    tuple[pos]
}

impl Table for HashTable {
    fn meta(&self) -> &Metadata {
        &self.meta
    }

    fn first(&self, process: &Pin<&mut Process>) -> Result<Term> {
        unimplemented!()
    }

    fn next(&self, process: &Pin<&mut Process>, key: Term) -> Result<Term> {
        unimplemented!()
    }

    fn last(&self, process: &Pin<&mut Process>) -> Result<Term> {
        unimplemented!()
    }

    fn prev(&self, process: &Pin<&mut Process>, key: Term) -> Result<Term> {
        unimplemented!()
    }

    // put
    fn insert(&self, process: &Pin<&mut Process>, value: Term, key_clash_fail: bool) -> Result<()> {
        // TODO deep copy that value
        let value = value.deep_clone(&self.heap);
        let key = get_key(self.meta().keypos, value);
        self.hashmap.insert(key, value);
        Ok(())
    }

    fn get(&self, process: &Pin<&mut Process>, key: Term) -> Result<Term> {
        let heap = &process.context_mut().heap;

        println!("debug: ----");
        self.hashmap.clone().into_iter().for_each(|(key, value)| {
            println!("key {} value {}", key, value);
        });
        println!("debug: end----");
        Ok(self
            .hashmap
            .get(&key)
            // TODO: bag types
            .map(|v| cons!(heap, v.deep_clone(heap), Term::nil()))
            .unwrap_or_else(|| Term::nil()))
    }

    fn get_element(&self, process: &Pin<&mut Process>, key: Term, index: usize) -> Result<Term> {
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

    fn update_element(&self, process: &Pin<&mut Process>, key: Term, list: Term) -> Result<Term> {
        let item = match self.hashmap.get_mut(&key) {
            Some(item) => item,
            None => return Ok(atom!(FALSE)), // return BadKey
        };

        println!("item! {}", *item);
        println!("values {}", list);
        // TODO verify that items are always tuples
        let item: &mut Tuple = match item.try_into_mut() {
            Ok(t) => t,
            Err(_) => unreachable!(),
        };

        // First verify that list is ok to avoid nasty rollback scenarios
        let list = Cons::try_from(&list).unwrap();
        let res: std::result::Result<Vec<_>, _> = list
            .iter()
            .map(|val| {
                // value contains tuple
                // tuple is arity 2 {pos, val}
                // and has pos as integer, >= 1 and isn't == to keypos and is in the db term tuple
                // arity range
                if let Ok(tup) = Tuple::try_from(&val) {
                    println!("a");
                    if tup.len() == 2 && tup[0].is_smallint() {
                        println!("tuplen");
                        let pos = (tup[0].to_int().unwrap() - 1) as usize; // 1 indexed
                        println!(
                            "pos {}, keypos {} len {}",
                            pos,
                            self.meta().keypos,
                            item.len()
                        );
                        if pos != self.meta().keypos && pos < item.len() {
                            return Ok((pos, tup[1]));
                        }
                    }
                }
                println!("update_element_3 failed!");
                Err(new_error(ErrorKind::BadItem))
            })
            .collect();

        // The point of no return, no failures from here on.
        res?.iter()
            .for_each(|(pos, val)| item[*pos] = val.deep_clone(&self.heap));
        println!("Updated items!");
        Ok(atom!(TRUE))
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

    // int (*db_select_chunk)(process: &Pin<&mut Process>,
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
        process: &Pin<&mut Process>,
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

    // fn select_continue(&mut self, process: &Pin<&mut Process>, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_delete(
        &self,
        vm: &vm::Machine,
        process: &Pin<&mut Process>,
        pattern: &pam::Pattern,
        flags: pam::r#match::Flag,
    ) -> Result<Term> {
        let heap = &process.context_mut().heap;
        let mut count = 0;
        let am_true = atom!(TRUE);
        self.hashmap.retain(|_key, val| {
            println!("running retain for {}", val);
            match pam::r#match::run(vm, process, pattern, *val, flags) {
                Some(res) if res == am_true => {
                    println!("deleting {}", val);
                    count += 1;
                    false
                } // don't keep
                _ => true,
            }
        });
        Ok(Term::uint(heap, count as u32))
    }

    // fn select_delete_continue(&mut self, process: &Pin<&mut Process>, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_count(&self, process: &Pin<&mut Process>, tid: Term, pattern: Term) -> Result<Term> {
        unimplemented!()
    }

    // fn select_count_continue(&self, process: &Pin<&mut Process>, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn select_replace(
        &mut self,
        process: &Pin<&mut Process>,
        tid: Term,
        pattern: Term,
    ) -> Result<Term> {
        unimplemented!()
    }

    // fn select_replace_continue(&mut self, process: &Pin<&mut Process>, continuation: Term) -> Result<Term> {
    //     unimplemented!()
    // }

    fn take(&mut self, process: &Pin<&mut Process>, key: Term) -> Result<Term> {
        unimplemented!()
    }

    /// takes reds, then returns new reds (equal to delete_all)
    fn clear(&mut self, process: &Pin<&mut Process>, reds: usize) -> Result<usize> {
        unimplemented!()
    }
}
