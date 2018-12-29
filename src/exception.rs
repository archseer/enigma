use crate::process::RcProcess;
use crate::process::InstrPtr;
use crate::value::{self, Value};
use crate::module::MFA;

/// http://erlang.org/doc/reference_manual/errors.html#exceptions
pub enum Exception {
    /// Runtime error or the process called erlang:error/1,2
    Error { reason: usize, value: Value },
    /// The process called exit/1
    Exit(Value),
    /// The process called throw/1
    Throw(Value),
}

/// Mapping from error code 'index' to atoms.
pub enum Reason {
    // Eterm error_atom[NUMBER_EXIT_CODES] = {
    //   am_internal_error,	/* 0 */
    //   am_normal,		/* 1 */
    //   am_internal_error,	/* 2 */
    //   am_badarg,		/* 3 */
    //   am_badarith,		/* 4 */
    //   am_badmatch,		/* 5 */
    //   am_function_clause,	/* 6 */
    //   am_case_clause,	/* 7 */
    //   am_if_clause,		/* 8 */
    //   am_undef,		/* 9 */
    //   am_badfun,		/* 10 */
    //   am_badarity,		/* 11 */
    //   am_timeout_value,	/* 12 */
    //   am_noproc,		/* 13 */
    //   am_notalive,		/* 14 */
    //   am_system_limit,	/* 15 */
    //   am_try_clause,	/* 16 */
    //   am_notsup,		/* 17 */
    //   am_badmap,		/* 18 */
    //   am_badkey,		/* 19 */
}

/// To fully understand the error handling, one must keep in mind that
/// when an exception is thrown, the search for a handler can jump back
/// and forth between Beam and native code. Upon each mode switch, a
/// dummy handler is inserted so that if an exception reaches that point,
/// the handler is invoked (like any handler) and transfers control so
/// that the search for a real handler is continued in the other mode.
/// Therefore, c_p->freason and c_p->fvalue must still hold the exception
/// info when the handler is executed, but normalized so that creation of
/// error terms and saving of the stack trace is only done once, even if
/// we pass through the error handling code several times.
///
/// When a new exception is raised, the current stack trace information
/// is quick-saved in a small structure allocated on the heap. Depending
/// on how the exception is eventually caught (perhaps by causing the
/// current process to terminate), the saved information may be used to
/// create a symbolic (human-readable) representation of the stack trace
/// at the point of the original exception.
// TODO: pc could be &Instruction
// return val is another pc pointer (usize + module)

