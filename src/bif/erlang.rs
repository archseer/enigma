use crate::atom;
use crate::bif;
use crate::bitstring;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Cons, Term, TryFrom, TryInto, Tuple, Variant};
use crate::vm;
use lexical;

pub fn make_tuple_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
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

pub fn make_tuple_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
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
    let init = Cons::try_from(&args[2])?;

    for item in init.iter() {
        let t = Tuple::try_from(&item)?;
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

pub fn append_element_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let t = Tuple::try_from(&args[0])?;
    let heap = &process.context_mut().heap;
    let new_tuple = value::tuple(heap, (t.len() + 1) as u32);
    new_tuple[..t.len()].copy_from_slice(&t[..]);
    unsafe {
        std::ptr::write(&mut new_tuple[t.len()], args[1]);
    }
    Ok(Term::from(new_tuple))
}

pub fn setelement_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let number = match args[0].into_number() {
        Ok(value::Num::Integer(i)) if !i < 1 => i - 1,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let t = Tuple::try_from(&args[1])?;
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

pub fn tuple_to_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let t = Tuple::try_from(&args[0])?;
    let mut n = (t.len() - 1) as i32;
    let mut list = Term::nil();
    let heap = &process.context_mut().heap;
    while n >= 0 {
        list = cons!(heap, t[n as usize], list);
        n -= 1;
    }
    Ok(list)
}

pub fn binary_to_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let binary = args[0];

    // TODO: extract as macro
    let (bytes, bitoffs, size) = match binary.get_boxed_header() {
        Ok(value::BOXED_BINARY) => {
            // TODO use ok_or to cast to some, then use ?
            let value = &binary.get_boxed_value::<bitstring::RcBinary>().unwrap();
            (&value.data[..], 0, value.data.len())
        }
        Ok(value::BOXED_SUBBINARY) => {
            // TODO use ok_or to cast to some, then use ?
            let value = &binary.get_boxed_value::<bitstring::SubBinary>().unwrap();
            (
                &value.original.data[value.offset..],
                value.bit_offset,
                value.size,
            )
        }
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let res = bitstring::bytes_to_list(
        &process.context_mut().heap,
        Term::nil(),
        bytes,
        size,
        bitoffs,
    );
    //println!("{}", res);
    Ok(res)
}

/// convert a list of ascii integers to an atom
pub fn list_to_atom_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // Eterm res;
    // byte *buf = (byte *) erts_alloc(ERTS_ALC_T_TMP, MAX_ATOM_SZ_LIMIT);
    // Sint written;
    // int i = erts_unicode_list_to_buf(BIF_ARG_1, buf, MAX_ATOM_CHARACTERS,
    //                                  &written);
    // if (i < 0) {
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // if (i == -2) {
    // BIF_ERROR(BIF_P, SYSTEM_LIMIT);
    // }
    // BIF_ERROR(BIF_P, BADARG);
    // }
    // res = erts_atom_put(buf, written, ERTS_ATOM_ENC_UTF8, 1);
    // ASSERT(is_atom(res));
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_RET(res);
    let list = Cons::try_from(&args[0])?;
    let string = value::cons::unicode_list_to_buf(list, atom::MAX_ATOM_CHARS)?;
    let atom = atom::from_str(string.as_str());
    Ok(Term::atom(atom))
}

/// conditionally convert a list of ascii integers to an atom
pub fn list_to_existing_atom_1(
    _vm: &vm::Machine,
    process: &RcProcess,
    args: &[Term],
) -> bif::Result {
    // byte *buf = (byte *) erts_alloc(ERTS_ALC_T_TMP, MAX_ATOM_SZ_LIMIT);
    // Sint written;
    // int i = erts_unicode_list_to_buf(BIF_ARG_1, buf, MAX_ATOM_CHARACTERS,
    //                                  &written);
    // if (i < 0) {
    // error:
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_ERROR(BIF_P, BADARG);
    // } else {
    // Eterm a;

    // if (erts_atom_get((char *) buf, written, &a, ERTS_ATOM_ENC_UTF8)) {
    // erts_free(ERTS_ALC_T_TMP, (void *) buf);
    // BIF_RET(a);
    // } else {
    // goto error;
    // }
    // }
    unimplemented!()
}

pub fn list_to_binary_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let mut bytes: Vec<u8> = Vec::new();
    let heap = &process.context_mut().heap;

    // if nil return an empty string
    if args[0].is_nil() {
        return Ok(Term::binary(heap, bitstring::Binary::new()));
    }

    let mut stack = Vec::new();
    let cons = Cons::try_from(&args[0])?;
    stack.push(cons.iter());

    // TODO fastpath for if [binary]
    while let Some(iter) = stack.last_mut() {
        if let Some(elem) = iter.next() {
            match elem.into_variant() {
                Variant::Integer(i @ 0...255) => {
                    // append int to bytes
                    bytes.push(i as u8);
                }
                Variant::Cons(ptr) => {
                    // push cons onto stack
                    let cons = unsafe { &*ptr };
                    stack.push(cons.iter())
                }
                Variant::Pointer(..) => match elem.to_bytes() {
                    Some(data) => bytes.extend_from_slice(data),
                    None => return Err(Exception::new(Reason::EXC_BADARG)),
                },
                _ => return Err(Exception::new(Reason::EXC_BADARG)),
            }
        } else {
            stack.pop();
        }
    }

    Ok(Term::binary(heap, bitstring::Binary::from(bytes)))
}
// TODO iolist_to_binary is the same, input can be a binary (is_binary() true), and we just return
// it (badarg on bitstring)

