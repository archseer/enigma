use crate::atom;
use crate::bif;
use crate::bitstring;
use crate::exception::{Exception, Reason};
use crate::module;
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use crate::process::{self, RcProcess};
use crate::value::{self, Cons, Term, TryInto, Tuple, Variant};
use crate::vm;
use hashbrown::HashMap;
use num::bigint::BigInt;
use num::bigint::ToBigInt;
use num::traits::Signed;
use once_cell::sync::Lazy;
use statrs;
use std::i32;
use std::ops::{Add, Mul, Sub};

mod chrono;
mod erlang;
mod map;
mod prim_file;

// maybe use https://github.com/sfackler/rust-phf

macro_rules! bif_map {
    ($($module:expr => {$($fun:expr, $arity:expr => $rust_fn:path,)*},)*) => {
        {
            let mut table: BifTable = HashMap::new();
            $(
                let module = atom::from_str($module);
                $(table.insert(module::MFA(module, atom::from_str($fun), $arity), $rust_fn);)*
            )*
            table
        }
    };
}

type BifResult = Result<Term, Exception>;
pub type BifFn = fn(&vm::Machine, &RcProcess, &[Term]) -> BifResult;
type BifTable = HashMap<module::MFA, BifFn>;

pub static BIFS: Lazy<BifTable> = sync_lazy! {
    bif_map![
        "erlang" => {
            "abs", 1 => bif_erlang_abs_1,
            "date", 0 => chrono::bif_erlang_date_0,
            "localtime", 0 => chrono::bif_erlang_localtime_0,
            "monotonic_time", 0 => chrono::bif_erlang_monotonic_time_0,
            "monotonic_time", 1 => chrono::bif_erlang_monotonic_time_1,
            "system_time", 0 => chrono::bif_erlang_system_time_0,
            "universaltime", 0 => chrono::bif_erlang_universaltime_0,
            "+", 2 => bif_erlang_add_2,
            "-", 2 => bif_erlang_sub_2,
            "*", 2 => bif_erlang_mult_2,
            "div", 2 => bif_erlang_intdiv_2,
            "rem", 2 => bif_erlang_mod_2,
            "spawn", 3 => bif_erlang_spawn_3,
            "spawn_link", 3 => bif_erlang_spawn_link_3,
            "self", 0 => bif_erlang_self_0,
            "send", 2 => bif_erlang_send_2,
            "is_atom", 1 => bif_erlang_is_atom_1,
            "is_list", 1 => bif_erlang_is_list_1,
            "is_tuple", 1 => bif_erlang_is_tuple_1,
            "is_float", 1 => bif_erlang_is_float_1,
            "is_integer", 1 => bif_erlang_is_integer_1,
            "is_number", 1 => bif_erlang_is_number_1,
            "is_port", 1 => bif_erlang_is_port_1,
            "is_reference" , 1 => bif_erlang_is_reference_1,
            "is_binary", 1 => bif_erlang_is_binary_1,
            "is_bitstring", 1 => bif_erlang_is_bitstring_1,
            "is_function", 1 => bif_erlang_is_function_1,
            "is_boolean", 1 => bif_erlang_is_boolean_1,
            "is_map", 1 => bif_erlang_is_map_1,
            "hd", 1 => bif_erlang_hd_1,
            "tl", 1 => bif_erlang_tl_1,
            "trunc", 1 => bif_erlang_trunc_1,
            "tuple_size", 1 => bif_erlang_tuple_size_1,
            "byte_size", 1 => bif_erlang_byte_size_1,
            "map_size", 1 => bif_erlang_map_size_1,
            "error", 1 => bif_erlang_error_1,
            "error", 2 => bif_erlang_error_2,
            //"raise", 3 => bif_erlang_raise_3,
            "throw", 1 => bif_erlang_throw_1,
            "exit", 1 => bif_erlang_exit_1,
            "whereis", 1 => bif_erlang_whereis_1,
            "nif_error", 1 => bif_erlang_nif_error_1,
            "nif_error", 2 => bif_erlang_nif_error_2,
            "load_nif", 2 => bif_erlang_load_nif_2,
            "apply", 2 => bif_erlang_apply_2,
            "apply", 3 => bif_erlang_apply_3,
            "register", 2 => bif_erlang_register_2,
            "function_exported", 3 => bif_erlang_function_exported_3,
            "process_flag", 2 => bif_erlang_process_flag_2,
            "make_tuple", 2 => erlang::bif_erlang_make_tuple_2,
            "make_tuple", 3 => erlang::bif_erlang_make_tuple_3,
            "append_element", 2 => erlang::bif_erlang_append_element_2,
            "setelement", 3 => erlang::bif_erlang_setelement_3,
            "tuple_to_list", 1 => erlang::bif_erlang_tuple_to_list_1,
            "binary_to_list", 1 => erlang::bif_erlang_binary_to_list_1,
            "list_to_atom", 1 => erlang::bif_erlang_list_to_atom_1,
            "++", 2 => erlang::bif_erlang_append_2,
            "append", 2 => erlang::bif_erlang_append_2,

            // pdict
            "get", 0 => bif_erlang_get_0,
            "get", 1 => bif_erlang_get_1,
            "get_keys", 0 => bif_erlang_get_keys_0,
            "get_keys", 1 => bif_erlang_get_keys_1,
            "put", 2 => bif_erlang_put_2,
            "erase", 0 => bif_erlang_erase_0,
            "erase", 1 => bif_erlang_erase_1,
        },
        "math" => {
            "cos", 1 => bif_math_cos_1,
            "cosh", 1 => bif_math_cosh_1,
            "sin", 1 => bif_math_sin_1,
            "sinh", 1 => bif_math_sinh_1,
            "tan", 1 => bif_math_tan_1,
            "tanh", 1 => bif_math_tanh_1,
            "acos", 1 => bif_math_acos_1,
            "acosh", 1 => bif_math_acosh_1,
            "asin", 1 => bif_math_asin_1,
            "asinh", 1 => bif_math_asinh_1,
            "atan", 1 => bif_math_atan_1,
            "atanh", 1 => bif_math_atanh_1,
            "erf", 1 => bif_math_erf_1,
            "erfc", 1 => bif_math_erfc_1,
            "exp", 1 => bif_math_exp_1,
            "log", 1 => bif_math_log_1,
            "log", 1 => bif_math_log_1,
            "log2", 1 => bif_math_log2_1,
            "log10", 1 => bif_math_log10_1,
            "sqrt", 1 => bif_math_sqrt_1,
            "atan2", 2 => bif_math_atan2_2,
            "pow", 2 => bif_math_pow_2,
        },
        "lists" => {
            "member", 2 => bif_lists_member_2,
            "reverse", 2 => bif_lists_reverse_2,
            "keymember", 3 => bif_lists_keymember_3,
            "keysearch", 3 => bif_lists_keysearch_3,
            "keyfind", 3 => bif_lists_keyfind_3,
        },
        "maps" => {
            "find", 2 => map::bif_maps_find_2,
            "get", 2 => map::bif_maps_get_2,
            "from_list", 1 => map::bif_maps_from_list_1,
            "is_key", 2 => map::bif_maps_is_key_2,
            "keys", 1 => map::bif_maps_keys_1,
            "merge", 2 => map::bif_maps_merge_2,
            "put", 3 => map::bif_maps_put_3,
            "remove", 2 => map::bif_maps_remove_2,
            "update", 3 => map::bif_maps_update_3,
            "values", 1 => map::bif_maps_values_1,
            "take", 2 => map::bif_maps_take_2,
        },
        "prim_file" => {
            "get_cwd_nif", 0 => prim_file::get_cwd_nif_0,
            "internal_native2name", 1 => prim_file::internal_native2name_1,
        },
    ]
};

