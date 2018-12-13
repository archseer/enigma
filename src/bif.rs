use crate::atom;
use crate::module;
use crate::value::Value;
use fnv::FnvHashMap;
use once_cell::sync::Lazy;

type BifFn = fn(Vec<&Value>) -> Value;
type BifTable = FnvHashMap<(usize, usize, u32), Box<BifFn>>;

static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs: BifTable = FnvHashMap::default();
    let erlang = atom::i_from_str("erlang");
    bifs.insert((erlang, atom::i_from_str("+"), 2), Box::new(bif_erlang_add_2));
    bifs.insert((erlang, atom::i_from_str("-"), 2), Box::new(bif_erlang_sub_2));
    bifs
};

#[inline]
pub fn apply(mfa: &module::MFA, args: Vec<&Value>) -> Value {
    (BIFS.get(mfa).unwrap())(args)
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
    panic!("Invalid arguments to erlang::-")
}
