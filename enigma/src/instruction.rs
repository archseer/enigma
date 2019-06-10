use instruction_codegen::instruction;

// mandatory for loop
use crate::vm::{Machine, op_deallocate};
use crate::{atom, port, bif, bitstring, module};
use crate::bitstring::Flag as BitFlag;
use crate::exception::{self, Exception, Reason};
use crate::exports_table::{Export};
use crate::instr_ptr::InstrPtr;
use crate::instruction::{self};
use crate::process::{self, RcProcess};
use crate::value::{self, Cons, Term, CastFrom, CastInto, CastIntoMut, Tuple, Variant};
use std::time;

use futures::{
  compat::*,
  future::{FutureExt, TryFutureExt},
  // io::AsyncWriteExt,
  stream::StreamExt,
  // sink::SinkExt,
};
// use futures::prelude::*;
// end mandatory for loop

use std::convert::TryInto;

// for the load transform
use crate::loader::LValue;
use crate::immix::Heap;
// end for the load transform

use crate::bif::Fn as BifFn;

pub trait IntoWithHeap<T>: Sized {
    /// Performs the conversion.
    fn into_with_heap(&self, constants: &mut Vec<Term>, heap: &Heap) -> T;
}
pub trait FromWithHeap<T>: Sized {
    /// Performs the conversion.
    fn from_with_heap(_: &T, constants: &mut Vec<Term>, heap: &Heap) -> Self;
}
impl<T, U> IntoWithHeap<U> for T where U: FromWithHeap<T>
{
    fn into_with_heap(&self, constants: &mut Vec<Term>, heap: &Heap) -> U {
        U::from_with_heap(self, constants, heap)
    }
}


pub type Bytes = Vec<u8>;

impl FromWithHeap<LValue> for Bytes {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::Str(s) => s.clone(),
            _ => panic!("{:?} passed", value),
        }
    }
}

pub type Arity = u8;
pub type Label = u32;
pub type Atom = u32;

pub type FloatRegs = u8;
// shrink these from u16 to u8 once swap instruction is used
pub type Regs = u16;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RegisterX(pub Regs);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RegisterY(pub Regs);

pub type ExtendedList = Box<Vec<Entry>>;
pub type JumpTable = Box<Vec<Label>>;

#[derive(Debug, Clone)]
pub enum Entry {
    Literal(u32),
    Label(u32),
    ExtendedLiteral(u32),
    Value(Source),
}

