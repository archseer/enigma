pub use self::table::PID;
use crate::atom;
use crate::bitstring;
use crate::exception::{Exception, Reason};
use crate::immix::Heap;
use crate::instr_ptr::InstrPtr;
use crate::instruction;
use crate::mailbox::Mailbox;
use crate::module::{Module, MFA};
// use crate::servo_arc::Arc; can't do receiver self
use crate::signal_queue::SignalQueue;
pub use crate::signal_queue::{ExitKind, Signal};
use crate::value::{self, CastInto, Term};
use crate::vm::Machine;
use hashbrown::{HashMap, HashSet};
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::compat::*;
use futures::prelude::*;
use std::pin::Pin;
use tokio::prelude::*;

pub mod registry;

pub mod table;

/// Heavily inspired by inko

pub type RcProcess = Pin<Arc<Process>>;

pub type Ref = usize;

// TODO: max registers should be a MAX_REG constant for (x and freg), OTP uses 1024
// regs should be growable and shrink on live
// also, only store "live" regs in the execution context and swap them into VM/scheduler
// ---> sched should have it's own ExecutionContext
// also this way, regs could be a &mut [] slice with no clone?

pub const MAX_REG: usize = 1024;
// pub const MAX_REG: usize = 255;

bitflags! {
    pub struct Flag: u8 {
        const INITIAL = 0;
        const TRAP_EXIT = (1 << 0);
    }
}

// #[derive(Debug)]
pub struct ExecutionContext {
    /// X registers.
    pub x: [Term; MAX_REG],
    /// Floating point registers.
    pub f: [f64; 16],
    /// Stack (accessible through Y registers).
    pub stack: Vec<Term>,
    /// Stores continuation pointers
    pub callstack: Vec<(instruction::Regs, Option<InstrPtr>)>,
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
    pub bs: bitstring::Builder,
    ///
    pub exc: Option<Exception>,
    /// Reductions left
    pub reds: usize,

    /// Waker associated with the wait
    pub recv_channel: Option<futures::channel::oneshot::Receiver<()>>,
    pub timeout: Option<futures::channel::oneshot::Sender<()>>,
}

impl ExecutionContext {
    #[inline(always)]
    pub fn expand_arg(&self, arg: instruction::Source) -> Term {
        use instruction::Source;
        match arg {
            Source::X(i) => unsafe { *self.x.get_unchecked(i.0 as usize) },
            Source::Y(i) => self.stack[self.stack.len() - (i.0 + 1) as usize],
            Source::Constant(i) => unsafe { (*self.ip.module).constants[i as usize] },
            // TODO: optimize away into a reference somehow at load time
            Source::ExtendedLiteral(i) => unsafe { (*self.ip.module).literals[i as usize] },
            // Source::Constant(i) => self.i,
            // value => unreachable!("expand unimplemented for {:?}", value),
        }
    }

    // TODO: hoping to remove Entry in the future
    #[inline]
    pub fn expand_entry(&self, arg: &instruction::Entry) -> Term {
        use instruction::Entry;
        match *arg {
            // TODO: optimize away into a reference somehow at load time
            Entry::ExtendedLiteral(i) => unsafe { (*self.ip.module).literals[i as usize] },
            Entry::Value(src) => self.expand_arg(src),
            _ => unreachable!(),
        }
    }

    /// Optimization over expand_arg: only fetches X or Y.
    #[inline]
    pub fn fetch_register(&mut self, register: instruction::Register) -> Term {
        use instruction::Register;
        match register {
            Register::X(reg) => unsafe {*self.x.get_unchecked(reg.0 as usize)},
            Register::Y(reg) => self.stack[self.stack.len() - (reg.0 + 1) as usize],
        }
    }

    #[inline]
    pub fn set_register(&mut self, register: instruction::Register, value: Term) {
        use instruction::Register;
        match register {
            Register::X(reg) => unsafe {
                *self.x.get_unchecked_mut(reg.0 as usize) = value;
            }
            Register::Y(reg) => {
                let len = self.stack.len();
                self.stack[(len - (reg.0 + 1) as usize)] = value;
            }
        }
    }
}

