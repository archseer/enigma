#![macro_use]

macro_rules! to_expr {
    ($e:expr) => {
        $e
    };
}

/// Performs an integer binary operation that may overflow into a bigint.
///
/// This macro takes the following arguments:
///
/// * `$process`: the process that is performing the operation.
/// * `$context`: the current ExecutionContext.
/// * `$op`: the binary operator to use for non overflowing operations.
/// * `$overflow`: the method to use for an overflowing operation.
#[macro_export]
macro_rules! integer_overflow_op {
    (
        $heap:expr,
        $args:expr,
        $op:ident,
        $overflow:ident
    ) => {{
        // TODO: figure out if we can reduce amount of cloning here.
        match [$args[0].into_variant(), $args[1].into_variant()] {
            [Variant::Integer(rec), Variant::Integer(arg)] => {
                // Example: int + int -> int
                //
                // This will produce a bigint if the produced integer overflowed or
                // doesn't fit in a tagged pointer.

                let (result, overflowed) = rec.$overflow(arg);

                if overflowed {
                    // If the operation overflowed we need to retry it but using
                    // big integers.
                    let result = to_expr!(BigInt::from(rec).$op(BigInt::from(arg)));

                    Term::BigInt(Box::new(result))
                // $heap.allocate(object_value::bigint(result))
                // } else if ObjectPointer::integer_too_large(result) {
                //     // An operation that doesn't overflow may still produce a number
                //     // too large to store in a tagged pointer. In this case we'll
                //     // allocate the result as a heap integer.
                //     $heap.allocate(object_value::integer(result))
                } else {
                    Term::int(result)
                }
            }
            [Term::BigInt(rec), Variant::Integer(arg)] => {
                // Example: bigint + int -> bigint

                let rec = rec.clone();

                // in i32 range
                let bigint = if arg >= i64::from(i32::MIN) && arg <= i64::from(i32::MAX) {
                    to_expr!(rec.$op(arg as i32))
                } else {
                    to_expr!(rec.$op(BigInt::from(arg)))
                };

                Term::BigInt(Box::new(bigint))
                // $heap.allocate(object_value::bigint(bigint))
            }
            [Variant::Integer(rec), Term::BigInt(arg)] => {
                // Example: int + bigint -> bigint

                let rec = BigInt::from(rec);
                let bigint = to_expr!(rec.$op(*arg.clone()));

                Term::BigInt(Box::new(bigint))
                // $heap.allocate(object_value::bigint(bigint))
            }
            [Term::BigInt(rec), Term::BigInt(arg)] => {
                // Example: bigint + bigint -> bigint

                let bigint = to_expr!(rec.clone().$op(arg.clone()));

                Term::BigInt(Box::new(bigint))
                // $heap.allocate(object_value::bigint(bigint))
            }
            _ => {
                return Err(Exception::new(Reason::EXC_BADARG));
            }
        }
    }};
}
