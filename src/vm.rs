use crate::module::Module;
use crate::opcodes::Opcode;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Machine {
    // atom table is accessible globally as ATOMS
    // export table
    // module table
    modules: HashMap<usize, Module>,
    // register table??

    // registers
    // program pointer/reference?
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module: Module) {
        // TODO: use a module atom name
        self.modules.insert(0, module);
    }

    // value is an atom
    pub fn run(&mut self, module: Module, fun: usize) {
        println!("one");
        let local = module.atoms.get(&fun).unwrap();
        println!("two: {:?}, fun:{:?}, local: {:?}", module.funs, fun, local);
        let mut pc = module.funs.get(&(1, 0)).unwrap().clone();
        // TODO: modify imports to get *local working

        loop {
            let ref instruction = module.instructions[pc];
            match &instruction.op {
                Opcode::FuncInfo => println!("Running a function..."),
                Opcode::Move => {}
                Opcode::Return => {}
                opcode => println!("Unimplemented opcode {:?}", opcode),
            }
            pc = pc + 1
        }
    }
}
