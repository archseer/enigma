use crate::pool::Job;
use crate::process_table::PID;
use crate::vm;
use std::cell::UnsafeCell;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Heavily inspired by inko

pub type RcProcess = Arc<Process>;

pub struct LocalData {}

pub struct Process {
    /// Data stored in a process that should only be modified by a single thread
    /// at once.
    pub context: UnsafeCell<LocalData>,

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
        global_allocator: RcGlobalAllocator,
        config: &Config,
    ) -> RcProcess {
        let local_data = LocalData {
            allocator: LocalAllocator::new(global_allocator.clone(), config),
            context: Box::new(context),
            mailbox: Mailbox::new(global_allocator, config),
            panic_handler: ObjectPointer::null(),
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
        block: &Block,
        global_allocator: RcGlobalAllocator,
        config: &Config,
    ) -> RcProcess {
        let context = ExecutionContext::from_isolated_block(block);

        Process::with_rc(pid, context, global_allocator, config)
    }
}

pub fn allocate(state: &vm::Machine, block: &Block) -> Result<RcProcess, String> {
    let mut process_table = state.process_table.lock().unwrap();

    let pid = process_table
        .reserve()
        .ok_or_else(|| "No PID could be reserved".to_string())?;

    let process = Process::from_block(pid, block, state.global_allocator.clone(), &state.config);

    process_table.map(pid, process.clone());

    Ok(process)
}

pub fn spawn(state: &vm::Machine, block_ptr: ObjectPointer) -> Result<ObjectPointer, String> {
    let block_obj = block_ptr.block_value()?;
    let new_proc = allocate(&state, block_obj)?;
    let new_pid = new_proc.pid;
    let pid_ptr = new_proc.allocate_usize(new_pid, state.integer_prototype);

    state.process_pool.schedule(Job::normal(new_proc));

    Ok(pid_ptr)
}
