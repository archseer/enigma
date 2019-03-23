use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::loader::Loader;
use crate::module::{self, Module};
use crate::process::Process;
use crate::value::{self, Cons, Term, TryFrom, TryInto, Variant};
use crate::vm;
use std::pin::Pin;

pub fn pre_loaded_0(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    use std::path::Path;
    let heap = &process.context_mut().heap;

    let iter = vm::PRE_LOADED
        .iter()
        .map(|path| Path::new(path).file_stem().unwrap().to_str().unwrap())
        .map(|name| Term::atom(atom::from_str(name)));

    Ok(Cons::from_iter(iter, heap))
}

pub fn prepare_loading_2(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    // arg[0] module name atom, arg[1] raw bytecode bytes
    let heap = &process.context_mut().heap;

    // TODO merge new + load_file?
    let loader = Loader::new();

    args[1]
        .to_bytes()
        .ok_or_else(|| Exception::new(Reason::EXC_BADARG))
        .and_then(|bytes| {
            loader
                .load_file(bytes)
                // we box to allocate a permanent space, then we unbox since we'll carry around
                // the raw pointer that we will Box::from_raw when finalizing.
                .map(|module| {
                    Term::boxed(heap, value::BOXED_MODULE, Box::into_raw(Box::new(module)))
                })
                .or_else(|_| Ok(tup2!(heap, atom!(ERROR), atom!(BADFILE))))
        })
}

pub fn has_prepared_code_on_load_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    match args[0].try_into() {
        Ok(value) => {
            let value: &*mut Module = value;
            unsafe { Ok(Term::boolean((**value).on_load.is_some())) }
        }
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn finish_loading_1(
    vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    value::Cons::try_from(&args[0])?
        .iter()
        .map(|v| {
            v.try_into()
                .map(|value: &*mut Module| unsafe { Box::from_raw(*value) })
        })
        .collect::<Result<Vec<Box<Module>>, _>>()
        .map_err(|_| Exception::new(Reason::EXC_BADARG))
        .and_then(|mods| {
            module::finish_loading_modules(vm, mods);
            Ok(atom!(OK))
        })
}

pub fn get_module_info_2(
    vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    let name = match args[0].into_variant() {
        Variant::Atom(i) => i,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let registry = vm.modules.lock();
    let module = registry.lookup(name).unwrap();
    let res = get_module_info(module, args[1]).unwrap();
    Ok(res)
}

fn get_module_info(module: &Module, what: Term) -> bif::Result {
    match what.into_variant() {
        Variant::Atom(atom::MODULE) => Ok(Term::atom(module.name)),
        //Variant::Atom(atom::MD5) => md5_of_module(p, code_hdr),
        Variant::Atom(atom::EXPORTS) => unimplemented!(),
        Variant::Atom(atom::FUNCTIONS) => unimplemented!(),
        Variant::Atom(atom::NIFS) => unimplemented!(),
        Variant::Atom(atom::ATTRIBUTES) => unimplemented!(),
        Variant::Atom(atom::COMPILE) => unimplemented!(),
        Variant::Atom(atom::NATIVE_ADDRESSES) => unimplemented!(),
        Variant::Atom(atom::NATIVE) => unimplemented!(),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}
