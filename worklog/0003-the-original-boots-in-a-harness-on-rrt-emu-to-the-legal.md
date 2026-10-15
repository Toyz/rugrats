---
number: 3
title: The original boots in a harness on rrt-emu, to the legal screen
date: 2026-10-14
area: tooling, engine, decomp, video, test
files: crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/src/hw.rs, crates/rugrats-hle/src/bus.rs, crates/rugrats-hle/src/mdec.rs, crates/rugrats-data/src/mdec.rs, symbols/slus_006.txt
started: 2026-10-14T18:57:43-07:00
took: 17m
---

# 3. The original boots in a harness on rrt-emu, to the legal screen

`rugrats-hle` runs the unmodified `SLUS_006.50` in retro_rt's R3000A
(`rrt::emu`) for 600 frames in 4.8 seconds and reaches the opening
cinematic. On the way it draws the legal screen, decoded from
`LEGAL512.BS` through a model of the MDEC, and loads 32 files.

## How it is put together

There is no emulator to compare against on this machine, so the reference is
built from parts that already exist: rrt-emu's CPU, GTE and BIOS-table HLE,
and hwtr's GPU rasterizer and I/O model, carried onto a new `PsxBus` (2 MB
RAM mirrored to 8 MB, scratchpad, I/O, BIOS area reading zero). rrt-emu has
no interrupts and no exceptions; the harness supplies both at the level the
game sees them. libetc's `ResetCallback` (0x8009b2fc) hands the BIOS a
setjmp buffer whose resume point calls the dispatcher at 0x8009b3d4, which
reads I_STAT and I_MASK and calls one handler per bit. So an interrupt is:
set the I_STAT bit, and when I_MASK lets it through, interrupts are enabled
(the critical-section flag rrt's BIOS keeps) and the dispatcher is
installed, call the dispatcher on top of the running program with every
register saved and restored. The vertical blank is raised once per frame; a
DMA's completion raises IRQ 3 through DICR 2000 steps after it started, which
is what libgpu's drawing queue waits for.

Only four things are replaced at their entry points: `VSync` (0x8009af48)
ends the frame and raises the vertical blank instead of spinning on the root
counter; the game's own `read_file` (0x80019794) copies the file from the
disc, since the original reads through libcd's ready callback; `printf`
(0x800b1478) is logged; and libcd's drive calls (CdReset, CdSync, CdReady,
CdControl, CdControlF, CdControlB) answer "done" because no drive is
modelled yet. BIOS calls the game makes beyond rrt's table are added in
the harness: A0:71 `_96_init`, A0:AB/AC/AD and B0:5C/5D for the memory card.

## Three faults on the way

The nested dispatcher call first ran 5 million steps in libgpu's GPU
time-out loop (0x8009e8e4): a VSync callback calls `DrawSync(0)`, which
waits for the DMA to finish, and the device clock only advanced between run
slices. Nested calls now run in slices too and advance the devices, but
never nest an interrupt. Before that, no interrupt was delivered at all: the
flag rrt's `critical_sections` returns is "interrupts enabled", not "inside a
critical section". And the hwtr jump-table fix of [[2]] was a precondition
for reading any of this.

## The MDEC

The logos and the legal screen are `.BS` stills: libpress decodes the
variable-length codes on the CPU and feeds run-length blocks to the MDEC by
DMA 0, reading pixels back by DMA 1. `rugrats_data::mdec` is the decoder
proper (dequantise, zigzag, the two-pass IDCT through the uploaded scale
table, YUV to RGB at the MDEC's coefficients) as a pure function, tested on
hand-made blocks (`a_dc_only_block_reads_back_at_its_zigzag_start`,
`a_run_skips_coefficients_and_lands_in_zigzag_order`,
`negative_levels_divide_toward_zero`, `a_flat_block_through_a_flat_matrix_stays_flat`).
The port will use the same code for its stills and movies. The harness wraps
it in the command protocol (decode, quant tables, scale table) and lets a
DMA 1 started before the data wait for it.

The libraries are mapped in `docs/engine/libraries.md`, the boot in
`docs/engine/boot.md`.

**Still unknown:**
- how the opening cinematic streams: it reads sectors by DMA 3 inside libcd's ready callback, which needs a model of the drive
- what the words game_init patches at 0x8009f2f8, 0x8009f870 and 0x8009fc70 are
- whether the MDEC model's colour conversion and IDCT match the hardware bit for bit