impl ExecutionContext {
    pub fn new(module: *const Module) -> ExecutionContext {
        ExecutionContext {
            x: [Term::nil(); MAX_REG],
            f: [0.0f64; 16],
            stack: Vec::with_capacity(32),
            callstack: Vec::with_capacity(8),
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
            reds: 0,
            timeout: None,
            recv_channel: None,
        }
    }
}

bitflags! {
    pub struct StateFlag: u8 {
        const INITIAL = 0;

        const PRQ_OFFSET = 0;
        const PRQ_BITS = 3;

        const PRQ_MAX = 0b100;
        const PRQ_HIGH = 0b11;
        const PRQ_MEDIUM = 0b10;
        const PRQ_LOW = 0b01;

        const PRQ_MASK = (1 << Self::PRQ_BITS.bits) - 1;
    }
}

pub struct LocalData {
    // allocator, panic handler
    context: Box<ExecutionContext>,

    pub state: StateFlag,

    parent: PID,

    pub group_leader: PID,

    // name (atom)
    pub name: Option<u32>,

    pub initial_call: MFA,

    /// error handler, defaults to error_handler
    pub error_handler: u32,

    // links (tree)
    pub links: HashSet<PID>,
    // monitors (tree)
    pub monitors: HashMap<Ref, PID>,
    // lt_monitors (list)
    pub lt_monitors: Vec<(PID, Ref)>,

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
unsafe impl Send for ExecutionContext {}
unsafe impl Sync for ExecutionContext {}
unsafe impl Sync for Process {}
impl RefUnwindSafe for Process {}

impl Process {
    pub fn with_rc(
        pid: PID,
        parent: PID,
        group_leader: PID,
        context: ExecutionContext,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            // allocator: LocalAllocator::new(global_allocator.clone(), config),
            context: Box::new(context),
            flags: Flag::INITIAL,
            state: StateFlag::INITIAL,
            parent,
            group_leader,
            name: None,
            initial_call: MFA(0, 0, 0),
            error_handler: atom::ERROR_HANDLER,
            links: HashSet::new(),
            monitors: HashMap::new(),
            lt_monitors: Vec::new(),
            signal_queue: SignalQueue::new(),
            mailbox: Mailbox::new(),
            thread_id: None,
            dictionary: HashMap::new(),
        };

        Arc::pin(Process {
            pid,
            local_data: UnsafeCell::new(local_data),
            waiting_for_message: AtomicBool::new(false),
        })
    }

    pub fn from_block(
        pid: PID,
        parent: PID,
        group_leader: PID,
        module: *const Module,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::new(module);

        Process::with_rc(
            pid,
            parent,
            group_leader,
            context, /*global_allocator, config*/
        )
    }

    #[allow(clippy::mut_from_ref)]
    pub fn context_mut(&self) -> &mut ExecutionContext {
        &mut *self.local_data_mut().context
    }

    pub fn context(&self) -> &ExecutionContext {
        &*self.local_data_mut().context
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
        self.wake_up()
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
        self.wake_up()
    }

    // awkward result, but it works
    pub fn receive(&self) -> Result<Option<Term>, Exception> {
        let local_data = self.local_data_mut();

        if !local_data.mailbox.has_messages() {
            self.process_incoming()?
        }
        Ok(local_data.mailbox.receive())
    }

    pub fn wake_up(&self) {
        // TODO: will require locking
        self.waiting_for_message.store(false, Ordering::Relaxed);
        match self.context_mut().timeout.take() {
            Some(chan) => chan.send(()).unwrap(),
            None => (),
        };
        // TODO: pass through the chan.send result ret
    }

    pub fn set_waiting_for_message(&self, value: bool) {
        self.waiting_for_message.store(value, Ordering::Relaxed)
    }

