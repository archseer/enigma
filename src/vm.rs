use crate::atom;
use crate::port;
use crate::bif;
use crate::bitstring;
use crate::ets::{RcTableRegistry, TableRegistry};
use crate::exception::{self, Exception, Reason};
use crate::exports_table::{Export, ExportsTable, RcExportsTable};
use crate::instr_ptr::InstrPtr;
use crate::module;
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::instruction::{self, Instruction};
use crate::process::registry::Registry as ProcessRegistry;
use crate::process::table::Table as ProcessTable;
use crate::port::{Table as PortTable, RcTable as RcPortTable};
use crate::process::{self, RcProcess};
use crate::persistent_term::{Table as PersistentTermTable};
use crate::servo_arc::Arc;
use crate::value::{self, Cons, Term, CastFrom, CastInto, CastIntoMut, Tuple, Variant};
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
  stream::StreamExt,
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
                if let Variant::Float(value::Float(f)) = $context.x[*reg as usize].into_variant() {
                    f
                } else {
                    unreachable!()
                }
            }
            FRegister::Y(reg) => {
                let len = $context.stack.len();
                if let Variant::Float(value::Float(f)) =
                    $context.stack[len - (reg + 1) as usize].into_variant()
                {
                    f
                } else {
                    unreachable!()
                }
            }
            FRegister::FloatReg(reg) => $context.f[*reg as usize],
            _ => unreachable!(),
        }
    }};
}

macro_rules! op_jump {
    ($context:expr, $label:expr) => {{
        $context.ip.ptr = $label;
    }};
}

macro_rules! op_jump_ptr {
    ($context:expr, $ptr:expr) => {{
        $context.ip = $ptr;
    }};
}

macro_rules! op_deallocate {
    ($context:expr, $nwords:expr) => {{
        let (_, cp) = $context.callstack.pop().unwrap();
        $context
            .stack
            .truncate($context.stack.len() - $nwords as usize);

        $context.cp = cp;
    }};
}

macro_rules! call_error_handler {
    ($vm:expr, $process:expr, $mfa:expr) => {{
        call_error_handler($vm, $process, $mfa, atom::UNDEFINED_FUNCTION)?;
    }};
}