// static BeamInstr*
// handle_error(Process* c_p, BeamInstr* pc, Eterm* reg, ErtsCodeMFA *bif_mfa)
// {
pub fn handle_error(process: &RcProcess, pc: usize, reg: &Value, bif_mfa: &MFA) -> Option<InstrPtr> {
       let heap = &process.context_mut().heap;
//     Eterm Value = c_p->fvalue;
//     let mut value = ;
       let args = Value::Atom(atom::TRUE);

       let context = process.context_mut();

//     ASSERT(c_p->freason != TRAP); /* Should have been handled earlier. */
//     if (c_p->freason & EXF_RESTORE_NIF) {
//      	erts_nif_export_restore_error(c_p, &pc, reg, &bif_mfa);
//     }

// #ifdef DEBUG
//     if (bif_mfa) {
// 	/* Verify that bif_mfa does not point into our nif export */
// 	NifExport *nep = ERTS_PROC_GET_NIF_TRAP_EXPORT(c_p);
// 	ASSERT(!nep || !ErtsInArea(bif_mfa, (char *)nep, sizeof(NifExport)));
//     }
// #endif

//     c_p->i = pc;    /* In case we call erts_exit(). */
//     /*
//      * Check if we have an arglist for the top level call. If so, this
//      * is encoded in Value, so we have to dig out the real Value as well
//      * as the Arglist.
//      */
//     if (c_p->freason & EXF_ARGLIST) { // TODO: this is a special case for BIF error/2
// 	  Eterm* tp;
// 	  ASSERT(is_tuple(Value));
// 	  tp = tuple_val(Value);
// 	  Value = tp[1];
// 	  args = tp[2];
//     }

//     /*
//      * Save the stack trace info if the EXF_SAVETRACE flag is set. The
//      * main reason for doing this separately is to allow throws to later
//      * become promoted to errors without losing the original stack
//      * trace, even if they have passed through one or more catch and
//      * rethrow. It also makes the creation of symbolic stack traces much
//      * more modular.
//      */
       // if (c_p->freason & EXF_SAVETRACE) {
       //     save_stacktrace(c_p, pc, reg, bif_mfa, args);
       // }

       // Throws that are not caught are turned into 'nocatch' errors
//
       if let Some(Exception::Throw{ catches }) = context.exc {
           if context.catches <= 0 {
            //  value = tup2!(heap, Value::Atom(atom::NOCATCH), value);
            //  c_p->freason = EXC_ERROR;
           }
       }

       // Get the fully expanded error term */
       value = expand_error_value(process, c_p->freason, value);

       // Save final error term and stabilize the exception flags so no
       // further expansion is done.
       c_p->fvalue = Value;
       c_p->freason = PRIMARY_EXCEPTION(c_p->freason);

       //  Find a handler or die
       if ((context.catches > 0 || IS_TRACED_FL(c_p, F_EXCEPTION_TRACE))
           && !(c_p->freason & EXF_PANIC)) {
// 	BeamInstr *new_pc;
//         /* The Beam handler code (catch_end or try_end) checks reg[0]
// 	   for THE_NON_VALUE to see if the previous code finished
// 	   abnormally. If so, reg[1], reg[2] and reg[3] should hold the
// 	   exception class, term and trace, respectively. (If the
// 	   handler is just a trap to native code, these registers will
// 	   be ignored.) */
// 	reg[0] = THE_NON_VALUE;
// 	reg[1] = exception_tag[GET_EXC_CLASS(c_p->freason)];
// 	reg[2] = Value;
// 	reg[3] = c_p->ftrace;
//         if ((new_pc = next_catch(c_p, reg))) {
// 	    c_p->cp = 0;	/* To avoid keeping stale references. */
//             ERTS_RECV_MARK_CLEAR(c_p); /* No longer safe to use this position */
// 	    return new_pc;
// 	}
// 	if (c_p->catches > 0) erts_exit(ERTS_ERROR_EXIT, "Catch not found");
       }
//     ERTS_UNREQ_PROC_MAIN_LOCK(c_p);
//     terminate_proc(c_p, Value);
//     ERTS_REQ_PROC_MAIN_LOCK(c_p);
       None
}

/// Find the nearest catch handler
/// TODO: return is instr pointer
fn next_catch(process: &RcProcess, reg: &Value) -> Option<InstrPtr> {
       let context = process.context_mut();
       let active_catches = context.catches > 0;
       let mut prev = 0;
       let mut ptr = context.stack.len();

       debug_assert!(context.stack.last().is_cp());
       // ASSERT(ptr <= STACK_START(c_p));
       // if (ptr == STACK_START(c_p)) return NULL;

       // TODO: tracing instr handling here

       while ptr > 0 {
           match &context.stack[ptr-1] {
               &Value::Catch(..) => {
                   if active_catches {
                       // ASSERT(ptr < STACK_START(c_p));
                       // Unwind the stack up to the current frame.
                       context.stack.truncate(prev);
                       // TODO: tracing handling here
                       // return catch_pc(*ptr);
                   }
               }
               &Value::CP(..) => {
                   prev = ptr;
                   // TODO: OTP does tracing instr handling here
               }
               _ => ()
           }
           ptr -= 1;
       }
       return None
}

/// Terminating the process when an exception is not caught
// static void
// terminate_proc(Process* c_p, Eterm Value)
// {
//     Eterm *hp;
//     Eterm Args = NIL;

//     /* Add a stacktrace if this is an error. */
//     if (GET_EXC_CLASS(c_p->freason) == EXTAG_ERROR) {
//         Value = add_stacktrace(c_p, Value, c_p->ftrace);
//     }
//     /* EXF_LOG is a primary exception flag */
//     if (c_p->freason & EXF_LOG) {
// 	int alive = erts_is_alive;
// 	erts_dsprintf_buf_t *dsbufp = erts_create_logger_dsbuf();

//         /* Build the format message */
// 	erts_dsprintf(dsbufp, "Error in process ~p ");
// 	if (alive)
// 	    erts_dsprintf(dsbufp, "on node ~p ");
// 	erts_dsprintf(dsbufp, "with exit value:~n~p~n");

