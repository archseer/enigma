use crate::atom;
use crate::arc_without_weak::ArcWithoutWeak;
use crate::bif;
use crate::module;
use crate::module_registry::{ModuleRegistry, RcModuleRegistry};
use crate::opcodes::Opcode;
use crate::pool::{Job, JoinGuard as PoolJoinGuard, Pool, Worker};
use crate::process::{self, ExecutionContext, RcProcess};
use crate::process_table::ProcessTable;
use crate::value::{self, Value};
use std::panic;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;

/// A reference counted State.
pub type RcState = Arc<State>;
/// Reference counted ModuleRegistry.

pub struct State {
    /// Table containing all processes.
    pub process_table: Mutex<ProcessTable<RcProcess>>,
    /// TODO: Use priorities later on
    pub process_pool: Pool<RcProcess>,

    /// The start time of the VM (more or less).
    pub start_time: time::Instant,
}

#[derive(Clone)]
pub struct Machine {
    /// VM internal state
    pub state: RcState,

    // env config, arguments, panic handler

    // atom table is accessible globally as ATOMS
    // export table
    /// Module registry
    pub modules: RcModuleRegistry,
}

macro_rules! set_register {
    ($context:expr, $register:expr, $value:expr) => {{
        match $register {
            Value::X(reg) => {
                $context.x[*reg] = $value;
            }
            Value::Y(reg) => {
                let len = $context.stack.len();
                $context.stack[len - (*reg + 2)] = $value;
            }
            _reg => unimplemented!(),
        }
    }};
}
macro_rules! op_deallocate {
    ($context:expr, $nwords:expr) => {{
        let cp = $context.stack.pop().unwrap();
        $context.stack.truncate($context.stack.len() - $nwords);
        if let Value::CP(cp) = cp {
            $context.cp = cp;
        } else {
            panic!("Bad CP value! {:?}", cp)
        }
    }};
}

macro_rules! op_call_ext {
    ($vm:expr, $context:expr, $process:expr, $arity:expr, $dest: expr) => {{
        let mfa = unsafe { &(*$context.module).imports[*$dest] };

        // TODO: precompute which exports are bifs
        // also precompute the bif lookup
        // call_ext_only Ar=u Bif=u$is_bif => \
        // allocate u Ar | call_bif Bif | deallocate_return u
        if bif::is_bif(mfa) {
            // make a slice out of arity x registers
            let args = &$context.x[0..*$arity];
            let val = bif::apply($vm, $process, mfa, args).unwrap(); // TODO handle fail
            set_register!($context, &Value::X(0), val); // HAXX
            op_return!($context);
        } else {
            unimplemented!()
        }
    }};
}

macro_rules! op_return {
    ($context:expr) => {{
        if let Some(i) = $context.cp {
            op_jump!($context, i);
            $context.cp = None;
        } else {
            println!("Process exited with normal, x0: {}", $context.x[0]);
            break;
        }
    }};
}

macro_rules! op_jump {
    ($context:expr, $label:expr) => {{
        $context.ip = $label;
    }};
}

macro_rules! op_is_type {
    ($vm:expr, $context:expr, $args:expr, $op:ident) => {{
        debug_assert_eq!($args.len(), 2);

        let val = $vm.expand_arg($context, &$args[1]);

        if !val.$op() {
            // TODO: patch the labels to point to exact offsets to avoid labels lookup
            let fail = $args[0].to_usize();

            op_jump!($context, fail);
        }
    }};
}

macro_rules! safepoint_and_reduce {
    ($vm:expr, $process:expr, $reductions:expr) => {{
        // if $vm.gc_safepoint(&$process) {
        //     return Ok(());
        // }

        // Reduce once we've exhausted all the instructions in a
        // context.
        if $reductions > 0 {
            $reductions -= 1;
        } else {
            $vm.state
                .process_pool
                .schedule(Job::normal($process.clone()));
            return Ok(());
        }
    }};
}

