use crate::atom;
use crate::immix::Heap;
use crate::bitstring::Binary;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto};
use crate::vm;
use std::fs::File;
use std::io::Read;

fn error_to_tuple(heap: &Heap, error: std::io::Error) -> Term {
    // TODO:
    tup2!(heap, atom!(ERROR), atom!(VALUE))
}

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

/// Reads an entire file into \c result, stopping after \c size bytes or EOF. It will read until
/// EOF if size is 0.
pub fn read_file_nif_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // arg[0] = filename
    let heap = &process.context_mut().heap;

    // read into buffer, alloc into RcBinary
    println!("before cast {}", args[0]);

    // TODO bitstrings or non zero offsets can fail ...
    let path = match args[0].try_into() {
       Ok(cons) => value::cons::unicode_list_to_buf(cons, 2048).unwrap(),
       _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    println!("Reading {}", path);

    // TODO: need feature(try_blocks) for try { }
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Ok(error_to_tuple(heap, err))
    };

    println!("file");

    // TODO: maybe read file metadata and preallocate with_capacity
    let mut bytes = Vec::new();

    if let Err(err) = file.read_to_end(&mut bytes) {
        return Ok(error_to_tuple(heap, err))
    };
    println!("{:?} bytes", bytes);
    Ok(tup2!(heap, atom!(OK), Term::binary(heap, Binary::from(bytes))))
}

// TODO: maybe we should pass around as OsString which is null terminated dunno
pub fn internal_native2name_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // we already validated the name into unicode in the previous command
    return Ok(args[0])
}

pub fn internal_name2native_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // we already validated the name into unicode in the previous command
    return Ok(args[0])
}

#[cfg(test)]
mod tests {
    use super::*;

}
