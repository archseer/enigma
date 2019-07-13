use crate::atom::Atom;
use crate::{bitstring, module, instruction};
use crate::exception::{self, Exception, Reason};
use crate::process::{self, RcProcess};
use crate::servo_arc::Arc;
use crate::value::{Cons, Term};

use crate::ets::{RcTableRegistry, TableRegistry};
use crate::exports_table::ExportsTable;
use crate::module_registry::ModuleRegistry;
use crate::port::{Table as PortTable, RcTable as RcPortTable};
use crate::persistent_term::{Table as PersistentTermTable};
use crate::process::{
    registry::Registry as ProcessRegistry,
    table::Table as ProcessTable,
};

use std::cell::RefCell;
// use log::debug;
use parking_lot::{Mutex, RwLock};
use std::panic;
use std::sync::atomic::AtomicUsize;
use std::time;

// use tokio::prelude::*;
use futures::{
    prelude::*,
};

/// A reference counted State.
pub type RcMachine = Arc<Machine>;

pub struct Machine {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,

    /// Table containing named processes.
    pub process_registry: Mutex<ProcessRegistry<RcProcess>>,

    pub port_table: RcPortTable,
    /// TODO: Use priorities later on

    /// Start time of the VM boot-up
    pub start_time: time::Instant,

    pub next_ref: AtomicUsize,

    /// PID pointing to the process handling system-wide logging.
    pub system_logger: AtomicUsize,

    /// Futures pool for running processes
    pub process_pool: tokio::runtime::Runtime,

    /// Futures pool for running I/O and other utility tasks ("dirty scheduler")
    pub runtime: tokio::runtime::Runtime,

    pub exit: Option<futures::channel::oneshot::Sender<()>>,

    /// Exports table
    pub exports: RwLock<ExportsTable>,

    /// Module registry
    pub modules: Mutex<ModuleRegistry>,

    pub ets_tables: RcTableRegistry,

    pub persistent_terms: PersistentTermTable,
}

thread_local!(
    static CURRENT: RefCell<Option<Arc<Machine>>> = RefCell::new(None);
);

impl Machine {
    /// Get current running machine.
    pub fn current() -> Arc<Machine> {
        CURRENT.with(|cell| match *cell.borrow() {
            Some(ref vm) => vm.clone(),
            None => panic!("Machine is not running"),
        })
    }

    /// Set current running machine.
    pub(crate) fn is_set() -> bool {
        CURRENT.with(|cell| cell.borrow().is_some())
    }

    /// Set current running machine.
    #[doc(hidden)]
    pub fn set_current(vm: Arc<Machine>) {
        CURRENT.with(|s| {
            *s.borrow_mut() = Some(vm);
        })
    }

    /// Execute function with machine reference.
    pub fn with_current<F, R>(f: F) -> R
    where
        F: FnOnce(&Machine) -> R,
    {
        CURRENT.with(|cell| match *cell.borrow_mut() {
            Some(ref vm) => f(vm),
            None => panic!("Machine is not running"),
        })
    }
}

pub const PRE_LOADED_NAMES: &[&str] = &[
    "erts_code_purger",
    "erl_init",
    "init",
    "prim_buffer",
    "prim_eval",
    "prim_inet",
    "prim_file",
    "zlib",
    "prim_zip",
    "erl_prim_loader",
    "erlang",
    "erts_internal",
    "erl_tracer",
    "erts_literal_area_collector",
    "erts_dirty_process_signal_handler",
    "atomics",
    "counters",
    "persistent_term",
];

/// Embedded bytecode for OTP bootstrap
pub const PRE_LOADED: &[&[u8]] = &[
    include_bytes!("../../otp/erts/preloaded/ebin/erts_code_purger.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erl_init.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/init.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/prim_buffer.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/prim_eval.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/prim_inet.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/prim_file.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/zlib.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/prim_zip.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erl_prim_loader.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erlang.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erts_internal.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erl_tracer.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erts_literal_area_collector.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/erts_dirty_process_signal_handler.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/atomics.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/counters.beam"),
    include_bytes!("../../otp/erts/preloaded/ebin/persistent_term.beam"),
];

impl Machine {
    pub fn new() -> Arc<Machine> {
        let vm = Arc::new(Machine {
            process_table: Mutex::new(ProcessTable::new()),
            process_registry: Mutex::new(ProcessRegistry::new()),
            port_table: PortTable::new(),
            start_time: time::Instant::now(),
            process_pool: unsafe { std::mem::uninitialized() }, // I'm sorry, but we need a ref to vm in threadpool
            runtime: unsafe { std::mem::uninitialized() }, // I'm sorry, but we need a ref to vm in threadpool
            exit: None,
            next_ref: AtomicUsize::new(1),
            system_logger: AtomicUsize::new(0),
            exports: ExportsTable::with_rc(),
            modules: ModuleRegistry::with_rc(),
            ets_tables: TableRegistry::with_rc(),
            persistent_terms: PersistentTermTable::new(),
        });

        // initialize tokio here

        // let (trigger, exit) = futures::sync::oneshot::channel();

        // Create the runtime
        let machine = vm.clone();
        let runtime = tokio::runtime::Builder::new()
            .panic_handler(|err| std::panic::resume_unwind(err))
            .after_start(move || {
                Machine::set_current(machine.clone()); // ughh double clone
            })
            .build()
            .expect("failed to start new Runtime");

        let machine = vm.clone();
        let process_pool = tokio::runtime::Builder::new()
            .panic_handler(|err| std::panic::resume_unwind(err))
            .after_start(move || {
                Machine::set_current(machine.clone());
            })
            .build()
            .expect("failed to start new Runtime");

        unsafe {
            std::ptr::write(&vm.runtime as *const tokio::runtime::Runtime as *mut tokio::runtime::Runtime, runtime);
            std::ptr::write(&vm.process_pool as *const tokio::runtime::Runtime as *mut tokio::runtime::Runtime, process_pool);
        }

        vm
    }

