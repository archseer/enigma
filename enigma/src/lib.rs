// The nightly features that are commonly needed with async / await
#![feature(async_await, alloc_layout_extra)]
#![feature(arbitrary_self_types)]
#![recursion_limit = "1024"] // yeowch

#[macro_use]
mod macros;
#[macro_use]
pub mod exception;
#[macro_use]
pub mod vm;
#[macro_use]
pub mod nanbox;
pub mod atom;
mod bif;
pub mod bitstring;
pub mod etf;
pub mod ets;
pub mod exports_table;
mod immix;
mod instr_ptr;
pub mod instruction;
pub mod instruction2;
pub mod loader;
pub mod mailbox;
pub mod module;
pub mod module_registry;
mod numeric;
pub mod opcodes;
pub mod persistent_term;
pub mod port;
pub mod process;
pub mod regex;
pub mod servo_arc;
pub mod signal_queue;
pub mod value;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

// extracted from itertools
trait Itertools: Iterator {
    #[inline]
    fn fold_results<A, E, B, F>(&mut self, mut start: B, mut f: F) -> Result<B, E>
    where
        Self: Iterator<Item = Result<A, E>>,
        F: FnMut(B, A) -> B,
    {
        for elt in self {
            match elt {
                Ok(v) => start = f(start, v),
                Err(u) => return Err(u),
            }
        }
        Ok(start)
    }
}

impl<T: ?Sized> Itertools for T where T: Iterator {}
