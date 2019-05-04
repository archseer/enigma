use crate::atom;
use crate::bif;
use crate::exports_table::ExportsTable;
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader::{FuncInfo, Instruction};
use crate::value::{self, Term, TryFrom, Variant};
use crate::vm::Machine;
use hashbrown::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MFA(pub u32, pub u32, pub u32);

impl std::fmt::Display for MFA {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{}/{}",
            atom::to_str(self.0).unwrap(),
            atom::to_str(self.1).unwrap(),
            self.2
        )
    }
}

// TODO: to be TryFrom once rust stabilizes the trait
use crate::value::{Boxed, WrongBoxError, BOXED_EXPORT};
impl TryFrom<Term> for MFA {
    type Error = WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == BOXED_EXPORT {
                    return Ok(&(*(ptr as *const Boxed<MFA>)).value);
                }
            }
        }
        Err(WrongBoxError)
    }
}

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
    pub on_load: Option<u32>,
}

impl Module {
    fn process_exports(&self, exports: &mut ExportsTable) {
        // process_exports
        let funs = &self.funs;
        let module = self as *const Module;
        self.exports.iter().for_each(|export| {
            // a bit awkward, export is (func, arity, label),
            // we need (module, func, arity).
            let mfa = MFA(self.name, export.0, export.1);
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

    pub fn load_nifs(&mut self, vm: &Machine, nifs: &[(u32, u32, bif::Fn)]) {
        use crate::loader::LValue;
        let mut exports = vm.exports.write();

        for (name, arity, fun) in nifs {
            // find func_info
            if let Some(i) = self.instructions.iter().position(|ins| {
                let lname = LValue::Atom(*name);
                let larity = LValue::Literal(*arity);
                ins.op == crate::opcodes::Opcode::FuncInfo
                    && ins.args[1] == lname
                    && ins.args[2] == larity
            }) {
                let mfa = MFA(self.name, *name, *arity);
                exports.insert(mfa, crate::exports_table::Export::Bif(*fun));

                let pos = self.imports.len();
                self.imports.push(mfa);
                // replace instruction immediately after with call_nif
                self.instructions[i + 1] = Instruction {
                    op: crate::opcodes::Opcode::CallExtOnly,
                    args: vec![LValue::Literal(*arity), LValue::Literal(pos as u32)],
                };
                println!("NIF replaced {}", mfa);
            } else {
                panic!("NIF stub not found")
            }
        }
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

pub fn finish_loading_modules(vm: &Machine, modules: Vec<Box<Module>>) {
    for module in modules {
        let mut registry = vm.modules.lock();
        let module = registry.add_module(module.name, module);

        {
            let mut exports = vm.exports.write();
            module.process_exports(&mut *exports);
        } // drop exports here so load_nifs will not deadlock

        // FIXME: this is a nasty patch to load_nif on beam_lib.
        if module.name == atom::BEAM_LIB {
            // FIXME: eww, we recast a reference
            let module = unsafe { &mut *(module as *const Module as *mut Module) };
            let nifs = crate::bif::NIFS.get(&atom::BEAM_LIB).unwrap();
            module.load_nifs(vm, nifs);
        }
    }
}

// Ugh
// TODO: to be TryFrom once rust stabilizes the trait
impl TryFrom<Term> for *mut Module {
    type Error = value::WrongBoxError;

    #[inline]
    fn try_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_MODULE {
                    return Ok(&(*(ptr as *const value::Boxed<*mut Module>)).value);
                }
            }
        }
        Err(value::WrongBoxError)
    }
}
