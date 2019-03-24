![Enigma](/enigma.png)

Enigma VM
=========

[![Build status](https://api.travis-ci.org/archseer/enigma.svg?branch=master)](https://travis-ci.org/archseer/enigma)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/archseer/enigma?svg=true)](https://ci.appveyor.com/project/archseer/enigma)

An implementation of the Erlang VM in Rust. We aim to be complete, correct and fast (in that order of importance). However my TotallySerious™ fibonacci microbenchmarks are currently on-par with OTP (but I'm missing 99% of the runtime :)

OTP 22+ compatible (sans the distributed bits for now) &mdash; all your code should eventually run on Enigma unchanged. Deprecated opcodes won't be supported.

# Why?

Because it's fun and I've been learning a lot. BEAM and HiPE are awesome, but
they're massive (~300k SLOC). A small implementation makes it easier for new
people to learn Erlang internals. We also get a platform to quickly iterate on
ideas for inclusion into BEAM.

##### Why Rust?

I read the BEAM book followed by the Rust book. Two birds with one stone?

# Installation

Only prerequisite to building Enigma is Rust. Use [rustup](https://rustup.rs/)
to install the latest nightly rust. At this time we don't support stable / beta
anymore, because we're relying on async/await, which is scheduled to run in
stable some time in 2019.

To boot up OTP you will also need to compile the standard library first. At the
moment, that relies on the regular BEAM build system:

```bash
git submodule update --init --depth 1
cd otp
./otp_build autoconf
./otp_build configure
make libs
make local_setup
```

We hope to simplify this step in the future (once enigma can run the compiler).

Run `cargo install` to install the dependencies, `cargo run` to build and run
the VM. Expect crashes, but a lot of the functionality is already available.

We will distribute binaries for various platforms, once we reach a certain level of usability.

# Goals, ideas & experiments

Process scheduling is implemented on top of rust futures:
- A process is simply a long running future, scheduled on top of
    tokio-threadpool work-stealing queue
- A timer is a delay/timeout future relying on tokio-timer time-wheel
- Ports are futures we can await on
- File I/O is AsyncRead/AsyncWrite awaitable
- NIF/BIFs are futures that yield at certain points to play nice with reductions
    (allows a much simpler yielding implementation)

Future possibilities:
- Be able to run the Erlang bootstrap (and all OTP)
- Be able to run Elixir
- Write more documentation about more sparsely documented BEAM aspects (binary matching, time wheel, process monitors, etc).
- Feature parity with OTP
- Explore using immix as a GC for Erlang
- BIF as a generator function (yield to suspend/on reduce)
- Cross-compile to WebAssembly ([threading](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html) is coming)
- Use Commentz-Walter for binary matching. ["Commentz-Walter is an algorithm that combines Aho-Corasick with Boyer-Moore. (Only implementation I know of is in GNU grep.)"](https://github.com/rust-lang/regex/issues/197))

# Initial non-goals

Until we can run a large subset of OTP code, it doesn't make sense to consider these.

- Distributed Erlang nodes
- Tracing / debugging support
- BEAM compatible NIFs / FFI

Note: NIF/FFI compatibility with OTP is going to be quite some work. Until then,
a rust-style NIF interface will be available.

# Feature status

This section is a quick overview of what's supported, and what's the next general features that will be worked on.

You can view a detailed breakdown on [opcode](/notes/opcodes.org) or [BIF](/notes/bifs.org) progress.

Plan:

- [x] implement enough instructions to run bootstrap
- [x] implement enough BIFs to get preloaded bootstrap to load
- [ ] implement enough to get the full system to boot (`init:start`)
- [ ] get the REPL to run
- [ ] get OTP tests to run

Features:

- [x] Floating point math ([float registers](https://pdfs.semanticscholar.org/7347/354eaaad96d40e12ea4373178b784fc39bfc.pdf))
- [x] Spawn & message sending
- [x] Lambdas / anonymous functions
- [x] Stack traces
- [x] Exceptions
- [x] Process Dictionary
- [x] Links
- [x] Monitors
- [x] Signal queue
- [x] error_handler system hooks (export stubs)
- [ ] Deep term comparison (lists, tuples, maps)
- [ ] Timers
- [x] Maps
  - [x] Basic type implementation
  - [x] BIF functions
  - [x] Map specific opcodes
- [ ] Binaries
  - [x] Basic type implementation
  - [ ] Binary building
  - [x] Binary matching
  - [x] Bitstring (bit-level) matching
    - [ ] Combine repeated utf8 matches?
  - [ ] Binary searching
    - multi pattern via [aho-corasick](https://github.com/BurntSushi/aho-corasick)
    - single pattern via [boyer-moore](https://github.com/killerswan/boyer-moore-search) | [needle booyer-moore](https://docs.rs/needle/0.1.1/needle/) | [regex - booyer-moore](https://github.com/ethanpailes/regex/commit/d2e28f959ac384db62f7cbeba1576cf39a75b294)
- [ ] File IO
    - [x] basic read_file
- [ ] [NIF](http://erlang.org/doc/man/erl_nif.html)
- [ ] Ports
    - [ ] spawn_drv
    - [ ] fd_drv
    - [ ] vanilla_driver ?
    - [ ] forker_driver
    - [ ] inet_drv
    - [ ] ram_file_drv
- [ ] External Term representation
  - [x] Most of decoding
  - [ ] Encoding
- [ ] ETS
  - [x] basic PAM implementation
- [ ] GC!
- [ ] Code reloading
- [ ] Tracing/debugging support
- [ ] beam_makeops compatible load-time opcode transformer
- [ ] Optimize select_val with a jump table

# Contributing

Contributors are very welcome!

The easiest way to get started is to look at the `notes` folder and pick a BIF or an opcode to implement. Take a look at `src/bif.rs` and the `bif` folder on how other BIFs are implemented. There's also a few issues open with the `good first issue` tag, which would be a good introduction to the codebase.

Test coverage is currently lacking, and there's varying levels of documentation; I will be addressing these as soon as I solidify the core data structures.

We also have a #enigma channel on the [Elixir Slack](https://elixir-slackin.herokuapp.com/).
