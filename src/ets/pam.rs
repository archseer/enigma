//! Pattern matching abstract machine (PAM)
use super::*;

use crate::value::{Variant};

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

        // STACK_TYPE(Eterm) stack;
        // STACK_TYPE(UWord) text;
        let stack = Vec::new();
        let text = Vec::new();

        heap.size = DEFAULT_SIZE;
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
                                    PUSH2(text, matchMap, num_iters);
                                }
                                structure_checked = 0;

                                hashmap_iterator_init(&wstack, t, 0);

                                while ((kv=hashmap_iterator_next(&wstack)) != NULL) {
                                    Eterm key = CAR(kv);
                                    Eterm value = CDR(kv);
                                    if (db_is_variable(key) >= 0) {
                                        if self.err_info {
                                            add_err(self.err_info, "Variable found in map key.", -1, 0UL, dmcError);
                                        }
                                        DESTROY_WSTACK(wstack);
                                        return Err(());
                                    } else if (key == am_Underscore) {
                                        if (self.err_info) {
                                            add_err(self.err_info,
                                                    "Underscore found in map key.",
                                                    -1, 0UL, dmcError);
                                        }
                                        DESTROY_WSTACK(wstack);
                                        return Err(());
                                    }
                                    PUSH2(text, matchKey, private_copy(&self, key));
                                    {
                                        int old_stack = ++(self.stack_used);
                                        res = self.one_term(&heap, &stack, &text,
                                                        value);
                                        ASSERT(res != retFail);
                                        if (old_stack != self.stack_used) {
                                            ASSERT(old_stack + 1 == self.stack_used);
                                            PUSH(text, matchSwap);
                                        }
                                        if (self.stack_used > self.stack_need) {
                                            self.stack_need = self.stack_used;
                                        }
                                        PUSH(text, matchPop);
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
                                        return Err(());
                                    }
                                }
                                break;
                            }
                            _ => {
                                goto simple_term;
                            }
                        }
                    }
                    Variant::Cons(..) => {
                        if !structure_checked {
                            text.push(Opcode::MatchList);
                        }
                        structure_checked = false; // Whatever it is, we did not pop it
                        if self.one_term(&heap, &stack, &text, CAR(list_val(t))) != retOk {
                            return Err(());
                        }
                        t = CDR(list_val(t));
                        continue;
                    }
                    _ =>  {// Nil and non proper tail end's or single terms as match expressions.
                    //simple_term:
                        structure_checked = false;
                        if self.one_term(&heap, &stack, &text, t) != retOk {
                            return Err(());
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
                 PEEK(text,clause_start) == matchBind) ||
                ((text.len() - 1) == 1 + clause_start &&
                 PEEK(text, clause_start) == matchSkip);

            if flags & DCOMP_TRACE {
                if self.special {
                    if (PEEK(text, clause_start) == matchBind) {
                        POKE(text, clause_start, matchArrayBind);
                    }
                } else {
                    assert!(STACK_NUM(text) >= 1);
                    if PEEK(text, clause_start) != matchTuple {
                        /* If it isn't "special" and the argument is
                           not a tuple, the expression is not valid
                           when matching an array*/
                        if self.err_info {
                            add_err(self.err_info,
                                        "Match head is invalid in "
                                        "this self.",
                                        -1, 0UL,
                                        dmcError);
                        }
                        return Err(());
                    }
                    POKE(text, clause_start, matchArray);
                }
            }

            // ... and the guards
            self.is_guard = true;
            if compile_guard_expr(&self, &heap, &text, self.guardexpr[self.current_match]) != retOk {
                return Err(());
            }
            self.is_guard = false;
            if ((self.cflags & DCOMP_TABLE) &&
                !is_list(self.bodyexpr[self.current_match])) {
                if (self.err_info) {
                    add_err(self.err_info,
                                "Body clause does not return "
                                "anything.", -1, 0UL,
                                dmcError);
                }
                return Err(());
            }
            if (compile_guard_expr(&self, &heap, &text, self.bodyexpr[self.current_match]) != retOk) {
                return Err(());
            }

            // The compilation does not bail out when error information is requested, so we need to
            // detect that here...
            if self.err_info != NULL && self.err_info.error_added {
                return Err(());
            }


            // If the matchprogram comes here, the match is successful
            text.push(Opcode::MatchHalt);
            // Fill in try-me-else label if there is one.
            if current_try_label >= 0 {
                POKE(text, current_try_label, STACK_NUM(text));
            }
        } /* for (self.current_match = 0 ...) */


        /*
        ** Done compiling
        ** Allocate enough space for the program,
        ** heap size is in 'heap_used', stack size is in 'stack_need'
        ** and text size is simply STACK_NUM(text).
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
                                    (STACK_NUM(text) * sizeof(UWord))),
                                    erts_db_match_prog_destructor);
        ret = Binary2MatchProg(bp);
        ret.saved_program_buf = NULL;
        ret.saved_program = NIL;
        ret.term_save = self.save;
        ret.num_bindings = heap.vars_used;
        ret.single_variable = self.special;
        sys_memcpy(ret.text, STACK_DATA(text),
                STACK_NUM(text) * sizeof(UWord));
        ret.stack_offset = heap.vars_used*sizeof(MatchVariable) + FENCE_PATTERN_SIZE;
        ret.heap_size = ret.stack_offset + self.stack_need * sizeof(Eterm*) + FENCE_PATTERN_SIZE;

    #ifdef DEBUG
        ret.prog_end = ret.text + STACK_NUM(text);
    #endif

        // Fall through to cleanup code, but self.save should not be free'd
        self.save = NULL;
        // ...
        return bp;
    }

    /// Handle one term in the match expression (not the guard)
    #[inline]
    fn one_term(&mut self, DMCHeap *heap, STACK_TYPE(Eterm) *stack, STACK_TYPE(UWord) *text, Eterm c) -> DMCRet {
        // Sint n;
        // Eterm *hp;
        // Uint sz, sz2, sz3;
        // Uint i, j;

        switch (c & _TAG_PRIMARY_MASK) {
            case TAG_PRIMARY_IMMED1:
                if ((n = db_is_variable(c)) >= 0) { /* variable */
                    if heap.vars[n].is_bound {
                        PUSH2(*text, matchCmp, n);
                    } else { /* Not bound, bind! */
                        if n >= heap.vars_used {
                            heap.vars_used = n + 1;
                        }
                        PUSH2(*text, matchBind, n);
                        heap.vars[n].is_bound = true;
                    }
                } else if c == atom!(UNDERSCORE) {
                    text.push(Opcode::MatchSkip);
                } else {
                    // Any immediate value
                    text.push(Opcode::MatchEq(c as usize));
                }
                break;
            case TAG_PRIMARY_LIST:
                PUSH(*text, matchPushL);
                ++(self.stack_used);
                PUSH(*stack, c);
                break;
            case TAG_HEADER_FLOAT:
                PUSH2(*text, matchEqFloat, (Uint) float_val(c)[1]);
        #ifdef ARCH_64
                PUSH(*text, (Uint) 0);
        #else
                PUSH(*text, (Uint) float_val(c)[2]);
        #endif
                break;
            case TAG_PRIMARY_BOXED: {
                Eterm hdr = *boxed_val(c);
                switch ((hdr & _TAG_HEADER_MASK) >> _TAG_PRIMARY_SIZE) {
                case (_TAG_HEADER_ARITYVAL >> _TAG_PRIMARY_SIZE):
                    n = arityval(*tuple_val(c));
                    PUSH2(*text, matchPushT, n);
                    ++(self.stack_used);
                    PUSH(*stack, c);
                    break;
                case (_TAG_HEADER_MAP >> _TAG_PRIMARY_SIZE):
                    if (is_flatmap(c))
                        n = flatmap_get_size(flatmap_val(c));
                    else
                        n = hashmap_size(c);
                    PUSH2(*text, matchPushM, n);
                    ++(self.stack_used);
                    PUSH(*stack, c);
                    break;
                case (_TAG_HEADER_REF >> _TAG_PRIMARY_SIZE):
                {
                    Eterm* ref_val = internal_ref_val(c);
                    PUSH(*text, matchEqRef);
                    n = thing_arityval(ref_val[0]);
                    for (i = 0; i <= n; ++i) {
                        PUSH(*text, ref_val[i]);
                    }
                    break;
                }
                case (_TAG_HEADER_POS_BIG >> _TAG_PRIMARY_SIZE):
                case (_TAG_HEADER_NEG_BIG >> _TAG_PRIMARY_SIZE):
                {
                    Eterm* bval = big_val(c);
                    n = thing_arityval(bval[0]);
                    PUSH(*text, matchEqBig);
                    for (i = 0; i <= n; ++i) {
                        PUSH(*text, (Uint) bval[i]);
                    }
                    break;
                }
                default: /* BINARY, FUN, VECTOR, or EXTERNAL */
                    PUSH2(*text, matchEqBin, private_copy(self, c));
                    break;
                }
                break;
            }
            _ => panic!("match_compile: Bad object on heap: {}", c),
        }

        Ok(())
    }

    fn compile_guard_expr(&self, DMCHeap *heap, STACK_TYPE(UWord) *text, Eterm l) -> DMCRet {
        // DMCRet ret;
        // int constant;
        // Eterm t;

        if l != Term::nil() {
            if !l.is_list(l) {
                RETURN_ERROR("Match expression is not a list.", self, constant);
            }
            if (!(self.is_guard)) {
                PUSH(*text, matchCatch);
            }
            while (is_list(l)) {
                constant = 0;
                t = CAR(list_val(l));
                if ret = expr(self, heap, text, t, &constant)) != retOk {
                    return ret;
                }
                if (constant) {
                    do_emit_constant(self, text, t);
                }
                l = CDR(list_val(l));
                if (self.is_guard) {
                    PUSH(*text,matchTrue);
                } else {
                    PUSH(*text,matchWaste);
                }
                --self.stack_used;
            }
            if l != NIL {
                RETURN_ERROR("Match expression is not a proper list.", self, constant);
            }
            if !self.is_guard && (context.cflags & DCOMP_TABLE) {
                ASSERT(matchWaste == TOP(*text));
                (void) POP(*text);
                PUSH(*text, matchReturn); // Same impact on stack as matchWaste
            }
        }
        Ok(())
    }

    /*
    ** Match guard compilation
    */

    fn do_emit_constant(&self, STACK_TYPE(UWord) *text, Eterm t) {
            int sz;
            ErlHeapFragment *emb;
            Eterm *hp;
            Eterm tmp;

            if (IS_CONST(t)) {
                tmp = t;
            } else {
                sz = my_size_object(t);
                emb = new_message_buffer(sz);
                hp = emb->mem;
                tmp = my_copy_struct(t,&hp,&(emb->off_heap));
                emb->next = self.save;
                self.save = emb;
            }
            PUSH2(*text, matchPushC, (Uint)tmp);
            if (++self.stack_used > self.stack_need)
                self.stack_need = self.stack_used;
    }

    #define RETURN_ERROR_X(VAR, ContextP, ConstantF, String, ARG)            \
        (((ContextP)->err_info != NULL)				         \
        ? ((ConstantF) = 0,						 \
            vadd_err((ContextP)->err_info, dmcError, VAR, String, ARG),  \
            retOk)						                 \
        : retFail)

    #define RETURN_ERROR(String, ContextP, ConstantF) \
        return RETURN_ERROR_X(-1, ContextP, ConstantF, String, 0)

    #define RETURN_VAR_ERROR(String, N, ContextP, ConstantF) \
        return RETURN_ERROR_X(N, ContextP, ConstantF, String, 0)

    #define RETURN_TERM_ERROR(String, T, ContextP, ConstantF) \
        return RETURN_ERROR_X(-1, ContextP, ConstantF, String, T)

    #define WARNING(String, ContextP) \
    add_err((ContextP)->err_info, String, -1, 0UL, dmcWarning)

    #define VAR_WARNING(String, N, ContextP) \
    add_err((ContextP)->err_info, String, N, 0UL, dmcWarning)

    #define TERM_WARNING(String, T, ContextP) \
    add_err((ContextP)->err_info, String, -1, T, dmcWarning)

    static DMCRet list(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        int c1;
        int c2;
        int ret;

        if ((ret = expr(context, heap, text, CAR(list_val(t)), &c1)) != retOk)
            return ret;

        if ((ret = expr(context, heap, text, CDR(list_val(t)), &c2)) != retOk)
            return ret;

        if (c1 && c2) {
            *constant = 1;
            return retOk;
        } 
        *constant = 0;
        if (!c1) {
            /* The CAR is not a constant, so if the CDR is, we just push it,
            otherwise it is already pushed. */
            if (c2)
                do_emit_constant(context, text, CDR(list_val(t)));
            PUSH(*text, matchConsA);
        } else { /* !c2 && c1 */
            do_emit_constant(context, text, CAR(list_val(t)));
            PUSH(*text, matchConsB);
        }
        --self.stack_used; /* Two objects on stack becomes one */
        return retOk;
    }

    static void
    rearrange_constants(&mut self, STACK_TYPE(UWord) *text,
                            int textpos, Eterm *p, Uint nelems)
    {
        STACK_TYPE(UWord) instr_save;
        Uint i;

        INIT_STACK(instr_save);
        while (STACK_NUM(*text) > textpos) {
            PUSH(instr_save, POP(*text));
        }
        for (i = nelems; i--;) {
            do_emit_constant(context, text, p[i]);
        }
        while(!EMPTY(instr_save)) {
            PUSH(*text, POP(instr_save));
        }
        FREE(instr_save);
    }

    static DMCRet
    array(&mut self, DMCHeap *heap, STACK_TYPE(UWord) *text,
            Eterm *p, Uint nelems, int *constant)
    {
        int all_constant = 1;
        int textpos = STACK_NUM(*text);
        Uint i;

        /*
        ** We remember where we started to layout code,
        ** assume all is constant and back up and restart if not so.
        ** The array should be laid out with the last element first,
        ** so we can memcpy it to the eheap.
        */
        for (i = nelems; i--;) {
            DMCRet ret;
            int c;

            ret = expr(context, heap, text, p[i], &c);
            if (ret != retOk) {
                return ret;
            }
            if (!c && all_constant) {
                all_constant = 0;
                if (i < nelems - 1) {
                    rearrange_constants(context, text, textpos,
                                            p + i + 1, nelems - i - 1);
                }
            } else if (c && !all_constant) {
                do_emit_constant(context, text, p[i]);
            }
        }
        *constant = all_constant;
        return retOk;
    }

    static DMCRet
    tuple(&mut self, DMCHeap *heap, STACK_TYPE(UWord) *text,
            Eterm t, int *constant)
    {
        int all_constant;
        Eterm *p = tuple_val(t);
        Uint nelems = arityval(*p);
        DMCRet ret;

        ret = array(context, heap, text, p + 1, nelems, &all_constant);
        if (ret != retOk) {
            return ret;
        }
        if (all_constant) {
            *constant = 1;
            return retOk;
        }
        PUSH2(*text, matchMkTuple, nelems);
        self.stack_used -= (nelems - 1);
        *constant = 0;
        return retOk;
    }

    static DMCRet
    map(&mut self, DMCHeap *heap, STACK_TYPE(UWord) *text,
            Eterm t, int *constant)
    {
        int nelems;
        int constant_values;
        DMCRet ret;
        if (is_flatmap(t)) {
            flatmap_t *m = (flatmap_t *)flatmap_val(t);
            Eterm *values = flatmap_get_values(m);

            nelems = flatmap_get_size(m);
            ret = array(context, heap, text, values, nelems, &constant_values);

            if (ret != retOk) {
                return ret;
            }
            if (constant_values) {
                *constant = 1;
                return retOk;
            }
            PUSH2(*text, matchPushC, private_copy(context, m->keys));
            if (++self.stack_used > self.stack_need) {
                self.stack_need = self.stack_used;
            }
            PUSH2(*text, matchMkFlatMap, nelems);
            self.stack_used -= nelems;
            *constant = 0;
            return retOk;
        } else {
            DECLARE_WSTACK(wstack);
            Eterm *kv;
            int c;

            ASSERT(is_hashmap(t));

            hashmap_iterator_init(&wstack, t, 1);
            constant_values = 1;
            nelems = hashmap_size(t);

            while ((kv=hashmap_iterator_prev(&wstack)) != NULL) {
                if ((ret = expr(context, heap, text, CDR(kv), &c)) != retOk) {
                    DESTROY_WSTACK(wstack);
                    return ret;
                }
                if (!c)
                    constant_values = 0;
            }

            if (constant_values) {
                *constant = 1;
                DESTROY_WSTACK(wstack);
                return retOk;
            }

            *constant = 0;

            hashmap_iterator_init(&wstack, t, 1);

            while ((kv=hashmap_iterator_prev(&wstack)) != NULL) {
                /* push key */
                if ((ret = expr(context, heap, text, CAR(kv), &c)) != retOk) {
                    DESTROY_WSTACK(wstack);
                    return ret;
                }
                if (c) {
                    do_emit_constant(context, text, CAR(kv));
                }
                /* push value */
                if ((ret = expr(context, heap, text, CDR(kv), &c)) != retOk) {
                    DESTROY_WSTACK(wstack);
                    return ret;
                }
                if (c) {
                    do_emit_constant(context, text, CDR(kv));
                }
            }
            PUSH2(*text, matchMkHashMap, nelems);
            self.stack_used -= nelems;
            DESTROY_WSTACK(wstack);
            return retOk;
        }
    }

    static DMCRet whole_expression(&mut self,
                                    DMCHeap *heap,
                                    STACK_TYPE(UWord) *text,
                                    Eterm t,
                                    int *constant)
    {
        if (self.cflags & DCOMP_TRACE) {
            /* Hmmm, convert array to list... */
            if (self.special) {
            PUSH(*text, matchPushArrayAsListU);
            } else { 
                ASSERT(is_tuple(self.matchexpr
                                [self.current_match]));
                PUSH(*text, matchPushArrayAsList);
            }
        } else {
            PUSH(*text, matchPushExpr);
        }
        ++self.stack_used;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        *constant = 0;
        return retOk;
    }

    /* Figure out which PushV instruction to use.
    */
    static void add_pushv_variant(&mut self, DMCHeap *heap,
                                    STACK_TYPE(UWord) *text, Uint n)
    {
        DMCVariable* v = &heap->vars[n];
        MatchOps instr = matchPushV;

        ASSERT(n < heap->vars_used && v->is_bound);
        if (!self.is_guard) {
            if(!v->is_in_body) {
                instr = matchPushVResult;
                v->is_in_body = 1;
            }
        }
        PUSH(*text, instr);
        PUSH(*text, n);
    }

    static DMCRet variable(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Uint n = db_is_variable(t);

        if (n >= heap->vars_used || !heap->vars[n].is_bound) {
            RETURN_VAR_ERROR("Variable $%%d is unbound.", n, context, *constant);
        }

        add_pushv_variant(context, heap, text, n);

        ++self.stack_used;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        *constant = 0;
        return retOk;
    }

    static DMCRet all_bindings(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        int i;
        int heap_used = 0;

        PUSH(*text, matchPushC);
        PUSH(*text, NIL);
        for (i = heap->vars_used - 1; i >= 0; --i) {
            if (heap->vars[i].is_bound) {
                add_pushv_variant(context, heap, text, i);
                PUSH(*text, matchConsB);
                heap_used += 2;
            }
        }
        ++self.stack_used;
        if ((self.stack_used + 1) > self.stack_need)
            self.stack_need = (self.stack_used + 1);
        *constant = 0;
        return retOk;
    }

    static DMCRet const(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);

        if (a != 2) {
            RETURN_TERM_ERROR("Special form 'const' called with more than one "
                            "argument in %T.", t, context, *constant);
        }
        *constant = 1;
        return retOk;
    }

    static DMCRet and(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        
        if (a < 2) {
            RETURN_TERM_ERROR("Special form 'and' called without arguments "
                            "in %T.", t, context, *constant);
        }
        *constant = 0;
        for (i = a; i > 1; --i) {
            if ((ret = expr(context, heap, text, p[i], &c)) != retOk)
                return ret;
            if (c) 
                do_emit_constant(context, text, p[i]);
        }
        PUSH(*text, matchAnd);
        PUSH(*text, (Uint) a - 1);
        self.stack_used -= (a - 2);
        return retOk;
    }

    static DMCRet or(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        
        if (a < 2) {
            RETURN_TERM_ERROR("Special form 'or' called without arguments "
                            "in %T.", t, context, *constant);
        }
        *constant = 0;
        for (i = a; i > 1; --i) {
            if ((ret = expr(context, heap, text, p[i], &c)) != retOk)
                return ret;
            if (c) 
                do_emit_constant(context, text, p[i]);
        }
        PUSH(*text, matchOr);
        PUSH(*text, (Uint) a - 1);
        self.stack_used -= (a - 2);
        return retOk;
    }


    static DMCRet andalso(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;

        if (a < 2) {
            RETURN_TERM_ERROR("Special form 'andalso' called without"
                            " arguments "
                            "in %T.", t, context, *constant);
        }
        *constant = 0;
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            if ((ret = expr(context, heap, text, p[i], &c)) != retOk)
                return ret;
            if (c) 
                do_emit_constant(context, text, p[i]);
            if (i == a) {
                PUSH(*text, matchJump);
            } else {
                PUSH(*text, matchAndAlso);
            }
            PUSH(*text, lbl);
            lbl = STACK_NUM(*text)-1;
            --(self.stack_used);
        }
        PUSH(*text, matchPushC);
        PUSH(*text, am_true);
        lbl_val = STACK_NUM(*text);
        while (lbl) {
            lbl_next = PEEK(*text, lbl);
            POKE(*text, lbl, lbl_val-lbl-1);
            lbl = lbl_next;
        }
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet orelse(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;
        
        if (a < 2) {
            RETURN_TERM_ERROR("Special form 'orelse' called without arguments "
                            "in %T.", t, context, *constant);
        }
        *constant = 0;
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            if ((ret = expr(context, heap, text, p[i], &c)) != retOk)
                return ret;
            if (c) 
                do_emit_constant(context, text, p[i]);
            if (i == a) {
                PUSH(*text, matchJump);
            } else {
                PUSH(*text, matchOrElse);
            }
            PUSH(*text, lbl);
            lbl = STACK_NUM(*text)-1;
            --(self.stack_used);
        }
        PUSH(*text, matchPushC);
        PUSH(*text, am_false);
        lbl_val = STACK_NUM(*text);
        while (lbl) {
            lbl_next = PEEK(*text, lbl);
            POKE(*text, lbl, lbl_val-lbl-1);
            lbl = lbl_next;
        }
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet message(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        

        if (!(self.cflags & DCOMP_TRACE)) {
            RETURN_ERROR("Special form 'message' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if (self.is_guard) {
            RETURN_ERROR("Special form 'message' called in guard context.",
                        context, 
                        *constant);
        }

        if (a != 2) {
            RETURN_TERM_ERROR("Special form 'message' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        *constant = 0;
        if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
            return ret;
        }
        if (c) { 
            do_emit_constant(context, text, p[2]);
        }
        PUSH(*text, matchReturn);
        PUSH(*text, matchPushC);
        PUSH(*text, am_true);
        /* Push as much as we remove, stack_need is untouched */
        return retOk;
    }

    static DMCRet self(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'self' called with arguments "
                            "in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchSelf);
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet return_trace(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if (!(self.cflags & DCOMP_TRACE)) {
            RETURN_ERROR("Special form 'return_trace' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if (self.is_guard) {
            RETURN_ERROR("Special form 'return_trace' called in "
                        "guard context.", context, *constant);
        }

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'return_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchSetReturnTrace); /* Pushes 'true' on the stack */
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet exception_trace(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if (!(self.cflags & DCOMP_TRACE)) {
            RETURN_ERROR("Special form 'exception_trace' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if (self.is_guard) {
            RETURN_ERROR("Special form 'exception_trace' called in "
                        "guard context.", context, *constant);
        }

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'exception_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchSetExceptionTrace); /* Pushes 'true' on the stack */
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static int check_trace(const char* op,
                        DMCContext *context,
                        int *constant,
                        int need_cflags,
                        int allow_in_guard,
                        DMCRet* retp)
    {
        if (!(self.cflags & DCOMP_TRACE)) {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "used in wrong dialect.", op);
            return 0;
        }
        if ((self.cflags & need_cflags) != need_cflags) {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "not allow for this trace event.", op);
            return 0;
        }
        if (self.is_guard && !allow_in_guard) {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "called in guard context.", op);
            return 0;
        }
        return 1;
    }

    static DMCRet is_seq_trace(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!check_trace("is_seq_trace", context, constant, DCOMP_ALLOW_TRACE_OPS, 1, &ret))
            return ret;

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'is_seq_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchIsSeqTrace); 
        /* Pushes 'true' or 'false' on the stack */
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet set_seq_token(&mut self,
                                    DMCHeap *heap,
                                    STACK_TYPE(UWord) *text,
                                    Eterm t,
                                    int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        
        if (!check_trace("set_seq_trace", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if (a != 3) {
            RETURN_TERM_ERROR("Special form 'set_seq_token' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        *constant = 0;
        if ((ret = expr(context, heap, text, p[3], &c)) != retOk) {
            return ret;
        }
        if (c) { 
            do_emit_constant(context, text, p[3]);
        }
        if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
            return ret;
        }
        if (c) { 
            do_emit_constant(context, text, p[2]);
        }
        if (self.cflags & DCOMP_FAKE_DESTRUCTIVE) {
            PUSH(*text, matchSetSeqTokenFake);
        } else {
            PUSH(*text, matchSetSeqToken);
        }
        --self.stack_used; /* Remove two and add one */
        return retOk;
    }

    static DMCRet get_seq_token(&mut self,
                                    DMCHeap *heap,
                                    STACK_TYPE(UWord) *text,
                                    Eterm t,
                                    int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if (!check_trace("get_seq_token", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'get_seq_token' called with "
                            "arguments in %T.", t, context, 
                            *constant);
        }

        *constant = 0;
        PUSH(*text, matchGetSeqToken);
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }



    static DMCRet display(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        

        if (!(self.cflags & DCOMP_TRACE)) {
            RETURN_ERROR("Special form 'display' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if (self.is_guard) {
            RETURN_ERROR("Special form 'display' called in guard context.",
                        context, 
                        *constant);
        }

        if (a != 2) {
            RETURN_TERM_ERROR("Special form 'display' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        *constant = 0;
        if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
            return ret;
        }
        if (c) { 
            do_emit_constant(context, text, p[2]);
        }
        PUSH(*text, matchDisplay);
        /* Push as much as we remove, stack_need is untouched */
        return retOk;
    }

    static DMCRet process_dump(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if (!check_trace("process_dump", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'process_dump' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchProcessDump); /* Creates binary */
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    static DMCRet enable_trace(&mut self,
                                DMCHeap *heap,
                                STACK_TYPE(UWord) *text,
                                Eterm t,
                                int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        
        if (!check_trace("enable_trace", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 2:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchEnableTrace);
            /* Push as much as we remove, stack_need is untouched */
            break;
        case 3:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[3], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[3]);
            }
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchEnableTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'enable_trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        return retOk;
    }

    static DMCRet disable_trace(&mut self,
                                    DMCHeap *heap,
                                    STACK_TYPE(UWord) *text,
                                    Eterm t,
                                    int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;

        if (!check_trace("disable_trace", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 2:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchDisableTrace);
            /* Push as much as we remove, stack_need is untouched */
            break;
        case 3:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[3], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[3]);
            }
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchDisableTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'disable_trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        return retOk;
    }

    static DMCRet trace(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        
        if (!check_trace("trace", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 3:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[3], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[3]);
            }
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        case 4:
            *constant = 0;
            if ((ret = expr(context, heap, text, p[4], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[4]);
            }
            if ((ret = expr(context, heap, text, p[3], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[3]);
            }
            if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
                return ret;
            }
            if (c) { 
                do_emit_constant(context, text, p[2]);
            }
            PUSH(*text, matchTrace3);
            self.stack_used -= 2; /* Remove three and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        return retOk;
    }



    static DMCRet caller(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!check_trace("caller", context, constant,
                        (DCOMP_CALL_TRACE|DCOMP_ALLOW_TRACE_OPS), 0, &ret))
            return ret;
    
        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'caller' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = 0;
        PUSH(*text, matchCaller); /* Creates binary */
        if (++self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }


    
    static DMCRet silent(&mut self,
                            DMCHeap *heap,
                            STACK_TYPE(UWord) *text,
                            Eterm t,
                            int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int c;
        
        if (!check_trace("silent", context, constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;
    
        if (a != 2) {
            RETURN_TERM_ERROR("Special form 'silent' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        *constant = 0;
        if ((ret = expr(context, heap, text, p[2], &c)) != retOk) {
            return ret;
        }
        if (c) { 
            do_emit_constant(context, text, p[2]);
        }
        PUSH(*text, matchSilent);
        PUSH(*text, matchPushC);
        PUSH(*text, am_true);
        /* Push as much as we remove, stack_need is untouched */
        return retOk;
    }
    


    static DMCRet fun(&mut self,
                        DMCHeap *heap,
                        STACK_TYPE(UWord) *text,
                        Eterm t,
                        int *constant)
    {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        int c;
        int i;
        DMCRet ret;
        DMCGuardBif *b;
    
        /* Special forms. */
        switch (p[1]) {
        case am_const:
            return const(context, heap, text, t, constant);
        case am_and:
            return and(context, heap, text, t, constant);
        case am_or:
            return or(context, heap, text, t, constant);
        case am_andalso:
        case am_andthen:
            return andalso(context, heap, text, t, constant);
        case am_orelse:
            return orelse(context, heap, text, t, constant);
        case am_self:
            return self(context, heap, text, t, constant);
        case am_message:
            return message(context, heap, text, t, constant);
        case am_is_seq_trace:
            return is_seq_trace(context, heap, text, t, constant);
        case am_set_seq_token:
            return set_seq_token(context, heap, text, t, constant);
        case am_get_seq_token:
            return get_seq_token(context, heap, text, t, constant);
        case am_return_trace:
            return return_trace(context, heap, text, t, constant);
        case am_exception_trace:
            return exception_trace(context, heap, text, t, constant);
        case am_display:
            return display(context, heap, text, t, constant);
        case am_process_dump:
            return process_dump(context, heap, text, t, constant);
        case am_enable_trace:
            return enable_trace(context, heap, text, t, constant);
        case am_disable_trace:
            return disable_trace(context, heap, text, t, constant);
        case am_trace:
            return trace(context, heap, text, t, constant);
        case am_caller:
            return caller(context, heap, text, t, constant);
        case am_silent:
            return silent(context, heap, text, t, constant);
        case am_set_tcw:
            if (self.cflags & DCOMP_FAKE_DESTRUCTIVE) {
                b = lookup_bif(am_set_tcw_fake, ((int) a) - 1);
            } else {
                b = lookup_bif(p[1], ((int) a) - 1);
            }
            break;
        default:
            b = lookup_bif(p[1], ((int) a) - 1);
        }


        if (b == NULL) {
            if (self.err_info != NULL) {
                /* Ugly, should define a better RETURN_TERM_ERROR interface... */
                char buff[100];
                erts_snprintf(buff, sizeof(buff),
                        "Function %%T/%d does_not_exist.",
                        (int)a - 1);
                RETURN_TERM_ERROR(buff, p[1], context, *constant);
            } else {
                return retFail;
            }
        } 
        ASSERT(b->arity == ((int) a) - 1);
        if (! (b->flags & 
            (1 << 
                ((self.cflags & DCOMP_DIALECT_MASK) + 
                (self.is_guard ? DBIF_GUARD : DBIF_BODY))))) {
            /* Body clause used in wrong context. */
            if (self.err_info != NULL) {
                /* Ugly, should define a better RETURN_TERM_ERROR interface... */
                char buff[100];
                erts_snprintf(buff, sizeof(buff),
                        "Function %%T/%d cannot be called in this context.",
                        (int)a - 1);
                RETURN_TERM_ERROR(buff, p[1], context, *constant);
            } else {
                return retFail;
            }
        }	

        *constant = 0;

        for (i = a; i > 1; --i) {
            if ((ret = expr(context, heap, text, p[i], &c)) != retOk)
                return ret;
            if (c) 
                do_emit_constant(context, text, p[i]);
        }
        switch (b->arity) {
        case 0:
            PUSH(*text, matchCall0);
            break;
        case 1:
            PUSH(*text, matchCall1);
            break;
        case 2:
            PUSH(*text, matchCall2);
            break;
        case 3:
            PUSH(*text, matchCall3);
            break;
        default:
            erts_exit(ERTS_ERROR_EXIT,"ets:match() internal error, "
                    "guard with more than 3 arguments.");
        }
        PUSH(*text, (UWord) b->biff);
        self.stack_used -= (((int) a) - 2);
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        return retOk;
    }

    
    fn expr(&mut self, DMCHeap *heap, STACK_TYPE(UWord) *text, Eterm t, int *constant) -> DMCRet {
        DMCRet ret;
        Eterm tmp;
        Eterm *p;

        switch (t & _TAG_PRIMARY_MASK) {
            case TAG_PRIMARY_LIST:
                if ((ret = list(context, heap, text, t, constant)) != retOk)
                    return ret;
                break;
            case TAG_PRIMARY_BOXED:
                if (is_map(t)) {
                    return map(context, heap, text, t, constant);
                }
                if (!is_tuple(t)) {
                    goto simple_term;
                }
                p = tuple_val(t);
#ifdef HARDDEBUG
                erts_fprintf(stderr,"%d %d %d %d\n",arityval(*p),is_tuple(tmp = p[1]),
                is_atom(p[1]),db_is_variable(p[1]));
#endif
                if (arityval(*p) == 1 && is_tuple(tmp = p[1])) {
                    if ((ret = tuple(context, heap, text, tmp, constant)) != retOk)
                        return ret;
                } else if (arityval(*p) >= 1 && is_atom(p[1]) && 
                           !(db_is_variable(p[1]) >= 0)) {
                    if ((ret = fun(context, heap, text, t, constant)) != retOk)
                        return ret;
                } else
                    RETURN_TERM_ERROR("%T is neither a function call, nor a tuple "
                                      "(tuples are written {{ ... }}).", t,
                                      context, *constant);
                break;
            case TAG_PRIMARY_IMMED1:
                if (db_is_variable(t) >= 0) {
                    if ((ret = variable(context, heap, text, t, constant)) 
                        != retOk)
                        return ret;
                    break;
                } else if (t == am_DollarUnderscore) {
                    if ((ret = whole_expression(context, heap, text, t, constant)) 
                        != retOk)
                        return ret;
                    break;
                } else if (t == am_DollarDollar) {
                    if ((ret = all_bindings(context, heap, text, t, constant)) 
                        != retOk)
                        return ret;
                    break;
                }	    
                /* Fall through */
            default:
                simple_term:
                    *constant = 1;
        }
        return retOk;
    }


}

