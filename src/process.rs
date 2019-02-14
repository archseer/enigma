pub use self::table::PID;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader::LValue;
use crate::mailbox::Mailbox;
pub use crate::mailbox::Signal;
use crate::module::{Module, MFA};
use crate::pool::Job;
use crate::value::{self, Term, TryInto};
use crate::vm::RcState;
use hashbrown::HashMap;
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::servo_arc::Arc;

pub mod registry;
pub mod table;
pub mod tree;

/// Heavily inspired by inko

pub type RcProcess = Arc<Process>;

// TODO: max registers should be a MAX_REG constant for (x and freg), OTP uses 1024
// regs should be growable and shrink on live
// also, only store "live" regs in the execution context and swap them into VM/scheduler
// ---> sched should have it's own ExecutionContext
// also this way, regs could be a &mut [] slice with no clone?

pub const MAX_REG: usize = 16;

bitflags! {
    pub struct Flag: u8 {
        const INITIAL = 0;
        const TRAP_EXIT = (1 << 0);
    }
}

pub struct ExecutionContext {
    /// X registers.
    pub x: [Term; MAX_REG],
    /// Floating point registers.
    pub f: [f64; MAX_REG],
    /// Stack (accessible through Y registers).
    pub stack: Vec<Term>,
    pub heap: Heap,
    /// Number of catches on stack.
    pub catches: usize,
    /// Program pointer, points to the current instruction.
    pub ip: InstrPtr,
    /// Continuation pointer
    pub cp: Option<InstrPtr>,
    /// Current function
    pub current: MFA,
    pub live: usize,
    /// binary construction state
    pub bs: *mut Vec<u8>,
    ///
    pub exc: Option<Exception>,
    pub flags: Flag,
}

impl ExecutionContext {
    #[inline]
    // TODO: expand_arg should return by value
    pub fn expand_arg(&self, arg: &LValue) -> Term {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            LValue::ExtendedLiteral(i) => unsafe { (*self.ip.module).literals[*i as usize] },
            LValue::X(i) => self.x[*i as usize],
            LValue::Y(i) => self.stack[self.stack.len() - (*i + 2) as usize],
            LValue::Integer(i) => Term::int(*i as i32), // TODO: make LValue i32
            LValue::Atom(i) => Term::atom(*i),
            LValue::Nil => Term::nil(),
            value => unimplemented!("expand unimplemented for {:?}", value),
        }
    }
}

impl ExecutionContext {
    pub fn new(module: *const Module) -> ExecutionContext {
        ExecutionContext {
            x: [Term::nil(); 16],
            f: [0.0f64; 16],
            stack: Vec::new(),
            heap: Heap::new(),
            catches: 0,
            ip: InstrPtr { ptr: 0, module },
            cp: None,
            live: 0,

            exc: None,

            current: (0, 0, 0),

            // register: Register::new(block.code.registers as usize),
            // binding: Binding::with_rc(block.locals(), block.receiver),
            // line: block.code.line,

            // TODO: not great
            bs: unsafe { std::mem::uninitialized() },

            flags: Flag::INITIAL,
        }
    }
}

pub struct LocalData {
    // allocator, panic handler
    context: Box<ExecutionContext>,

    parent: Option<PID>,

    // links (tree)
    links: tree::Tree,
    // monitors (tree) + lt_monitors (list)

    // signals are sent on death, and the receiving side cleans up it's link/mon structures
    pub mailbox: Mailbox,

    /// The ID of the thread this process is pinned to.
    pub thread_id: Option<u8>,

    /// A [process dictionary](https://www.erlang.org/course/advanced#dict)
    pub dictionary: HashMap<Term, Term>,
}

pub struct Process {
    /// Data stored in a process that should only be modified by a single thread
    /// at once.
    pub local_data: UnsafeCell<LocalData>,

    /// The process identifier of this process.
    pub pid: PID,

    /// If the process is waiting for a message.
    pub waiting_for_message: AtomicBool,
}

unsafe impl Sync for LocalData {}
unsafe impl Send for LocalData {}
unsafe impl Sync for Process {}
impl RefUnwindSafe for Process {}

