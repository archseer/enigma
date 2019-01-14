use crate::atom;
use crate::bif;
use crate::bitstring;
use crate::exception::{Exception, Reason};
use crate::module;
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use crate::process::{self, RcProcess};
use crate::servo_arc::Arc;
use crate::value::{self, Value};
use crate::vm;
use hamt_rs::HamtMap;
use hashbrown::HashMap;
use num::bigint::BigInt;
use num::bigint::ToBigInt;
use num::traits::Signed;
use once_cell::sync::Lazy;
use std::i32;
use std::ops::{Add, Mul, Sub};

mod chrono;
mod map;

// maybe use https://github.com/sfackler/rust-phf

type BifResult = Result<Value, Exception>;
pub type BifFn = fn(&vm::Machine, &RcProcess, &[Value]) -> BifResult;
type BifTable = HashMap<(u32, u32, u32), BifFn>;

pub static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs: BifTable = HashMap::new();
    let erlang = atom::from_str("erlang");
    bifs.insert((erlang, atom::from_str("abs"), 1), bif_erlang_abs_1);
    bifs.insert((erlang, atom::from_str("date"), 0), chrono::bif_erlang_date_0);
    bifs.insert((erlang, atom::from_str("localtime"), 0), chrono::bif_erlang_localtime_0);
    bifs.insert((erlang, atom::from_str("monotonic_time"), 0), chrono::bif_erlang_monotonic_time_0);
    bifs.insert((erlang, atom::from_str("system_time"), 0), chrono::bif_erlang_system_time_0);
    bifs.insert((erlang, atom::from_str("universaltime"), 0), chrono::bif_erlang_universaltime_0);
    bifs.insert((erlang, atom::from_str("+"), 2), bif_erlang_add_2);
    bifs.insert((erlang, atom::from_str("-"), 2), bif_erlang_sub_2);
    bifs.insert((erlang, atom::from_str("*"), 2), bif_erlang_mult_2);
    bifs.insert((erlang, atom::from_str("div"), 2), bif_erlang_intdiv_2);
    bifs.insert((erlang, atom::from_str("rem"), 2), bif_erlang_mod_2);
    bifs.insert((erlang, atom::from_str("spawn"), 3), bif_erlang_spawn_3);
    bifs.insert((erlang, atom::from_str("self"), 0), bif_erlang_self_0);
    bifs.insert((erlang, atom::from_str("send"), 2), bif_erlang_send_2);
    bifs.insert((erlang, atom::from_str("is_atom"), 1), bif_erlang_is_atom_1);
    bifs.insert((erlang, atom::from_str("is_list"), 1), bif_erlang_is_list_1);
    bifs.insert((erlang, atom::from_str("is_tuple"), 1), bif_erlang_is_tuple_1);
    bifs.insert((erlang, atom::from_str("is_float"), 1), bif_erlang_is_float_1);
    bifs.insert((erlang, atom::from_str("is_integer"), 1), bif_erlang_is_integer_1);
    bifs.insert((erlang, atom::from_str("is_number"), 1), bif_erlang_is_number_1);
    bifs.insert((erlang, atom::from_str("is_port"), 1), bif_erlang_is_port_1);
    bifs.insert((erlang, atom::from_str("is_reference"), 1), bif_erlang_is_reference_1);
    bifs.insert((erlang, atom::from_str("is_binary"), 1), bif_erlang_is_binary_1);
    bifs.insert((erlang, atom::from_str("is_function"), 1), bif_erlang_is_function_1);
    bifs.insert((erlang, atom::from_str("is_boolean"), 1), bif_erlang_is_boolean_1);
    bifs.insert((erlang, atom::from_str("is_map"), 1), bif_erlang_is_map_1);
    bifs.insert((erlang, atom::from_str("hd"), 1), bif_erlang_hd_1);
    bifs.insert((erlang, atom::from_str("tl"), 1), bif_erlang_tl_1);
    bifs.insert((erlang, atom::from_str("trunc"), 1), bif_erlang_trunc_1);
    bifs.insert((erlang, atom::from_str("tuple_size"), 1), bif_erlang_tuple_size_1);
    bifs.insert((erlang, atom::from_str("byte_size"), 1), bif_erlang_byte_size_1);
    bifs.insert((erlang, atom::from_str("error"), 1), bif_erlang_error_1);
    bifs.insert((erlang, atom::from_str("error"), 2), bif_erlang_error_2);
    //bifs.insert((erlang, atom::from_str("raise"), 3), bif_erlang_raise_3);
    bifs.insert((erlang, atom::from_str("throw"), 1), bif_erlang_throw_1);
    bifs.insert((erlang, atom::from_str("exit"), 1), bif_erlang_exit_1);
    bifs.insert((erlang, atom::from_str("nif_error"), 1), bif_erlang_nif_error_1);
    bifs.insert((erlang, atom::from_str("nif_error"), 2), bif_erlang_nif_error_2);

    bifs.insert((erlang, atom::from_str("load_nif"), 2), bif_erlang_load_nif_2);
    bifs.insert((erlang, atom::from_str("apply"), 2), bif_erlang_apply_2);
    bifs.insert((erlang, atom::from_str("apply"), 3), bif_erlang_apply_3);
    bifs.insert((erlang, atom::from_str("register"), 2), bif_erlang_register_2);
    bifs.insert((erlang, atom::from_str("function_exported"), 3), bif_erlang_function_exported_3);
    bifs.insert((erlang, atom::from_str("process_flag"), 2), bif_erlang_process_flag_2);
    // math
    let math = atom::from_str("math");
    bifs.insert((math, atom::from_str("cos"), 1), bif_math_cos_1);
    bifs.insert((math, atom::from_str("cosh"), 1), bif_math_cosh_1);
    bifs.insert((math, atom::from_str("sin"), 1), bif_math_sin_1);
    bifs.insert((math, atom::from_str("sinh"), 1), bif_math_sinh_1);
    bifs.insert((math, atom::from_str("tan"), 1), bif_math_tan_1);
    bifs.insert((math, atom::from_str("tanh"), 1), bif_math_tanh_1);
    bifs.insert((math, atom::from_str("acos"), 1), bif_math_acos_1);
    bifs.insert((math, atom::from_str("acosh"), 1), bif_math_acosh_1);
    bifs.insert((math, atom::from_str("asin"), 1), bif_math_asin_1);
    bifs.insert((math, atom::from_str("asinh"), 1), bif_math_asinh_1);
    bifs.insert((math, atom::from_str("atan"), 1), bif_math_atan_1);
    bifs.insert((math, atom::from_str("atanh"), 1), bif_math_atanh_1);
    bifs.insert((math, atom::from_str("log"), 1), bif_math_log_1);
    bifs.insert((math, atom::from_str("log2"), 1), bif_math_log2_1);
    bifs.insert((math, atom::from_str("log10"), 1), bif_math_log10_1);
    bifs.insert((math, atom::from_str("sqrt"), 1), bif_math_sqrt_1);
    bifs.insert((math, atom::from_str("atan2"), 2), bif_math_atan2_2);
    // pdict
    bifs.insert((erlang, atom::from_str("get"), 0), bif_erlang_get_0);
    bifs.insert((erlang, atom::from_str("get"), 1), bif_erlang_get_1);
    bifs.insert((erlang, atom::from_str("get_keys"), 0), bif_erlang_get_keys_0);
    bifs.insert((erlang, atom::from_str("get_keys"), 1), bif_erlang_get_keys_1);
    bifs.insert((erlang, atom::from_str("put"), 2), bif_erlang_put_2);
    bifs.insert((erlang, atom::from_str("erase"), 0), bif_erlang_erase_0);
    bifs.insert((erlang, atom::from_str("erase"), 1), bif_erlang_erase_1);
    // lists
    let lists = atom::from_str("lists");
    bifs.insert((lists, atom::from_str("member"), 2), bif_lists_member_2);
    bifs.insert((lists, atom::from_str("reverse"), 2), bif_lists_reverse_2);
    bifs.insert((lists, atom::from_str("keymember"), 3), bif_lists_keymember_3);
    bifs.insert((lists, atom::from_str("keysearch"), 3), bif_lists_keysearch_3);
    bifs.insert((lists, atom::from_str("keyfind"), 3), bif_lists_keyfind_3);
    // maps
    let maps = atom::from_str("maps");
    bifs.insert((maps, atom::from_str("find"), 2), map::bif_maps_find_2);
    bifs.insert((maps, atom::from_str("get"), 2), map::bif_maps_get_2);
    bifs.insert((maps, atom::from_str("from_list"), 1), map::bif_maps_from_list_1);
    bifs.insert((maps, atom::from_str("is_key"), 2), map::bif_maps_is_key_2);
    bifs.insert((maps, atom::from_str("keys"), 1), map::bif_maps_keys_1);
    bifs.insert((maps, atom::from_str("merge"), 2), map::bif_maps_merge_2);
    bifs.insert((maps, atom::from_str("put"), 3), map::bif_maps_put_3);
    bifs.insert((maps, atom::from_str("remove"), 2), map::bif_maps_remove_2);
    bifs.insert((maps, atom::from_str("update"), 3), map::bif_maps_update_3);
    bifs.insert((maps, atom::from_str("values"), 1), map::bif_maps_values_1);
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
fn bif_erlang_spawn_3(vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    if let [Value::Atom(module), Value::Atom(func), arglist] = &args[..] {
        let registry = vm.modules.lock();
        let module = registry.lookup(*module).unwrap();
        // TODO: avoid the clone here since we copy later
        return process::spawn(&vm.state, module, *func, arglist.clone());
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_abs_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(i.abs())),
        Value::Float(value::Float(f)) => Ok(Value::Float(value::Float(f.abs()))),
        Value::BigInt(i) => Ok(Value::BigInt(Box::new((**i).abs()))),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn bif_erlang_add_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(integer_overflow_op!(None, args, add, overflowing_add))
}

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

