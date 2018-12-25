use crate::atom;
use crate::module;
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use crate::process::{self, RcProcess};
use crate::value::Value;
use crate::vm;
use fnv::FnvHashMap;
use num::bigint::BigInt;
use once_cell::sync::Lazy;
use std::i32;
use std::ops::{Add, Mul, Sub};

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
    bifs.insert((erlang, atom::i_from_str("is_atom"), 1), Box::new(bif_erlang_is_atom_1));
    bifs.insert((erlang, atom::i_from_str("is_list"), 1), Box::new(bif_erlang_is_list_1));
    bifs.insert((erlang, atom::i_from_str("is_tuple"), 1), Box::new(bif_erlang_is_tuple_1));
    bifs.insert((erlang, atom::i_from_str("is_float"), 1), Box::new(bif_erlang_is_float_1));
    bifs.insert((erlang, atom::i_from_str("is_integer"), 1), Box::new(bif_erlang_is_integer_1));
    bifs.insert((erlang, atom::i_from_str("is_number"), 1), Box::new(bif_erlang_is_number_1));
    bifs.insert((erlang, atom::i_from_str("is_port"), 1), Box::new(bif_erlang_is_port_1));
    bifs.insert((erlang, atom::i_from_str("is_reference"), 1), Box::new(bif_erlang_is_reference_1));
    bifs.insert((erlang, atom::i_from_str("is_function"), 1), Box::new(bif_erlang_is_function_1));
    bifs.insert((erlang, atom::i_from_str("is_boolean"), 1), Box::new(bif_erlang_is_boolean_1));
    // math
    let math = atom::i_from_str("math");
    bifs.insert((math, atom::i_from_str("cos"), 1), Box::new(bif_math_cos_1));
    bifs.insert((math, atom::i_from_str("cosh"), 1), Box::new(bif_math_cosh_1));
    bifs.insert((math, atom::i_from_str("sin"), 1), Box::new(bif_math_sin_1));
    bifs.insert((math, atom::i_from_str("sinh"), 1), Box::new(bif_math_sinh_1));
    bifs.insert((math, atom::i_from_str("tan"), 1), Box::new(bif_math_tan_1));
    bifs.insert((math, atom::i_from_str("tanh"), 1), Box::new(bif_math_tanh_1));
    bifs.insert((math, atom::i_from_str("acos"), 1), Box::new(bif_math_acos_1));
    bifs.insert((math, atom::i_from_str("acosh"), 1), Box::new(bif_math_acosh_1));
    bifs.insert((math, atom::i_from_str("asin"), 1), Box::new(bif_math_asin_1));
    bifs.insert((math, atom::i_from_str("asinh"), 1), Box::new(bif_math_asinh_1));
    bifs.insert((math, atom::i_from_str("atan"), 1), Box::new(bif_math_atan_1));
    bifs.insert((math, atom::i_from_str("atanh"), 1), Box::new(bif_math_atanh_1));
    bifs.insert((math, atom::i_from_str("log"), 1), Box::new(bif_math_log_1));
    bifs.insert((math, atom::i_from_str("log2"), 1), Box::new(bif_math_log2_1));
    bifs.insert((math, atom::i_from_str("log10"), 1), Box::new(bif_math_log10_1));
    bifs.insert((math, atom::i_from_str("sqrt"), 1), Box::new(bif_math_sqrt_1));
    bifs.insert((math, atom::i_from_str("atan2"), 2), Box::new(bif_math_atan2_2));
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
fn bif_erlang_spawn_3(vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
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

fn bif_erlang_is_atom_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_atom()))
}

fn bif_erlang_is_list_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_list()))
}

fn bif_erlang_is_tuple_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_tuple()))
}

fn bif_erlang_is_float_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_float()))
}

fn bif_erlang_is_integer_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_integer()))
}

fn bif_erlang_is_number_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_number()))
}

fn bif_erlang_is_port_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_port()))
}

fn bif_erlang_is_reference_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_ref()))
}

// TODO: is_binary, is_function, is_record

fn bif_erlang_is_function_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_function()))
}

fn bif_erlang_is_boolean_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_boolean()))
}

macro_rules! trig_func {
    (
    $arg:expr,
    $op:ident
) => {{
        let res = match $arg {
            Value::Integer(i) => i as f64, // TODO: potentially unsafe
            Value::Float(f) => f,
            Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
            _ => return Err("argument error".to_string()),
        };
        Ok(Value::Float(res.$op()))
    }};
}

fn bif_math_cos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], cos)
}

fn bif_math_cosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], cosh)
}

fn bif_math_sin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sin)
}

fn bif_math_sinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sinh)
}

fn bif_math_tan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], tan)
}

fn bif_math_tanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], tanh)
}
fn bif_math_acos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], acos)
}

fn bif_math_acosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], acosh)
}

fn bif_math_asin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], asin)
}

fn bif_math_asinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], asinh)
}

fn bif_math_atan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], atan)
}

fn bif_math_atanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], atanh)
}

fn bif_math_log_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], ln)
}

fn bif_math_log2_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], log2)
}

fn bif_math_log10_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], log10)
}

fn bif_math_sqrt_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    trig_func!(args[0], sqrt)
}

fn bif_math_atan2_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let res = match args[0] {
        Value::Integer(i) => i as f64, // TODO: potentially unsafe
        Value::Float(f) => f,
        Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
        _ => return Err("argument error".to_string()),
    };
    let arg = match args[1] {
        Value::Integer(i) => i as f64, // TODO: potentially unsafe
        Value::Float(f) => f,
        Value::BigInt(..) => panic!("Unimplemented math function for BigInt"),
        _ => return Err("argument error".to_string()),
    };
    Ok(Value::Float(res.atan2(arg)))
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

    #[test]
    fn test_bif_erlang_is_atom_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Atom(atom::TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Atom(atom::FALSE)));
    }

    // TODO: test rest of is_type funcs

    #[test]
    fn test_bif_math_cos_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(1)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(1.0_f64.cos())));

        let args = vec![Value::Float(1.0)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(1.0_f64.cos())));
    }
}
