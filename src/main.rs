mod loader;
mod opcodes;
use crate::opcodes::*;

fn main() {
    let bytes = include_bytes!("../hello.beam");
    let chunk = loader::load_file(bytes).unwrap();

    // TODO: scan over the Code chunk bits, but we'll need to know
    // bit length of each instruction.
    let mut pc = 0;
    loop {
        let (_, res) = loader::scan_instructions(chunk.code).unwrap();
        println!("res: {:#?}", res);

        let instruction = loader::Instruction {
            op: Opcode::Line,
            args: vec![],
        };

        match instruction.op {
            Opcode::Line => {
                // one operand, Integer
                break;
            }
            opcode => println!("Unimplemented opcode {:?}", opcode),
        }
    }
}

// --------------------

#[derive(Debug)]
pub struct Context {
    // atom table
// export table
// module table
// register table??

// registers
// program pointer/reference?
}
