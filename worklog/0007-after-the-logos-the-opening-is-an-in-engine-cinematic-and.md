---
number: 7
title: After the logos: the opening is an in-engine cinematic, and the oracle renders it empty
date: 2026-10-14
area: engine, decomp, tooling
files: crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/src/printf.rs, crates/rugrats-hle/src/main.rs, docs/engine/cinematics.md
started: 2026-10-14T19:43:05-07:00
took: 11m
---

# 7. After the logos: the opening is an in-engine cinematic, and the oracle renders it empty

After the n-Space logo, main reaches "Opening Cine Init" (0x8008f078),
which loads database 22 (files 521, 456, 488, 264, 392, 424: textures,
rooms, collision, models, hierarchy, loops) and the menu's eye and mouth
TIMs, and plays an in-engine cinematic through `play_cine` (0x8007424c,
`cinemat.c`). The cinematic is the engine rendering the database's scene,
steered by OPEN.BIN's data sectors, one sector a frame, with the XA
soundtrack (file 1, channel 1) interleaved. The stream's first sector is a
header: 0xf00dbabe, version 3, 0xdead, 1412 frames, 1380 bytes of setup.
`docs/engine/cinematics.md` has what is known.

In the oracle the opening shows only the clear colour, a flat skin tone,
for as long as it was run (2000+ frames). Followed down: the stream starts
(Setloc 00:02:26 = LBA 26, ReadS, Setmode 0xe9, Setfilter 1,1) and the
ready callback 0x80019c88 receives its sectors; the frame counter at
0x800c4c34 advances (0x1c after about 110 frames); `render` (0x80030b80) is
called every frame. The game's own debug print (0x8002f284, now logged by
the oracle at `game=debug`) then says why nothing appears: "Render --
Camera outside room" and "0 polys (0 terrain, 0 models)" every frame. The
camera block at 0x800e5428 has room -1 at +0x80.

Ruled out: the empty sectors (submode 0) interleaved in OPEN.BIN. Keeping
them from the game in the drive model changes nothing.

The oracle gained `--poke ADDR=WORD` (written before every frame) and the
debug print hook. `--count` replaces a function's hook when used on a
hooked address, so it cannot count `printf` or the debug print.

**Still unknown:**
- why the camera is outside every room in the oracle's opening, and whether the fault is the oracle's or a missing input
- the per-frame record's fields and the cinematic table's entries
