![Enigma](/enigma.png)

Enigma VM
=========

[![Build status](https://api.travis-ci.org/archseer/enigma.svg?branch=master)](https://travis-ci.org/archseer/enigma)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/archseer/enigma?svg=true)](https://ci.appveyor.com/project/archseer/enigma)

An implementation of the Erlang VM in Rust. We aim to be complete, correct and fast (in that order of importance). However my TotallySerious™ fibonacci microbenchmarks are currently on-par with OTP (but I'm missing 99% of the runtime :)

We aim to be OTP 22+ compatible (sans the
distributed bits for now) &mdash; all your code should eventually run on Enigma unchanged. Deprecated opcodes won't be supported.

# Why?

Because it's fun and I've been learning a lot. BEAM and HiPE are awesome, but they're massive (~300k SLOC). A small implementation makes it easier for new people to learn Erlang internals. We also get a platform to quickly iterate on ideas for inclusion into BEAM.

##### Why Rust?

I read the BEAM book followed by the Rust book. Two birds with one stone?

# Installation

Only prerequisite to building Enigma is Rust. Use [rustup](https://rustup.rs/) (or your preferred package manager) to install latest rust (minimum version is the 2018 edition / ‎1.31, and stable is supported).

Run `cargo install` to install the dependencies, `cargo run` to build and run the VM. Expect heavy
crashes, but a basic spawn + send multi-process model already works.

A CI & release build system will be set up to distribute binaries for various platforms, once we reach a certain level of usability.

# Goals / ideas / experiments

- Be able to run the Erlang bootstrap (and all OTP)
- Be able to run Elixir
- Write more documentation about more sparsely documented BEAM aspects (binary matching, time wheel, process monitors, etc).
- Ideally one day, feature parity with OTP
- Explore using immix as a GC for Erlang
- Explore using RocksDB / Sled / embedded DB equivalent for the ETS implementation.
- Process as a generator function (yield to suspend/on reduce)
- Use Commentz-Walter for binary matching. ["Commentz-Walter is an algorithm that combines Aho-Corasick with Boyer-Moore. (Only implementation I know of is in GNU grep.)"](https://github.com/rust-lang/regex/issues/197))
- Cross-compile to WebAssembly ([threading](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html) is coming)

# Initial non-goals

Until we can run a large subset of OTP code, it doesn't make sense to consider these.

- Distributed Erlang nodes
- Tracing / debugging support
- NIFs / FFI

Note: NIF/FFI compatibility with OTP is going to be quite some work. At the moment I plan to trick the inet & file implementations by fake-loading the internal NIFs, then re-implementing those via a compatible Rust / BIF interface.

# Feature status

This section is a quick overview of what's supported, and what's the next general features that will be worked on.

You can view a detailed breakdown on [opcode](/notes/opcodes.org) or [BIF](/notes/bifs.org) progress.

Plan:

- [ ] implement all the instructions
- [x] implement enough BIFs to get preloaded
bootstrap to load
- [ ] implement enough to get the full system to boot (`init:start`)
- [ ] get OTP tests to run

Features:

- [x] Floating point math ([float registers](https://pdfs.semanticscholar.org/7347/354eaaad96d40e12ea4373178b784fc39bfc.pdf))
- [x] Spawn & message sending
- [x] Lambdas / anonymous functions
- [x] Stack traces
- [x] Exceptions
- [x] Process Dictionary
- [ ] Monitors ([rbtree](https://crates.io/crates/intrusive-collections))
- [ ] Signal queue
- [ ] error_handler system hooks (export stubs)
- [ ] Deep term comparison (lists, tuples, maps)
- [ ] Timers
- [ ] Maps
  - [x] Basic type implementation
  - [x] BIF functions
  - [x] Map specific opcodes
- [ ] Binaries
  - [x] Basic type implementation
  - [ ] Binary building
  - [ ] Binary matching
  - [ ] Binary searching
    - multi pattern via [aho-corasick](https://github.com/BurntSushi/aho-corasick)
    - single pattern via [boyer-moore](https://github.com/killerswan/boyer-moore-search) | [needle booyer-moore](https://docs.rs/needle/0.1.1/needle/) | [regex - booyer-moore](https://github.com/ethanpailes/regex/commit/d2e28f959ac384db62f7cbeba1576cf39a75b294)
  - [ ] Bitstring (bit-level) matching
- [ ] File IO
- [ ] [NIF](http://erlang.org/doc/man/erl_nif.html)
- [ ] Ports
    - [ ] inet_drv
    - [ ] ram_file_drv
- [ ] External Term representation
  - [x] Most of decoding
  - [ ] Encoding
- [ ] ETS
- [ ] GC!
- [ ] Code reloading
- [ ] Tracing/debugging support
- [ ] beam_makeops compatible load-time opcode transformer
- [ ] Optimize select_val with a jump table
- [ ] Directly embed imports as some form of a pointer reference

# Contributing

Contributors are very welcome!

The easiest way to get started is to look at the `notes` folder and pick a BIF or an opcode to implement. Take a look at `src/bif.rs` and the `bif` folder on how other BIFs are implemented.

Test coverage is currently lacking, and there's varying levels of documentation; I will be addressing these as soon as I solidify the core data structures.

I'm also relying on a few external rust crates for various parts of implementation, I'd like to bring the dependency count down once I'm able to run the OTP emulator tests.

# Acknowledgements

- [Yorick Peterse's Inko](https://gitlab.com/inko-lang/inko/), from which I've stolen the process scheduling code and a lot of the VM design.
- @kvakvs for [ErlangRT](https://github.com/kvakvs/ErlangRT) which I've used for an extensive reference, along with his [BEAM Wisdoms](http://beam-wisdoms.clau.se/en/latest/) website.
- [The BEAM Book](https://github.com/happi/theBeamBook), which spurred this interest in the first place.
- [bumpalo](https://github.com/fitzgen/bumpalo) for the basis of bump allocation.