fn bif_erlang_is_binary_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_binary()))
}

fn bif_erlang_is_function_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_function()))
}

// TODO: is_function_2, is_record

fn bif_erlang_is_boolean_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_boolean()))
}

fn bif_erlang_is_map_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Ok(Value::boolean(args[0].is_map()))
}

macro_rules! trig_func {
    (
    $arg:expr,
    $op:ident
) => {{
        let res = match $arg {
            Value::Integer(i) => i as f64, // TODO: potentially unsafe
            Value::Float(value::Float(f)) => f,
            Value::BigInt(..) => unimplemented!(),
            _ => return Err(Exception::new(Reason::EXC_BADARG)),
        };
        Ok(Value::Float(value::Float(res.$op())))
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
        Value::Float(value::Float(f)) => f,
        Value::BigInt(..) => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arg = match args[1] {
        Value::Integer(i) => i as f64, // TODO: potentially unsafe
        Value::Float(value::Float(f)) => f,
        Value::BigInt(..) => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Value::Float(value::Float(res.atan2(arg))))
}

fn bif_erlang_tuple_size_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let res = match &args[0] {
        Value::Tuple(t) => unsafe { (**t).len },
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Value::Integer(i64::from(res)))
}

fn bif_erlang_byte_size_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let res = match &args[0] {
        Value::Binary(str) => str.data.len(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Value::Integer(res as i64)) // TODO: cast potentially unsafe
}

fn bif_erlang_throw_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_THROWN, args[0].clone()))
}

