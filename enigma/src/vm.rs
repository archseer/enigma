use crate::atom::{self, Atom};
use crate::bitstring;
use crate::ets::{RcTableRegistry, TableRegistry};
use crate::exception::{self, Exception, Reason};
use crate::exports_table::{Export, ExportsTable, RcExportsTable};
use crate::module; 
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::instruction;
use crate::process::registry::Registry as ProcessRegistry;
use crate::process::table::Table as ProcessTable;
use crate::port::{Table as PortTable, RcTable as RcPortTable};
use crate::process::{self, RcProcess};
use crate::persistent_term::{Table as PersistentTermTable};
use crate::servo_arc::Arc;
use crate::value::{self, Cons, Term};
use std::cell::RefCell;
// use log::debug;
use parking_lot::Mutex;
use std::panic;
use std::sync::atomic::AtomicUsize;
use std::time;

// use tokio::prelude::*;
use futures::{
  compat::*,
  future::{FutureExt, TryFutureExt},
  // io::AsyncWriteExt,
  // stream::StreamExt,
  // sink::SinkExt,
};
// use futures::prelude::*;


/// A reference counted State.
pub type RcMachine = Arc<Machine>;

pub struct Machine {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,
    pub process_registry: Mutex<ProcessRegistry<RcProcess>>,

    pub port_table: RcPortTable,
    /// TODO: Use priorities later on

    /// The start time of the VM (more or less).
    pub start_time: time::Instant,

    pub next_ref: AtomicUsize,

    /// PID pointing to the process handling system-wide logging.
    pub system_logger: AtomicUsize,

    pub process_pool: tokio::runtime::Runtime,
    pub runtime: tokio::runtime::Runtime,

    pub exit: Option<futures::channel::oneshot::Sender<()>>,

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    /// export table
    pub exports: RcExportsTable,

    /// Module registry
    pub modules: RcModuleRegistry,

    pub ets_tables: RcTableRegistry,

    pub persistent_terms: PersistentTermTable,
}