//         /* Build the args in reverse order */
// 	hp = HAlloc(c_p, 2);
// 	Args = CONS(hp, Value, Args);
// 	if (alive) {
// 	    hp = HAlloc(c_p, 2);
// 	    Args = CONS(hp, erts_this_node->sysname, Args);
// 	}
// 	hp = HAlloc(c_p, 2);
// 	Args = CONS(hp, c_p->common.id, Args);

// 	erts_send_error_term_to_logger(c_p->group_leader, dsbufp, Args);
//     }
//     /*
//      * If we use a shared heap, the process will be garbage-collected.
//      * Must zero c_p->arity to indicate that there are no live registers.
//      */
//     c_p->arity = 0;
//     erts_do_exit_process(c_p, Value);
// }

/// Build and add a symbolic stack trace to the error value.
fn add_stacktrace(process: &RcProcess, value: Value, exc: Value) -> Value {
    let origin = build_stacktrace(process, exc);
    tup2!(heap, value, origin)
}

/// Forming the correct error value from the internal error code.
/// This does not update c_p->fvalue or c_p->freason.
// Eterm
// expand_error_value(Process* c_p, Uint freason, Eterm Value) {
//     Eterm* hp;
//     Uint r;

//     r = GET_EXC_INDEX(freason);
//     ASSERT(r < NUMBER_EXIT_CODES); /* range check */
//     ASSERT(is_value(Value));

//     switch (r) {
//     case (GET_EXC_INDEX(EXC_PRIMARY)):
//         /* Primary exceptions use fvalue as it is */
// 	break;
//     case (GET_EXC_INDEX(EXC_BADMATCH)):
//     case (GET_EXC_INDEX(EXC_CASE_CLAUSE)):
//     case (GET_EXC_INDEX(EXC_TRY_CLAUSE)):
//     case (GET_EXC_INDEX(EXC_BADFUN)):
//     case (GET_EXC_INDEX(EXC_BADARITY)):
//     case (GET_EXC_INDEX(EXC_BADMAP)):
//     case (GET_EXC_INDEX(EXC_BADKEY)):
//         /* Some common exceptions: value -> {atom, value} */
//         ASSERT(is_value(Value));
// 	hp = HAlloc(c_p, 3);
// 	Value = TUPLE2(hp, error_atom[r], Value);
// 	break;
//     default:
//         /* Other exceptions just use an atom as descriptor */
//         Value = error_atom[r];
// 	break;
//     }
// #ifdef DEBUG
//     ASSERT(Value != am_internal_error);
// #endif
//     return Value;
// }

// /*
//  * Quick-saving the stack trace in an internal form on the heap. Note
//  * that c_p->ftrace will point to a cons cell which holds the given args
//  * and the saved data (encoded as a bignum).
//  *
//  * There is an issue with line number information. Line number
//  * information is associated with the address *before* an operation
//  * that may fail or be stored stored on the stack. But continuation
//  * pointers point after its call instruction, not before. To avoid
//  * finding the wrong line number, we'll need to adjust them so that
//  * they point at the beginning of the call instruction or inside the
//  * call instruction. Since its impractical to point at the beginning,
//  * we'll do the simplest thing and decrement the continuation pointers
//  * by one.
//  *
//  * Here is an example of what can go wrong. Without the adjustment
//  * of continuation pointers, the call at line 42 below would seem to
//  * be at line 43:
//  *
//  * line 42
//  * call ...
//  * line 43
//  * gc_bif ...
//  *
//  * (It would be much better to put the arglist - when it exists - in the
//  * error value instead of in the actual trace; e.g. '{badarg, Args}'
//  * instead of using 'badarg' with Args in the trace. The arglist may
//  * contain very large values, and right now they will be kept alive as
//  * long as the stack trace is live. Preferably, the stack trace should
//  * always be small, so that it does not matter if it is long-lived.
//  * However, it is probably not possible to ever change the format of
//  * error terms.)
//  */
// static void
// save_stacktrace(Process* c_p, BeamInstr* pc, Eterm* reg,
// 		ErtsCodeMFA *bif_mfa, Eterm args) {
//     struct StackTrace* s;
//     int sz;
//     int depth = erts_backtrace_depth;    /* max depth (never negative) */
//     if (depth > 0) {
// 	/* There will always be a current function */
// 	depth --;
//     }

