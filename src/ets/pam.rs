//! Pattern matching abstract machine (PAM)
use super::*;

struct Pattern {}

pub(crate) struct Compiler {
    matchexpr: Vec<Term>,
    guardexpr: Vec<Term>,
    bodyexpr: Vec<Term>,
    cflags: usize,
    stack_used: usize,
    stack_need: usize,
}

impl Compiler {
    pub(crate) fn new(matchexpr: Vec<Term>, guardexpr: Vec<Term>, bodyexpr: Vec<Term>, num_match: usize, cflags: usize) -> Self {
        Self {
            stack_need: 0,
            stack_used: 0,
            // save: NULL,
            // copy: NULL,
            num_match,
            matchexpr,
            guardexpr,
            bodyexpr,
            err_info,
            cflags
        }
    }

    /// The actual compiling of the match expression and the guards.
    pub(crate) fn match_compile(&mut self) -> Result<Vec<u8>, Error> {
        // DMCHeap heap;
        // MatchProg *ret = NULL;
        // Eterm t;
        // Uint i;
        // Uint num_iters;
        // int structure_checked;
        // DMCRet res;
        let mut current_try_label = -1;
        // Binary *bp = NULL;

        // DMC_STACK_TYPE(Eterm) stack;
        // DMC_STACK_TYPE(UWord) text;
        let stack = Vec::new();
        let text = Vec::new();

        heap.size = DMC_DEFAULT_SIZE;
        heap.vars = heap.vars_def;

        // Compile the match expression.
        heap.vars_used = 0;

        for i in 0..self.num_match { // long loop ahead
            // sys_memset(heap.vars, 0, heap.size * sizeof(*heap.vars));
            self.current_match = i;
            let mut t = self.matchexpr[self.current_match];
            self.stack_used = 0;
            let structure_checked = false;

            if self.current_match < self.num_match - 1 {
                text.push(Opcode::MatchTryMeElse);
                current_try_label = text.len() - 1;
                text.push(0);
            } else {
                current_try_label = -1;
            }

            let clause_start = text.len() - 1; // the "special" test needs it
            loop {
                match t.into_variant() {
                    Variant::Pointer(..) => {
                        match self.get_boxed_header().unwrap() {
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
                                        if self.err_info {
                                            add_dmc_err(self.err_info, "Variable found in map key.", -1, 0UL, dmcError);
                                        }
                                        DESTROY_WSTACK(wstack);
                                        goto error;
                                    } else if (key == am_Underscore) {
                                        if (self.err_info) {
                                            add_dmc_err(self.err_info,
                                                    "Underscore found in map key.",
                                                    -1, 0UL, dmcError);
                                        }
                                        DESTROY_WSTACK(wstack);
                                        goto error;
                                    }
                                    DMC_PUSH2(text, matchKey, dmc_private_copy(&self, key));
                                    {
                                        int old_stack = ++(self.stack_used);
                                        res = self.one_term(&heap, &stack, &text,
                                                        value);
                                        ASSERT(res != retFail);
                                        if (old_stack != self.stack_used) {
                                            ASSERT(old_stack + 1 == self.stack_used);
                                            DMC_PUSH(text, matchSwap);
                                        }
                                        if (self.stack_used > self.stack_need) {
                                            self.stack_need = self.stack_used;
                                        }
                                        DMC_PUSH(text, matchPop);
                                        --(self.stack_used);
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
                                    if (res = self.one_term(&heap, &stack, &text, val)) != retOk {
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
                        if self.one_term(&heap, &stack, &text, CAR(list_val(t))) != retOk {
                            goto error;
                        }
                        t = CDR(list_val(t));
                        continue;
                    }
                    _ =>  {// Nil and non proper tail end's or single terms as match expressions.
                    //simple_term:
                        structure_checked = false;
                        if self.one_term(&heap, &stack, &text, t) != retOk {
                            goto error;
                        }
                        break;
                    }
                }

                // The *program's* stack just *grows* while we are traversing one composite data
                // structure, we can check the stack usage here

                // if (self.stack_used > self.stack_need)
                    // self.stack_need = self.stack_used;

                // We are at the end of one composite data structure, pop sub structures and emit
                // a matchPop instruction (or break)
                if let Some(val) = stack.pop() {
                    t = val;
                    text.push(Opcode::MatchPop);
                    structure_checked = true; // Checked with matchPushT or matchPushL
                    --(self.stack_used);
                } else {
                    break;
                }
            } // end type loop

            // There is one single top variable in the match expression
            // if the text is two Uint's and the single instruction
            // is 'matchBind' or it is only a skip.
            self.special =
                ((text.len() - 1) == 2 + clause_start &&
                DMC_PEEK(text,clause_start) == matchBind) ||
                ((text.len() - 1) == 1 + clause_start &&
                DMC_PEEK(text, clause_start) == matchSkip);

            if flags & DCOMP_TRACE {
                if (self.special) {
                    if (DMC_PEEK(text, clause_start) == matchBind) {
                        DMC_POKE(text, clause_start, matchArrayBind);
                    }
                } else {
                    assert!(DMC_STACK_NUM(text) >= 1);
                    if DMC_PEEK(text, clause_start) != matchTuple {
                        /* If it isn't "special" and the argument is
                        not a tuple, the expression is not valid
                        when matching an array*/
                        if self.err_info {
                            add_dmc_err(self.err_info,
                                        "Match head is invalid in "
                                        "this self.",
                                        -1, 0UL,
                                        dmcError);
                        }
                        goto error;
                    }
                    DMC_POKE(text, clause_start, matchArray);
                }
            }


            // ... and the guards
            self.is_guard = true;
            if compile_guard_expr(&self, &heap, &text, self.guardexpr[self.current_match]) != retOk {
                goto error;
            }
            self.is_guard = false;
            if ((self.cflags & DCOMP_TABLE) &&
                !is_list(self.bodyexpr[self.current_match])) {
                if (self.err_info) {
                    add_dmc_err(self.err_info,
                                "Body clause does not return "
                                "anything.", -1, 0UL,
                                dmcError);
                }
                goto error;
            }
            if (compile_guard_expr(&self, &heap, &text, self.bodyexpr[self.current_match]) != retOk) {
                goto error;
            }

            // The compilation does not bail out when error information is requested, so we need to
            // detect that here...
            if self.err_info != NULL && self.err_info.error_added {
                goto error;
            }


            // If the matchprogram comes here, the match is successful
            text.push(Opcode::MatchHalt);
            // Fill in try-me-else label if there is one.
            if current_try_label >= 0 {
                DMC_POKE(text, current_try_label, DMC_STACK_NUM(text));
            }
        } /* for (self.current_match = 0 ...) */


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
        ret->term_save = self.save;
        ret->num_bindings = heap.vars_used;
        ret->single_variable = self.special;
        sys_memcpy(ret->text, DMC_STACK_DATA(text),
                DMC_STACK_NUM(text) * sizeof(UWord));
        ret->stack_offset = heap.vars_used*sizeof(MatchVariable) + FENCE_PATTERN_SIZE;
        ret->heap_size = ret->stack_offset + self.stack_need * sizeof(Eterm*) + FENCE_PATTERN_SIZE;

    #ifdef DMC_DEBUG
        ret->prog_end = ret->text + DMC_STACK_NUM(text);
    #endif

        // Fall through to cleanup code, but self.save should not be free'd
        self.save = NULL;
    error: // Here is were we land when compilation failed.
        if (self.save != NULL) {
            free_message_buffer(self.save);
            self.save = NULL;
        }
        DMC_FREE(stack);
        DMC_FREE(text);
        if (self.copy != NULL)
            free_message_buffer(self.copy);
        if (heap.vars != heap.vars_def)
            erts_free(ERTS_ALC_T_DB_MS_CMPL_HEAP, (void *) heap.vars);
        return bp;
    }

    /// Handle one term in the match expression (not the guard)
    #[inline]
    fn one_term(&mut self, DMCHeap *heap, DMC_STACK_TYPE(Eterm) *stack, DMC_STACK_TYPE(UWord) *text, Eterm c) -> DMCRet {
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
                    //     for (j = 0; j < self->num_match; ++j) {
                    //         sz += size_object(self->matchexpr[j]);
                    //         sz2 += size_object(self->guardexpr[j]);
                    //         sz3 += size_object(self->bodyexpr[j]);
                    //     }
                    //     self->copy =
                    //         new_message_buffer(sz + sz2 + sz3 +
                    //                         self->num_match);
                    //     save_hp = hp = self->copy->mem;
                    //     hp += self->num_match;
                    //     for (j = 0; j < self->num_match; ++j) {
                    //         self->matchexpr[j] =
                    //             copy_struct(self->matchexpr[j],
                    //                         size_object(self->matchexpr[j]), &hp,
                    //                         &(self->copy->off_heap));
                    //         self->guardexpr[j] =
                    //             copy_struct(self->guardexpr[j],
                    //                         size_object(self->guardexpr[j]), &hp,
                    //                         &(self->copy->off_heap));
                    //         self->bodyexpr[j] =
                    //             copy_struct(self->bodyexpr[j],
                    //                         size_object(self->bodyexpr[j]), &hp,
                    //                         &(self->copy->off_heap));
                    //     }
                    //     for (j = 0; j < self->num_match; ++j) {
                    //         /* the actual expressions can be
                    //         atoms in their selves, place them first */
                    //         *save_hp++ = self->matchexpr[j];
                    //     }
                    //     heap->size = match_compact(self->copy,
                    //                             self->err_info);
                    //     for (j = 0; j < self->num_match; ++j) {
                    //         /* restore the match terms, as they
                    //         may be atoms that changed */
                    //         self->matchexpr[j] = self->copy->mem[j];
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
                ++(self->stack_used);
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
                    ++(self->stack_used);
                    DMC_PUSH(*stack, c);
                    break;
                case (_TAG_HEADER_MAP >> _TAG_PRIMARY_SIZE):
                    if (is_flatmap(c))
                        n = flatmap_get_size(flatmap_val(c));
                    else
                        n = hashmap_size(c);
                    DMC_PUSH2(*text, matchPushM, n);
                    ++(self->stack_used);
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
                    DMC_PUSH2(*text, matchEqBin, dmc_private_copy(self, c));
                    break;
                }
                break;
            }
            _ => panic!("match_compile: Bad object on heap: {}", c),
        }

        Ok(())
    }
}

