---
number: 5
title: The port begins: rugrats on retro_rt shows the legal screen, pixel for pixel
date: 2026-10-14
area: build, video, format, test, design
files: crates/rugrats/src/main.rs, crates/rugrats/src/disc.rs, crates/rugrats/src/still.rs, crates/rugrats-data/src/bs.rs, crates/rugrats/tests/against_original.rs, crates/rugrats-hle/src/cdrom.rs
resolves: 4.2
started: 2026-10-14T19:20:07-07:00
took: 12m
---

# 5. The port begins: rugrats on retro_rt shows the legal screen, pixel for pixel

`rugrats` is the port's program: retro_rt's loop at the NTSC rate, the
player's disc read through the game's own file table, and the boot as far as
it is ported. It shows the legal screen for 250 frames, as `show_still`
(0x80030010) holds it, and then stops where the port ends: the opening
cinematic. Nothing from the PlayStation runs in it.

The still is decoded natively. `rugrats_data::bs` turns `LEGAL512.BS` into
the run-length stream libpress hands the MDEC, and the test
`legal512_decodes_to_what_libpress_hands_the_mdec` holds it to the stream
dumped from the original running in the oracle (`RUGRATS_MDEC_DUMP`): all
42816 halfwords equal. The first attempt ran out of bits at bit 263765 of
263776; libpress leaves a tail shorter than a block undecoded and pads with
0xfe00, and so does the port now. The MDEC's quant and scale tables are not
written into the source: libpress keeps them in the executable as ready
commands (0x40000001 at 0x800c1f20, 0x60000000 at 0x800c1fa4), and
`GameDisc::open` reads them from the player's `SLUS_006.50`, refusing an
executable whose command words differ.

`the_legal_screen_matches_the_original` runs the original in the oracle to
`show_still`'s hold loop (VSync returning to 0x80030120), waits 30 frames for
libgpu's queue to draw, and compares the displayed 512 x 240 with the port's
picture: 0 of 122880 pixels differ. That fixes the draw's one subtlety, the
quads' colour 0x7f7f7f (0x800c3f9c), which scales each 5-bit channel by
127/128.

## Two oracle faults the test found

The test first met a screen of textures, because CD initialisation took 220
budget frames: libcd's `CD_cw` waits for an acknowledge, and the drive model
raised INT3 with its interrupt-enable register at 0. The BIOS leaves it at
0x1f for the game; the model now starts there, and raises IRQ 2 on the
rising edge of flag-and-enable, as the line behaves, so a response flagged
before interrupts are enabled is not lost. Initialisation now takes about
140000 steps, and that also answers [[4]]'s time-out on the first CdlNop.
Then the first hold frame was still black: the original's draw is queued in
libgpu and lands a frame or more later.

## Not an HLE

The oracle exists to check ports like this one; the program is the port.
`rugrats` is a library (`disc`, `still`) and a thin program, so tests can call
the ported code directly.

**Still unknown:**
- the opening cinematic's stream format (OPEN.BIN: data sectors starting with 0xf00dbabe and a frame counter, interleaved with XA audio)
- BS version 3 (E3SCR512.BS)