#[inline]
pub fn is_bif(mfa: &module::MFA) -> bool {
    BIFS.contains_key(mfa)
}

#[inline]
pub fn apply(vm: &vm::Machine, process: &RcProcess, mfa: &module::MFA, args: &[Term]) -> BifResult {
    (BIFS.get(mfa).unwrap())(vm, process, args)
}

// let val: Vec<_> = module
//     .imports
//     .iter()
//     .map(|mfa| {
//         (
//             atom::to_str(&mfa.0).unwrap(),
//             atom::to_str(&mfa.1).unwrap(),
//             mfa.2,
//         )
//     })
//     .collect();

/// Bif implementations
fn bif_erlang_spawn_3(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    let module = match args[0].into_variant() {
        Variant::Atom(module) => module,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let func = match args[1].into_variant() {
        Variant::Atom(func) => func,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arglist = args[2];

    let registry = vm.modules.lock();
    let module = registry.lookup(module).unwrap();
    // TODO: avoid the clone here since we copy later
    process::spawn(
        &vm.state,
        process,
        module,
        func,
        arglist,
        process::SpawnFlag::NONE,
    )
}

fn bif_erlang_spawn_link_3(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    let module = match args[0].into_variant() {
        Variant::Atom(module) => module,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let func = match args[1].into_variant() {
        Variant::Atom(func) => func,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arglist = args[2];

    let registry = vm.modules.lock();
    let module = registry.lookup(module).unwrap();
    // TODO: avoid the clone here since we copy later
    process::spawn(
        &vm.state,
        process,
        module,
        func,
        arglist,
        process::SpawnFlag::LINK,
    )
}

fn bif_erlang_abs_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    match &args[0].into_number() {
        Ok(value::Num::Integer(i)) => Ok(Term::int(i.abs())),
        Ok(value::Num::Float(f)) => Ok(Term::from(f.abs())),
        Ok(value::Num::Bignum(i)) => Ok(Term::bigint(heap, i.abs())),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn bif_erlang_add_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, add, overflowing_add))
}

fn bif_erlang_sub_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, sub, overflowing_sub))
}

