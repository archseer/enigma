use crate::atom;
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader::FuncInfo;
use crate::module::MFA;
use crate::process::RcProcess;
use crate::value::{self, CastFrom, CastInto, Term, Variant};

/// http://erlang.org/doc/reference_manual/errors.html#exceptions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exception {
    pub reason: Reason, // bitflags
    pub value: Term,
    pub trace: Term,
}

impl Exception {
    #[inline]
    pub fn new(reason: Reason) -> Self {
        Exception {
            reason,
            value: Term::nil(),
            trace: Term::nil(),
        }
    }

    #[inline]
    pub fn with_value(reason: Reason, value: Term) -> Self {
        Exception {
            reason,
            value,
            trace: Term::nil(),
        }
    }
}

impl From<value::WrongBoxError> for Exception {
    fn from(_value: value::WrongBoxError) -> Self {
        Exception::new(Reason::EXC_BADARG)
    }
}

impl From<crate::ets::error::Error> for Exception {
    fn from(value: crate::ets::error::Error) -> Self {
        match value.kind() {
            crate::ets::error::ErrorKind::BadItem => Exception::new(Reason::EXC_BADARG),
            _ => unimplemented!(),
        }
    }
}

// impl From<std::io::Error> for Exception {
//     fn from(error: std::io::Error) -> Self {
//         let reason = match error {
//
//         }
//
//         Exception::new(reason)
//     }
// }

