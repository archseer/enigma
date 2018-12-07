use crate::atom::AtomTable;

#[derive(Debug)]
pub struct Machine {
    pub atom_table: AtomTable,
    // export table
    // module table
    // register table??

    // registers
    // program pointer/reference?
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            atom_table: AtomTable::new(),
        }
    }
}