fn bif_erlang_mult_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, mul, overflowing_mul))
}

fn bif_erlang_intdiv_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(
        heap,
        args,
        floored_division,
        overflowing_floored_division
    ))
}

fn bif_erlang_mod_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // TODO: should be rem but it's mod
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, modulo, overflowing_modulo))
}

fn bif_erlang_self_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    Ok(Term::pid(process.pid))
}

fn bif_erlang_send_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // args: dest <term (pid/atom)>, msg <term>
    let pid = args[0];
    let msg = args[1];
    let res = process::send_message(&vm.state, process, pid, msg).unwrap();
    Ok(res)
}

fn bif_erlang_is_atom_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_atom()))
}

fn bif_erlang_is_list_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_list()))
}

fn bif_erlang_is_tuple_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_tuple()))
}

fn bif_erlang_is_float_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_float()))
}

fn bif_erlang_is_integer_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_integer()))
}

fn bif_erlang_is_number_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_number()))
}

fn bif_erlang_is_port_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_port()))
}

fn bif_erlang_is_reference_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_ref()))
}

fn bif_erlang_is_binary_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_binary()))
}

fn bif_erlang_is_bitstring_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_bitstring()))
}

fn bif_erlang_is_function_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_function()))
}

// TODO: is_function_2, is_record

fn bif_erlang_is_boolean_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_boolean()))
}

fn bif_erlang_is_map_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Ok(Term::boolean(args[0].is_map()))
}

macro_rules! trig_func {
    (
    $arg:expr,
    $op:ident
) => {{
        let res = match $arg.into_number() {
            Ok(value::Num::Integer(i)) => f64::from(i),
            Ok(value::Num::Float(f)) => f,
            Ok(value::Num::Bignum(..)) => unimplemented!(),
            Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
        };
        Ok(Term::from(res.$op()))
    }};
}

fn bif_math_cos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], cos)
}

fn bif_math_cosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], cosh)
}

fn bif_math_sin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], sin)
}

fn bif_math_sinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], sinh)
}

fn bif_math_tan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], tan)
}

fn bif_math_tanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], tanh)
}

fn bif_math_acos_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], acos)
}

fn bif_math_acosh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], acosh)
}

fn bif_math_asin_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], asin)
}

fn bif_math_asinh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], asinh)
}

fn bif_math_atan_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], atan)
}

fn bif_math_atanh_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], atanh)
}

fn bif_math_erf_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(statrs::function::erf::erf(res)))
}

fn bif_math_erfc_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(1.0_f64 - statrs::function::erf::erf(res)))
}

fn bif_math_exp_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let res: f64 = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(res.powf(std::f64::consts::E)))
}

fn bif_math_log_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], ln)
}

fn bif_math_log2_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], log2)
}

fn bif_math_log10_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], log10)
}

fn bif_math_sqrt_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    trig_func!(args[0], sqrt)
}

fn bif_math_atan2_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arg = match args[1].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(res.atan2(arg)))
}

fn bif_math_pow_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let base = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let index = match args[1].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    Ok(Term::from(base.powf(index)))
}

