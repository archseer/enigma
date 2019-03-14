use crate::atom;
use crate::bif;
use crate::bitstring;
use crate::ets;
use crate::exception::{Exception, Reason, StackTrace};
use crate::loader;
use crate::module;
use crate::process::{self, Process};
use crate::value::{self, Cons, Term, TryFrom, TryInto, Tuple, Variant};
use crate::vm;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::pin::Pin;
use std::sync::Arc;

pub mod arith;
mod chrono;
pub mod erlang;
mod info;
mod lists;
mod load;
mod maps;
mod os;
mod pdict;
mod prim_file;

macro_rules! trap {
    ($context:expr, $ptr:expr, $($arg:expr),*) => {{
        // TODO set current arity
        $context.ip = $ptr;
        let mut _i = 0usize;
        $(
            $context.x[_i] = $arg;
            _i += 1usize;
        )*
        return Err(Exception::new(Reason::TRAP));
    }};
}

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

pub type Result = std::result::Result<Term, Exception>;
pub type Fn = fn(&vm::Machine, &Pin<&mut Process>, &[Term]) -> Result;
type BifTable = HashMap<module::MFA, Fn>;

pub static BIFS: Lazy<BifTable> = sync_lazy! {
    bif_map![
        "erlang" => {
            "abs", 1 => arith::abs_1,
            "date", 0 => chrono::date_0,
            "localtime", 0 => chrono::localtime_0,
            "monotonic_time", 0 => chrono::monotonic_time_0,
            "monotonic_time", 1 => chrono::monotonic_time_1,
            "system_time", 0 => chrono::system_time_0,
            "system_time", 1 => chrono::system_time_1,
            "universaltime", 0 => chrono::universaltime_0,
            "posixtime_to_universaltime", 1 => chrono::posixtime_to_universaltime_1,
            "universaltime_to_localtime", 1 => chrono::universaltime_to_localtime_1,
            "+", 2 => arith::add_2,
            "-", 2 => arith::sub_2,
            "*", 2 => arith::mult_2,
            "div", 2 => arith::intdiv_2,
            "rem", 2 => arith::mod_2,// TODO: confirm if this is ok
            "spawn", 3 => bif_erlang_spawn_3,
            "spawn_link", 3 => bif_erlang_spawn_link_3,
            "spawn_opt", 1 => bif_erlang_spawn_opt_1,
            "link", 1 => bif_erlang_link_1,
            "unlink", 1 => bif_erlang_unlink_1,
            "monitor", 2 => bif_erlang_monitor_2,
            "demonitor", 1 => bif_erlang_demonitor_1,
            "demonitor", 2 => bif_erlang_demonitor_2,
            "self", 0 => bif_erlang_self_0,
            "send", 2 => bif_erlang_send_2,
            "send", 3 => bif_erlang_send_2,// TODO: send/3 acts as send/2 until distributed nodes work
            "!", 2 => bif_erlang_send_2,
            "is_atom", 1 => bif_erlang_is_atom_1,
            "is_list", 1 => bif_erlang_is_list_1,
            "is_tuple", 1 => bif_erlang_is_tuple_1,
            "is_float", 1 => bif_erlang_is_float_1,
            "is_integer", 1 => bif_erlang_is_integer_1,
            "is_number", 1 => bif_erlang_is_number_1,
            "is_port", 1 => bif_erlang_is_port_1,
            "is_reference" , 1 => bif_erlang_is_reference_1,
            "is_pid" , 1 => bif_erlang_is_pid_1,
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
            "length", 1 => bif_erlang_length_1,
            "error", 1 => bif_erlang_error_1,
            "error", 2 => bif_erlang_error_2,
            "raise", 3 => bif_erlang_raise_3,
            "throw", 1 => bif_erlang_throw_1,
            "exit", 1 => bif_erlang_exit_1,
            "exit", 2 => bif_erlang_exit_2,
            "whereis", 1 => bif_erlang_whereis_1,
            "nif_error", 1 => bif_erlang_nif_error_1,
            "nif_error", 2 => bif_erlang_nif_error_2,
            "load_nif", 2 => bif_erlang_load_nif_2,
            "apply", 2 => bif_erlang_apply_2,
            "apply", 3 => bif_erlang_apply_3,
            "register", 2 => bif_erlang_register_2,
            "unregister", 1 => bif_erlang_unregister_1,
            "function_exported", 3 => bif_erlang_function_exported_3,
            "module_loaded", 1 => bif_erlang_module_loaded_1,
            "process_flag", 2 => bif_erlang_process_flag_2,
            "process_info", 2 => info::process_info_2,
            "group_leader", 0 => info::group_leader_0,
            "make_tuple", 2 => erlang::make_tuple_2,
            "make_tuple", 3 => erlang::make_tuple_3,
            "append_element", 2 => erlang::append_element_2,
            "setelement", 3 => erlang::setelement_3,
            "element", 2 => erlang::element_2,
            "tuple_to_list", 1 => erlang::tuple_to_list_1,
            "binary_to_list", 1 => erlang::binary_to_list_1,
            "binary_to_term", 1 => erlang::binary_to_term_1,
            "list_to_atom", 1 => erlang::list_to_atom_1,
            "list_to_binary", 1 => erlang::list_to_binary_1,
            "atom_to_list", 1 => erlang::atom_to_list_1,
            "integer_to_list", 1 => erlang::integer_to_list_1,
            "list_to_integer", 1 => erlang::list_to_integer_1,
            "++", 2 => erlang::append_2,
            "append", 2 => erlang::append_2,
            "make_ref", 0 => erlang::make_ref_0,
            "process_info", 2 => info::process_info_2,
            "system_info", 1 => info::system_info_1,
            "get_module_info", 2 => load::get_module_info_2,
            "node", 0 => erlang::node_0,
            "node", 1 => erlang::node_1,
            "display", 1 => erlang::display_1,
            "display_string", 1 => erlang::display_string_1,

            // logic
            "and", 2 => erlang::and_2,
            "or", 2 => erlang::or_2,
            "xor", 2 => erlang::xor_2,
            "not", 1 => erlang::not_1,

            ">", 2 => erlang::sgt_2,
            ">=", 2 => erlang::sge_2,
            "<", 2 => erlang::slt_2,
            "=<", 2 => erlang::sle_2,
            "=:=", 2 => erlang::seq_2,
            "==", 2 => erlang::seqeq_2,
            "=/=", 2 => erlang::sneq_2,
            "/=", 2 => erlang::sneqeq_2,
            // "/", 2 => erlang::div_2,
            // "bor", 2 => erlang::bor_2,
            // "band", 2 => erlang::band_2,
            // "bxor", 2 => erlang::bxor_2,
            // "bsl", 2 => erlang::bsl_2,
            // "bsr", 2 => erlang::bsr_2,
            // "bnot", 1 => erlang::bnot_1,
            // "-", 1 => erlang::sminus_1,
            // "+", 1 => erlang::splus_1,

           // loader
            "prepare_loading", 2 => load::prepare_loading_2,
            "has_prepared_code_on_load", 1 => load::has_prepared_code_on_load_1,
            "finish_loading", 1 => load::finish_loading_1,
            "pre_loaded", 0 => load::pre_loaded_0,

            // pdict
            "get", 0 => pdict::get_0,
            "get", 1 => pdict::get_1,
            "get_keys", 0 => pdict::get_keys_0,
            "get_keys", 1 => pdict::get_keys_1,
            "put", 2 => pdict::put_2,
            "erase", 0 => pdict::erase_0,
            "erase", 1 => pdict::erase_1,
        },
        "math" => {
            "cos", 1 => arith::math_cos_1,
            "cosh", 1 => arith::math_cosh_1,
            "sin", 1 => arith::math_sin_1,
            "sinh", 1 => arith::math_sinh_1,
            "tan", 1 => arith::math_tan_1,
            "tanh", 1 => arith::math_tanh_1,
            "acos", 1 => arith::math_acos_1,
            "acosh", 1 => arith::math_acosh_1,
            "asin", 1 => arith::math_asin_1,
            "asinh", 1 => arith::math_asinh_1,
            "atan", 1 => arith::math_atan_1,
            "atanh", 1 => arith::math_atanh_1,
            "erf", 1 => arith::math_erf_1,
            "erfc", 1 => arith::math_erfc_1,
            "exp", 1 => arith::math_exp_1,
            "log", 1 => arith::math_log_1,
            "log", 1 => arith::math_log_1,
            "log2", 1 => arith::math_log2_1,
            "log10", 1 => arith::math_log10_1,
            "sqrt", 1 => arith::math_sqrt_1,
            "atan2", 2 => arith::math_atan2_2,
            "pow", 2 => arith::math_pow_2,
        },
        "lists" => {
            "member", 2 => lists::member_2,
            "reverse", 2 => lists::reverse_2,
            "keymember", 3 => lists::keymember_3,
            "keysearch", 3 => lists::keysearch_3,
            "keyfind", 3 => lists::keyfind_3,
        },
        "maps" => {
            "find", 2 => maps::find_2,
            "get", 2 => maps::get_2,
            "from_list", 1 => maps::from_list_1,
            "to_list", 1 => maps::to_list_1,
            "is_key", 2 => maps::is_key_2,
            "keys", 1 => maps::keys_1,
            "merge", 2 => maps::merge_2,
            "put", 3 => maps::put_3,
            "remove", 2 => maps::remove_2,
            "update", 3 => maps::update_3,
            "values", 1 => maps::values_1,
            "take", 2 => maps::take_2,
            "new", 0 => maps::new_0,
        },
        "ets" => {
            "new", 2 => ets::bif::new_2,
            "whereis", 1 => ets::bif::whereis_1,
            "insert", 2 => ets::bif::insert_2,
            "insert_new", 2 => ets::bif::insert_new_2,
            "lookup", 2 => ets::bif::lookup_2,
            "lookup_element", 3 => ets::bif::lookup_element_3,
            "delete", 1 => ets::bif::delete_1,
            "select", 2 => ets::bif::select_2,
            "select_delete", 2 => ets::bif::select_delete_2,
            "update_element", 3 => ets::bif::update_element_3,
        },
        "os" => {
            "list_env_vars", 0 => os::list_env_vars_0,
            "get_env_var", 1 => os::get_env_var_1,
            "set_env_var", 2 => os::set_env_var_2,
            "unset_env_var", 1 => os::unset_env_var_1,
        },
        "erts_internal" => {
            "group_leader", 2 => info::group_leader_2,
            "garbage_collect", 1 => garbage_collect_1,
        },
    ]
};

