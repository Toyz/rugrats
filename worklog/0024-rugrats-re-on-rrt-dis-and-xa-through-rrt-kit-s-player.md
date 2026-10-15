---
number: 24
title: rugrats-re on rrt-dis, and XA through rrt-kit's Player
date: 2026-10-15
area: tooling, audio
files: crates/rugrats-re/src/main.rs, crates/rugrats-re/src/bios.rs, crates/rugrats-re/Cargo.toml, Cargo.toml, Cargo.lock, README.md, crates/rugrats/src/movie.rs, crates/rugrats/src/cine.rs, crates/rugrats/examples/cine_ids.rs
started: 2026-10-15T00:25:55-06:00
took: 23m
---

# 24. rugrats-re on rrt-dis, and XA through rrt-kit's Player

retro_rt now has rrt-dis (disassembly, analysis, a lifting IR and the
decompiler that started here) and rrt-kit's `xa::Player`, which is the
drive drop rule from worklog 22 moved upstream. The port now uses both.

rugrats-re on rrt-dis (retro_rt 719d0fa, then 070248b):

- `Program::load` is rrt-dis throughout: `PsxExe::parse`,
  `Symbols::parse`, `find_gp`, `Program::analyze`, `Listing`, `Pseudo`.
- The commands and their arguments are unchanged. `disasm --func`,
  `pseudo`, `xrefs`, `calls` and `callers` also take a name or an address
  inside a function.
- BIOS stub naming stays in this repo (`bios.rs`), since rrt-dis leaves
  it to each project. `crates/rugrats-psx` is deleted.
- Against the old tool's baselines:
  - It finds 2237 functions against 2161. The new ones include
    `VSyncCallbacks` and `DMACallback`.
  - All 80 old jump tables match, plus one new one.
  - Unowned words drop from 2145 to 191.
  - 1936 of 2161 functions decompile identically once the presentation
    changes from worklog 12 are normalised. The rest differ in
    folding, BIOS calls, frame sizes of tiny stubs (now correct), and
    conditions that test the register just written.
- Three rrt-dis problems were noted for retro_rt:
  - `value_before` walks back across a `jal`, which leaves junk store
    slots (0x53, 0x55) in `Program::slots`.
  - Arity inside a call cycle depends on decompile order
    (fn_8002e67c).
  - `D_` names both an address and the memory at it.

XA: `movie::XaPlayer` is gone, and the movies and the cinematic stream
use `xa::Player::new(xa::DOUBLE_SPEED)`. The opening's sound is still
47.1 s (`cine_ids` now prints it), and the tests pass.

**Still unknown:**

- nothing