fn bif_erlang_tuple_size_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    if let Ok(Tuple { len, .. }) = args[0].try_into() {
        return Ok(Term::int(*len as i32));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_byte_size_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    let res = match &args[0].try_into() {
        Ok(value::Boxed {
            header: value::BOXED_BINARY,
            value: str,
        }) => {
            let str: &bitstring::RcBinary = str; // type annotation
            str.data.len()
        }
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::int(res as i32)) // TODO: cast potentially unsafe
}

fn bif_erlang_map_size_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    if let Ok(value::Map { map, .. }) = &args[0].try_into() {
        return Ok(Term::int(map.len() as i32));
    }
    Err(Exception::with_value(Reason::EXC_BADARG, args[0]))
}

fn bif_erlang_throw_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_THROWN, args[0]))
}

fn bif_erlang_exit_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    println!("exiting a proc with {}", args[0]);
    Err(Exception::with_value(Reason::EXC_EXIT, args[0]))
}

fn bif_erlang_error_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    Err(Exception::with_value(Reason::EXC_ERROR, args[0]))
}

fn bif_erlang_error_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0], args[1]),
    ))
}

fn bif_erlang_whereis_1(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    /* (Atom, Pid|Port)   */
    if let Variant::Atom(name) = args[0].into_variant() {
        if let Some(process) = vm.state.process_registry.lock().whereis(name) {
            return Ok(Term::pid(process.pid));
        }
        return Ok(atom!(UNDEFINED));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_nif_error_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // cheating: we don't have dynamic loading yet, so if there's a nif_error call, assume
    // we were supposed to call an equivalent bif.

    // if undefined or undef, try to lookup a bif with same name and dispatch!
    match args[0].into_variant() {
        Variant::Atom(atom::UNDEFINED) | Variant::Atom(atom::UNDEF) | Variant::Atom(atom::NOT_LOADED) => {
            if let Some((mfa, _)) = process.context().ip.lookup_func_info() {
                if let Some(bif) = BIFS.get(&mfa) {
                    println!("bif as nif {}", mfa);
                    return bif(vm, process, args);
                }
            }
        }
        _ => ()
    };

    Err(Exception::with_value(Reason::EXC_ERROR, args[0]))
}

fn bif_erlang_nif_error_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0], args[1]),
    ))
}

fn bif_erlang_load_nif_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    println!("Tried loading nif: {} with args {}", args[0], args[1]);

    Ok(Term::atom(atom::OK))
}

pub fn bif_erlang_apply_2(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    // fun (closure), args
    // maps to i_apply_fun

    unreachable!("apply/2 called without macro override")
}

pub fn bif_erlang_apply_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    // module, function (atom), args
    unreachable!("apply/3 called without macro override");
    // maps to i_apply
}

/// this sets some process info- trapping exits or the error handler
pub fn bif_erlang_process_flag_2(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Term],
) -> BifResult {
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

/// register(atom, Process|Port) registers a global process or port (for this node)
fn bif_erlang_register_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    /* (Atom, Pid|Port)   */
    if let Variant::Atom(name) = args[0].into_variant() {
        vm.state
            .process_registry
            .lock()
            .register(name, process.clone());
        return Ok(atom!(TRUE));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_function_exported_3(
    vm: &vm::Machine,
    _process: &RcProcess,
    args: &[Term],
) -> BifResult {
    if !args[0].is_atom() || !args[1].is_atom() || !args[2].is_smallint() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let arity = args[2].to_u32();
    let mfa = module::MFA(args[0].to_u32(), args[1].to_u32(), arity);

    if vm.exports.read().lookup(&mfa).is_some() || bif::is_bif(&mfa) {
        return Ok(atom!(TRUE));
    }
    Ok(atom!(FALSE))
}

// Process dictionary

/// Get the whole pdict.
fn bif_erlang_get_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict.iter().fold(Term::nil(), |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, *key, *val);

        // make cons
        value::cons(heap, tuple, res)
    });
    Ok(result)
}

/// Get the value for key in pdict.
fn bif_erlang_get_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    Ok(pdict
        .get(&(args[0]))
        .cloned() // TODO: try to avoid the clone if possible
        .unwrap_or_else(|| atom!(UNDEFINED)))
}

