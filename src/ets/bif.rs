use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value;
use crate::value::{Cons, Term, TryFrom, Tuple, Type, Variant};
use crate::vm;
use crate::Itertools;

use super::error::{new_error, ErrorKind};
use super::hash_table::HashTable;
use super::*;
use super::{pam, Status};

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
    // get DB_WRITE, lock kind, ets_insert_2
    match term.get_type() {
        // TODO: inefficient
        Type::Atom => {
            let key = term.to_u32();
            let lock = vm.ets_tables.lock();
            lock.get_named(key as usize)
        }
        Type::Ref => {
            let key = term.to_ref().unwrap(); // TODO: HANDLE Atom
            let lock = vm.ets_tables.lock();
            lock.get(key)
        }
        _ => None,
    }
    .ok_or_else(|| Exception::new(Reason::EXC_BADARG))
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

pub fn lookup_element_3(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let table = get_table(vm, args[0])?;

    let index = match args[2].into_number() {
        Ok(value::Num::Integer(i)) if i > 0 => (i - 1) as usize,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    // for some reason just returning won't work
    Ok(table.get_element(process, args[1], index)?)
}

/// Deletes an entire table.
pub fn delete_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let table = get_table(vm, args[0])?;

    // TODO: set access bits to none to disable access

    // TODO: transfer ownership to current process just in case

    {
        // remove table from index
        let mut tables = vm.ets_tables.lock();
        tables.remove(&table);
    }

    // TODO: bump reds

    Ok(atom!(TRUE))
}

pub fn select_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    unimplemented!()
}

// Check if object represents a "match" variable i.e and atom $N where N is an integer.

struct MpInfo {
    /// The match_spec is not "impossible"
    something_can_match: bool,
    key_given: bool,

    /// Default list of "pre-found" buckets
    // wmp_prefound: dlists[10],
    /// Buckets to search if keys are given, = dlists initially
    // struct mp_prefound* lists,
    /// Number of elements in "lists", = 0 initially
    // num_lists: usize,

    ///  The compiled match program
    mp: pam::Pattern,
}

/// For the select functions, analyzes the pattern and determines which
/// slots should be searched. Also compiles the match program
fn analyze_pattern(
    table: &RcTable,
    pattern: Term, /* extra_validator: Fn optional callback */
) -> Result<MpInfo> {
    // Eterm *ptpl;
    // Eterm sbuff[30];
    // Eterm *buff = sbuff;
    // Eterm key = NIL;
    // HashValue hval = NIL;

    let pattern = Cons::try_from(&pattern)?;

    let lst = pattern.iter();
    let num_heads = lst.count();

    if !lst.is_nil() {
        // proper list...
        return Err(new_error(ErrorKind::BadParameter));
    }

    // let lists = Vec::with_capacity(num_heads);

    let mut mpi = MpInfo {
        // lists,
        // lists: mpi.dlists,
        // num_lists: 0,
        key_given: true,
        something_can_match: false,
        mp: std::mem::uninitialized(),
    };

    let matches: Vec<Term> = Vec::with_capacity(num_heads);
    let guards: Vec<Term> = Vec::with_capacity(num_heads);
    let bodies: Vec<Term> = Vec::with_capacity(num_heads);

    for tup in pattern {
        // Eterm match;
        // Eterm guard;
        // Eterm body;

        let ptpl = Tuple::try_from(&tup)?;
        if ptpl.len() != 3 {
            return Err(new_error(ErrorKind::BadParameter));
        }

        let tpl = ptpl[0];
        let body = ptpl[2];
        matches.push(ptpl[0]);
        guards.push(ptpl[1]);
        bodies.push(ptpl[2]);

        // if extra_validator != NULL && !extra_validator(tb->common.keypos, match, guard, body) {
        //    return Err(new_error(ErrorKind::BadParameter));
        // }

        if (!is_list(body)
            || CDR(list_val(body)) != NIL
            || CAR(list_val(body)) != atom!(DOLLAR_UNDERSCORE))
        {}

        if !mpi.key_given {
            continue;
        }

        if tpl == atom!(UNDERSCORE) || pam::is_variable(tpl).is_some() {
            mpi.key_given = false;
            mpi.something_can_match = true;
        } else {
            if let Some(key) = tpl.get(table.meta().keypos) {
                if !db_has_variable(key) {
                    // Bound key
                    // int ix, search_slot;
                    // HashDbTerm** bp;
                    // erts_rwmtx_t* lck;
                    hval = MAKE_HASH(key);
                    lck = RLOCK_HASH(tb, hval);
                    ix = hash_to_ix(tb, hval);
                    bp = &BUCKET(tb, ix);
                    if lck == NULL {
                        search_slot = search_list(tb, key, hval, *bp) != NULL;
                    } else {
                        /* No point to verify if key exist now as there may be
                        concurrent inserters/deleters anyway */
                        RUNLOCK_HASH(lck);
                        search_slot = true;
                    }

                    if search_slot {
                        // let j = 0;
                        // loop {
                        // if j == mpi->num_lists) {
                        // mpi->lists[mpi->num_lists].bucket = bp;
                        // mpi->lists[mpi->num_lists].ix = ix;
                        // ++mpi->num_lists;
                        // break;
                        // }
                        // if mpi->lists[j].bucket == bp {
                        // assert!(mpi->lists[j].ix == ix);
                        // break;
                        // }
                        // assert!(mpi->lists[j].ix != ix);

                        //     j += 1;
                        // }
                        mpi.something_can_match = true;
                    }
                } else {
                    mpi.key_given = false;
                    mpi.something_can_match = true;
                }
            }
        }
    }

    // It would be nice not to compile the match_spec if nothing could match,
    // but then the select calls would not fail like they should on bad
    // match specs that happen to specify non existent keys etc.

    let compiler = pam::Compiler::new(matches, guards, bodies, num_heads, DCOMP_TABLE, flags);
    mpi.mp = compiler.match_compile();
    if mpi.mp == NULL {
        //if buff != sbuff { erts_free(ERTS_ALC_T_DB_TMP, buff); }
        return Err(new_error(ErrorKind::BadParameter));
    }
    //if buff != sbuff { erts_free(ERTS_ALC_T_DB_TMP, buff); }

    Ok(mpi)
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