fn bif_erlang_exit_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_EXIT, args[0].clone()))
}

fn bif_erlang_error_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_ERROR, args[0].clone()))
}

fn bif_erlang_error_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0].clone(), args[1].clone()),
    ))
}

fn bif_erlang_nif_error_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_ERROR, args[0].clone()))
}

fn bif_erlang_nif_error_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0].clone(), args[1].clone()),
    ))
}

fn bif_erlang_load_nif_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    println!("Tried loading nif: {} with args {}", args[0], args[1]);

    Ok(Value::Atom(atom::OK))
}

pub fn bif_erlang_apply_2(_vm: &vm::Machine, _process: &RcProcess, _args: &[Value]) -> BifResult {
    // fun (closure), args
    // maps to i_apply_fun

    unreachable!("apply/2 called without macro override")
}

pub fn bif_erlang_apply_3(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    // module, function (atom), args
    unreachable!("apply/3 called without macro override");
}

/// this sets some process info- trapping exits or the error handler
pub fn bif_erlang_process_flag_2(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Value],
) -> BifResult {
    match &args[0] {
        Value::Atom(atom::TRAP_EXIT) => {
            let context = process.context_mut();
            let old_value = context.flags.contains(process::Flag::TRAP_EXIT);
            match &args[1] {
                // TODO atom to_bool, then pass that in as 2 arg
                Value::Atom(atom::TRUE) => context.flags.set(process::Flag::TRAP_EXIT, true),
                Value::Atom(atom::FALSE) => context.flags.set(process::Flag::TRAP_EXIT, false),
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            }
            Ok(Value::boolean(old_value))
        }
        Value::Atom(i) => unimplemented!(
            "erlang:process_flag/2 not implemented for {:?}",
            atom::from_index(*i)
        ),
        _ => unreachable!(),
    }
}

