pub mod arith;

#[macro_export]
macro_rules! tup2 {
    ($heap:expr, $element1:expr, $element2:expr) => {{
        let tuple = value::tuple($heap, 2);
        unsafe {
            std::ptr::write(&mut tuple[0], $element1);
            std::ptr::write(&mut tuple[1], $element2);
        }
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! tup3 {
    ($heap:expr, $element1:expr, $element2:expr, $element3:expr) => {{
        let tuple = value::tuple($heap, 3);
        // use ptr write to avoid dropping uninitialized values!
        unsafe {
            std::ptr::write(&mut tuple[0], $element1);
            std::ptr::write(&mut tuple[1], $element2);
            std::ptr::write(&mut tuple[2], $element3);
        }
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! tup4 {
    ($heap:expr, $element1:expr, $element2:expr, $element3:expr, $element4:expr) => {{
        let tuple = value::tuple($heap, 4);
        unsafe {
            std::ptr::write(&mut tuple[0], $element1);
            std::ptr::write(&mut tuple[1], $element2);
            std::ptr::write(&mut tuple[2], $element3);
            std::ptr::write(&mut tuple[3], $element4);
        }
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! cons {
    ($heap:expr, $head:expr, $tail:expr) => {{
        value::cons($heap, $head, $tail)
    }};
}

#[macro_export]
macro_rules! bitstring {
    ($heap:expr, $str:expr) => {{
        let mut list = Value::Nil;
        for char in $str.bytes().rev() {
            list = cons!($heap, Value::Character(char), list);
        }
        list
    }};
}

#[macro_export]
macro_rules! atom {
    ($const:ident) => {
        Value::Atom(atom::$const)
    };
}

#[macro_export]
macro_rules! str_to_atom {
    ($str:expr) => {
        Value::Atom(atom::from_str($str))
    };
}

// based off of maplit: https://github.com/bluss/maplit/blob/master/src/lib.rs
#[macro_export]
macro_rules! map {
    (@single $($x:tt)*) => (());
    //(@count $($rest:expr),*) => (<[()]>::len(&[$(map!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { map!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            // let _cap = map!(@count $($key),*);
            // let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            let mut _map = value::HAMT::new();
            $(
                _map = _map.plus($key, $value);
            )*
            Value::Map(value::Map(Arc::new(_map)))
        }
    };
}

#[macro_export]
macro_rules! iter_to_list {
    ($heap: expr, $iter:expr) => {
        $iter.fold(Value::Nil, |res, val| value::cons($heap, val, res))
    }
}
