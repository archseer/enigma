use crate::atom;
use crate::bif;
use crate::module;
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::opcodes::Opcode;
use crate::pool::{Job, JoinGuard as PoolJoinGuard, Pool, Worker};
use crate::process::{self, ExecutionContext, RcProcess};
use crate::process_table::ProcessTable;
use crate::value::Value;
use std::panic;
use std::sync::Arc;
use std::sync::Mutex;

/// A reference counted State.
pub type RcState = Arc<State>;
/// Reference counted ModuleRegistry.

pub struct State {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,
    /// Use priorities later on
    pub process_pool: Pool<RcProcess>,
}

#[derive(Clone)]
pub struct Machine {
    pub state: RcState,

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    // export table
    // module table
    modules: RcModuleRegistry,
}

macro_rules! set_register {
    ($context:expr, $register:expr, $value:expr) => {{
        match $register {
            Value::X(reg) => {
                // TODO: remove these clones by using some form of mem::swap/replace
                $context.x[*reg] = $value.clone();
            }
            Value::Y(reg) => {
                let len = $context.stack.len();
                $context.stack[len - (*reg + 2)] = $value.clone();
            }
            reg => panic!("Unhandled register type! {:?}", reg),
        }
    }};
}

impl Machine {
    pub fn new() -> Machine {
        let primary_threads = 8;
        let process_pool = Pool::new(primary_threads, Some("primary".to_string()));

        let state = State {
            process_table: Mutex::new(ProcessTable::new()),
            process_pool,
        };

        Machine {
            state: Arc::new(state),
            modules: ModuleRegistry::with_rc(),
        }
    }

    /// Starts the VM
    ///
    /// This method will block the calling thread until it returns.
    ///
    /// This method returns true if the VM terminated successfully, false
    /// otherwise.
    pub fn start(&self, file: &str) {
        //self.configure_rayon();

        let primary_guard = self.start_primary_threads();

        self.start_main_process(file);

        // Joining the pools only fails in case of a panic. In this case we
        // don't want to re-panic as this clutters the error output.
        if primary_guard.join().is_err() {
            println!("Primary guard error!")
            //self.set_exit_status(1);
        }
    }

    fn terminate(&self) {
        self.state.process_pool.terminate();
    }

    fn start_primary_threads(&self) -> PoolJoinGuard<()> {
        let machine = self.clone();
        let pool = &self.state.process_pool;

        pool.run(move |worker, process| machine.run_with_error_handling(worker, &process))
    }

    /// Starts the main process
    pub fn start_main_process(&self, path: &str) {
        let process = {
            let module = module::load_module(&self.modules, path).unwrap();

            process::allocate(&self.state, module).unwrap()
        };

        let process = Job::normal(process);
        self.state.process_pool.schedule(process);
    }

    #[inline]
    fn expand_arg<'a>(&'a self, context: &'a ExecutionContext, arg: &'a Value) -> &'a Value {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            Value::ExtendedLiteral(i) => unsafe { (*context.module).literals.get(*i).unwrap() },
            Value::X(i) => &context.x[*i],
            Value::Y(i) => &context.stack[context.stack.len() - (*i + 2)],
            value => value,
        }
    }

    /// Executes a single process, terminating in the event of an error.
    pub fn run_with_error_handling(&self, worker: &mut Worker, process: &RcProcess) {
        // We are using AssertUnwindSafe here so we can pass a &mut Worker to
        // run()/panic(). This might be risky if values captured are not unwind
        // safe, so take care when capturing new variables.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            if let Err(message) = self.run(process) {
                //self.panic(worker, process, &message);
                panic!(message);
            }
        }));

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
        }
    }

    pub fn run(&self, process: &RcProcess) -> Result<(), String> {
        let context = process.context_mut();

        // temp
        let fun = atom::i_from_str("fib");
        context.x[0] = Value::Integer(23);
        // end temp
        unsafe {
            context.ip = *(*context.module).funs.get(&(fun, 1)).unwrap();
        }

        loop {
            let ins = unsafe { &(*context.module).instructions[context.ip] };
            context.ip = context.ip + 1;
            match &ins.op {
                Opcode::FuncInfo => {}//println!("Running a function..."),
                Opcode::Move => {
                    // arg1 can be either a value or a register
                    let val = self.expand_arg(&context, &ins.args[0]);
                    set_register!(context, &ins.args[1], val)
                }
                Opcode::Return => {
                    if context.cp == -1 {
                        println!("Process exited with normal, x0: {:?}", context.x[0]);
                        break;
                    }
                    context.ip = context.cp as usize;
                    context.cp = -1;
                }
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i)] = &ins.args[..] {
                        context.cp = context.ip as isize;
                        context.ip = *i - 2;
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::AllocateZero => {
                    // literal stackneed, literal live
                    if let [Value::Literal(need), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*need {
                            context.stack.push(Value::Nil())
                        }
                        context.stack.push(Value::CP(context.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::Deallocate => {
                    // literal nwords
                    if let [Value::Literal(nwords)] = &ins.args[..] {
                        let cp = context.stack.pop().unwrap();
                        context.stack.truncate(context.stack.len() - nwords);
                        if let Value::CP(cp) = cp {
                            context.cp = cp;
                        } else {
                            panic!("Bad CP value! {:?}", cp)
                        }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::IsLt => {
                    // Checks relation, that arg1 IS LESS than arg2, jump to arg0 otherwise.
                    // Structure: is_lt(on_false:CP, a:src, b:src)
                    // assert_arity(gen_op::OPCODE_IS_LT, 3);
                    // shared_equality_opcode(vm, ctx, curr_p, true, Ordering::Less, false)
                    assert_eq!(ins.args.len(), 3);

                    let l = self.expand_arg(&context, &ins.args[0]).to_usize();
                    let fail = unsafe { (*context.module).labels.get(&(l)).unwrap() };

                    let v1 = self.expand_arg(&context, &ins.args[1]);
                    let v2 = self.expand_arg(&context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        context.ip = *fail;
                    }
                }
                Opcode::IsEq => {
                    assert_eq!(ins.args.len(), 3);

                    let l = self.expand_arg(&context, &ins.args[0]).to_usize();
                    let fail = unsafe { (*context.module).labels.get(&(l)).unwrap() };

                    let v1 = self.expand_arg(&context, &ins.args[1]);
                    let v2 = self.expand_arg(&context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        context.ip = *fail;
                    }
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        let args = vec![
                            self.expand_arg(&context, &ins.args[3]),
                            self.expand_arg(&context, &ins.args[4]),
                        ];
                        let val = unsafe { bif::apply((*context.module).imports.get(*i).unwrap(), args) };

                        set_register!(context, &ins.args[5], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }

        // Terminate once the main process has finished execution.
        if process.is_main() {
            self.terminate();
        }

        Ok(())
    }
}
