use crate::bif::BifResult;
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto, Tuple};
use crate::exception::{Exception, Reason};
use crate::vm;

pub fn bif_erlang_make_tuple_2(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_make_tuple_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_append_element_2(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_setelement_3(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
    unimplemented!()
}

pub fn bif_erlang_tuple_to_list_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> BifResult {
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
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_tuple());
            if let Ok(tuple) = x.try_into() {
                let tuple: &Tuple = tuple;
                tuple.iter()
                    .for_each(|x| assert_eq!(x, &str_to_atom!("test")));
                assert_eq!(tuple.len, 2);
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_make_tuple_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let number = Term::from(2.1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_make_tuple_2_bad_arg_negative_number() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let number = Term::int(-1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
    #[test]
    fn test_bif_erlang_tuple_make_tuple_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(&heap, vec![
            tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
            tup2!(&heap, Term::int(5), str_to_atom!("zz")),
            tup2!(&heap, Term::int(2), str_to_atom!("aa")),
        ].iter().map(|x| *x));


        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_tuple());
            if let Ok(tuple) = x.try_into() {
                let tuple: &Tuple = tuple;
                assert_eq!(tuple.len, 5);
                assert_eq!(tuple[0], Term::from(1));
                assert_eq!(tuple[1], str_to_atom!("aa"));
                assert_eq!(tuple[2], Term::from(1));
                assert_eq!(tuple[3], Term::from(1));
                assert_eq!(tuple[4], str_to_atom!("zz"));
            } else {
                panic!();
            }
        } else {
            panic!();
        }

    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_wrong_type_of_number() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::from(2.1);
        let default = Term::from(1);
        let init_list = iter_to_list!(&heap, vec![
            tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
            tup2!(&heap, Term::int(5), str_to_atom!("zz")),
            tup2!(&heap, Term::int(2), str_to_atom!("aa")),
        ].iter().map(|x| *x));

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_negative_number() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(-1);
        let default = Term::from(1);
        let init_list = iter_to_list!(&heap, vec![
            tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
            tup2!(&heap, Term::int(5), str_to_atom!("zz")),
            tup2!(&heap, Term::int(2), str_to_atom!("aa")),
        ].iter().map(|x| *x));

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_wrong_type_of_init_list() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let _heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = Term::from(1);

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_wrong_structure_init_list() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(&heap, vec![
            tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
            tup3!(&heap, Term::int(5), str_to_atom!("zz"), Term::from(1)),
            tup2!(&heap, Term::from(2.0), str_to_atom!("aa")),
        ].iter().map(|x| *x));

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_init_list_out_of_range() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(&heap, vec![
            tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
            tup2!(&heap, Term::int(5), str_to_atom!("zz")),
            tup2!(&heap, Term::int(10), str_to_atom!("aa")),
        ].iter().map(|x| *x));

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_append_element_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &process.context_mut().heap;
        let tuple = tup2!(&heap, Term::int(0), Term::int(1));
        let append = Term::int(2);
        let args = vec![tuple, append];

        let res = bif_erlang_append_element_2(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_tuple());
            if let Ok(tuple) = x.try_into() {
                let tuple: &Tuple = tuple;
                assert_eq!(tuple.len, 3);
                for (i, x) in tuple.iter().enumerate() {
                    assert_eq!(x, &Term::int(i as i32));
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_append_element_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_tuple = Term::int(0);
        let append = Term::int(2);
        let args = vec![bad_tuple, append];

        let res = bif_erlang_append_element_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_setelement_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(1);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = bif_erlang_setelement_3(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_tuple());
            if let Ok(tuple) = x.try_into() {
                let tuple: &Tuple = tuple;
                assert_eq!(tuple.len, 3);
                assert_eq!(tuple[0], str_to_atom!("test"));
                assert_eq!(tuple[1], Term::from(99));
                assert_eq!(tuple[2], Term::from(2));
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_setelement_3_bad_arg_wrong_type_of_index() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::from(1.1);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = bif_erlang_setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_setelement_3_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let _heap = &process.context_mut().heap;
        let index = Term::int(1);
        let tuple = Term::from(1);
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = bif_erlang_setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_setelement_3_bad_arg_tuple_out_of_range() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(9);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = bif_erlang_setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_to_list_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let heap = &process.context_mut().heap;
        let tuple = tup2!(&heap, str_to_atom!("test"), tup2!(&heap, Term::from(1), Term::from(2)));
        let args = vec![tuple];

        let res = bif_erlang_tuple_to_list_1(&vm, &process, &args);
        if let Ok(x) = res {
            assert!(x.is_list());
            if let Ok(cons) = x.try_into() {
                let cons: &value::Cons = cons;
                assert_eq!(cons.iter().count(), 2);
                let mut iter = cons.iter();
                assert_eq!(iter.next().unwrap(), &str_to_atom!("test"));
                assert_eq!(iter.next().unwrap(), &tup2!(&heap, Term::from(1), Term::from(2)));
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_to_list_1_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_tuple = Term::from(1);
        let args = vec![bad_tuple];

        let res = bif_erlang_tuple_to_list_1(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
}
