/// Execution of the match program, this is Pam.
/// May return THE_NON_VALUE, which is a bailout.
/// the parameter 'arity' is only used if 'term' is actually an array,
/// i.e. 'DCOMP_TRACE' was specified 
fn prog_match(c_p: &RcProcess, pself: &RcProcess, bprog: pam::Pattern, term: Term, /*Eterm *termp, arity: usize,*/ enum erts_pam_run_flags in_flags/*, Uint32 *return_flags*/) -> Result<Term> {
    // termp seems to be null except on tracing
    // arity seems to be null except on tracing
    // return_flags are a tracing thing
    // c_p and pself are same except on tracing
    MatchProg *prog = Binary2MatchProg(bprog);
    const Eterm *ep, *tp, **sp;
    Eterm t;
    Eterm *esp;
    MatchVariable* variables; // a vec with default len of non_value values, will be actual var bindings
    ErtsCodeMFA *cp;
    const UWord *pc = prog->text;
    Eterm *ehp;
    Eterm ret;
    Uint n;
    int i;
    unsigned do_catch;
    ErtsMatchPseudoProcess *mpsp;
    Process *psp;
    Process* build_proc;
    Process *tmpp;
    Process *current_scheduled;
    ErtsSchedulerData *esdp;
    Eterm (*bif)(Process*, ...);
    Eterm bif_args[3];
    int fail_label;
// #ifdef DMC_DEBUG
//     Uint *heap_fence;
//     Uint *stack_fence;
//     Uint save_op;
// #endif /* DMC_DEBUG */

    ERTS_UNDEF(n,0);
    ERTS_UNDEF(current_scheduled,NULL);

    assert!(c_p || !(in_flags & ERTS_PAM_COPY_RESULT));

    mpsp = get_match_pseudo_process(c_p, prog->heap_size);
    psp = &mpsp->process;

    // We need to lure the scheduler into believing in the pseudo process, 
    // because of floating point exceptions. Do *after* mpsp is set!!!

    esdp = erts_get_scheduler_data();
    if (esdp) {
        current_scheduled = esdp->current_process;
    }
    /* SMP: psp->scheduler_data is set by get_match_pseudo_process */

// #ifdef DMC_DEBUG
//     save_op = 0;
//     heap_fence = (Eterm*)((char*) mpsp->u.heap + prog->stack_offset) - 1;
//     stack_fence = (Eterm*)((char*) mpsp->u.heap + prog->heap_size) - 1;
//     *heap_fence = FENCE_PATTERN;
//     *stack_fence = FENCE_PATTERN;
// #endif /* DMC_DEBUG */

// #ifdef HARDDEBUG
// #define FAIL() {erts_printf("Fail line %d\n",__LINE__); goto fail;}
// #else
// #define FAIL() goto fail
// #endif
#define FAIL_TERM am_EXIT /* The term to set as return when bif fails and do_catch != 0 */

    *return_flags = 0U;
    variables = mpsp->u.variables;

restart:
    let mut esp = &term;
    // esp = (Eterm*)((char*)mpsp->u.heap + prog->stack_offset); // seems to be estack pointer
    let esp = Vec::new(); // TODO with some default capacity -- seems to be used as a stack for special form & guard bifs
    // sp = (const Eterm **)esp; // current stack pointer
    let mut sp = iter;
    let mut ret = atom!(TRUE);
    let mut do_catch = false;
    let mut fail_label = -1;
    build_proc = psp;
    if (esdp)
        esdp->current_process = psp;

    // ep ==> current element (iter.next())
    // sp ==> current iterator (could be iter) - tuple or list
    // esp ==> stack of iterators

// #ifdef DEBUG
//     assert!(variables == mpsp->u.variables);
//     for (i=0; i<prog->num_bindings; i++) {
//      variables[i].term = THE_NON_VALUE;
//      variables[i].proc = NULL;
//     }
// #endif

    loop {

    // #ifdef DMC_DEBUG
        // if (*heap_fence != FENCE_PATTERN) {
            // erts_exit(ERTS_ERROR_EXIT, "Heap fence overwritten in db_prog_match after op "
                     // "0x%08x, overwritten with 0x%08x.", save_op, *heap_fence);
        // }
        // if (*stack_fence != FENCE_PATTERN) {
            // erts_exit(ERTS_ERROR_EXIT, "Stack fence overwritten in db_prog_match after op "
                     // "0x%08x, overwritten with 0x%08x.", save_op, 
                     // *stack_fence);
        // }
        // save_op = *pc;
    // #endif
        let pc += 1;
        match pc {
            Opcode::MatchTryMeElse(fail_label) => {
                assert!(fail_label == -1);
            }
            // Opcode::MatchArray(n) => { // only when DCOMP_TRACE, is always first instruction.
            //     if n != arity {
            //         FAIL();
            //     }
            //     ep = termp;
            // }
            // Opcode::MatchArrayBind() => { // When the array size is unknown. (also only on DCOMP_TRACE)
            //     assert!(termp || arity == 0);
            //     variables[n].term = dpm_array_to_list(psp, termp, arity);
            // }
            Opcode::MatchTuple(n) => { // *ep is a tuple of arity n
                if !is_tuple(*ep) {
                    FAIL();
                }
                ep = tuple_val(*ep);
                if arityval(*ep) != n {
                    FAIL();
                }
                ++ep;
            }
            Opcode::MatchPushT(n) => { // *ep is a tuple of arity n, push ptr to first element
                if !is_tuple(*ep) {
                    FAIL();
                }
                tp = tuple_val(*ep);
                if arityval(*tp) != n {
                    FAIL();
                }
                *sp++ = tp + 1;
                ++ep;
            }
            Opcode::MatchList() => {
                if !is_list(*ep) {
                    FAIL();
                }
                ep = list_val(*ep);
            }
            Opcode::MatchPushL() => {
                if !is_list(*ep)
                    FAIL();
                *sp++ = list_val(*ep);
                ++ep;
            }
            Opcode::MatchMap() => {
                if !is_map(*ep) {
                    FAIL();
                }
                if is_flatmap(*ep) {
                    if (flatmap_get_size(flatmap_val(*ep)) < n) {
                        FAIL();
                    }
                } else {
                    assert!(is_hashmap(*ep));
                    if hashmap_size(*ep) < n {
                        FAIL();
                    }
                }
                ep = flatmap_val(*ep);
            }
            Opcode::MatchPushM() => {
                if !is_map(*ep) {
                    FAIL();
                }
                if is_flatmap(*ep) {
                    if flatmap_get_size(flatmap_val(*ep)) < n {
                        FAIL();
                    }
                } else {
                    assert!(is_hashmap(*ep));
                    if hashmap_size(*ep) < n {
                        FAIL();
                    }
                }
                *sp++ = flatmap_val(*ep++);
            }
            Opcode::MatchKey(t) => {
                tp = erts_maps_get(t, make_boxed(ep));
                if !tp {
                    FAIL();
                }
                *sp++ = ep;
                ep = tp;
            }
            Opcode::MatchPop() => {
                ep = *(--sp);
            }
            Opcode::MatchSwap() => {
                tp = sp[-1];
                sp[-1] = sp[-2];
                sp[-2] = tp;
            }
            Opcode::MatchBind(n) => {
                variables[n].term = *ep++;
            }
            Opcode::MatchCmp(n) => {
                if !EQ(variables[n].term, *ep) {
                    FAIL();
                }
                ++ep;
            }
            Opcode::MatchEqBin(t) => {
                if !EQ(t,*ep) {
                    FAIL();
                }
                ++ep;
            }
            Opcode::MatchEqFloat(f) => {
                if !is_float(*ep) {
                    FAIL();
                }
                // if sys_memcmp(float_val(*ep) + 1, pc, sizeof(double)) {
                //     FAIL();
                // }
                ++ep;
            }
            Opcode::MatchEqRef(reference) => {
                if !is_ref(*ep) {
                    FAIL();
                }
                if !EQ(reference, *ep) {
                    FAIL();
                }
                ++ep;
            }
            Opcode::MatchEqBig(bignum) => {
                if !is_big(*ep) {
                    FAIL();
                }
                if !EQ(bignum, *ep) {
                    FAIL();
                }
                ++ep;
            }
            Opcode::MatchEq(t) => {
                assert!(is_immed(t));
                if t != *ep++ {
                    FAIL();
                }
            }
            Opcode::MatchSkip() => {
                ++ep;
            }
            /* 
            * Here comes guard & body instructions
            */
            Opcode::MatchPushC(c) => { // Push constant
                if (in_flags & ERTS_PAM_COPY_RESULT)
                    && do_catch && !is_immed(c) {
                    *esp++ = copy_object(c, c_p);
                }
                else {
                    *esp++ = c;
                }
            }
            Opcode::MatchConsA() => {
                ehp = HAllocX(build_proc, 2, HEAP_XTRA);
                CDR(ehp) = *--esp;
                CAR(ehp) = esp[-1];
                esp[-1] = make_list(ehp);
            }
            Opcode::MatchConsB() => {
                ehp = HAllocX(build_proc, 2, HEAP_XTRA);
                CAR(ehp) = *--esp;
                CDR(ehp) = esp[-1];
                esp[-1] = make_list(ehp);
            }
            Opcode::MatchMkTuple(n) => {
                ehp = HAllocX(build_proc, n+1, HEAP_XTRA);
                t = make_tuple(ehp);
                *ehp++ = make_arityval(n);
                while n-- {
                    *ehp++ = *--esp;
                }
                *esp++ = t;
            }
            // Opcode::MatchMkFlatMap(n) => {
            //     ehp = HAllocX(build_proc, MAP_HEADER_FLATMAP_SZ + n, HEAP_XTRA);
            //     t = *--esp;
            //     {
            //         flatmap_t *m = (flatmap_t *)ehp;
            //         m->thing_word = MAP_HEADER_FLATMAP;
            //         m->size = n;
            //         m->keys = t;
            //     }
            //     t = make_flatmap(ehp);
            //     ehp += MAP_HEADER_FLATMAP_SZ;
            //     while (n--) {
            //         *ehp++ = *--esp;
            //     }
            //     *esp++ = t;
            // }
            Opcode::MatchMkHashMap(n) => {
                esp -= 2*n;
                ehp = HAllocX(build_proc, 2*n, HEAP_XTRA);
                {
                    ErtsHeapFactory factory;
                    Uint ix;
                    for (ix = 0; ix < 2*n; ix++){
                        ehp[ix] = esp[ix];
                    }
                    erts_factory_proc_init(&factory, build_proc);
                    t = erts_hashmap_from_array(&factory, ehp, n, 0);
                    erts_factory_close(&factory);
                }
                *esp++ = t;
            }
            Opcode::MatchCall0(bif) => {
                t = (*bif)(build_proc, bif_args);
                if is_non_value(t) {
                    if (do_catch)
                        t = FAIL_TERM;
                    else
                        FAIL();
                }
                *esp++ = t;
            }
            Opcode::MatchCall1(bif) => {
                t = (*bif)(build_proc, esp-1);
                if is_non_value(t) {
                    if (do_catch)
                        t = FAIL_TERM;
                    else
                        FAIL();
                }
                esp[-1] = t;
            }
            Opcode::MatchCall2(bif) => {
                bif_args[0] = esp[-1];
                bif_args[1] = esp[-2];
                t = (*bif)(build_proc, bif_args);
                if is_non_value(t) {
                    if (do_catch)
                        t = FAIL_TERM;
                    else
                        FAIL();
                }
                --esp;
                esp[-1] = t;
            }
            Opcode::MatchCall3(bif) => {
                bif_args[0] = esp[-1];
                bif_args[1] = esp[-2];
                bif_args[2] = esp[-3];
                t = (*bif)(build_proc, bif_args);
                if is_non_value(t) {
                    if (do_catch)
                        t = FAIL_TERM;
                    else
                        FAIL();
                }
                esp -= 2;
                esp[-1] = t;
            }
            Opcode::MatchPushVResult(n) => {
                assert!(is_value(variables[n].term));
                if (!(in_flags & ERTS_PAM_COPY_RESULT)) {
                    // equivalent to MatchPushV
                    *esp++ = variables[n].term;
                } else {
                    // Build copy on callers heap
                    assert!(is_value(variables[n].term));
                    assert!(!variables[n].proc);
                    variables[n].term = copy_object_x(variables[n].term, c_p, HEAP_XTRA);
                    *esp++ = variables[n].term;
                    #ifdef DEBUG
                    variables[n].proc = c_p;
                    #endif

                }
            }
            Opcode::MatchPushV(n) => {
                assert!(is_value(variables[n].term));
                *esp++ = variables[n].term;
            }
            Opcode::MatchPushExpr() => {
                if in_flags & ERTS_PAM_COPY_RESULT {
                    Uint sz;
                    Eterm* top;
                    sz = size_object(term);
                    top = HAllocX(build_proc, sz, HEAP_XTRA);
                    if in_flags & ERTS_PAM_CONTIGUOUS_TUPLE {
                        assert!(is_tuple(term));
                        *esp++ = copy_shallow(tuple_val(term), sz, &top, &MSO(build_proc));
                    } else {
                        *esp++ = copy_struct(term, sz, &top, &MSO(build_proc));
                    }
                } else {
                    *esp++ = term;
                }
            }
            Opcode::MatchPushArrayAsList() => {
                n = arity; /* Only happens when 'term' is an array */
                tp = termp;
                ehp = HAllocX(build_proc, n*2, HEAP_XTRA);
                *esp++  = make_list(ehp);
                while n-- {
                    *ehp++ = *tp++;
                    *ehp = make_list(ehp + 1);
                    ehp++; /* As pointed out by Mikael Pettersson the expression
                            (*ehp++ = make_list(ehp + 1)) that I previously
                            had written here has undefined behaviour. */
                }
                ehp[-1] = NIL;
            }
            Opcode::MatchPushArrayAsListU() => {
                // This instruction is NOT efficient.
                *esp++ = dpm_array_to_list(build_proc, termp, arity);
            }
            Opcode::MatchTrue() => {
                if *--esp != atom!(TRUE) {
                    FAIL();
                }
            }
            Opcode::MatchOr(n) => {
                t = atom!(FALSE);
                while n-- {
                    if *--esp == atom!(TRUE) {
                        t = atom!(TRUE);
                    } else if *esp != atom!(FALSE) {
                        esp -= n;
                        if do_catch {
                            t = FAIL_TERM;
                            break;
                        } else {
                            FAIL();
                        }
                    }
                }
                *esp++ = t;
            }
            Opcode::MatchAnd(n) => {
                t = atom!(TRUE);
                while n-- {
                    if *--esp == atom!(FALSE) {
                        t = atom!(FALSE);
                    } else if *esp != atom!(TRUE) {
                        esp -= n;
                        if do_catch {
                            t = FAIL_TERM;
                            break;
                        } else {
                            FAIL();
                        }
                    }
                }
                *esp++ = t;
            }
            Opcode::MatchOrElse(n) => {
                if *--esp == atom!(TRUE) {
                    ++esp;
                    pc += n;
                } else if *esp != atom!(FALSE) {
                    if do_catch {
                        *esp++ = FAIL_TERM;
                        pc += n;
                    } else {
                        FAIL();
                    }
                }
            }
            Opcode::MatchAndAlso(n) => {
                if *--esp == atom!(FALSE) {
                    esp++;
                    pc += n;
                } else if *esp != atom!(TRUE) {
                    if do_catch {
                        *esp++ = FAIL_TERM;
                        pc += n;
                    } else {
                        FAIL();
                    }
                }
            }
            Opcode::MatchJump(n) => {
                pc += n;
            }
            Opcode::MatchSelf() => {
                *esp++ = self->common.id;
            }
            Opcode::MatchWaste() => {
                --esp;
            }
            Opcode::MatchReturn() => {
                ret = *--esp;
            }
            // Opcode::MatchProcessDump => {
            //     erts_dsprintf_buf_t *dsbufp = erts_create_tmp_dsbuf(0);
            //     assert!(c_p == self);
            //     print_process_info(ERTS_PRINT_DSBUF, (void *) dsbufp, c_p, ERTS_PROC_LOCK_MAIN);
            //     *esp++ = new_binary(build_proc, (byte *)dsbufp->str, dsbufp->str_len);
            //     erts_destroy_tmp_dsbuf(dsbufp);
            //     break;
            // }
            // Opcode::MatchDisplay => { /* Debugging, not for production! */
            //     erts_printf("%T\n", esp[-1]);
            //     esp[-1] = atom!(TRUE);
            // }
            // Opcode::MatchSetReturnTrace => {
            //     *return_flags |= MATCH_SET_RETURN_TRACE;
            //     *esp++ = atom!(TRUE);
            // }
            // Opcode::MatchSetExceptionTrace => {
            //     *return_flags |= MATCH_SET_EXCEPTION_TRACE;
            //     *esp++ = atom!(TRUE);
            //     break;
            // Opcode::MatchIsSeqTrace => {
            //     assert!(c_p == self);
            //     if (have_seqtrace(SEQ_TRACE_TOKEN(c_p))) {
            //         *esp++ = atom!(TRUE);
            //     } else {
            //         *esp++ = atom!(FALSE);
            //     }
            // }
            // Opcode::MatchSetSeqToken => {
            //     assert!(c_p == self);
            //     t = erts_seq_trace(c_p, esp[-1], esp[-2], 0);
            //     if is_non_value(t) {
            //         esp[-2] = FAIL_TERM;
            //     } else {
            //         esp[-2] = t;
            //     }
            //     --esp;
            // }
            // Opcode::MatchSetSeqTokenFake => {
            //     assert!(c_p == self);
            //     t = seq_trace_fake(c_p, esp[-1]);
            //     if is_non_value(t) {
            //         esp[-2] = FAIL_TERM;
            //     } else {
            //         esp[-2] = t;
            //     }
            //     --esp;
            // }
            // Opcode::MatchGetSeqToken => {
            //     assert!(c_p == self);
            //     if have_no_seqtrace(SEQ_TRACE_TOKEN(c_p)) {
            //         *esp++ = NIL;
            //     } else {
            //         Eterm token;
            //         Uint token_sz;

            //         assert!(SEQ_TRACE_TOKEN_ARITY(c_p) == 5);
            //         assert!(is_immed(SEQ_TRACE_TOKEN_FLAGS(c_p)));
            //         assert!(is_immed(SEQ_TRACE_TOKEN_SERIAL(c_p)));
            //         assert!(is_immed(SEQ_TRACE_TOKEN_LASTCNT(c_p)));

            //         token = SEQ_TRACE_TOKEN(c_p);
            //         token_sz = size_object(token);

            //         ehp = HAllocX(build_proc, token_sz, HEAP_XTRA);
            //         *esp++ = copy_struct(token, token_sz, &ehp, &MSO(build_proc));
            //     }
            // }
            // Opcode::MatchEnableTrace => {
            //     assert!(c_p == self);
            //     if n = erts_trace_flag2bit(esp[-1]) {
            //         erts_proc_lock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         set_tracee_flags(c_p, ERTS_TRACER(c_p), 0, n);
            //         erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         esp[-1] = atom!(TRUE);
            //     } else {
            //         esp[-1] = FAIL_TERM;
            //     }
            // }
            // Opcode::MatchEnableTrace2 => {
            //     assert!(c_p == self);
            //     n = erts_trace_flag2bit((--esp)[-1]);
            //     esp[-1] = FAIL_TERM;
            //     if n {
            //         if tmpp = get_proc(c_p, ERTS_PROC_LOCK_MAIN, esp[0], ERTS_PROC_LOCKS_ALL) {
            //             /* Always take over the tracer of the current process */
            //             set_tracee_flags(tmpp, ERTS_TRACER(c_p), 0, n);
            //             if (tmpp == c_p)
            //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL_MINOR);
            //             else
            //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
            //             esp[-1] = atom!(TRUE);
            //         }
            //     }
            // }
            // Opcode::MatchDisableTrace => {
            //     assert!(c_p == self);
            //     if n = erts_trace_flag2bit(esp[-1]) {
            //         erts_proc_lock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         set_tracee_flags(c_p, ERTS_TRACER(c_p), n, 0);
            //         erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         esp[-1] = atom!(TRUE);
            //     } else {
            //         esp[-1] = FAIL_TERM;
            //     }
            // }
            // Opcode::MatchDisableTrace2 => {
            //     assert!(c_p == self);
            //     n = erts_trace_flag2bit((--esp)[-1]);
            //     esp[-1] = FAIL_TERM;
            //     if (n) {
            //         if ( (tmpp = get_proc(c_p, ERTS_PROC_LOCK_MAIN, esp[0], ERTS_PROC_LOCKS_ALL))) {
            //             /* Always take over the tracer of the current process */
            //             set_tracee_flags(tmpp, ERTS_TRACER(c_p), n, 0);
            //             if (tmpp == c_p)
            //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL_MINOR);
            //             else
            //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
            //             esp[-1] = atom!(TRUE);
            //         }
            //     }
            // }
            // Opcode::MatchCaller => {
            //     assert!(c_p == self);
            //     if (!(c_p->cp) || !(cp = find_function_from_pc(c_p->cp))) {
            //         *esp++ = am_undefined;
            //     } else {
            //         ehp = HAllocX(build_proc, 4, HEAP_XTRA);
            //         *esp++ = make_tuple(ehp);
            //         ehp[0] = make_arityval(3);
            //         ehp[1] = cp->module;
            //         ehp[2] = cp->function;
            //         ehp[3] = make_small((Uint) cp->arity);
            //     }
            // }
            // Opcode::MatchSilent => {
            //     assert!(c_p == self);
            //     --esp;
            //     if (in_flags & ERTS_PAM_IGNORE_TRACE_SILENT)
            //     break;
            //     if (*esp == atom!(TRUE)) {
            //         erts_proc_lock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         ERTS_TRACE_FLAGS(c_p) |= F_TRACE_SILENT;
            //         erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //     }
            //     else if (*esp == atom!(FALSE)) {
            //         erts_proc_lock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         ERTS_TRACE_FLAGS(c_p) &= ~F_TRACE_SILENT;
            //         erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //     }
            // }
            // Opcode::MatchTrace2 => {
            //     assert!(c_p == self);
            //     {
            //         /*    disable         enable                                */
            //         Uint  d_flags  = 0,   e_flags  = 0;  /* process trace flags */
            //         ErtsTracer tracer = erts_tracer_nil;
            //         /* XXX Atomicity note: Not fully atomic. Default tracer
            //         * is sampled from current process but applied to
            //         * tracee and tracer later after releasing main
            //         * locks on current process, so ERTS_TRACER_PROC(c_p)
            //         * may actually have changed when tracee and tracer
            //         * gets updated. I do not think nobody will notice.
            //         * It is just the default value that is not fully atomic.
            //         * and the real argument settable from match spec
            //         * {trace,[],[{{tracer,Tracer}}]} is much, much older.
            //         */
            //         int   cputs = 0;
            //         erts_tracer_update(&tracer, ERTS_TRACER(c_p));
                    
            //         if (! erts_trace_flags(esp[-1], &d_flags, &tracer, &cputs) ||
            //             ! erts_trace_flags(esp[-2], &e_flags, &tracer, &cputs) ||
            //             cputs ) {
            //             (--esp)[-1] = FAIL_TERM;
            //             ERTS_TRACER_CLEAR(&tracer);
            //             break;
            //         }
            //         erts_proc_lock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         (--esp)[-1] = set_match_trace(c_p, FAIL_TERM, tracer,
            //                                     d_flags, e_flags);
            //         erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         ERTS_TRACER_CLEAR(&tracer);
            //     }
            // }
            // Opcode::MatchTrace3 => {
            //     assert!(c_p == self);
            //     {
            //         /*    disable         enable                                */
            //         Uint  d_flags  = 0,   e_flags  = 0;  /* process trace flags */
            //         ErtsTracer tracer = erts_tracer_nil;
            //         /* XXX Atomicity note. Not fully atomic. See above. 
            //         * Above it could possibly be solved, but not here.
            //         */
            //         int   cputs = 0;
            //         Eterm tracee = (--esp)[0];

            //         erts_tracer_update(&tracer, ERTS_TRACER(c_p));
                    
            //         if (! erts_trace_flags(esp[-1], &d_flags, &tracer, &cputs) ||
            //             ! erts_trace_flags(esp[-2], &e_flags, &tracer, &cputs) ||
            //             cputs ||
            //             ! (tmpp = get_proc(c_p, ERTS_PROC_LOCK_MAIN, 
            //                             tracee, ERTS_PROC_LOCKS_ALL))) {
            //             (--esp)[-1] = FAIL_TERM;
            //             ERTS_TRACER_CLEAR(&tracer);
            //             break;
            //         }
            //         if (tmpp == c_p) {
            //             (--esp)[-1] = set_match_trace(c_p, FAIL_TERM, tracer,
            //                                         d_flags, e_flags);
            //             erts_proc_unlock(c_p, ERTS_PROC_LOCKS_ALL_MINOR);
            //         } else {
            //             erts_proc_unlock(c_p, ERTS_PROC_LOCK_MAIN);
            //             (--esp)[-1] = set_match_trace(tmpp, FAIL_TERM, tracer,
            //                                         d_flags, e_flags);
            //             erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
            //             erts_proc_lock(c_p, ERTS_PROC_LOCK_MAIN);
            //         }
            //         ERTS_TRACER_CLEAR(&tracer);
            //     }
            // }
            Opcode::MatchCatch() => {  // Match success, now build result
                do_catch = true;
                if in_flags & ERTS_PAM_COPY_RESULT {
                    build_proc = c_p;
                    if esdp {
                        esdp->current_process = c_p;
                    }
                }
            }
            Opcode::MatchHalt() => {
                goto success;
            }
            _ => erts_exit(ERTS_ERROR_EXIT, "Internal error: unexpected opcode in match program.");
        }
    }
fail:
    *return_flags = 0U;
    if fail_label >= 0 { // We failed during a "TryMeElse", lets restart, with the next match program
        pc = (prog->text) + fail_label;
        cleanup_match_pseudo_process(mpsp, 1);
        goto restart;
    }
    ret = THE_NON_VALUE;
success:

// #ifdef DMC_DEBUG
//     if (*heap_fence != FENCE_PATTERN) {
//         erts_exit(ERTS_ERROR_EXIT, "Heap fence overwritten in db_prog_match after op "
//                  "0x%08x, overwritten with 0x%08x.", save_op, *heap_fence);
//     }
//     if (*stack_fence != FENCE_PATTERN) {
//         erts_exit(ERTS_ERROR_EXIT, "Stack fence overwritten in db_prog_match after op "
//                  "0x%08x, overwritten with 0x%08x.", save_op, 
//                  *stack_fence);
//     }
// #endif

    if esdp {
        esdp->current_process = current_scheduled;
    }

    ret
}