bitflags! {
    pub struct Reason: u32 {
        /// There are three primary exception classes:
        ///
        /// - exit Process termination - not an error.
        /// - error Error (adds stacktrace; will be logged).
        /// - thrown Nonlocal return (turns into a "nocatch" error if not caught by the process).
        ///
        /// In addition, we define a number of exit codes as a convenient
        /// short-hand: instead of building the error descriptor term at the time
        /// the exception is raised, it is built as necessary when the exception
        /// is handled. Examples are EXC_NORMAL, EXC_BADARG, EXC_BADARITH, etc.
        /// Some of these have convenient aliases, like BADARG and BADARITH.

        /// Tag
        const EXT_OFFSET = 0;
        const EXT_BITS = 2;

        /// Runtime error or the process called erlang:error/1,2
        const EXT_ERROR = 0b00;
        /// The process called exit/1
        const EXT_EXIT  = 0b01;
        /// The process called throw/1
        const EXT_THROW = 0b10;

        const EXT_TAGBITS = (1 << Self::EXT_BITS.bits) - 1;

        /// Exit code flags
        ///
        /// These flags make is easier and quicker to decide what to do with the
        /// exception in the early stages, before a handler is found, and also
        /// maintains some separation between the class tag and the actions.

        const EXF_OFFSET = Self::EXT_TAGBITS.bits;
        const EXF_BITS = 7;

        /// ignore catches
        const EXF_PANIC       = 1 << (0 + Self::EXF_OFFSET.bits);
        /// nonlocal return
        const EXF_THROWN      = 1 << (1 + Self::EXF_OFFSET.bits);
        /// write to logger on termination
        const EXF_LOG         = 1 << (2 + Self::EXF_OFFSET.bits);
        /// occurred in native code
        const EXF_NATIVE      = 1 << (3 + Self::EXF_OFFSET.bits);
        /// save stack trace in internal form
        const EXF_SAVETRACE   = 1 << (4 + Self::EXF_OFFSET.bits);
        /// has arglist for top of trace
        const EXF_ARGLIST     = 1 << (5 + Self::EXF_OFFSET.bits);
        /// restore original bif/nif
        const EXF_RESTORE_NIF = 1 << (6 + Self::EXF_OFFSET.bits);

        const EXF_FLAGBITS  = (((1<<(Self::EXF_BITS.bits + Self::EXF_OFFSET.bits))-1) & !((1<<(Self::EXF_OFFSET.bits))-1));

        /// Primary exception
        const EXF_PRIMARY = Self::EXF_PANIC.bits | Self::EXF_THROWN.bits | Self::EXF_LOG.bits | Self::EXF_NATIVE.bits;

        /// Exit codes used for raising a fresh exception. The primary exceptions
        /// share index 0 in the descriptor table. EXC_NULL signals that no
        /// exception has occurred. The primary exit codes EXC_EXIT, EXC_ERROR
        /// and EXC_THROWN are the basis for all other exit codes, and must
        /// always have the EXF_SAVETRACE flag set so that a trace is saved
        /// whenever a new exception occurs; the flag is then cleared.
        /// Initial value for p->freason

        /// Error code used for indexing into the short-hand error descriptor table.
        const EXC_OFFSET = Self::EXF_OFFSET.bits + Self::EXF_BITS.bits;
        const EXC_BITS   = 5;

        const EXC_CODEBITS = (((1<<(Self::EXC_BITS.bits + Self::EXC_OFFSET.bits))-1) & !((1<<(Self::EXC_OFFSET.bits))-1));

        /// Default value on boot.
        const EXC_NULL = 0;

        const EXC_PRIMARY = Self::EXF_SAVETRACE.bits;

        /// Generic error (exit term in p->fvalue)
        const EXC_ERROR  = (Self::EXC_PRIMARY.bits | Self::EXT_ERROR.bits | Self::EXF_LOG.bits);
        /// Generic exit (exit term in p->fvalue)
        const EXC_EXIT   = (Self::EXC_PRIMARY.bits | Self::EXT_EXIT.bits);
        /// Generic nonlocal return (thrown term in p->fvalue)
        const EXC_THROWN = (Self::EXC_PRIMARY.bits | Self::EXT_THROW.bits | Self::EXF_THROWN.bits);

        /// Error with given arglist term (exit reason in p->fvalue)
        const EXC_ERROR_2  = (Self::EXC_ERROR.bits | Self::EXF_ARGLIST.bits);

        /// Normal exit (reason 'normal')
        const EXC_NORMAL          = ((1 << Self::EXC_OFFSET.bits) | Self::EXC_EXIT.bits);
        /// Things that shouldn't happen
        const EXC_INTERNAL_ERROR  = ((2 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits | Self::EXF_PANIC.bits);
        /// Bad argument to a BIF
        const EXC_BADARG          = ((3 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Bad arithmetic
        const EXC_BADARITH        = ((4 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Bad match in function body
        const EXC_BADMATCH        = ((5 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// No matching function head
        const EXC_FUNCTION_CLAUSE = ((6 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// No matching case clause
        const EXC_CASE_CLAUSE     = ((7 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// No matching if clause
        const EXC_IF_CLAUSE       = ((8 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// No farity that matches
        const EXC_UNDEF           = ((9 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Not an existing fun
        const EXC_BADFUN          = ((10 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Attempt to call fun with wrong number of arguments.
        const EXC_BADARITY        = ((11 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Bad time out value
        const EXC_TIMEOUT_VALUE   = ((12 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        ///* No process or port
        const EXC_NOPROC          = ((13 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        ///* Not distributed
        const EXC_NOTALIVE        = ((14 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Ran out of something
        const EXC_SYSTEM_LIMIT    = ((15 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// No matching try clause
        const EXC_TRY_CLAUSE      = ((16 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Not supported
        const EXC_NOTSUP          = ((17 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Bad map
        const EXC_BADMAP          = ((18 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);
        /// Bad key in map
        const EXC_BADKEY          = ((19 << Self::EXC_OFFSET.bits) | Self::EXC_ERROR.bits);

        /*
         * Internal pseudo-error codes.
         */

        /// BIF Trap to erlang code
        const TRAP = (1 << Self::EXC_OFFSET.bits);
    }
}

macro_rules! exception_class {
    ($x:expr) => {
        $x & Reason::EXT_TAGBITS
    };
}

macro_rules! primary_exception {
    ($x:expr) => {
        $x & (Reason::EXF_PRIMARY | Reason::EXT_TAGBITS)
    };
}
// #define PRIMARY_EXCEPTION(x) ((x) & (EXF_PRIMARY | EXC_CLASSBITS))

macro_rules! native_exception {
    ($x:expr) => {
        $x & Reason::EXF_NATIVE
    };
}
// #define NATIVE_EXCEPTION(x) ((x) | EXF_NATIVE)

// get_exc_index
macro_rules! exception_code {
    ($x:expr) => {
        ($x.bits & Reason::EXC_CODEBITS.bits) >> Reason::EXC_OFFSET.bits
    };
}

const MAX_BACKTRACE_SIZE: u32 = 64;
pub const DEFAULT_BACKTRACE_SIZE: u32 = 8;

const EXIT_TAGS: [u32; 3] = [atom::ERROR, atom::EXIT, atom::THROW];

/// Mapping from error code 'index' to atoms.
const EXIT_CODES: [u32; 20] = [
    atom::INTERNAL_ERROR, // 0
    atom::NORMAL,
    atom::INTERNAL_ERROR,
    atom::BADARG,
    atom::BADARITH,
    atom::BADMATCH,
    atom::FUNCTION_CLAUSE,
    atom::CASE_CLAUSE,
    atom::IF_CLAUSE,
    atom::UNDEF,
    atom::BADFUN,
    atom::BADARITY,
    atom::TIMEOUT_VALUE,
    atom::NO_PROC,
    atom::NOT_ALIVE,
    atom::SYSTEM_LIMIT,
    atom::TRY_CLAUSE,
    atom::NOT_SUP,
    atom::BADMAP,
    atom::BADKEY, // 19
];

/// The quick-saved stack trace structure
#[derive(Debug)]
pub struct StackTrace {
    /// original exception reason is saved in the struct
    pub reason: Reason, // bitflags
    ///
    pub pc: Option<InstrPtr>,
    pub current: MFA,
    // /// number of saved pointers in trace[]
    // int depth;
    // BeamInstr *trace[1];  /* varying size - must be last in struct */
    pub trace: Vec<InstrPtr>,
    pub complete: bool,
} // TODO: make all fields private with a constructor

impl CastFrom<Term> for StackTrace {
    type Error = value::WrongBoxError;

    #[inline]
    fn cast_from(value: &Term) -> Result<&Self, value::WrongBoxError> {
        if let Variant::Pointer(ptr) = value.into_variant() {
            unsafe {
                if *ptr == value::BOXED_STACKTRACE {
                    return Ok(&(*(ptr as *const value::Boxed<Self>)).value);
                }
            }
        }
        Err(value::WrongBoxError)
    }
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
// return val is another pc pointer (u32 + module)

// static BeamInstr*
// handle_error(Process* c_p, BeamInstr* pc, ErtsCodeMFA *bif_mfa)
// {
pub fn handle_error(
    process: &RcProcess,
    mut exc: Exception, /*, bif_mfa: &MFA*/
) -> Option<InstrPtr> {
    // print!(
    //     "handling error... proc pid={} exc={}\r\n",
    //     process.pid, exc.value
    // );
    let heap = &process.context_mut().heap;
    let args = Term::atom(atom::TRUE);

    let context = process.context_mut();
    // let exc = &mut context.exc.unwrap();

    assert!(exc.reason != Reason::TRAP); /* Should have been handled earlier. */
    //     if (c_p->freason & EXF_RESTORE_NIF) {
    //      	erts_nif_export_restore_error(c_p, reg, &bif_mfa);
    //     }

    //     /*
    //      * Check if we have an arglist for the top level call. If so, this
    //      * is encoded in Term, so we have to dig out the real Term as well
    //      * as the Arglist.
    //      */
    //     if (c_p->freason & EXF_ARGLIST) { // TODO: this is a special case for BIF error/2
    // 	  Eterm* tp;
    // 	  ASSERT(is_tuple(Value));
    // 	  tp = tuple_val(Value);
    // 	  Value = tp[1];
    // 	  args = tp[2];
    //     }

    /*
     * Save the stack trace info if the EXF_SAVETRACE flag is set. The
     * main reason for doing this separately is to allow throws to later
     * become promoted to errors without losing the original stack
     * trace, even if they have passed through one or more catch and
     * rethrow. It also makes the creation of symbolic stack traces much
     * more modular.
     */
    if exc.reason.contains(Reason::EXF_SAVETRACE) {
        save_stacktrace(process, &mut exc, /*bif_mfa,*/ args);
    }

    // Throws that are not caught are turned into 'nocatch' errors
    if exc.reason.contains(Reason::EXF_THROWN) && context.catches == 0 {
        exc.value = tup2!(heap, Term::atom(atom::NOCATCH), exc.value);
        exc.reason = Reason::EXC_ERROR;
    }

    // Get the fully expanded error term
    exc.value = expand_error_value(process, exc.reason, exc.value);

    // Save final error term and stabilize the exception flags so no
    // further expansion is done.
    exc.reason = primary_exception!(exc.reason);

    //  Find a handler or die
    if context.catches > 0 && !exc.reason.contains(Reason::EXF_PANIC) {
        // The Beam handler code (catch_end or try_end) checks reg[0]
        // for THE_NON_VALUE to see if the previous code finished
        // abnormally. If so, reg[1], reg[2] and reg[3] should hold the
        // exception class, term and trace, respectively. (If the
        // handler is just a trap to native code, these registers will
        // be ignored.)
        context.x[0] = Term::none();
        context.x[1] = Term::atom(EXIT_TAGS[exception_class!(exc.reason).bits as usize]);
        context.x[2] = exc.value;
        context.x[3] = exc.trace;
        if let Some(new_pc) = next_catch(process) {
            context.cp = None; // To avoid keeping stale references.
            process.local_data_mut().mailbox.reset(); // No longer safe to use this position
                                                      // TODO: ^ maybe only reset mark and not save
            return Some(new_pc);
        } else {
            //erts_exit(ERTS_ERROR_EXIT, "Catch not found")
            panic!("Catch not found")
        }
    }
    terminate_process(process, exc);
    None
}

/// Find the nearest catch handler
/// TODO: return is instr pointer
fn next_catch(process: &RcProcess) -> Option<InstrPtr> {
    let context = process.context_mut();
    let mut ptr = context.stack.len();
    let mut prev = ptr;

    // debug_assert!(context.stack.last().unwrap().is_cp());
    // ASSERT(ptr <= STACK_START(c_p));
    if ptr == 0 {
        return None;
    }

    // TODO: tracing instr handling here

    while ptr > 0 {
        match context.stack[ptr - 1].get_boxed_header() {
            Ok(value::BOXED_CATCH) => {
                // this is a mess because we store the cp stack separately now.
                let mut prev = context.stack.len();
                let mut c = 0;
                let iter = context.callstack.iter().rev();
                for &(n, _) in iter {
                    let n = n as usize;
                    if prev - n <= (ptr - 1) {
                        break;
                    }
                    prev -= n;
                    c += 1;
                }

                let ptr = *context.stack[ptr - 1]
                    .get_boxed_value::<InstrPtr>()
                    .unwrap();

                // Unwind the stack up to the current frame.
                context.stack.truncate(prev);
                context.callstack.truncate(context.callstack.len() - c);
                // context.stack.shrink_to_fit();
                // TODO: tracing handling here
                return Some(ptr);
            }
            // Ok(value::BOXED_CP) => {
            //     prev = ptr;
            //     // TODO: OTP does tracing instr handling here
            // }
            _ => (),
        }
        ptr -= 1;
    }
    None
}

/// Terminating the process when an exception is not caught
fn terminate_process(process: &RcProcess, mut exc: Exception) {
    // let heap = &process.context_mut().heap;

    // Add a stacktrace if this is an error.
    if exception_class!(exc.reason) == Reason::EXT_ERROR {
        exc.value = add_stacktrace(process, exc.value, exc.trace);
    }
    // EXF_LOG is a primary exception flag
    if exc.reason.contains(Reason::EXF_LOG) {
        // int alive = erts_is_alive;
        // erts_dsprintf_buf_t *dsbufp = erts_create_logger_dsbuf();

        // Build the format message
        // erts_dsprintf(dsbufp, "Error in process ~p ");
        // if (alive)
        //     erts_dsprintf(dsbufp, "on node ~p ");
        // erts_dsprintf(dsbufp, "with exit value:~n~p~n");

        // Build the args in reverse order
        // hp = HAlloc(process, 2);
        // Args = CONS(hp, value, Args);
        // if (alive) {
        //     hp = HAlloc(process, 2);
        //     Args = CONS(hp, erts_this_node->sysname, Args);
        // }
        // hp = HAlloc(process, 2);
        // Args = CONS(hp, process->common.id, Args);

        // erts_send_error_term_to_logger(process->group_leader, dsbufp, Args);
        println!(
            "Error in process {} with exit value: {}",
            process.pid, exc.value
        );
    }
}

/// Build and add a symbolic stack trace to the error value.
pub fn add_stacktrace(process: &RcProcess, value: Term, trace: Term) -> Term {
    let heap = &process.context_mut().heap;
    let origin = build_stacktrace(process, trace);
    tup2!(heap, value, origin)
}

/// Forming the correct error value from the internal error code.
/// This does not update c_p->fvalue or c_p->freason.
fn expand_error_value(process: &RcProcess, reason: Reason, value: Term) -> Term {
    match exception_code!(reason) {
        // primary
        0 => {
            // Primary exceptions use fvalue as it is
            value
        }
        atom::BADMATCH
        | atom::CASE_CLAUSE
        | atom::TRY_CLAUSE
        | atom::BADFUN
        | atom::BADARITY
        | atom::BADKEY => {
            let heap = &process.context_mut().heap;
            //Some common exceptions: value -> {atom, value}
            //    ASSERT(is_value(Value)); TODO: check that is not non-value
            let error_atom = Term::atom(EXIT_CODES[exception_code!(reason) as usize]);
            tup2!(heap, error_atom, value)
        }
        _ => {
            // Other exceptions just use an atom as descriptor
            Term::atom(EXIT_CODES[exception_code!(reason) as usize])
        }
    }
}

/// Quick-saving the stack trace in an internal form on the heap. Note
/// that c_p->ftrace will point to a cons cell which holds the given args
/// and the saved data (encoded as a bignum).
///
/// There is an issue with line number information. Line number
/// information is associated with the address *before* an operation
/// that may fail or be stored stored on the stack. But continuation
/// pointers point after its call instruction, not before. To avoid
/// finding the wrong line number, we'll need to adjust them so that
/// they point at the beginning of the call instruction or inside the
/// call instruction. Since its impractical to point at the beginning,
/// we'll do the simplest thing and decrement the continuation pointers
/// by one.
///
/// Here is an example of what can go wrong. Without the adjustment
/// of continuation pointers, the call at line 42 below would seem to
/// be at line 43:
///
/// line 42
/// call ...
/// line 43
/// gc_bif ...
///
/// (It would be much better to put the arglist - when it exists - in the
/// error value instead of in the actual trace; e.g. '{badarg, Args}'
/// instead of using 'badarg' with Args in the trace. The arglist may
/// contain very large values, and right now they will be kept alive as
/// long as the stack trace is live. Preferably, the stack trace should
/// always be small, so that it does not matter if it is long-lived.
/// However, it is probably not possible to ever change the format of
/// error terms.)

// save_stacktrace(Process* c_p, BeamInstr* pc, Eterm* reg,
// 		ErtsCodeMFA *bif_mfa, Eterm args) {
fn save_stacktrace(
    process: &RcProcess,
    exc: &mut Exception,
    /*bif_mfa: &MFA,*/ mut args: Term,
) {
    let context = process.context_mut();
    // let pc = context.ip;
    let mut depth = DEFAULT_BACKTRACE_SIZE;
    // int depth = erts_backtrace_depth;    /* max depth (never negative) */
    if depth > 0 {
        // There will always be a current function
        depth -= 1;
    }

    let heap = &process.context_mut().heap;
    // Create a container for the exception data
    let boxed = heap.alloc(value::Boxed {
        header: value::BOXED_STACKTRACE,
        value: StackTrace {
            reason: exc.reason,
            trace: Vec::new(),
            current: context.current,
            pc: None,
            complete: false,
        },
    });

    let s = &mut boxed.value;

    // If the failure was in a BIF other than 'error/1', 'error/2', 'exit/1' or 'throw/1', save
    // BIF-MFA and save the argument registers by consing up an arglist.
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
    // 	args = make_arglist(process, &context.x, bif_mfa->arity); /* Overwrite CAR(c_p->ftrace) */
    //     } else {

    //     non_bif_stacktrace:
    /*
     * For a function_clause error, the arguments are in the beam
     * registers, c_p->cp is valid, and c_p->current is set.
     */

    if s.reason.contains(Reason::EXC_FUNCTION_CLAUSE) {
        // ASSERT(s->current);
        let a = s.current.2;
        args = make_arglist(process, a as usize); // Overwrite CAR(c_p->ftrace)
                                                  // Save first stack entry
                                                  // ASSERT(c_p->cp);
                                                  // if (depth > 0) {
                                                  //     s->trace[s->depth++] = c_p->cp - 1;
                                                  //     depth--;
                                                  // }
                                                  // s->pc = NULL; /* Ignore pc */
    } else {
        if let Some(cp) = &context.cp {
            if depth > 0
            /* != None && cp != pc*/
            {
                s.trace.push(cp.clone()); // -1
                depth -= 1;
            }
        }
        s.pc = Some(context.ip);
    }
    // }

    // Save the actual stack trace
    erts_save_stacktrace(process, &mut s.trace, depth);

    // Package args and stack trace
    // c_p->ftrace = CONS(hp, args, make_big((Eterm *) s));
    exc.trace = cons!(heap, args, Term::from(boxed));
}

pub fn erts_save_stacktrace(process: &RcProcess, trace: &mut Vec<InstrPtr>, mut depth: u32) {
    let context = process.context_mut();
    if depth == 0 {
        return;
    }
    let mut ptr = context.callstack.len();

    /*
     * Traverse the stack backwards and add all unique continuation
     * pointers to the buffer, up to the maximum stack trace size.
     *
     * Skip trace stack frames.
     */
    while ptr > 0 && depth > 0 {
        let (_, cp) = context.callstack[ptr - 1];
        if let Some(cp) = cp {
            if Some(&cp) != trace.last() {
                // Record non-duplicates only
                trace.push(cp); // -1
                depth -= 1;
            }
        }
        ptr -= 1
    }
}

// Getting the relevant fields from the term pointed to by ftrace
pub fn get_trace_from_exc(trace: &Term) -> Option<&StackTrace> {
    match trace.into_variant() {
        Variant::Nil(..) => None,
        Variant::Cons(cons) => unsafe {
            if let Ok(value) = (*cons).tail.cast_into() {
                Some(value)
            } else {
                unreachable!()
            }
        },
        _ => unreachable!(),
    }
}

pub fn get_args_from_exc(trace: Term) -> Term {
    match trace.into_variant() {
        Variant::Nil(value::Special::Nil) => Term::nil(),
        Variant::Cons(cons) => unsafe { (*cons).head },
        _ => unreachable!(),
    }
}

fn is_raised_exc(exc: Term) -> bool {
    match exc.into_variant() {
        Variant::Nil(value::Special::Nil) => false,
        Variant::Cons(cons) => unsafe {
            //return bignum_header_is_neg(*big_val(CDR(list_val(exc))));
            if let Ok(StackTrace { complete: true, .. }) = (*cons).tail.cast_into() {
                return true;
            }
            false
        },
        _ => unreachable!(),
    }
}

/// Creating a list with the argument registers
// static Eterm
fn make_arglist(process: &RcProcess, mut a: usize) -> Term {
    let context = process.context_mut();
    let mut args = Term::nil();
    while a > 0 {
        args = cons!(&context.heap, context.x[a - 1], args);
        a -= 1;
    }
    args
}

/// Building a symbolic representation of a saved stack trace. Note that
/// the exception object 'exc', unless NIL, points to a cons cell which
/// holds the given args and the quick-saved data (encoded as a bignum).
///
/// If the bignum is negative, the given args is a complete stacktrace.
pub fn build_stacktrace(process: &RcProcess, exc: Term) -> Term {
    let heap = &process.context_mut().heap;

    // TODO: awkward
    let s = get_trace_from_exc(&exc);
    if s.is_none() {
        return Term::nil();
    }
    let s = s.unwrap();

    if is_raised_exc(exc) {
        return get_args_from_exc(exc);
    }

    // Find the current function. If the saved s->pc is null, then the
    // saved s->current should already contain the proper value.
    let fi = if let Some(pc) = s.pc {
        pc.lookup_func_info()
    // } else if exception_code!(s.reason) == exception_code!(Reason::EXC_FUNCTION_CLAUSE) {
    //     // lookup_function_info(erts_codemfa_to_code(s->current), true)
    } else {
        //     s.current
        None
    };

    let depth = s.trace.len();
    // TODO: these clones are bad
    let mut trace = s.trace.clone();
    /*
     * If fi.current is still NULL, and we have no
     * stack at all, default to the initial function
     * (e.g. spawn_link(erlang, abs, [1])).
     */
    let args = if fi.is_some() {
        get_args_from_exc(exc)
    } else {
        if depth == 0 {
            // erts_set_current_function(&fi, &c_p->u.initial); loc = LINE_INVALID_LOCATION
        }
        Term::atom(atom::TRUE) // Just in case
    };

    // Allocate heap space and build the stacktrace.
    let mut res = Term::nil();
    while let Some(stack_ptr) = trace.pop() {
        let func_info = stack_ptr.lookup_func_info().unwrap();
        let mfa = erts_build_mfa_item(&func_info, heap, Term::atom(atom::TRUE));
        res = cons!(heap, mfa, res);
    }
    if let Some(fi) = fi {
        let mfa = erts_build_mfa_item(&fi, heap, args);
        res = cons!(heap, mfa, res);
    }

    // TODO: dealloc StackTrace
    res
}

/// Build a single {M,F,A,Loction} item to be part of a stack trace.
pub fn erts_build_mfa_item(fi: &(MFA, Option<FuncInfo>), heap: &Heap, args: Term) -> Term {
    let mut loc = Term::nil();

    if let Some((_file, line)) = fi.1 {
        // let file_term = if file == 0 {
        //     Atom* ap = atom_tab(atom_val(fi.mfa.module));
        //     file_term = buf_to_intlist(&hp, ".erl", 4, NIL);
        //     file_term = buf_to_intlist(&hp, (char*)ap->name, ap->len, file_term);
        // } else {
        //     file_term = erts_atom_to_string(&hp, (fi.fname_ptr)[file-1]);
        // };
        let file_term = Term::atom(2);

        let mut tuple = tup2!(heap, Term::atom(atom::LINE), Term::uint(heap, line));
        loc = cons!(heap, tuple, loc);

        tuple = tup2!(heap, Term::atom(atom::FILE), file_term);
        loc = cons!(heap, tuple, loc);
    }

    let mfa = fi.0;

    if args.is_list() || args.is_nil() {
        tup4!(heap, Term::atom(mfa.0), Term::atom(mfa.1), args, loc)
    } else {
        let arity = Term::int(mfa.2 as i32);
        tup4!(heap, Term::atom(mfa.0), Term::atom(mfa.1), arity, loc)
    }
}
