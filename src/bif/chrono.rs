use crate::atom;
use crate::bif;
use crate::exception::{Exception, Reason};
use crate::process::Process;
use crate::value::{self, Term, TryFrom, Tuple, Variant};
use crate::vm;
use chrono::prelude::*;
use num_bigint::ToBigInt;
use num_traits::ToPrimitive;
use std::pin::Pin;
use std::time::SystemTime;

/// http://erlang.org/doc/apps/erts/time_correction.html
/// http://erlang.org/doc/apps/erts/time_correction.html#Erlang_System_Time

pub fn date_0(_vm: &vm::Machine, process: &Pin<&mut Process>, _args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;
    let date = Local::today();

    Ok(tup3!(
        heap,
        Term::int(date.year() as i32),
        Term::int(date.month() as i32),
        Term::int(date.day() as i32)
    ))
}

pub fn localtime_0(_vm: &vm::Machine, process: &Pin<&mut Process>, _args: &[Term]) -> bif::Result {
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

pub fn monotonic_time_0(
    vm: &vm::Machine,
    process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    // TODO: needs https://github.com/rust-lang/rust/issues/50202
    // .as_nanos()

    let heap = &process.context_mut().heap;

    Ok(Term::bigint(
        heap,
        vm.elapsed_time().as_secs().to_bigint().unwrap(),
    ))
}

// TODO monotonic_time_1
pub fn monotonic_time_1(
    vm: &vm::Machine,
    process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    // TODO: needs https://github.com/rust-lang/rust/issues/50202
    // .as_nanos()
    let heap = &process.context_mut().heap;

    Ok(Term::bigint(
        heap,
        vm.elapsed_time().as_secs().to_bigint().unwrap(),
    ))
}

pub fn system_time_0(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
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

pub fn system_time_1(vm: &vm::Machine, process: &Pin<&mut Process>, args: &[Term]) -> bif::Result {
    let heap = &process.context_mut().heap;

    let time = match args[0].into_variant() {
        Variant::Atom(atom::SECOND) | Variant::Atom(atom::NATIVE) => SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_bigint()
            .unwrap(),
        Variant::Atom(atom::MILLISECOND) => SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_bigint()
            .unwrap(),
        Variant::Atom(atom::MICROSECOND) => SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .to_bigint()
            .unwrap(),
        Variant::Atom(atom::PERF_COUNTER) => vm.elapsed_time().as_secs().to_bigint().unwrap(),
        Variant::Integer(_) => unimplemented!(),
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };
    Ok(Term::bigint(heap, time))
}

// time_offset 0,1

// timestamp_0
//timestamp() ->
// ErlangSystemTime = erlang:system_time(microsecond),
// MegaSecs = ErlangSystemTime div 1000000000000,
// Secs = ErlangSystemTime div 1000000 - MegaSecs*1000000,
// MicroSecs = ErlangSystemTime rem 1000000,
// {MegaSecs, Secs, MicroSecs}.

pub fn universaltime_0(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
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

pub fn posixtime_to_universaltime_1(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    let heap = &process.context_mut().heap;

    let timestamp: i64 = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => i64::from(i),
        Ok(value::Num::Bignum(value)) => value
            .to_i64()
            .ok_or_else(|| Exception::new(Reason::EXC_BADARG))?,
        _ => return Err(Exception::new(Reason::EXC_BADARG)),
    };

    let dt = NaiveDateTime::from_timestamp_opt(timestamp, 0)
        .ok_or_else(|| Exception::new(Reason::EXC_BADARG))?;

    // hp = HAlloc(BIF_P, 4+4+3);
    let date = tup3!(
        heap,
        Term::int(dt.year() as i32),
        Term::int(dt.month() as i32),
        Term::int(dt.day() as i32)
    );
    let time = tup3!(
        heap,
        Term::int(dt.hour() as i32),
        Term::int(dt.minute() as i32),
        Term::int(dt.second() as i32)
    );
    Ok(tup2!(heap, date, time))
}

type ErlDateTime = ((i32, i32, i32), (i32, i32, i32));

/// Check and extract components from a tuple on form: {{Y,M,D},{H,M,S}}
fn time_to_parts(term: Term) -> Option<ErlDateTime> {
    // term to tuple
    if let Ok(wrapper) = Tuple::try_from(&term) {
        if wrapper.len() != 2 {
            return None;
        }

        let date = match Tuple::try_from(&wrapper[0]) {
            Ok(date) => {
                if date.len() != 3 {
                    return None;
                }
                date
            }
            _ => return None,
        };
        let year = date[0].to_i32()?;
        let month = date[1].to_i32()?;
        let day = date[2].to_i32()?;

        let time = match Tuple::try_from(&wrapper[1]) {
            Ok(time) => {
                if time.len() != 3 {
                    return None;
                }
                time
            }
            _ => return None,
        };
        let hour = time[0].to_i32()?;
        let minute = time[1].to_i32()?;
        let second = time[2].to_i32()?;
        return Some(((year, month, day), (hour, minute, second)));
    }
    None
}

pub fn universaltime_to_localtime_1(
    _vm: &vm::Machine,
    process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    let heap = &process.context_mut().heap;

    let ((year, month, day), (hour, minute, second)) =
        time_to_parts(args[0]).ok_or_else(|| Exception::new(Reason::EXC_BADARG))?;

    let dt = Utc
        .ymd(year, month as u32, day as u32)
        .and_hms(hour as u32, minute as u32, second as u32)
        .with_timezone(&Local);

    // hp = HAlloc(BIF_P, 4+4+3);
    let date = tup3!(
        heap,
        Term::int(dt.year() as i32),
        Term::int(dt.month() as i32),
        Term::int(dt.day() as i32)
    );
    let time = tup3!(
        heap,
        Term::int(dt.hour() as i32),
        Term::int(dt.minute() as i32),
        Term::int(dt.second() as i32)
    );
    Ok(tup2!(heap, date, time))
}