impl Machine {
    pub fn new() -> Machine {
        let primary_threads = 8;
        let process_pool = Pool::new(primary_threads, Some("primary".to_string()));

        let state = State {
            process_table: Mutex::new(ProcessTable::new()),
            process_pool,
            start_time: time::Instant::now(),
        };

        Machine {
            state: Arc::new(state),
            modules: ModuleRegistry::with_rc(),
        }
    }

    /// Starts the VM
    ///
    /// This method will block the calling thread until it returns.
    ///
    /// This method returns true if the VM terminated successfully, false
    /// otherwise.
    pub fn start(&self, file: &str) {
        //self.configure_rayon();

        let primary_guard = self.start_primary_threads();

        self.start_main_process(file);

        // Joining the pools only fails in case of a panic. In this case we
        // don't want to re-panic as this clutters the error output.
        if primary_guard.join().is_err() {
            println!("Primary guard error!")
            //self.set_exit_status(1);
        }
    }

    fn terminate(&self) {
        self.state.process_pool.terminate();
    }

    fn start_primary_threads(&self) -> PoolJoinGuard<()> {
        let machine = self.clone();
        let pool = &self.state.process_pool;

        pool.run(move |worker, process| machine.run_with_error_handling(worker, &process))
    }

    /// Starts the main process
    pub fn start_main_process(&self, path: &str) {
        let process = {
            let module = module::load_module(&self.modules, path).unwrap();

            process::allocate(&self.state, module).unwrap()
        };

        /* TEMP */
        let context = process.context_mut();

        // let fun = atom::from_str("fib");
        // let arity = 1;
        // context.x[0] = Value::Integer(23); // 28
        let fun = atom::from_str("start");
        let arity = 0;
        unsafe { op_jump!(context, (*context.module).funs[&(fun, arity)]) }
        // unsafe { println!("ins: {:?}", (*context.module).instructions) };
        /* TEMP */

        let process = Job::normal(process);
        self.state.process_pool.schedule(process);
    }