//     /* Create a container for the exception data */
//     sz = (offsetof(struct StackTrace, trace) + sizeof(BeamInstr *)*depth
//           + sizeof(Eterm) - 1) / sizeof(Eterm);
//     s = (struct StackTrace *) HAlloc(c_p, 1 + sz);
//     /* The following fields are inside the bignum */
//     s->header = make_pos_bignum_header(sz);
//     s->freason = c_p->freason;
//     s->depth = 0;

//     /*
//      * If the failure was in a BIF other than 'error/1', 'error/2',
//      * 'exit/1' or 'throw/1', save BIF-MFA and save the argument
//      * registers by consing up an arglist.
//      */
//     if (bif_mfa) {
// 	if (bif_mfa->module == am_erlang) {
// 	    switch (bif_mfa->function) {
// 	    case am_error:
// 		if (bif_mfa->arity == 1 || bif_mfa->arity == 2)
// 		    goto non_bif_stacktrace;
// 		break;
// 	    case am_exit:
// 		if (bif_mfa->arity == 1)
// 		    goto non_bif_stacktrace;
// 		break;
// 	    case am_throw:
// 		if (bif_mfa->arity == 1)
// 		    goto non_bif_stacktrace;
// 		break;
// 	    default:
// 		break;
// 	    }
// 	}
// 	s->current = bif_mfa;
// 	/* Save first stack entry */
// 	ASSERT(pc);
// 	if (depth > 0) {
// 	    s->trace[s->depth++] = pc;
// 	    depth--;
// 	}
// 	/* Save second stack entry if CP is valid and different from pc */
// 	if (depth > 0 && c_p->cp != 0 && c_p->cp != pc) {
// 	    s->trace[s->depth++] = c_p->cp - 1;
// 	    depth--;
// 	}
// 	s->pc = NULL;
// 	args = make_arglist(c_p, reg, bif_mfa->arity); /* Overwrite CAR(c_p->ftrace) */
//     } else {

//     non_bif_stacktrace:

// 	s->current = c_p->current;
//         /*
// 	 * For a function_clause error, the arguments are in the beam
// 	 * registers, c_p->cp is valid, and c_p->current is set.
// 	 */
// 	if ( (GET_EXC_INDEX(s->freason)) ==
// 	     (GET_EXC_INDEX(EXC_FUNCTION_CLAUSE)) ) {
// 	    int a;
// 	    ASSERT(s->current);
// 	    a = s->current->arity;
// 	    args = make_arglist(c_p, reg, a); /* Overwrite CAR(c_p->ftrace) */
// 	    /* Save first stack entry */
// 	    ASSERT(c_p->cp);
// 	    if (depth > 0) {
// 		s->trace[s->depth++] = c_p->cp - 1;
// 		depth--;
// 	    }
// 	    s->pc = NULL; /* Ignore pc */
// 	} else {
// 	    if (depth > 0 && c_p->cp != 0 && c_p->cp != pc) {
// 		s->trace[s->depth++] = c_p->cp - 1;
// 		depth--;
// 	    }
// 	    s->pc = pc;
// 	}
//     }

//     /* Package args and stack trace */
//     {
// 	Eterm *hp;
// 	hp = HAlloc(c_p, 2);
// 	c_p->ftrace = CONS(hp, args, make_big((Eterm *) s));
//     }

//     /* Save the actual stack trace */
//     erts_save_stacktrace(c_p, s, depth);
// }

// void
// erts_save_stacktrace(Process* p, struct StackTrace* s, int depth)
// {
//     if (depth > 0) {
// 	Eterm *ptr;
// 	BeamInstr *prev = s->depth ? s->trace[s->depth-1] : NULL;
// 	BeamInstr i_return_trace = beam_return_trace[0];
// 	BeamInstr i_return_to_trace = beam_return_to_trace[0];

