# Enigma

Another VM, Why? So I decided to dig into BEAM internals because I wanted to
start contributing to Erlang development, and I learn best by building. That,
plus writing this in Rust has been tons of fun (I also trust my Rust code a lot
more than my C code).

Current aim is to be OTP 22 compatible, (sans the distributed bits for now).
Performance? My TotallyLegit™ fibonacci benchmarks are currently on-par with OTP
(albeit I'm missing 99% of the runtime).

Plan: implement all the instructions -> implement enough BIFs to get preloaded
bootstrap to load -> implement enough to get the full system to boot -> OTP
tests to run.

# Installation

[rustup](https://rustup.rs/) to install latest rust (minimum version is 2018 edition).

`cargo install` to install the dependencies, `cargo run` to build. Expect heavy
crashes, but a basic spawn + send multi-process model already works.

# Goals

- Explore using immix as a GC for Erlang
- Be able to run the Erlang bootstrap
- Be able to run Elixir
- Ideally one day, feature parity with OTP
- Write more documentation about more sparsely documented BEAM aspects (binary
    matching, time wheel, process monitors, etc).

# (Initial?) non-goals

- Distributed Erlang nodes
- NIFs

# Ideas/Experiments

- Process as a generator function (yield to suspend/on reduce)

# TODO

- [ ] full external term representation
- [ ] deep term comparison
- [ ] GC!
- [ ] exports global table? that way we can call any method
- [ ] directly embed imports as some form of a pointer reference

- focus on getting preloaded modules to load: {:preLoaded,
    [:atomics, :counters, :erl_init, :erl_prim_loader, :erl_tracer, :erlang,
     :erts_code_purger, :erts_dirty_process_signal_handler, :erts_internal,
     :erts_literal_area_collector, :init, :persistent_term, :prim_buffer,
     :prim_eval, :prim_file, :prim_inet, :prim_zip, :zlib]},

# Special thanks

- [Yorick Peterse's Inko](https://gitlab.com/inko-lang/inko/), from which I've stolen the process scheduling code.
- @kvaks for [ErlangRT](https://github.com/kvakvs/ErlangRT) which I've used for an extensive reference, along with his [BEAM
    Wisdoms](http://beam-wisdoms.clau.se/en/latest/) website.
- [The BEAM Book](https://github.com/happi/theBeamBook)
- bumpalo for the basis of bump allocation
