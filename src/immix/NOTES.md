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


BINARIES:


/// Maximum number of bytes to place in a heap binary.
const ERL_ONHEAP_BIN_LIMIT: usize = 64;

http://www.erlang-factory.com/static/upload/media/1442407543231431thefunpartofwritingathriftcodec.pdf

# Constructing binaries

## creating new binary: two steps

1. allocate empty heap or RefC binary based on the required size
(bs_init, bs_init_writable beam ops)

bs_init2 Fail Sz Words Regs Flags Dst | binary_too_big(Sz) => system_limit Fail

2. write new data into it
bs_put_{integer,float,string,binary}

## appending to existing: two steps
1. make sure there is space in the end for new data (maybe expand binary and leave some extra space)
bs_append, bs_private_append beam ops
2. write new data to the end
bs_put_{integer,float,string,binary}

# Matching binaries

The native implementation functions for the module binary.
Searching is implemented using either Boyer-Moore or Aho-Corasick
depending on number of searchstrings (BM if one, AC if more than one).
Native implementation is mostly for efficiency, nothing
(except binary:referenced_byte_size) really *needs* to be implemented
in native code.

byte oriented searching

https://softwareengineering.stackexchange.com/questions/183725/which-string-search-algorithm-is-actually-the-fastest
https://github.com/rust-lang/regex/issues/197
http://www.cs.columbia.edu/~orestis/damon16.pdf
https://github.com/rust-lang/regex/issues/408

BM: https://github.com/ethanpailes/regex/commit/d2e28f959ac384db62f7cbeba1576cf39a75b294

_start_match2 - a match context is created (The match context points to the
first byte of the binary)
2.bs_skip_bits2 - 8 bit skipped (The match context is updated to point to the
second byte in the binary)
3.bs_get_binary - match out the remainder of the binary creating a SubBin
match context is not used anymore, it is garbage_collected at next gc run


1.bs_start_match2
2.bs_skip_bits2
3.-bs_get_binary- - no SubBin created (only size is checked that Bin1 is indeed a
binary and not a bitstring - bs_test_unit)
4. bs_start_match2 in match_byte - basically does nothing when it sees that it
was passed a match context instead of a binary.
5.bs_get_integer2 - 8 bit matched into an integer (The match context is updated)
6.bs_context_to_binary - in case of function_clause (3., 5. fails) the match context
is converted back to a binary (SubBin is created pointing to the binary object)
