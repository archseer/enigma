use crate::atom;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Term, TryInto};
use crate::vm;
use hamt_rs::HamtMap;

pub fn bif_maps_find_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let key = &args[0];
    let map = &args[1];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        match map.find(key) {
            Some(value) => {
                let heap = &process.context_mut().heap;
                return Ok(tup2!(&heap, atom!(OK), *value));
            }
            None => {
                return Ok(atom!(ERROR));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_get_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let target = &args[1];
        match map.find(target) {
            Some(value) => {
                return Ok(*value);
            }
            None => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, *target));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_from_list_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let mut list = &args[0];
    if !list.is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    let mut map = HamtMap::new();
    while let Ok(value::Cons { head, tail }) = list.try_into() {
        if let Ok(tuple) = head.try_into() {
            let tuple: &value::Tuple = tuple; // annoying, need type annotation
            if tuple.len != 2 {
                return Err(Exception::new(Reason::EXC_BADARG));
            }
            map = map.plus(tuple[0], tuple[1]);
        } else {
            return Err(Exception::new(Reason::EXC_BADARG));
        }
        list = tail;
    }
    let heap = &process.context_mut().heap;
    Ok(Term::map(heap, map))
}

pub fn bif_maps_is_key_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let target = &args[1];
        let exist = map.find(target).is_some();
        return Ok(Term::boolean(exist));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let heap = &process.context_mut().heap;
        let list = iter_to_list!(heap, map.iter().map(|(k, _)| k).cloned());
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_merge_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map1 = match args[0].try_into() {
        Ok(value::Map { map, .. }) => map,
        _ => return Err(Exception::with_value(Reason::EXC_BADMAP, args[0])),
    };
    let map2 = match args[1].try_into() {
        Ok(value::Map { map, .. }) => map,
        _ => return Err(Exception::with_value(Reason::EXC_BADMAP, args[1])),
    };
    let mut new_map = map2.clone();
    for (k, v) in map1.iter() {
        new_map = new_map.plus(*k, *v);
    }
    let heap = &process.context_mut().heap;
    Ok(Term::map(heap, new_map))
}

pub fn bif_maps_put_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let new_map = map.clone().plus(*key, *value);
        return Ok(Term::map(heap, new_map));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_remove_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let new_map = map.clone().minus(&key.clone());
        return Ok(Term::map(heap, new_map));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_update_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        match map.find(&key) {
            Some(_v) => {
                let new_map = map.clone().plus(key.clone(), value.clone());
                let heap = &process.context_mut().heap;
                return Ok(Term::map(heap, new_map));
            }
            None => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, *key));
            }
        }
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_values_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let map = &args[0];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let heap = &process.context_mut().heap;
        let list = iter_to_list!(heap, map.iter().map(|(_, v)| v).cloned());
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
}

