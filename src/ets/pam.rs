//! Pattern matching abstract machine (PAM)
use super::*;
mod error;
use error::*;

use crate::value::{self, Variant, Cons, Tuple, TryFrom};

struct Pattern {}

/// Compilation flags
///
/// The dialect is in the 3 least significant bits and are to be interspaced by
/// by at least 2 (decimal), thats why ((Uint) 2) isn't used. This is to be 
/// able to add DBIF_GUARD or DBIF BODY to it to use in the match_spec bif
/// table. The rest of the word is used like ordinary flags, one bit for each 
/// flag. Note that DCOMP_TABLE and DCOMP_TRACE are mutually exclusive.
bitflags! {
    pub struct Flag: u8 {
        /// Ets and dets. The body returns a value, and the parameter to the execution is a tuple.
        const DCOMP_TABLE = 1;
        /// Trace. More functions are allowed, and the parameter to the execution will be an array.
        const DCOMP_TRACE = 4;
        /// To mask out the bits marking dialect
        const DCOMP_DIALECT_MASK = 0x7;

        /// When this is active, no setting of trace control words or seq_trace tokens will be done.
        const DCOMP_FAKE_DESTRUCTIVE = 8;

        /// Allow lock seizing operations on the tracee and 3rd party processes
        const DCOMP_ALLOW_TRACE_OPS = 0x10;
        /// This is call trace
        const DCOMP_CALL_TRACE = 0x20;


        // /*
        // ** Flags for the guard bif's
        // */

        // /* These are offsets from the DCOMP_* value */
        // #define DBIF_GUARD 1
        // #define DBIF_BODY  0

        // /* These are the DBIF flag bits corresponding to the DCOMP_* value.
        //  * If a bit is set, the BIF is allowed in that context. */
        // #define DBIF_TABLE_GUARD (1 << (DCOMP_TABLE + DBIF_GUARD))
        // #define DBIF_TABLE_BODY  (1 << (DCOMP_TABLE + DBIF_BODY))
        // #define DBIF_TRACE_GUARD (1 << (DCOMP_TRACE + DBIF_GUARD))
        // #define DBIF_TRACE_BODY  (1 << (DCOMP_TRACE + DBIF_BODY))
        // #define DBIF_ALL \
        // DBIF_TABLE_GUARD | DBIF_TABLE_BODY | DBIF_TRACE_GUARD | DBIF_TRACE_BODY
    }
}

/// match VM instructions
enum Opcode {
    MatchArray, /* Only when parameter is an array (DCOMP_TRACE) */
    MatchArrayBind, /* ------------- " ------------ */
    MatchTuple,
    MatchPushT,
    MatchPushL,
    MatchPushM,
    MatchPop,
    MatchSwap,
    MatchBind,
    MatchCmp,
    MatchEqBin,
    MatchEqFloat,
    MatchEqBig,
    MatchEqRef,
    MatchEq,
    MatchList,
    MatchMap,
    MatchKey,
    MatchSkip,
    MatchPushC,
    MatchConsA, /* Car is below Cdr */
    MatchConsB, /* Cdr is below Car (unusual) */
    MatchMkTuple,
    MatchMkFlatMap,
    MatchMkHashMap,
    MatchCall0,
    MatchCall1,
    MatchCall2,
    MatchCall3,
    MatchPushV,
    MatchPushVResult, /* First variable reference in result */
    MatchPushExpr, /* Push the whole expression we're matching ('$_') */
    MatchPushArrayAsList, /* Only when parameter is an Array and 
			     not an erlang term  (DCOMP_TRACE) */
    MatchPushArrayAsListU, /* As above but unknown size */
    MatchTrue,
    MatchOr,
    MatchAnd,
    MatchOrElse,
    MatchAndAlso,
    MatchJump,
    MatchSelf,
    MatchWaste,
    MatchReturn,
    MatchProcessDump,
    MatchDisplay,
    MatchIsSeqTrace,
    MatchSetSeqToken,
    MatchGetSeqToken,
    MatchSetReturnTrace,
    MatchSetExceptionTrace,
    MatchCatch,
    MatchEnableTrace,
    MatchDisableTrace,
    MatchEnableTrace2,
    MatchDisableTrace2,
    MatchTryMeElse,
    MatchCaller,
    MatchHalt,
    MatchSilent,
    MatchSetSeqTokenFake,
    MatchTrace2,
    MatchTrace3,
}