// 	/*
// 	 * Traverse the stack backwards and add all unique continuation
// 	 * pointers to the buffer, up to the maximum stack trace size.
// 	 *
// 	 * Skip trace stack frames.
// 	 */
// 	ptr = p->stop;
// 	if (ptr < STACK_START(p) &&
// 	    (is_not_CP(*ptr)|| (*cp_val(*ptr) != i_return_trace &&
// 				*cp_val(*ptr) != i_return_to_trace)) &&
// 	    p->cp) {
// 	    /* Cannot follow cp here - code may be unloaded */
// 	    BeamInstr *cpp = p->cp;
// 	    int trace_cp;
// 	    if (cpp == beam_exception_trace || cpp == beam_return_trace) {
// 		/* Skip return_trace parameters */
// 		ptr += 2;
// 		trace_cp = 1;
// 	    } else if (cpp == beam_return_to_trace) {
// 		/* Skip return_to_trace parameters */
// 		ptr += 1;
// 		trace_cp = 1;
// 	    }
// 	    else {
// 		trace_cp = 0;
// 	    }
// 	    if (trace_cp && s->pc == cpp) {
// 		/*
// 		 * If process 'cp' points to a return/exception trace
// 		 * instruction and 'cp' has been saved as 'pc' in
// 		 * stacktrace, we need to update 'pc' in stacktrace
// 		 * with the actual 'cp' located on the top of the
// 		 * stack; otherwise, we will lose the top stackframe
// 		 * when building the stack trace.
// 		 */
// 		ASSERT(is_CP(p->stop[0]));
// 		s->pc = cp_val(p->stop[0]);
// 	    }
// 	}
// 	while (ptr < STACK_START(p) && depth > 0) {
// 	    if (is_CP(*ptr)) {
// 		if (*cp_val(*ptr) == i_return_trace) {
// 		    /* Skip stack frame variables */
// 		    do ++ptr; while (is_not_CP(*ptr));
// 		    /* Skip return_trace parameters */
// 		    ptr += 2;
// 		} else if (*cp_val(*ptr) == i_return_to_trace) {
// 		    /* Skip stack frame variables */
// 		    do ++ptr; while (is_not_CP(*ptr));
// 		} else {
// 		    BeamInstr *cp = cp_val(*ptr);
// 		    if (cp != prev) {
// 			/* Record non-duplicates only */
// 			prev = cp;
// 			s->trace[s->depth++] = cp - 1;
// 			depth--;
// 		    }
// 		    ptr++;
// 		}
// 	    } else ptr++;
// 	}
//     }
// }

// /*
//  * Getting the relevant fields from the term pointed to by ftrace
//  */
// static struct StackTrace *get_trace_from_exc(Eterm exc) {
//     if (exc == NIL) {
// 	return NULL;
//     } else {
// 	ASSERT(is_list(exc));
// 	return (struct StackTrace *) big_val(CDR(list_val(exc)));
//     }
// }

// static Eterm get_args_from_exc(Eterm exc) {
//     if (exc == NIL) {
// 	return NIL;
//     } else {
// 	ASSERT(is_list(exc));
// 	return CAR(list_val(exc));
//     }
// }

// static int is_raised_exc(Eterm exc) {
//     if (exc == NIL) {
//         return 0;
//     } else {
//         ASSERT(is_list(exc));
//         return bignum_header_is_neg(*big_val(CDR(list_val(exc))));
//     }
// }

// /*
//  * Creating a list with the argument registers
//  */
// static Eterm
// make_arglist(Process* c_p, Eterm* reg, int a) {
//     Eterm args = NIL;
//     Eterm* hp = HAlloc(c_p, 2*a);
//     while (a > 0) {
//         args = CONS(hp, reg[a-1], args);
// 	hp += 2;
// 	a--;
//     }
//     return args;
// }