type NifTable = HashMap<u32, Vec<(u32, u32, Fn)>>;

macro_rules! nif_map {
    ($($module:expr => {$($fun:expr, $arity:expr => $rust_fn:path,)*},)*) => {
        {
            let mut table: NifTable = HashMap::new();
            $(
                let module = atom::from_str($module);
                table.insert(module, vec![
                    $((atom::from_str($fun), $arity, $rust_fn),)*
                ]);
            )*
            table
        }
    };
}

pub static NIFS: Lazy<NifTable> = sync_lazy! {
    nif_map![
        "prim_file" => {
            "get_cwd_nif", 0 => prim_file::get_cwd_nif_0,
            "read_file_nif", 1 => prim_file::read_file_nif_1,
            "read_info_nif", 2 => prim_file::read_info_nif_2,
            "list_dir_nif", 1 => prim_file::list_dir_nif_1,
            "internal_native2name", 1 => prim_file::internal_native2name_1,
            "internal_name2native", 1 => prim_file::internal_name2native_1,
        },
    ]
};

#[inline]
pub fn is_bif(mfa: &module::MFA) -> bool {
    BIFS.contains_key(mfa)
}

#[inline]
pub fn apply(
    vm: &vm::Machine,
    process: &Pin<&mut Process>,
    mfa: &module::MFA,
    args: &[Term],
) -> Result {
    println!("bif_apply {}", mfa);
    match BIFS.get(mfa) {
        Some(fun) => fun(vm, process, args),
        None => unimplemented!("BIF {} not implemented", mfa),
    }
}

