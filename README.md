![Enigma](/enigma.png)

Enigma VM
=========

[![Build status](https://api.travis-ci.org/archseer/enigma.svg?branch=master)](https://travis-ci.org/archseer/enigma)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/archseer/enigma?svg=true)](https://ci.appveyor.com/project/archseer/enigma)

An implementation of the Erlang VM in Rust. We aim to be complete, correct and
fast, in that order of importance.

OTP 22+ compatible (sans the distributed bits for now) &mdash; all your code
should eventually run on Enigma unchanged. Deprecated opcodes won't be
supported.

## Why?

Because it's fun and I've been learning a lot. BEAM and HiPE are awesome, but
they're massive (~300k SLOC). A small implementation makes it easier for new
people to learn Erlang internals. We also get a platform to quickly iterate on
ideas for inclusion into BEAM.

## Installation

Only prerequisite to building Enigma is Rust. Use [rustup](https://rustup.rs/)
to install the latest nightly rust. At this time we don't support stable / beta
anymore, because we're relying on async/await, which is scheduled to run in
stable some time in Q3 2019.

To boot up OTP you will also need to compile the standard library. At the
moment, that relies on the BEAM build system:

```bash
git submodule update --init --depth 1
cd otp
/otp_build setup -a
make libs
make local_setup
```

We hope to simplify this step in the future (once enigma can run the compiler).

Run `cargo run` to install dependencies, build and run the VM. By default, it
will boot up the erlang shell ([iex also works, but has some rendering bugs](https://asciinema.org/a/zIjbf5AJx9YxEycyl9ij5ISVM)).

Expect crashes, but a lot of the functionality is already available.

Pre-built binaries for various platforms will be available, once we reach a certain level of stability.

## Feature status

We implement most of the opcodes, and about half of all BIFs. You can view
a detailed progress breakdown on [opcodes](/notes/opcodes.org) or [BIFs](/notes/bifs.org).

#### Roadmap

- [x] Get the full OTP kernel/stdlib to boot (`init:start`).
- [x] Get the Eshell to run.
- [x] Get OTP tests to run (works with a custom runner currently).
- [x] Get the erlang compiler to work.
- [x] Get IEx to run.
- [ ] Get OTP tests to pass.

#### Features

- [x] Floating point math
- [x] Spawn & message sending
- [x] Lambdas / anonymous functions
- [x] Exceptions & stack traces
- [x] Process dictionary
- [x] Signal queue
- [x] Links & monitors
- [ ] Timers
- [x] Maps
- [x] Binaries
- [ ] File IO
    - [x] open/read/close/read_file/write
    - [ ] Filesystem interaction
- [ ] [External NIFs](http://erlang.org/doc/man/erl_nif.html)
- [ ] Ports (might never be fully supported, we provide a few boot-critical ones as builtins: tty, fd)
- [ ] External Term representation
  - [x] Decoding
  - [ ] Encoding
- [ ] ETS
  - [x] PAM implementation
  - [x] All table types partially, but we do not provide any concurrency guarantees
- [ ] Regex (some support exists for basic matching)
- [ ] Garbage Collection (arena-based per-process at the moment)
- [ ] inet via socket nifs
- [ ] Code reloading
- [ ] Tracing/debugging support
- [ ] Load-time instruction transform
- [x] Load-time instruction specialization engine

### Goals, ideas & experiments

Process scheduling is implemented on top of rust futures:
- A process is simply a long running future, scheduled on top of
    tokio-threadpool work-stealing queue
- A timer is a delay/timeout future relying on tokio-timer time-wheel
- Ports are futures we can await on
- File I/O is AsyncRead/AsyncWrite awaitable
- NIF/BIFs are futures that yield at certain points to play nice with reductions
    (allows a much simpler yielding implementation)

Future possibilities:
- Write more documentation about more sparsely documented BEAM aspects (binary matching, time wheel, process monitors, ...)
- Explore using immix as a GC for Erlang
- Eir runtime
- JIT via Eir
- BIF as a generator function (yield to suspend/on reduce)
- Provide built-in adapter modules for [hyper](https://github.com/hyperium/hyper) as a Plug Adapter / HTTP client.
- Cross-compile to WebAssembly ([runtime](https://github.com/rustasync/runtime/))

#### Initial non-goals

Until the VM doesn't reach a certain level of completeness, it doesn't make sense to consider these.

- Distributed Erlang nodes
- Tracing / debugging support
- BEAM compatible NIFs / FFI

Note: NIF/FFI ABI compatibility with OTP is going to be quite some work. But,
a rust-style NIF interface will be available. It would also probably be possible
to make an adapter compatible with [rustler](https://github.com/rusterlium/rustler).

## Contributing

Contributors are very welcome!

The easiest way to get started is to look at the `notes` folder and pick a BIF
or an opcode to implement. Take a look at `src/bif.rs` and the `bif` folder on
how other BIFs are implemented. There's also a few issues open with the `good
first issue` tag, which would also be a good introduction to the internals.

Alternatively, search the codebase for `TODO`, `FIXME` or `unimplemented!`,
those mark various places where a partial implementation exists, but a bit more
work needs to be done.

Test coverage is currently lacking, and there's varying levels of documentation; I will be addressing these soon.

We also have a #enigma channel on the [Elixir Slack](https://elixir-slackin.herokuapp.com/).
