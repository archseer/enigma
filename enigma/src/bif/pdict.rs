//! Process dictionary
use crate::atom;
use crate::bif;
use crate::process::RcProcess;
use crate::value::{self, Term};
use crate::vm;

/// Get the whole pdict.
pub fn get_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict.iter().fold(Term::nil(), |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, *key, *val);

        // make cons
        cons!(heap, tuple, res)
    });
    Ok(result)
}

/// Get the value for key in pdict.
pub fn get_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let pdict = &process.local_data_mut().dictionary;
    Ok(pdict
        .get(&(args[0]))
        .cloned() // TODO: try to avoid the clone if possible
        .unwrap_or_else(|| atom!(UNDEFINED)))
}

/// Get all the keys in pdict.
pub fn get_keys_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict
        .keys()
        .fold(Term::nil(), |res, key| cons!(heap, *key, res));
    Ok(result)
}

/// Return all the keys that have val
pub fn get_keys_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let pdict = &process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    let result: Term = pdict.iter().fold(Term::nil(), |res, (key, val)| {
        if args[1] == *val {
            value::cons(heap, *key, res)
        } else {
            res
        }
    });
    Ok(result)
}

/// Set the key to val. Return undefined if a key was inserted, or old val if it was updated.
pub fn put_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict
        .insert(args[0], args[1])
        .unwrap_or_else(|| atom!(UNDEFINED)))
}

/// Remove all pdict entries, returning the pdict.
pub fn erase_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> bif::Result {
    // deletes all the entries, returning the whole dict tuple
    let pdict = &mut process.local_data_mut().dictionary;
    let heap = &process.context_mut().heap;

    // we use drain since it means we do a move instead of a copy
    let result: Term = pdict.drain().fold(Term::nil(), |res, (key, val)| {
        // make tuple
        let tuple = tup2!(heap, key, val);

        // make cons
        cons!(heap, tuple, res)
    });
    Ok(result)
}

/// Remove a single entry from the pdict and return it.
pub fn erase_1(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // deletes a single entry, returning the val
    let pdict = &mut process.local_data_mut().dictionary;
    Ok(pdict.remove(&(args[0])).unwrap_or_else(|| atom!(UNDEFINED)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::module;
    use crate::process;

    #[test]
    fn test_bif_pdict() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let args = vec![Term::atom(1), Term::int(2)];
        let res = put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Term::atom(1), Term::int(3)];
        let res = put_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(2)));

        let args = vec![Term::atom(2), Term::int(1)];
        let res = put_2(&vm, &process, &args);
        assert_eq!(res, Ok(atom!(UNDEFINED)));

        let args = vec![Term::atom(2)];
        let res = get_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));

        // TODO: add a assert helper for lists
        let args = vec![];
        let _res = get_0(&vm, &process, &args);
        // assert_eq!(res, Ok(Term::int(1)));
    }
}