impl Process {
    pub fn with_rc(
        pid: PID,
        parent: Option<PID>,
        context: ExecutionContext,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            // allocator: LocalAllocator::new(global_allocator.clone(), config),
            context: Box::new(context),
            parent,
            links: tree::Tree::new(tree::NodeAdapter::new()), // eew
            mailbox: Mailbox::new(),
            thread_id: None,
            dictionary: HashMap::new(),
        };

        Arc::new(Process {
            pid,
            local_data: UnsafeCell::new(local_data),
            waiting_for_message: AtomicBool::new(false),
        })
    }

    pub fn from_block(
        pid: PID,
        parent: Option<PID>,
        module: *const Module,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::new(module);

        Process::with_rc(pid, parent, context /*global_allocator, config*/)
    }

    #[allow(clippy::mut_from_ref)]
    pub fn context_mut(&self) -> &mut ExecutionContext {
        &mut *self.local_data_mut().context
    }

    #[allow(clippy::mut_from_ref)]
    pub fn local_data_mut(&self) -> &mut LocalData {
        unsafe { &mut *self.local_data.get() }
    }

    pub fn local_data(&self) -> &LocalData {
        unsafe { &*self.local_data.get() }
    }

    pub fn is_main(&self) -> bool {
        self.pid == 0
    }

    pub fn send_message(&self, sender: &RcProcess, message: Term) {
        if sender.pid == self.pid {
            self.local_data_mut().mailbox.send_internal(message);
        } else {
            self.local_data_mut().mailbox.send_external(Signal::Message(message));
        }
    }

    pub fn set_waiting_for_message(&self, value: bool) {
        self.waiting_for_message.store(value, Ordering::Relaxed);
    }

    pub fn is_waiting_for_message(&self) -> bool {
        self.waiting_for_message.load(Ordering::Relaxed)
    }

    pub fn receive(&self) -> Option<&Signal> {
        // get internal, if we ran out, start processing external
    }

    // we're in receive(), but ran out of internal messages, process external queue
    pub fn handle_incoming_signal(&self, signal: Signal) {
        match signal {
            Signal::Message(message) => {
                self.local_data_mut().mailbox.send_internal(message);
            }
            Signal::Exit(reason) => {

            }
        }
    }

    // handle_exit_signal(Process *c_p, ErtsSigRecvTracing *tracing,
    //                 ErtsMessage *sig, ErtsMessage ***next_nm_sig,
    //                 int *exited)
    pub fn handle_exit_signal(&self, signal: Signal) {
        // ErtsMessage *conv_msg = NULL;
        // ErtsExitSignalData *xsigd = NULL;
        // ErtsLinkData *ldp = NULL; /* Avoid erroneous warning... */
        // ErtsLink *dlnk = NULL; /* Avoid erroneous warning... */
        // Eterm tag = ((ErtsSignal *) sig)->common.tag;
        // Uint16 type = ERTS_PROC_SIG_TYPE(tag);
        // int op = ERTS_PROC_SIG_OP(tag);
        // int destroy = 0;
        // int ignore = 0;
        // int save = 0;
        // int exit = 0;
        // int cnt = 1;
        // Eterm reason;
        // Eterm from;

        // if exit 
        // if link_exit

        if (type == ERTS_SIG_Q_TYPE_GEN_EXIT) {
            xsigd = get_exit_signal_data(sig);
            from = xsigd->from;
            reason = xsigd->reason;
            if (op != ERTS_SIG_Q_OP_EXIT_LINKED)
                ignore = 0;
            else {
                ErtsLink *llnk = erts_link_tree_lookup(ERTS_P_LINKS(c_p), from);
                if (!llnk) {
                    /* Link no longer active; ignore... */
                    ignore = !0;
                    destroy = !0;
                }
                else {
                    ignore = 0;
                    erts_link_tree_delete(&ERTS_P_LINKS(c_p), llnk);
                    if (llnk->type != ERTS_LNK_TYPE_DIST_PROC)
                        erts_link_release(llnk);
                    else {
                        dlnk = erts_link_to_other(llnk, &ldp);
                        if (erts_link_dist_delete(dlnk))
                            erts_link_release_both(ldp);
                        else
                            erts_link_release(llnk);
                    }
                }
            }

            if (!ignore) {

                if ((op != ERTS_SIG_Q_OP_EXIT || reason != am_kill)
                    && (c_p->flags & F_TRAP_EXIT)) {
                    convert_prepared_sig_to_msg(c_p, sig,
                                                xsigd->message, next_nm_sig);
                    conv_msg = sig;
                }
                else if (reason == am_normal && !xsigd->u.normal_kills) {
                    /* Ignore it... */
                    destroy = !0;
                    ignore = !0;
                }
                else {
                    /* Terminate... */
                    save = !0;
                    exit = !0;
                    if (op == ERTS_SIG_Q_OP_EXIT && reason == am_kill)
                        reason = am_killed;
                }
            }
        }
        else { /* Link exit */
            ErtsLink *slnk = (ErtsLink *) sig;
            ErtsLink *llnk = erts_link_to_other(slnk, &ldp);

            ASSERT(type == ERTS_LNK_TYPE_PROC
                || type == ERTS_LNK_TYPE_PORT
                || type == ERTS_LNK_TYPE_DIST_PROC);

            from = llnk->other.item;
            reason = slnk->other.item; /* reason in other.item ... */
            ASSERT(is_pid(from) || is_internal_port(from));
            ASSERT(is_immed(reason));
            ASSERT(op == ERTS_SIG_Q_OP_EXIT_LINKED);
            dlnk = erts_link_tree_key_delete(&ERTS_P_LINKS(c_p), llnk);
            if (!dlnk) {
                ignore = !0; /* Link no longer active; ignore... */
                ldp = NULL;
            }
            else {
                Eterm pid;
                ErtsMessage *mp;
                ErtsProcLocks locks;
                Uint hsz;
                Eterm *hp;
                ErlOffHeap *ohp;
                ignore = 0;
                if (dlnk == llnk)
                    dlnk = NULL;
                else
                    ldp = NULL;

                ASSERT(is_immed(reason));

                if (!(c_p->flags & F_TRAP_EXIT)) {
                    if (reason == am_normal)
                        ignore = !0; /* Ignore it... */
                    else
                        exit = !0; /* Terminate... */
                }
                else {

                    /*
                    * Create and EXIT message and replace
                    * the original signal with the message...
                    */

                    locks = ERTS_PROC_LOCK_MAIN;

                    hsz = 4 + NC_HEAP_SIZE(from);

                    mp = erts_alloc_message_heap(c_p, &locks, hsz, &hp, &ohp);

                    if (locks != ERTS_PROC_LOCK_MAIN)
                        erts_proc_unlock(c_p, locks & ~ERTS_PROC_LOCK_MAIN);

                    pid = STORE_NC(&hp, ohp, from);

                    ERL_MESSAGE_TERM(mp) = TUPLE3(hp, am_EXIT, pid, reason);
                    ERL_MESSAGE_TOKEN(mp) = am_undefined;
                    if (is_immed(pid))
                        ERL_MESSAGE_FROM(mp) = pid;
                    else {
                        DistEntry *dep;
                        ASSERT(is_external_pid(pid));
                        dep = external_pid_dist_entry(pid);
                        ERL_MESSAGE_FROM(mp) = dep->sysname;
                    }

                    /* Replace original signal with the exit message... */
                    convert_to_msg(c_p, sig, mp, next_nm_sig);

                    cnt += 4;

                    conv_msg = mp;
                }
            }
            destroy = !0;
        }

        if (ignore|exit) {
            remove_nm_sig(c_p, sig, next_nm_sig);
            if (exit) {
                if (save) {
                    sig->data.attached = ERTS_MSG_COMBINED_HFRAG;
                    ERL_MESSAGE_TERM(sig) = xsigd->message;
                    erts_save_message_in_proc(c_p, sig);
                }
                /* Exit process... */
                erts_set_self_exiting(c_p, reason);

                cnt++;
            }
        }

        if (!exit) {
            if (conv_msg)
                erts_proc_notify_new_message(c_p, ERTS_PROC_LOCK_MAIN);
            if (op == ERTS_SIG_Q_OP_EXIT_LINKED && tracing->procs)
                getting_unlinked(c_p, from);
        }

        if (destroy) {
            cnt++;
            if (type == ERTS_SIG_Q_TYPE_GEN_EXIT) {
                sig->next = NULL;
                erts_cleanup_messages(sig);
            }
            else {
                if (ldp)
                    erts_link_release_both(ldp);
                else {
                    if (dlnk)
                        erts_link_release(dlnk);
                    erts_link_release((ErtsLink *) sig);
                }
            }
        }

        *exited = exit;

        return cnt;
    }

    // equivalent of erts_continue_exit_process
    pub fn exit(&self, state: &RcState, reason: Exception) {
        let local_data = self.local_data();

        // TODO: cancel timers

        // TODO: unregister process name

        // TODO: delete links
        for link in local_data.links.iter() {
            let msg = Signal::Exit(reason);
            self::send_message(state, self, link.other, msg);
            // erts_proc_sig_send_link_exit(c_p, c_p->common.id, lnk, reason, SEQ_TRACE_TOKEN(c_p));
        }

        // TODO: delete monitors
    }
}

