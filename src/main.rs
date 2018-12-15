mod arc_without_weak;
mod atom;
mod bif;
mod etf;
mod loader;
mod module;
mod opcodes;
mod pool;
mod process;
mod process_table;
mod queue;
mod value;
mod vm;

#[macro_use]
extern crate once_cell;
use crate::value::Value;

use time;

fn main() {
    let mut vm = vm::Machine::new();

    //vm.register_module(module);
    let start = time::precise_time_ns();
    if let Value::Atom(index) = atom::from_str("fib") {
        vm.run(module, index);
    }

    println!(
        "execution time: {:?}",
        (time::precise_time_ns() - start) // / 1000
    )
}
