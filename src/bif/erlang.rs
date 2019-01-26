use crate::bif::BifResult;
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto};
use crate::vm;

pub fn bif_erlang_make_tuple_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_append_element_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_make_tuple_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_setelement_3(_vm: &vm::Machine, process: &RcProcess, &args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_tuple_to_list_1(_vm: &vm::Machine, process: &RcProcess, &args: &[Term]) -> BifResult {
    unimplemented!()
}

#[cfg(test)]
mod tests {

}
