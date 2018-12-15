use crate::bif;
use crate::module::{self, Module};
use crate::opcodes::Opcode;
use crate::pool::{Job, Pool};
use crate::process::{self, RcProcess};
use crate::process_table::ProcessTable;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Machine {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,
    /// Use priorities later on
    pub process_pool: Pool<RcProcess>,

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    // export table
    // module table
    modules: HashMap<usize, Module>,
    // registers
    x: [Value; 16],
    stack: Vec<Value>,
    // program pointer/reference?
    ip: usize,
    // continuation pointer
    cp: isize, // TODO: ?!, isize is lossy here
    live: usize,
}

macro_rules! set_register {
    ($vm:expr, $register:expr, $value:expr) => {{
        match $register {
            Value::X(reg) => {
                // TODO: remove these clones by using some form of mem::swap/replace
                $vm.x[*reg] = $value.clone();
            }
            Value::Y(reg) => {
                let len = $vm.stack.len();
                $vm.stack[len - (*reg + 2)] = $value.clone();
            }
            reg => panic!("Unhandled register type! {:?}", reg),
        }
    }};
}

impl Machine {
    pub fn new() -> Machine {
        let primary_threads = 8;
        let process_pool = Pool::new(primary_threads, Some("primary".to_string()));

        unsafe {
            let mut vm = Machine {
                process_table: Mutex::new(ProcessTable::new()),
                process_pool,
                modules: HashMap::new(),
                x: std::mem::uninitialized(), //[Value::Nil(); 16],
                stack: Vec::new(),
                ip: 0,
                cp: -1,
                live: 0,
            };
            for (_i, el) in vm.x.iter_mut().enumerate() {
                // Overwrite `element` without running the destructor of the old value.
                // Since Value does not implement Copy, it is moved.
                std::ptr::write(el, Value::Nil());
            }
            vm
        }
    }

    fn terminate(&self) {
        self.process_pool.terminate();
    }

    /// Starts the main process
    pub fn start_main_process(&self, file: &str) {
        let process = {
            let module = module::load_file(&self, file).unwrap();

            process::allocate(&self, &module).unwrap()
        };

        let process = Job::normal(process);
        self.process_pool.schedule(process);
    }

    pub fn register_module(&mut self, module: Module) {
        // TODO: use a module atom name
        self.modules.insert(0, module);
    }

    #[inline]
    fn expand_arg<'a>(&'a self, module: &'a Module, arg: &'a Value) -> &'a Value {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            Value::ExtendedLiteral(i) => module.literals.get(*i).unwrap(),
            Value::X(i) => &self.x[*i],
            Value::Y(i) => &self.stack[self.stack.len() - (*i + 2)],
            value => value,
        }
    }

    pub fn run(&mut self, module: Module, fun: usize) {
        self.ip = *module.funs.get(&(fun, 1)).unwrap();
        self.x[0] = Value::Integer(23);

        loop {
            let ref ins = module.instructions[self.ip];
            self.ip = self.ip + 1;
            match &ins.op {
                Opcode::FuncInfo => {}//println!("Running a function..."),
                Opcode::Move => {
                    // arg1 can be either a value or a register
                    let val = self.expand_arg(&module, &ins.args[0]);
                    set_register!(self, &ins.args[1], val)
                }
                Opcode::Return => {
                    if self.cp == -1 {
                        println!("Process exited with normal, x0: {:?}", self.x[0]);
                        break;
                    }
                    self.ip = self.cp as usize;
                    self.cp = -1;
                }
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i)] = &ins.args[..] {
                        self.cp = self.ip as isize;
                        self.ip = *i - 2;
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::AllocateZero => {
                    // literal stackneed, literal live
                    if let [Value::Literal(need), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*need {
                            self.stack.push(Value::Nil())
                        }
                        self.stack.push(Value::CP(self.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::Deallocate => {
                    // literal nwords
                    if let [Value::Literal(nwords)] = &ins.args[..] {
                        let cp = self.stack.pop().unwrap();
                        self.stack.truncate(self.stack.len() - *nwords);
                        if let Value::CP(cp) = cp {
                            self.cp = cp;
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

                    let l = self.expand_arg(&module, &ins.args[0]).to_usize();
                    let fail = module.labels.get(&(l)).unwrap();

                    let v1 = self.expand_arg(&module, &ins.args[1]);
                    let v2 = self.expand_arg(&module, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        self.ip = *fail;
                    }
                }
                Opcode::IsEq => {
                    assert_eq!(ins.args.len(), 3);

                    let l = self.expand_arg(&module, &ins.args[0]).to_usize();
                    let fail = module.labels.get(&(l)).unwrap();

                    let v1 = self.expand_arg(&module, &ins.args[1]);
                    let v2 = self.expand_arg(&module, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        self.ip = *fail;
                    }
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        let args = vec![
                            self.expand_arg(&module, &ins.args[3]),
                            self.expand_arg(&module, &ins.args[4]),
                        ];
                        let val = bif::apply(module.imports.get(*i).unwrap(), args);

                        set_register!(self, &ins.args[5], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }
    }
}
