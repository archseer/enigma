mod atom;
mod etf;
mod loader;
mod opcodes;
mod value;
mod vm;

#[macro_use]
extern crate once_cell;

use crate::loader::Loader;
use crate::opcodes::*;

fn main() {
    let vm = vm::Machine::new();
    let mut loader = Loader::new(&vm);

    let bytes = include_bytes!("../hello.beam");
    let chunk = loader.load_file(bytes).unwrap();
}
