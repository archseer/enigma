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
    // program pointer/reference?
    ip: usize,
    // continuation pointer
    cp: isize, // TODO: ?!
}

impl Machine {
    pub fn new() -> Machine {
        unsafe {
            let mut vm = Machine {
                modules: HashMap::new(),
                x: std::mem::uninitialized(), //[Value::None(); 32],
                ip: 0,
                cp: -1,
            };
            for (_i, el) in vm.x.iter_mut().enumerate() {
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
        self.ip = module.funs.get(&(2, 0)).unwrap().clone();
        // TODO: modify imports to get *local working

        loop {
            let ref ins = module.instructions[self.ip];
            match &ins.op {
                Opcode::FuncInfo => println!("Running a function..."),
                Opcode::Move => {
                    println!("move: {:?}", ins.args);
                    // arg1 can be either a value or a register
                    match &ins.args[1] {
                        Term::X(reg) => {}
                        reg => panic!("Unhandled register type! {:?}", reg),
                    }
                }
                Opcode::Return => {
                    if self.cp == -1 {
                        println!("Process exited with normal");
                        break;
                    }
                    self.ip = self.cp as usize;
                    self.cp = -1;
                }
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
            self.ip = self.ip + 1
        }
    }
}