/// Get all the keys in pdict.
fn bif_erlang_get_keys_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict
        .keys()
        .fold(Term::nil(), |res, key| value::cons(heap, *key, res));
    Ok(result)
}

/// Return all the keys that have val
fn bif_erlang_get_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict.iter().fold(Term::nil(), |res, (key, val)| {
        if args[1] == *val {
            value::cons(heap, *key, res)
        } else {
            res
        }
    });
    Ok(result)
}

/// Set the key to val. Return undefined if a key was inserted, or old val if it was updated.
fn bif_erlang_put_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .insert(args[0], args[1])
        .unwrap_or_else(|| atom!(UNDEFINED)))
}

/// Remove all pdict entries, returning the pdict.
fn bif_erlang_erase_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    // deletes all the entries, returning the whole dict tuple
    let pdict = &mut process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    // we use drain since it means we do a move instead of a copy
    let result: Term = pdict.drain().fold(Term::nil(), |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, key, val);

        // make cons
        value::cons(heap, tuple, res)
    });
    Ok(result)
}

/// Remove a single entry from the pdict and return it.
fn bif_erlang_erase_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    // deletes a single entry, returning the val
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict.remove(&(args[0])).unwrap_or_else(|| atom!(UNDEFINED)))
}

fn bif_lists_member_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    // need to bump reductions as we go
    let reds_left = 1; // read from process
    let mut max_iter = 16 * reds_left;
    // bool non_immed_key;

    if args[1].is_nil() {
        return Ok(atom!(FALSE));
    } else if !args[1].is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let term = &args[0];
    // non_immed_key = is_not_immed(term);
    let mut list = &args[1];

    while let Ok(Cons { head, tail }) = list.try_into() {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(BIF_P);
            // BIF_TRAP2(bif_export[BIF_lists_member_2], BIF_P, term, list);
            // TODO: ^ trap schedules the process to continue executing (by storing the temp val
            // and passing it in the bif call)
        }

        if *head == *term {
            // || (non_immed_key && deep_equals) {
            // BIF_RET2(am_true, reds_left - max_iter/16);
            return Ok(atom!(TRUE));
        }
        list = tail;
    }

    if !list.is_list() {
        // BUMP_REDS(BIF_P, reds_left - max_iter/16);
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    Ok(atom!(FALSE)) // , reds_left - max_iter/16
}

// fn lists_reverse_alloc(process: &RcProcess, mut list: Term, mut tail: Term) -> BifResult {
//     const CELLS_PER_RED: usize = 40;

//     let max_cells = CELLS_PER_RED * process.context().reds;
//     let mut cells_left = max_cells;
//     let mut lookahead = list;

//     while cells_left != 0 && lookahead.is_list() {
//         lookahead = CDR(list_val(lookahead));
//         cells_left -= 1;
//     }

//     BUMP_REDS(c_p, (max_cells - cells_left) / CELLS_PER_RED);

//     if !lookahead.is_list() && !lookahead.is_nil() {
//         return Err(Exception::new(Reason::EXC_BADARG));
//     }

//     // alloc_top = HAlloc(c_p, 2 * (max_cells - cells_left));
//     // alloc_end = alloc_top + 2 * (max_cells - cells_left);

//     let heap = &process.context_mut().heap;

//     let list = list.iter().take(max_cells).fold(tail, |acc, val| cons!(heap, val, acc));

//     while alloc_top < alloc_end {
//         Term *pair = list_val(list);

//         tail = CONS(alloc_top, CAR(pair), tail);
//         list = CDR(pair);

//         debug_assert!(list.is_list() || list.is_nil());

//         alloc_top += 2;
//     }

//     if list.is_nil() {
//         return Ok(tail);
//     }

//     debug_assert!(tail.is_list() && cells_left == 0);
//     BIF_TRAP2(bif_export[BIF_lists_reverse_2], process, list, tail);
//     unimplemented!()
// }

