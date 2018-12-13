mod atom;
mod bif;
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
use time;

fn main() {
    let mut vm = vm::Machine::new();
    let loader = Loader::new(&vm);

    let bytes = include_bytes!("../fib.beam");
    let module = loader.load_file(bytes).unwrap();
    println!("module: {:?}", module);
    //vm.register_module(module);
    let start = time::precise_time_ns();
    if let Value::Atom(index) = atom::from_str("fib") {
        vm.run(module, index);
    }

    println!(
        "execution time: {:?}",
        (time::precise_time_ns() - start) / 1000
    )
}