/// bool tells us if is_constant
type DMCRet = std::result::Result<bool, Error>;

pub(crate) struct Compiler {
    matchexpr: Vec<Term>,
    guardexpr: Vec<Term>,
    bodyexpr: Vec<Term>,
    text: Vec<Opcode>,
    stack: Vec<Term>,
    cflags: Flag,
    stack_used: usize,
    stack_need: usize,
    num_match: usize,
    current_match: usize,
    special: bool,
    is_guard: bool,
    errors: Vec<Error>,
}

impl Compiler {
    pub(crate) fn new(matchexpr: Vec<Term>, guardexpr: Vec<Term>, bodyexpr: Vec<Term>, num_match: usize, cflags: Flag) -> Self {
        Self {
            text: Vec::new(),
            stack: Vec::new(),
            heap: NULL,
            stack_need: 0,
            stack_used: 0,
            // save: NULL,
            // copy: NULL,
            num_match,
            matchexpr,
            guardexpr,
            bodyexpr,
            errors: Vec::new(),
            cflags,
            special: false,
            is_guard: false,
            current_match: 0 // TODO can maybe remove
        }
    }

    /// The actual compiling of the match expression and the guards.
    pub(crate) fn match_compile(&mut self) -> std::result::Result<Vec<u8>, Error> {
        // MatchProg *ret = NULL;
        // Eterm t;
        // Uint i;
        // Uint num_iters;
        // int structure_checked;
        // DMCRet res;
        let mut current_try_label = -1;
        // Binary *bp = NULL;

        self.heap.size = DEFAULT_SIZE;
        self.heap.vars = self.heap.vars_def;
        self.heap.vars_used = 0;

        // Compile the match expression.
        for i in 0..self.num_match { // long loop ahead
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
            // TODO, are all these -1 ?
            loop {
                match t.into_variant() {
                    Variant::Pointer(..) => {
                        match t.get_boxed_header().unwrap() {
                            BOXED_MAP => {
                                let map = value::Map::try_from(&t).unwrap().0;
                                let num_iters = map.len();
                                if !structure_checked {
                                    self.text.push(Opcode::MatchMap(num_iters));
                                }
                                structure_checked = false;

                                for (key, value) in map.iter() {
                                    if self.db_is_variable(key) >= 0 {
                                        if self.err_info {
                                            add_err(self.err_info, "Variable found in map key.", -1, 0usize, dmcError);
                                        }
                                        return Err(());
                                    } else if key == atom!(UNDERSCORE) {
                                        if self.err_info {
                                            add_err(self.err_info, "Underscore found in map key.", -1, 0usize, dmcError);
                                        }
                                        return Err(());
                                    }
                                    self.text.push(Opcode::MatchKey(private_copy(&self, key)));
                                    {
                                        self.stack_used += 1;
                                        let old_stack = self.stack_used;
                                        self.one_term(value).unwrap();
                                        if old_stack != self.stack_used {
                                            assert!(old_stack + 1 == self.stack_used);
                                            self.text.push(Opcode::MatchSwap);
                                        }
                                        if self.stack_used > self.stack_need {
                                            self.stack_need = self.stack_used;
                                        }
                                        self.text.push(Opcode::MatchPop);
                                        self.stack_used -= 1;
                                    }
                                }
                            }
                            BOXED_TUPLE => {
                                let t = Tuple::try_from(&t).unwrap();
                                if !structure_checked { // i.e. we did not pop it
                                    self.text.push(Opcode::MatchTuple(t.len()));
                                }
                                structure_checked = false;
                                for val in t {
                                    self.one_term(t)?;
                                }
                            }
                            _ => {
                                // goto simple_term;
                                structure_checked = false;
                                self.one_term(t)?;
                            }
                        }
                    }
                    Variant::Cons(..) => {
                        if !structure_checked {
                            self.text.push(Opcode::MatchList);
                        }
                        structure_checked = false; // Whatever it is, we did not pop it
                        let cons = Cons::try_from(&t).unwrap();
                        self.one_term(cons.head)?;
                        t = cons.tail;
                        continue;
                    }
                    _ =>  { // Nil and non proper tail end's or single terms as match expressions.
                        //simple_term:
                        structure_checked = false;
                        self.one_term(t)?;
                    }
                }

                // The *program's* stack just *grows* while we are traversing one composite data
                // structure, we can check the stack usage here

                if self.stack_used > self.stack_need {
                    self.stack_need = self.stack_used;
                }

                // We are at the end of one composite data structure, pop sub structures and emit
                // a matchPop instruction (or break)
                if let Some(val) = self.stack.pop() {
                    t = val;
                    self.text.push(Opcode::MatchPop);
                    structure_checked = true; // Checked with matchPushT or matchPushL
                    self.stack_used -= 1;
                } else {
                    break;
                }
            } // end type loop

            // There is one single top variable in the match expression
            // if the text is two Uint's and the single instruction
            // is 'matchBind' or it is only a skip.
            self.special =
                ((self.text.len() - 1) == 2 + clause_start &&
                 self.text[clause_start] == Opcode::MatchBind) ||
                ((self.text.len() - 1) == 1 + clause_start &&
                 self.text[clause_start] == Opcode::MatchSkip);

            if self.cflags.contains(Flag::DCOMP_TRACE) {
                if self.special {
                    if self.text[clause_start] == Opcode::MatchBind {
                        self.text[clause_start] = Opcode::MatchArrayBind;
                    }
                } else {
                    assert!(STACK_NUM(self.text) >= 1);
                    if self.text[clause_start] != Opcode::MatchTuple {
                        // If it isn't "special" and the argument is not a tuple, the expression is not valid when matching an array
                        if self.err_info {
                            add_err(self.err_info, "Match head is invalid in this self.", -1, 0usize, dmcError);
                        }
                        return Err(());
                    }
                    self.text[clause_start] = Opcode::MatchArray;
                }
            }

            // ... and the guards
            self.is_guard = true;
            self.compile_guard_expr(self.guardexpr[self.current_match])?;
            self.is_guard = false;

            if self.cflags.contains(Flag::DCOMP_TABLE) && !self.bodyexpr[self.current_match].is_list() {
                if self.err_info {
                    add_err(self.err_info, "Body clause does not return anything.", -1, 0usize, dmcError);
                }
                return Err(());
            }

            self.compile_guard_expr(self.bodyexpr[self.current_match])?;

            // The compilation does not bail out when error information is requested, so we need to
            // detect that here...
            if self.err_info != NULL && self.err_info.error_added {
                return Err(());
            }


            // If the matchprogram comes here, the match is successful
            self.text.push(Opcode::MatchHalt);
            // Fill in try-me-else label if there is one.
            if current_try_label >= 0 {
                self.text[current_try_label] = STACK_NUM(self.text);
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
        // bp = erts_create_magic_binary(((sizeof(MatchProg) - sizeof(UWord)) +
        //                             (STACK_NUM(text) * sizeof(UWord))),
        //                             erts_db_match_prog_destructor);
        // ret = Binary2MatchProg(bp);
        // ret.saved_program_buf = NULL;
        // ret.saved_program = NIL;
        // ret.term_save = self.save;
        // ret.num_bindings = heap.vars_used;
        // ret.single_variable = self.special;
        // sys_memcpy(ret.text, STACK_DATA(text), STACK_NUM(text) * sizeof(UWord));
        // ret.stack_offset = heap.vars_used*sizeof(MatchVariable) + FENCE_PATTERN_SIZE;
        // ret.heap_size = ret.stack_offset + self.stack_need * sizeof(Eterm*) + FENCE_PATTERN_SIZE;

    // #ifdef DEBUG
    //     ret.prog_end = ret.text + STACK_NUM(text);
    // #endif

        // Fall through to cleanup code, but self.save should not be free'd
        self.save = NULL;
        // ...
        return bp;
    }

    /// Handle one term in the match expression (not the guard)
    fn one_term(&mut self, c: Term) -> DMCRet {
        // Sint n;
        // Eterm *hp;
        // Uint sz, sz2, sz3;
        // Uint i, j;

        match c.value.tag() as u8 {
            value::TERM_ATOM => {
                if (n = self.db_is_variable(c)) >= 0 { /* variable */
                    if self.heap.vars[n].is_bound {
                        self.text.push(Opcode::MatchCmp(n));
                    } else { /* Not bound, bind! */
                        if n >= self.heap.vars_used {
                            self.heap.vars_used = n + 1;
                        }
                        self.text.push(Opcode::MatchBind(n));
                        self.heap.vars[n].is_bound = true;
                    }
                } else if c == atom!(UNDERSCORE) {
                    self.text.push(Opcode::MatchSkip);
                } else {
                    // Any other atom value
                    self.text.push(Opcode::MatchEq(c as usize));
                }
            }
            value::TERM_CONS => {
                self.text.push(Opcode::MatchPushL);
                self.stack_used += 1;
                self.stack.push(c);
            }
            value::TERM_FLOAT => {
                self.text.push(Opcode::MatchEqFloat(c));
            // #ifdef ARCH_64
            //     PUSH(*self.text, 0);
            // #else
            //     PUSH(*self.text, float_val(c)[2] as usize);
            // #endif
            }
            value::TERM_POINTER => {
                match c.get_boxed_header().unwrap() { // inefficient, cast directly
                    value::BOXED_TUPLE => {
                        let n = Tuple::try_from(&c).unwrap().len();
                        self.text.push(Opcode::MatchPushT(n));
                        self.stack_used += 1;
                        self.stack.push(c);
                    }
                    value::BOXED_MAP => {
                        let n = value::Map::try_from(&c).unwrap().0.len();
                        self.text.push(Opcode::MatchPushM(n));
                        self.stack_used += 1;
                        self.stack.push(c);
                    }
                    value::BOXED_REF => {
                        Eterm* ref_val = internal_ref_val(c);
                        self.text.push(Opcode::MatchEqRef);
                        n = thing_arityval(ref_val[0]);
                        for (i = 0; i <= n; ++i) {
                            PUSH(*self.text, ref_val[i]);
                        }
                    }
                    value::BOXED_BIGINT => {
                        Eterm* bval = big_val(c);
                        n = thing_arityval(bval[0]);
                        self.text.push(Opcode::MatchEqBig);
                        for (i = 0; i <= n; ++i) {
                            PUSH(*self.text, (Uint) bval[i]);
                        }
                    }
                    _ => { /* BINARY, FUN, VECTOR, or EXTERNAL */
                        self.text.push(Opcode::MatchEqBin(private_copy(self, c)));
                    }
                }
            }
            _ => {
                // Any immediate value
                self.text.push(Opcode::MatchEq(c as usize));
            }
        }

        Ok(())
    }

    fn compile_guard_expr(&self, mut l: Term) -> DMCRet {
        // DMCRet ret;
        // int constant;
        // Eterm t;

        if l != Term::nil() {
            if !l.is_list() {
                RETURN_ERROR("Match expression is not a list.", self);
            }
            if !self.is_guard {
                self.text.push(Opcode::MatchCatch);
            }
            while let Ok(Cons { head: t, tail }) = l.try_into() {
                let constant = self.expr(t)?;
                if constant {
                    self.do_emit_constant(t);
                }
                l = tail;
                if self.is_guard {
                    self.text.push(Opcode::MatchTrue);
                } else {
                    self.text.push(Opcode::MatchWaste);
                }
                self.stack_used -= 1;
            }
            if !l.is_nil() {
                RETURN_ERROR("Match expression is not a proper list.", self);
            }
            if !self.is_guard && self.cflags.contains(Flag::DCOMP_TABLE) {
                assert!(Some(Opcode::MatchWaste) == self.text.last());
                self.text.pop();
                self.text.push(Opcode::MatchReturn); // Same impact on stack as matchWaste
            }
        }
        Ok(())
    }

    /*
    ** Match guard compilation
    */

    fn do_emit_constant(&self, t: Term) {
        // int sz;
        // ErlHeapFragment *emb;
        // Eterm *hp;
        // Eterm tmp;

        if IS_CONST(t) {
            tmp = t;
        } else {
            sz = my_size_object(t);
            emb = new_message_buffer(sz);
            hp = emb.mem;
            tmp = my_copy_struct(t,&hp,&emb.off_heap);
            emb->next = self.save;
            self.save = emb;
        }
        self.text.push(Opcode::MatchPushC(tmp));
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
           self.stack_need = self.stack_used;
        }
    }

    fn list(&mut self, t: Term) -> DMCRet {
        let cons = Cons::try_from(&t).unwrap();
        let c1 = self.expr(cons.head)?;
        let c2 = self.expr(cons.tail)?;

        if c1 && c2 {
            return Ok(true);
        } 
        if !c1 {
            /* The CAR is not a constant, so if the CDR is, we just push it,
            otherwise it is already pushed. */
            if c2 {
                self.do_emit_constant(cons.tail);
            }
            self.text.push(Opcode::MatchConsA);
        } else { /* !c2 && c1 */
            self.do_emit_constant(cons.head);
            self.text.push(Opcode::MatchConsB);
        }
        self.stack_used -= 1; /* Two objects on stack becomes one */
        Ok(false)
    }

    fn rearrange_constants(&mut self, textpos: usize, Eterm *p, nelems: usize) {
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

    fn array(&mut self, terms: &[Term]) -> DMCRet {
        let mut all_constant = true;
        int textpos = STACK_NUM(*text);
        Uint i;

        // We remember where we started to layout code,
        // assume all is constant and back up and restart if not so.
        // The array should be laid out with the last element first,
        // so we can memcpy it to the eheap.

        // p = terms, nemels = terms.len()

        for (i = nelems; i--;) {
            let res = self.expr(p[i])?;
            if !res && all_constant {
                all_constant = false;
                if i < nelems - 1 {
                    self.rearrange_constants(textpos, p + i + 1, nelems - i - 1);
                }
            } else if res && !all_constant {
                self.do_emit_constant(p[i]);
            }
        }
        Ok(all_constant)
    }

    fn tuple(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let nelems = t.len();

        let all_constant = self.array(&t[..])?;
        if all_constant {
            return Ok(true);
        }
        self.text.push(Opcode::MatchMkTuple(nelems));
        self.stack_used -= nelems - 1;
        Ok(false)
    }

    fn map(&mut self, t: Term) -> DMCRet {
        assert!(t.is_map());

        let map = value::Map::try_from(&t).unwrap().0;
        let mut constant_values = true;
        let nelems = map.len();

        for (_, val) in map {
            let c = self.expr(val)?;
            if !c {
                constant_values = false;
            }
        }

        if constant_values {
            return Ok(true);
        }

        // not constant

        for (key, value) in map {
            // push key
            let c = self.expr(key)?;
            if c {
                self.do_emit_constant(key);
            }
            // push value
            let c = self.expr(value)?;
            if c {
                self.do_emit_constant(value);
            }
        }
        self.text.push(Opcode::MatchMkHashMap(nelems));
        self.stack_used -= nelems;
        Ok(false)
    }

    fn whole_expression(&mut self, t: Term) -> DMCRet {
        if self.cflags.contains(Flag::DCOMP_TRACE) {
            // Hmmm, convert array to list...
            if self.special {
                self.text.push(Opcode::MatchPushArrayAsListU);
            } else { 
                assert!(self.matchexpr[self.current_match].is_tuple());
                self.text.push(Opcode::MatchPushArrayAsList);
            }
        } else {
            self.text.push(Opcode::MatchPushExpr);
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    /// Figure out which PushV instruction to use.
    fn add_pushv_variant(&mut self, n: usize) {
        let v = &mut self.heap.vars[n];
        let mut instr = Opcode::MatchPushV;

        assert!(n < self.heap.vars_used && v.is_bound);
        if !self.is_guard {
            if !v.is_in_body {
                instr = Opcode::MatchPushVResult;
                v.is_in_body = true;
            }
        }
        self.text.push(instr);
        self.text.push(n);
    }

    fn variable(&mut self, n: usize) -> DMCRet {
        // TODO this is already called inside expr(), just pass number in instead
        // optimize this in beam too
        // Uint n = db_is_variable(t);

        if n >= self.heap.vars_used || !self.heap.vars[n].is_bound {
            RETURN_VAR_ERROR("Variable $%%d is unbound.", n);
        }

        self.add_pushv_variant(n);

        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn all_bindings(&mut self, t: Term) -> DMCRet {
        int i;
        int heap_used = 0;

        self.text.push(Opcode::MatchPushC);
        self.text.push(NIL);
        for (i = self.heap.vars_used - 1; i >= 0; --i) {
            if self.heap.vars[i].is_bound {
                self.add_pushv_variant(i);
                self.text.push(Opcode::MatchConsB);
                heap_used += 2;
            }
        }
        self.stack_used += 1;
        if (self.stack_used + 1) > self.stack_need  {
            self.stack_need = (self.stack_used + 1);
        }
        Ok(false)
    }

    fn constant(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let a = t.len();

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "const", value: t, reason: "with more than one argument" }));
        }
        Ok(true)
    }

    fn and(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let a = t.len();
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "and", value: t, reason: "without arguments" }));
        }
        for val in &t[1..] { // skip the :&&
            let c = self.expr(val)?;
            if c {
                self.do_emit_constant(*val);
            }
        }
        self.text.push(Opcode::MatchAnd);
        self.text.push(a - 1);
        self.stack_used -= a - 2;
        Ok(false)
    }

    fn or(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let a = t.len();
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "or", value: t, reason: "without arguments" }));
        }
        for val in &t[1..] { // skip the :||
            let c = self.expr(val)?;
            if c {
                self.do_emit_constant(*val);
            }
        }
        self.text.push(Opcode::MatchOr);
        self.text.push(a - 1);
        self.stack_used -= a - 2;
        Ok(false)
    }


    fn andalso(&mut self, t: Term) -> DMCRet {
        let t = Tuple::try_from(&t).unwrap();
        let a = t.len();
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;

        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "andalso", value: t, reason: "without arguments" }));
        }
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
            if i == a {
                self.text.push(Opcode::MatchJump);
            } else {
                self.text.push(Opcode::MatchAndAlso);
            }
            self.text.push(lbl);
            lbl = STACK_NUM(*text)-1;
            self.stack_used -= 1;
        }
        self.text.push(Opcode::MatchPushC);
        self.text.push(am_true);
        lbl_val = STACK_NUM(*text);
        while (lbl) {
            lbl_next = PEEK(*text, lbl);
            text[lbl] = lbl_val-lbl-1;
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
           self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn orelse(&mut self, t: Term) -> DMCRet {
        Eterm *p = tuple_val(t);
        Uint a = arityval(*p);
        int i;
        int c;
        Uint lbl;
        Uint lbl_next;
        Uint lbl_val;
        
        if a < 2 {
            return Err(new_error(ErrorKind::Argument { form: "orelse", value: t, reason: "without arguments" }));
        }
        lbl = 0;
        for (i = 2; i <= a; ++i) {
            let c = self.expr(p[i])?;
            if c {
                self.do_emit_constant(p[i]);
            }
            if (i == a) {
                self.text.push(Opcode::MatchJump);
            } else {
                self.text.push(Opcode::MatchOrElse);
            }
            self.text.push(lbl);
            lbl = STACK_NUM(*text)-1;
            self.stack_used -= 1;
        }
        self.text.push(Opcode::MatchPushC);
        self.text.push(am_false);
        lbl_val = STACK_NUM(*text);
        while lbl {
            lbl_next = PEEK(*text, lbl);
            text[lbl] = lbl_val-lbl-1;
            lbl = lbl_next;
        }
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn message(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "message" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "message" }));
        }

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "message", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchReturn);
        self.text.push(Opcode::MatchPushC);
        self.text.push(atom!(TRUE));
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn selff(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "self", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSelf);
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn return_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "return_trace" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "return_trace" }));
        }

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "return_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSetReturnTrace); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn exception_trace(&mut self, Eterm t) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "exception_trace" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "exception_trace" }));
        }

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "exception_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchSetExceptionTrace); /* Pushes 'true' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn check_trace(&self, op: &str, need_cflags: Flag, allow_in_guard: bool) -> DMCRet {
        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: op }))
        }
        if (self.cflags & need_cflags) != need_cflags {
            RETURN_ERROR_X(-1, "Special form '%s' not allowed for this trace event.", op);
        }
        if self.is_guard && !allow_in_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: op }));
        }
        true
    }

    fn is_seq_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("is_seq_trace", Flag::DCOMP_ALLOW_TRACE_OPS, true)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "is_seq_trace", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchIsSeqTrace); 
        /* Pushes 'true' or 'false' on the stack */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn set_seq_token(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("set_seq_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 3 {
            return Err(new_error(ErrorKind::Argument { form: "set_seq_token", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[2])?;
        if c { 
            self.do_emit_constant(p[2]);
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        if self.cflags.contains(Flag::DCOMP_FAKE_DESTRUCTIVE) {
            self.text.push(Opcode::MatchSetSeqTokenFake);
        } else {
            self.text.push(Opcode::MatchSetSeqToken);
        }
        self.stack_used -= 1; /* Remove two and add one */
        Ok(false)
    }

    fn get_seq_token(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        self.check_trace("get_seq_token", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "get_seq_token", value: t, reason: "with arguments" }));
        }

        self.text.push(Opcode::MatchGetSeqToken);
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn display(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        if !self.cflags.contains(Flag::DCOMP_TRACE) {
            return Err(new_error(ErrorKind::WrongDialect { form: "display" }));
        }
        if self.is_guard {
            return Err(new_error(ErrorKind::CalledInGuard { form: "display" }));
        }

        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "display", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchDisplay);
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }

    fn process_dump(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();

        self.check_trace("process_dump", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "process_dump", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchProcessDump); /* Creates binary */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn enable_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();
        
        self.check_trace("enable_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            2 => {
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchEnableTrace);
                /* Push as much as we remove, stack_need is untouched */
            }
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchEnableTrace2);
                self.stack_used -= 1; /* Remove two and add one */
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "enable_trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn disable_trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();

        self.check_trace("disable_trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            2 => {
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchDisableTrace);
                /* Push as much as we remove, stack_need is untouched */
            }
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchDisableTrace2);
                self.stack_used -= 1; // Remove two and add one
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "disable_trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn trace(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let arity = p.len();
        
        self.check_trace("trace", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;

        match arity {
            3 => {
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchTrace2);
                --self.stack_used; /* Remove two and add one */
            }
            4 => {
                let c = self.expr(p[3])?;
                if c { 
                    self.do_emit_constant(p[3]);
                }
                let c = self.expr(p[2])?;
                if c { 
                    self.do_emit_constant(p[2]);
                }
                let c = self.expr(p[1])?;
                if c { 
                    self.do_emit_constant(p[1]);
                }
                self.text.push(Opcode::MatchTrace3);
                self.stack_used -= 2; /* Remove three and add one */
            }
            _ => return Err(new_error(ErrorKind::Argument { form: "trace", value: t, reason: "with wrong number of arguments" }))
        }
        Ok(false)
    }

    fn caller(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        
        self.check_trace("caller", (Flag::DCOMP_CALL_TRACE|Flag::DCOMP_ALLOW_TRACE_OPS), false)?;
    
        if a != 1 {
            return Err(new_error(ErrorKind::Argument { form: "caller", value: t, reason: "with arguments" }));
        }
        self.text.push(Opcode::MatchCaller); /* Creates binary */
        self.stack_used += 1;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }

    fn silent(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
      
        self.check_trace("silent", Flag::DCOMP_ALLOW_TRACE_OPS, false)?;
    
        if a != 2 {
            return Err(new_error(ErrorKind::Argument { form: "silent", value: t, reason: "with wrong number of arguments" }));
        }
        let c = self.expr(p[1])?;
        if c { 
            self.do_emit_constant(p[1]);
        }
        self.text.push(Opcode::MatchSilent);
        self.text.push(Opcode::MatchPushC);
        self.text.push(atom!(TRUE));
        /* Push as much as we remove, stack_need is untouched */
        Ok(false)
    }
    
    fn fun(&mut self, t: Term) -> DMCRet {
        let p = Tuple::try_from(&t).unwrap();
        let a = p.len();
        int i;
        DMCGuardBif *b;
    
        /* Special forms. */
        match p[0].into_variant() {
            Variant::Atom(atom::CONST) => return self.constant(t),
            Variant::Atom(atom::AND) => return self.and(t),
            Variant::Atom(atom::OR) => return self.or(t),
            Variant::Atom(atom::ANDALSO) => return self.andalso(t),
            Variant::Atom(atom::ANDTHEN) => return self.andalso(t),
            Variant::Atom(atom::ORELSE) => return self.orelse(t),
            Variant::Atom(atom::SELF) => return self.selff(t),
            Variant::Atom(atom::MESSAGE) => return self.message(t),
            Variant::Atom(atom::IS_SEQ_TRACE) => return self.is_seq_trace(t),
            Variant::Atom(atom::SET_SEQ_TOKEN) => return self.set_seq_token(t),
            Variant::Atom(atom::GET_SEQ_TOKEN) => return self.get_seq_token(t),
            Variant::Atom(atom::RETURN_TRACE) => return self.return_trace(t),
            Variant::Atom(atom::EXCEPTION_TRACE) => return self.exception_trace(t),
            Variant::Atom(atom::DISPLAY) => return self.display(t),
            Variant::Atom(atom::PROCESS_DUMP) => return self.process_dump(t),
            Variant::Atom(atom::ENABLE_TRACE) => return self.enable_trace(t),
            Variant::Atom(atom::DISABLE_TRACE) => return self.disable_trace(t),
            Variant::Atom(atom::TRACE) => return self.trace(t),
            Variant::Atom(atom::CALLER) => return self.caller(t),
            Variant::Atom(atom::SILENT) => return self.silent(t),
            Variant::Atom(atom::SET_TCW) => {
                b = if self.cflags.contains(Flag::DCOMP_FAKE_DESTRUCTIVE) {
                    lookup_bif(am_set_tcw_fake, a - 1);
                } else {
                    lookup_bif(p[1], a - 1);
                }
            }
            _ => b = lookup_bif(p[1],  a - 1),
        }


        if b == NULL {
            if self.err_info != NULL {
                /* Ugly, should define a better RETURN_TERM_ERROR interface... */
                char buff[100];
                erts_snprintf(buff, sizeof(buff),
                        "Function %%T/%d does_not_exist.",
                        (int)a - 1);
                RETURN_TERM_ERROR(buff, p[1], context);
            } else {
                return Err(());
            }
        } 
        assert!(b.arity == ((int) a) - 1);
        if !(b.flags & 
            (1 << 
                ((self.cflags & Flag::DCOMP_DIALECT_MASK) + 
                (self.is_guard ? DBIF_GUARD : DBIF_BODY)))) {
            /* Body clause used in wrong context. */
            if (self.err_info != NULL) {
                /* Ugly, should define a better RETURN_TERM_ERROR interface... */
                char buff[100];
                erts_snprintf(buff, sizeof(buff),
                        "Function %%T/%d cannot be called in this context.",
                        (int)a - 1);
                RETURN_TERM_ERROR(buff, p[1], context);
            } else {
                return Err(());
            }
        }	

        // not constant

        for (i = a; i > 1; --i) {
            let c = self.expr(p[i]);
            if c {
                self.do_emit_constant(p[i]);
            }
        }
        match b.arity {
            0 => self.text.push(Opcode::MatchCall0);
            1 => self.text.push(Opcode::MatchCall1),
            2 => self.text.push(Opcode::MatchCall2),
            3 => self.text.push(Opcode::MatchCall3),
            _ => panic!("ets:match() internal error, guard with more than 3 arguments.");
        }
        self.text.push(b.biff as usize);
        self.stack_used -= a - 2;
        if self.stack_used > self.stack_need {
            self.stack_need = self.stack_used;
        }
        Ok(false)
    }
    
    fn expr(&mut self, t: Term, constant: bool) -> DMCRet {
        match t.value.tag() as u8 {
            value::TERM_CONS => self.list(t),
            value::TERM_POINTER => {
                if t.is_map() {
                    return self.map(t);
                }
                if t.is_tuple() {
                    let p = Tuple::try_from(&t).unwrap();
                    // #ifdef HARDDEBUG
                    //                 erts_fprintf(stderr,"%d %d %d %d\n",arityval(*p),is_tuple(tmp = p[1]),
                    //                 is_atom(p[1]),db_is_variable(p[1]));
                    // #endif
                    if p.len() == 1 && p[0].is_tuple() {
                        self.tuple(p[0])
                    } else if p.len() >= 1 && p[0].is_atom() && self.db_is_variable().is_none() {
                        self.fun(t)
                    } else {
                        RETURN_TERM_ERROR("%T is neither a function call, nor a tuple (tuples are written {{ ... }}).", t);
                    }
                } else {
                    Ok(true)
                }
            }
            value::TERM_ATOM => { // immediate
                let n = self.db_is_variable(t);

                if n >= 0 {
                    self.variable(n)
                } else if t == atom!(DOLLAR_UNDERSCORE) {
                    self.whole_expression(t)
                } else if t == atom!(DOLLAR_DOLLAR) {
                    self.all_bindings(t)
                } else {
                    Ok(true)
                }  
            }
            // Fall through, immediate
            _ => Ok(true)
        }
    }

}

