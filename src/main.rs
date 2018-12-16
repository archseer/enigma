mod arc_without_weak;
mod atom;
mod bif;
mod etf;
mod loader;
mod module;
mod module_registry;
mod opcodes;
mod pool;
mod process;
mod process_table;
mod queue;
mod value;
mod vm;

#[macro_use]
extern crate once_cell;

use time;

fn main() {
    let mut vm = vm::Machine::new();

    //vm.register_module(module);
    let start = time::precise_time_ns();

    vm.start("./fib.beam");

    println!(
        "execution time: {:?}",
        (time::precise_time_ns() - start) // / 1000
    )
}