impl Entry {
    pub fn to_label(&self) -> u32 {
        match *self {
            Entry::Label(i) => i,
            _ => panic!(),
        }
    }
    pub fn into_register(&self) -> Register {
        match self {
            Entry::Value(s) => match *s {
                Source::X(i) => Register::X(i),
                Source::Y(i) => Register::Y(i),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
    pub fn into_value(&self) -> Source {
        match *self {
            Entry::ExtendedLiteral(i) => Source::ExtendedLiteral(i),
            Entry::Value(s) => s,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Register {
    X(RegisterX),
    Y(RegisterY),
}

impl FromWithHeap<LValue> for Register {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::X(i) => Register::X(RegisterX((*i).try_into().unwrap())),
            LValue::Y(i) => Register::Y(RegisterY((*i).try_into().unwrap())),
            _ => panic!("{:?} passed", value),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FRegister {
    FloatReg(FloatRegs),
    ExtendedLiteral(u32),
    X(RegisterX),
    Y(RegisterY),
}

impl FromWithHeap<LValue> for FRegister {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::FloatReg(i) => FRegister::FloatReg((*i).try_into().unwrap()),
            LValue::ExtendedLiteral(i) => FRegister::ExtendedLiteral(*i),
            LValue::X(i) => FRegister::X(RegisterX((*i).try_into().unwrap())),
            LValue::Y(i) => FRegister::Y(RegisterY((*i).try_into().unwrap())),
            _ => panic!("{:?} passed", value),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Source {
    Constant(u32), // index (max as u16? would shrink us down to 32)
    ExtendedLiteral(u32),
    X(RegisterX),
    Y(RegisterY),
}

impl FromWithHeap<LValue> for Source {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::Constant(c) => {
                let i = constants.len();
                constants.push(*c);
                Source::Constant(i.try_into().unwrap())
            }
            LValue::BigInt(num) => {
                // very unperformant, double indirection
                let i = constants.len();
                constants.push(Term::bigint(heap, num.clone()));
                Source::Constant(i.try_into().unwrap())
            }
            LValue::ExtendedLiteral(i) => Source::ExtendedLiteral(*i),
            LValue::X(i) => Source::X(RegisterX((*i).try_into().unwrap())),
            LValue::Y(i) => Source::Y(RegisterY((*i).try_into().unwrap())),
            _ => panic!("{:?} passed", value),
        }
    }
}

// temporary because no specialization
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Size {
    Constant(u32),
    Literal(u32),
    X(RegisterX),
    Y(RegisterY),
}

impl FromWithHeap<LValue> for Size {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::Constant(c) => {
                let i = constants.len();
                constants.push(*c);
                Size::Constant(i.try_into().unwrap())
            }
            LValue::Literal(i) => Size::Literal(*i),
            LValue::X(i) => Size::X(RegisterX((*i).try_into().unwrap())),
            LValue::Y(i) => Size::Y(RegisterY((*i).try_into().unwrap())),
            _ => panic!("{:?} passed", value),
        }
    }
}

impl Size {
    // FIXME: try to eliminate Size
    pub fn to_val(self, context: &crate::process::ExecutionContext) -> crate::value::Term {
        match self {
            Size::Literal(i) => crate::value::Term::uint(&context.heap, i),
            Size::Constant(i) => context.expand_arg(Source::Constant(i)),
            Size::X(i) => context.expand_arg(Source::X(i)),
            Size::Y(i) => context.expand_arg(Source::Y(i)),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bif(pub BifFn);

impl std::ops::Deref for Bif {
    type Target = BifFn;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for Bif {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<bif>")
    }
}

impl FromWithHeap<LValue> for Bif {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::Bif(c) => Bif(*c),
            _ => panic!("{:?} passed", value),
        }
    }
}

impl FromWithHeap<LValue> for bitstring::Flag {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        #[cfg(target_endian = "little")]
        macro_rules! native_endian {
            ($x:expr) => {
                if $x.contains(bitstring::Flag::BSF_NATIVE) {
                    $x.remove(bitstring::Flag::BSF_NATIVE);
                    $x.insert(bitstring::Flag::BSF_LITTLE);
                }
            };
        }

        #[cfg(target_endian = "big")]
        macro_rules! native_endian {
            ($x:expr) => {
                if $x.contains(bitstring::Flag::BSF_NATIVE) {
                    $x.remove(bitstring::Flag::BSF_NATIVE);
                    $x.remove(bitstring::Flag::BSF_LITTLE);
                }
            };
        }
        match *value {
            LValue::Literal(l) => {
                let mut flags = bitstring::Flag::from_bits(l.try_into().unwrap()).unwrap();
                native_endian!(flags);
                flags
            }
            _ => unimplemented!("to_flags for {:?}", value),
        }
    }
}

impl FromWithHeap<LValue> for ExtendedList {
    #[inline]
    fn from_with_heap(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> Self {
        match value {
            LValue::ExtendedList(l) => Box::new(
                l.iter()
                    .map(|i| match i {
                        LValue::Literal(i) => crate::instruction::Entry::Literal(*i),
                        LValue::ExtendedLiteral(i) => {
                            crate::instruction::Entry::ExtendedLiteral(*i)
                        }
                        LValue::Label(i) => crate::instruction::Entry::Label(*i),
                        _ => crate::instruction::Entry::Value(i.into_with_heap(constants, heap)),
                    })
                    .collect(),
            ),
            _ => panic!(),
        }
    }
}

// used by the codegen
#[inline]
pub fn to_const(value: &LValue, constants: &mut Vec<Term>, heap: &Heap) -> u32 {
    match value {
        LValue::Constant(c) => {
            let i = constants.len();
            constants.push(*c);
            i.try_into().unwrap()
        }
        LValue::BigInt(num) => {
            // very unperformant, double indirection
            let i = constants.len();
            constants.push(Term::bigint(heap, num.clone()));
            i.try_into().unwrap()
        }
        _ => panic!("{:?} passed", value),
    }
}

// TODO: can't derive Copy yet since we have extended list which needs cloning
// TODO: specialize bifs with f=0 as head vs body (ones jump, others don't)
// TODO: specialize arith into ops, and sub-specialize "increments" source + const

// TUPLE ELEMENTS: u24

// we could bitmask Source. In which case it would fit current loader size.
// type can be either const, x or y (needs two bits). Meaning we'd get a u30 for the payload.
// We could also embed it into Term, by abusing the Special type -- anything that would make
// Constant fit into the instruction and not require a separate table would be good.
// represent Vec<u8> string buffers as pointers into the string table (a slice would work?).


// need three things:
// generic op definitions
// op definitions with type permutations
// transform rules

const APPLY_2: bif::Fn = bif::bif_erlang_apply_2;
const APPLY_3: bif::Fn = bif::bif_erlang_apply_3;

instruction!(
    fn func_info(module: s, function: s, arity: t) {
        return Err(Exception::new(Reason::EXC_FUNCTION_CLAUSE));
    },
    fn jump(label: l) {
        op_jump!(context, label)
    },
    fn r#move(src: xyqc, dest: xy) {
        let val = #src;
        #dest = val;
    },
    fn swap(src: xy, dest: xy) {
        let val = #src;
        #dest = val;
    },
    fn r#return() {
        op_return!(process, context);
    },
    fn send() {
        // send x1 to x0, write result to x0
        let pid = context.x[0];
        let msg = context.x[1];
        let res = match pid.into_variant() {
            Variant::Port(id) => port::send_message(vm, process.pid, id, msg),
            _ => process::send_message(vm, process.pid, pid, msg),
        }?;
        context.x[0] = res;
    },
    fn remove_message() {
        // Unlink the current message from the message queue. Remove any timeout.
        process.local_data_mut().mailbox.remove();
        // clear timeout
        context.timeout.take();
        // reset savepoint of the mailbox
        process.local_data_mut().mailbox.reset();
    },
    fn timeout() {
        //  Reset the save point of the mailbox and clear the timeout flag.
        process.local_data_mut().mailbox.reset();
        // clear timeout
        context.timeout.take();
    },
    fn loop_rec(fail: l, source: s) {
        // TODO: source is supposed to be the location, but it's always x0
        // grab message from queue, put to x0, if no message, jump to fail label
        if let Some(msg) = process.receive()? {
            // info!("recv proc pid={:?} msg={}", process.pid, msg);
            context.x[0] = msg
        } else {
            op_jump!(context, fail);
        }
    },
    fn loop_rec_end(label: l) {
        // Advance the save pointer to the next message and jump back to Label.

        process.local_data_mut().mailbox.advance();
        op_jump!(context, label);
    },
    fn wait(label: l) {
        // jump to label, set wait flag on process

        op_jump!(context, label);

        // TODO: this currently races if the process is sending us
        // a message while we're in the process of suspending.

        // set wait flag
        // process.set_waiting_for_message(true);

        // LOCK mailbox on looprec, unlock on wait/waittimeout
        let cancel = process.context_mut().recv_channel.take().unwrap();
        cancel.await; // suspend process

        // println!("pid={} resumption ", process.pid);
        process.process_incoming()?;
    },
    fn wait_timeout(label: l, time: s) {
        // Sets up a timeout of Time milliseconds and saves the address of the
        // following instruction as the entry point if the timeout triggers.

        if process.context_mut().timeout.is_none() {
            // if this is outside of loop_rec, this will be blank
            let (trigger, cancel) = futures::channel::oneshot::channel::<()>();
            process.context_mut().recv_channel = Some(cancel);
            process.context_mut().timeout = Some(trigger);
        }

        let cancel = process.context_mut().recv_channel.take().unwrap();

        match context.expand_arg(time).into_variant() {
            Variant::Atom(atom::INFINITY) => {
                // just a normal &Instruction::Wait
                // println!("infinity wait");
                op_jump!(context, label);

                cancel.await; // suspend process
                // println!("select! resumption pid={}", process.pid);
            },
            Variant::Integer(ms) => {
                let when = time::Duration::from_millis(ms as u64);
                use tokio::prelude::FutureExt;

                match cancel.into_future().boxed().compat().timeout(when).compat().await {
                    Ok(()) =>  {
                        // jump to success (start of recv loop)
                        op_jump!(context, label);
                        // println!("select! resumption pid={}", process.pid);
                    }
                    Err(_err) => {
                        // timeout
                        // println!("select! delay timeout {} pid={} ms={} m={:?}", ms, process.pid, context.expand_arg(&ins.args[1]), ins.args[1]);
                        // println!("select! delay timeout {} pid={} ms={} err={:?}", ms, process.pid, context.expand_arg(&ins.args[1]), err);

                        // remove channel
                        context.timeout.take();

                        // continue to next instruction (timeout op)
                    }

                }
                // select! { // suspend process
                //     _t = future => {
                //         // jump to success (start of recv loop)
                //         let label = ins.args[0].to_u32();
                //         op_jump!(context, label);
                //         println!("select! resumption");
                //     },

                //     // did not work _ = tokio::timer::Delay::new(when) => {
                //     _ = Delay::new(when) => {
                //         // timeout
                //         println!("select! delay timeout");

                //         // remove channel
                //         // context.timeout.take();

                //         () // continue to next instruction (timeout op)
                //     }
                // };
            },
            // TODO: bigint
            _ => unreachable!("{}", context.expand_arg(time))
        }
        process.process_incoming()?;
    },
    fn recv_mark(_fail: l) {
        process.local_data_mut().mailbox.mark();
    },
    fn recv_set(_fail: l) {
        process.local_data_mut().mailbox.set();
    },
    fn call(arity: t, label: l) {
        // store arity as live
        context.cp = Some(context.ip);
        op_jump!(context, label);

        // if process.pid > 70 {
        //     let (mfa, _) = context.ip.lookup_func_info().unwrap();
        //     info!("pid={} action=call mfa={}", process.pid, mfa);
        // }
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_last(arity: t, label: l, words: r) {
        // store arity as live
        op_deallocate(context, words);

        op_jump!(context, label);

        // if process.pid > 70 {
        // let (mfa, _) = context.ip.lookup_func_info().unwrap();
        // info!("pid={} action=call_last mfa={}", process.pid, mfa);
        // }
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_only(arity: t, label: l) {
        // store arity as live
        op_jump!(context, label);

        // if process.pid > 70 {
        // let (mfa, _) = context.ip.lookup_func_info().unwrap();
        // info!("pid={} action=call_only mfa={}", process.pid, mfa);
        // }
        safepoint_and_reduce!(vm, process, context.reds);
    },
    // TODO: module.imports lookups as embedded MFA exports
    fn call_ext(arity: t, destination: u) {
        // save pointer onto CP
        context.cp = Some(context.ip);

        op_call_ext!(vm, context, &process, arity, destination);
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_ext_only(arity: t, destination: u) {
        op_call_ext!(vm, context, &process, arity, destination);
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_ext_last(arity: t, destination: u, words: r) {
        op_deallocate(context, words);

        op_call_ext!(vm, context, &process, arity, destination);
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_bif(arity: t, bif: b) {
        // call a bif, store result in x0

        let args = &context.x[0..arity as usize];
        match bif(vm, &process, args) {
            Ok(val) => context.x[0] = val,
            Err(exc) => return Err(exc),
        }
    },
    fn call_bif_last(arity: t, bif: b, words: r) {
        // call a bif, store result in x0, return
        op_deallocate(context, words);

        let args = &context.x[0..arity as usize];
        match bif(vm, &process, args) {
            Ok(val) => {
                context.x[0]= val;
                op_return!(process, context);
            },
            Err(exc) => return Err(exc),
        }
    },
    fn call_bif_only(arity: t, bif: b) {
        // call a bif, store result in x0, return

        let args = &context.x[0..arity as usize];
        match bif(vm, &process, args) {
            Ok(val) => {
                context.x[0]= val;
                op_return!(process, context);
            },
            Err(exc) => return Err(exc),
        }
    },
    fn apply_fun() {
        // save pointer onto CP
        context.cp = Some(context.ip);

        op_apply_fun!(vm, context, process)
    },
    fn apply_fun_last(words: r) {
        op_deallocate(context, words);

        op_apply_fun!(vm, context, process)
    },
    fn apply_fun_only() {
        op_apply_fun!(vm, context, process)
    },
    fn i_apply() {
        // different from apply(), it's a bif override
        context.cp = Some(context.ip);

        op_apply!(vm, context, process);
    },
    fn i_apply_last(words: r) {
        op_deallocate(context, words);

        op_apply!(vm, context, process);
    },
    fn i_apply_only() {
        op_apply!(vm, context, process);
    },
    fn bif0(bif: b, reg: d) {
        let val = bif(vm, &process, &[]).unwrap(); // bif0 can't fail
        context.set_register(reg, val);
    },
    fn bif1(fail: l, bif: b, arg1: s, reg: d) {
        let args = &[#arg1];
        match bif(vm, &process, args) {
            Ok(val) => context.set_register(reg, val),
            Err(exc) => cond_fail!(context, fail, exc),
        }
    },
    fn bif2(fail: l, bif: b, arg1: s, arg2: s, reg: d) {
        let args = &[#arg1, #arg2];
        match bif(vm, &process, args) {
            Ok(val) => context.set_register(reg, val),
            Err(exc) => cond_fail!(context, fail, exc),
        }
    },
    fn allocate(stackneed: r, live: r) {
        context
            .stack
            .resize(context.stack.len() + stackneed as usize, NIL);
        context.callstack.push((stackneed, context.cp.take()));
    },
    fn allocate_heap(stackneed: r, heapneed: r, live: r) {
        // TODO: this also zeroes the values, make it dynamically change the
        // capacity/len of the Vec to bypass initing these.

        // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
        // num of X regs. save cp on stack.
        context
            .stack
            .resize(context.stack.len() + stackneed as usize, NIL);
        // TODO: check heap for heapneed space!
        context.callstack.push((stackneed, context.cp.take()));
    },
    fn allocate_zero(stackneed: r, live: r) {
        context
            .stack
            .resize(context.stack.len() + stackneed as usize, NIL);
        context.callstack.push((stackneed, context.cp.take()));
    },
    fn allocate_heap_zero(stackneed: r, heapneed: r, live: r) {
        // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
        // num of X regs. save cp on stack.
        context
            .stack
            .resize(context.stack.len() + stackneed as usize, NIL);
        // TODO: check heap for heapneed space!
        context.callstack.push((stackneed, context.cp.take()));
    },
    fn test_heap(_heapneed: r, n: r) { // TODO args
        // println!("TODO: TestHeap unimplemented!");
    },
    fn init(n: d) {
        context.set_register(n, NIL)
    },
    fn deallocate(n: r) {
        op_deallocate(context, n);
    },
    fn is_ge(fail: l, arg1: s, arg2: s) {
        if let Some(std::cmp::Ordering::Less) = #arg1.erl_partial_cmp(&#arg2) {
            op_jump!(context, fail);
        } else {
            // ok
        }
    },
    fn is_lt(fail: l, arg1: s, arg2: s) {
        if let Some(std::cmp::Ordering::Less) = #arg1.erl_partial_cmp(&#arg2) {
            // ok
        } else {
            op_jump!(context, fail);
        }
    },
    fn is_eq(fail: l, arg1: s, arg2: s) {
        if let Some(std::cmp::Ordering::Equal) = #arg1.erl_partial_cmp(&#arg2) {
            // ok
        } else {
            op_jump!(context, fail);
        }
    },
    fn is_ne(fail: l, arg1: s, arg2: s) {
        if let Some(std::cmp::Ordering::Equal) = #arg1.erl_partial_cmp(&#arg2) {
            op_jump!(context, fail);
        }
    },
    fn is_eq_exact(fail: l, arg1: s, arg2: s) {
        if #arg1.eq(&#arg2) {
            // ok
        } else {
            op_jump!(context, fail);
        }
    },
    fn is_ne_exact(fail: l, arg1: s, arg2: s) {
        if #arg1.eq(&#arg2) {
            op_jump!(context, fail);
        } else {
            // ok
        }
    },
    fn is_integer(fail: l, arg1: xy) {
        if !#arg1.is_integer() {
            op_jump!(context, fail);
        }
    },
    fn is_float(fail: l, arg1: xy) {
        if !#arg1.is_float() {
            op_jump!(context, fail);
        }
    },
    fn is_number(fail: l, arg1: xy) {
        if !#arg1.is_number() {
            op_jump!(context, fail);
        }
    },
    fn is_atom(fail: l, arg1: xy) {
        if !#arg1.is_atom() {
            op_jump!(context, fail);
        }
    },
    fn is_reference(fail: l, arg1: xy) {
        if !#arg1.is_ref() {
            op_jump!(context, fail);
        }
    },
    fn is_port(fail: l, arg1: xy) {
        if !#arg1.is_port() {
            op_jump!(context, fail);
        }
    },
    fn is_pid(fail: l, arg1: xy) {
        if !#arg1.is_pid() {
            op_jump!(context, fail);
        }
    },
    fn is_nil(fail: l, arg1: xy) {
        if !#arg1.is_nil() {
            op_jump!(context, fail);
        }
    },
    fn is_binary(fail: l, arg1: xy) {
        if !#arg1.is_binary() {
            op_jump!(context, fail);
        }
    },
    fn is_bitstr(fail: l, arg1: xy) {
        if !#arg1.is_bitstring() {
            op_jump!(context, fail);
        }
    },
    fn is_list(fail: l, arg1: xy) {
        if !#arg1.is_list() {
            op_jump!(context, fail);
        }
    },
    fn is_nonempty_list(fail: l, arg1: xy) {
        if !#arg1.is_non_empty_list() {
            op_jump!(context, fail);
        }
    },
    fn is_tuple(fail: l, arg1: xy) {
        if !#arg1.is_tuple() {
            op_jump!(context, fail);
        }
    },
    fn is_function(fail: l, arg1: xy) {
        if !#arg1.is_function() {
            op_jump!(context, fail);
        }
    },
    fn is_boolean(fail: l, arg1: xy) {
        if !#arg1.is_boolean() {
            op_jump!(context, fail);
        }
    },
    fn is_map(fail: l, arg1: xy) {
        if !#arg1.is_map() {
            op_jump!(context, fail);
        }
    },
    fn is_function2(fail: l, arg1: xy, arity: s) {
        // TODO: needs to verify exports too
        let value = #arg1;
        let arity = #arity.to_uint().unwrap();
        if let Ok(closure) = value::Closure::cast_from(&value) {
            if closure.mfa.2 == arity {
                continue;
            }
        }
        if let Ok(mfa) = module::MFA::cast_from(&value) {
            if mfa.2 == arity {
                continue;
            }
        }
        op_jump!(context, fail);
    },
    fn test_arity(fail: l, arg1: xy, arity: u) {
        let val = #arg1;
        let t = Tuple::cast_from(&val).expect("test_arity: expected tuple");
        if t.len != arity {
            op_jump!(context, fail);
        }
    },
    fn select_val(arg: xy, fail: l, destinations: m) {
        // loop over dests
        // TODO: jump table if (int, label) pairs
        let arg = #arg.into_variant();
        let mut jumped = false;

        for chunk in destinations.chunks_exact(2) {
            if context.expand_arg(chunk[0].into_value()).into_variant() == arg {
                let label = chunk[1].to_label();
                op_jump!(context, label);
                jumped = true;
                break;
            }
        }
        // if we ran out of options, jump to fail
        if !jumped {
            op_jump!(context, fail);
        }
    },
    fn select_tuple_arity(arg: xy, fail: l, arities: m) {
        if let Ok(tup) = Tuple::cast_from(&#arg) {
            let len = tup.len;
            let mut jumped = false;

            for chunk in arities.chunks_exact(2) {
                if let instruction::Entry::Literal(n) = chunk[0] {
                    if n == len {
                        let label = chunk[1].to_label();
                        op_jump!(context, label);
                        jumped = true;
                        break
                    }
                }
            }
            // if we ran out of options, jump to fail
            if !jumped {
                op_jump!(context, fail);
            }
        } else {
            op_jump!(context, fail);
        }
    },
    fn jump_on_val(arg: s, fail: l, table: T, min: i) {
        // SelectVal optimized with a jump table
        if let Some(i) = #arg.to_int() {
            // TODO: optimize for min: 0
            let index = (i - min) as usize;
            if index < table.len() {
                op_jump!(context, table[index]);
            } else {
                op_jump!(context, fail);
            }
        } else {
            op_jump!(context, fail);
        }
    },
    fn get_list(source: xy, head: xy, tail: xy) {
        let val = #source;
        if let Ok(value::Cons { head: h, tail: t }) = val.cast_into() {
            #head = *h;
            #tail = *t;
        } else {
            panic!("badarg to GetList")
        }
    },
    fn get_tuple_element(source: xy, element: u, destination: xy) {
        let source = #source;
        let n = element as usize;
        if let Ok(t) = Tuple::cast_from(&source) {
            #destination = t[n];
        } else {
            panic!("GetTupleElement: source is of wrong type")
        }
    },
    fn set_tuple_element(new_element: s, tuple: s, position: u) {
        let el = #new_element;
        let tuple = #tuple;
        let pos = position as usize;
        if let Ok(t) = tuple.cast_into_mut() {
            let tuple: &mut value::Tuple = t; // annoying, need type annotation
            tuple[pos] = el;
        } else {
            panic!("GetTupleElement: source is of wrong type")
        }
    },
    fn put_list(head: s, tail: s, destination: xy) {
        // Creates a cons cell with [H|T] and places the value into Dst.
        let head = #head;
        let tail = #tail;
        let cons = cons!(&context.heap, head, tail);
        #destination = cons;
    },
    fn put_tuple2(destination: xy, list: m) {
        let arity = list.len();

        let tuple = value::tuple(&context.heap, arity as u32);
        for i in 0..arity {
            unsafe {
                std::ptr::write(&mut tuple[i], context.expand_entry(&list[i]));
            }
        }
        #destination = Term::from(tuple);
    },
    fn badmatch(value: s) {
        let value = #value;
        println!("Badmatch: {}", value);
        return Err(Exception::with_value(Reason::EXC_BADMATCH, value));
    },
    fn if_end() {
        // Raises the if_clause exception.
        return Err(Exception::new(Reason::EXC_IF_CLAUSE));
    },
    fn case_end(value: s) {
        // Raises the case_clause exception with the value of Arg0
        let value = #value;
        println!("err=case_clause val={}", value);
        // err=case_clause val={{820515101, #Ref<0.0.0.105>}, :timeout, {:gen_server, :cast, [#Pid<61>, :repeated_filesync]}}
        return Err(Exception::with_value(Reason::EXC_CASE_CLAUSE, value));
    },
    fn try(register: d, fail: l) {
        // TODO: try is identical to catch, and is remapped in the OTP loader
        // create a catch context that wraps f - fail label, and stores to y - reg.
        context.catches += 1;
        context.set_register(
            register,
            Term::catch(
                &context.heap,
                InstrPtr {
                    ptr: fail,
                    module: context.ip.module
                }
            )
        );
    },
    fn try_end(register: d) {
        context.catches -= 1;
        context.set_register(register, NIL) // TODO: make_blank macro
    },
    fn try_case(register: d) {
        // pops a catch context in y  Erases the label saved in the Arg0 slot. Noval in R0 indicate that something is caught. If so, R0 is set to R1, R1 — to R2, R2 — to R3.

        // TODO: this initial part is identical to TryEnd
        context.catches -= 1;
        context.set_register(register, NIL); // TODO: make_blank macro

        assert!(context.x[0].is_none());
        // TODO: c_p->fvalue = NIL;
        // TODO: make more efficient via memmove
        context.x[0] = context.x[1];
        context.x[1] = context.x[2];
        context.x[2] = context.x[3];
    },
    fn try_case_end(value: s) {
        // Raises a try_clause exception with the value read from Arg0.
        let value = #value;
        return Err(Exception::with_value(Reason::EXC_TRY_CLAUSE, value));
    },
    fn catch(register: d, fail: l) {
        // create a catch context that wraps f - fail label, and stores to y - reg.
        context.catches += 1;
        context.set_register(
            register,
            Term::catch(
                &context.heap,
                InstrPtr {
                    ptr: fail,
                    module: context.ip.module
                }
            )
        );
    },
    fn catch_end(register: d) {
        // Pops a “catch” context. Erases the label saved in the Arg0 slot. Noval in R0
        // indicates that something is caught. If R1 contains atom throw then R0 is set
        // to R2. If R1 contains atom error than a stack trace is added to R2. R0 is
        // set to {exit,R2}.
        //
        // difference fron try is, try will exhaust all the options then fail, whereas
        // catch will keep going upwards.

        // TODO: this initial part is identical to TryEnd
        context.catches -= 1; // TODO: this is overflowing
        context.set_register(register, NIL); // TODO: make_blank macro

        if context.x[0].is_none() {
            // c_p->fvalue = NIL;
            if context.x[1] == Term::atom(atom::THROW) {
                context.x[0] = context.x[2]
            } else {
                if context.x[1] == Term::atom(atom::ERROR) {
                    context.x[2] =
                        exception::add_stacktrace(&process, context.x[2], context.x[3]);
                }
                // only x(2) is included in the rootset here
                // if (E - HTOP < 3) { check for heap space, otherwise garbage collect
                // ..
                //     FCALLS -= erts_garbage_collect_nobump(c_p, 3, reg+2, 1, FCALLS);
                // }
                context.x[0] =
                    tup2!(&context.heap, Term::atom(atom::EXIT_U), context.x[2]);
            }
        }
    },
    fn raise(trace: d, value: s) {
        // Raises the exception. The instruction is garbled by backward compatibility. Arg0 is a stack trace
        // and Arg1 is the value accompanying the exception. The reason of the raised exception is dug up
        // from the stack trace
        let trace = #trace;
        let value = #value;

        let reason = if let Some(s) = exception::get_trace_from_exc(&trace) {
            primary_exception!(s.reason)
        } else {
            Reason::EXC_ERROR
        };
        return Err(Exception {
            reason,
            value,
            trace,
        });
    },
    fn apply(arity: t) {
        context.cp = Some(context.ip);

        op_fixed_apply!(vm, context, &process, arity);
        // call this fixed_apply, used for ops (apply, apply_last).
        // apply is the func that's equivalent to erlang:apply/3 (and instrs)
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn apply_last(arity: t, nwords: r) {
        op_deallocate(context, nwords);

        op_fixed_apply!(vm, context, &process, arity);
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn gc_bif1(fail: l, _live: r, bif: b, arg1: s, reg: d) {
        // TODO: GcBif needs to handle GC as necessary
        let args = &[#arg1];
        match bif(vm, &process, args) {
            Ok(val) => context.set_register(reg, val),
            Err(exc) => cond_fail!(context, fail, exc),
        }
    },
    fn gc_bif2(fail: l, _live: r, bif: b, arg1: s, arg2: s, reg: d) {
        // TODO: GcBif needs to handle GC as necessary
        let args = &[#arg1, #arg2];
        match bif(vm, &process, args) {
            Ok(val) => context.set_register(reg, val),
            Err(exc) => cond_fail!(context, fail, exc),
        }
    },
    fn gc_bif3(fail: l, _live: r, bif: b, arg1: s, arg2: s, arg3: s, reg: d) {
        // TODO: GcBif needs to handle GC as necessary
        let args = &[#arg1, #arg2, #arg3];
        match bif(vm, &process, args) {
            Ok(val) => context.set_register(reg, val),
            Err(exc) => cond_fail!(context, fail, exc),
        }
    },
    // TODO unit needs to be u16
    fn bs_add(fail: l, size1: s, size2: s, unit: r, destination: d) {
        // Calculates the total of the number of bits in Src1 and the number of units in Src2. Stores the result to Dst.
        // bs_add(Fail, Src1, Src2, Unit, Dst)
        // dst = (src1 + src2) * unit

        // TODO: trickier since we need to check both nums are positive and can fit
        // into int/bigint

        // optimize when one append is 0 and unit is 1, it's just a move
        // bs_add Fail S1=i==0 S2 Unit=u==1 D => move S2 D

        // TODO use fail label
        let s1 = #size1.to_uint().unwrap();
        let s2 = #size2.to_uint().unwrap();

        let res = Term::uint(&context.heap, (s1 + s2) * unit as u32);
        context.set_register(destination, res)
    },
    fn bs_init2(fail: l, size: S, words: r, regs: r, flags: F, destination: d) {
        // optimize when init is with empty size
        // bs_init2 Fail Sz=u Words=u==0 Regs Flags Dst => i_bs_init Sz Regs Dst

        // Words is heap alloc size
        // regs is live regs for GC
        // flags is unused

        // bs_init2 Fail Sz Words=u==0 Regs Flags Dst => \
        //   i_bs_init_fail Sz Fail Regs Dst
        //   op1 = size, op2 = 0?,
        //   verify that the size is within limits (if non0) or jump to fail
        //   allocate binary + procbin
        //   set as non writable initially??

        // TODO: use a current_string ptr to be able to write to the Arc wrapped str
        // alternatively, loop through the instrs until we hit a non bs_ instr.
        // that way, no unsafe ptrs!
        let size = size.to_val(context).to_int().unwrap() as usize;
        let mut binary = bitstring::Binary::with_size(size);
        binary.is_writable = false;
        let term = Term::binary(&context.heap, binary);
        // TODO ^ ensure this pointer stays valid after heap alloc
        context.bs = bitstring::Builder::new(term.get_boxed_value::<bitstring::RcBinary>().unwrap());
        context.set_register(destination, term);
    },
    fn bs_put_integer(fail: l, size: s, unit: u, flags: F, source: s) {
        // Size can be atom all
        // TODO: fail label
        let size = match #size.into_variant() {
            Variant::Integer(i) => i as usize,
            // LValue::Literal(i) => i as usize, // TODO: unsure if correct
            Variant::Atom(atom::START) => 0,
            _ => unreachable!("{:?}", size),
        };

        let size = size * (unit as usize);

        context.bs.put_integer(size as usize, flags, #source);
    },
    fn bs_put_binary(fail: l, size: s, unit: u, flags: F, source: s) {
        // TODO: fail label

        let source = #source;

        let header = source.get_boxed_header().unwrap();
        match header {
            value::BOXED_BINARY | value::BOXED_SUBBINARY => {
                match #size.into_variant() {
                    Variant::Atom(atom::ALL) => context.bs.put_binary_all(source),
                    _ => unimplemented!("bs_put_binary size {:?}", size),
                }
            }
            _ => panic!("Bad argument to BsPutBinary: {}", source)
        }
    },
    fn bs_put_float(fail: l, size: s, unit: u, flags: F, source: s) {
        // Size can be atom all
        // TODO: fail label
        if unit != 8 {
            unimplemented!("bs_put_float unit != 8");
            //fail!(context, fail);
        }

        if let Variant::Float(value::Float(f)) = #source.into_variant() {
            match #size.into_variant() {
                Variant::Atom(atom::ALL) => context.bs.put_float(f),
                _ => unimplemented!("bs_put_float size {:?}", size),
            }
        } else {
            panic!("Bad argument to BsPutFloat")
        }
    },
    fn bs_put_string(binary: v) {
        // BsPutString uses the StrT strings table! needs to be patched in loader
        context.bs.put_bytes(binary);
    },
    fn bs_get_tail(cxt: d, destination: d, live: r) {
        // very similar to the old BsContextToBinary
        if let Ok(mb) = #cxt.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            // TODO; original calculated the hole size and overwrote MatchBuffer mem in place?
            let offs = mb.offset;
            let size = mb.size - offs;

            let res = Term::subbinary(
                &context.heap,
                bitstring::SubBinary::new(mb.original.clone(), size, offs, false),
            );
            context.set_register(destination, res);
        } else {
            // next0
            unreachable!()
        }
    },
    fn bs_start_match3(fail: l, source: s, live: r, destination: d) {
        let cxt = #source;

        if !cxt.is_pointer() {
            fail!(context, fail);
        }

        let header = cxt.get_boxed_header().unwrap();

        // Reserve a slot for the start position.

        match header {
            value::BOXED_MATCHBUFFER => {
                context.set_register(destination, cxt);
            }
            value::BOXED_BINARY | value::BOXED_SUBBINARY => {
                // Uint wordsneeded = ERL_BIN_MATCHBUFFER_SIZE(slots);
                // $GC_TEST_PRESERVE(wordsneeded, live, context);

                let result = bitstring::start_match_3(&context.heap, cxt);

                if let Some(res) = result {
                    context.set_register(destination, res)
                } else {
                    fail!(context, fail);
                }
            }
            _ => {
                fail!(context, fail);
            }
        }
    },
    fn bs_get_position(cxt: d, destination: d, live: r) {
        if let Ok(mb) = #cxt.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            // TODO: unsafe cast
            context.set_register(destination, Term::uint(&context.heap, mb.offset as u32));
        } else {
            unreachable!()
        };
    },
    fn bs_set_position(cxt: d, position: s) {
        if let Ok(mb) = #cxt.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let pos = #position.to_int().unwrap();
            mb.offset = pos as usize;
        } else {
            unreachable!()
        };
    },
    fn bs_get_integer2(fail: l, ms: d, live: r, size: s, unit: u, flags: F, destination: d) {
        let size = #size.to_int().unwrap() as usize;
        let unit = unit as usize;

        let bits = size * unit;

        // let size = size * (flags as usize >> 3); TODO: this was just because flags
        // & size were packed together on BEAM

        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            // fast path for common ops
            let res = match (bits, flags.contains(bitstring::Flag::BSF_LITTLE), flags.contains(bitstring::Flag::BSF_SIGNED)) {
                (8, true, true) => {
                    // little endian, signed
                    mb.get_bytes(1).map(|b| Term::int(i32::from(i8::from_le_bytes((*b).try_into().unwrap()))))
                },
                (8, true, false) => {
                    // little endian unsigned
                    mb.get_bytes(1).map(|b| Term::uint(&context.heap, u32::from(u8::from_le_bytes((*b).try_into().unwrap()))))
                },
                (8, false, true) => {
                    // big endian signed
                    mb.get_bytes(1).map(|b| Term::int(i32::from(i8::from_be_bytes((*b).try_into().unwrap()))))
                },
                (8, false, false) => {
                    // big endian unsigned
                    mb.get_bytes(1).map(|b| Term::uint(&context.heap, u32::from(u8::from_be_bytes((*b).try_into().unwrap()))))
                },
                (16, true, true) => {
                    // little endian, signed
                    mb.get_bytes(2).map(|b| Term::int(i32::from(i16::from_le_bytes((*b).try_into().unwrap()))))
                },
                (16, true, false) => {
                    // little endian unsigned
                    mb.get_bytes(2).map(|b| Term::uint(&context.heap, u32::from(u16::from_le_bytes((*b).try_into().unwrap()))))
                },
                (16, false, true) => {
                    // big endian signed
                    mb.get_bytes(2).map(|b| Term::int(i32::from(i16::from_be_bytes((*b).try_into().unwrap()))))
                },
                (16, false, false) => {
                    // big endian unsigned
                    mb.get_bytes(2).map(|b| Term::uint(&context.heap, u32::from(u16::from_be_bytes((*b).try_into().unwrap()))))
                },
                (32, true, true) => {
                    // little endian, signed
                    mb.get_bytes(4).map(|b| Term::int(i32::from_le_bytes((*b).try_into().unwrap())))
                },
                (32, true, false) => {
                    // little endian unsigned
                    mb.get_bytes(4).map(|b| Term::uint(&context.heap, u32::from_le_bytes((*b).try_into().unwrap())))
                },
                (32, false, true) => {
                    // big endian signed
                    mb.get_bytes(4).map(|b| Term::int(i32::from_be_bytes((*b).try_into().unwrap())))
                },
                (32, false, false) => {
                    // big endian unsigned
                    mb.get_bytes(4).map(|b| Term::uint(&context.heap, u32::from_be_bytes((*b).try_into().unwrap())))
                },
                (64, false, true) => {
                    // big endian signed
                    mb.get_bytes(8).map(|b| Term::int64(&context.heap, i64::from_be_bytes((*b).try_into().unwrap())))
                },
                (64, false, false) => {
                    // big endian unsigned
                    mb.get_bytes(8).map(|b| Term::uint64(&context.heap, u64::from_be_bytes((*b).try_into().unwrap())))
                },
                (128, false, true) => {
                    use num_bigint::ToBigInt;
                    // big endian signed
                    mb.get_bytes(16).map(|b| Term::bigint(&context.heap, i128::from_be_bytes((*b).try_into().unwrap()).to_bigint().unwrap()))
                },
                (128, false, false) => {
                    use num_bigint::ToBigInt;
                    // big endian unsigned
                    mb.get_bytes(16).map(|b| Term::bigint(&context.heap, u128::from_be_bytes((*b).try_into().unwrap()).to_bigint().unwrap()))
                },
                // slow fallback
                _ => {
                    mb.get_integer(&context.heap, bits, flags)
                }
            };

            if let Some(res) = res {
                context.set_register(destination, res)
            } else {
                fail!(context, fail);
            }
        };
    },
    fn bs_get_float2(fail: l, ms: d, live: r, size: s, unit: u, flags: F, destination: d) {
        let size = match #size.into_variant() {
            Variant::Integer(size) if size <= 64 => size as usize,
            _ => {
                fail!(context, fail);
            }
        };

        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let res = mb.get_float(&context.heap, size as usize, flags);
            if let Some(res) = res {
                context.set_register(destination, res)
            } else {
                fail!(context, fail);
            }
        };
    },
    fn bs_get_binary2(fail: l, ms: d, live: r, size: s, unit: u, flags: F, destination: d) {
        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            // proc pid=37 reds=907 mod="re" offs=870 ins=BsGetBinary2 args=[Label(916), X(5), Literal(9), X(2), Literal(8), Literal(0), X(2)]
            let heap = &context.heap;
            let unit = unit as usize;
            let res = match #size.into_variant() {
                Variant::Integer(size) => mb.get_binary(heap, size as usize * unit, flags),
                Variant::Atom(atom::ALL) => mb.get_binary_all(heap, flags),
                arg => unreachable!("get_binary2 for {:?}", arg),
            };

            if let Some(res) = res {
                context.set_register(destination, res)
            } else {
                fail!(context, fail);
            }
        }
    },

    fn bs_skip_bits2(fail: l, ms: d, size: s, unit: u, flags: F) {
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let size = #size.to_int().unwrap() as usize;

            let new_offset = mb.offset + (size * unit as usize);

            if new_offset <= mb.size {
                mb.offset = new_offset;
            } else {
                fail!(context, fail);
            }
        } else {
            unreachable!()
        }
    },
    fn bs_test_tail2(fail: l, ms: d, bits: u) {
        // TODO: beam specializes bits 0
        // Checks that the matching context Arg1 has exactly Arg2 unmatched bits. Jumps
        // to the label Arg0 if it is not so.
        // if size 0 == Jumps to the label in Arg0 if the matching context Arg1 still have unmatched bits.
        let offset = bits as usize;

        if let Ok(mb) = bitstring::MatchBuffer::cast_from(&#ms) {
            if mb.remaining() != offset {
                fail!(context, fail);
            }
        } else {
            unreachable!()
        }
    },
    fn bs_test_unit(fail: l, cxt: d, unit: u) {
        // Checks that the size of the remainder of the matching context is divisible
        // by unit, else jump to fail

        if let Ok(mb) = bitstring::MatchBuffer::cast_from(&#cxt) {
            if mb.remaining() % (unit as usize) != 0 {
                fail!(context, fail);
            }
        } else {
            unreachable!()
        }
    },
    fn bs_match_string(fail: l, cxt: d, bits: u, string: v) {
        if let Ok(mb) = #cxt.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let bits = bits as usize;

            if mb.remaining() < bits {
                fail!(context, fail);
            }
            // offs = mb->offset & 7;
            // if (offs == 0 && (bits & 7) == 0) {
            //     if (sys_memcmp(bytes, mb->base+(mb->offset>>3), bits>>3)) {
            //         $FAIL($Fail);
            //     }

            // No need for memcmp fastpath, cmp_bits already does that.
            unsafe {
                if bitstring::cmp_bits(
                    string.as_ptr(),
                    0,
                    mb.original.data.as_ptr().add(mb.offset >> 3),
                    mb.offset & 7,
                    bits,
                    ) != std::cmp::Ordering::Equal
                {
                    fail!(context, fail);
                }
            }
            mb.offset += bits;
        } else {
            unreachable!()
        }
    },
    fn bs_init_writable() {
        context.x[0] = bitstring::init_writable(&process, context.x[0]);
    },
    fn bs_append(fail: l, size: s, extra: r, live: r, unit: u, bin: s, _flags: F, destination: d) {
        // append and init also sets the string as current (state.current_binary) [seems to be used to copy string literals too]
        let size = #size;
        let extra_words = extra as usize;
        let unit = unit as usize;
        let src = #bin;

        let res = bitstring::append(&process, src, size, extra_words, unit);

        if let Some(res) = res {
            context.set_register(destination, res)
        } else {
            // TODO: execute fail only if non zero, else raise
            /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
            fail!(context, fail);
        }
    },
    fn bs_private_append(fail: l, size: s, unit: u, bin: s, _flags: F, destination: d) {
        let size = #size;
        let unit = unit as usize;
        let src = #bin;

        let res = bitstring::private_append(&process, src, size, unit);

        if let Some(res) = res {
            context.set_register(destination, res)
        } else {
            /* TODO not yet: c_p->freason is already set (to BADARG or SYSTEM_LIMIT). */
            fail!(context, fail);
        }
        unimplemented!("bs_private_append") // TODO
    },
    fn bs_init_bits(fail: l, size: s, words: r, regs: r, flags: F, destination: d) {
        // bs_init_bits Fail Sz=u Words=u==0 Regs Flags Dst => i_bs_init Sz Regs Dst

        // Words is heap alloc size
        // regs is live regs for GC
        // flags is unused?

        // size is in bits
        // debug_assert_eq!(ins.args.len(), 6);
        // TODO: RcBinary has to be set to is_writable = false
        // context.x[0] = bitstring::init_bits(&process, context.x[0], ..);
        unimplemented!("bs_init_bits") // TODO
    },
    fn bs_get_utf8(fail: l, ms: d, size: S, flags: F, destination: d) {
        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let res = mb.get_utf8();
            if let Some(res) = res {
                context.set_register(destination, res)
            } else {
                fail!(context, fail);
            }
        };
    },
    fn bs_get_utf16(fail: l, ms: d, size: S, flags: F, destination: d) {
        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let res = mb.get_utf16(flags);
            if let Some(res) = res {
                context.set_register(destination, res)
            } else {
                fail!(context, fail);
            }
        };
    },
    fn bs_get_utf32(fail: l, ms: d, size: S, flags: F, destination: d) {
        unimplemented!()
    },
    fn bs_skip_utf8(fail: l, ms: d, size: S, flags: F) {
        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let res = mb.get_utf8();
            if res.is_none() {
                fail!(context, fail);
            }
        } else {
            unreachable!()
        }
    },
    fn bs_skip_utf16(fail: l, ms: d, size: S, flags: F) {
        // TODO: this cast can fail
        if let Ok(mb) = #ms.get_boxed_value_mut::<bitstring::MatchBuffer>() {
            let res = mb.get_utf16(flags);
            if res.is_none() {
                fail!(context, fail);
            }
        } else {
            unreachable!()
        }
    },
    fn bs_skip_utf32(fail: l, ms: d, size: S, flags: F) {
        unimplemented!()
    },
    fn bs_utf8_size(fail: l, source: s, destination: d) {
        unimplemented!()
    },
    fn bs_utf16_size(fail: l, source: s, destination: d) {
        unimplemented!()
    },
    fn bs_put_utf8(fail: l, flags: F, source: s) {
        unimplemented!()
    },
    fn bs_put_utf16(fail: l, flags: F, source: s) {
        unimplemented!()
    },
    fn bs_put_utf32(fail: l, flags: F, source: s) {
        unimplemented!()
    },
    fn fclearerror() {
        // TODO: BEAM checks for unhandled errors
        context.f[0] = 0.0;
    },
    fn fcheckerror(_fail: l) {
        // I think it always checks register fr0
        if !context.f[0].is_finite() {
            return Err(Exception::new(Reason::EXC_BADARITH));
        }
    },
    fn fmove(source: R, destination: R) {
        // basically a normal move, except if we're moving out of floats we make a Val
        // otherwise we keep as a float
        let f = expand_float!(context, source);

        match destination {
            FRegister::X(reg) => {
                context.x[reg.0 as usize] = Term::from(f);
            }
            FRegister::Y(reg) => {
                let len = context.stack.len();
                context.stack[len - (reg.0 + 1) as usize] = Term::from(f);
            }
            FRegister::FloatReg(reg) => {
                context.f[reg as usize] = f;
            }
            _reg => unimplemented!(),
        }
    },
    fn fconv(source: R, destination: L) {
        // reg (x), dest (float reg)
        let value = match source {
            FRegister::ExtendedLiteral(i) => unsafe { (*context.ip.module).literals[i as usize] },
            FRegister::X(reg) => context.x[reg.0 as usize],
            FRegister::Y(reg) => {
                let len = context.stack.len();
                context.stack[len - (reg.0 + 1) as usize]
            }
            FRegister::FloatReg(reg) => Term::from(context.f[reg as usize]),
        };

        let val: f64 = match value.into_number() {
            Ok(value::Num::Float(f)) => f,
            Ok(value::Num::Integer(i)) => f64::from(i),
            Ok(_) => unimplemented!(),
            // TODO: bignum if it fits into float
            Err(_) => return Err(Exception::new(Reason::EXC_BADARITH)),
        };

        context.f[destination as usize] = val;
    },
    fn fadd(fail: l, a: L, b: L, destination: L) {
        op_float!(context, a, b, destination, +)
    },
    fn fsub(fail: l, a: L, b: L, destination: L) {
        op_float!(context, a, b, destination, -)
    },
    fn fmul(fail: l, a: L, b: L, destination: L) {
        op_float!(context, a, b, destination, *)
    },
    fn fdiv(fail: l, a: L, b: L, destination: L) {
        op_float!(context, a, b, destination, /)
    },
    fn fnegate(source: L, destination: L) {
        context.f[destination as usize] = -context.f[source as usize];
    },
    fn trim(nwords: r, _remaining: r) {
        let (n, cp) = context.callstack.pop().unwrap();
        context
            .stack
            .truncate(context.stack.len() - nwords as usize);
        context.callstack.push((n - nwords, cp));
    },
    fn make_fun2(i: u) {
        // nfree means capture N x-registers into the closure
        let module = context.ip.get_module();
        let lambda = &module.lambdas[i as usize];

        let binding = if lambda.nfree != 0 {
            Some(context.x[0..(lambda.nfree as usize)].to_vec())
        } else {
            None
        };

        context.x[0] = Term::closure(
            &context.heap,
            value::Closure {
                // arity is arity minus nfree (beam_emu.c)
                mfa: module::MFA(module.name, lambda.name, lambda.arity - lambda.nfree),
                // TODO: use module id instead later
                ptr: lambda.offset,
                binding,
            },
        );
    },
    fn call_fun(arity: t) {
        let value = context.x[arity as usize];
        context.cp = Some(context.ip);
        op_call_fun!(vm, context, process, value, arity)
    },
    fn get_hd(source: xy, destination: xy) {
        if let Ok(value::Cons { head, .. }) = #source.cast_into() {
            #destination = *head;
        } else {
            unreachable!()
        }
    },
    fn get_tl(source: xy, destination: xy) {
        if let Ok(value::Cons { tail, .. }) = #source.cast_into() {
            #destination = *tail;
        } else {
            unreachable!()
        }
    },
    fn put_map_assoc(_fail: l, map: s, destination: d, live: r, list: m) {
        let mut map = match (#map).cast_into() {
            Ok(value::Map(map)) => map.clone(),
            _ => unreachable!(),
        };

        let mut iter = list.chunks_exact(2);
        while let Some([key, value]) = iter.next() {
            // TODO: optimize by having the ExtendedList store Term instead of LValue
            map.insert(context.expand_entry(key), context.expand_entry(value));
        }
        context.set_register(destination, Term::map(&context.heap, map))
    },
    fn has_map_fields(fail: l, source: d, list: m) {
        let map = match (#source).cast_into() {
            Ok(value::Map(map)) => map.clone(),
            _ => unreachable!(),
        };

        // N is a list of the type [key => dest_reg], if any of these fields don't
        // exist, jump to fail label.
        let mut iter = list.iter();
        while let Some(key) = iter.next() {
            if map.contains_key(&context.expand_entry(key)) {
                // ok
            } else {
                op_jump!(context, fail);
                break;
            }
        }
    },
    fn get_map_elements(fail: l, source: d, list: m) {
        let map = match (#source).cast_into() {
            Ok(value::Map(map)) => map.clone(),
            _ => unreachable!(),
        };

        // N is a list of the type [key => dest_reg], if any of these fields don't
        // exist, jump to fail label.
        let mut iter = list.chunks_exact(2);
        while let Some([key, dest]) = iter.next() {
            if let Some(&val) = map.get(&context.expand_entry(key)) {
                context.set_register(dest.into_register(), val)
            } else {
                op_jump!(context, fail);
                break; // TODO: original impl loops over everything
            }
        }
    },
    fn put_map_exact(_fail: l, map: s, destination: d, live: r, list: m) {
        // TODO: in the future, put_map_exact is an optimization for flatmaps (tuple
        // maps), where the keys in the tuple stay the same, since you can memcpy the
        // key tuple (and other optimizations)
        let mut map = match #map.cast_into() {
            Ok(value::Map(map)) => map.clone(),
            _ => unreachable!(),
        };

        let mut iter = list.chunks_exact(2);
        while let Some([key, value]) = iter.next() {
            // TODO: optimize by having the ExtendedList store Term instead of LValue
            map.insert(context.expand_entry(key), context.expand_entry(value));
        }
        context.set_register(destination, Term::map(&context.heap, map))
    },
    fn is_tagged_tuple(fail: l, source: d, arity: u, atom: s) {
        // TODO optimize by pre-unpacking stuff
        let reg = #source;
        let atom = #atom;
        if let Ok(tuple) = Tuple::cast_from(&reg) {
            if tuple.len == 0 || tuple.len != arity || !tuple[0].eq(&atom) {
                fail!(context, fail);
            } else {
                // ok
            }
        } else {
            fail!(context, fail);
        }
    },
    fn build_stacktrace() {
        context.x[0] = exception::build_stacktrace(&process, context.x[0]);
    },
    fn raw_raise() {
        let class = &context.x[0];
        let value = context.x[1];
        let trace = context.x[2];

        match class.into_variant() {
            Variant::Atom(atom::ERROR) => {
                let mut reason = Reason::EXC_ERROR;
                reason.remove(Reason::EXF_SAVETRACE);
                return Err(Exception {
                    reason,
                    value,
                    trace,
                });
            }
            Variant::Atom(atom::EXIT) => {
                let mut reason = Reason::EXC_EXIT;
                reason.remove(Reason::EXF_SAVETRACE);
                return Err(Exception {
                    reason,
                    value,
                    trace,
                });
            }
            Variant::Atom(atom::THROW) => {
                let mut reason = Reason::EXC_THROWN;
                reason.remove(Reason::EXF_SAVETRACE);
                return Err(Exception {
                    reason,
                    value,
                    trace,
                });
            }
            _ => context.x[0] = Term::atom(atom::BADARG),
        }
    },
);
