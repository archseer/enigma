![Enigma](/enigma.png)

Enigma VM
=========

[![Build status](https://api.travis-ci.org/archseer/enigma.svg?branch=master)](https://travis-ci.org/archseer/enigma)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/archseer/enigma?svg=true)](https://ci.appveyor.com/project/archseer/enigma)

An implementation of the Erlang VM in Rust. We aim to be complete, correct and fast (in that order of importance). However my TotallySerious™ fibonacci microbenchmarks are currently on-par with OTP (but I'm missing 99% of the runtime :)

OTP 22+ compatible (sans the distributed bits for now) &mdash; all your code should eventually run on Enigma unchanged. Deprecated opcodes won't be supported.

## Why?

Because it's fun and I've been learning a lot. BEAM and HiPE are awesome, but
they're massive (~300k SLOC). A small implementation makes it easier for new
people to learn Erlang internals. We also get a platform to quickly iterate on
ideas for inclusion into BEAM.

##### Why Rust?

I read the BEAM book followed by the Rust book. Two birds with one stone?

## Installation

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

Run `cargo run` to install dependencies, build and run the VM. By default, it
will try to boot up the erl shell ([which boots, but its currently semi-functional](https://asciinema.org/a/yVKI5dAdDXGq11azUSjbQV42y)).

Expect crashes, but a lot of the functionality is already available.

We will distribute binaries for various platforms, once we reach a certain level of usability.

## Goals, ideas & experiments

Process scheduling is implemented on top of rust futures:
- A process is simply a long running future, scheduled on top of
    tokio-threadpool work-stealing queue
- A timer is a delay/timeout future relying on tokio-timer time-wheel
- Ports are futures we can await on
- File I/O is AsyncRead/AsyncWrite awaitable
- NIF/BIFs are futures that yield at certain points to play nice with reductions
    (allows a much simpler yielding implementation)

Future possibilities:
- Be able to run Elixir
- Write more documentation about more sparsely documented BEAM aspects (binary matching, time wheel, process monitors, etc).
- Feature parity with OTP
- Explore using immix as a GC for Erlang
- BIF as a generator function (yield to suspend/on reduce)
- Provide built-in adapter modules for [hyper](https://github.com/hyperium/hyper) as a Plug Adapter / HTTP client.
- Cross-compile to WebAssembly ([threading](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html) is coming)
- Use Commentz-Walter for binary matching. ["Commentz-Walter is an algorithm that combines Aho-Corasick with Boyer-Moore. (Only implementation I know of is in GNU grep.)"](https://github.com/rust-lang/regex/issues/197))

### Initial non-goals

Until the VM doesn't reach a certain level of completeness, it doesn't make sense to consider these.

- Distributed Erlang nodes
- Tracing / debugging support
- BEAM compatible NIFs / FFI

Note: NIF/FFI ABI compatibility with OTP is going to be quite some work. But,
a rust-style NIF interface will be available. It would also probably be possible
to make an adapter compatible with [rustler](https://github.com/rusterlium/rustler).

## Feature status

You can view a detailed progress breakdown on [opcodes](/notes/opcodes.org) or [BIFs](/notes/bifs.org).

#### Roadmap

- [x] Implement enough instructions and BIFs to get the preloads to load.
- [x] Get the full OTP kernel/stdlib to boot (`init:start`).
- [x] Get the Eshell to run.
- [ ] Get IEx to run.
- [ ] Get OTP tests to run.

#### Features

- [x] Floating point math ([float registers](https://pdfs.semanticscholar.org/7347/354eaaad96d40e12ea4373178b784fc39bfc.pdf))
- [x] Spawn & message sending
- [x] Lambdas / anonymous functions
- [x] Stack traces
- [x] Exceptions
- [x] Process dictionary
- [x] Signal queue
- [x] Links & monitors
- [x] error_handler system hooks (export stubs)
- [x] Deep term comparison (lists, tuples, maps)
- [ ] Timers
- [x] Maps
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
    - [x] Basic read_file
    - [x] File open/read/close
    - [ ] Filesystem interaction
- [ ] [Externally loaded NIFs](http://erlang.org/doc/man/erl_nif.html)
- [ ] Ports
    - [x] ttysl_drv
    - [ ] spawn_drv
    - [ ] fd_drv
    - [ ] vanilla_driver ?
    - [ ] forker_driver
    - [ ] inet_drv
    - [ ] ram_file_drv (we currently override the few functions to use rust based NIFs)
- [ ] External Term representation
  - [x] Decoding
  - [ ] Encoding
- [ ] ETS
  - [x] PAM implementation
  - [x] All table types partially, but we do not provide any concurrency guarantees
- [ ] Regex (some support exists for basic matching)
- [ ] GC!
- [ ] Code reloading
- [ ] Tracing/debugging support
- [ ] beam_makeops compatible load-time opcode transformer

## Contributing

Contributors are very welcome!

The easiest way to get started is to look at the `notes` folder and pick a BIF or an opcode to implement. Take a look at `src/bif.rs` and the `bif` folder on how other BIFs are implemented. There's also a few issues open with the `good first issue` tag, which would be a good introduction to the codebase.

Alternatively, search the codebase for `TODO`, `FIXME` or `unimplemented!`,
those mark various places where a partial implementation exists, but a bit more
work needs to be done.

Test coverage is currently lacking, and there's varying levels of documentation; I will be addressing these as soon as I solidify the core data structures.

We also have a #enigma channel on the [Elixir Slack](https://elixir-slackin.herokuapp.com/).
