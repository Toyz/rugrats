---
number: 2
title: The image puts its rodata between hand-written asm and the compiled code
date: 2026-10-14
area: decomp, tooling
files: crates/rugrats-psx/src/program.rs, crates/rugrats-re/src/main.rs, symbols/slus_006.txt
resolves: 1.4
started: 2026-10-14T18:55:15-07:00
took: 2m
---

# 2. The image puts its rodata between hand-written asm and the compiled code

`SLUS_006.50`'s text image is laid out in three parts, not hwtr's code then
data:

| range | what |
| --- | --- |
| 0x80010000 - 0x80012760 | hand-written assembly: `addi sp` frames, GTE moves, private `jal` helpers that pass values in s1-s3 and t1-t3 (a polygon emitter: it writes GPU packet words with `lui t8, 0x3400` codes into a buffer at v0) |
| 0x80012760 - 0x80017c0c | read-only data: format strings from "Before ResetHierarchy" on, the switch jump tables, ending in "<NULL>" |
| 0x80017c0c - 0x800b2794 | compiled C, then data to 0x800c5000 |

The jump-table matcher carried over from hwtr stopped at the first target
not below the table (`t >= table`), true when tables follow the code and
never true here, so all 80 of the game's switch tables were missed and their
case blocks became thousands of bogus gap functions. The test is now that a
target lies within 64 KB of its `jr`, which holds for both layouts.

The rodata itself had been seeded as code by the gap and scan heuristics. The
analysis now takes known data ranges (`Program::analyze_with_data`);
`rugrats-re` reads them from `rodata_start` and `rodata_end` in
`symbols/slus_006.txt`. The result: 2161 functions, 80 jump tables, and 2145
unowned code words, down from 15800. The largest remaining gaps
(0x800a624c 2140 bytes, 0x800a6c54 1108, 0x80097bd8 872) are in the library
range and reached only through pointers, still to be looked at.

**Still unknown:**
- what reaches the unowned stretches at 0x800a624c, 0x800a6c54 and 0x80097bd8
- where the hand-written asm's entry points are and what calls them
