use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, TryInto, Tuple};
use crate::vm;

use super::*;

pub fn new_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // DbTable* tb = NULL;
    // Eterm list;
    // Eterm val;
    // Eterm ret;
    // Eterm heir;
    // UWord heir_data;
    // Uint32 status;
    // Sint keypos;
    // int is_named, is_compressed;
    // int is_fine_locked, frequent_read;
    // int cret;
    // DbTableMethod* meth;

    if !args[0].is_atom() {
        return Err(Exception::new(Reason::EXC_BADARG))
    }
    // TODO: is_list already checks for nil? needs impl change maybe
    if !(args[1].is_nil() || args[1].is_list()) {
        return Err(Exception::new(Reason::EXC_BADARG))
    }

    let status = DB_SET | DB_PROTECTED;
    keypos = 1;
    is_named = 0;
    is_fine_locked = 0;
    frequent_read = 0;
    heir = am_none;
    heir_data = (UWord) am_undefined;
    is_compressed = erts_ets_always_compress;

    list = BIF_ARG_2;

    match args[1].try_into() {
        Ok(cons) => {
            let cons: &Cons = cons; // type annotation
            let heap = &process.context_mut().heap;
            for val in cons.iter() {
                match val.into_variant() {
                    Variant::Atom(atom::BAG) => {
                        status.insert(Status::DB_BAG);
                        status.remove(Status::DB_SET | Status::DB_DUPLICATE_BAG | Status::DB_ORDERED_SET | Status::DB_CA_ORDERED_SET);
                    }
                    Variant::Atom(atom::DUPLICATE_BAG) => {
                        status.insert(Status::DB_DUPLICATE_BAG);
                        status.remove(Status::DB_SET | Status::DB_BAG | Status::DB_ORDERED_SET | Status::DB_CA_ORDERED_SET);
                    }
                    Variant::Atom(atom::ORDERED_SET) => {
                        status.insert(Status::DB_ORDERED_SET);
                        status.remove(Status::DB_SET | Status::DB_DUPLICATE_BAG | Status::DB_SET | Status::DB_CA_ORDERED_SET);
                    }
                    Variant::Pointer(_ptr) => {
                        // tuple
                        match args[1].try_into() {
                            Ok(tup) => {
                                let tup: &Tuple = tup; // type annotation
                                if t.len() == 2 {
                                    return Err(Exception::new(Reason::EXC_BADARG));
                                }

                            }
                            _ => return Err(Exception::new(Reason::EXC_BADARG)),
                        }
                        if (arityval(tp[0]) == 2) {
                            if (tp[1] == am_keypos
                                && is_small(tp[2]) && (signed_val(tp[2]) > 0)) {
                                keypos = signed_val(tp[2]);
                            }		
                            else if (tp[1] == am_write_concurrency) {
                                if (tp[2] == am_true) {
                                    is_fine_locked = 1;
                                } else if (tp[2] == am_false) {
                                    is_fine_locked = 0;
                                } else break;
                            }
                            else if (tp[1] == am_read_concurrency) {
                                if (tp[2] == am_true) {
                                    frequent_read = 1;
                                } else if (tp[2] == am_false) {
                                    frequent_read = 0;
                                } else break;
                                
                            }
                            else if (tp[1] == am_heir && tp[2] == am_none) {
                                heir = am_none;
                                heir_data = am_undefined;
                            }
                            else break;
                        }
                        else if (arityval(tp[0]) == 3 && tp[1] == am_heir
                                && is_internal_pid(tp[2])) {
                            heir = tp[2];
                            heir_data = tp[3];
                        }
                        else break;
                    }
                    else if (val == am_public) {
                        status |= DB_PUBLIC;
                        status &= ~(DB_PROTECTED|DB_PRIVATE);
                    }
                    else if (val == am_private) {
                        status |= DB_PRIVATE;
                        status &= ~(DB_PROTECTED|DB_PUBLIC);
                    }
                    else if (val == am_named_table) {
                        is_named = 1;
                        status |= DB_NAMED_TABLE;
                    }
                    else if (val == am_compressed) {
                        is_compressed = 1;
                    }
                    else if (val == am_set || val == am_protected)
                        ;
                }
            }
        }
        // TODO skip if args is nil
        _ => return Err(Exception::new(Reason::EXC_BADARG)) 
    }

    if !list.is_nil() { /* bad opt or not a well formed list */
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    if (IS_TREE_TABLE(status) && is_fine_locked && !(status & DB_PRIVATE)) {
        meth = &db_catree;
        status |= DB_CA_ORDERED_SET;
        status &= ~(DB_SET | DB_BAG | DB_DUPLICATE_BAG | DB_ORDERED_SET);
        status |= DB_FINE_LOCKED;
    } else if (IS_HASH_TABLE(status)) {
	meth = &db_hash;
	if (is_fine_locked && !(status & DB_PRIVATE)) {
	    status |= DB_FINE_LOCKED;
	}
    } else if (IS_TREE_TABLE(status)) {
	meth = &db_tree;
    } else {
	BIF_ERROR(BIF_P, BADARG);
    }

    if frequent_read && !(status & DB_PRIVATE) {
	status |= DB_FREQ_READ;
    }

    // we create table outside any table lock and take the unusal cost of destroy table if it fails
    // to find a slot 
    {
        DbTable init_tb;

	erts_atomic_init_nob(&init_tb.common.memory_size, 0);
	tb = (DbTable*) erts_db_alloc(ERTS_ALC_T_DB_TABLE,
				      &init_tb, sizeof(DbTable));
	erts_atomic_init_nob(&tb->common.memory_size,
				 erts_atomic_read_nob(&init_tb.common.memory_size));
    }

    tb->common.meth = meth;
    tb->common.the_name = BIF_ARG_1;
    tb->common.status = status;    
    tb->common.type = status;
    /* Note, 'type' is *read only* from now on... */
    erts_refc_init(&tb->common.fix_count, 0);
    db_init_lock(tb, status & (DB_FINE_LOCKED|DB_FREQ_READ));
    tb->common.keypos = keypos;
    tb->common.owner = BIF_P->common.id;
    set_heir(BIF_P, tb, heir, heir_data);

    erts_atomic_init_nob(&tb->common.nitems, 0);

    tb->common.fixing_procs = NULL;
    tb->common.compress = is_compressed;
// #ifdef ETS_DBG_FORCE_TRAP
//     erts_atomic_init_nob(&tb->common.dbg_force_trap, erts_ets_dbg_force_trap);
// #endif

    // had an assert here before, hence unwrap
    cret = meth->db_create(BIF_P, tb).unwrap();

    make_btid(tb);

    let ret = if is_named {
        args[0]
    } else {
        make_tid(BIF_P, tb)
    };

    save_sched_table(BIF_P, tb);
    save_owned_table(BIF_P, tb);

    if is_named && !insert_named_tab(BIF_ARG_1, tb, 0) {
        tid_clear(BIF_P, tb);
        delete_owned_table(BIF_P, tb);

	db_lock(tb,LCK_WRITE);
	free_heir_data(tb);
	tb->common.meth->db_free_empty_table(tb);
	db_unlock(tb,LCK_WRITE);
        table_dec_refc(tb, 0);
	BIF_ERROR(BIF_P, BADARG);
    }
    
    BIF_P->flags |= F_USING_DB; /* So we can remove tb if p dies */

// #ifdef HARDDEBUG
//     erts_fprintf(stderr,
// 		"ets:new(%T,%T)=%T; Process: %T, initial: %T:%T/%bpu\n",
// 		 BIF_ARG_1, BIF_ARG_2, ret, BIF_P->common.id,
// 		 BIF_P->u.initial[0], BIF_P->u.initial[1], BIF_P->u.initial[2]);
// #endif

    ret
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
// insert_2
// insert_new_2
// rename_2
// whereis_1
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