/// Bif implementations
fn bif_erlang_spawn_3(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
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

fn bif_erlang_spawn_link_3(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
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

fn bif_erlang_spawn_opt_1(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    use process::SpawnFlag;

    // arg 0 is a 4 value tuple
    let tup: &Tuple = match Tuple::try_from(&args[0]) {
        Ok(tup) => {
            if tup.len() != 4 {
                return Err(Exception::new(Reason::EXC_BADARG));
            }
            tup
        }
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let module = match tup[0].into_variant() {
        Variant::Atom(module) => module,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let func = match tup[1].into_variant() {
        Variant::Atom(func) => func,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arglist = tup[2];

    let opts = Cons::try_from(&tup[3])?;

    let flag = opts.iter().fold(SpawnFlag::NONE, |acc, val| {
        match val.into_variant() {
            Variant::Atom(atom::LINK) => acc | SpawnFlag::LINK,
            Variant::Atom(atom::MONITOR) => acc | SpawnFlag::MONITOR,
            _ => {
                if let Ok(tup) = Tuple::try_from(&val) {
                    if tup.len() != 2 {
                        unimplemented!("error!");
                        // return Err(Exception::new(Reason::EXC_BADARG));
                    }
                    match tup[0].into_variant() {
                        Variant::Atom(atom::MESSAGE_QUEUE_DATA) => acc, // TODO: implement
                        opt => unimplemented!("Unimplemented spawn_opt for {}", opt),
                    }
                } else {
                    unimplemented!("Unimplemented spawn_opt for badarg");
                    // return Err(Exception::new(Reason::EXC_BADARG));
                }
            }
        }
    });

    let registry = vm.modules.lock();
    let module = registry.lookup(module).unwrap();
    // TODO: avoid the clone here since we copy later
    process::spawn(&vm.state, process, module, func, arglist, flag)
}

fn bif_erlang_link_1(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // arg[0] = pid/port

    match args[0].into_variant() {
        Variant::Pid(pid) => {
            if pid == process.pid {
                return Ok(atom!(TRUE));
            }

            {
                // scope the process_table lock
                if !vm.state.process_table.lock().contains_key(pid) {
                    // if pid doesn't exist fail with noproc
                    if process
                        .local_data()
                        .flags
                        .contains(process::Flag::TRAP_EXIT)
                    {
                        return Err(Exception::new(Reason::EXC_NOPROC));
                    } else {
                        // if trapping exits, fail with exit signal that has reason noproc instead
                        let heap = &process.context_mut().heap;
                        let from = Term::pid(process.pid);
                        process::send_message(
                            &vm.state,
                            process,
                            from,
                            tup3!(heap, atom!(EXIT), from, atom!(NOPROC)),
                        );
                        return Ok(atom!(TRUE));
                    }
                }
            }

            // add the pid to our link tree
            process.local_data_mut().links.insert(pid);

            // send LINK signal to the other process return true
            process::send_signal(&vm.state, pid, process::Signal::Link { from: process.pid });
            // TODO do we need to check the return value here? ^^
            Ok(atom!(TRUE))
        }
        Variant::Port(_) => unimplemented!(),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn bif_erlang_unlink_1(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // arg[0] = pid/port

    match args[0].into_variant() {
        Variant::Pid(pid) => {
            // remove the pid to our link tree
            process.local_data_mut().links.remove(&pid);

            // send LINK signal to the other process return true
            process::send_signal(
                &vm.state,
                pid,
                process::Signal::Unlink { from: process.pid },
            );
            Ok(atom!(TRUE))
        }
        // TODO: port
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn bif_erlang_monitor_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // arg[0] = pid/port
    let heap = &process.context_mut().heap;
    let reference = vm.state.next_ref();
    let ref_term = Term::reference(heap, reference);
    // TODO: this will still alloc on badarg ^

    match args[0].into_variant() {
        Variant::Atom(atom::PROCESS) => {
            let pid = match args[1].into_variant() {
                Variant::Pid(pid) => {
                    if pid == process.pid {
                        return Ok(ref_term);
                    }
                    pid
                }
                Variant::Atom(name) => {
                    if let Some(process) = vm.state.process_registry.lock().whereis(name) {
                        process.pid
                    } else {
                        println!("registered name {} not found!", args[1]);
                        return Err(Exception::new(Reason::EXC_BADARG));
                    }
                }
                // TODO: {atom name, node}
                Variant::Pointer(_) => unimplemented!("monitor for {}", args[1]),
                Variant::Port(_) => unimplemented!("monitor for {}", args[1]),
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            };

            // add the pid to our monitor tree
            process.local_data_mut().monitors.insert(reference, pid);

            // send MONITOR signal to the other process return true
            let sent = process::send_signal(
                &vm.state,
                pid,
                process::Signal::Monitor {
                    from: process.pid,
                    reference,
                },
            );

            if !sent {
                process::send_signal(
                    &vm.state,
                    process.pid,
                    process::Signal::MonitorDown {
                        from: process.pid,
                        // TODO: could be just reason: term
                        reason: Exception::with_value(Reason::EXC_ERROR, atom!(NOPROC)),
                        reference,
                    },
                );
            }

            Ok(ref_term)
        }
        Variant::Atom(atom::PORT) => unimplemented!(),
        Variant::Atom(atom::TIME_OFFSET) => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn demonitor(
    vm: &vm::Machine,
    process: &Pin<&mut Process>,
    reference: Term,
) -> std::result::Result<bool, Exception> {
    // TODO: inefficient, we do get_boxed_ twice
    if reference.get_boxed_header() == Ok(value::BOXED_REF) {
        let reference = reference.get_boxed_value().unwrap();
        // remove the pid from our monitor tree
        if let Some(pid) = process.local_data_mut().monitors.remove(&reference) {
            // send DEMONITOR signal to the other process return true
            process::send_signal(
                &vm.state,
                pid,
                process::Signal::Demonitor {
                    from: process.pid,
                    reference: *reference,
                },
            );
            return Ok(true);
        }
        return Ok(false);
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_demonitor_1(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // arg[0] = ref
    demonitor(vm, process, args[0])?;
    Ok(atom!(TRUE))
}

fn bif_erlang_demonitor_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    if args[1].is_nil() {
        return bif_erlang_demonitor_1(vm, process, args);
    }

    let mut flush = false;
    let mut info = false;

    for val in Cons::try_from(&args[1])? {
        match val.into_variant() {
            Variant::Atom(atom::INFO) => info = true,
            Variant::Atom(atom::FLUSH) => flush = true,
            _ => return Err(Exception::new(Reason::EXC_BADARG)),
        }
    }

    let mut res = atom!(TRUE);

    if demonitor(vm, process, args[0])? {
        // ok
        // TODO: if multi and flush, also trap
    } else {
        if info {
            res = atom!(FALSE);
        }

        if flush {
            use crate::exports_table::Export;
            // TODO: precompute this lookup
            let multi = atom!(FALSE); // TODO: get this to work later
            if let Some(Export::Fun(ptr)) = vm.exports.read().lookup(&module::MFA(
                atom::ERTS_INTERNAL,
                atom::FLUSH_MONITOR_MESSAGES,
                3,
            )) {
                trap!(process.context_mut(), ptr, args[0], multi, res);
            } else {
                unreachable!()
            }
        }
    }
    Ok(res)
}

fn bif_erlang_self_0(_vm: &vm::Machine, process: &Pin<&mut Process>, _args: &[Term]) -> Result {
    Ok(Term::pid(process.pid))
}

fn bif_erlang_send_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // args: dest <term (pid/atom)>, msg <term>
    let pid = args[0];
    let msg = args[1];
    process::send_message(&vm.state, process, pid, msg)
}

pub fn bif_erlang_is_atom_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_atom()))
}

pub fn bif_erlang_is_list_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_list()))
}

pub fn bif_erlang_is_tuple_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_tuple()))
}

pub fn bif_erlang_is_float_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_float()))
}

