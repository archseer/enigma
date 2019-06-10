use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, CastFrom, Variant};
use crate::vm;
use crate::Itertools;

pub fn process_info_aux(
    _vm: &vm::Machine,
    process: &RcProcess,
    item: Term,
    always_wrap: bool,
) -> bif::Result {
    use crate::process::Flag;
    let heap = &process.context_mut().heap;

    // TODO: bump process regs
    // (*reds)++;

    // ASSERT(rp);

    /*
     * Q: Why this ERTS_PI_FLAG_ALWAYS_WRAP flag?
     *
     * A: registered_name is strange. If process has no registered name,
     *    process_info(Pid, registered_name) returns [], and
     *    the result of process_info(Pid) has no {registered_name, Name}
     *    tuple in the resulting list. This is inconsistent with all other
     *    options, but we do not dare to change it.
     *
     *    When process_info/2 is called with a list as second argument,
     *    registered_name behaves as it should, i.e. a
     *    {registered_name, []} will appear in the resulting list.
     *
     *    If ERTS_PI_FLAG_ALWAYS_WRAP is set, process_info_aux() always
     *    wrap the result in a key two tuple.
     */

    let item = match item.into_variant() {
        Variant::Atom(i) => i,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let local_data = process.local_data();

    let res = match item {
        atom::REGISTERED_NAME => {
            if let Some(name) = local_data.name {
                Term::atom(name)
            } else {
                if always_wrap {
                    Term::nil()
                } else {
                    return Ok(Term::nil());
                }
            }
        }
        atom::CURRENT_FUNCTION => {
            //process.context_mut().ip.lookup_func_info()
            unimplemented!()
        }
        atom::CURRENT_LOCATION => {
            let fi = process.context_mut().ip.lookup_func_info().unwrap();
            crate::exception::erts_build_mfa_item(&fi, heap, Term::nil())
        }
        atom::CURRENT_STACKTRACE => {
            let mut trace = Vec::with_capacity(crate::exception::DEFAULT_BACKTRACE_SIZE as usize);
            crate::exception::erts_save_stacktrace(
                process,
                &mut trace,
                crate::exception::DEFAULT_BACKTRACE_SIZE,
            );
            trace.into_iter().rev().fold(Term::nil(), |acc, ptr| {
                let func_info = ptr.lookup_func_info().unwrap();
                cons!(
                    heap,
                    crate::exception::erts_build_mfa_item(&func_info, heap, Term::nil()),
                    acc
                )
            })
        }
        atom::INITIAL_CALL => {
            let call = local_data.initial_call;
            tup3!(
                heap,
                Term::atom(call.0),
                Term::atom(call.1),
                Term::uint(heap, call.2)
            )
        }
        atom::STATUS => {
            // TODO: quick cheat
            atom!(RUNNING)
        }
        atom::MESSAGES => {
            // TODO: quick cheat
            Term::nil()
        }
        atom::MESSAGE_QUEUE_LEN => Term::uint(heap, local_data.mailbox.len() as u32),
        atom::MESSAGE_QUEUE_DATA => unimplemented!(),
        atom::LINKS => local_data
            .links
            .iter()
            .fold(Term::nil(), |acc, pid| cons!(heap, Term::pid(*pid), acc)),
        atom::MONITORED_BY => unimplemented!(),
        atom::DICTIONARY => {
            let pdict = &process.local_data_mut().dictionary;
            let heap = &process.context_mut().heap;

            pdict.iter().fold(Term::nil(), |res, (key, val)| {
                let tuple = tup2!(heap, *key, *val);
                cons!(heap, tuple, res)
            })
        }
        atom::TRAP_EXIT => Term::boolean(local_data.flags.contains(Flag::TRAP_EXIT)),
        atom::ERROR_HANDLER => unimplemented!(),
        atom::HEAP_SIZE => {
            // TODO: temporary
            Term::int(512)
        }
        atom::STACK_SIZE => {
            // TODO: temporary
            Term::int(512)
        }
        atom::MEMORY => {
            // TODO: temporary
            Term::int(1024)
        }
        atom::GARBAGE_COLLECTION => unimplemented!(),
        atom::GARBAGE_COLLECTION_INFO => unimplemented!(),
        atom::GROUP_LEADER => unimplemented!(),
        atom::REDUCTIONS => Term::uint(heap, process.context().reds as u32),
        atom::PRIORITY => {
            // TODO: temporary
            atom!(NORMAL)
        },
        atom::TRACE => unimplemented!(),
        atom::BINARY => unimplemented!(),
        atom::SEQUENTIAL_TRACE_TOKEN => unimplemented!(),
        atom::CATCH_LEVEL => unimplemented!(),
        atom::BACKTRACE => unimplemented!(),
        atom::LAST_CALLS => unimplemented!(),
        atom::TOTAL_HEAP_SIZE => unimplemented!(),
        atom::SUSPENDING => unimplemented!(),
        atom::MIN_HEAP_SIZE => unimplemented!(),
        atom::MIN_BIN_VHEAP_SIZE => unimplemented!(),
        atom::MAX_HEAP_SIZE => unimplemented!(),
        atom::MAGIC_REF => unimplemented!(),
        atom::FULLSWEEP_AFTER => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    Ok(tup2!(heap, Term::atom(item), res))
}

pub fn process_info_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // args are pid, `[item, .. ]` or just `item`.
    // response is `[tup,..]` or just `tup`
    if !args[0].is_pid() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let pid = args[0].to_pid().unwrap();

    // TODO optimize for if process.pid == pid
    let proc = {
        let table = vm.process_table.lock();
        table.get(pid)
    };

    if let Some(proc) = proc {
        match Cons::cast_from(&args[1]) {
            Ok(cons) => {
                let heap = &process.context_mut().heap;
                cons.iter()
                    .map(|val| process_info_aux(vm, &proc, *val, true))
                    .fold_results(Term::nil(), |acc, val| cons!(heap, val, acc))
            }
            _ => process_info_aux(vm, &proc, args[1], false),
        }
    } else {
        return Ok(atom!(UNDEFINED));
    }
}

pub fn fun_info_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    if let Ok(closure) = value::Closure::cast_from(&args[0]) {
        match args[1].into_variant() {
            Variant::Atom(atom::TYPE) => Ok(atom!(LOCAL)),
            Variant::Atom(atom::PID) => unimplemented!(),
            Variant::Atom(atom::MODULE) => Ok(Term::atom(closure.mfa.0)),
            Variant::Atom(atom::NEW_INDEX) => unimplemented!(),
            Variant::Atom(atom::NEW_UNIQ) => unimplemented!(),
            Variant::Atom(atom::INDEX) => unimplemented!(),
            Variant::Atom(atom::UNIQ) => unimplemented!(),
            Variant::Atom(atom::ENV) => {
                if let Some(terms) = &closure.binding {
                    Ok(Cons::from_iter(terms.iter().copied(), heap))
                } else {
                    Ok(Term::nil())
                }
            }
            Variant::Atom(atom::REFC) => unimplemented!(),
            Variant::Atom(atom::ARITY) => Ok(Term::atom(closure.mfa.2)),
            Variant::Atom(atom::NAME) => unimplemented!(),
            _ => unimplemented!(),
        }
    } else if let Ok(mfa) = crate::module::MFA::cast_from(&args[0]) {
        match args[1].into_variant() {
            Variant::Atom(atom::TYPE) => Ok(atom!(EXTERNAL)),
            Variant::Atom(atom::PID) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::MODULE) => Ok(Term::atom(mfa.0)),
            Variant::Atom(atom::NEW_INDEX) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::NEW_UNIQ) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::INDEX) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::UNIQ) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::ENV) => Ok(Term::nil()),
            Variant::Atom(atom::REFC) => Ok(atom!(UNDEFINED)),
            Variant::Atom(atom::ARITY) => Ok(Term::uint(heap, mfa.2)),
            Variant::Atom(atom::NAME) => Ok(Term::atom(mfa.1)),
            _ => unimplemented!(),
        }
    } else {
        Err(Exception::new(Reason::EXC_BADARG))
    }
}

