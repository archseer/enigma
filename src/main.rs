mod atom;
mod etf;
mod loader;
mod module;
mod opcodes;
mod value;
mod vm;

#[macro_use]
extern crate once_cell;
use crate::value::Value;

use crate::loader::Loader;

fn main() {
    let mut vm = vm::Machine::new();
    let loader = Loader::new(&vm);

    let bytes = include_bytes!("../hello.beam");
    let module = loader.load_file(bytes).unwrap();
    println!("module: {:?}", module);
    //vm.register_module(module);
    if let Value::Atom(index) = atom::from_str("hello") {
        vm.run(module, index);
    }
}
