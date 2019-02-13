use libenigma::vm;

use getopts::{Options, ParsingStyle};
use std::env;
use std::process;

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();

    let vm = vm::Machine::new();

    // erlexec defaults:
    //  ["/usr/local/Cellar/erlang/21.2.4/lib/erlang/erts-10.2.3/bin/enigma.smp", "--", "-root", "/usr/local/Cellar/erlang/21.2.4/lib/erlang", "-progname", "erlcat", "--", "-home", "/Users/speed", "--", "-kernel" , "shell_history", "enabled"]
    println!("{:?}", args);

    vm.preload_modules();

    vm.start("./examples/Elixir.Bin.beam");
    // vm.start("./examples/fib.beam");

    println!("execution time: {:?}", vm.elapsed_time());
    0
}

fn main() {
    process::exit(run());
}
