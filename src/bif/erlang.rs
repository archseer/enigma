use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto, Tuple};
use crate::vm;

pub fn bif_erlang_make_tuple_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let num = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 0 => i,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let heap = &process.context_mut().heap;
    let tuple = value::tuple(heap, num as u32);
    for i in 0..num {
        unsafe {
            std::ptr::write(&mut tuple[i as usize], args[1]);
        }
    }
    Ok(Term::from(tuple))
}

pub fn bif_erlang_make_tuple_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let num = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 0 => i,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let heap = &process.context_mut().heap;
    let tuple = value::tuple(heap, num as u32);
    for i in 0..num {
        unsafe {
            std::ptr::write(&mut tuple[i as usize], args[1]);
        }
    }
    let init: &value::Cons = match args[2].try_into() {
        Ok(cons) => cons,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    for item in init.iter() {
        let t: &Tuple = match item.try_into() {
            Ok(tuple) => tuple, // FIXME do the len checking here
            _ => return Err(Exception::new(Reason::EXC_BADARG)),
        };
        if t.len != 2 {
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        let n = match t[0].into_number() {
            Ok(value::Num::Integer(i)) if !i < 1 && i - 1 < num => i - 1,
            _ => return Err(Exception::new(Reason::EXC_BADARG)),
        };
        unsafe {
            std::ptr::write(&mut tuple[n as usize], t[1]);
        }
    }
    Ok(Term::from(tuple))
}

pub fn bif_erlang_append_element_2(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Term],
) -> BifResult {
    if !args[0].is_tuple() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    let t: &Tuple = match args[0].try_into() {
        Ok(tuple) => tuple,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let heap = &process.context_mut().heap;
    let new_tuple = value::tuple(heap, (t.len() + 1) as u32);
    new_tuple[..t.len()].copy_from_slice(&t[..]);
    unsafe {
        std::ptr::write(&mut new_tuple[t.len()], args[1]);
    }
    Ok(Term::from(new_tuple))
}

pub fn bif_erlang_setelement_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let number = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 1 => i - 1,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let t: &Tuple = match args[1].try_into() {
        Ok(tuple) => tuple,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    if number >= t.len() as i32 {
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    let heap = &process.context_mut().heap;
    let new_tuple = value::tuple(heap, t.len() as u32);
    unsafe {
        new_tuple[..t.len()].copy_from_slice(&t[..]);
        std::ptr::write(&mut new_tuple[number as usize], args[2]);
    }
    Ok(Term::from(new_tuple))
}

pub fn bif_erlang_tuple_to_list_1(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Term],
) -> BifResult {
    let t: &Tuple = match args[0].try_into() {
        Ok(tuple) => tuple,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let mut n = (t.len() - 1) as i32;
    let mut list = Term::nil();
    let heap = &process.context_mut().heap;
    while n >= 0 {
        list = cons!(heap, t[n as usize], list);
        n -= 1;
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom;
    use crate::module;
    use crate::process;
    use crate::value::Cons;

    #[test]
    fn test_bif_erlang_make_tuple_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let number = Term::int(2);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = bif_erlang_make_tuple_2(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_tuple());
        if let Ok(tuple) = x.try_into() {
            let tuple: &Tuple = tuple;
            tuple
                .iter()
                .for_each(|x| assert_eq!(x, &str_to_atom!("test")));
            assert_eq!(tuple.len, 2);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_make_tuple_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

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
        let process = process::allocate(&vm.state, None, module).unwrap();

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
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = Cons::from_iter(
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(5), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .iter(),
            &heap,
        );

        let args = vec![number, default, init_list];

        let res = bif_erlang_make_tuple_3(&vm, &process, &args);
        let x = res.unwrap();
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
    }

    #[test]
    fn test_bif_erlang_tuple_make_tuple_3_bad_arg_wrong_type_of_number() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::from(2.1);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

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
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(-1);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(2), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

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
        let process = process::allocate(&vm.state, None, module).unwrap();
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
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup3!(&heap, Term::int(4), str_to_atom!("zz"), Term::from(1)),
                tup2!(&heap, Term::from(2.0), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

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
        let process = process::allocate(&vm.state, None, module).unwrap();
        let heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = iter_to_list!(
            &heap,
            vec![
                tup2!(&heap, Term::int(2), str_to_atom!("ignored")),
                tup2!(&heap, Term::int(4), str_to_atom!("zz")),
                tup2!(&heap, Term::int(10), str_to_atom!("aa")),
            ]
            .iter()
            .map(|x| *x)
        );

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
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let tuple = tup2!(&heap, Term::int(0), Term::int(1));
        let append = Term::int(2);
        let args = vec![tuple, append];

        let res = bif_erlang_append_element_2(&vm, &process, &args);
        let x = res.unwrap();
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
    }

    #[test]
    fn test_bif_erlang_append_element_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

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
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(2);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = bif_erlang_setelement_3(&vm, &process, &args);
        let x = res.unwrap();
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
    }

    #[test]
    fn test_bif_erlang_setelement_3_bad_arg_wrong_type_of_index() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

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
        let process = process::allocate(&vm.state, None, module).unwrap();

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
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(4);
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
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let tuple = tup2!(
            &heap,
            str_to_atom!("test"),
            tup2!(&heap, Term::from(1), Term::from(2))
        );
        let args = vec![tuple];

        let res = bif_erlang_tuple_to_list_1(&vm, &process, &args);
        let x = res.unwrap();
        assert!(x.is_list());
        if let Ok(cons) = x.try_into() {
            let cons: &value::Cons = cons;
            assert_eq!(cons.iter().count(), 2);
            let mut iter = cons.iter();
            assert_eq!(iter.next().unwrap(), &str_to_atom!("test"));
            assert_eq!(
                iter.next().unwrap(),
                &tup2!(&heap, Term::from(1), Term::from(2))
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn test_bif_erlang_tuple_to_list_1_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

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
