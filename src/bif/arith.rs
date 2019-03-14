use crate::bif;
use crate::exception::{Exception, Reason};
use crate::numeric::division::{FlooredDiv, OverflowingFlooredDiv};
use crate::numeric::modulo::{Modulo, OverflowingModulo};
use crate::process::Process;
use crate::value::{self, Term, TryInto};
use crate::vm;
use num::bigint::BigInt;
use std::pin::Pin;
// use num::bigint::ToBigInt;
use num::traits::Signed;
use statrs;
use std::ops::{Add, Mul, Sub};

pub fn abs_1(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    match &args[0].into_number() {
        Ok(value::Num::Integer(i)) => Ok(Term::int(i.abs())),
        Ok(value::Num::Float(f)) => Ok(Term::from(f.abs())),
        Ok(value::Num::Bignum(i)) => Ok(Term::bigint(heap, i.abs())),
        _ => Err(Exception::new(Reason::EXC_BADARG)),
    }
}

pub fn add_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, add, overflowing_add))
}

pub fn sub_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, sub, overflowing_sub))
}

pub fn mult_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, mul, overflowing_mul))
}

pub fn intdiv_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(
        heap,
        args,
        floored_division,
        overflowing_floored_division
    ))
}

pub fn mod_2(_vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    // TODO: should be rem but it's mod
    let heap = &process.context_mut().heap;
    Ok(integer_overflow_op!(heap, args, modulo, overflowing_modulo))
}

macro_rules! trig_func {
    (
    $arg:expr,
    $op:ident
) => {{
        let res = match $arg.into_number() {
            Ok(value::Num::Integer(i)) => f64::from(i),
            Ok(value::Num::Float(f)) => f,
            Ok(value::Num::Bignum(..)) => unimplemented!(),
            Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
        };
        Ok(Term::from(res.$op()))
    }};
}

pub fn math_cos_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], cos)
}

pub fn math_cosh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], cosh)
}

pub fn math_sin_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], sin)
}

pub fn math_sinh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], sinh)
}

pub fn math_tan_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], tan)
}

pub fn math_tanh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], tanh)
}

pub fn math_acos_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], acos)
}

pub fn math_acosh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], acosh)
}

pub fn math_asin_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], asin)
}

pub fn math_asinh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], asinh)
}

pub fn math_atan_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], atan)
}

pub fn math_atanh_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], atanh)
}

pub fn math_erf_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(statrs::function::erf::erf(res)))
}

pub fn math_erfc_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(1.0_f64 - statrs::function::erf::erf(res)))
}

pub fn math_exp_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let res: f64 = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(res.powf(std::f64::consts::E)))
}

pub fn math_log_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], ln)
}

pub fn math_log2_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], log2)
}

pub fn math_log10_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], log10)
}

pub fn math_sqrt_1(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    trig_func!(args[0], sqrt)
}

pub fn math_atan2_2(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let arg = match args[1].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::from(res.atan2(arg)))
}

pub fn math_pow_2(_vm: &vm::Machine, _process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let base = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    let index = match args[1].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    Ok(Term::from(base.powf(index)))
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
        while let Ok(value::Cons { head, tail }) = cons.try_into() {
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
    fn test_abs_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(-1)];
        let res = abs_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_add_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(1), Term::int(2)];
        let res = add_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(3)));
    }

    #[test]
    fn test_sub_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(2), Term::int(1)];
        let res = sub_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    #[test]
    fn test_mult_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(2), Term::int(4)];
        let res = mult_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(8)));
    }

    #[test]
    fn test_intdiv_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(8), Term::int(4)];
        let res = intdiv_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(2)));
    }

    #[test]
    fn test_mod_2() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(4), Term::int(3)];
        let res = mod_2(&vm, &process, &args);
        assert_eq!(res, Ok(Term::int(1)));
    }

    // trig

    #[test]
    fn test_math_cos_1() {
        let vm = vm::Machine::new();
        let module: *const module::Module = std::ptr::null();
        let process = process::allocate(&vm.state, 0, module).unwrap();
        let args = vec![Term::int(1)];
        let res = math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::from(1.0_f64.cos())));

        let args = vec![Term::from(1.0)];
        let res = math_cos_1(&vm, &process, &args);
        assert_eq!(res, Ok(Term::from(1.0_f64.cos())));
    }

    // TODO: test rest of math funcs

}
