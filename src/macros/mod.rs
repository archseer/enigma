pub mod arith;

#[macro_export]
macro_rules! tup2 {
    ($heap:expr, $element1:expr, $element2:expr) => {{
        let tuple = value::tuple($heap, 2);
        tuple[0] = $element1;
        tuple[1] = $element2;
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! tup3 {
    ($heap:expr, $element1:expr, $element2:expr, $element3:expr) => {{
        let tuple = value::tuple($heap, 3);
        tuple[0] = $element1;
        tuple[1] = $element2;
        tuple[2] = $element3;
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! tup4 {
    ($heap:expr, $element1:expr, $element2:expr, $element3:expr, $element4:expr) => {{
        let tuple = value::tuple($heap, 4);
        tuple[0] = $element1;
        tuple[1] = $element2;
        tuple[2] = $element3;
        tuple[3] = $element4;
        Value::Tuple(tuple)
    }};
}

#[macro_export]
macro_rules! cons {
    ($heap:expr, $head:expr, $tail:expr) => {{
        value::cons($heap, $head, $tail)
    }};
}