/// register(atom, Process|Port) registers a global process or port (for this node)
fn bif_erlang_register_2(vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    /* (Atom, Pid|Port)   */
    if let Value::Atom(name) = args[0] {
        vm.state
            .process_registry
            .lock()
            .register(name, process.clone());
        return Ok(Value::Atom(atom::TRUE));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_function_exported_3(
    vm: &vm::Machine,
    process: &RcProcess,
    args: &[Value],
) -> BifResult {
    if !args[0].is_atom() || !args[1].is_atom() || !args[2].is_smallint() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let arity = args[2].to_u32();
    let mfa = (args[0].to_u32(), args[1].to_u32(), arity);

    if vm.exports.read().lookup(&mfa).is_some() || bif::is_bif(&mfa) {
        return Ok(Value::Atom(atom::TRUE));
    }
    Ok(Value::Atom(atom::FALSE))
}

// Process dictionary

/// Get the whole pdict.
fn bif_erlang_get_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict.iter().fold(Value::Nil, |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, key.clone(), val.clone());

        // make cons
        value::cons(heap, tuple, res)
    });
    Ok(result)
}

/// Get the value for key in pdict.
fn bif_erlang_get_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    Ok(pdict
        .get(&(args[0]))
        .cloned() // TODO: try to avoid the clone if possible
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

/// Get all the keys in pdict.
fn bif_erlang_get_keys_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict
        .keys()
        .fold(Value::Nil, |res, key| value::cons(heap, key.clone(), res));
    Ok(result)
}

/// Return all the keys that have val
fn bif_erlang_get_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Value = pdict.iter().fold(Value::Nil, |res, (key, val)| {
        if args[1] == *val {
            value::cons(heap, key.clone(), res)
        } else {
            res
        }
    });
    Ok(result)
}

/// Set the key to val. Return undefined if a key was inserted, or old val if it was updated.
fn bif_erlang_put_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .insert(args[0].clone(), args[1].clone())
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

/// Remove all pdict entries, returning the pdict.
fn bif_erlang_erase_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Value]) -> BifResult {
    // deletes all the entries, returning the whole dict tuple
    let pdict = &mut process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    // we use drain since it means we do a move instead of a copy
    let result: Value = pdict.drain().fold(Value::Nil, |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, key, val);

        // make cons
        value::cons(heap, tuple, res)
    });
    Ok(result)
}

/// Remove a single entry from the pdict and return it.
fn bif_erlang_erase_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // deletes a single entry, returning the val
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .remove(&(args[0]))
        .unwrap_or_else(|| Value::Atom(atom::UNDEFINED)))
}

