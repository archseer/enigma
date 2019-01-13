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

