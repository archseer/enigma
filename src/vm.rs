use crate::bif;
use crate::module::Module;
use crate::opcodes::Opcode;
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Machine {
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
    cp: isize, // TODO: ?!
    live: usize,
}

impl Machine {
    pub fn new() -> Machine {
        unsafe {
            let mut vm = Machine {
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

    pub fn register_module(&mut self, module: Module) {
        // TODO: use a module atom name
        self.modules.insert(0, module);
    }

    // value is an atom
    pub fn run(&mut self, module: Module, fun: usize) {
        let local = module.atoms.get(&fun).unwrap();
        println!("run: {:?}, fun:{:?}, local: {:?}", module.funs, fun, local);
        self.ip = module.funs.get(&(1, 1)).unwrap().clone();
        self.x[0] = Value::Integer(23);
        // TODO: modify imports to get *local working

        loop {
            let ref ins = module.instructions[self.ip];
            self.ip = self.ip + 1;
            match &ins.op {
                Opcode::FuncInfo => {}//println!("Running a function..."),
                Opcode::Move => {
                    // arg1 can be either a value or a register
                    let val = self.load_arg(&module, &ins.args[0]);
                    // TODO: remove these clones by using some form of mem::swap/replace
                    match &ins.args[1] {
                        Value::X(reg) => {
                            self.x[*reg as usize] = val.clone();
                        }
                        Value::Y(reg) => {
                            let len = self.stack.len();
                            self.stack[len - (*reg as usize + 2)] = val.clone();
                        }
                        reg => panic!("Unhandled register type! {:?}", reg),
                    }
                }
                Opcode::Return => {
                    if self.cp == -1 {
                        println!("Process exited with normal, x0: {:?}", self.x[0]);
                        println!("x: {:?}", self.x);
                        println!("y: {:?}", self.stack);
                        break;
                    }
                    self.ip = self.cp as usize;
                    self.cp = -1;
                }
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(a), Value::Label(i)] = &ins.args[..] {
                        self.cp = self.ip as isize;
                        self.ip = *i as usize - 2;
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
                        self.stack.truncate(self.stack.len() - *nwords as usize);
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

                    let l = self.load_arg(&module, &ins.args[0]).to_usize();
                    let fail = module.labels.get(&(l)).unwrap();

                    let v1 = self.load_arg(&module, &ins.args[1]);
                    let v2 = self.load_arg(&module, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(&v2) {
                        // ok
                    } else {
                        self.ip = *fail;
                    }
                }
                Opcode::IsEq => {
                    assert_eq!(ins.args.len(), 3);

                    let l = self.load_arg(&module, &ins.args[0]).to_usize();
                    let fail = module.labels.get(&(l)).unwrap();

                    let v1 = self.load_arg(&module, &ins.args[1]);
                    let v2 = self.load_arg(&module, &ins.args[2]);

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
                            self.load_arg(&module, &ins.args[3]),
                            self.load_arg(&module, &ins.args[4]),
                        ];
                        let val = bif::apply(module.imports.get(*i as usize).unwrap(), args);

                        // TODO: dedup in a func
                        match &ins.args[5] {
                            Value::X(reg) => {
                                self.x[*reg as usize] = val;
                            }
                            Value::Y(reg) => {
                                let len = self.stack.len();
                                self.stack[len - (*reg as usize + 2)] = val;
                            }
                            reg => panic!("Unhandled register type! {:?}", reg),
                        }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }
    }

    #[inline]
    fn load_arg<'a>(&'a self, module: &'a Module, arg: &'a Value) -> &'a Value {
        match arg {
            Value::ExtendedLiteral(i) => module.literals.get(*i).unwrap(),
            Value::X(i) => &self.x[*i as usize],
            Value::Y(i) => &self.stack[self.stack.len() - (*i as usize + 2)],
            value => value,
        }
    }
}