pub fn bif_maps_take_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> BifResult {
    let key = &args[0];
    let map = &args[1];
    if let Ok(value::Map { map, .. }) = map.try_into() {
        let result = if let Some(value) = map.find(key) {
            let new_map = map.clone().minus(key);
            let heap = &process.context_mut().heap;
            tup2!(heap, *value, Term::map(heap, new_map))
        } else {
            str_to_atom!("error")
        };
        return Ok(result);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, *map))
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

        if let Ok(tuple) = res.unwrap().try_into() {
            let tuple: &value::Tuple = tuple; // annoying, need type annotation
            assert_eq!(tuple.len, 2);
            let mut iter = tuple.iter();
            assert_eq!(iter.next(), Some(&atom!(OK)));
            assert_eq!(iter.next(), Some(&Term::int(3)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_find_2_error() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

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
        let args = vec![bad_map.clone(), Term::atom(atom::from_str("test"))];

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
        println!("aaa");
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
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_is_key_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let map = map!(heap, str_to_atom!("test") => Term::int(1));
        let args = vec![map, str_to_atom!("test")];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Term::boolean(true)));
    }

    #[test]
    fn test_maps_is_key_2_false() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let map = map!(heap, str_to_atom!("test") => Term::int(3));
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
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let args = vec![map];

        if let Ok(value::Cons { head, tail }) =
            bif_maps_keys_1(&vm, &process, &args).unwrap().try_into()
        {
            unsafe {
                let key1 = head;
                assert_eq!(key1, &str_to_atom!("test"));
                if let Ok(value::Cons { head, .. }) = tail.try_into() {
                    let key2 = head;
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
        let heap = &process.context_mut().heap;

        let map1 =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let map2 =
            map!(heap, str_to_atom!("test") => Term::int(3), str_to_atom!("test3") => Term::int(4));
        let args = vec![map1.clone(), map2.clone()];

        let res = bif_maps_merge_2(&vm, &process, &args);
        if let Ok(value::Map { map, .. }) = res.unwrap().try_into() {
            assert_eq!(map.len(), 3);
            assert_eq!(map.find(&str_to_atom!("test")), Some(&Term::int(1)));
            assert_eq!(map.find(&str_to_atom!("test2")), Some(&Term::int(2)));
            assert_eq!(map.find(&str_to_atom!("test3")), Some(&Term::int(4)));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_merge_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
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
        let heap = &process.context_mut().heap;

        let key = str_to_atom!("test");

        let value = Term::int(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![Term::map(heap, map), key.clone(), value.clone()];

        let res = bif_maps_put_3(&vm, &process, &args);

        if let Ok(value::Map { map, .. }) = res.unwrap().try_into() {
            assert_eq!(map.find(&key), Some(&value));
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
        let heap = &process.context_mut().heap;

        let key = str_to_atom!("test");
        let map = map!(heap, key.clone() => Term::int(1));
        let args = vec![map, key.clone()];

        let res = bif_maps_remove_2(&vm, &process, &args);

        if let Ok(value::Map { map, .. }) = res.unwrap().try_into() {
            assert_eq!(map.find(&key).is_none(), true);
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
        let heap = &process.context_mut().heap;

        let key = str_to_atom!("test");
        let value = Term::int(1);
        let update_value = Term::int(2);
        let map = map!(heap, key.clone() => value.clone());
        let args = vec![map, key.clone(), update_value.clone()];

        let res = bif_maps_update_3(&vm, &process, &args);

        if let Ok(value::Map { map, .. }) = res.unwrap().try_into() {
            assert_eq!(map.find(&key), Some(&update_value));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_maps_update_3_bad_key() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();
        let heap = &process.context_mut().heap;

        let key = str_to_atom!("test");
        let value = Term::int(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![Term::map(heap, map), key.clone(), value.clone()];

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
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let args = vec![map];

        if let Ok(value::Cons { head, tail }) =
            bif_maps_values_1(&vm, &process, &args).unwrap().try_into()
        {
            unsafe {
                let key1 = head;
                assert_eq!(key1, &Term::int(1));
                if let Ok(value::Cons { head, .. }) = tail.try_into() {
                    let key2 = head;
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
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
        let key = str_to_atom!("test2");
        let args = vec![key.clone(), map.clone()];

        let res = bif_maps_take_2(&vm, &process, &args);
        if let Ok(tuple) = res.unwrap().try_into() {
            let tuple: &value::Tuple = tuple; // annoying, need type annotation
            let mut iter = tuple.iter();
            assert_eq!(Term::int(2), iter.next().unwrap().clone());
            if let Ok(value::Map { map, .. }) = iter.next().unwrap().try_into() {
                assert_eq!(map.len(), 1);
                assert_eq!(map.find(&str_to_atom!("test")), Some(&Term::int(1)));
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
        let heap = &process.context_mut().heap;

        let map =
            map!(heap, str_to_atom!("test") => Term::int(1), str_to_atom!("test2") => Term::int(2));
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
