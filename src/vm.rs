use crate::atom;
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
    y: [Value; 16],
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
                y: std::mem::uninitialized(), //[Value::Nil(); 16],
                ip: 0,
                cp: -1,
                live: 0,
            };
            for (_i, el) in vm.x.iter_mut().enumerate() {
                // Overwrite `element` without running the destructor of the old value.
                // Since Value does not implement Copy, it is moved.
                std::ptr::write(el, Value::Nil());
            }
            for (_i, el) in vm.y.iter_mut().enumerate() {
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
        println!("two: {:?}, fun:{:?}, local: {:?}", module.funs, fun, local);
        self.ip = module.funs.get(&(3, 0)).unwrap().clone();
        // TODO: modify imports to get *local working

        loop {
            let ref ins = module.instructions[self.ip];
            println!("ip: {:?} ins {:?}", self.ip, ins);
            self.ip = self.ip + 1;
            match &ins.op {
                Opcode::FuncInfo => println!("Running a function..."),
                Opcode::Move => {
                    println!("move: {:?}", ins.args);
                    // arg1 can be either a value or a register
                    let val = self.load_arg(&module, &ins.args[0]).unwrap();
                    match &ins.args[1] {
                        Value::X(reg) => {
                            self.x[*reg as usize] = val;
                            println!("reg: {}", *reg as usize);
                        }
                        Value::Y(reg) => {
                            self.y[*reg as usize] = val;
                            println!("reg: {}", *reg as usize);
                        }
                        reg => panic!("Unhandled register type! {:?}", reg),
                    }
                }
                Opcode::Return => {
                    if self.cp == -1 {
                        println!("Process exited with normal, x0: {:?}", self.x[0]);
                        println!("x: {:?}", self.x);
                        println!("y: {:?}", self.y);
                        break;
                    }
                    self.ip = self.cp as usize;
                    self.cp = -1;
                }
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    println!("call! {:?}", ins.args);
                    if let [Value::Literal(a), Value::Label(i)] = &ins.args[..] {
                        self.cp = self.ip as isize;
                        self.ip = *i as usize - 2;
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        let args = vec![
                            self.load_arg(&module, &ins.args[3]).unwrap(),
                            self.load_arg(&module, &ins.args[4]).unwrap(),
                        ];
                        let res = bif::apply(module.imports.get(*i as usize).unwrap(), args);
                        println!("res: {}", res);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }
    }

    /// In the future, the removal is two part: replace atoms etc
    /// load time structures with Values while loading.
    /// Second, probably move some of the terms into vals (regs etc)
    fn load_arg(&self, module: &Module, arg: &Value) -> Result<Value, &str> {
        match arg {
            Value::Atom(i) => {
                if *i == 0 {
                    return Ok(Value::Nil());
                }
                Ok(Value::Atom(*module.atoms.get(&(*i - 1)).unwrap()))
            }
            Value::ExtendedLiteral(i) => Ok(module.literals.get(*i).unwrap().clone()),
            Value::X(i) => Ok(self.x[*i as usize].clone()),
            Value::Y(i) => Ok(self.y[*i as usize].clone()),
            value => Ok(value.clone()),
        }
    }
}