impl Machine {
    pub fn next_ref(&self) -> process::Ref {
        self.next_ref
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
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

#[macro_export]
macro_rules! expand_float {
    ($context:expr, $value:expr) => {{
        match &$value {
            FRegister::ExtendedLiteral(i) => unsafe {
                if let Variant::Float(value::Float(f)) =
                    (*$context.ip.module).literals[*i as usize].into_variant()
                {
                    f
                } else {
                    unreachable!()
                }
            },
            FRegister::X(reg) => {
                if let Variant::Float(value::Float(f)) = $context.x[reg.0 as usize].into_variant() {
                    f
                } else {
                    unreachable!()
                }
            }
            FRegister::Y(reg) => {
                let len = $context.stack.len();
                if let Variant::Float(value::Float(f)) =
                    $context.stack[len - (reg.0 + 1) as usize].into_variant()
                {
                    f
                } else {
                    unreachable!()
                }
            }
            FRegister::FloatReg(reg) => $context.f[*reg as usize],
        }
    }};
}

#[macro_export]
macro_rules! op_jump {
    ($context:expr, $label:expr) => ($context.ip.ptr = $label);
}

#[macro_export]
macro_rules! op_jump_ptr {
    ($context:expr, $ptr:expr) => ($context.ip = $ptr);
}

#[inline]
pub fn op_deallocate(context: &mut process::ExecutionContext, nwords: u16) {
    let (_, cp) = context.callstack.pop().unwrap();
    context
        .stack
        .truncate(context.stack.len() - nwords as usize);

    context.cp = cp;
}

#[macro_export]
macro_rules! call_error_handler {
    ($vm:expr, $process:expr, $mfa:expr) => 
        (crate::vm::call_error_handler($vm, $process, $mfa, atom::UNDEFINED_FUNCTION)?);
}

// func is atom
#[inline]
pub fn call_error_handler(
    vm: &Machine,
    process: &RcProcess,
    mfa: &module::MFA,
    func: atom::Atom,
) -> Result<(), Exception> {
    // debug!("call_error_handler mfa={}, mfa)
    let context = process.context_mut();

    // Search for the error_handler module.
    let ptr =
        match vm
            .exports
            .read()
            .lookup(&module::MFA(process.local_data().error_handler, func, 3))
        {
            Some(Export::Fun(ptr)) => ptr,
            Some(_) => unimplemented!("call_error_handler for non-fun"),
            None => {
                println!("function not found {}", mfa);
                // no error handler
                // TODO: set current to mfa
                return Err(Exception::new(Reason::EXC_UNDEF));
            }
        };

    // Create a list with all arguments in the x registers.
    // TODO: I don't like from_iter requiring copied
    let args = Cons::from_iter(context.x[0..mfa.2 as usize].iter().copied(), &context.heap);

    // Set up registers for call to error_handler:<func>/3.
    context.x[0] = Term::atom(mfa.0); // module
    context.x[1] = Term::atom(mfa.1); // func
    context.x[2] = args;
    op_jump_ptr!(context, ptr);
    Ok(())
}

#[macro_export]
macro_rules! op_call_ext {
    ($vm:expr, $context:expr, $process:expr, $arity:expr, $dest: expr) => {{
        // TODO: precompute these
        let mfa = unsafe { &(*$context.ip.module).imports[$dest as usize] };

        // if $process.pid >= 95 {
        // info!("pid={} action=call_ext mfa={}", $process.pid, mfa);
        // }

        let export = { $vm.exports.read().lookup(mfa) }; // drop the exports lock

        match export {
            Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
            Some(Export::Bif(APPLY_2)) => unreachable!("apply/2 called via call_ext"),
            Some(Export::Bif(APPLY_3)) => unreachable!("apply/3 called via call_ext"),
            Some(Export::Bif(_)) => unreachable!("bif called without call_bif: {}", mfa),
            // Some(Export::Bif(bif)) => {
            //     // precomputed in most cases, but not for nifs
            //     // TODO: return needs to still be handled here then :/
            //     op_call_bif!($vm, $context, $process, bif, $arity as usize)
            // }
            None => {
                // call error_handler here
                call_error_handler!($vm, $process, mfa);
            }
        }
    }};
}

#[macro_export]
macro_rules! op_call_bif {
    ($vm:expr, $context:expr, $process:expr, $bif:expr, $arity:expr) => {{
        // make a slice out of arity x registers
        let args = &$context.x[0..$arity];
        match $bif($vm, $process, args) {
            Ok(val) => {
                $context.x[0] = val; // HAXX, return value emulated
            }
            Err(exc) => return Err(exc),
        }
    }};
}
#[macro_export]
macro_rules! op_call_fun {
    ($vm:expr, $context:expr, $process:expr, $value:expr, $arity:expr) => {{
        if let Ok(closure) = value::Closure::cast_from(&$value) {
            // keep X regs set based on arity
            // set additional X regs based on lambda.binding
            // set x from 1 + arity (x0 is func, followed by call params) onwards to binding
            if let Some(binding) = &closure.binding {
                let arity = $arity as usize;
                $context.x[arity..arity + binding.len()].copy_from_slice(&binding[..]);
            }

            // TODO: closure needs to jump_ptr to the correct module.
            let ptr = {
                // temporary HAXX
                let registry = $vm.modules.lock();
                // let module = module::load_module(&self.modules, path).unwrap();
                let module = registry.lookup(closure.mfa.0).unwrap();

                InstrPtr {
                    module,
                    ptr: closure.ptr,
                }
            };

            op_jump_ptr!($context, ptr);
        } else if let Ok(mfa) = module::MFA::cast_from(&$value) {
            // TODO: deduplicate this part
            let export = { $vm.exports.read().lookup(&mfa) }; // drop the exports lock

            match export {
                Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
                Some(Export::Bif(bif)) => {
                    op_call_bif!($vm, $context, &$process, bif, mfa.2 as usize)
                }
                None => {
                    // println!("apply setup_error_handler");
                    call_error_handler!($vm, &$process, &mfa);
                    // apply_setup_error_handler
                }
            }
        } else {
            unreachable!()
        }
    }};
}

#[macro_export]
macro_rules! op_apply {
    ($vm:expr, $context:expr, $process:expr) => {{
        let mut module = $context.x[0];
        let mut func = $context.x[1];
        let mut args = $context.x[2];

        loop {
            if !func.is_atom() || !module.is_atom() {
                $context.x[0] = module;
                $context.x[1] = func;
                $context.x[2] = Term::nil();
                println!(
                    "pid={} tried to apply not a func {}:{}",
                    $process.pid, module, func
                );

                return Err(Exception::new(Reason::EXC_BADARG));
            }

            if module.to_atom().unwrap() != atom::ERLANG || func.to_atom().unwrap() != atom::APPLY {
                break;
            }

            // Handle apply of apply/3...

            // continually loop over args to resolve
            let a = args;

            if let Ok(cons) = Cons::cast_from(&a) {
                let m = cons.head;
                let a = cons.tail;

                if let Ok(cons) = Cons::cast_from(&a) {
                    let f = cons.head;
                    let a = cons.tail;

                    if let Ok(cons) = Cons::cast_from(&a) {
                        let a = cons.head;
                        if cons.tail.is_nil() {
                            module = m;
                            func = f;
                            args = a;
                            continue;
                        }
                    }
                }
            }
            break; // != erlang:apply/3
        }

        let mut arity = 0;

        // Walk down the 3rd parameter of apply (the argument list) and copy
        // the parameters to the x registers (reg[]).

        while let Ok(value::Cons { head, tail }) = args.cast_into() {
            if arity < process::MAX_REG - 1 {
                $context.x[arity] = *head;
                arity += 1;
                args = *tail
            } else {
                return Err(Exception::new(Reason::EXC_SYSTEM_LIMIT));
            }
        }

        if !args.is_nil() {
            // Must be well-formed list
            return Err(Exception::new(Reason::EXC_BADARG));
        }

        /*
         * Get the index into the export table, or failing that the export
         * entry for the error handler module.
         *
         * Note: All BIFs have export entries; thus, no special case is needed.
         */

        let mfa = module::MFA(module.to_atom().unwrap(), func.to_atom().unwrap(), arity as u32);

        // println!("pid={} applying/3... {}", $process.pid, mfa);

        let export = { $vm.exports.read().lookup(&mfa) }; // drop the exports lock

        match export {
            Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
            Some(Export::Bif(APPLY_2)) => {
                // TODO: this clobbers the registers

                // TODO: rewrite these two into Apply instruction calls
                // I'm cheating here, *shrug*
                op_apply_fun!($vm, $context, $process)
            }
            // Some(Export::Bif(_)) => unreachable!("op_apply: bif called without call_bif: {}", mfa),
            Some(Export::Bif(bif)) => {
                // TODO: this would still need to return on a bif if it's i_apply_only/_last

                // TODO: apply_bif_error_adjustment(p, ep, reg, arity, I, stack_offset);
                // ^ only happens in apply/fixed_apply
                op_call_bif!($vm, $context, $process, bif, arity)
            }
            None => {
                // println!("apply setup_error_handler pid={}", $process.pid);
                call_error_handler!($vm, $process, &mfa);
                // apply_setup_error_handler
            }
        }
        //    if ((ep = erts_active_export_entry(module, function, arity)) == NULL) {
        //      if ((ep = apply_setup_error_handler(p, module, function, arity, reg)) == NULL)
        //        goto error;
        //    }
        //    TODO: runs on apply and fixed apply
        //    apply_bif_error_adjustment(p, ep, reg, arity, I, stack_offset);

        // op_jump_ptr!($context, ptr)
    }};
}

#[macro_export]
macro_rules! op_apply_fun {
    ($vm:expr, $context:expr, $process:expr) => {{
        // Walk down the 3rd parameter of apply (the argument list) and copy
        // the parameters to the x registers (reg[]).

        let fun = $context.x[0];
        let mut args = $context.x[1];
        let mut arity = 0;

        while let Ok(value::Cons { head, tail }) = args.cast_into() {
            if arity < process::MAX_REG - 1 {
                $context.x[arity] = *head;
                arity += 1;
                args = *tail
            } else {
                return Err(Exception::new(Reason::EXC_SYSTEM_LIMIT));
            }
        }

        if !args.is_nil() {
            /* Must be well-formed list */
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        //context.x[arity] = fun;

        op_call_fun!($vm, $context, $process, fun, arity);
    }};
}

#[macro_export]
macro_rules! op_return {
    ($process:expr, $context:expr) => {{
        if let Some(i) = $context.cp.take() {
            op_jump_ptr!($context, i);
        } else {
            // println!("Process pid={} exited with normal, x0: {}", $process.pid, $context.x[0]);
            return Ok(process::State::Done)
        }
    }};
}

#[macro_export]
macro_rules! fail {
    ($context:expr, $label:expr) => {{
        op_jump!($context, $label);
        continue;
    }};
}

#[macro_export]
macro_rules! cond_fail {
    ($context:expr, $fail:expr, $exc: expr) => {{
        if $fail != 0 {
            op_jump!($context, $fail);
            continue;
        } else {
            return Err($exc);
        }
    }};
}

#[macro_export]
macro_rules! op_fixed_apply {
    ($vm:expr, $context:expr, $process:expr, $arity:expr) => {{
        let arity = $arity as usize;
        let module = $context.x[arity];
        let func = $context.x[arity + 1];

        if !func.is_atom() || !module.is_atom() {
            $context.x[0] = module;
            $context.x[1] = func;
            $context.x[2] = Term::nil();

            return Err(Exception::new(Reason::EXC_BADARG));
        }

        let module = module.to_atom().unwrap();
        let func = func.to_atom().unwrap();

        // Handle apply of apply/3...
        if module == atom::ERLANG && func == atom::APPLY && $arity == 3 {
            op_apply!($vm, $context, $process);
            continue;
        }

        /*
         * Get the index into the export table, or failing that the export
         * entry for the error handler module.
         *
         * Note: All BIFs have export entries; thus, no special case is needed.
         */

        // TODO: make arity a U8 on the type
        let mfa = module::MFA(module, func, $arity as u32);

        let export = { $vm.exports.read().lookup(&mfa) }; // drop the exports lock

        match export {
            Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
            Some(Export::Bif(bif)) => {
                // TODO: apply_bif_error_adjustment(p, ep, reg, arity, I, stack_offset);
                // ^ only happens in apply/fixed_apply

                op_call_bif!($vm, $context, $process, bif, arity)
            }
            None => {
                // println!("fixed_apply setup_error_handler pid={}", $process.pid);
                call_error_handler!($vm, $process, &mfa);
                // apply_setup_error_handler
            }
        }
        //    if ((ep = erts_active_export_entry(module, function, arity)) == NULL) {
        //      if ((ep = apply_setup_error_handler(p, module, function, arity, reg)) == NULL)
        //        goto error;
        //    }
        //    TODO: runs on apply and fixed apply
        //    apply_bif_error_adjustment(p, ep, reg, arity, I, stack_offset);

        // op_jump_ptr!($context, ptr)
    }};
}

#[macro_export]
macro_rules! op_is_type {
    ($context:expr, $fail:expr, $arg:expr, $op:ident) => {
        if !$context.expand_arg($arg).$op() {
            op_jump!($context, $fail);
        }
    };
}

#[macro_export]
macro_rules! to_expr {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! op_float {
    ($context:expr, $a:expr, $b:expr, $dest:expr, $op:tt) => {{
        $context.f[$dest as usize] = to_expr!($context.f[$a as usize] $op $context.f[$b as usize]);
    }};
}

#[macro_export]
macro_rules! safepoint_and_reduce {
    ($vm:expr, $process:expr, $reductions:expr) => {{
        // if $vm.gc_safepoint(&$process) {
        //     return Ok(());
        // }

        // Reduce once we've exhausted all the instructions in a context.
        if $reductions > 0 {
            $reductions -= 1;
        } else {
            // $vm
            //     .process_pool
            //     .schedule(Job::normal($process.clone()));
            return Ok(process::State::Yield);
        }
    }};
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

    pub fn elapsed_time(&self) -> time::Duration {
        self.start_time.elapsed()
    }

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

        // Create an infinite stream of "Ctrl+C" notifications. Each item received
        // on this stream may represent multiple ctrl-c signals.
        use futures01::future::Future;
        use futures01::stream::Stream;
        let ctrl_c = tokio_signal::ctrl_c().flatten_stream();

        // Wait until the runtime becomes idle and shut it down.
        vm.runtime.block_on(ctrl_c.into_future()).ok().unwrap();
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
        context.x[1] = value::Cons::from_iter(
            args.into_iter()
                .map(|arg| Term::binary(&context.heap, bitstring::Binary::from(arg.into_bytes()))),
            &context.heap,
        );
        // println!("argv {}", context.x[1]);
        op_jump!(context, module.funs[&(fun, arity)]);

        let future = run_with_error_handling(process);
        self.process_pool.executor().spawn(future.unit_error().boxed().compat());

        // ------

        // start system processes
        // TODO: start as a special process and panic if it halts
        let module = registry.lookup(Atom::from("erts_code_purger")).unwrap();
        let process = process::allocate(&self, 0 /* itself */, 0, module).unwrap();
        let context = process.context_mut();
        let fun = Atom::from("start");
        let arity = 0;
        op_jump!(context, module.funs[&(fun, arity)]);
        let future = run_with_error_handling(process);
        self.process_pool.executor().spawn(future.unit_error().boxed().compat());


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
