use crate::bif::BifResult;
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto, Tuple};
use crate::exception::{Exception, Reason};
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

pub fn bif_erlang_setelement_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_tuple_to_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{self};
    use crate::module;
    use crate::atom;

    #[test]
    fn test_bif_erlang_make_tuple_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let number = Term::int(2);
        let body = str_to_atom!("test");
        let args = vec![number, body];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_tuple());
            if let Ok(tuple) = x.try_into() {
                let tuple: &Tuple = tuple;
                tuple.iter()
                    .for_each(|x| assert_eq!(str_to_atom!("test"), *x));
                assert_eq!(tuple.len, 2);
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_make_tuple_2_bad_arg() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let number = Term::from(2.1);
        let body = str_to_atom!("test");
        let args = vec![number, body];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
}
