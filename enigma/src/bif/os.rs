use crate::atom;
use crate::bif;

use crate::exception::{Exception, Reason};

use crate::process::RcProcess;
use crate::value::{self, CastFrom, Cons, Term};
use crate::vm;
use std::env;

pub fn list_env_vars_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(env::vars()
        .map(|(key, val)| tup2!(heap, bitstring!(heap, key), bitstring!(heap, val)))
        .fold(Term::nil(), |acc, val| cons!(heap, val, acc)))
}
pub fn get_env_var_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let cons = Cons::cast_from(&args[0])?;
    let name = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    match env::var(name) {
        Ok(var) => Ok(bitstring!(heap, var)),
        Err(env::VarError::NotPresent) => Ok(atom!(FALSE)),
        Err(env::VarError::NotUnicode(..)) => Err(Exception::new(Reason::EXC_BADARG)), // TODO
    }
}

pub fn set_env_var_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let cons = Cons::cast_from(&args[0])?;
    let name = value::cons::unicode_list_to_buf(cons, 2048).unwrap();
    let cons = Cons::cast_from(&args[1])?;
    let val = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    env::set_var(name, val);
    Ok(atom!(TRUE))
}

pub fn unset_env_var_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let cons = Cons::cast_from(&args[0])?;
    let name = value::cons::unicode_list_to_buf(cons, 2048).unwrap();

    env::remove_var(name);
    Ok(atom!(TRUE))
}

pub fn getpid_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(Term::uint(heap, std::process::id()))
}

#[cfg(test)]
mod tests {}
