pub use self::table::PID;
use crate::atom;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::loader::LValue;
use crate::mailbox::Mailbox;
use crate::module::{Module, MFA};
use crate::pool::Job;
use crate::servo_arc::Arc;
use crate::signal_queue::SignalQueue;
pub use crate::signal_queue::{ExitKind, Signal};
use crate::value::{self, Term, TryInto};
use crate::vm::RcState;
use hashbrown::HashMap;
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};

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

            current: MFA(0, 0, 0),

            // register: Register::new(block.code.registers as usize),
            // binding: Binding::with_rc(block.locals(), block.receiver),
            // line: block.code.line,

            // TODO: not great
            bs: unsafe { std::mem::uninitialized() },
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
    pub signal_queue: SignalQueue,

    pub mailbox: Mailbox,

    pub flags: Flag,

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
            flags: Flag::INITIAL,
            parent,
            links: tree::Tree::new(tree::NodeAdapter::new()), // eew
            signal_queue: SignalQueue::new(),
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

    pub fn send_signal(&self, signal: Signal) {
        self.local_data_mut().signal_queue.send_external(signal);
    }

    // TODO: remove
    pub fn send_message(&self, from: PID, message: Term) {
        if from == self.pid {
            // skip the signal_queue completely
            self.local_data_mut().mailbox.send(message);
        } else {
            self.local_data_mut()
                .signal_queue
                .send_external(Signal::Message {
                    value: message,
                    from,
                });
        }
    }

    // awkward result, but it works
    pub fn receive(&self) -> Result<Option<&Term>, Exception> {
        let local_data = self.local_data_mut();

        println!("receiving");
        if !local_data.mailbox.has_messages() {
            self.process_incoming()?
        }
        Ok(local_data.mailbox.receive())
    }

    pub fn set_waiting_for_message(&self, value: bool) {
        self.waiting_for_message.store(value, Ordering::Relaxed);
    }

    pub fn is_waiting_for_message(&self) -> bool {
        self.waiting_for_message.load(Ordering::Relaxed)
    }

    // we're in receive(), but ran out of internal messages, process external queue
    /// An Err signals that we're now exiting.
    pub fn process_incoming(&self) -> Result<(), Exception> {
        // get internal, if we ran out, start processing external
        println!("signals");
        while let Some(signal) = self.local_data_mut().signal_queue.receive() {
            match signal {
                Signal::Message { value, .. } => {
                    self.local_data_mut().mailbox.send(value);
                }
                Signal::Exit { .. } => {
                    self.handle_exit_signal(signal)?;
                }
            }
        }
        return Ok(());
    }

    /// Return value is true if the process is now terminating.
    pub fn handle_exit_signal(&self, signal: Signal) -> Result<(), Exception> {
        // this is extremely awkward, wish we could enforce a signal variant on the function signature
        // we're also technically matching twice since process_incoming also pattern matches.
        // TODO: inline?
        println!("exit signals");
        if let Signal::Exit { kind, from, reason } = signal {
            let mut reason = reason.value;
            let local_data = self.local_data_mut();

            if kind == ExitKind::ExitLinked {
                // delete from link tree
                if local_data.links.find_mut(&from).remove().is_none() {
                    // if it was already deleted, ignore
                    return Ok(());
                }
            }

            if reason != atom!(KILL) && local_data.flags.contains(Flag::TRAP_EXIT) {
                // if reason is immed, create an EXIT message tuple instead and replace
                // (push to internal msg queue as message)
                let msg = tup3!(
                    &self.context_mut().heap,
                    atom!(EXIT),
                    Term::pid(from),
                    reason
                );
                // TODO: ensure we do process wakeup
                // erts_proc_notify_new_message(c_p, ERTS_PROC_LOCK_MAIN);
                local_data.mailbox.send(msg);
                Ok(())
            } else if reason == atom!(NORMAL)
            /*&& xsigd.u.normal_kills */
            {
                /* TODO: for exit/2, exit_signal/2 implement normal kills
                 * Preserve the very old and *very strange* behaviour
                 * of erlang:exit/2...
                 *
                 * - terminate ourselves even though exit reason
                 *   is normal (unless we trap exit)
                 * - terminate ourselves before exit/2 return
                 */

                // ignore
                Ok(())
            } else {
                // terminate
                // save = true;
                if
                /*op == ERTS_SIG_Q_OP_EXIT && */
                reason == atom!(KILL) {
                    reason = atom!(KILLED);
                }

                // if save { // something to do with heap fragments I think mainly to remove it from proc
                //     sig->data.attached = ERTS_MSG_COMBINED_HFRAG;
                //     ERL_MESSAGE_TERM(sig) = xsigd->message;
                //     erts_save_message_in_proc(c_p, sig);
                // }

                // Exit process...

                // kill catches
                self.context_mut().catches = 0;
                println!("resetting catches");

                // return an exception to trigger process exit
                Err(Exception::with_value(Reason::EXT_EXIT, reason))
            }
        // if destroy { cleanup messages up to signal? }
        } else {
            unreachable!()
        }
    }

    // equivalent of erts_continue_exit_process
    pub fn exit(&self, state: &RcState, reason: Exception) {
        let local_data = self.local_data_mut();

        // set state to exiting

        // TODO: cancel timers

        // TODO: unregister process name

        // delete links
        let mut cursor = local_data.links.front_mut();
        while let Some(link) = cursor.remove() {
            // TODO: reason has to be deep cloned, make a constructor
            println!("sending exit signal");
            let msg = Signal::Exit {
                reason: reason.clone(),
                from: self.pid,
                kind: ExitKind::ExitLinked,
            };
            self::send_signal(state, link.other, msg);
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
        receiver.send_message(process.pid, msg);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.set_waiting_for_message(false);

            state.process_pool.schedule(Job::normal(receiver));
        }
    }
    // TODO: if err, we return err that's then put in x0?

    Ok(msg)
}

pub fn send_signal(state: &RcState, pid: PID, signal: Signal) -> Result<(), Exception> {
    if let Some(receiver) = state.process_table.lock().get(pid) {
        receiver.send_signal(signal);

        if receiver.is_waiting_for_message() {
            // wake up
            receiver.set_waiting_for_message(false);

            state.process_pool.schedule(Job::normal(receiver));
        }
    }
    // TODO: if err, we return err ?

    Ok(())
}
