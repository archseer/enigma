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
- Use Commentz-Walter for binary matching. ["Commentz-Walter is an algorithm that combines Aho-Corasick with Boyer-Moore. (Only implementation I know of is in GNU grep.)"](https://github.com/rust-lang/regex/issues/197))
    https://en.wikipedia.org/wiki/Commentz-Walter_algorithm

# TODO

x => done, - => partial

- [x] Process Dictionary
- [ ] PutTuple
- [-] lists module
- [-] Date/Time (non monotonic) via https://github.com/chronotope/chrono
- [ ] Equality ops
- [ ] Timers
- [ ] Precalculate Bif0/1/2,GcBif1/2/3 as vals that reference a bif ptr directly (no more imports+bifs hash lookups)
- [x] Cross-module calls (need to store module in ip/cp)
- [ ] try/catch/raise/etc.
- [ ] Binaries (SIMD: https://github.com/AdamNiederer/faster / https://doc.rust-lang.org/1.26.0/std/simd/index.html<Paste>)
- [ ] Binary matching (multi: https://github.com/BurntSushi/aho-corasick
+ single: https://github.com/killerswan/boyer-moore-search / https://docs.rs/needle/0.1.1/needle/ / https://github.com/ethanpailes/regex/commit/d2e28f959ac384db62f7cbeba1576cf39a75b294)
- [x] float registers (https://pdfs.semanticscholar.org/7347/354eaaad96d40e12ea4373178b784fc39bfc.pdf)
- [ ] Ports
    - [ ] inet_drv
    - [ ] ram_file_drv
- [ ] Monitors (rbtree: https://crates.io/crates/intrusive-collections)
- [ ] Maps (small maps: tuples, large maps: https://github.com/michaelwoerister/hamt-rs)
- [ ] File IO base NIF http://erlang.org/doc/man/erl_nif.html
- [ ] Full External Term Representation
- [ ] Optimize select_val with a jump table
- [ ] ETS
- [ ] deep term comparison
- [ ] GC!
- [ ] Tracing/debugging support
- [ ] exports global table? that way we can call any method
- [ ] directly embed imports as some form of a pointer reference

Focus on getting preloaded modules to load: {:preLoaded,
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
