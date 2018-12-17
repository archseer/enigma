use crate::module::Module;
use crate::pool::Job;
pub use crate::process_table::PID;
use crate::value::Value;
use crate::vm::RcState;
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Heavily inspired by inko

pub type RcProcess = Arc<Process>;

pub struct ExecutionContext {
    // registers
    pub x: [Value; 16],
    pub stack: Vec<Value>,
    // program pointer/reference?
    pub ip: usize,
    // continuation pointer
    pub cp: isize, // TODO: ?!, isize is lossy here
    pub live: usize,
    // pointer to the current code
    pub module: *const Module,
}

impl ExecutionContext {
    pub fn new(module: *const Module) -> ExecutionContext {
        unsafe {
            let mut ctx = ExecutionContext {
                x: std::mem::uninitialized(), //[Value::Nil(); 16],
                stack: Vec::new(),
                ip: 0,
                cp: -1,
                live: 0,

                // register: Register::new(block.code.registers as usize),
                // binding: Binding::with_rc(block.locals(), block.receiver),
                module: module,
                // line: block.code.line,
            };
            for (_i, el) in ctx.x.iter_mut().enumerate() {
                // Overwrite `element` without running the destructor of the old value.
                // Since Value does not implement Copy, it is moved.
                std::ptr::write(el, Value::Nil());
            }
            ctx
        }
    }
}

pub struct LocalData {
    // allocator, mailbox, panic handler
    context: Box<ExecutionContext>,

    /// The ID of the thread this process is pinned to.
    pub thread_id: Option<u8>,
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
        context: ExecutionContext,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            // allocator: LocalAllocator::new(global_allocator.clone(), config),
            context: Box::new(context),
            // mailbox: Mailbox::new(global_allocator, config),
            thread_id: None,
        };

        Arc::new(Process {
            pid,
            local_data: UnsafeCell::new(local_data),
            waiting_for_message: AtomicBool::new(false),
        })
    }

    pub fn from_block(
        pid: PID,
        module: *const Module,
        // global_allocator: RcGlobalAllocator,
        // config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::new(module);

        Process::with_rc(pid, context /*global_allocator, config*/)
    }

    #[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
    pub fn context_mut(&self) -> &mut ExecutionContext {
        &mut *self.local_data_mut().context
    }

    #[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
    pub fn local_data_mut(&self) -> &mut LocalData {
        unsafe { &mut *self.local_data.get() }
    }

    pub fn local_data(&self) -> &LocalData {
        unsafe { &*self.local_data.get() }
    }

    pub fn is_main(&self) -> bool {
        self.pid == 0
    }
}

pub fn allocate(state: &RcState, module: *const Module) -> Result<RcProcess, String> {
    let mut process_table = state.process_table.lock().unwrap();

    let pid = process_table
        .reserve()
        .ok_or_else(|| "No PID could be reserved".to_string())?;

    let process = Process::from_block(
        pid, module, /*, state.global_allocator.clone(), &state.config*/
    );

    process_table.map(pid, process.clone());

    Ok(process)
}

pub fn spawn(
    state: &RcState,
    module: *const Module,
    func: usize,
    args: Value,
) -> Result<Value, String> {
    // let block_obj = block_ptr.block_value()?;
    let new_proc = allocate(&state, module)?;
    let new_pid = new_proc.pid;
    // let pid_ptr = new_proc.allocate_usize(new_pid, state.integer_prototype);
    let pid_ptr = Value::Pid(new_pid);

    state.process_pool.schedule(Job::normal(new_proc));

    Ok(pid_ptr)
}
