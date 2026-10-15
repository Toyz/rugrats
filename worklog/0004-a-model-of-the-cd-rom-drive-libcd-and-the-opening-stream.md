---
number: 4
title: A model of the CD-ROM drive: libcd and the opening stream run natively in the oracle
date: 2026-10-14
area: tooling, engine, video, audio
files: crates/rugrats-hle/src/cdrom.rs, crates/rugrats-hle/src/hw.rs, crates/rugrats-hle/src/lib.rs
resolves: 3.1
started: 2026-10-14T19:15:50-07:00
took: 4m
---

# 4. A model of the CD-ROM drive: libcd and the opening stream run natively in the oracle

The oracle's libcd hooks of [[3]] are gone: `rugrats-hle` now has a model of
the drive behind 0x1f801800-0x1f801803 (`cdrom.rs`), and libcd runs as it
does on the console. Commands take parameters from the FIFO and answer with
INT3, then INT2 for the slow ones (Init, Pause, Stop, SeekL/P, GetID,
MotorOn, ReadTOC), each response waiting until the previous is
acknowledged. ReadN and ReadS deliver INT1 per sector at the mode's speed;
the request register's bit 7 loads the data FIFO with 2048 bytes (mode bit 5
clear) or 0x924 from the header (set), drained by the CPU or DMA 3. With
mode bit 6 set, real-time audio sectors that pass the filter go to the sound
side and never raise INT1. The game's `read_file` shortcut stays as the
default (`--real-cd` turns it off).

The opening stream now runs: 1500 frames, ReadS three times, 318 XA audio
sectors to the sound side. The game sends Getstat 1, Setloc 5, MotorOn 2,
Init 1, Demute 4, Setmode 6, SeekL 5, ReadS 3 in that stretch.

Two faults fixed on the way. An interrupt that arrived while the hand-written
renderer at 0x80010000 was running crashed the handler on `lw v0, 92(gp)`
with gp = 1: the asm uses gp as a scratch register. On the console the BIOS
longjmps into the setjmp buffer `ResetCallback` gave HookEntryInt
(0x800bbad0: ra, sp, fp, s0-s7, gp as libc's setjmp at 0x800b2758 stores
them), so the dispatcher runs on libetc's own stack (sp 0x800bcab0) with the
C gp. The harness now enters the dispatcher with those registers. And libetc
writes DICR by halfword (0x1f8010f6); the DMA and interrupt registers now
take narrower accesses as parts of the word.

**Still unknown:**
- how the opening stream's video sectors are decoded (the CINEPAK file names suggest Cinepak, not MDEC)
- why the first CdlNop after CdInit times out ("CD_cw:(CdlNop) Sync=NoIntr")
