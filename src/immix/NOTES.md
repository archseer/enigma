Global allocator allocates new blocks and recycles old blocks.

Individual process/mbox allocators usually have a ref to the global alloc + bucket.

A bucket is a sequence of blocks with objs of the same age.

bucket.allocate(global_alloc, obj)



Most register machines do still have a stack used for passing arguments to
functions and saving return addresses. BEAM has both a stack and registers, but
just as in WAM the stack slots are accessible through registers called
Y-registers. BEAM also has a number of X-registers, and a special register X0
(sometimes also called R0) which works as an accumulator where results are
stored.

The X registers are used as argument registers for function calls and register
X0 is used for the return value.

The X registers are stored in a C-array in the BEAM emulator and they are
globally accessible from all functions. The X0 register is cached in a local
variable mapped to a physical machine register in the native machine on most
architectures.

The Y registers are stored in the stack frame of the caller and only accessible
by the calling functions. To save a value across a function call BEAM allocates
a stack slot for it in the current stack frame and then moves the value to
a Y register.


The stack is used for keeping track of program execution by storing return
addresses, for passing arguments to functions, and for keeping local variables.
Larger structures, such as lists and tuples are stored on the heap.

only heap allocate lists, etc.


TUPLES should allocate as a len + ptr and be usable as a slice.