fn bif_lists_member_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // need to bump reductions as we go
    let reds_left = 1; // read from process
    let mut max_iter = 16 * reds_left;
    // bool non_immed_key;

    if args[1].is_nil() {
        return Ok(Value::Atom(atom::FALSE));
    } else if !args[1].is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let term = &args[0];
    // non_immed_key = is_not_immed(term);
    let mut list = &args[1];

    while let Value::List(l) = *list {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(BIF_P);
            // BIF_TRAP2(bif_export[BIF_lists_member_2], BIF_P, term, list);
            // TODO: ^ trap schedules the process to continue executing (by storing the temp val
            // and passing it in the bif call)
        }

        unsafe {
            let item = &(*l).head;
            if *item == *term {
                // || (non_immed_key && deep_equals) {
                // BIF_RET2(am_true, reds_left - max_iter/16);
                return Ok(Value::Atom(atom::TRUE));
            }
            list = &(*l).tail;
        }
    }

    if !list.is_list() {
        // BUMP_REDS(BIF_P, reds_left - max_iter/16);
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    Ok(Value::Atom(atom::FALSE)) // , reds_left - max_iter/16
}

// static BIF_RETTYPE lists_reverse_alloc(Process *c_p,
//                                        Eterm list_in,
//                                        Eterm tail_in)
// {
//     static const Uint CELLS_PER_RED = 40;

//     Eterm *alloc_top, *alloc_end;
//     Uint cells_left, max_cells;
//     Eterm list, tail;
//     Eterm lookahead;

//     list = list_in;
//     tail = tail_in;

//     cells_left = max_cells = CELLS_PER_RED * ERTS_BIF_REDS_LEFT(c_p);
//     lookahead = list;

//     while (cells_left != 0 && is_list(lookahead)) {
//         lookahead = CDR(list_val(lookahead));
//         cells_left--;
//     }

//     BUMP_REDS(c_p, (max_cells - cells_left) / CELLS_PER_RED);

//     if (is_not_list(lookahead) && is_not_nil(lookahead)) {
//         BIF_ERROR(c_p, BADARG);
//     }

//     alloc_top = HAlloc(c_p, 2 * (max_cells - cells_left));
//     alloc_end = alloc_top + 2 * (max_cells - cells_left);

//     while (alloc_top < alloc_end) {
//         Eterm *pair = list_val(list);

//         tail = CONS(alloc_top, CAR(pair), tail);
//         list = CDR(pair);

//         ASSERT(is_list(list) || is_nil(list));

//         alloc_top += 2;
//     }

//     if (is_nil(list)) {
//         BIF_RET(tail);
//     }

//     ASSERT(is_list(tail) && cells_left == 0);
//     BIF_TRAP2(bif_export[BIF_lists_reverse_2], c_p, list, tail);
// }

// static BIF_RETTYPE lists_reverse_onheap(Process *c_p,
//                                         Eterm list_in,
//                                         Eterm tail_in)
// {
//     static const Uint CELLS_PER_RED = 60;

//     Eterm *alloc_start, *alloc_top, *alloc_end;
//     Uint cells_left, max_cells;
//     Eterm list, tail;

//     list = list_in;
//     tail = tail_in;

//     cells_left = max_cells = CELLS_PER_RED * ERTS_BIF_REDS_LEFT(c_p);

//     ASSERT(HEAP_LIMIT(c_p) >= HEAP_TOP(c_p) + 2);
//     alloc_start = HEAP_TOP(c_p);
//     alloc_end = HEAP_LIMIT(c_p) - 2;
//     alloc_top = alloc_start;

//     /* Don't process more cells than we have reductions for. */
//     alloc_end = MIN(alloc_top + (cells_left * 2), alloc_end);

//     while (alloc_top < alloc_end && is_list(list)) {
//         Eterm *pair = list_val(list);

//         tail = CONS(alloc_top, CAR(pair), tail);
//         list = CDR(pair);

//         alloc_top += 2;
//     }

//     cells_left -= (alloc_top - alloc_start) / 2;
//     HEAP_TOP(c_p) = alloc_top;