    pub fn next_ref(&self) -> process::Ref {
        self.next_ref
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Elapsed time since VM start
    pub fn elapsed_time(&self) -> time::Duration {
        self.start_time.elapsed()
    }

    /// Preload all the bootstrap modules
    pub fn preload_modules(&self) {
        PRE_LOADED.iter().for_each(|bytecode| {
            module::load_bytes(self, bytecode).unwrap();
        })
    }

    /// Starts the VM
    ///
    /// This method will block the calling thread until it returns.
    ///
    /// This method returns true if the VM terminated successfully, false
    /// otherwise.
    pub fn start(self: &Arc<Self>, args: Vec<String>) {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();

        let vm = unsafe { &mut *(&**self as *const Machine as *mut Machine) };

        vm.exit = Some(tx);

        self.start_main_process(args);

        // Wait until the runtime becomes idle and shut it down.
        vm.runtime.block_on(async {
            let mut ctrl_c = tokio_signal::CtrlC::new().await.unwrap();
            ctrl_c.next().await;
        })
        // TODO: proper shutdown
        // self.process_pool.shutdown_on_idle().wait().unwrap();
        // runtime.shutdown_now().wait(); // TODO: block_on
    }

    fn terminate(&self) {}

    /// Starts the main process
    pub fn start_main_process(&self, args: Vec<String>) {
        // println!("Starting main process...");
        let registry = self.modules.lock();

        // start the actual book process
        let module = registry.lookup(Atom::from("erl_init")).unwrap();
        let process = process::allocate(&self, 0 /* itself */, 0, module).unwrap();

        let context = process.context_mut();
        let fun = Atom::from("start");
        let arity = 2;
        context.x[0] = Term::atom(Atom::from("init"));
        context.x[1] = Cons::from_iter(
            args.into_iter()
                .map(|arg| Term::binary(&context.heap, bitstring::Binary::from(arg.into_bytes()))),
            &context.heap,
        );
        // println!("argv {}", context.x[1]);
        context.ip.ptr = module.funs[&(fun, arity)];

        let future = run_with_error_handling(process);
        self.process_pool.executor().spawn(future);

        // ------

        // start system processes
        // TODO: start as a special process and panic if it halts
        let module = registry.lookup(Atom::from("erts_code_purger")).unwrap();
        let process = process::allocate(&self, 0 /* itself */, 0, module).unwrap();
        let context = process.context_mut();
        let fun = Atom::from("start");
        let arity = 0;
        context.ip.ptr = module.funs[&(fun, arity)];
        let future = run_with_error_handling(process);
        self.process_pool.executor().spawn(future);


        // self.process_pool.schedule(process);
    }
}

/// Executes a single process, terminating in the event of an error.
pub async fn run_with_error_handling(mut process: RcProcess) {
    // We are using AssertUnwindSafe here so we can pass a &mut Worker to
    // run()/panic(). This might be risky if values captured are not unwind
    // safe, so take care when capturing new variables.
    //let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
    let vm = Machine::current();
    loop {
        match instruction::run(&*vm, &mut process).await {
            Err(message) => {
                if message.reason != Reason::TRAP {
                    // just a regular error
                    // HAXX: TODO clone() for now since handle_error consumes the msg
                    if let Some(new_pc) = exception::handle_error(&process, message.clone()) {
                        let context = process.context_mut();
                        context.ip = new_pc;
                        // yield
                    } else {
                        process.exit(&vm, message);
                        // println!("pid={} action=exited", process.pid);
                        break // crashed
                    }
                } else {
                    // we're trapping, ip was already set, now reschedule the process
                    eprintln!("TRAP!");
                    // yield
                }
            }
            Ok(process::State::Yield) => (), // yield
            Ok(process::State::Done) => {
                process.exit(&vm, Exception::with_value(Reason::EXC_EXIT, atom!(NORMAL)));

                // Terminate once the main process has finished execution.
                if process.is_main() {
                    vm.terminate();
                }

                break
            }, // exited OK
            // TODO: wait is an await on a oneshot
            // TODO: waittimeout is an select on a oneshot or a delay
        }
    }
    // }));

    /*
    if let Err(error) = result {
        if let Ok(message) = error.downcast::<String>() {
            //self.panic(worker, process, &message);
            panic!(message);
        } else {
            panic!("The VM panicked with an unknown error");
            /*
            self.panic(
                worker,
                process,
                &"The VM panicked with an unknown error",
            );*/
    };
    }*/
}
