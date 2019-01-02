use libenigma::vm;

fn main() {
    let vm = vm::Machine::new();

    vm.start("./examples/Elixir.Try.beam");

    println!("execution time: {:?}", vm.elapsed_time())
}