/// Building a symbolic representation of a saved stack trace. Note that
/// the exception object 'exc', unless NIL, points to a cons cell which
/// holds the given args and the quick-saved data (encoded as a bignum).
/// 
/// If the bignum is negative, the given args is a complete stacktrace.
pub fn build_stacktrace(process: &RcProcess, exc: Value) -> Value {
// Eterm
// build_stacktrace(Process* c_p, Eterm exc) {
//     struct StackTrace* s;
//     Eterm  args;
//     int    depth;
//     FunctionInfo fi;
//     FunctionInfo* stk;
//     FunctionInfo* stkp;
//     Eterm res = NIL;
//     Uint heap_size;
//     Eterm* hp;
//     Eterm mfa;
//     int i;

//     if (! (s = get_trace_from_exc(exc))) {
//         return NIL;
//     }
// #ifdef HIPE
//     if (s->freason & EXF_NATIVE) {
// 	return hipe_build_stacktrace(c_p, s);
//     }
// #endif
//     if (is_raised_exc(exc)) {
// 	return get_args_from_exc(exc);
//     }

//     /*
//      * Find the current function. If the saved s->pc is null, then the
//      * saved s->current should already contain the proper value.
//      */
//     if (s->pc != NULL) {
// 	erts_lookup_function_info(&fi, s->pc, 1);
//     } else if (GET_EXC_INDEX(s->freason) ==
// 	       GET_EXC_INDEX(EXC_FUNCTION_CLAUSE)) {
// 	erts_lookup_function_info(&fi, erts_codemfa_to_code(s->current), 1);
//     } else {
// 	erts_set_current_function(&fi, s->current);
//     }

//     depth = s->depth;
//     /*
//      * If fi.current is still NULL, and we have no
//      * stack at all, default to the initial function
//      * (e.g. spawn_link(erlang, abs, [1])).
//      */
//     if (fi.mfa == NULL) {
// 	if (depth <= 0)
//             erts_set_current_function(&fi, &c_p->u.initial);
// 	args = am_true; /* Just in case */
//     } else {
// 	args = get_args_from_exc(exc);
//     }

//     /*
//      * Look up all saved continuation pointers and calculate
//      * needed heap space.
//      */
//     stk = stkp = (FunctionInfo *) erts_alloc(ERTS_ALC_T_TMP,
// 				      depth*sizeof(FunctionInfo));
//     heap_size = fi.mfa ? fi.needed + 2 : 0;
//     for (i = 0; i < depth; i++) {
// 	erts_lookup_function_info(stkp, s->trace[i], 1);
// 	if (stkp->mfa) {
// 	    heap_size += stkp->needed + 2;
// 	    stkp++;
// 	}
//     }

//     /*
//      * Allocate heap space and build the stacktrace.
//      */
//     hp = HAlloc(c_p, heap_size);
//     while (stkp > stk) {
// 	stkp--;
// 	hp = erts_build_mfa_item(stkp, hp, am_true, &mfa);
// 	res = CONS(hp, mfa, res);
// 	hp += 2;
//     }
//     if (fi.mfa) {
// 	hp = erts_build_mfa_item(&fi, hp, args, &mfa);
// 	res = CONS(hp, mfa, res);
//     }

//     erts_free(ERTS_ALC_T_TMP, (void *) stk);
//     return res;
// }
}

// static BeamInstr*
// call_error_handler(Process* p, ErtsCodeMFA* mfa, Eterm* reg, Eterm func)
// {
//     Eterm* hp;
//     Export* ep;
//     int arity;
//     Eterm args;
//     Uint sz;
//     int i;

//     DBG_TRACE_MFA_P(mfa, "call_error_handler");
//     /*
//      * Search for the error_handler module.
//      */
//     ep = erts_find_function(erts_proc_get_error_handler(p), func, 3,
// 			    erts_active_code_ix());
//     if (ep == NULL) {		/* No error handler */
// 	p->current = mfa;
// 	p->freason = EXC_UNDEF;
// 	return 0;
//     }

//     /*
//      * Create a list with all arguments in the x registers.
//      */
//     arity = mfa->arity;
//     sz = 2 * arity;
//     if (HeapWordsLeft(p) < sz) {
// 	erts_garbage_collect(p, sz, reg, arity);
//     }
//     hp = HEAP_TOP(p);
//     HEAP_TOP(p) += sz;
//     args = NIL;
//     for (i = arity-1; i >= 0; i--) {
// 	args = CONS(hp, reg[i], args);
// 	hp += 2;
//     }

//     /*
//      * Set up registers for call to error_handler:<func>/3.
//      */
//     reg[0] = mfa->module;
//     reg[1] = mfa->function;
//     reg[2] = args;
//     return ep->addressv[erts_active_code_ix()];
// }

// static Export*
// apply_setup_error_handler(Process* p, Eterm module, Eterm function, Uint arity, Eterm* reg)
// {
//     Export* ep;

//     /*
//      * Find the export table index for the error handler. Return NULL if
//      * there is no error handler module.
//      */
//     if ((ep = erts_active_export_entry(erts_proc_get_error_handler(p),
// 				     am_undefined_function, 3)) == NULL) {
// 	return NULL;
//     } else {
// 	int i;
// 	Uint sz = 2*arity;
// 	Eterm* hp;
// 	Eterm args = NIL;

