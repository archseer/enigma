//use crate::servo_arc::Arc;
use crate::value::{Cons, Variant};
use crate::{atom, bitstring};
use regex::bytes::{Regex, RegexBuilder};

// pub mod error;
// use std::error::Error;
// pub use error::Result;

pub mod bif {
    use super::*;
    use crate::bif::Result;
    use crate::process::RcProcess;
    use crate::value::{self, CastFrom, Term};
    use crate::vm;

    pub fn version_0(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        // TODO: static regex version for now
        let version = "1.1.7";
        Ok(Term::binary(
            &process.context_mut().heap,
            bitstring::Binary::from(version.as_bytes().to_owned()),
        ))
    }

    pub fn run_2(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        run_3(vm, process, &[args[0], args[1], Term::nil()])
    }

    pub fn run_3(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        use std::borrow::Cow;
        let heap = &process.context_mut().heap;
        // println!("run/3: {} {}", args[0], args[1]);
        let string = crate::bif::erlang::list_to_iodata(args[0]).unwrap(); // TODO: error handling

        let regex = match args[1].get_boxed_header() {
            Ok(value::BOXED_REGEX) => {
                let regex = regex::bytes::Regex::cast_from(&args[1]).unwrap();
                Cow::Borrowed(regex)
            }
            _ => {
                let pattern = crate::bif::erlang::list_to_iodata(args[1]).unwrap(); // TODO: error handling

                // TODO verify args
                let regex = Regex::new(std::str::from_utf8(&pattern).unwrap()).unwrap();
                Cow::Owned(regex)
            }
        };

        //println!("inspect {:?} -- {:?}", string, pattern);
        // str = [115, 117, 112, 101, 114, 118, 105, 115, 111, 114, 58, 32, 123, 108, 111, 99, 97, 108, 44, 107, 101, 114, 110, 101, 108, 95, 115, 117, 112, 125, 10, 32, 32, 32, 32, 115, 116, 97, 114, 116, 101, 100, 58, 32, 91, 123, 112, 105, 100, 44, 60, 48, 46, 48, 46, 52, 52, 62, 125, 44, 123, 105, 100, 44, 99, 111, 100, 101, 95, 115, 101, 114, 118, 101, 114, 125, 44, 123, 109, 102, 97, 114, 103, 115, 44, 123, 99, 111, 100, 101, 44, 115, 116, 97, 114, 116, 95, 108, 105, 110, 107, 44, 91, 93, 125, 125, 44, 123, 114, 101, 115, 116, 97, 114, 116, 95, 116, 121, 112, 101, 44, 112, 101, 114, 109, 97, 110, 101, 110, 116, 125, 44, 123, 115, 104, 117, 116, 100, 111, 119, 110, 44, 50, 48, 48, 48, 125, 44, 123, 99, 104, 105, 108, 100, 95, 116, 121, 112, 101, 44, 119, 111, 114, 107, 101, 114, 125, 93]
        // pat = [44, 63, 13, 63, 10, 32, 42]
        // iex(5)> :re.run(str, pat, [:unicode, :global])
        // {:match, [[{30, 5}]]}

        let res = regex.find_iter(&string).fold(Term::nil(), |acc, m| {
            cons!(
                heap,
                tup2!(
                    heap,
                    Term::uint64(heap, m.start() as u64),
                    Term::uint64(heap, (m.end() - m.start()) as u64)
                ),
                acc
            )
        });

        // println!("re_run_3 inputs {:?} and {:?}", string, pattern);
        // println!("re_run_3 res {}", res);

        if !res.is_nil() {
            Ok(tup2!(heap, atom!(MATCH), cons!(heap, res, Term::nil())))
        } else {
            Ok(atom!(NOMATCH))
        }
    }

    pub fn compile_1(vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        compile_2(vm, process, &[args[0], args[1], Term::nil()])
    }

    pub fn compile_2(_vm: &vm::Machine, process: &RcProcess, args: &[Term]) -> Result {
        let heap = &process.context_mut().heap;

        let pattern = crate::bif::erlang::list_to_iodata(args[0]).unwrap(); // TODO: error handling

        // this is terrible, but the syntax is incompatible
        let pattern = String::from_utf8(pattern).unwrap().replace("(?<", "(?P<");

        let mut builder = RegexBuilder::new(&pattern);
        let mut unicode = false;

        if !args[1].is_list() {
            for value in Cons::cast_from(&args[1])?.iter() {
                match value.into_variant() {
                    Variant::Atom(atom::CASELESS) => builder.case_insensitive(true),
                    Variant::Atom(atom::UNICODE) => {
                        unicode = true;
                        builder.unicode(true)
                    }
                    Variant::Atom(atom::UNGREEDY) => builder.swap_greed(true),
                    Variant::Atom(atom::MULTILINE) => builder.multi_line(true),
                    Variant::Atom(atom::DOTALL) => builder.dot_matches_new_line(true),
                    _ => unimplemented!("{}", args[1]),
                };
            }
        }

        // TODO verify args
        let regex = builder.build().unwrap();

        Ok(tup!(
            heap,
            atom!(RE_PATTERN),
            Term::uint64(heap, regex.captures_len() as u64),
            Term::int(if unicode { 1 } else { 0 }),
            Term::int(0), // TODO: use_crlf
            Term::regex(heap, regex)
        ))
    }

}
