use crate::atom;
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use num::bigint::BigInt;
use std::ops::{Add, Mul, Sub};
#[macro_use]
use crate::macros::arith;
use crate::module;
use crate::process::{self, RcProcess};
use crate::value::Value;
use crate::vm;
use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use std::i32;

type BifResult = Result<Value, String>;
type BifFn = fn(&vm::Machine, &RcProcess, &[Value]) -> BifResult;
type BifTable = FnvHashMap<(usize, usize, usize), Box<BifFn>>;

static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs: BifTable = FnvHashMap::default();
    let erlang = atom::i_from_str("erlang");
    bifs.insert((erlang, atom::i_from_str("+"), 2), Box::new(bif_erlang_add_2));
    bifs.insert((erlang, atom::i_from_str("-"), 2), Box::new(bif_erlang_sub_2));
    bifs.insert((erlang, atom::i_from_str("*"), 2), Box::new(bif_erlang_mult_2));
    bifs.insert((erlang, atom::i_from_str("div"), 2), Box::new(bif_erlang_intdiv_2));
    bifs.insert((erlang, atom::i_from_str("rem"), 2), Box::new(bif_erlang_mod_2));
    bifs.insert((erlang, atom::i_from_str("spawn"), 3), Box::new(bif_erlang_spawn_3));
    bifs.insert((erlang, atom::i_from_str("self"), 0), Box::new(bif_erlang_self_0));
    bifs.insert((erlang, atom::i_from_str("send"), 2), Box::new(bif_erlang_send_2));
    bifs
};

#[inline]
pub fn is_bif(mfa: &module::MFA) -> bool {
    BIFS.contains_key(mfa)
}

#[inline]
pub fn apply(
    vm: &vm::Machine,
    process: &RcProcess,
    mfa: &module::MFA,
    args: &[Value],
) -> BifResult {
    (BIFS.get(mfa).unwrap())(vm, process, args)
}

// let val: Vec<_> = module
//     .imports
//     .iter()
//     .map(|mfa| {
//         (
//             atom::from_index(&mfa.0).unwrap(),
//             atom::from_index(&mfa.1).unwrap(),
//             mfa.2,
//         )
//     })
//     .collect();

/// Bif implementations
#[inline]
fn bif_erlang_spawn_3(vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    if let [Value::Atom(module), Value::Atom(func), arglist] = &args[..] {
        let registry = vm.modules.lock().unwrap();
        let module = registry.lookup(*module).unwrap();
        // TODO: avoid the clone here since we copy later
        return process::spawn(&vm.state, module, *func, arglist.clone());
    }
    Err("Invalid arguments to erlang::spawn/3".to_string())
}

#[inline]
fn bif_erlang_add_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, add, overflowing_add))
}

#[inline]
fn bif_erlang_sub_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, sub, overflowing_sub))
}

fn bif_erlang_mult_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, mul, overflowing_mul))
}

fn bif_erlang_intdiv_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(
        None,
        args,
        floored_division,
        overflowing_floored_division
    ))
}

fn bif_erlang_mod_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    // TODO: should be rem but it's mod
    Ok(integer_overflow_op!(None, args, modulo, overflowing_modulo))
}

fn bif_erlang_self_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    Ok(Value::Pid(process.pid))
}

fn bif_erlang_send_2(vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // args: dest <pid>, msg <term>
    let pid = &args[0];
    let msg = &args[1];
    let res = process::send_message(&vm.state, process, pid, msg)
        .unwrap()
        .clone();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bif_erlang_add_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let res = bif_erlang_add_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_bif_erlang_sub_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(2), Value::Integer(1)];
        let res = bif_erlang_sub_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_erlang_mult_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(2), Value::Integer(4)];
        let res = bif_erlang_mult_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(8)));
    }

    #[test]
    fn test_bif_erlang_intdiv_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(8), Value::Integer(4)];
        let res = bif_erlang_intdiv_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(2)));
    }

    #[test]
    fn test_bif_erlang_mod_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(4), Value::Integer(3)];
        let res = bif_erlang_mod_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_erlang_self_0() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![];
        let res = bif_erlang_self_0(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Pid(process.pid)));
    }

    // TODO: test send_2
}
