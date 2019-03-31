use crate::atom;
use crate::port;
use crate::bif;
use crate::bitstring;
use crate::ets::{RcTableRegistry, TableRegistry};
use crate::exception::{self, Exception, Reason};
use crate::exports_table::{Export, ExportsTable, RcExportsTable};
use crate::instr_ptr::InstrPtr;
use crate::loader::LValue;
use crate::module;
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::opcodes::Opcode;
use crate::process::registry::Registry as ProcessRegistry;
use crate::process::table::Table as ProcessTable;
use crate::port::{Table as PortTable, RcTable as RcPortTable};
use crate::process::{self, RcProcess};
use crate::servo_arc::Arc;
use crate::value::{self, Cons, Term, TryFrom, TryInto, TryIntoMut, Tuple, Variant};
use std::cell::RefCell;
// use log::debug;
use parking_lot::Mutex;
use std::mem::transmute;
use std::panic;
use std::sync::atomic::AtomicUsize;
use std::time;

use tokio::await;
use tokio::prelude::*;

use futures::future::{self, FutureExt};
use futures::select;
// use futures::stream::{self, StreamExt};

use futures_native_timers::{Delay, Interval};

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

    // pub exit:

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    /// export table
    pub exports: RcExportsTable,

    /// Module registry
    pub modules: RcModuleRegistry,

    pub ets_tables: RcTableRegistry,
}

