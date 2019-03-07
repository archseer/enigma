//! Pattern matching abstract machine (PAM)
use super::*;

use crate::value::{Variant};

struct Pattern {}

/// bool tells us if is_constant
type DMCRet = Result<bool, Error>;

pub(crate) struct Compiler {
    matchexpr: Vec<Term>,
    guardexpr: Vec<Term>,
    bodyexpr: Vec<Term>,
    text: Vec<usize>,
    cflags: usize,
    stack_used: usize,
    stack_need: usize,
}

impl Compiler {
    pub(crate) fn new(matchexpr: Vec<Term>, guardexpr: Vec<Term>, bodyexpr: Vec<Term>, num_match: usize, cflags: usize) -> Self {
        Self {
            text: Vec::new(),
            heap: NULL,
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
        // MatchProg *ret = NULL;
        // Eterm t;
        // Uint i;
        // Uint num_iters;
        // int structure_checked;
        // DMCRet res;
        let mut current_try_label = -1;
        // Binary *bp = NULL;

        // STACK_TYPE(Eterm) stack;
        let stack = Vec::new();

        self.heap.size = DEFAULT_SIZE;
        self.heap.vars = self.heap.vars_def;

        // Compile the match expression.
        self.heap.vars_used = 0;

        for i in 0..self.num_match { // long loop ahead
            // sys_memset(self.heap.vars, 0, self.heap.size * sizeof(*self.heap.vars));
            self.current_match = i;
            let mut t = self.matchexpr[self.current_match];
            self.stack_used = 0;
            let structure_checked = false;

            if self.current_match < self.num_match - 1 {
                self.text.push(Opcode::MatchTryMeElse);
                current_try_label = self.text.len() - 1;
                self.text.push(0);
            } else {
                current_try_label = -1;
            }

            let clause_start = self.text.len() - 1; // the "special" test needs it
            loop {
                match t.into_variant() {
                    Variant::Pointer(..) => {
                        match self.get_boxed_header().unwrap() {
                            BOXED_MAP => {
                                DECLARE_WSTACK(wstack);
                                Eterm *kv;
                                num_iters = hashmap_size(t);
                                if (!structure_checked) {
                                    PUSH2(self.text, matchMap, num_iters);
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
                                    PUSH2(self.text, matchKey, private_copy(&self, key));
                                    {
                                        int old_stack = ++(self.stack_used);
                                        res = self.one_term(&stack, value);
                                        ASSERT(res != retFail);
                                        if (old_stack != self.stack_used) {
                                            ASSERT(old_stack + 1 == self.stack_used);
                                            self.text.push(matchSwap);
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
                                    self.text.push(Opcode::MatchTuple(t.len()))
                                }
                                structure_checked = false;
                                for val in t {
                                    if (res = self.one_term(&stack, val)) != retOk {
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
                            self.text.push(Opcode::MatchList);
                        }
                        structure_checked = false; // Whatever it is, we did not pop it
                        if self.one_term(&stack, CAR(list_val(t))) != retOk {
                            return Err(());
                        }
                        t = CDR(list_val(t));
                        continue;
                    }
                    _ =>  {// Nil and non proper tail end's or single terms as match expressions.
                    //simple_term:
                        structure_checked = false;
                        if self.one_term(&stack, t) != retOk {
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
                    self.text.push(Opcode::MatchPop);
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
            if compile_guard_expr(&self, self.guardexpr[self.current_match]) != retOk {
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
            if compile_guard_expr(&self, self.bodyexpr[self.current_match]) != retOk {
                return Err(());
            }

            // The compilation does not bail out when error information is requested, so we need to
            // detect that here...
            if self.err_info != NULL && self.err_info.error_added {
                return Err(());
            }


            // If the matchprogram comes here, the match is successful
            self.text.push(Opcode::MatchHalt);
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
    fn one_term(&mut self, STACK_TYPE(Eterm) *stack, Eterm c) -> DMCRet {
        // Sint n;
        // Eterm *hp;
        // Uint sz, sz2, sz3;
        // Uint i, j;

        switch (c & _TAG_PRIMARY_MASK) {
            case TAG_PRIMARY_IMMED1:
                if (n = db_is_variable(c)) >= 0 { /* variable */
                    if self.heap.vars[n].is_bound {
                        self.text.push(matchCmp(n))
                    } else { /* Not bound, bind! */
                        if n >= self.heap.vars_used {
                            self.heap.vars_used = n + 1;
                        }
                        self.text.push(matchBind(n))
                        self.heap.vars[n].is_bound = true;
                    }
                } else if c == atom!(UNDERSCORE) {
                    self.text.push(Opcode::MatchSkip);
                } else {
                    // Any immediate value
                    self.text.push(Opcode::MatchEq(c as usize));
                }
                break;
            case TAG_PRIMARY_LIST:
                self.text.push(matchPushL);
                ++(self.stack_used);
                PUSH(*stack, c);
                break;
            case TAG_HEADER_FLOAT:
                PUSH2(*self.text, matchEqFloat, (Uint) float_val(c)[1]);
        #ifdef ARCH_64
                PUSH(*self.text, (Uint) 0);
        #else
                PUSH(*self.text, (Uint) float_val(c)[2]);
        #endif
                break;
            case TAG_PRIMARY_BOXED: {
                Eterm hdr = *boxed_val(c);
                switch ((hdr & _TAG_HEADER_MASK) >> _TAG_PRIMARY_SIZE) {
                case (_TAG_HEADER_ARITYVAL >> _TAG_PRIMARY_SIZE):
                    n = arityval(*tuple_val(c));
                    self.text.push(matchPushT(n))
                    self.stack_used += 1;
                    PUSH(*stack, c);
                    break;
                case (_TAG_HEADER_MAP >> _TAG_PRIMARY_SIZE):
                    if (is_flatmap(c))
                        n = flatmap_get_size(flatmap_val(c));
                    else
                        n = hashmap_size(c);
                    self.text.push(matchPushM(n))
                    self.stack_used += 1;
                    PUSH(*stack, c);
                    break;
                case (_TAG_HEADER_REF >> _TAG_PRIMARY_SIZE):
                {
                    Eterm* ref_val = internal_ref_val(c);
                    self.text.push(matchEqRef);
                    n = thing_arityval(ref_val[0]);
                    for (i = 0; i <= n; ++i) {
                        PUSH(*self.text, ref_val[i]);
                    }
                    break;
                }
                case (_TAG_HEADER_POS_BIG >> _TAG_PRIMARY_SIZE):
                case (_TAG_HEADER_NEG_BIG >> _TAG_PRIMARY_SIZE):
                {
                    Eterm* bval = big_val(c);
                    n = thing_arityval(bval[0]);
                    self.text.push(matchEqBig);
                    for (i = 0; i <= n; ++i) {
                        PUSH(*self.text, (Uint) bval[i]);
                    }
                    break;
                }
                default: /* BINARY, FUN, VECTOR, or EXTERNAL */
                    PUSH2(*self.text, matchEqBin, private_copy(self, c));
                    break;
                }
                break;
            }
            _ => panic!("match_compile: Bad object on heap: {}", c),
        }

        Ok(())
    }

    fn compile_guard_expr(&self, l: Term) -> DMCRet {
        // DMCRet ret;
        // int constant;
        // Eterm t;

        if l != Term::nil() {
            if !l.is_list(l) {
                RETURN_ERROR("Match expression is not a list.", self, constant);
            }
            if !self.is_guard {
                self.text.push(matchCatch);
            }
            while is_list(l) {
                constant = 0;
                t = CAR(list_val(l));
                let constant = expr(self, t)?;
                if constant {
                    do_emit_constant(self, t);
                }
                l = CDR(list_val(l));
                if self.is_guard {
                    self.text.push(matchTrue);
                } else {
                    self.text.push(matchWaste);
                }
                --self.stack_used;
            }
            if l != NIL {
                RETURN_ERROR("Match expression is not a proper list.", self, constant);
            }
            if !self.is_guard && (context.cflags & DCOMP_TABLE) {
                ASSERT(matchWaste == TOP(*text));
                (void) POP(*text);
                self.text.push(matchReturn); // Same impact on stack as matchWaste
            }
        }
        Ok(())
    }

    /*
    ** Match guard compilation
    */

    fn do_emit_constant(&self, t: Term) {
        int sz;
        ErlHeapFragment *emb;
        Eterm *hp;
        Eterm tmp;

        if IS_CONST(t) {
            tmp = t;
        } else {
            sz = my_size_object(t);
            emb = new_message_buffer(sz);
            hp = emb->mem;
            tmp = my_copy_struct(t,&hp,&(emb->off_heap));
            emb->next = self.save;
            self.save = emb;
        }
        PUSH2(*self.text, matchPushC, (Uint)tmp);
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
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

    fn list(&mut self, t: Term) -> DMCRet {
        let c1 = self.expr(CAR(list_val(t)))?;
        let c2 = self.expr(CDR(list_val(t)))?;

        if c1 && c2 {
            return Ok(true);
        } 
        if !c1 {
            /* The CAR is not a constant, so if the CDR is, we just push it,
            otherwise it is already pushed. */
            if c2 {
                self.do_emit_constant(CDR(list_val(t)));
            }
            self.text.push(matchConsA);
        } else { /* !c2 && c1 */
            self.do_emit_constant(CAR(list_val(t)));
            self.text.push(matchConsB);
        }
        --self.stack_used; /* Two objects on stack becomes one */
        Ok(false)
    }

    fn rearrange_constants(&mut self, int textpos, Eterm *p, nelems: usize) {
        STACK_TYPE(UWord) instr_save;
        Uint i;

        INIT_STACK(instr_save);
        while STACK_NUM(*text) > textpos {
            PUSH(instr_save, POP(*text));
        }
        for (i = nelems; i--;) {
            self.do_emit_constant(p[i]);
        }
        while(!EMPTY(instr_save)) {
            PUSH(*text, POP(instr_save));
        }
        FREE(instr_save);
    }

    fn array(&mut self, Eterm *p, nelems: usize) -> DMCRet {
        let mut all_constant = true;
        int textpos = STACK_NUM(*text);
        Uint i;

        /*
        ** We remember where we started to layout code,
        ** assume all is constant and back up and restart if not so.
        ** The array should be laid out with the last element first,
        ** so we can memcpy it to the eheap.
        */
        for (i = nelems; i--;) {
            let res = self.expr(p[i])?;
            if !res && all_constant {
                all_constant = false;
                if i < nelems - 1 {
                    self.rearrange_constants(textpos,
                                            p + i + 1, nelems - i - 1);
                }
            } else if res && !all_constant {
                self.do_emit_constant(p[i]);
            }
        }
        Ok(all_constant)
    }

    fn tuple(&mut self, t: Term) -> DMCRet {
        int all_constant;
        Eterm *p = tuple_val(t);
        Uint nelems = arityval(*p);

        let all_constant = self.array(p + 1, nelems)?;
        if all_constant {
            return Ok(true);
        }
        self.text.push(matchMkTuple(nelems))
        self.stack_used -= (nelems - 1);
        Ok(false)
    }

    fn map(&mut self, t: Term) -> DMCRet {
        int nelems;
        if is_flatmap(t) {
            flatmap_t *m = (flatmap_t *)flatmap_val(t);
            Eterm *values = flatmap_get_values(m);

            nelems = flatmap_get_size(m);
            let constant_values = self.array(values, nelems)?;

            if constant_values {
                return Ok(true);
            }
            PUSH2(*text, matchPushC, self.private_copy(m->keys));
            self.stack_used += 1;
            if self.stack_used > self.stack_need {
                self.stack_need = self.stack_used;
            }
            self.text.push(matchMkFlatMap(nelems))
            self.stack_used -= nelems;
            Ok(false)
        } else {
            DECLARE_WSTACK(wstack);
            Eterm *kv;
            int c;

            assert!(is_hashmap(t));

            hashmap_iterator_init(&wstack, t, 1);
            let mut constant_values = true;
            nelems = hashmap_size(t);

            while ((kv=hashmap_iterator_prev(&wstack)) != NULL) {
                let c = self.expr(CDR(kv))?;
                if !c {
                    constant_values = false;
                }
            }

            if constant_values {
                return Ok(true);
            }

            // not constant

            hashmap_iterator_init(&wstack, t, 1);

            while (kv=hashmap_iterator_prev(&wstack)) != NULL {
                /* push key */
                let c = self.expr(CAR(kv))?;
                if c {
                    self.do_emit_constant(CAR(kv));
                }
                /* push value */
                let c = self.expr(CDR(kv))?;
                if c {
                    self.do_emit_constant(CDR(kv));
                }
            }
            self.text.push(matchMkHashMap(nelems))
            self.stack_used -= nelems;
            DESTROY_WSTACK(wstack);
            Ok(false)
        }
    }

    fn whole_expression(&mut self, t: Term) -> DMCRet {
        if self.cflags & DCOMP_TRACE {
            /* Hmmm, convert array to list... */
            if self.special {
                self.text.push(matchPushArrayAsListU);
            } else { 
                assert!(is_tuple(self.matchexpr [self.current_match]));
                self.text.push(matchPushArrayAsList);
            }
        } else {
            self.text.push(matchPushExpr);
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    /// Figure out which PushV instruction to use.
    fn add_pushv_variant(&mut self, Uint n) {
        DMCVariable* v = &self.heap.vars[n];
        MatchOps instr = matchPushV;

        assert!(n < self.heap.vars_used && v->is_bound);
        if !self.is_guard {
            if !v->is_in_body {
                instr = matchPushVResult;
                v->is_in_body = 1;
            }
        }
        self.text.push(instr);
        self.text.push(n);
    }

    fn variable(&mut self, t: Term) -> DMCRet {
        Uint n = db_is_variable(t);

        if n >= self.heap.vars_used || !self.heap.vars[n].is_bound {
            RETURN_VAR_ERROR("Variable $%%d is unbound.", n, context);
        }

        self.add_pushv_variant(n);

        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn all_bindings(&mut self, t: Term) -> DMCRet {
        int i;
        int heap_used = 0;

        self.text.push(matchPushC);
        self.text.push(NIL);
        for (i = self.heap.vars_used - 1; i >= 0; --i) {
            if self.heap.vars[i].is_bound {
                self.add_pushv_variant(i);
                self.text.push(matchConsB);
                heap_used += 2;
            }
        }
        self.stack_used += 1;
        if (self.stack_used + 1) > self.stack_need  {
            self.stack_need = (self.stack_used + 1);
        }
        Ok(false)
    }

    fn const(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);

        if a != 2 {
            RETURN_TERM_ERROR("Special form 'const' called with more than one "
                            "argument in %T.", t, context);
        }
        Ok(true)
    }

    fn and(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        
        if a < 2 {
            RETURN_TERM_ERROR("Special form 'and' called without arguments "
                            "in %T.", t, context);
        }
        for (i = a; i > 1; --i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
        }
        self.text.push(matchAnd);
        PUSH(*text, (Uint) a - 1);
        self.stack_used -= (a - 2);
        Ok(false)
    }

    fn or(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        
        if a < 2 {
            RETURN_TERM_ERROR("Special form 'or' called without arguments "
                            "in %T.", t, context, *constant);
        }
        for (i = a; i > 1; --i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
        }
        self.text.push(matchOr);
        PUSH(*text, (Uint) a - 1);
        self.stack_used -= (a - 2);
        Ok(false)
    }


    fn andalso(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;

        if a < 2 {
            RETURN_TERM_ERROR("Special form 'andalso' called without"
                            " arguments "
                            "in %T.", t, context);
        }
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
            if i == a {
                self.text.push(matchJump);
            } else {
                self.text.push(matchAndAlso);
            }
            self.text.push(lbl);
            lbl = STACK_NUM(*text)-1;
            --(self.stack_used);
        }
        self.text.push(matchPushC);
        self.text.push(am_true);
        lbl_val = STACK_NUM(*text);
        while (lbl) {
            lbl_next = PEEK(*text, lbl);
            POKE(*text, lbl, lbl_val-lbl-1);
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn orelse(&mut self, t: Term, constant: bool) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;
        
        if a < 2 {
            RETURN_TERM_ERROR("Special form 'orelse' called without arguments "
                            "in %T.", t, context);
        }
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
            if (i == a) {
                self.text.push(matchJump);
            } else {
                self.text.push(matchOrElse);
            }
            self.text.push(lbl);
            lbl = STACK_NUM(*text)-1;
            --(self.stack_used);
        }
        self.text.push(matchPushC);
        self.text.push(am_false);
        lbl_val = STACK_NUM(*text);
        while lbl {
            lbl_next = PEEK(*text, lbl);
            POKE(*text, lbl, lbl_val-lbl-1);
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn message(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if !(self.cflags & DCOMP_TRACE) {
            RETURN_ERROR("Special form 'message' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if self.is_guard {
            RETURN_ERROR("Special form 'message' called in guard context.",
                        context, 
                        *constant);
        }

        if a != 2 {
            RETURN_TERM_ERROR("Special form 'message' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        self.text.push(matchReturn);
        self.text.push(matchPushC);
        self.text.push(am_true);
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn self(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if a != 1 {
            RETURN_TERM_ERROR("Special form 'self' called with arguments "
                            "in %T.", t, context, *constant);
        }
        self.text.push(matchSelf);
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn return_trace(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if !(self.cflags & DCOMP_TRACE) {
            RETURN_ERROR("Special form 'return_trace' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if self.is_guard {
            RETURN_ERROR("Special form 'return_trace' called in "
                        "guard context.", context, *constant);
        }

        if a != 1 {
            RETURN_TERM_ERROR("Special form 'return_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        self.text.push(matchSetReturnTrace); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn exception_trace(&mut self, Eterm t) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        
        if !(self.cflags & DCOMP_TRACE) {
            RETURN_ERROR("Special form 'exception_trace' used in wrong dialect.",
                        context, 
                        *constant);
        }
        if self.is_guard {
            RETURN_ERROR("Special form 'exception_trace' called in "
                        "guard context.", context, *constant);
        }

        if a != 1 {
            RETURN_TERM_ERROR("Special form 'exception_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        self.text.push(matchSetExceptionTrace); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn check_trace(&self, const char* op, constant: bool, int need_cflags, int allow_in_guard, DMCRet* retp) -> bool {
        if !(self.cflags & DCOMP_TRACE) {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "used in wrong dialect.", op);
            return false;
        }
        if (self.cflags & need_cflags) != need_cflags {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "not allow for this trace event.", op);
            return false;
        }
        if self.is_guard && !allow_in_guard {
            *retp = RETURN_ERROR_X(-1, context, *constant, "Special form '%s' "
                                "called in guard context.", op);
            return false;
        }
        true
    }

    fn is_seq_trace(&mut self, t: Term, constant: bool) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!self.check_trace("is_seq_trace", constant, DCOMP_ALLOW_TRACE_OPS, 1, &ret))
            return ret;

        if a != 1 {
            RETURN_TERM_ERROR("Special form 'is_seq_trace' called with "
                            "arguments in %T.", t, context, *constant);
        }
        *constant = false;
        self.text.push(matchIsSeqTrace); 
        /* Pushes 'true' or 'false' on the stack */
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(())
    }

    fn set_seq_token(&mut self, Eterm t) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!self.check_trace("set_seq_trace", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if a != 3 {
            RETURN_TERM_ERROR("Special form 'set_seq_token' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        let c = self.expr(p[3])?;
        if c { 
            self.do_emit_constant(p[3]);
        }
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        if (self.cflags & DCOMP_FAKE_DESTRUCTIVE) {
            self.text.push(matchSetSeqTokenFake);
        } else {
            self.text.push(matchSetSeqToken);
        }
        --self.stack_used; /* Remove two and add one */
        Ok(false)
    }

    fn get_seq_token(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if (!self.check_trace("get_seq_token", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'get_seq_token' called with "
                            "arguments in %T.", t, context, 
                            *constant);
        }

        self.text.push(matchGetSeqToken);
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn display(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        

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
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        self.text.push(matchDisplay);
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn process_dump(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if (!self.check_trace("process_dump", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        if a != 1 {
            RETURN_TERM_ERROR("Special form 'process_dump' called with "
                            "arguments in %T.", t, context, *constant);
        }
        self.text.push(matchProcessDump); /* Creates binary */
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    fn enable_trace(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!self.check_trace("enable_trace", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 2:
            *constant = false;
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchEnableTrace);
            /* Push as much as we remove, stack_need is untouched */
            break;
        case 3:
            let c = self.expr(p[3])?;
            if c { 
                self.do_emit_constant(p[3]);
            }
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchEnableTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'enable_trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        Ok(false)
    }

    fn disable_trace(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;

        if (!self.check_trace("disable_trace", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 2:
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchDisableTrace);
            /* Push as much as we remove, stack_need is untouched */
            break;
        case 3:
            let c = self.expr(p[3])?;
            if c { 
                self.do_emit_constant(p[3]);
            }
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchDisableTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'disable_trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        Ok(false)
    }

    fn trace(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!self.check_trace("trace", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;

        switch (a) {
        case 3:
            let c = self.expr(p[3])?;
            if c { 
                self.do_emit_constant(p[3]);
            }
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchTrace2);
            --self.stack_used; /* Remove two and add one */
            break;
        case 4:
            let c = self.expr(p[4])?;
            if c { 
                self.do_emit_constant(p[4]);
            }
            let c = self.expr(p[3])?;
            if c { 
                self.do_emit_constant(p[3]);
            }
            let c = self.expr(p[2])?;
            if c { 
                self.do_emit_constant(p[2]);
            }
            self.text.push(matchTrace3);
            self.stack_used -= 2; /* Remove three and add one */
            break;
        default:
            RETURN_TERM_ERROR("Special form 'trace' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        Ok(false)
    }



    fn caller(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
        
        if (!self.check_trace("caller", constant, (DCOMP_CALL_TRACE|DCOMP_ALLOW_TRACE_OPS), 0, &ret))
            return ret;
    
        if (a != 1) {
            RETURN_TERM_ERROR("Special form 'caller' called with "
                            "arguments in %T.", t, context, *constant);
        }
        self.text.push(matchCaller); /* Creates binary */
        self.stack_used += 1;
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }


    
    fn silent(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        DMCRet ret;
      
        if (!self.check_trace("silent", constant, DCOMP_ALLOW_TRACE_OPS, 0, &ret))
            return ret;
    
        if (a != 2) {
            RETURN_TERM_ERROR("Special form 'silent' called with wrong "
                            "number of arguments in %T.", t, context, 
                            *constant);
        }
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        self.text.push(matchSilent);
        self.text.push(matchPushC);
        self.text.push(am_true);
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }
    


    fn fun(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        int i;
        DMCRet ret;
        DMCGuardBif *b;
    
        /* Special forms. */
        switch (p[1]) {
        case am_const:
            return self.const(t);
        case am_and:
            return self.and(t);
        case am_or:
            return self.or(t);
        case am_andalso:
        case am_andthen:
            return self.andalso(t);
        case am_orelse:
            return self.orelse(t);
        case am_self:
            return self.self(t);
        case am_message:
            return self.message(t);
        case am_is_seq_trace:
            return self.is_seq_trace(t);
        case am_set_seq_token:
            return self.set_seq_token(t);
        case am_get_seq_token:
            return self.get_seq_token(t);
        case am_return_trace:
            return self.return_trace(t);
        case am_exception_trace:
            return self.exception_trace(t);
        case am_display:
            return self.display(t);
        case am_process_dump:
            return self.process_dump(t);
        case am_enable_trace:
            return self.enable_trace(t);
        case am_disable_trace:
            return self.disable_trace(t);
        case am_trace:
            return self.trace(t);
        case am_caller:
            return self.caller(t);
        case am_silent:
            return self.silent(t);
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


        if b == NULL {
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
        assert!(b->arity == ((int) a) - 1);
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

        // not constant

        for (i = a; i > 1; --i) {
            let c = self.expr(p[i]);
            if c {
                self.do_emit_constant(p[i]);
            }
        }
        switch (b->arity) {
        case 0:
            self.text.push(matchCall0);
            break;
        case 1:
            self.text.push(matchCall1);
            break;
        case 2:
            self.text.push(matchCall2);
            break;
        case 3:
            self.text.push(matchCall3);
            break;
        default:
            erts_exit(ERTS_ERROR_EXIT,"ets:match() internal error, "
                    "guard with more than 3 arguments.");
        }
        PUSH(*text, (UWord) b->biff);
        self.stack_used -= (((int) a) - 2);
        if (self.stack_used > self.stack_need)
            self.stack_need = self.stack_used;
        Ok(false)
    }

    
    fn expr(&mut self, t: Term, constant: bool) -> DMCRet {
        DMCRet ret;
        Eterm tmp;
        Eterm *p;

        switch (t & _TAG_PRIMARY_MASK) {
            case TAG_PRIMARY_LIST:
                self.list(t, constant)
            case TAG_PRIMARY_BOXED:
                if (is_map(t)) {
                    return self.map(t, constant);
                }
                if (is_tuple(t)) {
                    p = tuple_val(t);
                    // #ifdef HARDDEBUG
                    //                 erts_fprintf(stderr,"%d %d %d %d\n",arityval(*p),is_tuple(tmp = p[1]),
                    //                 is_atom(p[1]),db_is_variable(p[1]));
                    // #endif
                    if (arityval(*p) == 1 && is_tuple(tmp = p[1])) {
                        self.tuple(tmp, constant)
                    } else if (arityval(*p) >= 1 && is_atom(p[1]) && 
                               !(db_is_variable(p[1]) >= 0)) {
                        self.fun(t, constant)
                    } else
                        RETURN_TERM_ERROR("%T is neither a function call, nor a tuple "
                                          "(tuples are written {{ ... }}).", t,
                                          context, *constant);
                }
                else 
                    return Ok(true)
            case TAG_PRIMARY_IMMED1:
                if (db_is_variable(t) >= 0) {
                    self.variable(t, constant)
                } else if (t == am_DollarUnderscore) {
                    self.whole_expression(t, constant)
                } else if (t == am_DollarDollar) {
                    self.all_bindings(t, constant)
                }	    
                /* Fall through */
            _ => Ok(true)
        }
        Ok(true)
    }


}

