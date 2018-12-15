use crate::loader::{Instruction, Loader};
use crate::value::Value;
use crate::vm;
use fnv::FnvHashMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub type MFA = (usize, usize, u32); // function, arity, label

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
    pub lambdas: Vec<Lambda>,
    pub funs: FnvHashMap<(usize, usize), usize>, // (fun name as atom, arity) -> offset
    pub labels: FnvHashMap<usize, usize>,        // label -> offset
    pub instructions: Vec<Instruction>,
}

pub fn load_file(vm: &vm::Machine, path: &str) -> Result<Module, std::io::Error> {
    let mut file = File::open("foo.txt")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    let loader = Loader::new(&vm);
    let module = loader.load_file(&bytes[..]).unwrap();
    vm.register_module(module);
    Ok(module)
}