//     ASSERT(cells_left >= 0 && cells_left <= max_cells);
//     BUMP_REDS(c_p, (max_cells - cells_left) / CELLS_PER_RED);

//     if (is_nil(list)) {
//         BIF_RET(tail);
//     } else if (is_list(list)) {
//         if (cells_left > CELLS_PER_RED) {
//             return lists_reverse_alloc(c_p, list, tail);
//         }

//         BUMP_ALL_REDS(c_p);
//         BIF_TRAP2(bif_export[BIF_lists_reverse_2], c_p, list, tail);
//     }

//     BIF_ERROR(c_p, BADARG);
// }

fn bif_lists_reverse_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    // Handle legal and illegal non-lists quickly.
    if args[0].is_nil() {
        return Ok(args[1].clone());
    } else if !args[1].is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    /* We build the reversal on the unused part of the heap if possible to save
     * us the trouble of having to figure out the list size. We fall back to
     * lists_reverse_alloc when we run out of space. */
    // if (HeapWordsLeft(BIF_P) > 8) {
    //     return lists_reverse_onheap(BIF_P, BIF_ARG_1, BIF_ARG_2);
    // }

    // return lists_reverse_alloc(BIF_P, BIF_ARG_1, BIF_ARG_2);

    unimplemented!()
}

fn bif_lists_keymember_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let res = keyfind(bif_lists_keyfind_3, process, args);

    if let Ok(Value::Tuple(..)) = res {
        return Ok(Value::Atom(atom::TRUE));
    }
    res
}

fn bif_lists_keysearch_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let res = keyfind(bif_lists_keyfind_3, process, args);

    if let Ok(Value::Tuple(t)) = res {
        let heap = &process.context_mut().heap;
        let tuple = tup2!(heap, Value::Atom(atom::VALUE), Value::Tuple(t));
        return Ok(tuple);
    }
    res
}

fn bif_lists_keyfind_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    keyfind(bif_lists_keyfind_3, process, args)
}