// 	/*
// 	 * Always copy args from registers to a new list; this ensures
// 	 * that we have the same behaviour whether or not this was
// 	 * called from apply or fixed_apply (any additional last
// 	 * THIS-argument will be included, assuming that arity has been
// 	 * properly adjusted).
// 	 */
// 	if (HeapWordsLeft(p) < sz) {
// 	    erts_garbage_collect(p, sz, reg, arity);
// 	}
// 	hp = HEAP_TOP(p);
// 	HEAP_TOP(p) += sz;
// 	for (i = arity-1; i >= 0; i--) {
// 	    args = CONS(hp, reg[i], args);
// 	    hp += 2;
// 	}
// 	reg[0] = module;
// 	reg[1] = function;
// 	reg[2] = args;
//     }

//     return ep;
// }

// static ERTS_INLINE void
// apply_bif_error_adjustment(Process *p, Export *ep,
// 			   Eterm *reg, Uint arity,
// 			   BeamInstr *I, Uint stack_offset)
// {
//     /*
//      * I is only set when the apply is a tail call, i.e.,
//      * from the instructions i_apply_only, i_apply_last_P,
//      * and apply_last_IP.
//      */
//     if (I
// 	&& BeamIsOpCode(ep->beam[0], op_apply_bif)
//         && (ep == bif_export[BIF_error_1]
// 	    || ep == bif_export[BIF_error_2]
// 	    || ep == bif_export[BIF_exit_1]
// 	    || ep == bif_export[BIF_throw_1])) {
// 	/*
// 	 * We are about to tail apply one of the BIFs
// 	 * erlang:error/1, erlang:error/2, erlang:exit/1,
// 	 * or erlang:throw/1. Error handling of these BIFs is
// 	 * special!
// 	 *
// 	 * We need 'p->cp' to point into the calling
// 	 * function when handling the error after the BIF has
// 	 * been applied. This in order to get the topmost
// 	 * stackframe correct. Without the following adjustment,
// 	 * 'p->cp' will point into the function that called
// 	 * current function when handling the error. We add a
// 	 * dummy stackframe in order to achieve this.
// 	 *
// 	 * Note that these BIFs unconditionally will cause
// 	 * an exception to be raised. That is, our modifications
// 	 * of 'p->cp' as well as the stack will be corrected by
// 	 * the error handling code.
// 	 *
// 	 * If we find an exception/return-to trace continuation
// 	 * pointer as the topmost continuation pointer, we do not
// 	 * need to do anything since the information already will
// 	 * be available for generation of the stacktrace.
// 	 */
// 	int apply_only = stack_offset == 0;
// 	BeamInstr *cpp;

// 	if (apply_only) {
// 	    ASSERT(p->cp != NULL);
// 	    cpp = p->cp;
// 	}
// 	else {
// 	    ASSERT(is_CP(p->stop[0]));
// 	    cpp = cp_val(p->stop[0]);
// 	}

// 	if (cpp != beam_exception_trace
// 	    && cpp != beam_return_trace
// 	    && cpp != beam_return_to_trace) {
// 	    Uint need = stack_offset /* bytes */ / sizeof(Eterm);
// 	    if (need == 0)
// 		need = 1; /* i_apply_only */
// 	    if (p->stop - p->htop < need)
// 		erts_garbage_collect(p, (int) need, reg, arity+1);
// 	    p->stop -= need;

// 	    if (apply_only) {
// 		/*
// 		 * Called from the i_apply_only instruction.
// 		 *
// 		 * 'p->cp' contains continuation pointer pointing
// 		 * into the function that called current function.
// 		 * We push that continuation pointer onto the stack,
// 		 * and set 'p->cp' to point into current function.
// 		 */
// 		p->stop[0] = make_cp(p->cp);
// 		p->cp = I;
// 	    }
// 	    else {
// 		/*
// 		 * Called from an i_apply_last_p, or apply_last_IP,
// 		 * instruction.
// 		 *
// 		 * Calling instruction will after we return read
// 		 * a continuation pointer from the stack and write
// 		 * it to 'p->cp', and then remove the topmost
// 		 * stackframe of size 'stack_offset'.
// 		 *
// 		 * We have sized the dummy-stackframe so that it
// 		 * will be removed by the instruction we currently
// 		 * are executing, and leave the stackframe that
// 		 * normally would have been removed intact.
// 		 *
// 		 */
// 		p->stop[0] = make_cp(I);
// 	    }
// 	}
//     }
// }