impl Machine {
    pub fn next_ref(&self) -> process::Ref {
        self.next_ref
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}


/* meh, copied from tokio async/await feature */
use std::future::Future as StdFuture;
use std::pin::Pin;
use std::task::{Poll, Waker};
fn map_ok<T: StdFuture>(future: T) -> impl StdFuture<Output = Result<(), ()>> {
    MapOk(future)
}

struct MapOk<T>(T);

impl<T> MapOk<T> {
    fn future<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut T> {
        unsafe { Pin::map_unchecked_mut(self, |x| &mut x.0) }
    }
}

impl<T: StdFuture> StdFuture for MapOk<T> {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        match self.future().poll(waker) {
            Poll::Ready(_) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
    }
}
/* meh */

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

macro_rules! set_register {
    ($context:expr, $register:expr, $value:expr) => {{
        match $register {
            &LValue::X(reg) => {
                $context.x[reg as usize] = $value;
            }
            &LValue::Y(reg) => {
                let len = $context.stack.len();
                $context.stack[(len - (reg + 2) as usize)] = $value;
            }
            _reg => unimplemented!("set_reg"),
        }
    }};
}

macro_rules! expand_float {
    ($context:expr, $value:expr) => {{
        match &$value {
            LValue::ExtendedLiteral(i) => unsafe {
                if let Variant::Float(value::Float(f)) =
                    (*$context.ip.module).literals[*i as usize].into_variant()
                {
                    f
                } else {
                    unreachable!()
                }
            },
            LValue::X(reg) => {
                if let Variant::Float(value::Float(f)) = $context.x[*reg as usize].into_variant() {
                    f
                } else {
                    unreachable!()
                }
            }
            LValue::Y(reg) => {
                let len = $context.stack.len();
                if let Variant::Float(value::Float(f)) =
                    $context.stack[len - (*reg + 2) as usize].into_variant()
                {
                    f
                } else {
                    unreachable!()
                }
            }
            LValue::FloatReg(reg) => $context.f[*reg as usize],
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
        let cp = $context.stack.pop().unwrap();
        $context
            .stack
            .truncate($context.stack.len() - $nwords as usize);

        if let Ok(value::Boxed {
            header: value::BOXED_CP,
            value: cp,
        }) = cp.try_into()
        {
            $context.cp = *cp;
        } else {
            panic!("Bad CP value! {:?}", cp)
        }
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
    process: &Pin<&mut process::Process>,
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
    // TODO: I don't like from_iter requiring cloned
    let args = Cons::from_iter(context.x[0..mfa.2 as usize].iter().cloned(), &context.heap);

    // Set up registers for call to error_handler:<func>/3.
    context.x[0] = Term::atom(mfa.0); // module
    context.x[1] = Term::atom(mfa.1); // func
    context.x[2] = Term::from(args);
    op_jump_ptr!(context, ptr);
    Ok(())
}

const APPLY_2: bif::Fn = bif::bif_erlang_apply_2;
const APPLY_3: bif::Fn = bif::bif_erlang_apply_3;

macro_rules! op_call_ext {
    ($vm:expr, $context:expr, $process:expr, $arity:expr, $dest: expr, $return: expr) => {{
        let mfa = unsafe { &(*$context.ip.module).imports[*$dest as usize] };

        // println!("pid={} action=call_ext mfa={}", $process.pid, mfa);

        let export = { $vm.exports.read().lookup(mfa) }; // drop the exports lock

        match export {
            Some(Export::Fun(ptr)) => op_jump_ptr!($context, ptr),
            Some(Export::Bif(APPLY_2)) => {
                // I'm cheating here, *shrug*
                op_apply_fun!($vm, $context)
            }
            Some(Export::Bif(APPLY_3)) => {
                // I'm cheating here, *shrug*
                op_apply!($vm, $context, $process, $return);
            }
            Some(Export::Bif(bif)) => {
                // TODO: precompute which exports are bifs
                // also precompute the bif lookup
                // call_ext_only Ar=u Bif=u$is_bif => \
                // allocate u Ar | call_bif Bif | deallocate_return u

                // make a slice out of arity x registers
                let args = &$context.x[0..*$arity as usize];
                match bif($vm, $process, args) {
                    // TODO: should use call_bif?
                    Ok(val) => {
                        set_register!($context, &LValue::X(0), val); // HAXX
                        if $return {
                            // TODO: figure out returns
                            op_return!($process, $context);
                        }
                    }
                    Err(exc) => return Err(exc),
                }
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
                set_register!($context, &LValue::X(0), val); // HAXX
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
        // Walk down the 3rd parameter of apply (the argument list) and copy
        // the parameters to the x registers (reg[]).
        let module = $context.x[0];
        let func = $context.x[1];
        let mut tmp = $context.x[2];

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

        // Handle apply of apply/3...
        if module.to_u32() == atom::ERLANG && func.to_u32() == atom::APPLY {
            unimplemented!("apply of apply")
            // continually loop over args to resolve
        }

        let mut arity = 0;

        while let Ok(value::Cons { head, tail }) = tmp.try_into() {
            if arity < process::MAX_REG - 1 {
                $context.x[arity] = *head;
                arity += 1;
                tmp = *tail
            } else {
                return Err(Exception::new(Reason::EXC_SYSTEM_LIMIT));
            }
        }

        if !tmp.is_nil() {
            /* Must be well-formed list */
            return Err(Exception::new(Reason::EXC_BADARG));
        }

        /*
         * Get the index into the export table, or failing that the export
         * entry for the error handler module.
         *
         * Note: All BIFs have export entries; thus, no special case is needed.
         */

        let mfa = module::MFA(module.to_u32(), func.to_u32(), arity as u32);

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
    ($vm:expr, $context:expr) => {{
        // Walk down the 3rd parameter of apply (the argument list) and copy
        // the parameters to the x registers (reg[]).

        let fun = $context.x[0];
        let mut tmp = $context.x[1];
        let mut arity = 0;

        while let Ok(value::Cons { head, tail }) = tmp.try_into() {
            if arity < process::MAX_REG - 1 {
                $context.x[arity] = *head;
                arity += 1;
                tmp = *tail
            } else {
                return Err(Exception::new(Reason::EXC_SYSTEM_LIMIT));
            }
        }

        if !tmp.is_nil() {
            /* Must be well-formed list */
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        //context.x[arity] = fun;

        if let Ok(closure) = value::Closure::try_from(&fun) {
            op_call_fun!($vm, $context, closure, arity);
        } else {
            // TODO raise error
            unimplemented!("TODO: possible EXPORT")
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
            break;
        }
    }};
}

macro_rules! fail {
    ($context:expr, $label:expr) => {{
        let fail = $label.to_u32();
        op_jump!($context, fail);
        continue;
    }};
}

macro_rules! cond_fail {
    ($context:expr, $label:expr, $exc: expr) => {{
        let fail = $label.to_u32();
        if fail != 0 {
            op_jump!($context, fail);
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

        // Handle apply of apply/3...
        if module.to_u32() == atom::ERLANG && func.to_u32() == atom::APPLY && $arity == 3 {
            unimplemented!("fixed_apply of apply")
            // return apply(p, reg, I, stack_offset);
        }

        /*
         * Get the index into the export table, or failing that the export
         * entry for the error handler module.
         *
         * Note: All BIFs have export entries; thus, no special case is needed.
         */

        let mfa = module::MFA(module.to_u32(), func.to_u32(), $arity);

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
    ($context:expr, $args:expr, $op:ident) => {{
        debug_assert_eq!($args.len(), 2);

        let val = $context.expand_arg(&$args[1]);

        if !val.$op() {
            // TODO: patch the labels to point to exact offsets to avoid labels lookup
            let fail = $args[0].to_u32();

            op_jump!($context, fail);
        }
    }};
}

macro_rules! to_expr {
    ($e:expr) => {
        $e
    };
}

macro_rules! op_float {
    ($context:expr, $args:expr, $op:tt) => {{
        debug_assert_eq!($args.len(), 4);

        // TODO: I think this fail is always unused
        if let [_fail, LValue::FloatReg(a), LValue::FloatReg(b), LValue::FloatReg(dest)] = $args {
            $context.f[*dest as usize] = to_expr!($context.f[*a as usize] $op $context.f[*b as usize]);
        } else {
            unreachable!()
        }
    }};
}

macro_rules! safepoint_and_reduce {
    ($vm:expr, $process:expr, $reductions:expr) => {{
        // if $vm.gc_safepoint(&$process) {
        //     return Ok(());
        // }

        // Reduce once we've exhausted all the instructions in a
        // context.
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

pub const PRE_LOADED: &[&str] = &[
    "examples/preloaded/ebin/erts_code_purger.beam",
    "examples/preloaded/ebin/erl_init.beam",
    "examples/preloaded/ebin/init.beam",
    "examples/preloaded/ebin/prim_buffer.beam",
    "examples/preloaded/ebin/prim_eval.beam",
    "examples/preloaded/ebin/prim_inet.beam",
    "examples/preloaded/ebin/prim_file.beam",
    "examples/preloaded/ebin/zlib.beam",
    "examples/preloaded/ebin/prim_zip.beam",
    "examples/preloaded/ebin/erl_prim_loader.beam",
    "examples/preloaded/ebin/erlang.beam",
    "examples/preloaded/ebin/erts_internal.beam",
    "examples/preloaded/ebin/erl_tracer.beam",
    "examples/preloaded/ebin/erts_literal_area_collector.beam",
    "examples/preloaded/ebin/erts_dirty_process_signal_handler.beam",
    "examples/preloaded/ebin/atomics.beam",
    "examples/preloaded/ebin/counters.beam",
    "examples/preloaded/ebin/persistent_term.beam",
];

impl Machine {
    pub fn new() -> Arc<Machine> {
        Arc::new(Machine {
            process_table: Mutex::new(ProcessTable::new()),
            process_registry: Mutex::new(ProcessRegistry::new()),
            port_table: PortTable::new(),
            start_time: time::Instant::now(),

            next_ref: AtomicUsize::new(1),
            exports: ExportsTable::with_rc(),
            modules: ModuleRegistry::with_rc(),
            ets_tables: TableRegistry::with_rc(),
        })
    }

    pub fn preload_modules(&self) {
        PRE_LOADED.iter().for_each(|path| {
            module::load_module(self, path).unwrap();
        })
    }

    /// Starts the VM
    ///
    /// This method will block the calling thread until it returns.
    ///
    /// This method returns true if the VM terminated successfully, false
    /// otherwise.
    pub fn start(self: &Arc<Self>, args: Vec<String>) {
        // let (trigger, exit) = futures::sync::oneshot::channel();

        // Create the runtime
        let machine = self.clone();
        let mut runtime = tokio::runtime::Builder::new()
            .after_start(move || {
                Machine::set_current(machine.clone()); // ughh double clone
            })
            .build()
            .expect("failed to start new Runtime");

        // Spawn the server task
        // let future = async {
        //     println!("Hello");
        // };
        // use tokio_async_await::compat::backward;
        // let future = backward::Compat::new(map_ok(future));
        // use futures::compat::Compat;
        // let future = Compat::new(future);
        // runtime.spawn(future);

        self.start_main_process(&mut runtime, args);

        // Wait until the runtime becomes idle and shut it down.
        runtime.shutdown_on_idle().wait().unwrap();
    }

    fn terminate(&self) {}

    /// Starts the main process
    pub fn start_main_process(&self, runtime: &mut tokio::runtime::Runtime, args: Vec<String>) {
        println!("Starting main process...");
        let registry = self.modules.lock();
        //let module = unsafe { &*module::load_module(self, path).unwrap() };
        let module = registry.lookup(atom::from_str("erl_init")).unwrap();
        let process = process::allocate(&self, 0 /* itself */, module).unwrap();

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
        println!("argv {}", context.x[1]);
        op_jump!(context, module.funs[&(fun, arity)]);
        /* TEMP */

        let process = process::cast(process);
        // use futures::compat::Compat;
        // let future = Compat::new(self.run_with_error_handling(process));
        use tokio_async_await::compat::backward;
        let future = run_with_error_handling(process);
        let future = backward::Compat::new(map_ok(future));
        runtime.spawn(future);

        // self.process_pool.schedule(process);
    }
}

    /// Executes a single process, terminating in the event of an error.
    pub async fn run_with_error_handling(
        mut process: Pin<&mut process::Process>
    ) -> () {

        // We are using AssertUnwindSafe here so we can pass a &mut Worker to
        // run()/panic(). This might be risky if values captured are not unwind
        // safe, so take care when capturing new variables.
        //let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let vm = Machine::current();
        loop {
            match await!(vm.run(&mut process)) {
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
                            println!("pid={} action=exited", process.pid);
                            break // crashed
                        }
                    } else {
                        // we're trapping, ip was already set, now reschedule the process
                        eprintln!("TRAP!");
                        // yield
                    }
                }
                Ok(process::State::Yield) => (), // yield
                Ok(process::State::Done) => break, // exited OK
                // TODO: wait is an await on a oneshot
                // TODO: waittimeout is an select on a oneshot or a delay
            }
        }
        ()
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
    #[allow(clippy::cyclomatic_complexity)]
    pub fn run<'a: 'd, 'b: 'd, 'c: 'd, 'd>(
        &'a self,
        process: &'b mut Pin<&'c mut process::Process>,
    ) -> impl std::future::Future<Output = Result<process::State, Exception>> + Captures<'a> + Captures<'b> + Captures<'c> + 'd {
        async move {  // workaround for https://github.com/rust-lang/rust/issues/56238
        let context = process.context_mut();
        context.reds = 1000; // self.config.reductions;

        // process the incoming signal queue
        process.process_incoming()?;

        loop {
            let module = context.ip.get_module();
            // let module = unsafe { &(*context.ip.module) };
            let ins = &module.instructions[context.ip.ptr as usize];
            context.ip.ptr += 1;

            // println!(
            //     "proc pid={:?} reds={:?} mod={:?} offs={:?} ins={:?} args={:?}",
            //     process.pid,
            //     context.reds,
            //     atom::to_str(module.name).unwrap(),
            //     context.ip.ptr,
            //     ins.op,
            //     ins.args
            // );

            match &ins.op {
                Opcode::FuncInfo => {
                    // Raises function clause exception. Arg1, Arg2, Arg3 are MFA. Arg0 is a mystery
                    // coming out of nowhere, probably, needed for NIFs.

                    // happens if no other clause matches
                    eprintln!(
                        "function clause! {} {} {:?}",
                        context.expand_arg(&ins.args[0]),
                        context.expand_arg(&ins.args[1]),
                        ins.args[2]
                    );
                    return Err(Exception::new(Reason::EXC_FUNCTION_CLAUSE));
                }
                Opcode::Return => {
                    op_return!(process, context);
                }
                Opcode::Send => {
                    // send x1 to x0, write result to x0
                    let pid = context.x[0]; // TODO can be pid or atom name
                    let msg = context.x[1];
                    // println!("sending from {} to {} msg {}", process.pid, pid, msg);
                    let res = match pid.into_variant() {
                        Variant::Port(id) => {
                            let res = self.port_table.read().lookup(id).map(|port| port.chan.clone());
                            if let Some(mut chan) = res {
                                // TODO: error unhandled
                                use futures::sink::SinkExt as FuturesSinkExt;
                                match Tuple::try_from(&msg) {
                                    Ok(tup) => {
                                        match tup[0].into_variant() {
                                            Variant::Atom(atom::COMMAND) => {
                                                // TODO: validate tuple len 2
                                                let bytes = tup[1].to_bytes().unwrap().to_owned();
                                                // let fut = chan
                                                //     .send(port::Signal::Command(bytes))
                                                //     .map_err(|_| ())
                                                //     .boxed()
                                                //     .compat();
                                                // TODO: can probably do without await!, if we make sure we don't need 'static
                                                println!("sending! {:?}", bytes);
                                                tokio::spawn_async(async move { await!(chan.send(port::Signal::Command(bytes))); });
                                            }
                                            _ => unimplemented!("msg to port {}", msg),
                                        }
                                    }
                                    _ => unimplemented!()
                                }
                            } else {
                                // TODO: handle errors properly
                                println!("NOTFOUND");
                            }
                            Ok(msg)
                        }
                        _ => process::send_message(self, process.pid, pid, msg),
                    }?;
                    context.x[0] = res;
                }
                Opcode::RemoveMessage => {
                    // Unlink the current message from the message queue. Remove any timeout.
                    process.local_data_mut().mailbox.remove();
                    // clear timeout
                    context.timeout.take();
                    // reset savepoint of the mailbox
                    process.local_data_mut().mailbox.reset();
                }
                Opcode::Timeout => {
                    //  Reset the save point of the mailbox and clear the timeout flag.
                    process.local_data_mut().mailbox.reset();
                    // clear timeout
                    context.timeout.take();
                }
                Opcode::LoopRec => {
                    // label, source
                    // grab message from queue, put to x0, if no message, jump to fail label
                    if let Some(msg) = process.receive()? {
                        // println!("recv proc pid={:?} msg={}", process.pid, msg);
                        context.x[0] = msg
                    } else {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    }
                }
                Opcode::LoopRecEnd => {
                    // label
                    // Advance the save pointer to the next message and jump back to Label.
                    debug_assert_eq!(ins.args.len(), 1);

                    process.local_data_mut().mailbox.advance();

                    let label = ins.args[0].to_u32();
                    op_jump!(context, label);
                }
                Opcode::Wait => {
                    // label
                    // jump to label, set wait flag on process
                    debug_assert_eq!(ins.args.len(), 1);

                    let label = ins.args[0].to_u32();
                    op_jump!(context, label);

                    // TODO: this currently races if the process is sending us
                    // a message while we're in the process of suspending.

                    // set wait flag
                    // process.set_waiting_for_message(true);
                    use futures::compat::Compat;
                    // LOCK mailbox on looprec, unlock on wait/waittimeout
                    let cancel = process.context_mut().recv_channel.take().unwrap();
                    await!(Compat::new(cancel.fuse())); // suspend process

                    // println!("pid={} resumption ", process.pid);
                    process.process_incoming()?;
                }
                Opcode::WaitTimeout => {
                    // @spec wait_timeout Lable Time
                    // @doc  Sets up a timeout of Time milliseconds and saves the address of the
                    //       following instruction as the entry point if the timeout triggers.

                    use futures::compat::Compat;
                    let cancel = process.context_mut().recv_channel.take().unwrap();

                    match context.expand_arg(&ins.args[1]).into_variant() {
                        Variant::Atom(atom::INFINITY) => {
                            // just a normal Opcode::Wait
                            // println!("infinity wait");
                            let label = ins.args[0].to_u32();
                            op_jump!(context, label);

                            await!(Compat::new(cancel.fuse())); // suspend process
                            // println!("select! resumption pid={}", process.pid);
                        },
                        Variant::Integer(ms) => {
                            let when = time::Duration::from_millis(ms as u64);
                            use tokio::prelude::FutureExt;

                            match await!(Compat::new(cancel).timeout(when)) {
                                Ok(()) =>  {
                                    // jump to success (start of recv loop)
                                    let label = ins.args[0].to_u32();
                                    op_jump!(context, label);
                                    // println!("select! resumption pid={}", process.pid);
                                }
                                Err(_) => {
                                    // timeout
                                    // println!("select! delay timeout {} pid={} ms={} m={:?}", ms, process.pid, context.expand_arg(&ins.args[1]), ins.args[1]);

                                    // remove channel
                                    context.timeout.take();

                                    () // continue to next instruction (timeout op)
                                }

                            }
                            // use futures::compat::Compat;
                            // use tokio_async_await::compat::backward;
                            // let future = backward::Compat::new(cancel).fuse();
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
                        _ => unreachable!()
                    }
                    process.process_incoming()?;
                }
                Opcode::RecvMark => {
                    process.local_data_mut().mailbox.mark();
                }
                Opcode::RecvSet => {
                    process.local_data_mut().mailbox.set();
                }
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [LValue::Literal(_a), LValue::Label(i)] = &ins.args[..] {
                        context.cp = Some(context.ip);
                        op_jump!(context, *i);

                        // let (mfa, _) = context.ip.lookup_func_info().unwrap();
                        // println!("pid={} action=call mfa={}", process.pid, mfa);
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::CallLast => {
                    //literal arity, label jmp, nwords
                    // store arity as live
                    if let [LValue::Literal(_a), LValue::Label(i), LValue::Literal(nwords)] =
                        &ins.args[..]
                    {
                        op_deallocate!(context, *nwords);

                        op_jump!(context, *i);

                        // let (mfa, _) = context.ip.lookup_func_info().unwrap();
                        // println!("pid={} action=call_last mfa={}", process.pid, mfa);
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                // proc pid=38 reds=1974 mod="erl_parse" offs=2986 ins=CallOnly args=[Literal(7), Integer(2838)]

                // proc pid=38 reds=1961 mod="erl_parse" offs=5604 ins=CallOnly args=[Literal(7), Integer(2620)]
                // pid=38 action=call mfa=erl_parse:yeccpars2/7
                // proc pid=38 reds=1960 mod="erl_parse" offs=2621 ins=CallOnly args=[Literal(7), Label(4019)]
                // pid=38 action=call mfa=erl_parse:yeccpars2_139/7
                Opcode::CallOnly => {
                    //literal arity, label jmp
                    // store arity as live
                    //
                    // TODO: has a second arg as Integer???
                    if let [LValue::Literal(_a), i] = &ins.args[..] {
                        op_jump!(context, i.to_u32());

                        // let (mfa, _) = context.ip.lookup_func_info().unwrap();
                        // println!("pid={} action=call_only mfa={}", process.pid, mfa);
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::CallExt => {
                    //literal arity, literal destination (module.imports index)
                    if let [LValue::Literal(arity), LValue::Literal(dest)] = &ins.args[..] {
                        // save pointer onto CP
                        context.cp = Some(context.ip);

                        op_call_ext!(self, context, &process, arity, dest, false); // don't return on call_ext
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::CallExtOnly => {
                    //literal arity, literal destination (module.imports index)
                    if let [LValue::Literal(arity), LValue::Literal(dest)] = &ins.args[..] {
                        op_call_ext!(self, context, &process, arity, dest, true);
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::CallExtLast => {
                    //literal arity, literal destination (module.imports index), literal deallocate
                    if let [LValue::Literal(arity), LValue::Literal(dest), LValue::Literal(nwords)] =
                        &ins.args[..]
                    {
                        op_deallocate!(context, *nwords);

                        op_call_ext!(self, context, &process, arity, dest, true);
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::Bif0 => {
                    // literal import, x reg
                    if let [LValue::Literal(dest), reg] = &ins.args[..] {
                        // TODO: precompute these lookups
                        let mfa = &module.imports[*dest as usize];
                        let val = bif::apply(self, &process, mfa, &[]).unwrap(); // bif0 can't fail
                        set_register!(context, reg, val);
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Bif1 => {
                    // literal import, arg, x reg
                    if let [fail, LValue::Literal(dest), arg1, reg] = &ins.args[..] {
                        let args = &[context.expand_arg(arg1)];
                        // TODO: precompute these lookups
                        let mfa = &module.imports[*dest as usize];
                        match bif::apply(self, &process, mfa, args) {
                            Ok(val) => set_register!(context, reg, val),
                            Err(exc) => cond_fail!(context, fail, exc),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Bif2 => {
                    // literal import, arg, arg, x reg
                    if let [fail, LValue::Literal(dest), arg1, arg2, reg] = &ins.args[..] {
                        let args = &[context.expand_arg(arg1), context.expand_arg(arg2)];
                        // TODO: precompute these lookups
                        let mfa = &module.imports[*dest as usize];
                        match bif::apply(self, &process, mfa, args) {
                            Ok(val) => set_register!(context, reg, val),
                            Err(exc) => cond_fail!(context, fail, exc),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Allocate => {
                    // stackneed, live
                    if let [LValue::Literal(stackneed), LValue::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*stackneed {
                            context.stack.push(Term::nil())
                        }
                        context.stack.push(Term::cp(&context.heap, context.cp));
                        context.cp = None;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::AllocateHeap => {
                    // TODO: this also zeroes the values, make it dynamically change the
                    // capacity/len of the Vec to bypass initing these.

                    // literal stackneed, literal heapneed, literal live
                    // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
                    // num of X regs. save cp on stack.
                    if let [LValue::Literal(stackneed), LValue::Literal(_heapneed), LValue::Literal(_live)] =
                        &ins.args[..]
                    {
                        context
                            .stack
                            .resize(context.stack.len() + *stackneed as usize, Term::nil());
                        // TODO: check heap for heapneed space!
                        context.stack.push(Term::cp(&context.heap, context.cp));
                        context.cp = None;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::AllocateZero => {
                    // literal stackneed, literal live
                    if let [LValue::Literal(need), LValue::Literal(_live)] = &ins.args[..] {
                        context
                            .stack
                            .resize(context.stack.len() + *need as usize, Term::nil());
                        context.stack.push(Term::cp(&context.heap, context.cp));
                        context.cp = None;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::AllocateHeapZero => {
                    // literal stackneed, literal heapneed, literal live
                    // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
                    // num of X regs. save cp on stack.
                    if let [LValue::Literal(stackneed), LValue::Literal(_heapneed), LValue::Literal(_live)] =
                        &ins.args[..]
                    {
                        context
                            .stack
                            .resize(context.stack.len() + *stackneed as usize, Term::nil());
                        // TODO: check heap for heapneed space!
                        context.stack.push(Term::cp(&context.heap, context.cp));
                        context.cp = None;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::TestHeap => {
                    // println!("TODO: TestHeap unimplemented!");
                    ()
                }
                Opcode::Init => {
                    debug_assert_eq!(ins.args.len(), 1);
                    set_register!(context, &ins.args[0], Term::nil())
                }
                Opcode::Deallocate => {
                    // literal nwords
                    if let [LValue::Literal(nwords)] = &ins.args[..] {
                        op_deallocate!(context, *nwords)
                    } else {
                        unreachable!()
                    }
                }
                Opcode::IsGe => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    } else {
                        // ok
                    }
                }
                Opcode::IsLt => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsEq => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.erl_partial_cmp(&v2) {
                        // ok
                    } else {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsNe => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.erl_partial_cmp(&v2) {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsEqExact => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if v1.eq(&v2) {
                        // ok
                    } else {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsNeExact => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = context.expand_arg(&ins.args[1]);
                    let v2 = context.expand_arg(&ins.args[2]);

                    if v1.eq(&v2) {
                        let fail = ins.args[0].to_u32();
                        op_jump!(context, fail);
                    } else {
                        // ok
                    }
                }
                Opcode::IsInteger => op_is_type!(context, ins.args, is_integer),
                Opcode::IsFloat => op_is_type!(context, ins.args, is_float),
                Opcode::IsNumber => op_is_type!(context, ins.args, is_number),
                Opcode::IsAtom => op_is_type!(context, ins.args, is_atom),
                Opcode::IsPid => op_is_type!(context, ins.args, is_pid),
                Opcode::IsReference => op_is_type!(context, ins.args, is_ref),
                Opcode::IsPort => op_is_type!(context, ins.args, is_port),
                Opcode::IsNil => op_is_type!(context, ins.args, is_nil),
                Opcode::IsBinary => op_is_type!(context, ins.args, is_binary),
                Opcode::IsBitstr => op_is_type!(context, ins.args, is_bitstring),
                Opcode::IsList => op_is_type!(context, ins.args, is_list),
                Opcode::IsNonemptyList => op_is_type!(context, ins.args, is_non_empty_list),
                Opcode::IsTuple => op_is_type!(context, ins.args, is_tuple),
                Opcode::IsFunction => op_is_type!(context, ins.args, is_function),
                Opcode::IsBoolean => op_is_type!(context, ins.args, is_boolean),
                Opcode::IsMap => op_is_type!(context, ins.args, is_map),
                Opcode::IsFunction2 => {
                    // TODO: needs to verify exports too
                    let arity = ins.args[2].to_u32();
                    let value = &context.expand_arg(&ins.args[1]);
                    if let Ok(closure) = value::Closure::try_from(value) {
                        if closure.mfa.2 == arity {
                            continue;
                        }
                    }
                    if let Ok(mfa) = module::MFA::try_from(value) {
                        if mfa.2 == arity {
                            continue;
                        }
                    }
                    let fail = ins.args[0].to_u32();
                    op_jump!(context, fail);
                }
                Opcode::TestArity => {
                    // check tuple arity
                    if let [LValue::Label(fail), arg, LValue::Literal(arity)] = &ins.args[..] {
                        if let Ok(t) = Tuple::try_from(&context.expand_arg(arg)) {
                            if t.len != *arity {
                                op_jump!(context, *fail);
                            }
                        } else {
                            panic!("Bad argument to {:?}", ins.op)
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::SelectVal => {
                    // arg, fail, dests
                    // loop over dests
                    if let [arg, LValue::Label(fail), LValue::ExtendedList(vec)] = &ins.args[..] {
                        let arg = match context.expand_arg(arg).into_lvalue() {
                            Some(val) => val,
                            None => {
                                op_jump!(context, *fail);
                                continue;
                            }
                        };
                        let mut i = 0;
                        loop {
                            // if key matches, jump to the following label
                            if vec[i] == arg {
                                let label = vec[i + 1].to_u32();
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
                }
                Opcode::SelectTupleArity => {
                    // tuple fail dests
                    if let [arg, LValue::Label(fail), LValue::ExtendedList(vec)] = &ins.args[..] {
                        if let Ok(tup) = Tuple::try_from(&context.expand_arg(arg)) {
                            let len = LValue::Literal(tup.len);
                            let mut i = 0;
                            loop {
                                // if key matches, jump to the following label
                                if vec[i] == len {
                                    let label = vec[i + 1].to_u32();
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
                        } else {
                            op_jump!(context, *fail);
                        }
                    }
                }
                Opcode::Jump => {
                    debug_assert_eq!(ins.args.len(), 1);
                    let label = ins.args[0].to_u32();
                    op_jump!(context, label)
                }
                Opcode::Move => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // arg1 can be either a value or a register
                    let val = context.expand_arg(&ins.args[0]);
                    set_register!(context, &ins.args[1], val)
                }
                Opcode::GetList => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // source, head, tail
                    if let Ok(value::Cons { head, tail }) =
                        context.expand_arg(&ins.args[0]).try_into()
                    {
                        set_register!(context, &ins.args[1], *head);
                        set_register!(context, &ins.args[2], *tail);
                    } else {
                        panic!("badarg to GetList")
                    }
                }
                Opcode::GetTupleElement => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // source, element, dest
                    let source = context.expand_arg(&ins.args[0]);
                    let n = ins.args[1].to_u32();
                    if let Ok(t) = Tuple::try_from(&source) {
                        set_register!(context, &ins.args[2], t[n as usize])
                    } else {
                        panic!("GetTupleElement: source is of wrong type")
                    }
                }
                Opcode::SetTupleElement => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // new_el tuple pos
                    let el = context.expand_arg(&ins.args[0]);
                    let tuple = context.expand_arg(&ins.args[1]);
                    let pos = ins.args[2].to_u32() as usize;
                    if let Ok(t) = tuple.try_into_mut() {
                        let tuple: &mut value::Tuple = t; // annoying, need type annotation
                        tuple[pos] = el;
                    } else {
                        panic!("GetTupleElement: source is of wrong type")
                    }
                }
                Opcode::PutList => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // put_list H T Dst::slot()
                    // Creates a cons cell with [H|T] and places the value into Dst.
                    let head = context.expand_arg(&ins.args[0]);
                    let tail = context.expand_arg(&ins.args[1]);
                    let cons = cons!(&context.heap, head, tail);
                    set_register!(context, &ins.args[2], cons)
                }
                Opcode::PutTuple => {
                    // put_tuple dest size
                    // followed by multiple put() ops (put val [potentially regX/Y])
                    unimplemented!("Stray PutTuple that wasn't rewritten by the loader!")

                    // Code compiled with OTP 22 and later uses put_tuple2 to to construct a tuple.
                    // PutTuple + Put is before OTP 22 and we should transform in loader to put_tuple2
                }
                Opcode::Put => unimplemented!("Stray Put that wasn't rewritten by the loader!"),
                Opcode::PutTuple2 => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // op: PutTuple2, args: [X(0), ExtendedList([Y(1), Y(0), X(0)])] }
                    if let LValue::ExtendedList(list) = &ins.args[1] {
                        let arity = list.len();

                        let tuple = value::tuple(&context.heap, arity as u32);
                        for i in 0..arity {
                            unsafe {
                                std::ptr::write(&mut tuple[i], context.expand_arg(&list[i]));
                            }
                        }
                        set_register!(context, &ins.args[0], Term::from(tuple))
                    }
                }
                Opcode::Badmatch => {
                    let value = context.expand_arg(&ins.args[0]);
                    println!("Badmatch: {}", value);
                    return Err(Exception::with_value(Reason::EXC_BADMATCH, value));
                }
                Opcode::IfEnd => {
                    // Raises the if_clause exception.
                    return Err(Exception::new(Reason::EXC_IF_CLAUSE));
                }
                Opcode::CaseEnd => {
                    // Raises the case_clause exception with the value of Arg0
                    let value = context.expand_arg(&ins.args[0]);
                    println!("err=case_clause val={}", value);
                    return Err(Exception::with_value(Reason::EXC_CASE_CLAUSE, value));
                }
                Opcode::Try => {
                    // TODO: try is identical to catch, and is remapped in the OTP loader
                    // y f
                    // create a catch context that wraps f - fail label, and stores to y - reg.
                    context.catches += 1;
                    let fail = ins.args[1].to_u32();
                    set_register!(
                        context,
                        &ins.args[0],
                        Term::catch(
                            &context.heap,
                            InstrPtr {
                                ptr: fail,
                                module: context.ip.module
                            }
                        )
                    );
                }
                Opcode::TryEnd => {
                    debug_assert_eq!(ins.args.len(), 1);
                    // y
                    context.catches -= 1;
                    set_register!(context, &ins.args[0], Term::nil()) // TODO: make_blank macro
                }
                Opcode::TryCase => {
                    // pops a catch context in y  Erases the label saved in the Arg0 slot. Noval in R0 indicate that something is caught. If so, R0 is set to R1, R1 — to R2, R2 — to R3.

                    // TODO: this initial part is identical to TryEnd
                    context.catches -= 1;
                    set_register!(context, &ins.args[0], Term::nil()); // TODO: make_blank macro

                    assert!(context.x[0].is_none());
                    // TODO: c_p->fvalue = NIL;
                    // TODO: make more efficient
                    context.x[0] = context.x[1];
                    context.x[1] = context.x[2];
                    context.x[2] = context.x[3];
                }
                Opcode::TryCaseEnd => {
                    // Raises a try_clause exception with the value read from Arg0.
                    let value = context.expand_arg(&ins.args[0]);
                    return Err(Exception::with_value(Reason::EXC_TRY_CLAUSE, value));
                }
                Opcode::Catch => {
                    // create a catch context that wraps f - fail label, and stores to y - reg.
                    context.catches += 1;
                    let fail = ins.args[1].to_u32();
                    set_register!(
                        context,
                        &ins.args[0],
                        Term::catch(
                            &context.heap,
                            InstrPtr {
                                ptr: fail,
                                module: context.ip.module
                            }
                        )
                    );
                }
                Opcode::CatchEnd => {
                    debug_assert_eq!(ins.args.len(), 1);
                    // y
                    // Pops a “catch” context. Erases the label saved in the Arg0 slot. Noval in R0
                    // indicates that something is caught. If R1 contains atom throw then R0 is set
                    // to R2. If R1 contains atom error than a stack trace is added to R2. R0 is
                    // set to {exit,R2}.
                    //
                    // difference fron try is, try will exhaust all the options then fail, whereas
                    // catch will keep going upwards.
                    //

                    // TODO: this initial part is identical to TryEnd
                    context.catches -= 1; // TODO: this is overflowing
                    set_register!(context, &ins.args[0], Term::nil()); // TODO: make_blank macro

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
                                tup2!(&context.heap, Term::atom(atom::EXIT), context.x[2]);
                        }
                    }
                }
                Opcode::Raise => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // Raises the exception. The instruction is garbled by backward compatibility. Arg0 is a stack trace
                    // and Arg1 is the value accompanying the exception. The reason of the raised exception is dug up
                    // from the stack trace
                    let trace = context.expand_arg(&ins.args[0]);
                    let value = context.expand_arg(&ins.args[1]);

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
                Opcode::Apply => {
                    //literal arity

                    if let [LValue::Literal(arity)] = &ins.args[..] {
                        context.cp = Some(context.ip);

                        op_fixed_apply!(self, context, &process, *arity, false)
                    // call this fixed_apply, used for ops (apply, apply_last).
                    // apply is the func that's equivalent to erlang:apply/3 (and instrs)
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::ApplyLast => {
                    //literal arity, nwords (dealloc)
                    if let [LValue::Literal(arity), LValue::Literal(nwords)] = &ins.args[..] {
                        op_deallocate!(context, *nwords);

                        op_fixed_apply!(self, context, &process, *arity, true)
                    } else {
                        unreachable!()
                    }
                    safepoint_and_reduce!(self, process, context.reds);
                }
                Opcode::GcBif1 => {
                    // fail label, live, bif, arg1, dest
                    if let LValue::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[context.expand_arg(&ins.args[3])];
                        let mfa = &module.imports[*i as usize];
                        match bif::apply(self, &process, mfa, args) {
                            Ok(val) => set_register!(context, &ins.args[4], val),
                            Err(exc) => cond_fail!(context, ins.args[0], exc),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsAdd => {
                    // Calculates the total of the number of bits in Src1 and the number of units in Src2. Stores the result to Dst.
                    // bs_add(Fail, Src1, Src2, Unit, Dst)
                    // dst = (src1 + src2) * unit

                    // TODO: trickier since we need to check both nums are positive and can fit
                    // into int/bigint

                    // optimize when one append is 0 and unit is 1, it's just a move
                    // bs_add Fail S1=i==0 S2 Unit=u==1 D => move S2 D

                    if let [LValue::Label(_fail), s1, s2, LValue::Literal(unit), dest] =
                        &ins.args[..]
                    {
                        // TODO use fail label
                        let s1 = context.expand_arg(s1).to_u32();
                        let s2 = context.expand_arg(s2).to_u32();

                        let res = Term::int(((s1 + s2) * (*unit)) as i32);
                        set_register!(context, dest, res)
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsInit2 => {
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

                    if let [LValue::Label(_fail), s1, LValue::Literal(_words), LValue::Literal(_live), _flags, dest] =
                        &ins.args[..]
                    {
                        // TODO: use a current_string ptr to be able to write to the Arc wrapped str
                        // alternatively, loop through the instrs until we hit a non bs_ instr.
                        // that way, no unsafe ptrs!
                        let size = s1.to_u32();
                        let mut binary = bitstring::Binary::with_capacity(size as usize);
                        binary.is_writable = false;
                        context.bs = binary.get_mut();
                        // TODO ^ ensure this pointer stays valid after heap alloc
                        set_register!(context, dest, Term::binary(&context.heap, binary));
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsPutInteger => {
                    // gen_put_integer(GenOpArg Fail,GenOpArg Size, GenOpArg Unit, GenOpArg Flags, GenOpArg Src)
                    // Size can be atom all
                    unimplemented!("bs_put_integer")
                }
                Opcode::BsPutBinary => {
                    if let [fail, size, LValue::Literal(unit), _flags, src] = &ins.args[..] {
                        // TODO: fail label
                        if *unit != 8 {
                            unimplemented!("bs_put_binary unit != 8");
                            //fail!(context, fail);
                        }

                        if let Ok(value) = bitstring::RcBinary::try_from(&context.expand_arg(src)) {
                            match size {
                                LValue::Atom(atom::ALL) => unsafe {
                                    (*context.bs).extend_from_slice(&value.data);
                                },
                                _ => unimplemented!("bs_put_binary size {:?}", size),
                            }
                        } else {
                            panic!("Bad argument to {:?}", ins.op)
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsPutFloat => {
                    // gen_put_float(GenOpArg Fail,GenOpArg Size, GenOpArg Unit, GenOpArg Flags, GenOpArg Src)
                    // Size can be atom all
                    if let [fail, size, LValue::Literal(unit), _flags, src] = &ins.args[..] {
                        // TODO: fail label
                        if *unit != 8 {
                            unimplemented!("bs_put_float unit != 8");
                            //fail!(context, fail);
                        }

                        if let Variant::Float(value::Float(f)) =
                            context.expand_arg(src).into_variant()
                        {
                            match size {
                                LValue::Atom(atom::ALL) => unsafe {
                                    let bytes: [u8; 8] = transmute(f);
                                    (*context.bs).extend_from_slice(&bytes);
                                },
                                _ => unimplemented!("bs_put_float size {:?}", size),
                            }
                        } else {
                            panic!("Bad argument to {:?}", ins.op)
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsPutString => {
                    // BsPutString uses the StrT strings table! needs to be patched in loader
                    if let LValue::Binary(str) = &ins.args[0] {
                        unsafe {
                            (*context.bs).extend_from_slice(&str.data);
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsStartMatch2 => {
                    debug_assert_eq!(ins.args.len(), 5);
                    // fail, src, live, slots?, dst
                    // check if src is a match context with space for n slots (matches) else
                    // allocate one. if we can't, jump to fail.
                    // ibsstartmatch2 rxy f I I d
                    // Checks that Arg0 is the matching context with enough slots for saved
                    // offsets. The number of slots needed is given by Arg3. If there not enough
                    // slots of if Arg0 is a regular binary then recreates the matching context and
                    // saves it to Arg4. If something does not work jumps to Arg1. Arg2 is Live.

                    // Uint slots;
                    // Uint live;
                    // Eterm header;

                    let cxt = context.expand_arg(&ins.args[1]);

                    if !cxt.is_pointer() {
                        fail!(context, ins.args[0]);
                    }

                    let header = cxt.get_boxed_header().unwrap();

                    // Reserve a slot for the start position.
                    let slots = ins.args[3].to_u32() + 1;
                    let live = ins.args[2].to_u32();

                    match header {
                        value::BOXED_MATCHSTATE => {
                            if let Ok(ms) = cxt.get_boxed_value_mut::<bitstring::MatchState>() {
                                // Uint actual_slots = HEADER_NUM_SLOTS(header);
                                let actual_slots = ms.saved_offsets.len();

                                // We're not compatible with contexts created by bs_start_match3.
                                assert!(actual_slots >= 2);

                                ms.saved_offsets[0] = ms.mb.offset;
                                // TODO: we don't need realloc since Vec handles it for us
                                // if (ERTS_UNLIKELY(actual_slots < slots)) {
                                //     ErlBinMatchState* expanded;
                                //     Uint live = $Live;
                                //     Uint wordsneeded = ERL_BIN_MATCHSTATE_SIZE(slots);
                                //     $GC_TEST_PRESERVE(wordsneeded, live, context);
                                //     ms = (ErlBinMatchState *) boxed_val(context);
                                //     expanded = (ErlBinMatchState *) HTOP;
                                //     *expanded = *ms;
                                //     *HTOP = HEADER_BIN_MATCHSTATE(slots);
                                //     HTOP += wordsneeded;
                                //     HEAP_SPACE_VERIFIED(0);
                                //     context = make_matchstate(expanded);
                                //     $REFRESH_GEN_DEST();
                                // }
                                set_register!(context, &ins.args[4], cxt);
                            }
                        }
                        value::BOXED_BINARY => {
                            // Uint wordsneeded = ERL_BIN_MATCHSTATE_SIZE(slots);
                            // $GC_TEST_PRESERVE(wordsneeded, live, context);

                            let result = bitstring::start_match_2(&context.heap, cxt, slots);

                            if let Some(res) = result {
                                set_register!(context, &ins.args[4], res)
                            } else {
                                fail!(context, ins.args[0]);
                            }
                        }
                        _ => {
                            fail!(context, ins.args[0]);
                        }
                    }
                }
                Opcode::BsStartMatch3 => {
                    debug_assert_eq!(ins.args.len(), 4);
                    // fail, src, live, dst

                    let cxt = context.expand_arg(&ins.args[1]);

                    if !cxt.is_pointer() {
                        fail!(context, ins.args[0]);
                    }

                    let header = cxt.get_boxed_header().unwrap();

                    // Reserve a slot for the start position.
                    let live = ins.args[2].to_u32();

                    match header {
                        value::BOXED_MATCHSTATE => {
                            if let Ok(ms) = cxt.get_boxed_value_mut::<bitstring::MatchState>() {
                                let actual_slots = ms.saved_offsets.len();
                                // We're not compatible with contexts created by bs_start_match2.
                                assert!(actual_slots == 0);
                                set_register!(context, &ins.args[3], cxt);
                            }
                        }
                        value::BOXED_BINARY => {
                            // Uint wordsneeded = ERL_BIN_MATCHSTATE_SIZE(slots);
                            // $GC_TEST_PRESERVE(wordsneeded, live, context);

                            let result = bitstring::start_match_3(&context.heap, cxt);

                            if let Some(res) = result {
                                set_register!(context, &ins.args[3], res)
                            } else {
                                fail!(context, ins.args[0]);
                            }
                        }
                        _ => {
                            fail!(context, ins.args[0]);
                        }
                    }
                }
                Opcode::BsGetPosition => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // cxt dst live
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[0])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        // TODO: unsafe cast
                        set_register!(context, &ins.args[1], Term::int(ms.mb.offset as i32));
                    } else {
                        unreachable!()
                    };
                }
                Opcode::BsSetPosition => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // cxt pos

                    if let Ok(ms) = context
                        .expand_arg(&ins.args[0])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let pos = context.expand_arg(&ins.args[1]).to_u32();
                        ms.mb.offset = pos as usize;
                    } else {
                        unreachable!()
                    };
                }
                Opcode::BsGetInteger2 => {
                    debug_assert_eq!(ins.args.len(), 7);
                    // bs_get_integer2 Fail=f Ms=xy Live=u Sz=sq Unit=u Flags=u Dst=d

                    let size = ins.args[3].to_u32() as usize;
                    let unit = ins.args[4].to_u32() as usize;
                    let flags = ins.args[5].to_u32();

                    let bits = size * unit;

                    // match bits {
                    //     8 => unimplemented!()
                    //     16 => unimplemented!()
                    //     32 => unimplemented!()
                    //     _ => unimplemented!() // slow fallback
                    // }

                    // let size = size * (flags as usize >> 3); TODO: this was just because flags
                    // & size were packed together on BEAM

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms.mb.get_integer(
                            &context.heap,
                            bits,
                            bitstring::Flag::from_bits(flags as u8).unwrap(),
                        );
                        if let Some(res) = res {
                            set_register!(context, &ins.args[6], res)
                        } else {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsGetFloat2 => {
                    debug_assert_eq!(ins.args.len(), 7);
                    // bs_get_float2 Fail=f Ms=xy Live=u Sz=sq Unit=u Flags=u Dst=d

                    let size = match ins.args[3] {
                        LValue::Integer(size) if size <= 64 => size as usize,
                        _ => {
                            fail!(context, ins.args[0]);
                        }
                    };

                    let flags = ins.args[5].to_u32();
                    // let size = size * (flags as usize >> 3); TODO: this was just because flags
                    // & size were packed together on BEAM

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms.mb.get_float(
                            &context.heap,
                            size as usize,
                            bitstring::Flag::from_bits(flags as u8).unwrap(),
                        );
                        if let Some(res) = res {
                            set_register!(context, &ins.args[6], res)
                        } else {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsGetBinary2 => {
                    debug_assert_eq!(ins.args.len(), 7);

                    let flags = ins.args[5].to_u32();
                    // let size = size * (flags as usize >> 3); TODO: this was just because flags
                    // & size were packed together on BEAM

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let flags = bitstring::Flag::from_bits(flags as u8).unwrap();
                        let heap = &context.heap;
                        let res = match ins.args[3] {
                            LValue::Integer(size) => ms.mb.get_binary(heap, size as usize, flags),
                            LValue::Atom(atom::ALL) => ms.mb.get_binary_all(heap, flags),
                            _ => unreachable!(),
                        };

                        if let Some(res) = res {
                            set_register!(context, &ins.args[6], res)
                        } else {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsSkipBits2 => {
                    debug_assert_eq!(ins.args.len(), 5);
                    // fail, ms, size, unit, flags

                    if let Ok(ms) =
                        bitstring::MatchState::try_from(&context.expand_arg(&ins.args[1]))
                    {
                        let mb = &ms.mb;

                        let size = ins.args[2].to_u32();
                        let unit = ins.args[3].to_u32();

                        let new_offset = mb.offset + (size * unit) as usize;

                        if new_offset <= mb.size {
                            fail!(context, ins.args[0]);
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsTestTail2 => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // fail, ms, bits
                    // TODO: beam specializes bits 0
                    // Checks that the matching context Arg1 has exactly Arg2 unmatched bits. Jumps
                    // to the label Arg0 if it is not so.
                    // if size 0 == Jumps to the label in Arg0 if the matching context Arg1 still have unmatched bits.

                    let offset = ins.args[2].to_u32() as usize;

                    if let Ok(ms) =
                        bitstring::MatchState::try_from(&context.expand_arg(&ins.args[1]))
                    {
                        let mb = &ms.mb;

                        if mb.remaining() != offset {
                            fail!(context, ins.args[0]);
                        }
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsSave2 => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // cxt slot
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[0])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let slot = match ins.args[1] {
                            LValue::Integer(i) => i as usize,
                            LValue::Atom(atom::START) => 0,
                            _ => unreachable!(),
                        };
                        ms.saved_offsets[slot] = ms.mb.offset;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsRestore2 => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // cxt slot
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[0])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let slot = match ins.args[1] {
                            LValue::Integer(i) => i as usize,
                            LValue::Atom(atom::START) => 0,
                            _ => unreachable!(),
                        };
                        ms.mb.offset = ms.saved_offsets[slot];
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsContextToBinary => {
                    debug_assert_eq!(ins.args.len(), 1);
                    // Converts the matching context to a (sub)binary using almost the same code as
                    // i bs get binary all reuse rx f I.

                    // cxt slot
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[0])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let offs = ms.saved_offsets[0];
                        let size = ms.mb.size - offs;
                        // TODO; original calculated the hole size and overwrote MatchState mem in
                        // place.
                        let res = Term::subbinary(
                            &context.heap,
                            bitstring::SubBinary::new(ms.mb.original.clone(), size, offs, false),
                        );
                        set_register!(context, &ins.args[0], res);
                    } else {
                        // next0
                        unreachable!()
                    }
                }
                Opcode::BsTestUnit => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // fail cxt unit
                    // Checks that the size of the remainder of the matching context is divisible
                    // by unit, else jump to fail

                    if let Ok(ms) =
                        bitstring::MatchState::try_from(&context.expand_arg(&ins.args[1]))
                    {
                        let mb = &ms.mb;

                        let unit = ins.args[2].to_u32() as usize;

                        if mb.remaining() % unit != 0 {
                            fail!(context, ins.args[0]);
                        }
                    } else {
                        unreachable!()
                    }
                    unimplemented!("bs_test_unit") // TODO
                }
                Opcode::BsMatchString => {
                    debug_assert_eq!(ins.args.len(), 4);
                    // fail cxt bits str

                    // byte* bytes = (byte *) $Ptr;
                    // Uint bits = $Bits;
                    // ErlBinMatchBuffer* mb;
                    // Uint offs;

                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let mb = &mut ms.mb;

                        let bits = ins.args[2].to_u32() as usize;
                        let string = match &ins.args[3] {
                            LValue::Str(string) => string,
                            _ => unreachable!(),
                        };

                        if mb.remaining() < bits {
                            fail!(context, ins.args[0]);
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
                                fail!(context, ins.args[0]);
                            }
                        }
                        mb.offset += bits;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::BsInitWritable => {
                    debug_assert_eq!(ins.args.len(), 0);
                    context.x[0] = bitstring::init_writable(&process, context.x[0]);
                }
                // BsGet and BsSkip should be implemented over an Iterator inside a match context (.skip/take)
                // maybe we can even use nom for this
                Opcode::BsAppend => {
                    debug_assert_eq!(ins.args.len(), 8);
                    // append and init also sets the string as current (state.current_binary) [seems to be used to copy string literals too]

                    // bs_append Fail Size Extra Live Unit Bin Flags Dst => \
                    //   move Bin x | i_bs_append Fail Extra Live Unit Size Dst

                    let size = context.expand_arg(&ins.args[1]);
                    let extra_words = ins.args[2].to_u32() as usize;
                    let unit = ins.args[2].to_u32() as usize;
                    let src = context.expand_arg(&ins.args[4]);

                    let res = bitstring::append(&process, src, size, extra_words, unit);

                    if let Some(res) = res {
                        set_register!(context, &ins.args[5], res)
                    } else {
                        // TODO: execute fail only if non zero, else raise
                        /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
                        fail!(context, ins.args[0]);
                    }
                }
                Opcode::BsPrivateAppend => {
                    debug_assert_eq!(ins.args.len(), 6);
                    // bs_private_append Fail Size Unit Bin Flags Dst

                    let size = context.expand_arg(&ins.args[1]);
                    let unit = ins.args[2].to_u32() as usize;
                    let src = context.expand_arg(&ins.args[3]);

                    let res = bitstring::private_append(&process, src, size, unit);

                    if let Some(res) = res {
                        set_register!(context, &ins.args[5], res)
                    } else {
                        /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
                        fail!(context, ins.args[0]);
                    }
                    unimplemented!("bs_private_append") // TODO
                }
                Opcode::BsInitBits => {
                    debug_assert_eq!(ins.args.len(), 6);
                    // TODO: RcBinary has to be set to is_writable = false
                    unimplemented!("bs_init_bits") // TODO
                }
                Opcode::BsGetUtf8 => {
                    debug_assert_eq!(ins.args.len(), 5);
                    // fail ms u u dest

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms.mb.get_utf8();
                        if let Some(res) = res {
                            set_register!(context, &ins.args[4], res)
                        } else {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsGetUtf16 => {
                    debug_assert_eq!(ins.args.len(), 5);
                    // fail ms u flags dest

                    let flags = ins.args[5].to_u32();

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms
                            .mb
                            .get_utf16(bitstring::Flag::from_bits(flags as u8).unwrap());
                        if let Some(res) = res {
                            set_register!(context, &ins.args[4], res)
                        } else {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsSkipUtf8 => {
                    debug_assert_eq!(ins.args.len(), 4);
                    // fail ms u u

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms.mb.get_utf8();
                        if res.is_none() {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::BsSkipUtf16 => {
                    debug_assert_eq!(ins.args.len(), 4);
                    // fail ms u flags

                    let flags = ins.args[5].to_u32();

                    // TODO: this cast can fail
                    if let Ok(ms) = context
                        .expand_arg(&ins.args[1])
                        .get_boxed_value_mut::<bitstring::MatchState>()
                    {
                        let res = ms
                            .mb
                            .get_utf16(bitstring::Flag::from_bits(flags as u8).unwrap());
                        if res.is_none() {
                            fail!(context, ins.args[0]);
                        }
                    };
                }
                Opcode::Fclearerror => {
                    debug_assert_eq!(ins.args.len(), 0);
                    // TODO: BEAM checks for unhandled errors
                    context.f[0] = 0.0;
                }
                Opcode::Fcheckerror => {
                    debug_assert_eq!(ins.args.len(), 1);
                    // I think it always checks register fr0
                    if !context.f[0].is_finite() {
                        return Err(Exception::new(Reason::EXC_BADARITH));
                    }
                }
                Opcode::Fmove => {
                    // src, dest

                    // basically a normal move, except if we're moving out of floats we make a Val
                    // otherwise we keep as a float
                    let f = expand_float!(context, &ins.args[0]);

                    match &ins.args[1] {
                        &LValue::X(reg) => {
                            context.x[reg as usize] = Term::from(f);
                        }
                        &LValue::Y(reg) => {
                            let len = context.stack.len();
                            context.stack[len - (reg + 2) as usize] = Term::from(f);
                        }
                        &LValue::FloatReg(reg) => {
                            context.f[reg as usize] = f;
                        }
                        _reg => unimplemented!(),
                    }
                }
                Opcode::Fconv => {
                    // reg (x), dest (float reg)
                    let val: f64 = match context.expand_arg(&ins.args[0]).into_number() {
                        Ok(value::Num::Float(f)) => f,
                        Ok(value::Num::Integer(i)) => f64::from(i),
                        Ok(_) => unimplemented!(),
                        // TODO: bignum if it fits into float
                        Err(_) => return Err(Exception::new(Reason::EXC_BADARITH)),
                    };

                    if let LValue::FloatReg(dest) = ins.args[1] {
                        context.f[dest as usize] = val;
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Fadd => op_float!(context, &ins.args[..], +),
                Opcode::Fsub => op_float!(context, &ins.args[..], -),
                Opcode::Fmul => op_float!(context, &ins.args[..], *),
                Opcode::Fdiv => op_float!(context, &ins.args[..], /),
                Opcode::Fnegate => {
                    debug_assert_eq!(ins.args.len(), 2);
                    if let [LValue::FloatReg(a), LValue::FloatReg(dest)] = ins.args[..] {
                        context.f[dest as usize] = -context.f[a as usize];
                    } else {
                        unreachable!()
                    }
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let LValue::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[
                            context.expand_arg(&ins.args[3]),
                            context.expand_arg(&ins.args[4]),
                        ];
                        let mfa = &module.imports[*i as usize];
                        let val = bif::apply(self, &process, mfa, &args[..]).unwrap(); // TODO: handle fail

                        // TODO: consume fail label if not 0, else return error

                        set_register!(context, &ins.args[5], val)
                    } else {
                        unreachable!()
                    }
                }
                Opcode::GcBif3 => {
                    // fail label, live, bif, arg1, arg2, arg3, dest
                    if let LValue::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[
                            context.expand_arg(&ins.args[3]),
                            context.expand_arg(&ins.args[4]),
                            context.expand_arg(&ins.args[5]),
                        ];
                        let mfa = &module.imports[*i as usize];
                        let val = bif::apply(self, &process, mfa, &args[..]).unwrap(); // TODO: handle fail

                        // TODO: consume fail label if not 0, else return error

                        set_register!(context, &ins.args[6], val)
                    } else {
                        unreachable!()
                    }
                }
                Opcode::Trim => {
                    // trim N, _remain
                    // drop N words from stack, (but keeping the cp). Second arg unused?
                    let nwords = ins.args[0].to_u32();
                    let cp = context.stack.pop().unwrap();
                    context
                        .stack
                        .truncate(context.stack.len() - nwords as usize);
                    context.stack.push(cp);
                }
                Opcode::MakeFun2 => {
                    // literal n -> points to lambda
                    // nfree means capture N x-registers into the closure
                    let i = ins.args[0].to_u32();
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
                Opcode::CallFun => {
                    // literal arity
                    let arity = ins.args[0].to_u32();
                    let value = context.x[arity as usize];
                    context.cp = Some(context.ip);
                    // TODO: this clone is bad but the borrow checker complains (but doesn't on GetTl/GetHd)
                    if let Ok(closure) = value::Closure::try_from(&value) {
                        op_call_fun!(self, context, closure, arity)
                    } else if let Ok(mfa) = module::MFA::try_from(&value) {
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
                Opcode::GetHd => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // source head
                    if let Ok(value::Cons { head, .. }) =
                        Cons::try_from(&context.expand_arg(&ins.args[0]))
                    {
                        set_register!(context, &ins.args[1], *head);
                    } else {
                        unreachable!()
                    }
                }
                Opcode::GetTl => {
                    debug_assert_eq!(ins.args.len(), 2);
                    // source head
                    if let Ok(value::Cons { tail, .. }) =
                        context.expand_arg(&ins.args[0]).try_into()
                    {
                        set_register!(context, &ins.args[1], *tail);
                    } else {
                        unreachable!()
                    }
                }
                Opcode::PutMapAssoc => {
                    debug_assert_eq!(ins.args.len(), 5);
                    // F Map Dst Live Rest=* (atom with module name??)
                    if let [LValue::Label(fail), map, dest, live, LValue::ExtendedList(list)] =
                        &ins.args[..]
                    {
                        let mut map = match context.expand_arg(map).try_into() {
                            Ok(value::Map(map)) => map.clone(),
                            _ => unreachable!(),
                        };

                        let mut iter = list.chunks_exact(2);
                        while let Some([key, value]) = iter.next() {
                            // TODO: optimize by having the ExtendedList store Term instead of LValue
                            map = map.plus(context.expand_arg(key), context.expand_arg(value))
                        }
                        set_register!(context, dest, Term::map(&context.heap, map))
                    }
                }
                Opcode::HasMapFields => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // fail src N
                    // TODO: make a macro or make the try_into() return an Exception so we can use ?
                    if let [LValue::Label(fail), map, LValue::ExtendedList(list)] = &ins.args[..] {
                        let map = match context.expand_arg(map).try_into() {
                            Ok(value::Map(map)) => map.clone(),
                            _ => unreachable!(),
                        };

                        // N is a list of the type [key => dest_reg], if any of these fields don't
                        // exist, jump to fail label.
                        let mut iter = list.chunks_exact(2);
                        while let Some([key, _dest]) = iter.next() {
                            if map.find(&context.expand_arg(key)).is_some() {
                                // ok
                            } else {
                                op_jump!(context, *fail);
                                break;
                            }
                        }
                    }
                }
                Opcode::GetMapElements => {
                    debug_assert_eq!(ins.args.len(), 3);
                    // fail src N
                    // TODO: make a macro or make the try_into() return an Exception so we can use ?
                    if let [LValue::Label(fail), map, LValue::ExtendedList(list)] = &ins.args[..] {
                        let map = match context.expand_arg(map).try_into() {
                            Ok(value::Map(map)) => map.clone(),
                            _ => unreachable!(),
                        };

                        // N is a list of the type [key => dest_reg], if any of these fields don't
                        // exist, jump to fail label.
                        let mut iter = list.chunks_exact(2);
                        while let Some([key, dest]) = iter.next() {
                            if let Some(&val) = map.find(&context.expand_arg(key)) {
                                set_register!(context, dest, val)
                            } else {
                                op_jump!(context, *fail);
                                break; // TODO: original impl loops over everything
                            }
                        }
                    }
                }
                Opcode::PutMapExact => {
                    // TODO: in the future, put_map_exact is an optimization for flatmaps (tuple
                    // maps), where the keys in the tuple stay the same, since you can memcpy the
                    // key tuple (and other optimizations)
                    debug_assert_eq!(ins.args.len(), 5);
                    // F Map Dst Live Rest=* (atom with module name??)
                    if let [LValue::Label(fail), map, dest, live, LValue::ExtendedList(list)] =
                        &ins.args[..]
                    {
                        let mut map = match context.expand_arg(map).try_into() {
                            Ok(value::Map(map)) => map.clone(),
                            _ => unreachable!(),
                        };

                        let mut iter = list.chunks_exact(2);
                        while let Some([key, value]) = iter.next() {
                            // TODO: optimize by having the ExtendedList store Term instead of LValue
                            map = map.plus(context.expand_arg(key), context.expand_arg(value))
                        }
                        set_register!(context, dest, Term::map(&context.heap, map))
                    }
                }
                Opcode::IsTaggedTuple => {
                    debug_assert_eq!(ins.args.len(), 4);
                    let reg = context.expand_arg(&ins.args[1]);
                    let n = ins.args[2].to_u32();
                    let atom = context.expand_arg(&ins.args[3]);
                    if let Ok(tuple) = Tuple::try_from(&reg) {
                        if tuple.len == 0 || tuple.len != n || !tuple[0].eq(&atom) {
                            fail!(context, ins.args[0]);
                        } else {
                            // ok
                        }
                    } else {
                        fail!(context, ins.args[0]);
                    }
                }
                Opcode::BuildStacktrace => {
                    context.x[0] = exception::build_stacktrace(&process, context.x[0]);
                }
                Opcode::RawRaise => {
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
                opcode => unimplemented!("Unimplemented opcode {:?}: {:?}", opcode, ins),
            }
        }

        // Terminate once the main process has finished execution.
        if process.is_main() {
            self.terminate();
        }

        Ok(process::State::Done)
        }
    }

    pub fn elapsed_time(&self) -> time::Duration {
        self.start_time.elapsed()
    }
}
