use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::{self, Process};
use crate::value::{self, Term, TryFrom, Tuple, Variant};
use crate::vm;
use std::pin::Pin;

use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Delay;

pub fn send_after_3(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    // time, dest, msg
    let delay = match args[0].to_int() {
        Some(i) if i >= 0 => i,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    if !args[1].is_pid() {
        return Err(Exception::new(Reason::EXC_BADARG));
    }

    let dest = args[1];
    let msg = args[2];
    let from = process.pid;

    let when = Instant::now() + Duration::from_millis(u64::from(delay));
    let fut = Delay::new(when)
        .and_then(move |_| {
            vm::Machine::with_current(|vm| process::send_message(vm, from, dest, msg));
            Ok(())
        })
        .map_err(|e| panic!("delay errored; err={:?}", e));
    vm.runtime.executor().spawn(fut);

    let heap = &process.context_mut().heap;
    let reference = vm.next_ref();
    let ref_term = Term::reference(heap, reference);
    Ok(ref_term)
}
