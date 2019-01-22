use crate::bif;
use crate::exports_table::ExportsTable;
use crate::immix::Heap;
use crate::loader::{FuncInfo, Instruction};
use crate::process::InstrPtr;
use crate::value::Term;
use crate::vm::Machine;
use hashbrown::HashMap;

pub type MFA = (u32, u32, u32); // function, arity, label

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
    pub imports: Vec<MFA>, // mod,  func, arity
    pub exports: Vec<MFA>, // func, arity, label
    pub literals: Vec<Term>,
    pub literal_heap: Heap,
    pub lambdas: Vec<Lambda>,
    pub funs: HashMap<(u32, u32), u32>, // (fun name as atom, arity) -> offset
    pub instructions: Vec<Instruction>,
    // debugging info
    pub lines: Vec<FuncInfo>,
    /// Atom name of the module.
    pub name: u32,
}

impl Module {
    fn process_exports(&self, exports: &mut ExportsTable) {
        // process_exports
        let funs = &self.funs;
        let module = self as *const Module;
        self.exports.iter().for_each(|export| {
            // a bit awkward, export is (func, arity, label),
            // we need (module, func, arity).
            let mfa = (self.name, export.0, export.1);
            if !bif::is_bif(&mfa) {
                // only export if there's no bif override
                let ptr = InstrPtr {
                    module,
                    ptr: funs[&(export.0, export.1)],
                };
                exports.register(mfa, ptr);
            }
        });
    }
}

pub fn load_module(vm: &Machine, path: &str) -> Result<*const Module, std::io::Error> {
    let mut registry = vm.modules.lock();
    let mut exports = vm.exports.write();

    println!("Loading file: {}", path);
    registry.parse_module(path).map(|module| {
        module.process_exports(&mut *exports);
        module as *const Module
    })
}