pub fn binary_to_term_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // TODO: needs to yield mid parsing...
    if let Some(string) = args[0].to_bytes() {
        match crate::etf::decode(string, &process.context_mut().heap) {
            Ok((_, term)) => return Ok(term),
            Err(error) => panic!("binary_to_term error: {:?}", error),
        };
    }
    Err(Exception::new(Reason::EXC_BADARG))
}

pub fn atom_to_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_variant() {
        Variant::Atom(i) => {
            let string = atom::to_str(i).unwrap();
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn integer_to_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    match args[0].into_variant() {
        Variant::Integer(i) => {
            let string = lexical::to_string(i);
            let heap = &process.context_mut().heap;

            Ok(bitstring!(heap, string))
        }
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn display_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    println!("{}", args[0]);
    Ok(atom!(OK))
}

/// erlang:'++'/2
///
/// Adds a list to another (LHS ++ RHS). For historical reasons this is implemented by copying LHS
/// and setting its tail to RHS without checking that RHS is a proper list. [] ++ 'not_a_list' will
/// therefore result in 'not_a_list', and [1,2] ++ 3 will result in [1,2|3], and this is a bug that
/// we have to live with.
pub fn append_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let lhs = args[0];
    let rhs = args[1];

    let heap = &process.context_mut().heap;

    // This is buggy but expected, `[] ++ 'not_a_list'` has always resulted in 'not_a_list'.
    if lhs.is_nil() {
        return Ok(rhs);
    }

    // TODO: use into_variant match?

    // TODO: this same type of logic appears a lot, need to abstract it out, too much unsafe use
    if let Ok(value::Cons { head, tail }) = lhs.try_into() {
        // keep copying lhs until we reach the tail, point it to rhs
        let mut iter = tail;

        let c = heap.alloc(value::Cons {
            head: *head,
            tail: Term::nil(),
        });

        let mut ptr = c as *mut value::Cons;

        while let Ok(value::Cons { head, tail }) = iter.try_into() {
            let new_cons = heap.alloc(value::Cons {
                head: *head,
                tail: Term::nil(),
            });

            let prev = unsafe { &mut (*ptr).tail };
            ptr = new_cons as *mut value::Cons;
            std::mem::replace(prev, Term::from(new_cons));

            iter = tail;
        }

        // now link the copy to the rhs
        unsafe {
            (*ptr).tail = rhs;
        }
        return Ok(Term::from(c));
    }

    // assert!(!(BIF_P->flags & F_DISABLE_GC));
    Err(Exception::new(Reason::EXC_BADARG))
}

pub fn make_ref_0(vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let reference = vm.state.next_ref();

    // TODO: heap allocating these is not ideal
    Ok(Term::reference(heap, reference))
}

// for the time being, these two functions are constant since we don't do distributed
pub fn node_0(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    Ok(atom!(NO_NODE_NO_HOST))
}
pub fn node_1(_vm: &vm::Machine, _process: &RcProcess, _args: &[Term]) -> bif::Result {
    Ok(atom!(NO_NODE_NO_HOST))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom;
    use crate::module;
    use crate::process;
    use crate::value::Cons;

    #[test]
    fn test_make_tuple_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let number = Term::int(2);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
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
    fn test_make_tuple_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let number = Term::from(2.1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_make_tuple_2_bad_arg_negative_number() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let number = Term::int(-1);
        let default = str_to_atom!("test");
        let args = vec![number, default];

        let res = make_tuple_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
    #[test]
    fn test_tuple_make_tuple_3() {
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
            .into_iter(),
            &heap,
        );

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
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
    fn test_tuple_make_tuple_3_bad_arg_wrong_type_of_number() {
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

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_negative_number() {
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

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_wrong_type_of_init_list() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();
        let _heap = &process.context_mut().heap;

        let number = Term::int(5);
        let default = Term::from(1);
        let init_list = Term::from(1);

        let args = vec![number, default, init_list];

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_wrong_structure_init_list() {
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

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_make_tuple_3_bad_arg_init_list_out_of_range() {
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

        let res = make_tuple_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_append_element_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let tuple = tup2!(&heap, Term::int(0), Term::int(1));
        let append = Term::int(2);
        let args = vec![tuple, append];

        let res = append_element_2(&vm, &process, &args);
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
    fn test_append_element_2_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let bad_tuple = Term::int(0);
        let append = Term::int(2);
        let args = vec![bad_tuple, append];

        let res = append_element_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(2);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
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
    fn test_setelement_3_bad_arg_wrong_type_of_index() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::from(1.1);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let _heap = &process.context_mut().heap;
        let index = Term::int(1);
        let tuple = Term::from(1);
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_setelement_3_bad_arg_tuple_out_of_range() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let heap = &process.context_mut().heap;
        let index = Term::int(4);
        let tuple = tup3!(&heap, str_to_atom!("test"), Term::from(1), Term::from(2));
        let value = Term::from(99);
        let args = vec![index, tuple, value];

        let res = setelement_3(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_tuple_to_list_1() {
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

        let res = tuple_to_list_1(&vm, &process, &args);
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
    fn test_tuple_to_list_1_bad_arg_wrong_type_of_tuple() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, None, module).unwrap();

        let bad_tuple = Term::from(1);
        let args = vec![bad_tuple];

        let res = tuple_to_list_1(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
        } else {
            panic!();
        }
    }
}