    #[inline]
    fn expand_arg<'a>(&'a self, context: &'a ExecutionContext, arg: &'a Value) -> &Value {
        match arg {
            // TODO: optimize away into a reference somehow at load time
            Value::ExtendedLiteral(i) => unsafe { &(*context.module).literals[*i] },
            Value::X(i) => &context.x[*i],
            Value::Y(i) => &context.stack[context.stack.len() - (*i + 2)],
            value => value,
        }
    }

    /// Executes a single process, terminating in the event of an error.
    pub fn run_with_error_handling(&self, worker: &mut Worker, process: &RcProcess) {
        // We are using AssertUnwindSafe here so we can pass a &mut Worker to
        // run()/panic(). This might be risky if values captured are not unwind
        // safe, so take care when capturing new variables.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            if let Err(message) = self.run(process) {
                //self.panic(worker, process, &message);
                panic!(message);
            }
        }));

        if let Err(error) = result {
            if let Ok(message) = error.downcast::<String>() {
                //self.panic(worker, process, &message);
                panic!(message);
            } else {
                panic!("The VM panicked with an unknown error");
                /*
                self.panic(
                    worker,
                    process,
                    &"The VM panicked with an unknown error",
                );*/
            };
        }
    }

    pub fn run(&self, process: &RcProcess) -> Result<(), String> {
        let mut reductions = 2000; // self.state.config.reductions;
        let context = process.context_mut();

        // println!(
        //     "running proc pid {:?}, offset {:?}",
        //     process.pid, context.ip
        // );
        loop {
            let ins = unsafe { &(*context.module).instructions[context.ip] };
            // println!("running proc pid {:?}, ins {:?}", process.pid, ins.op);
            context.ip += 1;
            match &ins.op {
                Opcode::FuncInfo => {}//println!("Running a function..."),
                Opcode::Return => {
                    op_return!(context);
                }
                Opcode::Send => {
                    // send x1 to x0, write result to x0
                    let pid = &context.x[0];
                    let msg = &context.x[1];
                    let res = process::send_message(&self.state, process, pid, msg)?;
                    context.x[0] = res.clone();
                }
                Opcode::RemoveMessage => {
                    // Unlink the current message from the message queue. Remove any timeout.
                    process.local_data_mut().mailbox.remove();
                    // TODO: clear timeout
                }
                Opcode::Timeout => {
                    //  Reset the save point of the mailbox and clear the timeout flag.
                    process.local_data_mut().mailbox.reset();
                    // TODO: clear timeout
                }
                Opcode::LoopRec => { // label, source
                    // grab message from queue, put to x0, if no message, jump to fail label
                    if let Some(msg) = process.local_data_mut().mailbox.receive() {
                        // TODO: this is very hacky
                        unsafe { context.x[0] = (**msg).clone(); }
                    } else {
                        let fail = self.expand_arg(context, &ins.args[0]).to_usize();
                        op_jump!(context, fail);
                    }
                }
                Opcode::LoopRecEnd => { // label
                    // Advance the save pointer to the next message and jump back to Label.
                    debug_assert_eq!(ins.args.len(), 1);

                    process.local_data_mut().mailbox.advance();

                    let label = self.expand_arg(context, &ins.args[0]).to_usize();
                    op_jump!(context, label);
                }
                Opcode::Wait => { // label
                    // jump to label, set wait flag on process
                    debug_assert_eq!(ins.args.len(), 1);

                    let label = self.expand_arg(context, &ins.args[0]).to_usize();
                    op_jump!(context, label);

                    // TODO: this currently races if the process is sending us
                    // a message while we're in the process of suspending.

                    // set wait flag
                    process.set_waiting_for_message(true);
                    // return (suspend process)
                    return Ok(());

                }
                Opcode::WaitTimeout => {
                    // @spec wait_timeout Lable Time
                    // @doc  Sets up a timeout of Time milliseconds and saves the address of the
                    //       following instruction as the entry point if the timeout triggers.

                    // TODO: timeout and jump to label if time expires
                    // set wait flag
                    process.set_waiting_for_message(true);
                    // TODO: return (suspend process)
                }
                // TODO: RecvMark(label)/RecvSet(label) for ref based sends
                Opcode::Call => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i)] = &ins.args[..] {
                        context.cp = Some(context.ip);
                        op_jump!(context, *i - 1);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallLast => {
                    //literal arity, label jmp, nwords
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i), Value::Literal(nwords)] = &ins.args[..] {
                        op_deallocate!(context, nwords);

                        op_jump!(context, *i - 1);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallOnly => {
                    //literal arity, label jmp
                    // store arity as live
                    if let [Value::Literal(_a), Value::Label(i)] = &ins.args[..] {
                        op_jump!(context, *i - 1);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallExt => {
                    //literal arity, literal destination (module.imports index)
                    if let [Value::Literal(arity), Value::Literal(dest)] = &ins.args[..] {
                        // save pointer onto CP
                        context.cp = Some(context.ip);

                        op_call_ext!(self, context, process, arity, dest);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallExtOnly => {
                    //literal arity, literal destination (module.imports index)
                    if let [Value::Literal(arity), Value::Literal(dest)] = &ins.args[..] {
                        op_call_ext!(self, context, process, arity, dest);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::CallExtLast => {
                    //literal arity, literal destination (module.imports index), literal deallocate
                    if let [Value::Literal(arity), Value::Literal(dest), Value::Literal(nwords)] = &ins.args[..] {
                        op_deallocate!(context, nwords);

                        op_call_ext!(self, context, process, arity, dest);
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                    safepoint_and_reduce!(self, process, reductions);
                }
                Opcode::Bif0 => {
                    // literal export, x reg
                    if let [Value::Literal(dest), reg] = &ins.args[..] {
                        let mfa = unsafe { &(*context.module).imports[*dest] };
                        let val = bif::apply(self, process, mfa, &[]).unwrap(); // TODO handle fail
                        set_register!(context, reg, val); // HAXX
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::Allocate => {
                    // stackneed, live
                    if let [Value::Literal(stackneed), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*stackneed {
                            context.stack.push(Value::Nil())
                        }
                        context.stack.push(Value::CP(context.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::AllocateHeap => {
                    // literal stackneed, literal heapneed, literal live
                    // allocate stackneed space on stack, ensure heapneed on heap, if gc, keep live
                    // num of X regs. save CP on stack.
                    if let [Value::Literal(stackneed), Value::Literal(_heapneed), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*stackneed {
                            context.stack.push(Value::Nil())
                        }
                        // TODO: check heap for heapneed space!
                        context.stack.push(Value::CP(context.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::AllocateZero => {
                    // literal stackneed, literal live
                    if let [Value::Literal(need), Value::Literal(_live)] = &ins.args[..] {
                        for _ in 0..*need {
                            context.stack.push(Value::Nil())
                        }
                        context.stack.push(Value::CP(context.cp));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                // AllocateHeapZero
                // TestHeap
                // Init
                Opcode::Deallocate => {
                    // literal nwords
                    if let [Value::Literal(nwords)] = &ins.args[..] {
                        op_deallocate!(context, nwords)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::IsLt => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = self.expand_arg(context, &ins.args[1]);
                    let v2 = self.expand_arg(context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Less) = v1.partial_cmp(v2) {
                        // ok
                    } else {
                        let fail = self.expand_arg(context, &ins.args[0]).to_usize();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsEq => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = self.expand_arg(context, &ins.args[1]);
                    let v2 = self.expand_arg(context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.partial_cmp(v2) {
                        // ok
                    } else {
                        let fail = self.expand_arg(context, &ins.args[0]).to_usize();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsNe => {
                    debug_assert_eq!(ins.args.len(), 3);

                    let v1 = self.expand_arg(context, &ins.args[1]);
                    let v2 = self.expand_arg(context, &ins.args[2]);

                    if let Some(std::cmp::Ordering::Equal) = v1.partial_cmp(v2) {
                        let fail = self.expand_arg(context, &ins.args[0]).to_usize();
                        op_jump!(context, fail);
                    }
                }
                Opcode::IsInteger      => { op_is_type!(self, context, ins.args, is_integer) }
                Opcode::IsFloat        => { op_is_type!(self, context, ins.args, is_float) }
                Opcode::IsNumber       => { op_is_type!(self, context, ins.args, is_number) }
                Opcode::IsAtom         => { op_is_type!(self, context, ins.args, is_atom) }
                Opcode::IsPid          => { op_is_type!(self, context, ins.args, is_pid) }
                Opcode::IsReference    => { op_is_type!(self, context, ins.args, is_ref) }
                Opcode::IsPort         => { op_is_type!(self, context, ins.args, is_port) }
                Opcode::IsNil          => { op_is_type!(self, context, ins.args, is_nil) }
                Opcode::IsBinary       => { op_is_type!(self, context, ins.args, is_binary) }
                Opcode::IsList         => { op_is_type!(self, context, ins.args, is_list) }
                Opcode::IsNonemptyList => { op_is_type!(self, context, ins.args, is_non_empty_list) }
                Opcode::IsTuple        => { op_is_type!(self, context, ins.args, is_tuple) }
                Opcode::IsFunction     => { op_is_type!(self, context, ins.args, is_function) }
                Opcode::IsBoolean      => { op_is_type!(self, context, ins.args, is_boolean) }
                Opcode::IsFunction2    => {
                    if let Value::Closure(closure) = self.expand_arg(context, &ins.args[0]) {
                        let arity = self.expand_arg(context, &ins.args[1]).to_usize();
                        unsafe {
                            if (**closure).mfa.2 == arity {
                                continue;
                            }
                        }
                    }
                    let fail = self.expand_arg(context, &ins.args[2]).to_usize();
                    op_jump!(context, fail);
                }
                Opcode::TestArity => {
                    // check tuple arity
                    if let [Value::Label(fail), arg, Value::Literal(arity)] = &ins.args[..] {
                        if let Value::Tuple(t) = self.expand_arg(context, arg) {
                            unsafe {
                                if (**t).len() != *arity {
                                    op_jump!(context, *fail);
                                }
                            }
                        } else {
                            panic!("Bad argument to {:?}", ins.op)
                        }

                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::SelectVal => {
                    // arg, fail, dests
                    // loop over dests
                    if let [arg, Value::Label(fail), Value::ExtendedList(vec)] = &ins.args[..] {
                        let arg = self.expand_arg(context, arg);
                        let mut i = 0;
                        loop {
                            // if key matches, jump to the following label
                            if vec[i] == *arg {
                                let label = vec[i+1].to_usize();
                                op_jump!(context, label);
                                break;
                            }

                            i += 2;

                            // if we ran out of options, jump to fail
                            if i >= vec.len() {
                                op_jump!(context, *fail);
                                break;
                            }
                        }
                    }
                }
                Opcode::Jump => {
                    debug_assert_eq!(ins.args.len(), 1);
                    let label = self.expand_arg(context, &ins.args[0]).to_usize();
                    op_jump!(context, label)
                }
                Opcode::Move => {
                    // arg1 can be either a value or a register
                    let val = self.expand_arg(context, &ins.args[0]);
                    set_register!(context, &ins.args[1], val.clone()) // TODO: mem::move would be preferred
                }
                Opcode::GetList => {
                    // source, head, tail
                    if let Value::List(cons) = self.expand_arg(context, &ins.args[0]) {
                        let head = unsafe { (**cons).head.clone() };
                        let tail = unsafe { (**cons).tail.clone() };
                        set_register!(context, &ins.args[1], head);
                        set_register!(context, &ins.args[2], tail);
                    } else {
                        panic!("badarg to GetHd")
                    }
                }
                Opcode::GetTupleElement => {
                    // source, element, dest
                    let source = self.expand_arg(context, &ins.args[0]);
                    let n = self.expand_arg(context, &ins.args[1]).to_usize();
                    if let Value::Tuple(t) = source {
                        let elem = unsafe {
                            let slice: &[Value] = &(**t);
                            slice[n].clone()
                        };
                        set_register!(context, &ins.args[2], elem)
                    } else {
                        panic!("GetTupleElement: source is of wrong type")
                    }
                }
                Opcode::PutList => {
                    // put_list H T Dst::slot()
                    // Creates a cons cell with [H|T] and places the value into Dst.
                    let head = self.expand_arg(context, &ins.args[0]).clone();
                    let tail = self.expand_arg(context, &ins.args[1]).clone();
                    let cons = context.heap.alloc(value::Cons { head, tail });
                    set_register!(context, &ins.args[2], Value::List(cons))
                }
                Opcode::PutTuple => {
                    // put_tuple dest size
                    // followed by multiple put() ops (put val [potentially regX/Y])
                    unimplemented!()
                }
                Opcode::GcBif1 => {
                    // fail label, live, bif, arg1, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[
                            self.expand_arg(context, &ins.args[3]).clone(),
                        ];
                        let val = unsafe { bif::apply(self, process, &(*context.module).imports[*i], &args[..]).unwrap() }; // TODO: handle fail

                        // TODO: consume fail label if not 0

                        set_register!(context, &ins.args[4], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::BsAdd => {
                     // Calculates the total of the number of bits in Src1 and the number of units in Src2. Stores the result to Dst.
                     // bs_add(Fail, Src1, Src2, Unit, Dst)
                     // dst = (src1 + src2) * unit

                     // TODO: trickier since we need to check both nums are positive and can fit
                     // into int/bigint

                     // optimize when one append is 0 and unit is 1, it's just a move
                     // bs_add Fail S1=i==0 S2 Unit=u==1 D => move S2 D

                    if let [Value::Label(_fail), s1, s2, Value::Literal(unit), dest] = &ins.args[..] {
                        // TODO use fail label
                        let s1 = self.expand_arg(context, s1).to_usize();
                        let s2 = self.expand_arg(context, s2).to_usize();

                        let res = Value::Integer(((s1 + s2) * (*unit)) as i64);
                        set_register!(context, dest, res)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }

                }
                Opcode::BsInit2 => {
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

                    if let [Value::Label(_fail), s1, Value::Literal(_words), Value::Literal(_live), _flags, dest] = &ins.args[..] {
                        // TODO: use a current_string ptr to be able to write to the Arc wrapped str
                        // alternatively, loop through the instrs until we hit a non bs_ instr.
                        // that way, no unsafe ptrs!
                        let size = self.expand_arg(context, s1).to_usize();
                        let mut arc = ArcWithoutWeak::new(String::with_capacity(size));
                        context.bs = &mut (*arc); // TODO: this feels a bit off
                        set_register!(context, dest, Value::Binary(arc));
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::BsPutString => {
                    // BsPutString uses the StrT strings table! needs to be patched in loader

                    // needs a build context
                    if let Value::Binary(str) = &ins.args[0] {
                        unsafe { (*context.bs).push_str(&*str); }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::BsPutBinary => {
                    if let [Value::Label(fail), size, Value::Literal(unit), _flags, src] = &ins.args[..] {
                        // TODO: fail label
                        if *unit != 8 { unimplemented!() }

                        if let Value::Binary(str) = self.expand_arg(context, src) {
                            match size {
                                Value::Atom(atom::ALL) => {
                                    unsafe { (*context.bs).push_str(&*str); }
                                }
                                _ => unimplemented!()
                            }
                        } else {
                            panic!("Bad argument to {:?}", ins.op)
                        }
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::BsPutFloat => {
                    // gen_put_float(GenOpArg Fail,GenOpArg Size, GenOpArg Unit, GenOpArg Flags, GenOpArg Src)
                    // Size can be atom all
                    unimplemented!()
                }
                Opcode::BsPutInteger => {
                    // gen_put_integer(GenOpArg Fail,GenOpArg Size, GenOpArg Unit, GenOpArg Flags, GenOpArg Src)
                    // Size can be atom all
                    unimplemented!()
                }
                // BsGet and BsSkip should be implemented over an Iterator inside a match context (.skip/take)
                // maybe we can even use nom for this
                Opcode::BsAppend => { // append and init also sets the string as current (state.current_binary) [seems to be used to copy string literals too]

                    // bs_append Fail Size Extra Live Unit Bin Flags Dst => \
                    //   move Bin x | i_bs_append Fail Extra Live Unit Size Dst

                    if let [Value::Label(fail), Value::Integer(size), Value::Literal(extra_heap), Value::Literal(live), Value::Literal(unit), src, _flags, dest] = &ins.args[..] {
                        // TODO: execute fail if non zero
                        // unit: byte alignment (8 for binary)
                        //
                        // size * unit = total_bytes?

                        // make sure it's a binary otherwise badarg
                        // make sure it's a sub binary and it's writable
                        // if so, expand the current string and make it non-writable
                        // else, copy it into a new string, append and return a sub-binary refering to it

                    } else {
                        panic!("badarg for BsAppend")
                    }

                    // If the binary is a writable subbinary
                    // referencing a ProcBin with enough empty space then a new writable subbinary
                    // is created and the old one is made non-writable. In other cases, creates
                    // a new shared data, a new ProcBin, and a new subbinary. For all heap
                    // allocation, a space for more Arg1 words are requested. Arg2 is Live. Arg3 is
                    // unit. Saves the resultant subbinary to Arg4.
                }
                Opcode::GcBif2 => {
                    // fail label, live, bif, arg1, arg2, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[
                            self.expand_arg(context, &ins.args[3]).clone(),
                            self.expand_arg(context, &ins.args[4]).clone(),
                        ];
                        let val = unsafe { bif::apply(self, process, &(*context.module).imports[*i], &args[..]).unwrap() }; // TODO: handle fail

                        // TODO: consume fail label if not 0

                        set_register!(context, &ins.args[5], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::GcBif3 => {
                    // fail label, live, bif, arg1, arg2, arg3, dest
                    if let Value::Literal(i) = &ins.args[2] {
                        // TODO: GcBif needs to handle GC as necessary
                        let args = &[
                            self.expand_arg(context, &ins.args[3]).clone(),
                            self.expand_arg(context, &ins.args[4]).clone(),
                            self.expand_arg(context, &ins.args[5]).clone(),
                        ];
                        let val = unsafe { bif::apply(self, process, &(*context.module).imports[*i], &args[..]).unwrap() }; // TODO: handle fail

                        // TODO: consume fail label if not 0

                        set_register!(context, &ins.args[6], val)
                    } else {
                        panic!("Bad argument to {:?}", ins.op)
                    }
                }
                Opcode::Trim => {
                    // trim N, _remain
                    // drop N words from stack, (but keeping the CP). Second arg unused?
                    let nwords = ins.args[0].to_usize();
                    let cp = context.stack.pop().unwrap();
                    context.stack.truncate(context.stack.len() - nwords);
                    context.stack.push(cp);
                }
                Opcode::MakeFun2 => {
                    // literal n -> points to lambda
                    // nfree means capture N x-registers into the closure
                    let i = ins.args[0].to_usize();
                    let lambda = unsafe { &(*context.module).lambdas[i] };
                    println!("make_fun2 args: {:?}", lambda);

                    let binding = if lambda.nfree != 0 {
                        Some(context.x[0..(lambda.nfree as usize)].to_vec())
                    } else {
                        None
                    };

                    let closure = context.heap.alloc(value::Closure {
                        mfa: (0, lambda.name as usize, lambda.arity as usize), // TODO: module_id
                        ptr: lambda.offset,
                        binding
                    });
                    context.x[0] = Value::Closure(closure);
                }
                Opcode::CallFun => {
                    // literal arity
                    let arity = ins.args[0].to_usize();
                    if let Value::Closure(closure) = &context.x[arity] {
                        // store ip in cp
                        context.cp = Some(context.ip);
                        // keep X regs set based on arity
                        // set additional X regs based on lambda.binding
                        // set x from 1 + arity (x0 is func, followed by call params) onwards to binding
                        unsafe {
                            let closure = *closure;
                            if let Some(binding) = &(*closure).binding {
                                // TODO: maybe we can copy_from_slice in the future
                                context.x[arity..arity+binding.len()].clone_from_slice(&binding[..]);
                            }

                            op_jump!(context, (*closure).ptr);
                        }
                    } else {
                        panic!("badarg to CallFun")
                    }
                }
                Opcode::GetHd => {
                    // source head
                    if let Value::List(cons) = self.expand_arg(context, &ins.args[0]) {
                        let val = unsafe { (**cons).head.clone() };
                        set_register!(context, &ins.args[1], val);
                    } else {
                        panic!("badarg to GetHd")
                    }
                }
                Opcode::GetTl => {
                    // source head
                    if let Value::List(cons) = self.expand_arg(context, &ins.args[0]) {
                        let val = unsafe { (**cons).tail.clone() };
                        set_register!(context, &ins.args[1], val);
                    } else {
                        panic!("badarg to GetHd")
                    }
                }
                opcode => println!("Unimplemented opcode {:?}: {:?}", opcode, ins),
            }
        }

        // Terminate once the main process has finished execution.
        if process.is_main() {
            self.terminate();
        }

        Ok(())
    }

    pub fn elapsed_time(&self) -> time::Duration {
        self.state.start_time.elapsed()
    }
}
