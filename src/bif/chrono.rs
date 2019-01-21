use crate::bif::BifResult;
use crate::process::RcProcess;
use crate::value::{self, Term};
use crate::vm;
use chrono::{Datelike, Local, Timelike, Utc};
use num::bigint::ToBigInt;
use std::time::SystemTime;

/// http://erlang.org/doc/apps/erts/time_correction.html
/// http://erlang.org/doc/apps/erts/time_correction.html#Erlang_System_Time

#[inline]
pub fn bif_erlang_date_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let date = Local::today();

    Ok(tup3!(
        heap,
        Term::int(date.year() as i32),
        Term::int(date.month() as i32),
        Term::int(date.day() as i32)
    ))
}

#[inline]
pub fn bif_erlang_localtime_0(_vm: &vm::Machine, process: &RcProcess, _args: &[Term]) -> BifResult {
    let heap = &process.context_mut().heap;
    let datetime = Local::now();

    let date = tup3!(
        heap,
        Term::int(datetime.year() as i32),
        Term::int(datetime.month() as i32),
        Term::int(datetime.day() as i32)
    );
    let time = tup3!(
        heap,
        Term::int(datetime.hour() as i32),
        Term::int(datetime.minute() as i32),
        Term::int(datetime.second() as i32)
    );
    Ok(tup2!(heap, date, time))
}

// now_0 is deprecated

pub fn bif_erlang_monotonic_time_0(
    vm: &vm::Machine,
    process: &RcProcess,
    _args: &[Term],
) -> BifResult {
    // TODO: needs https://github.com/rust-lang/rust/issues/50202
    // .as_nanos()

    let heap = &process.context_mut().heap;

    Ok(Term::bigint(
        heap,
        vm.elapsed_time().as_secs().to_bigint().unwrap(),
    ))
}

// TODO monotonic_time_1

pub fn bif_erlang_system_time_0(
    _vm: &vm::Machine,
    process: &RcProcess,
    _args: &[Term],
) -> BifResult {
    let heap = &process.context_mut().heap;

    Ok(Term::bigint(
        heap,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_bigint()
            .unwrap(),
    ))
}

// TODO system_time_1
//
// time_offset 0,1

// timestamp_0
//timestamp() ->
// ErlangSystemTime = erlang:system_time(microsecond),
// MegaSecs = ErlangSystemTime div 1000000000000,
// Secs = ErlangSystemTime div 1000000 - MegaSecs*1000000,
// MicroSecs = ErlangSystemTime rem 1000000,
// {MegaSecs, Secs, MicroSecs}.

#[inline]
pub fn bif_erlang_universaltime_0(
    _vm: &vm::Machine,
    process: &RcProcess,
    _args: &[Term],
) -> BifResult {
    let heap = &process.context_mut().heap;
    let datetime = Utc::now();

    let date = tup3!(
        heap,
        Term::int(datetime.year() as i32),
        Term::int(datetime.month() as i32),
        Term::int(datetime.day() as i32)
    );
    let time = tup3!(
        heap,
        Term::int(datetime.hour() as i32),
        Term::int(datetime.minute() as i32),
        Term::int(datetime.second() as i32)
    );
    Ok(tup2!(heap, date, time))
}