pub fn bif_erlang_is_integer_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_integer()))
}

pub fn bif_erlang_is_number_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_number()))
}

pub fn bif_erlang_is_port_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_port()))
}

pub fn bif_erlang_is_reference_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_ref()))
}

pub fn bif_erlang_is_pid_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_pid()))
}

pub fn bif_erlang_is_binary_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_binary()))
}

pub fn bif_erlang_is_bitstring_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_bitstring()))
}

pub fn bif_erlang_is_function_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_function()))
}

// TODO: is_function_2, is_record

fn bif_erlang_is_boolean_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_boolean()))
}

pub fn bif_erlang_is_map_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    Ok(Term::boolean(args[0].is_map()))
}

fn bif_erlang_tuple_size_1(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    let tuple = Tuple::try_from(&args[0])?;
    Ok(Term::uint(&process.context_mut().heap, tuple.len))
}

fn bif_erlang_byte_size_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    // TODO: implement for SubBinary
    let res = match &args[0].try_into() {
        Ok(str) => {
            let str: &bitstring::RcBinary = str; // type annotation
            str.data.len()
        }
        _ => unimplemented!(),
        //_ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::int(res as i32)) // TODO: cast potentially unsafe
}

pub fn bif_erlang_map_size_1(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    let val = value::Map::try_from(&args[0])?;
    let heap = &process.context_mut().heap;

    Ok(Term::uint(heap, val.0.len() as u32))
}

