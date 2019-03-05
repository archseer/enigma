use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{Cons, Term, TryFrom, Tuple, Variant};
use crate::vm;
use crate::Itertools;

use super::hash_table::HashTable;
use super::Status;
use super::*;

pub fn new_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    if !args[0].is_atom() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    // TODO: is_list already checks for nil? needs impl change maybe
    if !(args[1].is_nil() || args[1].is_list()) {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let heap = &process.context_mut().heap;

    let mut status = Status::DB_SET | Status::DB_PROTECTED;
    let mut keypos = 1;
    let mut is_named = false;
    let mut is_fine_locked = false;
    let mut frequent_read = false;
    let mut heir = atom!(NONE);
    let mut heir_data = atom!(UNDEFINED);
    // is_compressed = erts_ets_always_compress;
    let mut is_compressed = false;

    // TODO skip if args is nil
    let cons = Cons::try_from(&args[1])?;

    for val in cons.iter() {
        match val.into_variant() {
            Variant::Atom(atom::BAG) => {
                status.insert(Status::DB_BAG);
                status.remove(
                    Status::DB_SET
                        | Status::DB_DUPLICATE_BAG
                        | Status::DB_ORDERED_SET
                        | Status::DB_CA_ORDERED_SET,
                );
            }
            Variant::Atom(atom::DUPLICATE_BAG) => {
                status.insert(Status::DB_DUPLICATE_BAG);
                status.remove(
                    Status::DB_SET
                        | Status::DB_BAG
                        | Status::DB_ORDERED_SET
                        | Status::DB_CA_ORDERED_SET,
                );
            }
            Variant::Atom(atom::ORDERED_SET) => {
                status.insert(Status::DB_ORDERED_SET);
                status.remove(
                    Status::DB_SET
                        | Status::DB_DUPLICATE_BAG
                        | Status::DB_SET
                        | Status::DB_CA_ORDERED_SET,
                );
            }
            Variant::Pointer(_ptr) => {
                let tup = Tuple::try_from(val)?;
                if tup.len() == 2 {
                    match tup[0].into_variant() {
                        Variant::Atom(atom::KEYPOS) => {
                            match tup[1].to_int() {
                                Some(i) if i > 0 => keypos = i as usize,
                                _ => return Err(Exception::new(Reason::EXC_BADARG)),
                            };
                        }
                        Variant::Atom(atom::WRITE_CONCURRENCY) => {
                            match tup[1].to_bool() {
                                Some(val) => is_fine_locked = val,
                                None => return Err(Exception::new(Reason::EXC_BADARG)),
                            };
                        }
                        Variant::Atom(atom::READ_CONCURRENCY) => {
                            match tup[1].to_bool() {
                                Some(val) => frequent_read = val,
                                None => return Err(Exception::new(Reason::EXC_BADARG)),
                            };
                        }
                        Variant::Atom(atom::HEIR) => {
                            if tup[1] == atom!(NONE) {
                                heir = atom!(NONE);
                                heir_data = atom!(UNDEFINED);
                            } else {
                                return Err(Exception::new(Reason::EXC_BADARG));
                            }
                        }
                        _ => return Err(Exception::new(Reason::EXC_BADARG)),
                    }
                } else if tup.len() == 3 {
                    //&& tup[0] == am_heir && is_internal_pid(tp[2]) {
                    unimplemented!()
                //     heir = tp[2];
                //     heir_data = tp[3];
                } else {
                    return Err(Exception::new(Reason::EXC_BADARG));
                }
            }
            Variant::Atom(atom::PUBLIC) => {
                status.insert(Status::DB_PUBLIC);
                status.remove(Status::DB_PROTECTED | Status::DB_PRIVATE);
            }
            Variant::Atom(atom::PRIVATE) => {
                status.insert(Status::DB_PRIVATE);
                status.remove(Status::DB_PROTECTED | Status::DB_PUBLIC);
            }
            Variant::Atom(atom::NAMED_TABLE) => {
                is_named = true;
                status |= Status::DB_NAMED_TABLE;
            }
            Variant::Atom(atom::COMPRESSED) => {
                is_compressed = true;
            }
            Variant::Atom(atom::SET) | Variant::Atom(atom::PROTECTED) => {}
            _ => return Err(Exception::new(Reason::EXC_BADARG)),
        }
    }

    // if !list.is_nil() { // bad opt or not a well formed list
    //     return Err(Exception::new(Reason::EXC_BADARG));
    // }

    /*if IS_TREE_TABLE(status) && is_fine_locked && !status.contains(Status::DB_PRIVATE) {
        status.insert(Status::DB_CA_ORDERED_SET);
        status.remove(Status::DB_SET | Status::DB_BAG | Status::DB_DUPLICATE_BAG | Status::DB_ORDERED_SET);
        status.insert(Status::DB_FINE_LOCKED);
    } */

    // if is_hash_table
    if let Status::DB_SET | Status::DB_BAG | Status::DB_DUPLICATE_BAG = table_kind!(status) {
        if is_fine_locked && !status.contains(Status::DB_PRIVATE) {
            status.insert(Status::DB_FINE_LOCKED);
        }
    }

    if frequent_read && !status.contains(Status::DB_PRIVATE) {
        status |= Status::DB_FREQ_READ;
    }

    // make_btid(tb);
    let tid = vm.state.next_ref();

    // meth: methods
    let meta = Metadata {
        tid,
        name: Some(args[0].to_u32() as usize), // unsound conversion
        status,
        kind: status, // Note, 'kind' is *read only* from now on...
        keypos,
        owner: process.pid,
        compress: is_compressed,
        // init fixing to count 0 and procs NULL
    };
    // erts_refc_init(&tb->common.fix_count, 0);
    // db_init_lock(tb, status & (DB_FINE_LOCKED|DB_FREQ_READ));
    // set_heir(BIF_P, tb, heir, heir_data);
    // erts_atomic_init_nob(&tb->common.nitems, 0);

    // #ifdef ETS_DBG_FORCE_TRAP
    //     erts_atomic_init_nob(&tb->common.dbg_force_trap, erts_ets_dbg_force_trap);
    // #endif

    // had an assert here before, hence unwrap
    let table = match table_kind!(status) {
        Status::DB_SET | Status::DB_BAG | Status::DB_DUPLICATE_BAG => {
            Arc::new(HashTable::new(meta, process))
        }
        Status::DB_ORDERED_SET => unimplemented!(), // SetTable::new
        Status::DB_CA_ORDERED_SET => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    {
        // TODO: need clone since insert_named might run, not ideal
        vm.ets_tables.lock().insert(tid, table.clone());
    }
    // process.save_sched_table(tabletb);
    // process.save_owned_table(table);

    if is_named {
        if vm
            .ets_tables
            .lock()
            .insert_named(args[0].to_u32() as usize, table)
        {
            return Ok(args[0]);
        }
        // table drops

        // tid_clear(BIF_P, tb);
        // delete_owned_table(BIF_P, tb);

        // db_lock(tb,LCK_WRITE);
        // free_heir_data(tb);
        // tb->common.meth->db_free_empty_table(tb);
        // db_unlock(tb,LCK_WRITE);
        // table_dec_refc(tb, 0);
        // BIF_ERROR(BIF_P, BADARG);

        Err(Exception::new(Reason::EXC_BADARG))
    } else {
        let reference = vm.state.next_ref();
        Ok(Term::reference(heap, reference))
    }

    // BIF_P->flags |= F_USING_DB; /* So we can remove tb if p dies */
    // #ifdef HARDDEBUG
    //     erts_fprintf(stderr,
    // 		"ets:new(%T,%T)=%T; Process: %T, initial: %T:%T/%bpu\n",
    // 		 BIF_ARG_1, BIF_ARG_2, ret, BIF_P->common.id,
    // 		 BIF_P->u.initial[0], BIF_P->u.initial[1], BIF_P->u.initial[2]);
    // #endif
}
pub fn whereis_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // atom
    let name = match args[0].into_variant() {
        Variant::Atom(name) => name,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let tid = vm
        .ets_tables
        .lock()
        .whereis(name as usize)
        .map(|tid| Term::reference(&process.context_mut().heap, tid));

    match tid {
        Some(tid) => Ok(tid),
        None => Ok(atom!(UNDEFINED)),
    }
}

#[inline]
fn get_table(vm: &vm::Machine, term: Term) -> std::result::Result<RcTable, Exception> {
    /*let key = match args[0].into_variant() {
        Variant::Atom(name) => name as usize,
        _ => unimplemented!()
    };*/
    let key = term.to_ref().unwrap(); // TODO: HANDLE Atom

    // get DB_WRITE, lock kind, ets_insert_2
    let lock = vm.ets_tables.lock();
    lock.get(key).ok_or_else(|| Exception::new(Reason::EXC_BADARG))
    // TODO: get_named for atom
}

pub fn insert_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    /* Write lock table if more than one object to keep atomicity */
    // let lock_kind = if (is_list(BIF_ARG_2) && CDR(list_val(BIF_ARG_2)) != NIL { LCK_WRITE } else { LCK_WRITE_REC };

    eprintln!("going in {}", args[0]);

    // find table
    let table = get_table(vm, args[0])?;

    if args[1].is_nil() {
        return Ok(atom!(TRUE));
    }

    let keypos = table.meta().keypos;

    let validate = |val: &Term| val.is_tuple() && Tuple::try_from(val).unwrap().len() > keypos;

    let res: Result<()> = if let Ok(cons) = Cons::try_from(&args[1]) {
        let valid = cons.iter().all(validate);
        // TODO if bad list
        // if (lst != NIL) { goto badarg; }
        if !valid {
            return Err(Exception::new(Reason::EXC_BADARG));
        }

        cons.iter()
            .map(|val| table.insert(process, *val, false))
            .collect::<Result<Vec<()>>>() // this is not efficient at all
            .map(|_| ())
    } else {
        // single param
        if !validate(&args[1]) {
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        table.insert(process, args[1], false)
    };

    match res {
        Ok(_) => Ok(atom!(TRUE)),
        // TODO use From ets::Error
        // TODO ERROR_SYSRES on SYSTEM_LIMIT
        Err(_) => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn lookup_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let table = get_table(vm, args[0])?;

    // for some reason just returning won't work
    Ok(table.get(process, args[1])?)
}

// safe_fixtable_2
// first_1
// next_2
// last_1
// prev_2
// take_2
// update_element_3
// update_counter_3
// update_counter_4
// insert_new_2
// rename_2
// lookup_2
// member_2
// lookup_element_3
// delete_1
// give_away_3
// setopts_2
// internal_delete_all_2
// delete_2
// delete_object_2
// select_delete_2
// internal_select_delete_2
// internal_request_all_0
// slot_2
// match_1
// match_2
// match_3
// select_3
// select_1
// select_2
// select_count_1
// select_count_2
// select_replace_1
// select_replace_2
// select_reverse_3
// select_reverse_1
// select_reverse_2
// match_object_1
// match_object_2
// match_object_3
// info_1
// info_2
// is_compiled_ms_1
// match_spec_compile_1
// match_spec_run_r_3