    // we're in receive(), but ran out of internal messages, process external queue
    /// An Err signals that we're now exiting.
    pub fn process_incoming(&self) -> Result<(), Exception> {
        // we want to start tracking for new messages a lot earlier
        let context = self.context_mut();

        if context.timeout.is_none() {
            let (trigger, cancel) = futures::channel::oneshot::channel::<()>();
            context.recv_channel = Some(cancel); // TODO: if timer already set, don't set again!!!
            context.timeout = Some(trigger); // TODO: if timer already set, don't set again!!!
        }

        // get internal, if we ran out, start processing external
        while let Some(signal) = self.local_data_mut().signal_queue.receive() {
            match signal {
                Signal::Message { value, .. } => {
                    self.local_data_mut().mailbox.send(value);
                }
                Signal::PortMessage { from, value, .. } => {
                    // we only get the binary, so construct message on heap
                    let heap = &self.context_mut().heap;
                    let binary = Term::from(heap.alloc(value::Boxed {
                        header: value::BOXED_BINARY,
                        value,
                    }));
                    let msg = tup2!(heap, Term::port(from), tup2!(heap, atom!(DATA), binary));
                    self.local_data_mut().mailbox.send(msg);
                }
                Signal::Exit { .. } => {
                    self.handle_exit_signal(signal)?;
                }
                Signal::Link { from } => {
                    self.local_data_mut().links.insert(from);
                }
                Signal::Unlink { from } => {
                    self.local_data_mut().links.remove(&from);
                }
                Signal::MonitorDown { .. } => {
                    // monitor down: delete from monitors tree, deliver :down message
                    self.handle_monitor_down_signal(signal);
                }
                Signal::Monitor { from, reference } => {
                    self.local_data_mut().lt_monitors.push((from, reference));
                }
                Signal::Demonitor { from, reference } => {
                    if let Some(pos) = self
                        .local_data_mut()
                        .lt_monitors
                        .iter()
                        .position(|(x, r)| *x == from && *r == reference)
                    {
                        self.local_data_mut().lt_monitors.remove(pos);
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_monitor_down_signal(&self, signal: Signal) {
        // Create a 'DOWN' message and replace the signal with it...
        if let Signal::MonitorDown {
            from,
            reason,
            reference,
        } = signal
        {
            // assert!(is_immed(reason));
            let heap = &self.context_mut().heap;
            let from = Term::pid(from);
            let reference = Term::reference(heap, reference as usize);
            let reason = reason.value;

            let msg = tup!(heap, atom!(DOWN_U), reference, atom!(PROCESS), from, reason);
            self.local_data_mut().mailbox.send(msg);
        // bump reds by 8?
        } else {
            unreachable!();
        }
    }

    /// Return value is true if the process is now terminating.
    pub fn handle_exit_signal(&self, signal: Signal) -> Result<(), Exception> {
        // this is extremely awkward, wish we could enforce a signal variant on the function signature
        // we're also technically matching twice since process_incoming also pattern matches.
        // TODO: inline?
        if let Signal::Exit { kind, from, reason } = signal {
            let mut reason = reason.value;
            let local_data = self.local_data_mut();

            if kind == ExitKind::ExitLinked {
                // delete from link tree
                if local_data.links.take(&from).is_none() {
                    // if it was already deleted, ignore
                    return Ok(());
                }
            }

            if reason != atom!(KILL) && local_data.flags.contains(Flag::TRAP_EXIT) {
                // if reason is immed, create an EXIT message tuple instead and replace
                // (push to internal msg queue as message)
                let msg = tup3!(
                    &self.context_mut().heap,
                    atom!(EXIT_U),
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

                // return an exception to trigger process exit
                Err(Exception::with_value(Reason::EXT_EXIT, reason))
            }
        // if destroy { cleanup messages up to signal? }
        } else {
            unreachable!()
        }
    }

    // equivalent of erts_continue_exit_process
    pub fn exit(&self, vm: &Machine, reason: Exception) {
        let local_data = self.local_data_mut();

        // print!("pid={} exiting reason={}\r\n", self.pid, reason.value);

        // set state to exiting

        // TODO: cancel timers

        // TODO: unregister process name

        // delete links
        for pid in local_data.links.drain() {
            // TODO: reason has to be deep cloned, make a constructor
            // println!("pid={} sending exit signal to from={}", self.pid, pid);
            let msg = Signal::Exit {
                reason: reason.clone(),
                from: self.pid,
                kind: ExitKind::ExitLinked,
            };
            self::send_signal(vm, pid, msg);
            // erts_proc_sig_send_link_exit(c_p, c_p->common.id, lnk, reason, SEQ_TRACE_TOKEN(c_p));
        }

        // delete monitors
        for (reference, pid) in local_data.monitors.drain() {
            // we're watching someone else
            // send_demonitor(mon)
            let msg = Signal::Demonitor {
                from: self.pid,
                reference,
            };
            self::send_signal(vm, pid, msg);
        }

        for (pid, reference) in local_data.lt_monitors.drain(..) {
            // we're being watched
            // send_monitor_down(mon, reason)
            let msg = Signal::MonitorDown {
                reason: reason.clone(),
                from: self.pid,
                reference,
            };
            self::send_signal(vm, pid, msg);
        }
    }
}

pub fn allocate(
    vm: &Machine,
    parent: PID,
    group_leader: PID,
    module: *const Module,
) -> Result<RcProcess, Exception> {
    let mut process_table = vm.process_table.lock();

    let pid = process_table
        .reserve()
        .ok_or_else(|| Exception::new(Reason::EXC_SYSTEM_LIMIT))?;

    let process = Process::from_block(
        pid,
        parent,
        group_leader,
        module, /*, vm.global_allocator.clone(), &vm.config*/
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
    vm: &Machine,
    parent: &RcProcess,
    module: *const Module,
    func: u32,
    args: Term,
    flags: SpawnFlag,
) -> Result<Term, Exception> {
    let new_proc = allocate(vm, parent.pid, parent.local_data().group_leader, module)?;
    let context = new_proc.context_mut();
    let mut ret = Term::pid(new_proc.pid);

    // Set the arglist into process registers.
    // TODO: it also needs to deep clone all the vals (for example lists etc)
    let mut i = 0;
    let mut cons = &args;
    while let Ok(value::Cons { head, tail }) = cons.cast_into() {
        context.x[i] = *head;
        i += 1;
        cons = tail;
    }
    // lastly, the tail
    context.x[i] = *cons;

    new_proc.local_data_mut().initial_call = MFA(unsafe { (*module).name }, func, i as u32);

    // print!(
    //     "Spawning... pid={} mfa={} args={}\r\n",
    //     new_proc.pid,
    //     new_proc.local_data().initial_call,
    //     args
    // );

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
        new_proc.local_data_mut().links.insert(parent.pid);

        parent.local_data_mut().links.insert(new_proc.pid);
    }

    if flags.contains(SpawnFlag::MONITOR) {
        let reference = vm.next_ref();

        parent
            .local_data_mut()
            .monitors
            .insert(reference, new_proc.pid);

        new_proc
            .local_data_mut()
            .lt_monitors
            .push((parent.pid, reference));

        let heap = &parent.context_mut().heap;
        ret = tup2!(heap, ret, Term::reference(heap, reference))
    }

    let future = crate::vm::run_with_error_handling(new_proc);
    vm.process_pool
        .executor()
        .spawn(future.unit_error().boxed().compat());

    Ok(ret)
}

pub fn send_message(vm: &Machine, sender: PID, pid: Term, msg: Term) -> Result<Term, Exception> {
    // println!("sending from={} to={}, msg={}", sender, pid, msg);
    let receiver = match pid.into_variant() {
        value::Variant::Atom(name) => {
            if let Some(process) = vm.process_registry.lock().whereis(name) {
                Some(process.clone())
            } else {
                println!("registered name {} not found!", pid);
                return Err(Exception::new(Reason::EXC_BADARG));
            }
        }
        value::Variant::Pid(pid) => vm.process_table.lock().get(pid),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    if let Some(receiver) = receiver {
        receiver.send_message(sender, msg);
    } else {
        println!("NOTFOUND");
    }
    // TODO: if err, we return err that's then put in x0?

    Ok(msg)
}

pub fn send_signal(vm: &Machine, pid: PID, signal: Signal) -> bool {
    if let Some(receiver) = vm.process_table.lock().get(pid) {
        receiver.send_signal(signal);
        return true;
    }
    // TODO: if err, we return err ?
    false
}

pub enum State {
    Done,
    Yield,
}
