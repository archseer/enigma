use crate::immix::Heap;
use crate::value::Term;
use hashbrown::HashMap;
use parking_lot::RwLock;

// erase(key)
// get()
// info()

pub struct Table {
    map: RwLock<HashMap<Term, Term>>,
    heap: Heap,
}

impl Table {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            heap: Heap::new(),
        }
    }

    // pub fn erase(&self, key: Term) -> bool {
    //     self.map.write().remove(&key).is_some()
    // }

    // pub fn all() -> &[Term] {}

    pub fn get(&self, key: Term) -> Option<Term> {
        self.map.read().get(&key).copied()
    }

    pub fn put(&self, key: Term, value: Term) {
        self.map.write().insert(key, value.deep_clone(&self.heap)); // TODO: copy key too?
    }

    // pub fn info() -> Term {
    //     // #{ count, memory }
    // }
}

// need get and put

pub mod bif {
    use crate::bif::Result;
    use crate::process::RcProcess;
    use crate::value::Term;
    use crate::vm;

    pub fn get_1(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> Result {
        match vm.persistent_terms.get(args[0]) {
            Some(val) => Ok(val),
            None => Err(badarg!()), // default
        }
    }

    pub fn get_2(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> Result {
        match vm.persistent_terms.get(args[0]) {
            Some(val) => Ok(val),
            None => Ok(args[1]), // default
        }
    }

    pub fn put_2(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> Result {
        vm.persistent_terms.put(args[0], args[1]);
        Ok(atom!(OK))
    }
}
