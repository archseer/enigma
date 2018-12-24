use crate::immix::Heap;
use crate::loader::Instruction;
use crate::module_registry::RcModuleRegistry;
use crate::value::Value;
use fnv::FnvHashMap;
use std::collections::HashMap;

pub type MFA = (usize, usize, usize); // function, arity, label

#[derive(Debug, PartialEq)]
pub struct Lambda {
    pub name: u32,
    pub arity: u32,
    pub offset: u32,
    pub index: u32,
    pub nfree: u32, // frozen values for closures
    pub ouniq: u32, // ?
}

// TODO: add new, remove pub for all these fields
#[derive(Debug)]
pub struct Module {
    pub atoms: HashMap<usize, usize>, // local -> global mapping
    pub imports: Vec<MFA>,
    pub exports: Vec<MFA>,
    pub literals: Vec<Value>,
    pub literal_heap: Heap,
    pub lambdas: Vec<Lambda>,
    pub funs: FnvHashMap<(usize, usize), usize>, // (fun name as atom, arity) -> offset
    pub instructions: Vec<Instruction>,
}

pub fn load_module(
    registry: &RcModuleRegistry,
    path: &str,
) -> Result<*const Module, std::io::Error> {
    let mut registry = registry.lock().unwrap();
    registry
        .parse_module(path)
        .map(|module| module as *const Module)
}
