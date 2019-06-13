use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::RcProcess;
use crate::value::{self, CastFrom, CastInto, Cons, Term, Tuple};
use crate::vm;

pub fn member_2(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    // need to bump reductions as we go
    let reds_left = 1; // read from process
    let mut max_iter = 16 * reds_left;
    // bool non_immed_key;

    if args[1].is_nil() {
        return Ok(atom!(FALSE));
    } else if !args[1].is_list() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let term = &args[0];
    // non_immed_key = is_not_immed(term);
    let mut list = &args[1];

    while let Ok(Cons { head, tail }) = list.cast_into() {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(BIF_P);
            // BIF_TRAP2(bif_export[member_2], BIF_P, term, list);
            // TODO: ^ trap schedules the process to continue executing (by storing the temp val
            // and passing it in the bif call)
        }

        if *head == *term {
            // || (non_immed_key && deep_equals) {
            // BIF_RET2(am_true, reds_left - max_iter/16);
            return Ok(atom!(TRUE));
        }
        list = tail;
    }

    if !list.is_list() {
        // BUMP_REDS(BIF_P, reds_left - max_iter/16);
        return Err(Exception::new(Reason::EXC_BADARG));
    }
    Ok(atom!(FALSE)) // , reds_left - max_iter/16
}

pub fn reverse_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    // Handle legal and illegal non-lists quickly.
    if args[0].is_nil() {
        return Ok(args[1]);
    }

    let cons = Cons::cast_from(&args[0])?;

    let heap = &process.context_mut().heap;
    // TODO: finish up the reduction counting implementation
    Ok(cons.iter().fold(args[1], |acc, val| cons!(heap, *val, acc)))
}

pub fn keymember_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    keyfind(keyfind_3, process, args).map(|res| {
        if res.is_tuple() {
            return atom!(TRUE);
        }
        res
    })
}

pub fn keysearch_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    keyfind(keyfind_3, process, args).map(|res| {
        if res.is_tuple() {
            let heap = &process.context_mut().heap;
            return tup2!(heap, atom!(VALUE), res);
        }
        res
    })
}

pub fn keyfind_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> bif::Result {
    keyfind(keyfind_3, process, args)
}

/// Swap process out after this number
const CONTEXT_REDS: usize = 4000;

fn keyfind(_func: bif::Fn, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let mut max_iter: isize = 10 * CONTEXT_REDS as isize;

    let key = args[0];
    let mut list = &args[2];

    let pos = match args[1].into_number() {
        Ok(value::Num::Integer(i)) if !i < 1 => i as usize - 1, // it's always 1-indexed
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    // OTP does 3 different loops based on key type (simple, immed, boxed), but luckily in rust we
    // just rely on Eq/PartialEq.

    while let Ok(Cons { head, tail }) = list.cast_into() {
        max_iter -= 1;
        if max_iter < 0 {
            // BUMP_ALL_REDS(p);
            // BIF_TRAP3(bif_export[Bif], p, key, pos_val, list);
        }

        let term = head;
        list = tail;
        if let Ok(tuple) = Tuple::cast_from(&term) {
            if pos <= (tuple.len as usize) && key == tuple[pos] {
                return Ok(*term);
            }
        }
    }

    if !list.is_nil() {
        // BIF_ERROR(p, BADARG);
    }
    Ok(atom!(FALSE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immix::Heap;
    use crate::module;
    use crate::process;

    /// Converts an erlang list to a value vector.
    fn to_vec(value: Term) -> Vec<Term> {
        let mut vec = Vec::new();
        let mut cons = &value;
        while let Ok(Cons { head, tail }) = cons.cast_into() {
            vec.push(*head);
            cons = &tail;
        }
        // lastly, the tail
        vec.push(*cons);
        vec
    }

    /// Converts a value vector to an erlang list.
    fn from_vec(heap: &Heap, vec: Vec<Term>) -> Term {
        vec.into_iter()
            .rev()
            .fold(Term::nil(), |res, val| value::cons(heap, val, res))
    }

    #[test]
    fn test_member_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Term::atom(1);
        let list = from_vec(heap, vec![Term::atom(3), Term::atom(2)]);
        let res = member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Term::atom(1);
        let list = from_vec(heap, vec![Term::atom(3), Term::atom(2), Term::atom(1)]);
        let res = member_2(&vm, &process, &[elem, list]);
        assert_eq!(res, Ok(atom!(TRUE)));
    }

    #[test]
    fn test_keyfind_3() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm, 0, 0, module).unwrap();
        let heap = &process.context_mut().heap;

        let elem = Term::atom(1);
        let pos = Term::int(5);
        let list = from_vec(heap, vec![]);
        let res = keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(atom!(FALSE)));

        let elem = Term::atom(3);
        let pos = Term::int(1);
        let target = tup2!(heap, Term::atom(3), Term::int(2));
        let list = from_vec(
            heap,
            vec![
                tup2!(heap, Term::atom(1), Term::int(4)),
                tup2!(heap, Term::atom(2), Term::int(3)),
                target,
                tup2!(heap, Term::atom(4), Term::int(1)),
            ],
        );
        let res = keyfind_3(&vm, &process, &[elem, pos, list]);
        assert_eq!(res, Ok(target));
    }
}
