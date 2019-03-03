use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::loader::Loader;
use crate::module::{self, Module};
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto};
use crate::vm;

pub fn prepare_loading_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
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
    _process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    match args[0].try_into() {
        Ok(value::Boxed { value, .. }) => {
            let value: &*mut Module = value;
            unsafe { Ok(Term::boolean((**value).on_load.is_some())) }
        }
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn finish_loading_1(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let cons = match args[0].try_into() {
        Ok(cons) => {
            let cons: &value::Cons = cons;
            cons
        }
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    cons.iter()
        .map(|v| match v.try_into() {
            Ok(value::Boxed { value, .. }) => {
                let value: &*mut Module = value;
                Ok(unsafe { Box::from_raw(*value) })
            }
            Err(x) => Err(x),
        })
        .collect::<Result<Vec<Box<Module>>, _>>()
        .map_err(|_| Exception::new(Reason::EXC_BADARG))
        .and_then(|mods| {
            module::finish_loading_modules(vm, mods);
            Ok(atom!(OK))
        })
}