// func is atom
#[inline]
fn call_error_handler(
    vm: &Machine,
    process: &RcProcess,
    mfa: &module::MFA,
    func: u32,
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

const APPLY_2: bif::Fn = bif::bif_erlang_apply_2;
const APPLY_3: bif::Fn = bif::bif_erlang_apply_3;

macro_rules! op_call_ext {
    ($vm:expr, $context:expr, $process:expr, $arity:expr, $dest: expr, $return: expr) => {{
        let mfa = unsafe { &(*$context.ip.module).imports[$dest as usize] };

        // if $process.pid > 70 {
        // info!("pid={} action=call_ext mfa={}", $process.pid, mfa);
        // }

        let export = { $vm.exports.read().lookup(mfa) }; // drop the exports lock

        match export {
            Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
            Some(Export::Bif(APPLY_2)) => {
                // TODO: rewrite these two into Apply instruction calls
                // I'm cheating here, *shrug*
                op_apply_fun!($vm, $context, $process)
            }
            Some(Export::Bif(APPLY_3)) => {
                // I'm cheating here, *shrug*
                op_apply!($vm, $context, $process, $return);
            }
            Some(Export::Bif(bif)) => {
                // TODO: precompute which exports are bifs to avoid exports table reads
                // call_ext_only Ar=u Bif=u$is_bif => \
                // allocate u Ar | call_bif Bif | deallocate_return u

                op_call_bif!($vm, $context, $process, bif, $arity as usize, $return)
            }
            None => {
                // call error_handler here
                call_error_handler!($vm, $process, mfa);
            }
        }
    }};
}

macro_rules! op_call_bif {
    ($vm:expr, $context:expr, $process:expr, $bif:expr, $arity:expr, $return:expr) => {{
        // precompute export lookup. once Pin<> is a thing we can be sure that
        // a ptr into the hashmap will always point to a module.
        // call_ext_only Ar=u Bif=u$is_bif => \
        // allocate u Ar | call_bif Bif | deallocate_return u

        // make a slice out of arity x registers
        let args = &$context.x[0..$arity];
        match $bif($vm, $process, args) {
            Ok(val) => {
                $context.x[0] = val; // HAXX, return value emulated
                if $return {
                    // TODO: figure out returns
                    op_return!($process, $context);
                }
            }
            Err(exc) => return Err(exc),
        }
    }};
}
macro_rules! op_call_fun {
    ($vm:expr, $context:expr, $closure:expr, $arity:expr) => {{
        // keep X regs set based on arity
        // set additional X regs based on lambda.binding
        // set x from 1 + arity (x0 is func, followed by call params) onwards to binding
        let closure = $closure;
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
    }};
}

//TODO: need op_apply_fun, but it should use x0 as mod, x1 as fun, etc
macro_rules! op_apply {
    ($vm:expr, $context:expr, $process:expr, $return: expr) => {{
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
            Some(Export::Bif(bif)) => {
                // TODO: apply_bif_error_adjustment(p, ep, reg, arity, I, stack_offset);
                // ^ only happens in apply/fixed_apply
                op_call_bif!($vm, $context, $process, bif, arity, $return)
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

        // TODO: this part matches CallFun, extract
        if let Ok(closure) = value::Closure::cast_from(&fun) {
            op_call_fun!($vm, $context, closure, arity);
        } else if let Ok(mfa) = module::MFA::cast_from(&fun) {
            // TODO: deduplicate this part
            let export = { $vm.exports.read().lookup(&mfa) }; // drop the exports lock

            match export {
                Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
                Some(Export::Bif(bif)) => {
                    op_call_bif!($vm, $context, &$process, bif, mfa.2 as usize, true) // TODO is return true ok
                }
                None => {
                    // println!("apply setup_error_handler");
                    call_error_handler!($vm, &$process, &mfa);
                    // apply_setup_error_handler
                }
            }
        } else {
            // TODO raise error
            unreachable!()
        }
    }};
}

macro_rules! op_return {
    ($process:expr, $context:expr) => {{
        if let Some(i) = $context.cp {
            op_jump_ptr!($context, i);
            $context.cp = None;
        } else {
            // println!("Process pid={} exited with normal, x0: {}", $process.pid, $context.x[0]);
            return Ok(process::State::Done)
        }
    }};
}

macro_rules! fail {
    ($context:expr, $label:expr) => {{
        op_jump!($context, $label);
        continue;
    }};
}

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

macro_rules! op_fixed_apply {
    ($vm:expr, $context:expr, $process:expr, $arity:expr, $return:expr) => {{
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
            op_apply!($vm, $context, $process, $return);
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

                op_call_bif!($vm, $context, $process, bif, arity, $return)
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

macro_rules! op_is_type {
    ($context:expr, $fail:expr, $arg:expr, $op:ident) => {{
        let val = $context.expand_arg($arg);

        if !val.$op() {
            // TODO: patch the labels to point to exact offsets to avoid labels lookup
            op_jump!($context, $fail);
        }
    }};
}

macro_rules! to_expr {
    ($e:expr) => {
        $e
    };
}

macro_rules! op_float {
    ($context:expr, $a:expr, $b:expr, $dest:expr, $op:tt) => {{
        $context.f[$dest as usize] = to_expr!($context.f[$a as usize] $op $context.f[$b as usize]);
    }};
}

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
    include_bytes!("../otp/erts/preloaded/ebin/erts_code_purger.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erl_init.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/init.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/prim_buffer.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/prim_eval.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/prim_inet.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/prim_file.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/zlib.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/prim_zip.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erl_prim_loader.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erlang.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erts_internal.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erl_tracer.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erts_literal_area_collector.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/erts_dirty_process_signal_handler.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/atomics.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/counters.beam"),
    include_bytes!("../otp/erts/preloaded/ebin/persistent_term.beam"),
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
        //let module = unsafe { &*module::load_module(self, path).unwrap() };
        let module = registry.lookup(atom::from_str("erl_init")).unwrap();
        let process = process::allocate(&self, 0 /* itself */, 0, module).unwrap();

        /* TEMP */
        let context = process.context_mut();
        let fun = atom::from_str("start");
        let arity = 2;
        context.x[0] = Term::atom(atom::from_str("init"));
        context.x[1] = value::Cons::from_iter(
            args.into_iter()
                .map(|arg| Term::binary(&context.heap, bitstring::Binary::from(arg.into_bytes()))),
            &context.heap,
        );
        // println!("argv {}", context.x[1]);
        op_jump!(context, module.funs[&(fun, arity)]);
        /* TEMP */

        let future = run_with_error_handling(process);
        self.process_pool.executor().spawn(future.unit_error().boxed().compat());

        // self.process_pool.schedule(process);
    }
}

    /// Executes a single process, terminating in the event of an error.
    pub async fn run_with_error_handling(
        mut process: RcProcess
    ) {

        // We are using AssertUnwindSafe here so we can pass a &mut Worker to
        // run()/panic(). This might be risky if values captured are not unwind
        // safe, so take care when capturing new variables.
        //let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let vm = Machine::current();
        loop {
            match vm.run(&mut process).await {
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

    pub trait Captures<'a> {}

    impl<'a, T> Captures<'a> for T {}

impl Machine {
    #[allow(clippy::cognitive_complexity)]
    pub fn run<'a: 'c, 'b: 'c, 'c>(
        &'a self,
        process: &'b mut RcProcess,
    ) -> impl std::future::Future<Output = Result<process::State, Exception>> + Captures<'a> + Captures<'b> + 'c {
        async move {  // workaround for https://github.com/rust-lang/rust/issues/56238
        let context = process.context_mut();
        context.reds = 2000; // self.config.reductions;

        // process the incoming signal queue
        process.process_incoming()?;

        loop {
            let module = context.ip.get_module();
            // let module = unsafe { &(*context.ip.module) };
            let ins = &module.instructions[context.ip.ptr as usize];
            context.ip.ptr += 1;

            // if process.pid > 70 {
            //     info!(
            //         "proc pid={:?} reds={:?} mod={:?} offs={:?} ins={:?}",
            //         process.pid,
            //         context.reds,
            //         atom::to_str(module.name).unwrap(),
            //         context.ip.ptr,
            //         ins,
            //     );
            // }

            match ins {
                &Instruction::FuncInfo { .. } => {
                    // Raises function clause exception.
                    // eprintln!("function clause! {} {} {:?}",
                    //     context.expand_arg(&ins.args[0]),
                    //     context.expand_arg(&ins.args[1]),
                    //     ins.args[2]
                    // );
                    return Err(Exception::new(Reason::EXC_FUNCTION_CLAUSE));
                }
                &Instruction::Jump(label) => {
                    op_jump!(context, label)
                }
                &Instruction::Move { source, destination } => {
                    let val = context.expand_arg(source);
                    context.set_register(destination, val)
                }
                &Instruction::Swap { a, b } => {
                    use instruction::Register;
                    let a: *mut Term = match a {
                        Register::X(i) => &mut context.x[i as usize],
                        Register::Y(i) => {
                            let len = context.stack.len();
                            &mut context.stack[len - (i + 1) as usize]
                        }
                    };
                    let b: *mut Term = match b {
                        Register::X(i) => &mut context.x[i as usize],
                        Register::Y(i) => {
                            let len = context.stack.len();
                            &mut context.stack[len - (i + 1) as usize]
                        }
                    };
                    unsafe { std::ptr::swap(a, b); }
                }
                &Instruction::Return => {
                    op_return!(process, context);
                }
                &Instruction::Send => {
                    // send x1 to x0, write result to x0
                    let pid = context.x[0];
                    let msg = context.x[1];
                    let res = match pid.into_variant() {
                        Variant::Port(id) => port::send_message(self, process.pid, id, msg),
                        _ => process::send_message(self, process.pid, pid, msg),
                    }?;
                    context.x[0] = res;
                }
                &Instruction::RemoveMessage => {
                    // Unlink the current message from the message queue. Remove any timeout.
                    process.local_data_mut().mailbox.remove();
                    // clear timeout
                    context.timeout.take();
                    // reset savepoint of the mailbox
                    process.local_data_mut().mailbox.reset();
                }
                &Instruction::Timeout => {
                    //  Reset the save point of the mailbox and clear the timeout flag.
                    process.local_data_mut().mailbox.reset();
                    // clear timeout
                    context.timeout.take();
                }
                &Instruction::LoopRec { label: fail, source } => {
                    // TODO: source is supposed to be the location, but it's always x0
                    // grab message from queue, put to x0, if no message, jump to fail label
                    if let Some(msg) = process.receive()? {
                        // println!("recv proc pid={:?} msg={}", process.pid, msg);
                        context.x[0] = msg
                    } else {
                        op_jump!(context, fail);
                    }
                }
                &Instruction::LoopRecEnd { label } => {
                    // Advance the save pointer to the next message and jump back to Label.

                    process.local_data_mut().mailbox.advance();
                    op_jump!(context, label);
                }
                &Instruction::Wait { label } => {
                    // jump to label, set wait flag on process

                    op_jump!(context, label);

                    // TODO: this currently races if the process is sending us
                    // a message while we're in the process of suspending.

                    // set wait flag
                    // process.set_waiting_for_message(true);

                    // LOCK mailbox on looprec, unlock on wait/waittimeout
                    let cancel = process.context_mut().recv_channel.take().unwrap();
                    cancel.await; // suspend process

                    // println!("pid={} resumption ", process.pid);
                    process.process_incoming()?;
                }
                &Instruction::WaitTimeout { label, time } => {
                    // Sets up a timeout of Time milliseconds and saves the address of the
                    // following instruction as the entry point if the timeout triggers.

                    if process.context_mut().timeout.is_none() {
                        // if this is outside of loop_rec, this will be blank
                        let (trigger, cancel) = futures::channel::oneshot::channel::<()>();
                        process.context_mut().recv_channel = Some(cancel);
                        process.context_mut().timeout = Some(trigger);
                    }

                    let cancel = process.context_mut().recv_channel.take().unwrap();

                    match context.expand_arg(time).into_variant() {
                        Variant::Atom(atom::INFINITY) => {
                            // just a normal &Instruction::Wait
                            // println!("infinity wait");
                            op_jump!(context, label);

                            cancel.await; // suspend process
                            // println!("select! resumption pid={}", process.pid);
                        },
                        Variant::Integer(ms) => {
                            let when = time::Duration::from_millis(ms as u64);
                            use tokio::prelude::FutureExt;

                            match cancel.into_future().boxed().compat().timeout(when).compat().await {
                                Ok(()) =>  {
                                    // jump to success (start of recv loop)
                                    op_jump!(context, label);
                                    // println!("select! resumption pid={}", process.pid);
                                }
                                Err(_err) => {
                                    // timeout
                                    // println!("select! delay timeout {} pid={} ms={} m={:?}", ms, process.pid, context.expand_arg(&ins.args[1]), ins.args[1]);
                                    // println!("select! delay timeout {} pid={} ms={} err={:?}", ms, process.pid, context.expand_arg(&ins.args[1]), err);

                                    // remove channel
                                    context.timeout.take();

                                    // continue to next instruction (timeout op)
                                }

                            }
                            // select! { // suspend process
                            //     _t = future => {
                            //         // jump to success (start of recv loop)
                            //         let label = ins.args[0].to_u32();
                            //         op_jump!(context, label);
                            //         println!("select! resumption");
                            //     },

                            //     // did not work _ = tokio::timer::Delay::new(when) => {
                            //     _ = Delay::new(when) => {
                            //         // timeout
                            //         println!("select! delay timeout");

                            //         // remove channel
                            //         // context.timeout.take();

                            //         () // continue to next instruction (timeout op)
                            //     }
                            // };
                        },
                        // TODO: bigint
                        _ => unreachable!("{}", context.expand_arg(time))
                    }
                    process.process_incoming()?;
                }
                &Instruction::RecvMark => {
                    process.local_data_mut().mailbox.mark();
                }
                &Instruction::RecvSet => {
                    process.local_data_mut().mailbox.set();
                }
                &Instruction::Call { arity, label: i } => {
                    // store arity as live
                    context.cp = Some(context.ip);
                    op_jump!(context, i);

                    // if process.pid > 70 {
                    //     let (mfa, _) = context.ip.lookup_func_info().unwrap();
                    //     info!("pid={} action=call mfa={}", process.pid, mfa);
                    // }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::CallLast { arity, label: i, words }=> {
                    // store arity as live
                    op_deallocate!(context, words);

                    op_jump!(context, i);

                    // if process.pid > 70 {
                    // let (mfa, _) = context.ip.lookup_func_info().unwrap();
                    // info!("pid={} action=call_last mfa={}", process.pid, mfa);
                    // }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::CallOnly { arity, label: i }=> {
                    // store arity as live
                    op_jump!(context, i);

                    // if process.pid > 70 {
                    // let (mfa, _) = context.ip.lookup_func_info().unwrap();
                    // info!("pid={} action=call_only mfa={}", process.pid, mfa);
                    // }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::CallExt { arity, destination } => {
                    // save pointer onto CP
                    context.cp = Some(context.ip);

                    op_call_ext!(self, context, &process, arity, destination, false); // don't return on call_ext
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::CallExtOnly { arity, destination }=> {
                    op_call_ext!(self, context, &process, arity, destination, true);
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::CallExtLast { arity, destination, words } => {
                    op_deallocate!(context, words);

                    op_call_ext!(self, context, &process, arity, destination, true);
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::Bif0 { bif, reg } => {
                    let val = bif(self, &process, &[]).unwrap(); // bif0 can't fail
                    context.set_register(reg, val);
                }
                &Instruction::Bif1 { fail, bif, arg1, reg } => {
                    let args = &[context.expand_arg(arg1)];
                    match bif(self, &process, args) {
                        Ok(val) => context.set_register(reg, val),
                        Err(exc) => cond_fail!(context, fail, exc),
                    }
                }
                &Instruction::Bif2 { fail, bif, arg1, arg2, reg } => {
                    let args = &[context.expand_arg(arg1), context.expand_arg(arg2)];
                    match bif(self, &process, args) {
                        Ok(val) => context.set_register(reg, val),
                        Err(exc) => cond_fail!(context, fail, exc),
                    }
                }
                &Instruction::Allocate { stackneed, live } => {
                    context
                        .stack
                        .resize(context.stack.len() + stackneed as usize, Term::nil());
                    context.callstack.push((stackneed, context.cp.take()));
                }
                &Instruction::AllocateHeap { stackneed, heapneed, live } => {
                    // TODO: this also zeroes the values, make it dynamically change the
                    // capacity/len of the Vec to bypass initing these.

                    // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
                    // num of X regs. save cp on stack.
                    context
                        .stack
                        .resize(context.stack.len() + stackneed as usize, Term::nil());
                    // TODO: check heap for heapneed space!
                    context.callstack.push((stackneed, context.cp.take()));
                }
                &Instruction::AllocateZero { stackneed, live } => {
                    context
                        .stack
                        .resize(context.stack.len() + stackneed as usize, Term::nil());
                    context.callstack.push((stackneed, context.cp.take()));
                }
                &Instruction::AllocateHeapZero { stackneed, heapneed, live } => {
                    // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
                    // num of X regs. save cp on stack.
                    context
                        .stack
                        .resize(context.stack.len() + stackneed as usize, Term::nil());
                    // TODO: check heap for heapneed space!
                    context.callstack.push((stackneed, context.cp.take()));
                }
                &Instruction::TestHeap { .. } => {
                    // println!("TODO: TestHeap unimplemented!");
                }
                &Instruction::Init { n } => {
                    context.set_register(n, Term::nil())
                }
                &Instruction::Deallocate { n } => {
                    op_deallocate!(context, n)
                }
                &Instruction::IsGe { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        op_jump!(context, fail);
                    } else {
                        // ok
                    }
                }
                &Instruction::IsLt { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        op_jump!(context, fail);
                    }
                }
                &Instruction::IsEq { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if let Some(std::cmp::Ordering::Equal) = v1.erl_partial_cmp(&v2) {
                        // ok
                    } else {
                        op_jump!(context, fail);
                    }
                }
                &Instruction::IsNe { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if let Some(std::cmp::Ordering::Equal) = v1.erl_partial_cmp(&v2) {
                        op_jump!(context, fail);
                    }
                }
                &Instruction::IsEqExact { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if v1.eq(&v2) {
                        // ok
                    } else {
                        op_jump!(context, fail);
                    }
                }
                &Instruction::IsNeExact { label: fail, arg1, arg2 } => {
                    let v1 = context.expand_arg(arg1);
                    let v2 = context.expand_arg(arg2);

                    if v1.eq(&v2) {
                        op_jump!(context, fail);
                    } else {
                        // ok
                    }
                }
                &Instruction::IsInteger { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_integer),
                &Instruction::IsFloat { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_float),
                &Instruction::IsNumber { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_number),
                &Instruction::IsAtom { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_atom),
                &Instruction::IsPid { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_pid),
                &Instruction::IsReference { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_ref),
                &Instruction::IsPort { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_port),
                &Instruction::IsNil { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_nil),
                &Instruction::IsBinary { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_binary),
                &Instruction::IsBitstr { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_bitstring),
                &Instruction::IsList { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_list),
                &Instruction::IsNonemptyList { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_non_empty_list),
                &Instruction::IsTuple { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_tuple),
                &Instruction::IsFunction { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_function),
                &Instruction::IsBoolean { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_boolean),
                &Instruction::IsMap { label: fail, arg1 } => op_is_type!(context, fail, arg1, is_map),
                &Instruction::IsFunction2 { label: fail, arg1, arity } => {
                    // TODO: needs to verify exports too
                    let value = context.expand_arg(arg1);
                    let arity = context.expand_arg(arity).to_uint().unwrap();
                    if let Ok(closure) = value::Closure::cast_from(&value) {
                        if closure.mfa.2 == arity {
                            continue;
                        }
                    }
                    if let Ok(mfa) = module::MFA::cast_from(&value) {
                        if mfa.2 == arity {
                            continue;
                        }
                    }
                    op_jump!(context, fail);
                }
                &Instruction::TestArity { label: fail, arg1, arity } => {
                    // check tuple arity
                    if let Ok(t) = Tuple::cast_from(&context.expand_arg(arg1)) {
                        if t.len != arity {
                            op_jump!(context, fail);
                        }
                    } else {
                        panic!("Bad argument to TestArity")
                    }
                }
                Instruction::SelectVal { arg, fail, destinations: vec } => {
                    // loop over dests
                    let arg = context.expand_arg(*arg).into_variant();
                    let mut i = 0;
                    loop {
                        // if key matches, jump to the following label
                        if context.expand_arg(vec[i].into_value()).into_variant() == arg {
                            let label = vec[i + 1].to_label();
                            op_jump!(context, label);
                            break;
                        }

                        i += 2;

                        // if we ran out of options, jump to fail
                        if i >= vec.len() {
                            op_jump!(context, *fail);
                            break;
                        }
                    }
                }
                Instruction::SelectTupleArity { arg, fail, arities: vec } => {
                    if let Ok(tup) = Tuple::cast_from(&context.expand_arg(*arg)) {
                        let len = tup.len;
                        let mut i = 0;
                        loop {
                            // if key matches, jump to the following label
                            if let instruction::Entry::Literal(n) = vec[i] {
                                if n == len {
                                    let label = vec[i + 1].to_label();
                                    op_jump!(context, label);
                                    break;
                                }
                            }

                            i += 2;

                            // if we ran out of options, jump to fail
                            if i >= vec.len() {
                                op_jump!(context, *fail);
                                break;
                            }
                        }
                    } else {
                        op_jump!(context, *fail);
                    }
                }
                &Instruction::GetList { source, head: h, tail: t } => {
                    if let Ok(value::Cons { head, tail }) = context.fetch_register(source).cast_into() {
                        context.set_register(h, *head);
                        context.set_register(t, *tail);
                    } else {
                        panic!("badarg to GetList")
                    }
                }
                &Instruction::GetTupleElement { source, element, destination } => {
                    let source = context.fetch_register(source);
                    let n = element as usize;
                    if let Ok(t) = Tuple::cast_from(&source) {
                        context.set_register(destination, t[n])
                    } else {
                        panic!("GetTupleElement: source is of wrong type")
                    }
                }
                &Instruction::SetTupleElement { new_element, tuple, position } => {
                    let el = context.expand_arg(new_element);
                    let tuple = context.expand_arg(tuple);
                    let pos = position as usize;
                    if let Ok(t) = tuple.cast_into_mut() {
                        let tuple: &mut value::Tuple = t; // annoying, need type annotation
                        tuple[pos] = el;
                    } else {
                        panic!("GetTupleElement: source is of wrong type")
                    }
                }
                &Instruction::PutList { head, tail, destination } => {
                    // Creates a cons cell with [H|T] and places the value into Dst.
                    let head = context.expand_arg(head);
                    let tail = context.expand_arg(tail);
                    let cons = cons!(&context.heap, head, tail);
                    context.set_register(destination, cons)
                }
                Instruction::PutTuple2 { source, list } => {
                    // op: PutTuple2, args: [X(0), ExtendedList([Y(1), Y(0), X(0)])] }
                    let arity = list.len();

                    let tuple = value::tuple(&context.heap, arity as u32);
                    for i in 0..arity {
                        unsafe {
                            std::ptr::write(&mut tuple[i], context.expand_entry(&list[i]));
                        }
                    }
                    let tuple = Term::from(tuple);
                    context.set_register(*source, tuple);
                }
                &Instruction::Badmatch { value } => {
                    let value = context.expand_arg(value);
                    println!("Badmatch: {}", value);
                    return Err(Exception::with_value(Reason::EXC_BADMATCH, value));
                }
                &Instruction::IfEnd => {
                    // Raises the if_clause exception.
                    return Err(Exception::new(Reason::EXC_IF_CLAUSE));
                }
                &Instruction::CaseEnd { value } => {
                    // Raises the case_clause exception with the value of Arg0
                    let value = context.expand_arg(value);
                    println!("err=case_clause val={}", value);
                    // err=case_clause val={{820515101, #Ref<0.0.0.105>}, :timeout, {:gen_server, :cast, [#Pid<61>, :repeated_filesync]}}
                    return Err(Exception::with_value(Reason::EXC_CASE_CLAUSE, value));
                }
                &Instruction::Try { fail, register } => {
                    // TODO: try is identical to catch, and is remapped in the OTP loader
                    // create a catch context that wraps f - fail label, and stores to y - reg.
                    context.catches += 1;
                    context.set_register(
                        register,
                        Term::catch(
                            &context.heap,
                            InstrPtr {
                                ptr: fail,
                                module: context.ip.module
                            }
                        )
                    );
                }
                &Instruction::TryEnd { register } => {
                    context.catches -= 1;
                    context.set_register(register, Term::nil()) // TODO: make_blank macro
                }
                &Instruction::TryCase { register } => {
                    // pops a catch context in y  Erases the label saved in the Arg0 slot. Noval in R0 indicate that something is caught. If so, R0 is set to R1, R1 — to R2, R2 — to R3.

                    // TODO: this initial part is identical to TryEnd
                    context.catches -= 1;
                    context.set_register(register, Term::nil()); // TODO: make_blank macro

                    assert!(context.x[0].is_none());
                    // TODO: c_p->fvalue = NIL;
                    // TODO: make more efficient via memmove
                    context.x[0] = context.x[1];
                    context.x[1] = context.x[2];
                    context.x[2] = context.x[3];
                }
                &Instruction::TryCaseEnd { value } => {
                    // Raises a try_clause exception with the value read from Arg0.
                    let value = context.expand_arg(value);
                    return Err(Exception::with_value(Reason::EXC_TRY_CLAUSE, value));
                }
                &Instruction::Catch { fail, register } => {
                    // create a catch context that wraps f - fail label, and stores to y - reg.
                    context.catches += 1;
                    context.set_register(
                        register,
                        Term::catch(
                            &context.heap,
                            InstrPtr {
                                ptr: fail,
                                module: context.ip.module
                            }
                        )
                    );
                }
                &Instruction::CatchEnd { register } => {
                    // Pops a “catch” context. Erases the label saved in the Arg0 slot. Noval in R0
                    // indicates that something is caught. If R1 contains atom throw then R0 is set
                    // to R2. If R1 contains atom error than a stack trace is added to R2. R0 is
                    // set to {exit,R2}.
                    //
                    // difference fron try is, try will exhaust all the options then fail, whereas
                    // catch will keep going upwards.

                    // TODO: this initial part is identical to TryEnd
                    context.catches -= 1; // TODO: this is overflowing
                    context.set_register(register, Term::nil()); // TODO: make_blank macro

                    if context.x[0].is_none() {
                        // c_p->fvalue = NIL;
                        if context.x[1] == Term::atom(atom::THROW) {
                            context.x[0] = context.x[2]
                        } else {
                            if context.x[1] == Term::atom(atom::ERROR) {
                                context.x[2] =
                                    exception::add_stacktrace(&process, context.x[2], context.x[3]);
                            }
                            // only x(2) is included in the rootset here
                            // if (E - HTOP < 3) { check for heap space, otherwise garbage collect
                            // ..
                            //     FCALLS -= erts_garbage_collect_nobump(c_p, 3, reg+2, 1, FCALLS);
                            // }
                            context.x[0] =
                                tup2!(&context.heap, Term::atom(atom::EXIT_U), context.x[2]);
                        }
                    }
                }
                &Instruction::Raise { trace, value } => {
                    // Raises the exception. The instruction is garbled by backward compatibility. Arg0 is a stack trace
                    // and Arg1 is the value accompanying the exception. The reason of the raised exception is dug up
                    // from the stack trace
                    let trace = context.fetch_register(trace);
                    let value = context.expand_arg(value);

                    let reason = if let Some(s) = exception::get_trace_from_exc(&trace) {
                        primary_exception!(s.reason)
                    } else {
                        Reason::EXC_ERROR
                    };
                    return Err(Exception {
                        reason,
                        value,
                        trace,
                    });
                }
                &Instruction::Apply { arity } => {
                    context.cp = Some(context.ip);

                    op_fixed_apply!(self, context, &process, arity, false);
                    // call this fixed_apply, used for ops (apply, apply_last).
                    // apply is the func that's equivalent to erlang:apply/3 (and instrs)
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::ApplyLast { arity, nwords } => {
                    op_deallocate!(context, nwords);

                    op_fixed_apply!(self, context, &process, arity, true);
                    safepoint_and_reduce!(self, process, context.reds);
                }
                &Instruction::GcBif1 { label: fail, bif, arg1, live, reg } => {
                    // TODO: GcBif needs to handle GC as necessary
                    let args = &[context.expand_arg(arg1)];
                    match bif(self, &process, args) {
                        Ok(val) => context.set_register(reg, val),
                        Err(exc) => cond_fail!(context, fail, exc),
                    }
                }
                &Instruction::BsAdd { fail, size1, size2, unit, destination: dest} => {
                    // Calculates the total of the number of bits in Src1 and the number of units in Src2. Stores the result to Dst.
                    // bs_add(Fail, Src1, Src2, Unit, Dst)
                    // dst = (src1 + src2) * unit

                    // TODO: trickier since we need to check both nums are positive and can fit
                    // into int/bigint

                    // optimize when one append is 0 and unit is 1, it's just a move
                    // bs_add Fail S1=i==0 S2 Unit=u==1 D => move S2 D

                    // TODO use fail label
                    let s1 = context.expand_arg(size1).to_uint().unwrap();
                    let s2 = context.expand_arg(size2).to_uint().unwrap();

                    let res = Term::uint(&context.heap, (s1 + s2) * unit as u32);
                    context.set_register(dest, res)
                }
                &Instruction::BsInit2 { fail, size, words, regs, flags, destination: dest } => {
                    // optimize when init is with empty size
                    // bs_init2 Fail Sz=u Words=u==0 Regs Flags Dst => i_bs_init Sz Regs Dst

                    // Words is heap alloc size
                    // regs is live regs for GC
                    // flags is unused

                    // bs_init2 Fail Sz Words=u==0 Regs Flags Dst => \
                    //   i_bs_init_fail Sz Fail Regs Dst
                    //   op1 = size, op2 = 0?,
                    //   verify that the size is within limits (if non0) or jump to fail
                    //   allocate binary + procbin
                    //   set as non writable initially??

                    // TODO: use a current_string ptr to be able to write to the Arc wrapped str
                    // alternatively, loop through the instrs until we hit a non bs_ instr.
                    // that way, no unsafe ptrs!
                    let size = size.to_val(context).to_int().unwrap() as usize;
                    let mut binary = bitstring::Binary::with_size(size);
                    binary.is_writable = false;
                    let term = Term::binary(&context.heap, binary);
                    // TODO ^ ensure this pointer stays valid after heap alloc
                    context.bs = bitstring::Builder::new(term.get_boxed_value::<bitstring::RcBinary>().unwrap());
                    context.set_register(dest, term);
                }
                &Instruction::BsPutInteger { fail, size, unit, flags, source: src } => {
                    // Size can be atom all
                    // TODO: fail label
                    let size = match context.expand_arg(size).into_variant() {
                        Variant::Integer(i) => i as usize,
                        // LValue::Literal(i) => i as usize, // TODO: unsure if correct
                        Variant::Atom(atom::START) => 0,
                        _ => unreachable!("{:?}", size),
                    };

                    let size = size * (unit as usize);

                    // ins: &Instruction { op: BsPutInteger, args: [Label(0), Integer(1024), Literal(1), Literal(0), Integer(0)] }
                    println!("put_integer src is {}\r", context.expand_arg(src));

                    context.bs.put_integer(size as usize, flags, context.expand_arg(src));
                }
                &Instruction::BsPutBinary { fail, size, unit, flags, source: src } => {
                    // TODO: fail label
                    if unit != 8 {
                        unimplemented!("bs_put_binary unit != 8");
                        //fail!(context, fail);
                    }

                    let source = context.expand_arg(src);

                    let header = source.get_boxed_header().unwrap();
                    match header {
                        value::BOXED_BINARY | value::BOXED_SUBBINARY => {
                            match context.expand_arg(size).into_variant() {
                                Variant::Atom(atom::ALL) => context.bs.put_binary_all(source),
                                _ => unimplemented!("bs_put_binary size {:?}", size),
                            }
                        }
                        _ => panic!("Bad argument to BsPutBinary: {}", context.expand_arg(src))
                    }
                }
                &Instruction::BsPutFloat { fail, size, unit, flags, source: src } => {
                    // Size can be atom all
                    // TODO: fail label
                    if unit != 8 {
                        unimplemented!("bs_put_float unit != 8");
                        //fail!(context, fail);
                    }

                    if let Variant::Float(value::Float(f)) =
                        context.expand_arg(src).into_variant()
                    {
                        match context.expand_arg(size).into_variant() {
                            Variant::Atom(atom::ALL) => context.bs.put_float(f),
                            _ => unimplemented!("bs_put_float size {:?}", size),
                        }
                    } else {
                        panic!("Bad argument to BsPutFloat")
                    }
                }
                Instruction::BsPutString { binary } => {
                    // BsPutString uses the StrT strings table! needs to be patched in loader
                    context.bs.put_bytes(binary);
                }
                &Instruction::BsGetTail { context: cxt, destination, live } => {
                    // very similar to the old BsContextToBinary
                     if let Ok(mb) = context
                         .fetch_register(cxt)
                         .get_boxed_value_mut::<bitstring::MatchBuffer>()
                         {
                           // TODO; original calculated the hole size and overwrote MatchBuffer mem in place?
                           let offs = mb.offset;
                           let size = mb.size - offs;

                           let res = Term::subbinary(
                               &context.heap,
                               bitstring::SubBinary::new(mb.original.clone(), size, offs, false),
                           );
                           context.set_register(destination, res);
                     } else {
                         // next0
                         unreachable!()
                     }
                }
                &Instruction::BsStartMatch3 { fail, source: src, live, destination: dst } => {
                    let cxt = context.fetch_register(src);

                    if !cxt.is_pointer() {
                        fail!(context, fail);
                    }

                    let header = cxt.get_boxed_header().unwrap();

                    // Reserve a slot for the start position.

                    match header {
                        value::BOXED_MATCHBUFFER => {
                            context.set_register(dst, cxt);
                        }
                        value::BOXED_BINARY | value::BOXED_SUBBINARY => {
                            // Uint wordsneeded = ERL_BIN_MATCHBUFFER_SIZE(slots);
                            // $GC_TEST_PRESERVE(wordsneeded, live, context);

                            let result = bitstring::start_match_3(&context.heap, cxt);

                            if let Some(res) = result {
                                context.set_register(dst, res)
                            } else {
                                fail!(context, fail);
                            }
                        }
                        _ => {
                            fail!(context, fail);
                        }
                    }
                }
                &Instruction::BsGetPosition { context: cxt, destination, live } => {
                    if let Ok(mb) = context
                        .fetch_register(cxt)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        // TODO: unsafe cast
                        context.set_register(destination, Term::uint(&context.heap, mb.offset as u32));
                    } else {
                        unreachable!()
                    };
                }
                &Instruction::BsSetPosition { context: cxt, position } => {
                    if let Ok(mb) = context
                        .fetch_register(cxt)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        let pos = context.expand_arg(position).to_int().unwrap();
                        mb.offset = pos as usize;
                    } else {
                        unreachable!()
                    };
                }
                &Instruction::BsGetInteger2 { fail, ms, live, size, unit, flags, destination } => {
                    let size = context.expand_arg(size).to_int().unwrap() as usize;
                    let unit = unit as usize;

                    let bits = size * unit;

                    // let size = size * (flags as usize >> 3); TODO: this was just because flags
                    // & size were packed together on BEAM

                    // TODO: this cast can fail
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        use std::convert::TryInto;
                        // fast path for common ops
                        let res = match (bits, flags.contains(bitstring::Flag::BSF_LITTLE), flags.contains(bitstring::Flag::BSF_SIGNED)) {
                            (8, true, true) => {
                                // little endian, signed
                                mb.get_bytes(1).map(|b| Term::int(i32::from(i8::from_le_bytes((*b).try_into().unwrap()))))
                            },
                            (8, true, false) => {
                                // little endian unsigned
                                mb.get_bytes(1).map(|b| Term::uint(&context.heap, u32::from(u8::from_le_bytes((*b).try_into().unwrap()))))
                            },
                            (8, false, true) => {
                                // big endian signed
                                mb.get_bytes(1).map(|b| Term::int(i32::from(i8::from_be_bytes((*b).try_into().unwrap()))))
                            },
                            (8, false, false) => {
                                // big endian unsigned
                                mb.get_bytes(1).map(|b| Term::uint(&context.heap, u32::from(u8::from_be_bytes((*b).try_into().unwrap()))))
                            },
                            (16, true, true) => {
                                // little endian, signed
                                mb.get_bytes(2).map(|b| Term::int(i32::from(i16::from_le_bytes((*b).try_into().unwrap()))))
                            },
                            (16, true, false) => {
                                // little endian unsigned
                                mb.get_bytes(2).map(|b| Term::uint(&context.heap, u32::from(u16::from_le_bytes((*b).try_into().unwrap()))))
                            },
                            (16, false, true) => {
                                // big endian signed
                                mb.get_bytes(2).map(|b| Term::int(i32::from(i16::from_be_bytes((*b).try_into().unwrap()))))
                            },
                            (16, false, false) => {
                                // big endian unsigned
                                mb.get_bytes(2).map(|b| Term::uint(&context.heap, u32::from(u16::from_be_bytes((*b).try_into().unwrap()))))
                            },
                            (32, true, true) => {
                                // little endian, signed
                                mb.get_bytes(4).map(|b| Term::int(i32::from_le_bytes((*b).try_into().unwrap())))
                            },
                            (32, true, false) => {
                                // little endian unsigned
                                mb.get_bytes(4).map(|b| Term::uint(&context.heap, u32::from_le_bytes((*b).try_into().unwrap())))
                            },
                            (32, false, true) => {
                                // big endian signed
                                mb.get_bytes(4).map(|b| Term::int(i32::from_be_bytes((*b).try_into().unwrap())))
                            },
                            (32, false, false) => {
                                // big endian unsigned
                                mb.get_bytes(4).map(|b| Term::uint(&context.heap, u32::from_be_bytes((*b).try_into().unwrap())))
                            },
                            (64, false, true) => {
                                // big endian signed
                                mb.get_bytes(8).map(|b| Term::int64(&context.heap, i64::from_be_bytes((*b).try_into().unwrap())))
                            },
                            (64, false, false) => {
                                // big endian unsigned
                                mb.get_bytes(8).map(|b| Term::uint64(&context.heap, u64::from_be_bytes((*b).try_into().unwrap())))
                            },
                            (128, false, true) => {
                                use num_bigint::ToBigInt;
                                // big endian signed
                                mb.get_bytes(16).map(|b| Term::bigint(&context.heap, i128::from_be_bytes((*b).try_into().unwrap()).to_bigint().unwrap()))
                            },
                            (128, false, false) => {
                                use num_bigint::ToBigInt;
                                // big endian unsigned
                                mb.get_bytes(16).map(|b| Term::bigint(&context.heap, u128::from_be_bytes((*b).try_into().unwrap()).to_bigint().unwrap()))
                            },
                            // slow fallback
                            _ => {
                                mb.get_integer(&context.heap, bits, flags)
                            }
                        };

                        if let Some(res) = res {
                            context.set_register(destination, res)
                        } else {
                            fail!(context, fail);
                        }
                    };
                }
                &Instruction::BsGetFloat2 { fail, ms, live, size, unit, flags, destination } => {
                    let size = match context.expand_arg(size).into_variant() {
                        Variant::Integer(size) if size <= 64 => size as usize,
                        _ => {
                            fail!(context, fail);
                        }
                    };

                    // TODO: this cast can fail
                    if let Ok(mb) = context.fetch_register(ms).get_boxed_value_mut::<bitstring::MatchBuffer>() {
                        let res = mb.get_float(&context.heap, size as usize, flags);
                        if let Some(res) = res {
                            context.set_register(destination, res)
                        } else {
                            fail!(context, fail);
                        }
                    };
                }
                &Instruction::BsGetBinary2 { fail, ms, live, size, unit, flags, destination } => {
                    // TODO: this cast can fail
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        // proc pid=37 reds=907 mod="re" offs=870 ins=BsGetBinary2 args=[Label(916), X(5), Literal(9), X(2), Literal(8), Literal(0), X(2)]
                        let heap = &context.heap;
                        let unit = unit as usize;
                        let res = match context.expand_arg(size).into_variant() {
                            Variant::Integer(size) => mb.get_binary(heap, size as usize * unit, flags),
                            Variant::Atom(atom::ALL) => mb.get_binary_all(heap, flags),
                            arg => unreachable!("get_binary2 for {:?}", arg),
                        };

                        if let Some(res) = res {
                            context.set_register(destination, res)
                        } else {
                            fail!(context, fail);
                        }
                    };
                }
                &Instruction::BsSkipBits2 { fail, ms, size, unit, flags } => {
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        let size = context.expand_arg(size).to_int().unwrap() as usize;

                        let new_offset = mb.offset + (size * unit as usize);

                        if new_offset <= mb.size {
                            mb.offset = new_offset;
                        } else {
                            fail!(context, fail);
                        }
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::BsTestTail2 { fail, ms, bits: offset } => {
                    // TODO: beam specializes bits 0
                    // Checks that the matching context Arg1 has exactly Arg2 unmatched bits. Jumps
                    // to the label Arg0 if it is not so.
                    // if size 0 == Jumps to the label in Arg0 if the matching context Arg1 still have unmatched bits.
                    let offset = offset as usize;

                    if let Ok(mb) = bitstring::MatchBuffer::cast_from(&context.fetch_register(ms)) {
                        if mb.remaining() != offset {
                            fail!(context, fail);
                        }
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::BsTestUnit { fail, context: cxt, unit } => {
                    // Checks that the size of the remainder of the matching context is divisible
                    // by unit, else jump to fail

                    if let Ok(mb) = bitstring::MatchBuffer::cast_from(&context.fetch_register(cxt)) {
                        if mb.remaining() % (unit as usize) != 0 {
                            fail!(context, fail);
                        }
                    } else {
                        unreachable!()
                    }
                }
                Instruction::BsMatchString { fail, context: cxt, bits, string } => {
                    // byte* bytes = (byte *) $Ptr;
                    // Uint bits = $Bits;
                    // ErlBinMatchBuffer* mb;
                    // Uint offs;

                    if let Ok(mb) = context.fetch_register(*cxt).get_boxed_value_mut::<bitstring::MatchBuffer>() {
                        let bits = *bits as usize;

                        if mb.remaining() < bits {
                            fail!(context, *fail);
                        }
                        // offs = mb->offset & 7;
                        // if (offs == 0 && (bits & 7) == 0) {
                        //     if (sys_memcmp(bytes, mb->base+(mb->offset>>3), bits>>3)) {
                        //         $FAIL($Fail);
                        //     }

                        // No need for memcmp fastpath, cmp_bits already does that.
                        unsafe {
                            if bitstring::cmp_bits(
                                string.as_ptr(),
                                0,
                                mb.original.data.as_ptr().add(mb.offset >> 3),
                                mb.offset & 7,
                                bits,
                            ) != std::cmp::Ordering::Equal
                            {
                                fail!(context, *fail);
                            }
                        }
                        mb.offset += bits;
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::BsInitWritable => {
                    context.x[0] = bitstring::init_writable(&process, context.x[0]);
                }
                // BsGet and BsSkip should be implemented over an Iterator inside a match context (.skip/take)
                // maybe we can even use nom for this
                &Instruction::BsAppend { fail ,size, extra, live, unit, bin, destination } => {
                    // append and init also sets the string as current (state.current_binary) [seems to be used to copy string literals too]

                    // bs_append Fail Size Extra Live Unit Bin Flags Dst => \
                    //   move Bin x | i_bs_append Fail Extra Live Unit Size Dst

                    let size = context.expand_arg(size);
                    let extra_words = extra as usize;
                    let unit = unit as usize;
                    let src = context.expand_arg(bin);

                    let res = bitstring::append(&process, src, size, extra_words, unit);

                    if let Some(res) = res {
                        context.set_register(destination, res)
                    } else {
                        // TODO: execute fail only if non zero, else raise
                        /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
                        fail!(context, fail);
                    }
                }
                &Instruction::BsPrivateAppend { fail, size, unit, bin, destination } => {
                    // bs_private_append Fail Size Unit Bin Flags Dst

                    let size = context.expand_arg(size);
                    let unit = unit as usize;
                    let src = context.expand_arg(bin);

                    let res = bitstring::private_append(&process, src, size, unit);

                    if let Some(res) = res {
                        context.set_register(destination, res)
                    } else {
                        /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
                        fail!(context, fail);
                    }
                    unimplemented!("bs_private_append") // TODO
                }
                &Instruction::BsInitBits { fail ,size, words, regs, flags, destination } => {
                    // bs_init_bits Fail Sz=u Words=u==0 Regs Flags Dst => i_bs_init Sz Regs Dst

                    // Words is heap alloc size
                    // regs is live regs for GC
                    // flags is unused?

                    // size is in bits
                    // debug_assert_eq!(ins.args.len(), 6);
                    // TODO: RcBinary has to be set to is_writable = false
                    // context.x[0] = bitstring::init_bits(&process, context.x[0], ..);
                    unimplemented!("bs_init_bits") // TODO
                }
                &Instruction::BsGetUtf8 { fail, ms, size, flags, destination } => {
                    // TODO: this cast can fail
                    if let Ok(mb) = context.fetch_register(ms).get_boxed_value_mut::<bitstring::MatchBuffer>() {
                        let res = mb.get_utf8();
                        if let Some(res) = res {
                            context.set_register(destination, res)
                        } else {
                            fail!(context, fail);
                        }
                    };
                }
                &Instruction::BsGetUtf16 { fail, ms, size, flags, destination } => {
                    // TODO: this cast can fail
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        let res = mb.get_utf16(flags);
                        if let Some(res) = res {
                            context.set_register(destination, res)
                        } else {
                            fail!(context, fail);
                        }
                    };
                }
                &Instruction::BsGetUtf32 { fail, ms, size, flags, destination } => {
                    unimplemented!()
                }
                &Instruction::BsSkipUtf8 { fail, ms, size, flags } => {
                    // TODO: this cast can fail
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        let res = mb.get_utf8();
                        if res.is_none() {
                            fail!(context, fail);
                        }
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::BsSkipUtf16 { fail, ms, size, flags } => {
                    // TODO: this cast can fail
                    if let Ok(mb) = context
                        .fetch_register(ms)
                        .get_boxed_value_mut::<bitstring::MatchBuffer>()
                    {
                        let res = mb.get_utf16(flags);
                        if res.is_none() {
                            fail!(context, fail);
                        }
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::BsSkipUtf32 { fail, ms, size, flags } => {
                    unimplemented!()
                }
                &Instruction::BsUtf8Size { fail, source, destination } => {
                    unimplemented!()
                }
                &Instruction::BsUtf16Size { fail, source, destination } => {
                    unimplemented!()
                }
                &Instruction::BsPutUtf8 { fail, source, flags } => {
                    unimplemented!()
                }
                &Instruction::BsPutUtf16 { fail, source, flags } => {
                    unimplemented!()
                }
                &Instruction::BsPutUtf32 { fail, source, flags } => {
                    unimplemented!()
                }
                &Instruction::Fclearerror => {
                    // TODO: BEAM checks for unhandled errors
                    context.f[0] = 0.0;
                }
                &Instruction::Fcheckerror => {
                    // I think it always checks register fr0
                    if !context.f[0].is_finite() {
                        return Err(Exception::new(Reason::EXC_BADARITH));
                    }
                }
                &Instruction::Fmove { source, destination } => {
                    use crate::instruction::FRegister;

                    // basically a normal move, except if we're moving out of floats we make a Val
                    // otherwise we keep as a float
                    let f = expand_float!(context, source);

                    match destination {
                        FRegister::X(reg) => {
                            context.x[reg as usize] = Term::from(f);
                        }
                        FRegister::Y(reg) => {
                            let len = context.stack.len();
                            context.stack[len - (reg + 1) as usize] = Term::from(f);
                        }
                        FRegister::FloatReg(reg) => {
                            context.f[reg as usize] = f;
                        }
                        _reg => unimplemented!(),
                    }
                }
                &Instruction::Fconv { source, destination } => {
                    // reg (x), dest (float reg)
                    use crate::instruction::FRegister;
                    let value = match source {
                        FRegister::ExtendedLiteral(i) => unsafe { (*context.ip.module).literals[i as usize] },
                        FRegister::X(reg) => context.x[reg as usize],
                        FRegister::Y(reg) => {
                            let len = context.stack.len();
                            context.stack[len - (reg + 1) as usize]
                        }
                        FRegister::FloatReg(reg) => Term::from(context.f[reg as usize]),
                        _ => unreachable!(),
                    };

                    let val: f64 = match value.into_number() {
                        Ok(value::Num::Float(f)) => f,
                        Ok(value::Num::Integer(i)) => f64::from(i),
                        Ok(_) => unimplemented!(),
                        // TODO: bignum if it fits into float
                        Err(_) => return Err(Exception::new(Reason::EXC_BADARITH)),
                    };

                    context.f[destination as usize] = val;
                }
                // fail is unused
                &Instruction::Fadd { fail: _, a, b, destination } => op_float!(context, a, b, destination, +),
                &Instruction::Fsub { fail: _, a, b, destination } => op_float!(context, a, b, destination, -),
                &Instruction::Fmul { fail: _, a, b, destination } => op_float!(context, a, b, destination, *),
                &Instruction::Fdiv { fail: _, a, b, destination } => op_float!(context, a, b, destination, /),
                &Instruction::Fnegate { source, destination } => {
                    context.f[destination as usize] = -context.f[source as usize];
                }
                &Instruction::GcBif2 { label: fail, bif, arg1, arg2, live, reg } => {
                    // TODO: GcBif needs to handle GC as necessary
                    let args = &[
                        context.expand_arg(arg1),
                        context.expand_arg(arg2),
                    ];

                    match bif(self, &process, args) {
                        Ok(val) => context.set_register(reg, val),
                        Err(exc) => cond_fail!(context, fail, exc),
                    }
                }
                &Instruction::GcBif3 { label: fail, bif, arg1, arg2, arg3, live, reg } => {
                    // TODO: GcBif needs to handle GC as necessary
                    let args = &[
                        context.expand_arg(arg1),
                        context.expand_arg(arg2),
                        context.expand_arg(arg3),
                    ];

                    match bif(self, &process, args) {
                        Ok(val) => context.set_register(reg, val),
                        Err(exc) => cond_fail!(context, fail, exc),
                    }
                }
                &Instruction::Trim { n: nwords, remaining }=> {
                    // trim N, _remain
                    // drop N words from stack, (but keeping the cp). Second arg unused?
                    // let cp = context.stack.pop().unwrap();
                    context
                        .stack
                        .truncate(context.stack.len() - nwords as usize);
                    // context.stack.push(cp);
                }
                &Instruction::MakeFun2 { i } => {
                    // nfree means capture N x-registers into the closure
                    let lambda = &module.lambdas[i as usize];

                    let binding = if lambda.nfree != 0 {
                        Some(context.x[0..(lambda.nfree as usize)].to_vec())
                    } else {
                        None
                    };

                    context.x[0] = Term::closure(
                        &context.heap,
                        value::Closure {
                            // arity is arity minus nfree (beam_emu.c)
                            mfa: module::MFA(module.name, lambda.name, lambda.arity - lambda.nfree),
                            // TODO: use module id instead later
                            ptr: lambda.offset,
                            binding,
                        },
                    );
                }
                &Instruction::CallFun { arity } => {
                    let value = context.x[arity as usize];
                    context.cp = Some(context.ip);
                    if let Ok(closure) = value::Closure::cast_from(&value) {
                        op_call_fun!(self, context, closure, arity)
                    } else if let Ok(mfa) = module::MFA::cast_from(&value) {
                        // TODO: deduplicate this part
                        let export = { self.exports.read().lookup(&mfa) }; // drop the exports lock

                        match export {
                            Some(Export::Fun(ptr)) => op_jump_ptr!(context, ptr),
                            Some(Export::Bif(bif)) => {
                                op_call_bif!(self, context, &process, bif, mfa.2 as usize, true) // TODO is return true ok
                            }
                            None => {
                                // println!("apply setup_error_handler");
                                call_error_handler!(self, &process, &mfa);
                                // apply_setup_error_handler
                            }
                        }
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::GetHd { source, head: reg } => {
                    if let Ok(value::Cons { head, .. }) =
                        Cons::cast_from(&context.fetch_register(source))
                    {
                        context.set_register(reg, *head);
                    } else {
                        unreachable!()
                    }
                }
                &Instruction::GetTl { source, tail: reg } => {
                    if let Ok(value::Cons { tail, .. }) =
                        context.fetch_register(source).cast_into()
                    {
                        context.set_register(reg, *tail);
                    } else {
                        unreachable!()
                    }
                }
                Instruction::PutMapAssoc { map, destination, live, rest: list } => {
                    let mut map = match context.expand_arg(*map).cast_into() {
                        Ok(value::Map(map)) => map.clone(),
                        _ => unreachable!(),
                    };

                    let mut iter = list.chunks_exact(2);
                    while let Some([key, value]) = iter.next() {
                        // TODO: optimize by having the ExtendedList store Term instead of LValue
                        map.insert(context.expand_entry(key), context.expand_entry(value));
                    }
                    context.set_register(*destination, Term::map(&context.heap, map))
                }
                Instruction::HasMapFields { label: fail, source, rest: list } => {
                    let map = match context.fetch_register(*source).cast_into() {
                        Ok(value::Map(map)) => map.clone(),
                        _ => unreachable!(),
                    };

                    // N is a list of the type [key => dest_reg], if any of these fields don't
                    // exist, jump to fail label.
                    let mut iter = list.iter();
                    while let Some(key) = iter.next() {
                        if map.contains_key(&context.expand_entry(key)) {
                            // ok
                        } else {
                            op_jump!(context, *fail);
                            break;
                        }
                    }
                }
                Instruction::GetMapElements { label: fail, source, rest: list } => {
                    let map = match context.fetch_register(*source).cast_into() {
                        Ok(value::Map(map)) => map.clone(),
                        _ => unreachable!(),
                    };

                    // N is a list of the type [key => dest_reg], if any of these fields don't
                    // exist, jump to fail label.
                    let mut iter = list.chunks_exact(2);
                    while let Some([key, dest]) = iter.next() {
                        if let Some(&val) = map.get(&context.expand_entry(key)) {
                            context.set_register(dest.into_register(), val)
                        } else {
                            op_jump!(context, *fail);
                            break; // TODO: original impl loops over everything
                        }
                    }
                }
                Instruction::PutMapExact { map, destination, live, rest: list } => {
                    // TODO: in the future, put_map_exact is an optimization for flatmaps (tuple
                    // maps), where the keys in the tuple stay the same, since you can memcpy the
                    // key tuple (and other optimizations)
                    let mut map = match context.expand_arg(*map).cast_into() {
                        Ok(value::Map(map)) => map.clone(),
                        _ => unreachable!(),
                    };

                    let mut iter = list.chunks_exact(2);
                    while let Some([key, value]) = iter.next() {
                        // TODO: optimize by having the ExtendedList store Term instead of LValue
                        map.insert(context.expand_entry(key), context.expand_entry(value));
                    }
                    context.set_register(*destination, Term::map(&context.heap, map))
                }
                &Instruction::IsTaggedTuple { fail, source, arity, atom } => {
                    let reg = context.fetch_register(source);
                    let atom = context.expand_arg(atom);
                    if let Ok(tuple) = Tuple::cast_from(&reg) {
                        if tuple.len == 0 || tuple.len != arity || !tuple[0].eq(&atom) {
                            fail!(context, fail);
                        } else {
                            // ok
                        }
                    } else {
                        fail!(context, fail);
                    }
                }
                &Instruction::BuildStacktrace => {
                    context.x[0] = exception::build_stacktrace(&process, context.x[0]);
                }
                &Instruction::RawRaise => {
                    let class = &context.x[0];
                    let value = context.x[1];
                    let trace = context.x[2];

                    match class.into_variant() {
                        Variant::Atom(atom::ERROR) => {
                            let mut reason = Reason::EXC_ERROR;
                            reason.remove(Reason::EXF_SAVETRACE);
                            return Err(Exception {
                                reason,
                                value,
                                trace,
                            });
                        }
                        Variant::Atom(atom::EXIT) => {
                            let mut reason = Reason::EXC_EXIT;
                            reason.remove(Reason::EXF_SAVETRACE);
                            return Err(Exception {
                                reason,
                                value,
                                trace,
                            });
                        }
                        Variant::Atom(atom::THROW) => {
                            let mut reason = Reason::EXC_THROWN;
                            reason.remove(Reason::EXF_SAVETRACE);
                            return Err(Exception {
                                reason,
                                value,
                                trace,
                            });
                        }
                        _ => context.x[0] = Term::atom(atom::BADARG),
                    }
                }
                // ins => unimplemented!("opcode unimplemented: {:?}", ins)
            }
        }
        }
    }
}
