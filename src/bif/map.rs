use crate::atom;
use crate::immix::Heap;
use crate::bif::BifResult;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, Value};
use crate::servo_arc::Arc;
use crate::vm;
use hamt_rs::HamtMap;

pub fn bif_maps_find_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let key = &args[0];
    let map = &args[1];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        match hamt_map.find(key) {
            Some(value) => {
                let heap = &process.context_mut().heap;
                return Ok(tup2!(&heap, atom!(OK), value.clone()));
            },
            _ => {
                return Ok(atom!(ERROR));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_get_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        match hamt_map.find(target) {
            Some(value) => {
                return Ok(value.clone());
            },
            _ => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, target.clone()));
            }
        };
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_from_list_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_is_key_2(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let target = &args[1];
        let exist = hamt_map.find(target).is_some();
        return Ok(Value::boolean(exist))
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let heap = &process.context_mut().heap;
        let mut vec = vec![];
        for (key, _) in hamt_map.iter() {
            vec.push(key.clone());
        }
        let list = vec_to_list!(heap, vec);
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_merge_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    unimplemented!();
}

pub fn bif_maps_put_3(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Value::Map(m) = map {
        let map = &(*m.0);
        let new_map = map.clone().plus(key.clone(), value.clone());
        return Ok(Value::Map(value::Map(Arc::new(new_map))));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_remove_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    let key = &args[1];
    if let Value::Map(m) = map {
        let map = &(*m.0);
        let new_map = map.clone().minus(&key.clone());
        return Ok(Value::Map(value::Map(Arc::new(new_map))));
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}


pub fn bif_maps_update_3(_vm: &vm::Machine, _process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    let key = &args[1];
    let value = &args[2];
    if let Value::Map(m) = map {
        let map = &(*m.0);
        match map.find(&key) {
            Some(_v) => {
                let new_map = map.clone().plus(key.clone(), value.clone());
                return Ok(Value::Map(value::Map(Arc::new(new_map))));
            },
            _ => {
                return Err(Exception::with_value(Reason::EXC_BADKEY, key.clone()));
            }
        }
        let new_map = map.clone().plus(key.clone(), value.clone());
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

pub fn bif_maps_values_1(_vm: &vm::Machine, process: &RcProcess, args: &[Value]) -> BifResult {
    let map = &args[0];
    if let Value::Map(m) = map {
        let hamt_map = &m.0;
        let heap = &process.context_mut().heap;
        let mut vec = vec![];
        for (_, value) in hamt_map.iter() {
            vec.push(value.clone());
        }
        let list = vec_to_list!(heap, vec);
        return Ok(list);
    }
    Err(Exception::with_value(Reason::EXC_BADMAP, map.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom;
    use crate::process::{self};
    use crate::module;

    #[test]
    fn test_maps_find_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");
        let map = map!(key.clone() => Value::Integer(3));
        let args = vec![key.clone(), map];

        let res = bif_maps_find_2(&vm, &process, &args);

        let heap = &Heap::new();
        if let Ok(Value::Tuple(tuple)) = res {
            unsafe {
                assert_eq!((*tuple).len, 2);
                let slice: &[Value] = &(**tuple);
                let mut iter = slice.iter();
                assert_eq!(iter.next(), Some(&atom!(OK)));
                assert_eq!(iter.next(), Some(&Value::Integer(3)));
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
        let map = map!(key.clone() => Value::Integer(3));
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

        let map = map!(str_to_atom!("test") => Value::Integer(3));
        let args = vec![map, str_to_atom!("test")];

        let res = bif_maps_get_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_maps_get_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Value::Integer(3);
        let args = vec![bad_map.clone(), Value::Atom(atom::from_str("test"))];

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

        let map = map!(str_to_atom!("test") => Value::Integer(3));
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
        unimplemented!();
    }

    #[test]
    fn test_maps_is_key_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Value::Integer(1));
        let args = vec![map, str_to_atom!("test")];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::boolean(true)));
    }

    #[test]
    fn test_maps_is_key_2_false() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let map = map!(str_to_atom!("test") => Value::Integer(3));
        let args = vec![map, str_to_atom!("false")];

        let res = bif_maps_is_key_2(&vm, &process, &args);

        assert_eq!(res, Ok(Value::boolean(false)));
    }

    #[test]
    fn test_maps_is_key_2_bad_map() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let bad_map = Value::Integer(3);
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

        let map = map!(str_to_atom!("test") => Value::Integer(1), str_to_atom!("test2") => Value::Integer(2));
        let args = vec![map];

        if let Ok(Value::List(cons)) = bif_maps_keys_1(&vm, &process, &args) {
            unsafe {
                let key1 = &(*cons).head;
                assert_eq!(key1, &str_to_atom!("test"));
                if let Value::List(tail) = (*cons).tail {
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

        let bad_map = Value::Integer(3);
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
        unimplemented!();
    }

    #[test]
    fn test_maps_put_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, module).unwrap();

        let key = str_to_atom!("test");

        let value = Value::Integer(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![Value::Map(value::Map(Arc::new(map))), key.clone(), value.clone()];

        let res = bif_maps_put_3(&vm, &process, &args);

        if let Ok(Value::Map(body)) = res {
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
        let value = Value::Integer(2);
        let bad_map = Value::Integer(3);
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
        let map = map!(key.clone() => Value::Integer(1));
        let args = vec![map, key.clone()];

        let res = bif_maps_remove_2(&vm, &process, &args);

        if let Ok(Value::Map(body)) = res {
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

        let args = vec![Value::Integer(1), Value::Integer(2)];

        let res = bif_maps_remove_2(&vm, &process, &args);

        if let Err(exception) = res {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, Value::Integer(1));
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
        let value = Value::Integer(1);
        let update_value = Value::Integer(2);
        let map = map!(key.clone() => value.clone());
        let args = vec![map, key.clone(), update_value.clone()];

        let res = bif_maps_update_3(&vm, &process, &args);

        if let Ok(Value::Map(body)) = res {
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
        let value = Value::Integer(2);
        let map: value::HAMT = HamtMap::new();
        let args = vec![Value::Map(value::Map(Arc::new(map))), key.clone(), value.clone()];

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
        let value = Value::Integer(2);
        let bad_map = Value::Integer(3);
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

        let map = map!(str_to_atom!("test") => Value::Integer(1), str_to_atom!("test2") => Value::Integer(2));
        let args = vec![map];

        if let Ok(Value::List(cons)) = bif_maps_values_1(&vm, &process, &args) {
            unsafe {
                let key1 = &(*cons).head;
                assert_eq!(key1, &Value::Integer(1));
                if let Value::List(tail) = (*cons).tail {
                    let key2 = &(*tail).head;
                    assert_eq!(key2, &Value::Integer(2));
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

        let bad_map = Value::Integer(3);
        let args = vec![bad_map.clone(), str_to_atom!("test")];

        if let Err(exception) = bif_maps_values_1(&vm, &process, &args) {
            assert_eq!(exception.reason, Reason::EXC_BADMAP);
            assert_eq!(exception.value, bad_map);
        } else {
            panic!();
        }
    }
}
