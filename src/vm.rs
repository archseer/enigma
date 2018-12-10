use crate::atom;
use crate::loader::Term;
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
    x: [Value; 32],
    y: [Value; 32],
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
                x: std::mem::uninitialized(), //[Value::None(); 32],
                y: std::mem::uninitialized(), //[Value::None(); 32],
                ip: 0,
                cp: -1,
                live: 0,
            };
            for (_i, el) in vm.x.iter_mut().enumerate() {
                // Overwrite `element` without running the destructor of the old value.
                // Since Value does not implement Copy, it is moved.
                std::ptr::write(el, Value::None());
            }
            for (_i, el) in vm.y.iter_mut().enumerate() {
                // Overwrite `element` without running the destructor of the old value.
                // Since Value does not implement Copy, it is moved.
                std::ptr::write(el, Value::None());
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
                        Term::X(reg) => {
                            self.x[*reg as usize] = val;
                            println!("reg: {}", *reg as usize);
                        }
                        Term::Y(reg) => {
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
                    if let [Term::Literal(a), Term::Label(i)] = &ins.args[..] {
                        self.cp = self.ip as isize;
                        self.ip = *i as usize - 2;
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Term::Literal(i) = &ins.args[2] {
                        // GCBifImpl2 func = (GCBifImpl2) mod->imports[bif].bif;
                        println!("gcbif2");
                        let val: Vec<_> = module
                            .imports
                            .iter()
                            .map(|mfa| {
                                (
                                    atom::from_index(
                                        // :(
                                        module.atoms.get(&(mfa.0 as usize - 1)).unwrap(),
                                    )
                                    .unwrap(),
                                    atom::from_index(
                                        module.atoms.get(&(mfa.1 as usize - 1)).unwrap(),
                                    )
                                    .unwrap(),
                                    mfa.2,
                                )
                            })
                            .collect();
                        println!("{:?}", val);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
        }
    }

    /// This is ugly, but: convert a loader::Term into a Value.
    /// In the future, the removal is two part: replace atoms etc
    /// load time structures with Values while loading.
    /// Second, probably move some of the terms into vals (regs etc)
    fn load_arg(&self, module: &Module, arg: &Term) -> Result<Value, &str> {
        match arg {
            Term::Integer(i) => Ok(Value::Integer(*i)),
            Term::Atom(i) => Ok(Value::Atom(*module.atoms.get(&(*i as usize)).unwrap())),
            Term::ExtendedLiteral(i) => Ok(module.literals.get(*i as usize).unwrap().clone()),
            Term::X(i) => Ok(self.x[*i as usize].clone()),
            Term::Y(i) => Ok(self.y[*i as usize].clone()),
            _ => panic!("unexpected term type {:?}", arg),
        }
    }
}