fn bif_lists_reverse_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    println!("lists reverse called");
    // Handle legal and illegal non-lists quickly.
    if args[0].is_nil() {
        return Ok(args[1]);
    }

    if !args[0].is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    if let Ok(cons) = args[0].try_into() {
        let cons: &value::Cons = cons; // annoying, need type annotation

        let heap = &process.context_mut().heap;
        // TODO: finish up the reduction counting implementation
        Ok(cons.iter().fold(args[1], |acc, val| cons!(heap, *val, acc)))
    } else {
        Err(Exception::new(Reason::EXC_BADARG))
    }

    /* We build the reversal on the unused part of the heap if possible to save
     * us the trouble of having to figure out the list size. We fall back to
     * lists_reverse_alloc when we run out of space. */
    // if (HeapWordsLeft(BIF_P) > 8) {
    //     return lists_reverse_onheap(BIF_P, BIF_ARG_1, BIF_ARG_2);
    // }

    // return lists_reverse_alloc(BIF_P, BIF_ARG_1, BIF_ARG_2);
}

fn bif_lists_keymember_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    keyfind(bif_lists_keyfind_3, process, args).map(|res| {
        if res.is_tuple() {
            return atom!(TRUE);
        }
        res
    })
}

fn bif_lists_keysearch_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    keyfind(bif_lists_keyfind_3, process, args).map(|res| {
        if res.is_tuple() {
            let heap = &process.context_mut().heap;
            return tup2!(heap, atom!(VALUE), res);
        }
        res
    })
}

fn bif_lists_keyfind_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    keyfind(bif_lists_keyfind_3, process, args)
}

