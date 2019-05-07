// Sample program
// compiling PAM [{{{:$handler_config$, :simple}, :$1, :$2}, [{:>=, :$1, 6}], [:$2]}]
// tuple(3)
// pusht(2)
// bind(1)
// bind(2)
// pop
// eq(:$handler_config$)
// eq(:simple)
// pushv(1)
// push_c(6)
// call2()
// true
// catch
// pushv_result(2)
// return
// halt
// ----
// tuple(2)
// pusht(2)
// eq(#Pid<40>)
// pop
// eq(:application_master)
// bind(1)
// catch
// push_c([])
// pushv_result(1)
// cons_b
// return
// halt
use super::*;
use crate::vm;

bitflags! {
    pub struct Flag: u8 {
        const TMP_RESULT = 1;
        const COPY_RESULT = 2;
        const CONTIGUOUS_TUPLE = 4;
        const IGNORE_TRACE_SILENT = 8;
    }
}

macro_rules! fail {
    () => {{
        // dbg!("Failed at");
        break;
    }};
}

/// Execution of the match program, this is Pam.
/// May return THE_NON_VALUE, which is a bailout.
/// the parameter 'arity' is only used if 'term' is actually an array,
/// i.e. 'DCOMP_TRACE' was specified
// termp seems to be null except on tracing
// arity seems to be null except on tracing
// return_flags are a tracing thing
// process and pself are same except on tracing
#[allow(dead_code)]
pub fn run(
    vm: &vm::Machine,
    process: &RcProcess,
    // pself: &RcProcess,
    pat: &pam::Pattern,
    term: Term,     /*Eterm *termp, arity: usize*/
    in_flags: Flag, /*, Uint32 *return_flags*/
) -> Option<Term> {
    // MatchProg *prog = Binary2MatchProg(bprog);
    // const Eterm *ep, *tp, **sp;
    // Eterm t;
    // Eterm *esp;
    // MatchVariable* variables; // a vec with default len of non_value values, will be actual var bindings
    // ErtsCodeMFA *cp;
    // const UWord *pc = prog->text;
    // Eterm *ehp;
    // Eterm ret;
    // Uint n;
    // int i;
    // unsigned do_catch;
    // ErtsMatchPseudoProcess *mpsp;
    // Process *psp;
    // Process* build_proc;
    // Process *tmpp;
    // Process *current_scheduled;
    // ErtsSchedulerData *esdp;
    // Eterm (*bif)(Process*, ...);
    // Eterm bif_args[3];
    // int fail_label;
    // #ifdef DMC_DEBUG
    //     Uint *heap_fence;
    //     Uint *stack_fence;
    //     Uint save_op;
    // #endif /* DMC_DEBUG */
    // ERTS_UNDEF(n,0);
    // ERTS_UNDEF(current_scheduled,NULL);

    // assert!(process || !(in_flags & ERTS_PAM_COPY_RESULT));

    // mpsp = get_match_pseudo_process(process, prog->heap_size);
    // psp = &mpsp->process;

    // We need to lure the scheduler into believing in the pseudo process,
    // because of floating point exceptions. Do *after* mpsp is set!!!

    // esdp = erts_get_scheduler_data();
    // if (esdp) {
    //     current_scheduled = esdp->current_process;
    // }
    /* SMP: psp->scheduler_data is set by get_match_pseudo_process */

    // #ifdef DMC_DEBUG
    //     save_op = 0;
    //     heap_fence = (Eterm*)((char*) mpsp->u.heap + prog->stack_offset) - 1;
    //     stack_fence = (Eterm*)((char*) mpsp->u.heap + prog->heap_size) - 1;
    //     *heap_fence = FENCE_PATTERN;
    //     *stack_fence = FENCE_PATTERN;
    // #endif /* DMC_DEBUG */
    // #ifdef HARDDEBUG
    // #define fail!() {erts_printf("Fail line %d\n",__LINE__); goto fail;}
    // #else
    // #define fail!() goto fail
    // #endif
    let _FAIL_TERM = atom!(EXIT); // The term to set as return when bif fails and do_catch != 0
    let mut pc = 0;

    //*return_flags = 0U;
    // variables = mpsp->u.variables;
    let mut variables = vec![Term::none(); pat.num_bindings + 1].into_boxed_slice();

    'restart: loop {
        let mut e: &Term;
        let initial = &[term];
        let mut ep: Box<Iterator<Item = &Term>> = Box::new(initial.iter());
        // esp = (Eterm*)((char*)mpsp->u.heap + prog->stack_offset); // seems to be estack pointer
        let mut esp = Vec::with_capacity(pat.stack_need); // TODO with some default capacity -- seems to be used as a stack for special form & guard bifs
        let mut sp: Vec<Box<Iterator<Item = &Term>>> = Vec::with_capacity(pat.stack_need); // TODO with some default capacity -- seems to be used as a stack for special form & guard bifs

        // sp = (const Eterm **)esp; // current stack pointer
        // let mut sp = iter;
        let mut ret = atom!(TRUE);
        let mut do_catch = false;
        let mut fail_label: Option<usize> = None;
        // build_proc = psp;
        // if esdp {
        //     esdp->current_process = psp;
        // }

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
            // println!("pam instr={}", pat.program[pc]);
            match pat.program[pc] {
                Opcode::TryMeElse(fail) => {
                    assert!(fail_label.is_none());
                    fail_label = Some(fail);
                }
                // Opcode::Array(n) => { // only when DCOMP_TRACE, is always first instruction.
                //     if n != arity {
                //         fail!();
                //     }
                //     ep = termp;
                // }
                // Opcode::ArrayBind() => { // When the array size is unknown. (also only on DCOMP_TRACE)
                //     assert!(termp || arity == 0);
                //     variables[n].term = dpm_array_to_list(psp, termp, arity);
                // }
                Opcode::Tuple(n) => {
                    // *ep is a tuple of arity n
                    e = ep.next().unwrap();

                    if let Ok(tup) = Tuple::try_from(&e) {
                        if tup.len() != n {
                            fail!();
                        }
                        ep = Box::new(tup.iter());
                    } else {
                        fail!()
                    }
                }
                Opcode::PushT(n) => {
                    // *ep is a tuple of arity n, push ptr to first element
                    e = ep.next().unwrap();
                    if let Ok(tup) = Tuple::try_from(&e) {
                        if tup.len() != n {
                            fail!();
                        }
                        sp.push(Box::new(tup.iter()));
                    } else {
                        fail!()
                    }
                }
                //                Opcode::List() => {
                //                    if !is_list(*ep) {
                //                        fail!();
                //                    }
                //                    ep = list_val(*ep);
                //                }
                //                Opcode::PushL() => {
                //                    if !is_list(*ep)
                //                        fail!();
                //                    *sp++ = list_val(*ep);
                //                    ++ep;
                //                }
                //                Opcode::Map(n) => {
                //                    if !is_map(*ep) {
                //                        fail!();
                //                    }
                //                    if hashmap_size(*ep) < n {
                //                        fail!();
                //                    }
                //                    ep = flatmap_val(*ep);
                //                }
                //                Opcode::PushM(n) => {
                //                    if !is_map(*ep) {
                //                        fail!();
                //                    }
                //                    if hashmap_size(*ep) < n {
                //                        fail!();
                //                    }
                //                    *sp++ = flatmap_val(*ep++);
                //                }
                //                Opcode::Key(t) => {
                //                    tp = erts_maps_get(t, make_boxed(ep));
                //                    if !tp {
                //                        fail!();
                //                    }
                //                    *sp++ = ep;
                //                    ep = tp;
                //                }
                Opcode::Pop() => {
                    ep = sp.pop().unwrap();
                }
                //                Opcode::Swap() => {
                //                    tp = sp[-1];
                //                    sp[-1] = sp[-2];
                //                    sp[-2] = tp;
                //                }
                Opcode::Bind(n) => {
                    variables[n] = *ep.next().unwrap();
                }
                //                Opcode::Cmp(n) => {
                //                    if !EQ(variables[n].term, *ep) {
                //                        fail!();
                //                    }
                //                    ++ep;
                //                }
                //                Opcode::EqBin(t) => {
                //                    if !EQ(t,*ep) {
                //                        fail!();
                //                    }
                //                    ++ep;
                //                }
                //                Opcode::EqFloat(f) => {
                //                    if !is_float(*ep) {
                //                        fail!();
                //                    }
                //                    // if sys_memcmp(float_val(*ep) + 1, pc, sizeof(double)) {
                //                    //     fail!();
                //                    // }
                //                    ++ep;
                //                }
                //                Opcode::EqRef(reference) => {
                //                    if !is_ref(*ep) {
                //                        fail!();
                //                    }
                //                    if !EQ(reference, *ep) {
                //                        fail!();
                //                    }
                //                    ++ep;
                //                }
                //                Opcode::EqBig(bignum) => {
                //                    if !is_big(*ep) {
                //                        fail!();
                //                    }
                //                    if !EQ(bignum, *ep) {
                //                        fail!();
                //                    }
                //                    ++ep;
                //                }
                Opcode::Eq(t) => {
                    // assert!(is_immed(t));
                    e = ep.next().unwrap();
                    if t != *e {
                        fail!();
                    }
                }
                Opcode::Skip() => {
                    ep.next();
                }
                /*
                 * Here comes guard & body instructions
                 */
                Opcode::PushC(c) => {
                    // Push constant
                    if in_flags.contains(Flag::COPY_RESULT) && do_catch && !c.is_immed() {
                        esp.push(c.deep_clone(&process.context_mut().heap));
                    } else {
                        esp.push(c);
                    }
                }
                Opcode::ConsA() => {
                    let tail = esp.pop().unwrap();
                    let head = esp.pop().unwrap(); // it's in reverse
                    esp.push(cons!(&process.context_mut().heap, head, tail))
                }
                Opcode::ConsB() => {
                    let head = esp.pop().unwrap();
                    let tail = esp.pop().unwrap();
                    esp.push(cons!(&process.context_mut().heap, head, tail))
                }
                //                Opcode::MkTuple(n) => {
                //                    ehp = HAllocX(build_proc, n+1, HEAP_XTRA);
                //                    t = make_tuple(ehp);
                //                    *ehp++ = make_arityval(n);
                //                    while n-- {
                //                        *ehp++ = *--esp;
                //                    }
                //                    *esp++ = t;
                //                }
                // Opcode::MkFlatMap(n) => {
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
                //                Opcode::MkHashMap(n) => {
                //                    esp -= 2*n;
                //                    ehp = HAllocX(build_proc, 2*n, HEAP_XTRA);
                //                    {
                //                        ErtsHeapFactory factory;
                //                        Uint ix;
                //                        for (ix = 0; ix < 2*n; ix++){
                //                            ehp[ix] = esp[ix];
                //                        }
                //                        erts_factory_proc_init(&factory, build_proc);
                //                        t = erts_hashmap_from_array(&factory, ehp, n, 0);
                //                        erts_factory_close(&factory);
                //                    }
                //                    *esp++ = t;
                //                }
                //                Opcode::Call0(bif) => {
                //                    t = (*bif)(build_proc, bif_args);
                //                    if is_non_value(t) {
                //                        if (do_catch)
                //                            t = FAIL_TERM;
                //                        else
                //                            fail!();
                //                    }
                //                    *esp++ = t;
                //                }
                Opcode::Call1(bif) => {
                    let arg0 = esp.pop().unwrap();
                    let args = &[arg0];
                    match bif(vm, process, args) {
                        Ok(t) => {
                            esp.push(t);
                        }
                        Err(_) => {
                            if do_catch {
                                // t = FAIL_TERM;
                            } else {
                                fail!();
                            }
                        }
                    }
                }
                Opcode::Call2(bif) => {
                    let arg1 = esp.pop().unwrap();
                    let arg0 = esp.pop().unwrap(); // it's in reverse
                    let args = &[arg0, arg1];
                    match bif(vm, process, args) {
                        Ok(t) => {
                            esp.push(t);
                        }
                        Err(_) => {
                            if do_catch {
                                // t = FAIL_TERM;
                            } else {
                                fail!();
                            }
                        }
                    }
                }
                Opcode::Call3(bif) => {
                    let arg2 = esp.pop().unwrap();
                    let arg1 = esp.pop().unwrap();
                    let arg0 = esp.pop().unwrap(); // it's in reverse
                    let args = &[arg0, arg1, arg2];
                    match bif(vm, process, args) {
                        Ok(t) => {
                            esp.push(t);
                        }
                        Err(_) => {
                            if do_catch {
                                // t = FAIL_TERM;
                            } else {
                                fail!();
                            }
                        }
                    }
                }
                Opcode::PushVResult(n) => {
                    assert!(!variables[n].is_none());
                    if !in_flags.contains(Flag::COPY_RESULT) {
                        // equivalent to MatchPushV
                        esp.push(variables[n])
                    } else {
                        // Build copy on callers heap
                        // assert!(!variables[n].proc);
                        variables[n] = variables[n].deep_clone(&process.context_mut().heap);
                        esp.push(variables[n])
                    }
                }
                Opcode::PushV(n) => {
                    assert!(!variables[n].is_none());
                    esp.push(variables[n])
                }
                Opcode::PushExpr() => {
                    if in_flags.contains(Flag::COPY_RESULT) {
                        if in_flags.contains(Flag::CONTIGUOUS_TUPLE) {
                            assert!(term.is_tuple());
                            // TODO: use a shallow copy later esp.push(copy_shallow(tuple_val(term), sz, &top, &MSO(build_proc)))
                            esp.push(term.deep_clone(&process.context_mut().heap)); // build_proc
                        } else {
                            esp.push(term.deep_clone(&process.context_mut().heap)); // build_proc
                        }
                    } else {
                        esp.push(term);
                    }
                }
                //                Opcode::PushArrayAsList() => {
                //                    n = arity; /* Only happens when 'term' is an array */
                //                    tp = termp;
                //                    ehp = HAllocX(build_proc, n*2, HEAP_XTRA);
                //                    *esp++  = make_list(ehp);
                //                    while n-- {
                //                        *ehp++ = *tp++;
                //                        *ehp = make_list(ehp + 1);
                //                        ehp++; /* As pointed out by Mikael Pettersson the expression
                //                                (*ehp++ = make_list(ehp + 1)) that I previously
                //                                had written here has undefined behaviour. */
                //                    }
                //                    ehp[-1] = NIL;
                //                }
                //                Opcode::PushArrayAsListU() => {
                //                    // This instruction is NOT efficient.
                //                    *esp++ = dpm_array_to_list(build_proc, termp, arity);
                //                }
                Opcode::True() => {
                    let e = esp.pop().unwrap();
                    if e != atom!(TRUE) {
                        fail!();
                    }
                }
                //                Opcode::Or(n) => {
                //                    t = atom!(FALSE);
                //                    while n-- {
                //                        if *--esp == atom!(TRUE) {
                //                            t = atom!(TRUE);
                //                        } else if *esp != atom!(FALSE) {
                //                            esp -= n;
                //                            if do_catch {
                //                                t = FAIL_TERM;
                //                                break;
                //                            } else {
                //                                fail!();
                //                            }
                //                        }
                //                    }
                //                    *esp++ = t;
                //                }
                //                Opcode::And(n) => {
                //                    t = atom!(TRUE);
                //                    while n-- {
                //                        if *--esp == atom!(FALSE) {
                //                            t = atom!(FALSE);
                //                        } else if *esp != atom!(TRUE) {
                //                            esp -= n;
                //                            if do_catch {
                //                                t = FAIL_TERM;
                //                                break;
                //                            } else {
                //                                fail!();
                //                            }
                //                        }
                //                    }
                //                    *esp++ = t;
                //                }
                //                Opcode::OrElse(n) => {
                //                    if *--esp == atom!(TRUE) { // check top item
                //                        ++esp; // undo --
                //                        pc += n;
                //                    } else if *esp != atom!(FALSE) {
                //                        if do_catch {
                //                            *esp++ = FAIL_TERM;
                //                            pc += n;
                //                        } else {
                //                            fail!();
                //                        }
                //                    }
                //                }
                //                Opcode::AndAlso(n) => {
                //                    if *--esp == atom!(FALSE) { // check top item
                //                        esp++; // undo --
                //                        pc += n;
                //                    } else if *esp != atom!(TRUE) {
                //                        if do_catch {
                //                            *esp++ = FAIL_TERM;
                //                            pc += n;
                //                        } else {
                //                            fail!();
                //                        }
                //                    }
                //                }
                //                Opcode::Jump(n) => {
                //                    pc += n;
                //                }
                //                Opcode::Selff() => {
                //                    *esp++ = self->common.id;
                //                }
                Opcode::Waste() => {
                    esp.pop();
                }
                Opcode::Return() => {
                    ret = esp.pop().unwrap();
                }
                // Opcode::ProcessDump => {
                //     erts_dsprintf_buf_t *dsbufp = erts_create_tmp_dsbuf(0);
                //     assert!(process == self);
                //     print_process_info(ERTS_PRINT_DSBUF, (void *) dsbufp, process, ERTS_PROC_LOCK_MAIN);
                //     *esp++ = new_binary(build_proc, (byte *)dsbufp->str, dsbufp->str_len);
                //     erts_destroy_tmp_dsbuf(dsbufp);
                //     break;
                // }
                // Opcode::Display => { /* Debugging, not for production! */
                //     erts_printf("%T\n", esp[-1]);
                //     esp[-1] = atom!(TRUE);
                // }
                // Opcode::SetReturnTrace => {
                //     *return_flags |= MATCH_SET_RETURN_TRACE;
                //     *esp++ = atom!(TRUE);
                // }
                // Opcode::SetExceptionTrace => {
                //     *return_flags |= MATCH_SET_EXCEPTION_TRACE;
                //     *esp++ = atom!(TRUE);
                //     break;
                // Opcode::IsSeqTrace => {
                //     assert!(process == self);
                //     if (have_seqtrace(SEQ_TRACE_TOKEN(process))) {
                //         *esp++ = atom!(TRUE);
                //     } else {
                //         *esp++ = atom!(FALSE);
                //     }
                // }
                // Opcode::SetSeqToken => {
                //     assert!(process == self);
                //     t = erts_seq_trace(process, esp[-1], esp[-2], 0);
                //     if is_non_value(t) {
                //         esp[-2] = FAIL_TERM;
                //     } else {
                //         esp[-2] = t;
                //     }
                //     --esp;
                // }
                // Opcode::SetSeqTokenFake => {
                //     assert!(process == self);
                //     t = seq_trace_fake(process, esp[-1]);
                //     if is_non_value(t) {
                //         esp[-2] = FAIL_TERM;
                //     } else {
                //         esp[-2] = t;
                //     }
                //     --esp;
                // }
                // Opcode::GetSeqToken => {
                //     assert!(process == self);
                //     if have_no_seqtrace(SEQ_TRACE_TOKEN(process)) {
                //         *esp++ = NIL;
                //     } else {
                //         Eterm token;
                //         Uint token_sz;

                //         assert!(SEQ_TRACE_TOKEN_ARITY(process) == 5);
                //         assert!(is_immed(SEQ_TRACE_TOKEN_FLAGS(process)));
                //         assert!(is_immed(SEQ_TRACE_TOKEN_SERIAL(process)));
                //         assert!(is_immed(SEQ_TRACE_TOKEN_LASTCNT(process)));

                //         token = SEQ_TRACE_TOKEN(process);
                //         token_sz = size_object(token);

                //         ehp = HAllocX(build_proc, token_sz, HEAP_XTRA);
                //         *esp++ = copy_struct(token, token_sz, &ehp, &MSO(build_proc));
                //     }
                // }
                // Opcode::EnableTrace => {
                //     assert!(process == self);
                //     if n = erts_trace_flag2bit(esp[-1]) {
                //         erts_proc_lock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         set_tracee_flags(process, ERTS_TRACER(process), 0, n);
                //         erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         esp[-1] = atom!(TRUE);
                //     } else {
                //         esp[-1] = FAIL_TERM;
                //     }
                // }
                // Opcode::EnableTrace2 => {
                //     assert!(process == self);
                //     n = erts_trace_flag2bit((--esp)[-1]);
                //     esp[-1] = FAIL_TERM;
                //     if n {
                //         if tmpp = get_proc(process, ERTS_PROC_LOCK_MAIN, esp[0], ERTS_PROC_LOCKS_ALL) {
                //             /* Always take over the tracer of the current process */
                //             set_tracee_flags(tmpp, ERTS_TRACER(process), 0, n);
                //             if (tmpp == process)
                //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL_MINOR);
                //             else
                //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
                //             esp[-1] = atom!(TRUE);
                //         }
                //     }
                // }
                // Opcode::DisableTrace => {
                //     assert!(process == self);
                //     if n = erts_trace_flag2bit(esp[-1]) {
                //         erts_proc_lock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         set_tracee_flags(process, ERTS_TRACER(process), n, 0);
                //         erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         esp[-1] = atom!(TRUE);
                //     } else {
                //         esp[-1] = FAIL_TERM;
                //     }
                // }
                // Opcode::DisableTrace2 => {
                //     assert!(process == self);
                //     n = erts_trace_flag2bit((--esp)[-1]);
                //     esp[-1] = FAIL_TERM;
                //     if (n) {
                //         if ( (tmpp = get_proc(process, ERTS_PROC_LOCK_MAIN, esp[0], ERTS_PROC_LOCKS_ALL))) {
                //             /* Always take over the tracer of the current process */
                //             set_tracee_flags(tmpp, ERTS_TRACER(process), n, 0);
                //             if (tmpp == process)
                //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL_MINOR);
                //             else
                //                 erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
                //             esp[-1] = atom!(TRUE);
                //         }
                //     }
                // }
                // Opcode::Caller => {
                //     assert!(process == self);
                //     if (!(process->cp) || !(cp = find_function_from_pc(process->cp))) {
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
                // Opcode::Silent => {
                //     assert!(process == self);
                //     --esp;
                //     if (in_flags & ERTS_PAM_IGNORE_TRACE_SILENT)
                //     break;
                //     if (*esp == atom!(TRUE)) {
                //         erts_proc_lock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         ERTS_TRACE_FLAGS(process) |= F_TRACE_SILENT;
                //         erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //     }
                //     else if (*esp == atom!(FALSE)) {
                //         erts_proc_lock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         ERTS_TRACE_FLAGS(process) &= ~F_TRACE_SILENT;
                //         erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //     }
                // }
                // Opcode::Trace2 => {
                //     assert!(process == self);
                //     {
                //         /*    disable         enable                                */
                //         Uint  d_flags  = 0,   e_flags  = 0;  /* process trace flags */
                //         ErtsTracer tracer = erts_tracer_nil;
                //         /* XXX Atomicity note: Not fully atomic. Default tracer
                //         * is sampled from current process but applied to
                //         * tracee and tracer later after releasing main
                //         * locks on current process, so ERTS_TRACER_PROC(process)
                //         * may actually have changed when tracee and tracer
                //         * gets updated. I do not think nobody will notice.
                //         * It is just the default value that is not fully atomic.
                //         * and the real argument settable from match spec
                //         * {trace,[],[{{tracer,Tracer}}]} is much, much older.
                //         */
                //         int   cputs = 0;
                //         erts_tracer_update(&tracer, ERTS_TRACER(process));

                //         if (! erts_trace_flags(esp[-1], &d_flags, &tracer, &cputs) ||
                //             ! erts_trace_flags(esp[-2], &e_flags, &tracer, &cputs) ||
                //             cputs ) {
                //             (--esp)[-1] = FAIL_TERM;
                //             ERTS_TRACER_CLEAR(&tracer);
                //             break;
                //         }
                //         erts_proc_lock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         (--esp)[-1] = set_match_trace(process, FAIL_TERM, tracer,
                //                                     d_flags, e_flags);
                //         erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         ERTS_TRACER_CLEAR(&tracer);
                //     }
                // }
                // Opcode::Trace3 => {
                //     assert!(process == self);
                //     {
                //         /*    disable         enable                                */
                //         Uint  d_flags  = 0,   e_flags  = 0;  /* process trace flags */
                //         ErtsTracer tracer = erts_tracer_nil;
                //         /* XXX Atomicity note. Not fully atomic. See above.
                //         * Above it could possibly be solved, but not here.
                //         */
                //         int   cputs = 0;
                //         Eterm tracee = (--esp)[0];

                //         erts_tracer_update(&tracer, ERTS_TRACER(process));

                //         if (! erts_trace_flags(esp[-1], &d_flags, &tracer, &cputs) ||
                //             ! erts_trace_flags(esp[-2], &e_flags, &tracer, &cputs) ||
                //             cputs ||
                //             ! (tmpp = get_proc(process, ERTS_PROC_LOCK_MAIN,
                //                             tracee, ERTS_PROC_LOCKS_ALL))) {
                //             (--esp)[-1] = FAIL_TERM;
                //             ERTS_TRACER_CLEAR(&tracer);
                //             break;
                //         }
                //         if (tmpp == process) {
                //             (--esp)[-1] = set_match_trace(process, FAIL_TERM, tracer,
                //                                         d_flags, e_flags);
                //             erts_proc_unlock(process, ERTS_PROC_LOCKS_ALL_MINOR);
                //         } else {
                //             erts_proc_unlock(process, ERTS_PROC_LOCK_MAIN);
                //             (--esp)[-1] = set_match_trace(tmpp, FAIL_TERM, tracer,
                //                                         d_flags, e_flags);
                //             erts_proc_unlock(tmpp, ERTS_PROC_LOCKS_ALL);
                //             erts_proc_lock(process, ERTS_PROC_LOCK_MAIN);
                //         }
                //         ERTS_TRACER_CLEAR(&tracer);
                //     }
                // }
                Opcode::Catch() => {
                    // Match success, now build result
                    do_catch = true;
                    if in_flags.contains(Flag::COPY_RESULT) {
                        // build_proc = process;
                        // if esdp {
                        //     esdp->current_process = process;
                        // }
                    }
                }
                Opcode::Halt() => {
                    // Success was inlined

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

                    //  if esdp {
                    //      esdp->current_process = current_scheduled;
                    //  }

                    return Some(ret);
                }
                _ => unreachable!(
                    "Internal error: unexpected opcode in match program. {}",
                    &pat.program[pc]
                ),
            }
            pc += 1;
        }

        // anything breaking out of this loop is a fail
        // *return_flags = 0U;
        if let Some(fail) = fail_label {
            // We failed during a "TryMeElse", lets restart, with the next match program
            pc += fail;
        // cleanup_match_pseudo_process(mpsp, 1);
        // break 'restart;
        } else {
            return None;
        }
    }
}
