use crate::atom;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::{self, RcProcess};
use crate::value::{self, Cons, Term, TryInto, Tuple, Variant};
use crate::vm;

pub fn process_info_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!();
    match args[0].into_variant() {
        Variant::Atom(atom::TRAP_EXIT) => {
            let local_data = process.local_data_mut();
            let old_value = local_data.flags.contains(process::Flag::TRAP_EXIT);
            match args[1].into_variant() {
                // TODO atom to_bool, then pass that in as 2 arg
                Variant::Atom(atom::TRUE) => local_data.flags.set(process::Flag::TRAP_EXIT, true),
                Variant::Atom(atom::FALSE) => local_data.flags.set(process::Flag::TRAP_EXIT, false),
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            }
            Ok(Term::boolean(old_value))
        }
        Variant::Atom(i) => unimplemented!(
            "erlang:process_flag/2 not implemented for {:?}",
            atom::to_str(i)
        ),
        _ => unreachable!(),
    }
}
