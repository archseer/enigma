use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value;
use crate::value::{Cons, Term, TryFrom, Tuple, Type, Variant};
use crate::vm;
use crate::Itertools;

use super::hash_table::HashTable;
use super::Status;
use super::*;
use super::error::{ErrorKind, new_error};

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

fn is_variable(obj: Term) -> Option<usize> {
    // byte *b;
    // int n;
    // int N;
    match obj.into_variant() {
        // TODO original checked for < 2 as error but we use nil, true, false as 0,1,2
        Variant::Atom(i) if i > 2 => {
            crate::atom::to_str(i)
                .ok()
                .map(|v| v.as_bytes())
                .and_then(|name| {
                    if name[0] == '$' as u8 {
                        lexical::try_parse::<usize, _>(&name[1..]).ok()
                    } else { None }
                })
        }
        _ => None
    }
}

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
    mp: Vec<u8>
}


/// For the select functions, analyzes the pattern and determines which
/// slots should be searched. Also compiles the match program
fn analyze_pattern(table: &RcTable, pattern: Term, /* extra_validator: Fn optional callback */) -> Result<MpInfo> {
    // Eterm *ptpl;
    // Eterm sbuff[30];
    // Eterm *buff = sbuff;
    // Eterm key = NIL;
    // HashValue hval = NIL;

    let pattern = Cons::try_from(&pattern)?;

    let lst = pattern.iter();
    let num_heads = lst.count();

    if !lst.is_nil() {// proper list...
	return Err(new_error(ErrorKind::BadParameter));
    }

    // let lists = Vec::with_capacity(num_heads);

    let mut mpi = MpInfo {
        // lists,
        // lists: mpi.dlists,
        // num_lists: 0,
        key_given: true,
        something_can_match: false,
        mp: Vec::new(),
    };

    let matches: Vec<Term> = Vec::with_capacity(num_heads);
    let guards: Vec<Term> = Vec::with_capacity(num_heads);
    let bodies: Vec<Term> = Vec::with_capacity(num_heads);

    for tup in pattern {
        // Eterm match;
        // Eterm guard;
        // Eterm body;

        let ptpl = Tuple::try_from(&tup)?;
	if ptpl.len() != 3  {
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

	if (!is_list(body) || CDR(list_val(body)) != NIL ||
	    CAR(list_val(body)) != atom!(DOLLAR_UNDERSCORE)) {
	}

	if !mpi.key_given {
	    continue;
	}

	if tpl == atom!(UNDERSCORE) || is_variable(tpl).is_some() {
	    mpi.key_given = false;
	    mpi.something_can_match = true;
	} else {
            if let Some(key) = tpl.get(table.meta().keypos) {
		if !db_has_variable(key) { // Bound key
		    int ix, search_slot;
		    HashDbTerm** bp;
		    erts_rwmtx_t* lck;
		    hval = MAKE_HASH(key);
		    lck = RLOCK_HASH(tb,hval);
		    ix = hash_to_ix(tb, hval);
		    bp = &BUCKET(tb,ix);
		    if lck == NULL {
			search_slot = search_list(tb,key,hval,*bp) != NULL;
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
    mpi.mp = match_compile(matches, guards, bodies, num_heads, DCOMP_TABLE, None);
    if mpi.mp == NULL {
	//if buff != sbuff { erts_free(ERTS_ALC_T_DB_TMP, buff); }
	return Err(new_error(ErrorKind::BadParameter));
    }
    //if buff != sbuff { erts_free(ERTS_ALC_T_DB_TMP, buff); }

    Ok(mpi)
}

/// The actual compiling of the match expression and the guards.
fn match_compile(matchexpr: Vec<Term>, guards: Vec<Term>, body: Vec<Term>, num_progs: usize, flags: usize) -> Result<Vec<u8>, Error> {
    // DMCHeap heap;
    // DMC_STACK_TYPE(Eterm) stack;
    // DMC_STACK_TYPE(UWord) text;
    // DMCContext context;
    // MatchProg *ret = NULL;
    // Eterm t;
    // Uint i;
    // Uint num_iters;
    // int structure_checked;
    // DMCRet res;
    let current_try_label = -1;
    // Binary *bp = NULL;
    // unsigned clause_start;

    // DMC_INIT_STACK(stack);
    let stack = Vec::new();
    // DMC_INIT_STACK(text);
    let text = Vec::new();

    context.stack_need = context.stack_used = 0;
    context.save = context.copy = NULL;
    context.num_match = num_progs;
    context.matchexpr = matchexpr;
    context.guardexpr = guards;
    context.bodyexpr = body;
    context.err_info = err_info;
    context.cflags = flags;

    heap.size = DMC_DEFAULT_SIZE;
    heap.vars = heap.vars_def;

    // Compile the match expression.
    heap.vars_used = 0;

    for i in 0..num_progs { // long loop ahead
	// sys_memset(heap.vars, 0, heap.size * sizeof(*heap.vars));
        context.current_match = i;
        let mut t = context.matchexpr[current_match];
	context.stack_used = 0;
	let structure_checked = false;

	if context.current_match < num_progs - 1 {
            text.push(Opcode::MatchTryMeElse)
	    current_try_label = text.len() - 1;
            text.push(0);
	} else {
	    current_try_label = -1;
	}

	clause_start = text.len() - 1; // the "special" test needs it
        loop {
            match t.into_variant() {
                Variant::Pointer(..) => {
                    match self.get_boxed_header().unwrap() {
                        // if (is_flatmap(t)) {
                        //     num_iters = flatmap_get_size(flatmap_val(t));
                        //     if (!structure_checked) {
                        //         DMC_PUSH2(text, matchMap, num_iters);
                        //     }
                        //     structure_checked = 0;
                        //     for (i = 0; i < num_iters; ++i) {
                        //         Eterm key = flatmap_get_keys(flatmap_val(t))[i];
                        //         if (db_is_variable(key) >= 0) {
                        //             if (context.err_info) {
                        //                 add_dmc_err(context.err_info,
                        //                             "Variable found in map key.",
                        //                             -1, 0UL, dmcError);
                        //             }
                        //             goto error;
                        //         } else if (key == am_Underscore) {
                        //             if (context.err_info) {
                        //                 add_dmc_err(context.err_info,
                        //                             "Underscore found in map key.",
                        //                             -1, 0UL, dmcError);
                        //             }
                        //             goto error;
                        //         }
                        //         DMC_PUSH2(text, matchKey, dmc_private_copy(&context, key));
                        //         {
                        //             int old_stack = ++(context.stack_used);
                        //             Eterm value = flatmap_get_values(flatmap_val(t))[i];
                        //             res = dmc_one_term(&context, &heap, &stack, &text,
                        //                                value);
                        //             ASSERT(res != retFail);
                        //             if (old_stack != context.stack_used) {
                        //                 ASSERT(old_stack + 1 == context.stack_used);
                        //                 DMC_PUSH(text, matchSwap);
                        //             }
                        //             if (context.stack_used > context.stack_need) {
                        //                 context.stack_need = context.stack_used;
                        //             }
                        //             DMC_PUSH(text, matchPop);
                        //             --(context.stack_used);
                        //         }
                        //     }
                        //     break;
                        // }
                        BOXED_MAP => {
                            DECLARE_WSTACK(wstack);
                            Eterm *kv;
                            num_iters = hashmap_size(t);
                            if (!structure_checked) {
                                DMC_PUSH2(text, matchMap, num_iters);
                            }
                            structure_checked = 0;

                            hashmap_iterator_init(&wstack, t, 0);

                            while ((kv=hashmap_iterator_next(&wstack)) != NULL) {
                                Eterm key = CAR(kv);
                                Eterm value = CDR(kv);
                                if (db_is_variable(key) >= 0) {
                                    if (context.err_info) {
                                        add_dmc_err(context.err_info,
                                                "Variable found in map key.",
                                                -1, 0UL, dmcError);
                                    }
                                    DESTROY_WSTACK(wstack);
                                    goto error;
                                } else if (key == am_Underscore) {
                                    if (context.err_info) {
                                        add_dmc_err(context.err_info,
                                                "Underscore found in map key.",
                                                -1, 0UL, dmcError);
                                    }
                                    DESTROY_WSTACK(wstack);
                                    goto error;
                                }
                                DMC_PUSH2(text, matchKey, dmc_private_copy(&context, key));
                                {
                                    int old_stack = ++(context.stack_used);
                                    res = dmc_one_term(&context, &heap, &stack, &text,
                                                    value);
                                    ASSERT(res != retFail);
                                    if (old_stack != context.stack_used) {
                                        ASSERT(old_stack + 1 == context.stack_used);
                                        DMC_PUSH(text, matchSwap);
                                    }
                                    if (context.stack_used > context.stack_need) {
                                        context.stack_need = context.stack_used;
                                    }
                                    DMC_PUSH(text, matchPop);
                                    --(context.stack_used);
                                }
                            }
                            DESTROY_WSTACK(wstack);
                            break;
                        }
                        BOXED_TUPLE => {
                            num_iters = arityval(*tuple_val(t));
                            if !structure_checked { // i.e. we did not pop it
                                text.push(Opcode::MatchTuple(t.len()))
                            }
                            structure_checked = false;
                            for val in t {
                                if (res = dmc_one_term(&context, &heap, &stack, &text, val)) != retOk {
                                    goto error;
                                }
                            }
                            break;
                        }
                        _ => {
                            goto simple_term;
                        }
                }
                Variant::Cons(..) =>  {
                    if !structure_checked {
                        text.push(Opcode::MatchList);
                    }
                    structure_checked = false; // Whatever it is, we did not pop it
                    if (res = dmc_one_term(&context, &heap, &stack, &text, CAR(list_val(t)))) != retOk {
                        goto error;
                    }
                    t = CDR(list_val(t));
                    continue;
                }
                _ =>  {// Nil and non proper tail end's or single terms as match expressions.
                //simple_term:
                    structure_checked = false;
                    if (res = dmc_one_term(&context, &heap, &stack, &text, t)) != retOk {
                        goto error;
                    }
                    break;
                }
	    }

            // The *program's* stack just *grows* while we are traversing one composite data
            // structure, we can check the stack usage here

	    // if (context.stack_used > context.stack_need)
		// context.stack_need = context.stack_used;

            // We are at the end of one composite data structure, pop sub structures and emit
            // a matchPop instruction (or break)
            if Some(val) = stack.pop() {
                t = val;
                text.push(Opcode::MatchPop);
		structure_checked = true; // Checked with matchPushT or matchPushL
		--(context.stack_used);
	    } else {
		break;
	    }
	} // end type loop

	// There is one single top variable in the match expression
	// if the text is two Uint's and the single instruction
	// is 'matchBind' or it is only a skip.
	context.special =
	    ((text.len() - 1) == 2 + clause_start &&
	     DMC_PEEK(text,clause_start) == matchBind) ||
	    ((text.len() - 1) == 1 + clause_start &&
	     DMC_PEEK(text, clause_start) == matchSkip);

	if flags & DCOMP_TRACE {
	    if (context.special) {
		if (DMC_PEEK(text, clause_start) == matchBind) {
		    DMC_POKE(text, clause_start, matchArrayBind);
		}
	    } else {
		ASSERT(DMC_STACK_NUM(text) >= 1);
		if (DMC_PEEK(text, clause_start) != matchTuple) {
		    /* If it isn't "special" and the argument is
		       not a tuple, the expression is not valid
		       when matching an array*/
		    if (context.err_info) {
			add_dmc_err(context.err_info,
				    "Match head is invalid in "
				    "this context.",
				    -1, 0UL,
				    dmcError);
		    }
		    goto error;
		}
		DMC_POKE(text, clause_start, matchArray);
	    }
	}


	// ... and the guards
	context.is_guard = true;
	if compile_guard_expr(&context, &heap, &text, context.guardexpr[context.current_match]) != retOk {
	    goto error;
        }
	context.is_guard = false;
	if ((context.cflags & DCOMP_TABLE) &&
	    !is_list(context.bodyexpr[context.current_match])) {
	    if (context.err_info) {
		add_dmc_err(context.err_info,
			    "Body clause does not return "
			    "anything.", -1, 0UL,
			    dmcError);
	    }
	    goto error;
	}
	if (compile_guard_expr(&context, &heap, &text, context.bodyexpr[context.current_match]) != retOk) {
	    goto error;
        }

        // The compilation does not bail out when error information is requested, so we need to
        // detect that here...
	if context.err_info != NULL && context.err_info.error_added {
	    goto error;
	}


	// If the matchprogram comes here, the match is successful
        text.push(Opcode::MatchHalt);
	// Fill in try-me-else label if there is one.
	if current_try_label >= 0 {
	    DMC_POKE(text, current_try_label, DMC_STACK_NUM(text));
	}
    } /* for (context.current_match = 0 ...) */


    /*
    ** Done compiling
    ** Allocate enough space for the program,
    ** heap size is in 'heap_used', stack size is in 'stack_need'
    ** and text size is simply DMC_STACK_NUM(text).
    ** The "program memory" is allocated like this:
    ** text ----> +-------------+
    **            |             |
    **              ..........
    **            +-------------+
    **
    **  The heap-eheap-stack block of a MatchProg is nowadays allocated
    **  when the match program is run (see db_prog_match()).
    **
    ** heap ----> +-------------+
    **              ..........
    ** eheap ---> +             +
    **              ..........
    ** stack ---> +             +
    **              ..........
    **            +-------------+
    ** The stack is expected to grow towards *higher* adresses.
    ** A special case is when the match expression is a single binding
    ** (i.e '$1'), then the field single_variable is set to 1.
    */
    bp = erts_create_magic_binary(((sizeof(MatchProg) - sizeof(UWord)) +
				   (DMC_STACK_NUM(text) * sizeof(UWord))),
				  erts_db_match_prog_destructor);
    ret = Binary2MatchProg(bp);
    ret->saved_program_buf = NULL;
    ret->saved_program = NIL;
    ret->term_save = context.save;
    ret->num_bindings = heap.vars_used;
    ret->single_variable = context.special;
    sys_memcpy(ret->text, DMC_STACK_DATA(text),
	       DMC_STACK_NUM(text) * sizeof(UWord));
    ret->stack_offset = heap.vars_used*sizeof(MatchVariable) + FENCE_PATTERN_SIZE;
    ret->heap_size = ret->stack_offset + context.stack_need * sizeof(Eterm*) + FENCE_PATTERN_SIZE;

#ifdef DMC_DEBUG
    ret->prog_end = ret->text + DMC_STACK_NUM(text);
#endif

    // Fall through to cleanup code, but context.save should not be free'd
    context.save = NULL;
error: // Here is were we land when compilation failed.
    if (context.save != NULL) {
	free_message_buffer(context.save);
	context.save = NULL;
    }
    DMC_FREE(stack);
    DMC_FREE(text);
    if (context.copy != NULL)
	free_message_buffer(context.copy);
    if (heap.vars != heap.vars_def)
	erts_free(ERTS_ALC_T_DB_MS_CMPL_HEAP, (void *) heap.vars);
    return bp;
}

/// Handle one term in the match expression (not the guard)
#[inline]
static fn dmc_one_term(DMCContext *context,
			   DMCHeap *heap,
			   DMC_STACK_TYPE(Eterm) *stack,
			   DMC_STACK_TYPE(UWord) *text,
			   Eterm c) -> DMCRet {
    // Sint n;
    // Eterm *hp;
    // Uint sz, sz2, sz3;
    // Uint i, j;

    switch (c & _TAG_PRIMARY_MASK) {
        case TAG_PRIMARY_IMMED1:
            if ((n = db_is_variable(c)) >= 0) { /* variable */
                // TODO: this branch is for realloc but we use vec so we're fine, hence no restarts
                // if n >= heap->size {
                //     // Ouch, big integer in match variable.
                //     Eterm *save_hp;
                //     ASSERT(heap->vars == heap->vars_def);
                //     sz = sz2 = sz3 = 0;
                //     for (j = 0; j < context->num_match; ++j) {
                //         sz += size_object(context->matchexpr[j]);
                //         sz2 += size_object(context->guardexpr[j]);
                //         sz3 += size_object(context->bodyexpr[j]);
                //     }
                //     context->copy =
                //         new_message_buffer(sz + sz2 + sz3 +
                //                         context->num_match);
                //     save_hp = hp = context->copy->mem;
                //     hp += context->num_match;
                //     for (j = 0; j < context->num_match; ++j) {
                //         context->matchexpr[j] =
                //             copy_struct(context->matchexpr[j],
                //                         size_object(context->matchexpr[j]), &hp,
                //                         &(context->copy->off_heap));
                //         context->guardexpr[j] =
                //             copy_struct(context->guardexpr[j],
                //                         size_object(context->guardexpr[j]), &hp,
                //                         &(context->copy->off_heap));
                //         context->bodyexpr[j] =
                //             copy_struct(context->bodyexpr[j],
                //                         size_object(context->bodyexpr[j]), &hp,
                //                         &(context->copy->off_heap));
                //     }
                //     for (j = 0; j < context->num_match; ++j) {
                //         /* the actual expressions can be
                //         atoms in their selves, place them first */
                //         *save_hp++ = context->matchexpr[j];
                //     }
                //     heap->size = match_compact(context->copy,
                //                             context->err_info);
                //     for (j = 0; j < context->num_match; ++j) {
                //         /* restore the match terms, as they
                //         may be atoms that changed */
                //         context->matchexpr[j] = context->copy->mem[j];
                //     }
                //     heap->vars = erts_alloc(ERTS_ALC_T_DB_MS_CMPL_HEAP,
                //                             heap->size*sizeof(DMCVariable));
                //     sys_memset(heap->vars, 0, heap->size * sizeof(DMCVariable));
                //     DMC_CLEAR(*stack);
                //     /*DMC_PUSH(*stack,NIL);*/
                //     DMC_CLEAR(*text);
                //     return retRestart;
                // }

                if heap->vars[n].is_bound {
                    DMC_PUSH2(*text, matchCmp, n);
                } else { /* Not bound, bind! */
                    if n >= heap->vars_used {
                        heap->vars_used = n + 1;
                    }
                    DMC_PUSH2(*text, matchBind, n);
                    heap->vars[n].is_bound = true;
                }
            } else if c == atom!(UNDERSCORE) {
                text.push(Opcode::MatchSkip);
            } else {
                // Any immediate value
                text.push(Opcode::MatchEq(c as usize));
            }
            break;
        case TAG_PRIMARY_LIST:
            DMC_PUSH(*text, matchPushL);
            ++(context->stack_used);
            DMC_PUSH(*stack, c);
            break;
        case TAG_HEADER_FLOAT:
            DMC_PUSH2(*text, matchEqFloat, (Uint) float_val(c)[1]);
    #ifdef ARCH_64
            DMC_PUSH(*text, (Uint) 0);
    #else
            DMC_PUSH(*text, (Uint) float_val(c)[2]);
    #endif
            break;
        case TAG_PRIMARY_BOXED: {
            Eterm hdr = *boxed_val(c);
            switch ((hdr & _TAG_HEADER_MASK) >> _TAG_PRIMARY_SIZE) {
            case (_TAG_HEADER_ARITYVAL >> _TAG_PRIMARY_SIZE):
                n = arityval(*tuple_val(c));
                DMC_PUSH2(*text, matchPushT, n);
                ++(context->stack_used);
                DMC_PUSH(*stack, c);
                break;
            case (_TAG_HEADER_MAP >> _TAG_PRIMARY_SIZE):
                if (is_flatmap(c))
                    n = flatmap_get_size(flatmap_val(c));
                else
                    n = hashmap_size(c);
                DMC_PUSH2(*text, matchPushM, n);
                ++(context->stack_used);
                DMC_PUSH(*stack, c);
                break;
            case (_TAG_HEADER_REF >> _TAG_PRIMARY_SIZE):
            {
                Eterm* ref_val = internal_ref_val(c);
                DMC_PUSH(*text, matchEqRef);
                n = thing_arityval(ref_val[0]);
                for (i = 0; i <= n; ++i) {
                    DMC_PUSH(*text, ref_val[i]);
                }
                break;
            }
            case (_TAG_HEADER_POS_BIG >> _TAG_PRIMARY_SIZE):
            case (_TAG_HEADER_NEG_BIG >> _TAG_PRIMARY_SIZE):
            {
                Eterm* bval = big_val(c);
                n = thing_arityval(bval[0]);
                DMC_PUSH(*text, matchEqBig);
                for (i = 0; i <= n; ++i) {
                    DMC_PUSH(*text, (Uint) bval[i]);
                }
                break;
            }
            default: /* BINARY, FUN, VECTOR, or EXTERNAL */
                DMC_PUSH2(*text, matchEqBin, dmc_private_copy(context, c));
                break;
            }
            break;
        }
        _ => panic!("match_compile: Bad object on heap: {}", c),
    }

    Ok(())
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