// kept the original OTP comment
/* returns the head of a list - this function is unecessary
and is only here to keep Robert happy (Even more, since it's OP as well) */
fn bif_erlang_hd_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    if let Value::List(cons) = args[0] {
        unsafe { return Ok((*cons).head.clone()) }
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

/* returns the tails of a list - same comment as above */
fn bif_erlang_tl_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    if let Value::List(cons) = args[0] {
        unsafe { return Ok((*cons).tail.clone()) }
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_trunc_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(*i)),
        Value::Float(value::Float(f)) => Ok(Value::Float(value::Float(f.trunc()))),
        Value::BigInt(v) => Ok(Value::BigInt(v.clone())),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

/// Swap process out after this number
const CONTEXT_REDS: usize = 4000;

fn keyfind(_func: BifFn, _process: &RcProcess, args: &[Value]) -> BifResult {
    let mut max_iter: isize = 10 * CONTEXT_REDS as isize;

    let key = &args[0];
    let pos_val = &args[1];
    let mut list = &args[2];

    let pos = pos_val.to_u32() as usize;

    // OTP does 3 different loops based on key type (simple, immed, boxed), but luckily in rust we
    // just rely on Eq/PartialEq.

    while let Value::List(ptr) = *list {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(p);
            // BIF_TRAP3(bif_export[Bif], p, key, pos_val, list);
        }

        let term = unsafe { &(*ptr).head };
        list = unsafe { &(*ptr).tail };
        if let Value::Tuple(ptr) = term {
            let tuple = unsafe { &**ptr };
            if pos <= (tuple.len as usize) && *key == tuple[pos] {
                return Ok(term.clone());
            }
        }
    }

    if !list.is_nil() {
        // BIF_ERROR(p, BADARG);
    }
    Ok(Value::Atom(atom::FALSE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immix::Heap;

    /// Converts an erlang list to a value vector.
    fn to_vec(value: Value) -> Vec<Value> {
        let mut vec = Vec::new();
        unsafe {
            let mut cons = &value;
            while let Value::List(ptr) = *cons {
                vec.push((*ptr).head.clone());
                cons = &(*ptr).tail;
            }
            // lastly, the tail
            vec.push((*cons).clone());
        }
        vec
    }

    /// Converts a value vector to an erlang list.
    fn from_vec(heap: &Heap, vec: Vec<Value>) -> Value {
        vec.into_iter()
            .rev()
            .fold(Value::Nil, |res, val| value::cons(heap, val, res))
    }

    #[test]
    fn test_bif_erlang_abs_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(-1)];
        let res = bif_erlang_abs_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));
    }

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
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_tuple_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &Heap::new();
        let args = vec![tup2!(heap, Value::Integer(1), Value::Integer(2))];
        let res = bif_erlang_is_tuple_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_tuple_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_list_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &Heap::new();
        let args = vec![value::cons(heap, Value::Integer(1), Value::Integer(2))];
        let res = bif_erlang_is_list_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_list_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
    #[test]
    fn test_bif_erlang_is_float_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Float(value::Float(3.00))];
        let res = bif_erlang_is_float_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_float_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_integer_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_integer_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_integer_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_number_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Integer(3)];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Float(value::Float(3.0))];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::BigInt(Box::new(10000_i32.to_bigint().unwrap()))];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_port_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Port(80)];
        let res = bif_erlang_is_port_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_port_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_reference_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Ref(197)];
        let res = bif_erlang_is_reference_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_reference_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_binary_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let arc = Arc::new(bitstring::Binary::new());
        let args = vec![Value::Binary(arc)];
        let res = bif_erlang_is_binary_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_binary_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
    #[test]
    fn test_bif_erlang_is_function_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let func: *const value::Closure = std::ptr::null();
        let args = vec![Value::Closure(func)];
        let res = bif_erlang_is_function_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_function_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
    #[test]
    fn test_bif_erlang_is_boolean_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Atom(atom::TRUE)];
        let res = bif_erlang_is_boolean_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_boolean_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_map_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(Value::Atom(1) => Value::Integer(1));
        let args = vec![map];
        let res = bif_erlang_is_map_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Value::Atom(3)];
        let res = bif_erlang_is_map_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_math_cos_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let args = vec![Value::Integer(1)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(value::Float(1.0_f64.cos()))));

        let args = vec![Value::Float(value::Float(1.0))];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Float(value::Float(1.0_f64.cos()))));
    }

    // TODO: test rest of math funcs

    #[test]
    fn test_bif_tuple_size_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &Heap::new();
        let args = vec![tup3!(
            heap,
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(1)
        )];
        let res = bif_erlang_tuple_size_1(&vm, &process, &args);

        assert_eq!(res, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_bif_pdict() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Value::Atom(1), Value::Integer(2)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Value::Atom(1), Value::Integer(3)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(2)));

        let args = vec![Value::Atom(2), Value::Integer(1)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Value::Atom(2)];
        let res = bif_erlang_get_1(&vm, &process, &args);
        assert_eq!(res, Ok(Value::Integer(1)));

        // TODO: add a assert helper for lists
        let args = vec![];
        let res = bif_erlang_get_0(&vm, &process, &args);
        // assert_eq!(res, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_bif_lists_member_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Value::Atom(1);
        let list = from_vec(heap, vec![Value::Atom(3), Value::Atom(2)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Value::Atom(1);
        let list = from_vec(heap, vec![Value::Atom(3), Value::Atom(2), Value::Atom(1)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(TRUE)));
    }

    #[test]
    fn test_bif_lists_keyfind_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Value::Atom(1);
        let pos = Value::Integer(5);
        let list = from_vec(heap, vec![]);
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Value::Atom(3);
        let pos = Value::Integer(0);
        let target = tup2!(heap, Value::Atom(3), Value::Integer(2));
        let list = from_vec(
            heap,
            vec![
                tup2!(heap, Value::Atom(1), Value::Integer(4)),
                tup2!(heap, Value::Atom(2), Value::Integer(3)),
                target.clone(),
                tup2!(heap, Value::Atom(4), Value::Integer(1)),
            ],
        );
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(target));
    }
}
