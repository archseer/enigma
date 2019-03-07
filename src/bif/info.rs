use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, TryFrom, Variant};
use crate::vm;
use crate::Itertools;

pub fn process_info_aux(
    _vm: &vm::Machine,
    process: &RcProcess,
    item: Term,
    always_wrap: bool,
) -> bif::Result {
    use crate::process::{self, Flag};
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
        atom::CURRENT_FUNCTION => unimplemented!(),
        atom::CURRENT_LOCATION => unimplemented!(),
        atom::CURRENT_STACKTRACE => unimplemented!(),
        atom::INITIAL_CALL => unimplemented!(),
        atom::STATUS => unimplemented!(),
        atom::MESSAGES => unimplemented!(),
        atom::MESSAGE_QUEUE_LEN => {
            Term::uint(heap, local_data.mailbox.len() as u32)
        },
        atom::MESSAGE_QUEUE_DATA => unimplemented!(),
        atom::LINKS => unimplemented!(),
        atom::MONITORED_BY => unimplemented!(),
        atom::DICTIONARY => {
            // TODO $ancestors ?
            let pdict = &process.local_data_mut().dictionary;
            let heap = &process.context_mut().heap;

            pdict.iter().fold(Term::nil(), |res, (key, val)| {
                // make tuple
                let tuple = tup2!(heap, *key, *val);

                // make cons
                cons!(heap, tuple, res)
            })
        },
        atom::TRAP_EXIT => {
            Term::boolean(local_data.flags.contains(Flag::TRAP_EXIT))
        },
        atom::ERROR_HANDLER => unimplemented!(),
        atom::HEAP_SIZE => unimplemented!(),
        atom::STACK_SIZE => unimplemented!(),
        atom::MEMORY => unimplemented!(),
        atom::GARBAGE_COLLECTION => unimplemented!(),
        atom::GARBAGE_COLLECTION_INFO => unimplemented!(),
        atom::GROUP_LEADER => unimplemented!(),
        atom::REDUCTIONS => unimplemented!(),
        atom::PRIORITY => unimplemented!(),
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

    let pid = args[0].to_u32();

    // TODO optimize for if process.pid == pid
    let proc = {
        let table = vm.state.process_table.lock();
        table.get(pid)
    };

    if let Some(proc) = proc {
        match Cons::try_from(&args[1]) {
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

#[cfg(target_family = "unix")]
const OS_FAMILY: u32 = atom::UNIX;

#[cfg(target_family = "windows")]
const OS_FAMILY: u32 = atom::WIN32;

pub fn system_info_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;

    match args[0].into_variant() {
        Variant::Atom(atom::OS_TYPE) => Ok(tup2!(heap, atom!(OS_TYPE), Term::atom(OS_FAMILY))),
        _ => unimplemented!("system_info for {}", args[0])
    }
}
