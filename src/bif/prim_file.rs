use crate::atom;
use crate::bitstring::Binary;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto, Tuple};
use crate::vm;

pub fn get_cwd_nif_0(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;

    match std::env::current_dir() {
        Ok(path) => {
            let path = path.to_str().unwrap();
            let bin = Binary::from(path.as_bytes());

            Ok(tup2!(heap, atom!(OK), Term::binary(heap, bin)))
        } 
        _ => return Err(Exception::new(Reason::EXC_INTERNAL_ERROR))
    }
    // TODO: make a function that converts io::Error to a tuple 
}

pub fn internal_native2name_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // we already validated the name into unicode in the previous command
    return Ok(args[0])
}

#[cfg(test)]
mod tests {
    use super::*;

}