pub fn bif_erlang_length_1(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    if args[0].is_nil() {
        return Ok(Term::int(0));
    }
    let cons = Cons::try_from(&args[0])?;
    let heap = &process.context_mut().heap;

    Ok(Term::uint(heap, cons.iter().count() as u32))
}

fn bif_erlang_throw_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    Err(Exception::with_value(Reason::EXC_THROWN, args[0]))
}

fn bif_erlang_exit_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    println!("exiting a proc with {}", args[0]);
    Err(Exception::with_value(Reason::EXC_EXIT, args[0]))
}

fn bif_erlang_exit_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // arg[0] pid/port
    // arg[1] reason

    match args[0].into_variant() {
        Variant::Pid(pid) => {
            process::send_signal(
                &vm.state,
                pid,
                // TODO: deep copy reason
                process::Signal::Exit {
                    from: process.pid,
                    reason: Exception::with_value(Reason::EXC_EXIT, args[1]),
                    kind: process::ExitKind::Exit,
                },
            );
            Ok(atom!(TRUE))
        }
        // TODO: port
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn bif_erlang_error_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    println!("raising val {}", args[0]);
    Err(Exception::with_value(Reason::EXC_ERROR, args[0]))
}

fn bif_erlang_error_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0], args[1]),
    ))
}

