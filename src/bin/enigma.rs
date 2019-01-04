use libenigma::vm;

use getopts::{Options, ParsingStyle};
use std::env;
use std::process;

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();

    let vm = vm::Machine::new();

    vm.preload_modules();
    vm.start("./examples/Elixir.Lambda.beam");

    println!("execution time: {:?}", vm.elapsed_time());
    0
}

fn main() {
    process::exit(run());
}
