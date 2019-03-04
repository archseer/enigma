pub mod arith;

#[macro_export]
macro_rules! tup {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(tup!(@single $rest)),*]));

    ($heap:expr, $($value:expr,)+) => { tup!($heap, $($value),+) };
    ($heap:expr, $($value:expr),*) => {
        {
            let _cap = tup!(@count $($value),*);
            let _tuple = value::tuple($heap, _cap as u32);
            let mut _i = 0usize;
            unsafe {
                $(
                    std::ptr::write(&mut _tuple[_i], $value);
                    _i += 1usize;
                )*
            }
            Term::from(_tuple)
        }

    };
    ($heap:expr, $element1:expr, $element2:expr, $element3:expr, $element4:expr) => {{
        let tuple = value::tuple($heap, 4);
        unsafe {
            std::ptr::write(&mut tuple[0], $element1);
            std::ptr::write(&mut tuple[1], $element2);
            std::ptr::write(&mut tuple[2], $element3);
            std::ptr::write(&mut tuple[3], $element4);
        }
        Term::from(tuple)
    }};
}

#[macro_export]
macro_rules! tup2 {
    ($heap:expr, $element1:expr, $element2:expr) => {{
        let tuple = value::tuple($heap, 2);
        unsafe {
            std::ptr::write(&mut tuple[0], $element1);
            std::ptr::write(&mut tuple[1], $element2);
        }
        Term::from(tuple)
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
        Term::from(tuple)
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
        Term::from(tuple)
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
        let mut list = Term::nil();
        for char in $str.bytes().rev() {
            list = cons!($heap, Term::int(i32::from(char)), list);
        }
        list
    }};
}

#[macro_export]
macro_rules! atom {
    ($const:ident) => {
        Term::atom(atom::$const)
    };
}

#[macro_export]
macro_rules! str_to_atom {
    ($str:expr) => {
        Term::atom(atom::from_str($str))
    };
}

// based off of maplit: https://github.com/bluss/maplit/blob/master/src/lib.rs
#[macro_export]
macro_rules! map {
    // (@single $($x:tt)*) => (());
    //(@count $($rest:expr),*) => (<[()]>::len(&[$(map!(@single $rest)),*]));

    ($heap:expr, $($key:expr => $value:expr,)+) => { map!($heap, $($key => $value),+) };
    ($heap:expr, $($key:expr => $value:expr),*) => {
        {
            // let _cap = map!(@count $($key),*);
            // let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            let mut _map = value::HAMT::new();
            $(
                _map = _map.plus($key, $value);
            )*
            Term::map($heap, _map)
        }
    };
}

#[macro_export]
macro_rules! iter_to_list {
    ($heap: expr, $iter:expr) => {
        $iter.fold(Term::nil(), |acc, val| cons!($heap, val, acc))
    };
}
