use crate::atom;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Term};
use crate::vm;
use hamt_rs::HamtMap;

pub fn bif_maps_find_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let key = &args[0];
    let map = &args[1];
    if let Term::Map(m) = map {
        let hamt_map = &m.0;
        match hamt_map.find(key) {
            Some(value) => {
                let heap = &process.context_mut().heap;
                return Ok(tup2!(&heap, atom!(OK), value.clone()));
            }
            None => {
                return Ok(atom!(ERROR));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_get_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Term::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        match hamt_map.find(target) {
            Some(value) => {
                return Ok(value.clone());
            }
            None => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, target.clone()));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_from_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    if !args[0].is_list() {
        return Err(Exception::with_value(Reason::EXC_BADARG, args[0].clone()));
    }
    let mut list = &args[0];
    let mut map = HamtMap::new();
    while let Term::List(l) = *list {
        unsafe {
            let item = &(*l).head;
            if let Term::Tuple(ptr) = item {
                let tuple = unsafe { &**ptr };
                if tuple.len != 2 {
                    return Err(Exception::with_value(Reason::EXC_BADARG, item.clone()));
                }
                map = map.plus(tuple[0].clone(), tuple[1].clone());
            } else {
                return Err(Exception::with_value(Reason::EXC_BADARG, item.clone()));
            }
            list = &(*l).tail;
        }
    }
    let heap = &process.context_mut().heap;
    Ok(Term::map(heap, map))
}

pub fn bif_maps_is_key_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Term::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        let exist = hamt_map.find(target).is_some();
        return Ok(Term::boolean(exist));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Term::Map(m) = map {
        let hamt_map = &m.0;
        let heap = &process.context_mut().heap;
        let list = iter_to_list!(heap, hamt_map.iter().map(|(k, _)| k).cloned());
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_merge_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map1 = match &args[0] {
        Term::Map(map) => &(*map.0),
        _ => return Err(Exception::with_value(Reason::EXC_BADMAP, args[0].clone())),
    };
    let map2 = match &args[1] {
        Term::Map(map) => &(*map.0),
        _ => return Err(Exception::with_value(Reason::EXC_BADMAP, args[1].clone())),
    };
    let mut new_map = map2.clone();
    for (k, v) in map1.iter() {
        new_map = new_map.plus(k.clone(), v.clone());
    }
    let heap = &process.context_mut().heap;
    Ok(Term::map(heap, new_map))
}

pub fn bif_maps_put_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Term::Map(m) = map {
        let map = &(*m.0);
        let new_map = map.clone().plus(key.clone(), value.clone());
        return Ok(Term::map(heap, new_map));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_remove_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    if let Term::Map(m) = map {
        let map = &(*m.0);
        let new_map = map.clone().minus(&key.clone());
        return Ok(Term::map(heap, new_map));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_update_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Term::Map(m) = map {
        let map = &(*m.0);
        match map.find(&key) {
            Some(_v) => {
                let new_map = map.clone().plus(key.clone(), value.clone());
                let heap = &process.context_mut().heap;
                return Ok(Term::map(heap, new_map));
            }
            None => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, key.clone()));
            }
        }
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_values_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Term::Map(m) = map {
        let hamt_map = &m.0;
        let heap = &process.context_mut().heap;
        let list = iter_to_list!(heap, hamt_map.iter().map(|(_, v)| v).cloned());
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_take_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let key = &args[0];
    let map = &args[1];
    if let Term::Map(m) = map {
        let result = if let Some(value) = (*m.0).find(key) {
            let new_map = (*m.0).clone().minus(key);
            let heap = &process.context_mut().heap;
            tup2!(heap, value.clone(), Term::map(heap, new_map))
        } else {
            str_to_atom!("error")
        };
        return Ok(result);
    }
    return Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom;
    use crate::module;
    use crate::process;

    #[test]
    fn test_maps_find_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let key = str_to_atom!("test");
        let map = map!(heap, key.clone() => Term::int(3));
        let args = vec![key.clone(), map];

        let res = bif_maps_find_2(&vm, &process, &args);

        if let Ok(Term::Tuple(tuple)) = res {
            unsafe {
                assert_eq!((*tuple).len, 2);
                let slice: &[Term] = &(**tuple);
                let mut iter = slice.iter();
                assert_eq!(iter.next(), Some(&atom!(OK)));
                assert_eq!(iter.next(), Some(&Term::int(3)));
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_find_2_error() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let map = map!(heap, key.clone() => Term::int(3));
        let args = vec![str_to_atom!("fail"), map];

        let res = bif_maps_find_2(&vm, &process, &args);

        assert_eq!(res, Ok(atom!(ERROR)));
    }

    #[test]
    fn test_maps_find_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![str_to_atom!("fail"), str_to_atom!("test")];

        if let Err(exception) = bif_maps_find_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, str_to_atom!("test"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_get_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let map = map!(heap, str_to_atom!("test") => Term::int(3));
        let args = vec![map, str_to_atom!("test")];

        let res = bif_maps_get_2(&vm, &process, &args);

        assert_eq!(res, Ok(Term::int(3)));
    }

    #[test]
    fn test_maps_get_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), Term::int(atom::from_str("test"))];

        if let Err(exception) = bif_maps_get_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_get_2_bad_key() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let map = map!(heap, str_to_atom!("test") => Term::int(3));
        let args = vec![map, str_to_atom!("fail")];

        if let Err(exception) = bif_maps_get_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADKEY);
            assert_eq!(exception.value, str_to_atom!("fail"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_from_list_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let list = cons!(
            heap,
            tup2!(heap, str_to_atom!("test"), Term::int(1)),
            cons!(
                heap,
                tup2!(heap, str_to_atom!("test2"), Term::int(2)),
                Term::nil()
            )
        );
        let args = vec![list];
        let res = bif_maps_from_list_1(&vm, &process, &args);

        if let Ok(Term::Map(m)) = res {
            assert_eq!(m.0.len(), 2);
            assert_eq!(m.0.find(&str_to_atom!("test")), Some(&Term::in(1)));
            assert_eq!(m.0.find(&str_to_atom!("test2")), Some(&Term::int(2)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_from_list_1_not_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let bad_list = Term::int(1);
        let args = vec![bad_list.clone()];
        let res = bif_maps_from_list_1(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
            assert_eq!(exception.value, bad_list.clone());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_from_list_1_bad_items() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let bad_tuple = tup3!(heap, Term::int(1), Term::int(2), Term::int(3));
        let list = cons!(
            heap,
            bad_tuple.clone(),
            cons!(
                heap,
                tup2!(heap, str_to_atom!("test2"), Term::int(2)),
                Term::nil()
            )
        );
        let args = vec![list];
        let res = bif_maps_from_list_1(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADARG);
            assert_eq!(exception.value, bad_tuple.clone());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_is_key_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1));
        let args = vec![map, str_to_atom!("test")];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Term::boolean(true)));
    }

    #[test]
    fn test_maps_is_key_2_false() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(3));
        let args = vec![map, str_to_atom!("false")];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Term::boolean(false)));
    }

    #[test]
    fn test_maps_is_key_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), str_to_atom!("test")];

        if let Err(exception) = bif_maps_is_key_2(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_keys_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let args = vec![map];

        if let Ok(Term::List(cons)) = bif_maps_keys_1(&vm, &process, &args) {
            unsafe {
                let key1 = &(*cons).head;
                assert_eq!(key1, &str_to_atom!("test"));
                if let Term::List(tail) = (*cons).tail {
                    let key2 = &(*tail).head;
                    assert_eq!(key2, &str_to_atom!("test2"));
                } else {
                    panic!();
                }
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_keys_1_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), str_to_atom!("test")];

        if let Err(exception) = bif_maps_keys_1(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_merge_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map1 =
            map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let map2 =
            map!(str_to_atom!("test") => Term::int(3), str_to_atom!("test3") => Term::int(4));
        let args = vec![map1.clone(), map2.clone()];

        let res = bif_maps_merge_2(&vm, &process, &args);
        if let Ok(Term::Map(body)) = res {
            assert_eq!(body.0.len(), 3);
            assert_eq!(body.0.find(&str_to_atom!("test")), Some(&Term::int(1)));
            assert_eq!(body.0.find(&str_to_atom!("test2")), Some(&Term::int(2)));
            assert_eq!(body.0.find(&str_to_atom!("test3")), Some(&Term::int(4)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_merge_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let bad_map = Term::int(2);

        let args = vec![map.clone(), bad_map.clone()];
        let res = bif_maps_merge_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map.clone());
        } else {
            panic!();
        }

        let args = vec![bad_map.clone(), map.clone()];
        let res = bif_maps_merge_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map.clone());
        } else {
            panic!();
        }

        // Will return the first bad map
        let bad_map2 = Term::int(3);
        let args = vec![bad_map.clone(), bad_map2.clone()];
        let res = bif_maps_merge_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map.clone());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_put_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");

        let value = Term::int(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![
            Term::Map(value::Map(Arc::new(map))),
            key.clone(),
            value.clone(),
        ];

        let res = bif_maps_put_3(&vm, &process, &args);

        if let Ok(Term::Map(body)) = res {
            assert_eq!(body.0.find(&key), Some(&value));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_put_3_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let value = Term::int(2);
        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), key.clone(), value.clone()];

        let res = bif_maps_put_3(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_remove_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let map = map!(key.clone() => Term::int(1));
        let args = vec![map, key.clone()];

        let res = bif_maps_remove_2(&vm, &process, &args);

        if let Ok(Term::Map(body)) = res {
            assert_eq!(body.0.find(&key).is_none(), true);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_remove_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let args = vec![Term::int(1), Term::int(2)];

        let res = bif_maps_remove_2(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, Term::int(1));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_update_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let value = Term::int(1);
        let update_value = Term::int(2);
        let map = map!(key.clone() => value.clone());
        let args = vec![map, key.clone(), update_value.clone()];

        let res = bif_maps_update_3(&vm, &process, &args);

        if let Ok(Term::Map(body)) = res {
            assert_eq!(body.0.find(&key), Some(&update_value));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_update_3_bad_key() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let value = Term::int(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![
            Term::Map(value::Map(Arc::new(map))),
            key.clone(),
            value.clone(),
        ];

        let res = bif_maps_update_3(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADKEY);
            assert_eq!(exception.value, str_to_atom!("test"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_update_3_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let value = Term::int(2);
        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), key.clone(), value.clone()];

        let res = bif_maps_update_3(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_values_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let args = vec![map];

        if let Ok(Term::List(cons)) = bif_maps_values_1(&vm, &process, &args) {
            unsafe {
                let key1 = &(*cons).head;
                assert_eq!(key1, &Term::int(1));
                if let Term::List(tail) = (*cons).tail {
                    let key2 = &(*tail).head;
                    assert_eq!(key2, &Term::int(2));
                } else {
                    panic!();
                }
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_values_1_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Term::int(3);
        let args = vec![bad_map.clone(), str_to_atom!("test")];

        if let Err(exception) = bif_maps_values_1(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_take_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let key = str_to_atom!("test2");
        let args = vec![key.clone(), map.clone()];

        let res = bif_maps_take_2(&vm, &process, &args);
        if let Ok(Term::Tuple(t)) = res {
            let tuple = unsafe { &(**t) };
            let mut iter = tuple.iter();
            assert_eq!(Term::int(2), iter.next().unwrap().clone());
            if let Term::Map(result_map) = iter.next().unwrap() {
                assert_eq!(result_map.0.len(), 1);
                assert_eq!(
                    result_map.0.find(&str_to_atom!("test")),
                    Some(&Term::int(1))
                );
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_take_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = str_to_atom!("test2");
        let key = str_to_atom!("test2");
        let args = vec![key.clone(), bad_map.clone()];

        let res = bif_maps_take_2(&vm, &process, &args);
        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map.clone());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_take_2_bad_key() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let key = str_to_atom!("test3");
        let args = vec![key.clone(), map.clone()];

        let res = bif_maps_take_2(&vm, &process, &args);
        if let Ok(value) = res {
            assert_eq!(value, str_to_atom!("error"));
        } else {
            panic!();
        }
    }
}
