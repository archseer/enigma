use crate::atom;
use crate::bif;
// use crate::exception::{Exception, Reason};
use crate::process::Process;
use crate::value::Term;
use crate::vm;
use std::pin::Pin;

// FIXME: these are all dummies unless dtrace is enabled

pub fn dt_put_tag_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    Ok(atom!(UNDEFINED))
}
pub fn dt_get_tag_0(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    Ok(atom!(UNDEFINED))
}
pub fn dt_get_tag_data_0(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    Ok(atom!(UNDEFINED))
}
pub fn dt_spread_tag_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    Ok(atom!(TRUE))
}

pub fn dt_restore_tag_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    _args: &[Term],
) -> bif::Result {
    Ok(atom!(TRUE))
}

// dynamic trace
// FIXME: these are all dummies unless dtrace is enabled + vm dynamic probes

pub fn dt_prepend_vm_tag_data_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    Ok(args[0])
}

pub fn dt_append_vm_tag_data_1(
    _vm: &vm::Machine,
    _process: &Pin<&mut Process>,
    args: &[Term],
) -> bif::Result {
    Ok(args[0])
}
