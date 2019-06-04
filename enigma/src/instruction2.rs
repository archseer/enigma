use instruction_codegen::instruction;

// mandatory for loop
use crate::vm::{Machine, op_deallocate};
use crate::{atom, loader, port, bif, bitstring, module};
use crate::exception::{self, Exception, Reason};
use crate::exports_table::{Export};
use crate::instr_ptr::InstrPtr;
use crate::instruction::{self};
use crate::process::{self, RcProcess};
use crate::servo_arc::Arc;
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

use std::ops::Index;
use std::ops::IndexMut;

use crate::bif::Fn as BifFn;
use crate::instruction::{Regs, RegisterX, RegisterY, Register, Source, Label, Arity, Atom, ExtendedList, JumpTable};

// need three things:
// generic op definitions
// op definitions with type permutations
// transform rules

const APPLY_2: bif::Fn = bif::bif_erlang_apply_2;
const APPLY_3: bif::Fn = bif::bif_erlang_apply_3;

instruction!(
    fn func_info() {
        return Err(Exception::new(Reason::EXC_FUNCTION_CLAUSE));
    },
    fn jump(label: l) {
        op_jump!(context, label)
    },
    fn r#move(src: cxy, dest: xy) {
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
            // println!("recv proc pid={:?} msg={}", process.pid, msg);
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
    fn recv_mark() {
        process.local_data_mut().mailbox.mark();
    },
    fn recv_set() {
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

        op_call_ext!(vm, context, &process, arity, destination, false); // don't return on call_ext
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_ext_only(arity: t, destination: u) {
        op_call_ext!(vm, context, &process, arity, destination, true);
        safepoint_and_reduce!(vm, process, context.reds);
    },
    fn call_ext_last(arity: t, destination: u, words: r) {
        op_deallocate(context, words);

        op_call_ext!(vm, context, &process, arity, destination, true);
        safepoint_and_reduce!(vm, process, context.reds);
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
    fn test_heap() { // TODO args
        // println!("TODO: TestHeap unimplemented!");
    },
    fn init(n: d) {
        context.set_register(n, NIL)
    },
    fn deallocate(n: r) {
        op_deallocate(context, n)
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
    fn is_integer(fail: l, arg1: s) {
        if !#arg1.is_integer() {
            op_jump!(context, fail);
        }
    },
    fn is_float(fail: l, arg1: s) {
        if !#arg1.is_float() {
            op_jump!(context, fail);
        }
    },
    fn is_number(fail: l, arg1: s) {
        if !#arg1.is_number() {
            op_jump!(context, fail);
        }
    },
    fn is_atom(fail: l, arg1: s) {
        if !#arg1.is_atom() {
            op_jump!(context, fail);
        }
    },
    fn is_reference(fail: l, arg1: s) {
        if !#arg1.is_ref() {
            op_jump!(context, fail);
        }
    },
    fn is_port(fail: l, arg1: s) {
        if !#arg1.is_port() {
            op_jump!(context, fail);
        }
    },
    fn is_nil(fail: l, arg1: s) {
        if !#arg1.is_nil() {
            op_jump!(context, fail);
        }
    },
    fn is_binary(fail: l, arg1: s) {
        if !#arg1.is_binary() {
            op_jump!(context, fail);
        }
    },
    fn is_bitstr(fail: l, arg1: s) {
        if !#arg1.is_bitstring() {
            op_jump!(context, fail);
        }
    },
    fn is_list(fail: l, arg1: s) {
        if !#arg1.is_list() {
            op_jump!(context, fail);
        }
    },
    fn is_non_empty_list(fail: l, arg1: s) {
        if !#arg1.is_non_empty_list() {
            op_jump!(context, fail);
        }
    },
    fn is_tuple(fail: l, arg1: s) {
        if !#arg1.is_tuple() {
            op_jump!(context, fail);
        }
    },
    fn is_function(fail: l, arg1: s) {
        if !#arg1.is_function() {
            op_jump!(context, fail);
        }
    },
    fn is_boolean(fail: l, arg1: s) {
        if !#arg1.is_boolean() {
            op_jump!(context, fail);
        }
    },
    fn is_map(fail: l, arg1: s) {
        if !#arg1.is_map() {
            op_jump!(context, fail);
        }
    },
    fn is_function2(fail: l, arg1: s, arity: s) {
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
    fn test_arity(fail: l, arg1: s, arity: u) {
        // check tuple arity
        if let Ok(t) = Tuple::cast_from(&#arg1) {
            if t.len != arity {
                op_jump!(context, fail);
            }
        } else {
            panic!("Bad argument to TestArity")
        }
    },
    fn select_val(arg: s, fail: l, destinations: m) {
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
    fn select_tuple_arity(arg: s, fail: l, arities: m) {
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
    // TODO JumpOnVal
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
    fn badmatch(value: d) {
        let value = #value;
        println!("Badmatch: {}", value);
        return Err(Exception::with_value(Reason::EXC_BADMATCH, value));
    },
    fn if_end() {
        // Raises the if_clause exception.
        return Err(Exception::new(Reason::EXC_IF_CLAUSE));
    },
    fn case_end(value: d) {
        // Raises the case_clause exception with the value of Arg0
        let value = #value;
        println!("err=case_clause val={}", value);
        // err=case_clause val={{820515101, #Ref<0.0.0.105>}, :timeout, {:gen_server, :cast, [#Pid<61>, :repeated_filesync]}}
        return Err(Exception::with_value(Reason::EXC_CASE_CLAUSE, value));
    },

);