#[cfg(target_family = "unix")]
const OS_FAMILY: u32 = atom::UNIX;

#[cfg(target_family = "windows")]
const OS_FAMILY: u32 = atom::WIN32;

pub fn system_info_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    use std::sync::atomic::Ordering;
    let heap = &process.context_mut().heap;

    match args[0].into_variant() {
        Variant::Atom(atom::OS_TYPE) => Ok(tup2!(heap, Term::atom(OS_FAMILY), atom!(TRUE))), // TODO: true should be :darwin
        Variant::Atom(atom::HIPE_ARCHITECTURE) => Ok(atom!(UNDEFINED)),
        Variant::Atom(atom::SYSTEM_VERSION) => Ok(bitstring!(heap, "Erlang/OTP 22 [erts-10.3.4] [source] [64-bit] [smp:8:8] [ds:8:8:10] [async-threads:1] [enigma]\n")),
        Variant::Atom(atom::SYSTEM_LOGGER) => {
            Ok(Term::pid(vm.system_logger.load(Ordering::Relaxed) as u32)) // TODO: unsafe
        }
        Variant::Atom(atom::VERSION) => {
            Ok(bitstring!(heap, "10.3.4"))
        }
        Variant::Atom(atom::MACHINE) => {
            Ok(bitstring!(heap, "ENIGMA")) // maybe needs to be BEAM
        }
        Variant::Atom(atom::OTP_RELEASE) => {
            Ok(bitstring!(heap, "22"))
        }
        // Variant::Atom(atom::START_TIME) => {
        //     Ok(Term::int(vm.start_time))
        // }
        // thread 'tokio-runtime-worker-7' panicked at 'not yet implemented: system_info for :start_time', src/bif/info.rs:174:14
        Variant::Pointer(..) => {
            if let Ok(tup) = value::Tuple::cast_from(&args[0]) {
               match tup[0].into_variant() {
                   Variant::Atom(atom::PURIFY) => return Err(Exception::new(Reason::EXC_BADARG)),
                   _ => unimplemented!("system_info for {}", args[0])
               }
            } else {
                unimplemented!("system_info for {}", args[0])
            }
        }
        _ => unimplemented!("system_info for {}", args[0]),
    }
}

pub fn system_flag_2(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    use std::sync::atomic::Ordering;
    match args[0].into_variant() {
        Variant::Atom(atom::SYSTEM_LOGGER) => {
            let pid = match args[1].into_variant() {
                Variant::Pid(pid) => pid,
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            };

            let old_pid = vm.system_logger.swap(pid as usize, Ordering::Relaxed);
            Ok(Term::pid(old_pid as u32)) // TODO: unsafe
        }
        _ => unimplemented!(),
    }
}

pub fn group_leader_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    Ok(Term::pid(process.local_data().group_leader))
}

pub fn group_leader_2(vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let pid = match args[0].to_pid() {
        Some(pid) => pid,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let _target = match args[1].to_pid() {
        Some(pid) => pid,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    // TODO optimize for if process.pid == pid
    let proc = {
        let table = vm.process_table.lock();
        table.get(pid)
    };

    if let Some(proc) = proc {
        // TODO: no locks, unsafe!
        proc.local_data_mut().group_leader = pid;
        Ok(atom!(TRUE))
    } else {
        Err(Exception::new(Reason::EXC_BADARG))
    }
}