// kept the original OTP comment
/* returns the head of a list - this function is unecessary
and is only here to keep Robert happy (Even more, since it's OP as well) */
fn bif_erlang_hd_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    if let Ok(Cons { head, .. }) = args[0].try_into() {
        return Ok(*head);
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

/* returns the tails of a list - same comment as above */
fn bif_erlang_tl_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> BifResult {
    if let Ok(Cons { tail, .. }) = args[0].try_into() {
        return Ok(*tail);
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_trunc_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    match &args[0].into_number() {
        Ok(value::Num::Integer(i)) => Ok(Term::int(*i)),
        Ok(value::Num::Float(f)) => Ok(Term::from(f.trunc())),
        Ok(value::Num::Bignum(v)) => Ok(Term::bigint(heap, v.clone())),
        Err(_) => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

/// Swap process out after this number
const CONTEXT_REDS: usize = 4000;

fn keyfind(_func: BifFn, _process: &RcProcess, args: &[Term]) -> BifResult {
    let mut max_iter: isize = 10 * CONTEXT_REDS as isize;

    let key = args[0];
    let pos_val = args[1];
    let mut list = &args[2];

    let pos = pos_val.to_u32() as usize;

    // OTP does 3 different loops based on key type (simple, immed, boxed), but luckily in rust we
    // just rely on Eq/PartialEq.

    while let Ok(Cons { head, tail }) = list.try_into() {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(p);
            // BIF_TRAP3(bif_export[Bif], p, key, pos_val, list);
        }

        let term = head;
        list = tail;
        if let Ok(tuple) = term.try_into() {
            let tuple: &Tuple = tuple; // annoying, need type annotation
            if pos <= (tuple.len as usize) && key == tuple[pos] {
                return Ok(*term);
            }
        }
    }

    if !list.is_nil() {
        // BIF_ERROR(p, BADARG);
    }
    Ok(atom!(FALSE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immix::Heap;

    /// Converts an erlang list to a value vector.
    fn to_vec(value: Term) -> Vec<Term> {
        let mut vec = Vec::new();
        let mut cons = &value;
        while let Ok(Cons { head, tail }) = cons.try_into() {
            vec.push(*head);
            cons = &tail;
        }
        // lastly, the tail
        vec.push(*cons);
        vec
    }

    /// Converts a value vector to an erlang list.
    fn from_vec(heap: &Heap, vec: Vec<Term>) -> Term {
        vec.into_iter()
            .rev()
            .fold(Term::nil(), |res, val| value::cons(heap, val, res))
    }

    #[test]
    fn test_bif_erlang_abs_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(-1)];
        let res = bif_erlang_abs_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_bif_erlang_add_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(1), Term::int(2)];
        let res = bif_erlang_add_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(3)));
    }

    #[test]
    fn test_bif_erlang_sub_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(2), Term::int(1)];
        let res = bif_erlang_sub_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_bif_erlang_mult_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(2), Term::int(4)];
        let res = bif_erlang_mult_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(8)));
    }

    #[test]
    fn test_bif_erlang_intdiv_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(8), Term::int(4)];
        let res = bif_erlang_intdiv_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(2)));
    }

    #[test]
    fn test_bif_erlang_mod_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(4), Term::int(3)];
        let res = bif_erlang_mod_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_bif_erlang_self_0() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![];
        let res = bif_erlang_self_0(&vm, &process, &args);
        assert_eq!(res, Ok(Term::pid(process.pid)));
    }

    // TODO: test send_2

    #[test]
    fn test_bif_erlang_is_atom_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_atom_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_tuple_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![tup2!(heap, Term::int(1), Term::int(2))];
        let res = bif_erlang_is_tuple_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_tuple_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_list_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![value::cons(heap, Term::int(1), Term::int(2))];
        let res = bif_erlang_is_list_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_list_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
    #[test]
    fn test_bif_erlang_is_float_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::from(3.00)];
        let res = bif_erlang_is_float_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_float_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_integer_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_integer_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_integer_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_number_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::int(3)];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::from(3.0)];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let heap = &process.context_mut().heap;
        let args = vec![Term::bigint(heap, 10000_i32.to_bigint().unwrap())];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_number_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_port_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::port(80)];
        let res = bif_erlang_is_port_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_port_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_reference_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![Term::reference(heap, 197)];
        let res = bif_erlang_is_reference_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_reference_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_binary_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let str = bitstring::Binary::new();
        let args = vec![Term::binary(heap, str)];
        let res = bif_erlang_is_binary_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_binary_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_bitstring_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let str = bitstring::Binary::new();
        let args = vec![Term::binary(heap, str)];
        let res = bif_erlang_is_bitstring_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_bitstring_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_function_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![Term::closure(
            heap,
            value::Closure {
                mfa: module::MFA(0, 0, 0),
                ptr: 0,
                binding: None,
            },
        )];
        let res = bif_erlang_is_function_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_function_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }
    #[test]
    fn test_bif_erlang_is_boolean_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::atom(atom::TRUE)];
        let res = bif_erlang_is_boolean_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_boolean_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_erlang_is_map_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let map = map!(heap, Term::atom(1) => Term::int(1));
        let args = vec![map];
        let res = bif_erlang_is_map_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(TRUE)));

        let args = vec![Term::atom(3)];
        let res = bif_erlang_is_map_1(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(FALSE)));
    }

    #[test]
    fn test_bif_math_cos_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let args = vec![Term::int(1)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::from(1.0_f64.cos())));

        let args = vec![Term::from(1.0)];
        let res = bif_math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::from(1.0_f64.cos())));
    }

    // TODO: test rest of math funcs

    #[test]
    fn test_bif_tuple_size_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![tup3!(heap, Term::int(1), Term::int(2), Term::int(1))];
        let res = bif_erlang_tuple_size_1(&vm, &process, &args);

        assert_eq!(res, Ok(Term::int(3)));
    }

    #[test]
    fn test_bif_map_size_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(3));
        let args = vec![map];
        let res = bif_erlang_map_size_1(&vm, &process, &args);

        assert_eq!(res, Ok(Term::int(2)));
    }

    #[test]
    fn test_bif_pdict() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let args = vec![Term::atom(1), Term::int(2)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Term::atom(1), Term::int(3)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(2)));

        let args = vec![Term::atom(2), Term::int(1)];
        let res = bif_erlang_put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Term::atom(2)];
        let res = bif_erlang_get_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));

        // TODO: add a assert helper for lists
        let args = vec![];
        let res = bif_erlang_get_0(&vm, &process, &args);
        // assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_bif_lists_member_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Term::atom(1);
        let list = from_vec(heap, vec![Term::atom(3), Term::atom(2)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Term::atom(1);
        let list = from_vec(heap, vec![Term::atom(3), Term::atom(2), Term::atom(1)]);
        let res = bif_lists_member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(TRUE)));
    }

    #[test]
    fn test_bif_lists_keyfind_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Term::atom(1);
        let pos = Term::int(5);
        let list = from_vec(heap, vec![]);
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Term::atom(3);
        let pos = Term::int(0);
        let target = tup2!(heap, Term::atom(3), Term::int(2));
        let list = from_vec(
            heap,
            vec![
                tup2!(heap, Term::atom(1), Term::int(4)),
                tup2!(heap, Term::atom(2), Term::int(3)),
                target,
                tup2!(heap, Term::atom(4), Term::int(1)),
            ],
        );
        let res = bif_lists_keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(target));
    }
}