fn bif_erlang_raise_3(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let heap = &process.context_mut().heap;

    // class, reason, stacktrace
    let mut class = match args[0].into_variant() {
        Variant::Atom(atom::ERROR) => Reason::EXC_ERROR,
        Variant::Atom(atom::EXIT) => Reason::EXC_EXIT,
        Variant::Atom(atom::THROW) => Reason::EXC_THROWN,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    // trace is already provided for us, so strip the flag so it's not overwritten
    class.remove(Reason::EXF_SAVETRACE);

    // TODO: check trace syntax

    let boxed = heap.alloc(value::Boxed {
        header: value::BOXED_STACKTRACE,
        value: StackTrace {
            reason: class, // TODO: use original reason instead
            trace: Vec::new(),
            // TODO: bad
            current: unsafe { std::mem::uninitialized() },
            pc: None,
            complete: true,
        },
    });

    println!("raising with {}", args[2]);
    Err(Exception {
        reason: class,
        value: args[1],
        trace: cons!(heap, args[2], Term::from(boxed)),
    })
}

fn bif_erlang_whereis_1(vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    /* (Atom, Pid|Port)   */
    if let Variant::Atom(name) = args[0].into_variant() {
        if let Some(process) = vm.state.process_registry.lock().whereis(name) {
            return Ok(Term::pid(process.pid));
        }
        return Ok(atom!(UNDEFINED));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_nif_error_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    println!("Tried running nif might be missing!!");
    Err(Exception::with_value(Reason::EXC_ERROR, args[0]))
}

fn bif_erlang_nif_error_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let heap = &process.context_mut().heap;

    Err(Exception::with_value(
        Reason::EXC_ERROR_2,
        tup2!(heap, args[0], args[1]),
    ))
}

fn bif_erlang_load_nif_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    println!("Tried loading nif: {} with args {}", args[0], args[1]);

    use loader::LValue;

    if let Ok(cons) = args[0].try_into() {
        let name = value::cons::unicode_list_to_buf(cons, 2048).unwrap();
        let atom = atom::from_str(&name);
        let nifs = match NIFS.get(&atom) {
            Some(nifs) => nifs,
            None => {
                println!("NIFS name: {}, atom {} not found", name, atom);
                return Ok(Term::atom(atom::OK));
            }
        };

        // TODO: this needs to be ensured to not eval after the module is loaded!
        let module = unsafe { &mut *(process.context_mut().ip.module as *mut module::Module) };

        let mut exports = vm.exports.write();

        for (name, arity, fun) in nifs {
            // find func_info
            if let Some(i) = module.instructions.iter().position(|ins| {
                let lname = LValue::Atom(*name);
                let larity = LValue::Literal(*arity);
                ins.op == crate::opcodes::Opcode::FuncInfo
                    && ins.args[1] == lname
                    && ins.args[2] == larity
            }) {
                let mfa = module::MFA(atom, *name, *arity);
                exports.insert(mfa, crate::exports_table::Export::Bif(*fun));

                let pos = module.imports.len();
                module.imports.push(mfa);
                // replace instruction immediately after with call_nif
                module.instructions[i + 1] = loader::Instruction {
                    op: crate::opcodes::Opcode::CallExtOnly,
                    args: vec![LValue::Literal(*arity), LValue::Literal(pos as u32)],
                };
                println!("NIF replaced {}", mfa);
            } else {
                panic!("NIF stub not found")
            }
        }

        return Ok(Term::atom(atom::OK));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

pub fn bif_erlang_apply_2(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> Result {
    // fun (closure), args
    // maps to i_apply_fun

    unreachable!("apply/2 called without macro override")
}

pub fn bif_erlang_apply_3(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> Result {
    // module, function (atom), args
    unreachable!("apply/3 called without macro override");
    // maps to i_apply
}

/// this sets some process info- trapping exits or the error handler
pub fn bif_erlang_process_flag_2(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
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
        Variant::Atom(atom::PRIORITY) => {
            use process::StateFlag;
            let flag = match args[1].into_variant() {
                // TODO atom to_bool, then pass that in as 2 arg
                Variant::Atom(atom::MAX) => StateFlag::PRQ_MAX,
                Variant::Atom(atom::HIGH) => StateFlag::PRQ_HIGH,
                Variant::Atom(atom::MEDIUM) => StateFlag::PRQ_MEDIUM,
                Variant::Atom(atom::LOW) => StateFlag::PRQ_LOW,
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            };
            let local_data = process.local_data_mut();

            let old_value = match local_data.state & StateFlag::PRQ_MASK {
                StateFlag::PRQ_MAX => atom!(MAX),
                _ => atom!(UNDEFINED),
            };
            local_data.state = flag;
            Ok(old_value)
        }
        Variant::Atom(i) => unimplemented!(
            "erlang:process_flag/2 not implemented for {:?}",
            atom::to_str(i)
        ),
        _ => unreachable!(),
    }
}

/// register(atom, Process|Port) registers a global process or port (for this node)
fn bif_erlang_register_2(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    /* (Atom, Pid|Port)   */
    if let Variant::Atom(name) = args[0].into_variant() {
        // need to lookup first to get a Pin<Arc<>>
        let arc = vm.state.process_table.lock().get(process.pid).unwrap();
        vm.state.process_registry.lock().register(name, arc);

        process.local_data_mut().name = Some(name);
        return Ok(atom!(TRUE));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

/// unregister(atom) unregisters a global process or port (for this node)
fn bif_erlang_unregister_1(
    vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    /* (Atom, Pid|Port)   */
    if let Variant::Atom(name) = args[0].into_variant() {
        let res = vm.state.process_registry.lock().unregister(name);

        return Ok(Term::boolean(res.is_some()));
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

fn bif_erlang_function_exported_3(
    vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
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

fn bif_erlang_module_loaded_1(
    vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> Result {
    if !args[0].is_atom() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let module = args[0].to_u32();

    if vm.modules.lock().lookup(module).is_some() {
        return Ok(atom!(TRUE));
    }
    Ok(atom!(FALSE))
}

// kept the original OTP comment
/* returns the head of a list - this function is unecessary
and is only here to keep Robert happy (Even more, since it's OP as well) */
pub fn bif_erlang_hd_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let cons = Cons::try_from(&args[0])?;
    Ok(cons.head)
}

/* returns the tails of a list - same comment as above */
pub fn bif_erlang_tl_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let cons = Cons::try_from(&args[0])?;
    Ok(cons.tail)
}

pub fn bif_erlang_trunc_1(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    let heap = &process.context_mut().heap;
    match &args[0].into_number() {
        Ok(value::Num::Integer(i)) => Ok(Term::int(*i)),
        Ok(value::Num::Float(f)) => Ok(Term::from(f.trunc())),
        Ok(value::Num::Bignum(v)) => Ok(Term::bigint(heap, v.clone())),
        Err(_) => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

fn garbage_collect_1(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> Result {
    // TODO: GC unimplemented
    Ok(atom!(TRUE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immix::Heap;
    use num::bigint::ToBigInt;

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

    // TODO: test send_2

    #[test]
    fn test_bif_erlang_is_atom_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
        let process = process::allocate(&vm.state, 0, module).unwrap();

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
        let process = process::allocate(&vm.state, 0, module).unwrap();
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
    fn test_bif_tuple_size_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let args = vec![tup3!(heap, Term::int(1), Term::int(2), Term::int(1))];
        let res = bif_erlang_tuple_size_1(&vm, &process, &args);

        assert_eq!(res, Ok(Term::int(3)));
    }

    #[test]
    fn test_bif_map_size_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(3));
        let args = vec![map];
        let res = bif_erlang_map_size_1(&vm, &process, &args);

        assert_eq!(res, Ok(Term::int(2)));
    }
}
