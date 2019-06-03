use libenigma::vm;

use std::env;
use std::process;

fn run() -> i32 {
    use simplelog::*;
    use std::fs::File;
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("enigma.log").unwrap(),
    )])
    .unwrap();

    let _args: Vec<String> = env::args().collect();

    // std::panic::set_hook(Box::new(|panic_info| {
    //     let backtrace = backtrace::Backtrace::new();
    //     println!("{:?}", panic_info);
    //     println!("{:?}", backtrace);
    // }));

    println!(
        "size_of: {}",
        std::mem::size_of::<libenigma::loader::Instruction>()
    );
    println!(
        "size_of: new_instr {}",
        std::mem::size_of::<libenigma::instruction::Instruction>()
    );
    println!(
        "size_of: {}",
        std::mem::size_of::<Vec<libenigma::loader::LValue>>()
    );
    let vm = vm::Machine::new();

    // erlexec defaults:
    let args: Vec<String> = vec![
        "/usr/local/Cellar/erlang/21.3.2/lib/erlang/erts-10.2.3/bin/enigma.smp",
        "--",
        "-root",
        //"/usr/local/Cellar/erlang/21.3.2/lib/erlang",
        // "otp",
        "/Users/speed/src/rust/enigma/otp",
        "-progname",
        "enigma",
        "--",
        "-home",
        dirs::home_dir()
            .expect("No home directory")
            .to_str()
            .unwrap(),
        "--",
        // "-init_debug",
        "-kernel",
        "start_distribution",
        "false",
        // "-kernel",
        // "logger_level",
        // "debug",
        // "-kernel",
        // "logger_log_progress",
        // "true",
        // "-kernel shell_history enabled",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    vm.preload_modules();

    vm.start(args);

    // println!("execution time: {:?}", vm.elapsed_time());
    0
}

fn main() {
    process::exit(run());
}
