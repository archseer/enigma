use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::loader::Loader;
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto};
use crate::vm;
use crate::atom;

pub fn prepare_loading_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // arg[0] module name atom, arg[1] raw bytecode bytes
    let heap = &process.context_mut().heap;

    // TODO merge new + load_file?
    let loader = Loader::new();

    args[1]
        .to_bytes()
        .ok_or(Exception::new(Reason::EXC_BADARG))
        .and_then(|bytes|
                  loader.load_file(bytes)
                  .map(|module| Term::boxed(heap, value::BOXED_MODULE, Box::new(module)))
                  .or_else(|_| Ok(tup2!(heap, atom!(ERROR), atom!(BADFILE))))
        )
}