pub fn allocate(
    state: &RcState,
    parent: Option<PID>,
    module: *const Module,
) -> Result<RcProcess, Exception> {
    let mut process_table = state.process_table.lock();

    let pid = process_table
        .reserve()
        .ok_or_else(|| Exception::new(Reason::EXC_SYSTEM_LIMIT))?;

    let process = Process::from_block(
        pid, parent, module, /*, state.global_allocator.clone(), &state.config*/
    );

    process_table.map(pid, process.clone());

    Ok(process)
}

bitflags! {
    pub struct SpawnFlag: u8 {
        const NONE = 0;
        const LINK = 1;
        const MONITOR = 2;
        // const USE_ARGS = 4;
        // const SYSTEM_PROC = 8;
        // const OFF_HEAP_MSGQ = 16;
        // const ON_HEAP_MSGQ = 32;
    }
}

pub fn spawn(
    state: &RcState,
    parent: &RcProcess,
    module: *const Module,
    func: u32,
    args: Term,
    flags: SpawnFlag,
) -> Result<Term, Exception> {
    println!("Spawning..");
    let new_proc = allocate(state, Some(parent.pid), module)?;
    let context = new_proc.context_mut();
    let pid = Term::pid(new_proc.pid);

    // Set the arglist into process registers.
    // TODO: it also needs to deep clone all the vals (for example lists etc)
    let mut i = 0;
    let mut cons = &args;
    while let Ok(value::Cons { head, tail }) = cons.try_into() {
        context.x[i] = *head;
        i += 1;
        cons = tail;
    }
    // lastly, the tail
    context.x[i] = *cons;

    // TODO: func to ip offset
    let func = unsafe {
        (*module)
            .funs
            .get(&(func, i as u32)) // arglist arity
            .expect("process::spawn could not locate func")
    };

    context.ip.ptr = *func;

    // Check if this process should be initially linked to its parent.
    if flags.contains(SpawnFlag::LINK) {
        new_proc.local_data_mut().links.insert(Arc::new(tree::Node {
            link: tree::Link::new(),
            other: parent.pid,
        }));

        // TODO: add constructor for Node
        parent.local_data_mut().links.insert(Arc::new(tree::Node {
            link: tree::Link::new(),
            other: new_proc.pid,
        }));
    }

    state.process_pool.schedule(Job::normal(new_proc));

    Ok(pid)
}

pub fn send_message(
    state: &RcState,
    process: &RcProcess,
    pid: PID,
    msg: Term,
) -> Result<Term, Exception> {
    if let Some(receiver) = state.process_table.lock().get(pid) {
        receiver.send_message(process, msg);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.set_waiting_for_message(false);

            state.process_pool.schedule(Job::normal(receiver));
        }
    }
    // TODO: if err, we return err that's then put in x0?

    Ok(msg)
}
