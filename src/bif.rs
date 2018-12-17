use crate::atom;
use crate::module;
use crate::process::{self, RcProcess};
use crate::value::Value;
use crate::vm;
use fnv::FnvHashMap;
use once_cell::sync::Lazy;

type BifFn = fn(&vm::Machine, &RcProcess, Vec<&Value>) -> Value;
type BifTable = FnvHashMap<(usize, usize, usize), Box<BifFn>>;

static BIFS: Lazy<BifTable> = sync_lazy! {
    let mut bifs: BifTable = FnvHashMap::default();
    let erlang = atom::i_from_str("erlang");
    bifs.insert((erlang, atom::i_from_str("+"), 2), Box::new(bif_erlang_add_2));
    bifs.insert((erlang, atom::i_from_str("-"), 2), Box::new(bif_erlang_sub_2));
    bifs.insert((erlang, atom::i_from_str("spawn"), 3), Box::new(bif_erlang_spawn_3));
    bifs
};

#[inline]
pub fn apply(vm: &vm::Machine, process: &RcProcess, mfa: &module::MFA, args: Vec<&Value>) -> Value {
    (BIFS.get(mfa).unwrap())(vm, process, args)
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
fn bif_erlang_spawn_3(vm: &vm::Machine, process: &RcProcess, args: Vec<&Value>) -> Value {
    // parent: TODO: track parent of process
    // arg[0] = atom for module
    // arg[1] = atom for function
    // arg[2] = arguments for func (well-formed list)
    // opts, options for spawn

    if let [Value::Atom(module), Value::Atom(func), args] = args[..] {
        let registry = vm.modules.lock().unwrap();
        let module = registry.lookup(*module).unwrap();
        process::spawn(&vm.state, module, *func, args.clone()).unwrap();
    }
    panic!("Invalid arguments to erlang::+")
}

#[inline]
fn bif_erlang_add_2(_vm: &vm::Machine, _process: &RcProcess, args: Vec<&Value>) -> Value {
    if let [Value::Integer(v1), Value::Integer(v2)] = &args[..] {
        return Value::Integer(v1 + v2);
    }
    panic!("Invalid arguments to erlang::+")
}

#[inline]
fn bif_erlang_sub_2(_vm: &vm::Machine, _process: &RcProcess, args: Vec<&Value>) -> Value {
    if let [Value::Integer(v1), Value::Integer(v2)] = &args[..] {
        return Value::Integer(v1 - v2);
    }
    panic!("Invalid arguments to erlang::-")
}
