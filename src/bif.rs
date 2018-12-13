use crate::atom;
use crate::module;
use crate::value::Value;
use once_cell::sync::Lazy;
use std::collections::HashMap;

type BifFn = fn(Vec<&Value>) -> Value;
type BifTable = HashMap<(&'static str, &'static str, u32), Box<BifFn>>;

static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs = BifTable::new();
    bifs.insert(("erlang", "+", 2), Box::new(bif_erlang_add_2));
    bifs.insert(("erlang", "-", 2), Box::new(bif_erlang_sub_2));
    bifs
};

pub fn apply(mfa: &module::MFA, args: Vec<&Value>) -> Value {
    (BIFS
        .get(&(
            &atom::from_index(&mfa.0).unwrap(),
            &atom::from_index(&mfa.1).unwrap(),
            mfa.2,
        ))
        .unwrap())(args)
}

// let val: Vec<_> = module
//     .imports
//     .iter()
//     .map(|mfa| {
//         (
//             atom::from_index(&mfa.0).unwrap(),
//             atom::from_index(&mfa.1).unwrap(),
//             mfa.2,
//         )
//     })
//     .collect();

/// Bif implementations
#[inline]
fn bif_erlang_add_2(args: Vec<&Value>) -> Value {
    if let [Value::Integer(v1), Value::Integer(v2)] = &args[..] {
        return Value::Integer(v1 + v2);
    }
    panic!("Invalid arguments to erlang::+")
}

#[inline]
fn bif_erlang_sub_2(args: Vec<&Value>) -> Value {
    if let [Value::Integer(v1), Value::Integer(v2)] = &args[..] {
        return Value::Integer(v1 - v2);
    }
    panic!("Invalid arguments to erlang::+")
}
