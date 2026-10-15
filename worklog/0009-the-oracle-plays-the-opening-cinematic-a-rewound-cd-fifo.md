---
number: 9
title: The oracle plays the opening cinematic: a rewound CD FIFO and a lost load delay
date: 2026-10-14
area: bug, tooling, engine, render
files: crates/rugrats-hle/src/cdrom.rs, crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/src/bus.rs, crates/rugrats-hle/src/main.rs, crates/rugrats-hle/src/hw.rs
resolves: 7.1
started: 2026-10-14T19:56:57-07:00
took: 9m
---

# 9. The oracle plays the opening cinematic: a rewound CD FIFO and a lost load delay

The opening cinematic now plays in the oracle: database 22's living room
with Grandpa, Phil and Lil, Chuckie, Spike and Angelica, 954535 primitives
over 3000 frames. Two faults in the oracle stood in the way, and the second
could have struck any interrupt.

## The data FIFO rewound

The cinematic's ready callback (0x80019c88) reads each data sector in two
pieces, `CdGetSector(buf, 4)` then `CdGetSector(buf + 16, 511)`, and each
call asks for the data again (request register bit 7). The drive model
reloaded the FIFO from the sector's start on every request, so the buffer
held the 12-byte header and subheader twice and every frame record sat 16
bytes off: the stream's first frame read as frame 4's header bytes, the
camera never reached a sensible place, and `render` reported "Camera
outside room" and 0 polygons each frame ([[7]]). The buffer at 0x800c7510
showed it, `02420200 80080001 80080001 05640004` repeated where the disc has
it once. A sector now loads into the FIFO once, when it is new; asking again
continues where the reading left off.

## A load in flight landed in the interrupt handler

With the stream right, the scene drew for 60 frames and the program jumped
to 0x00000000 from libgpu's DrawSync wait (0x8009e86c, `jr ra`), though the
stack slot it loads ra from held a sensible word and a watchpoint on it
(`RUGRATS_WATCH`) never fired. The R3000 lands a load one instruction late;
an interrupt taken in that window started the dispatcher with the load
still pending, and it landed on the handler's first instruction,
overwriting the registers the harness had just set, its return address
among them. The handler then returned into the interrupted program's code
and ran it on the interrupt stack. The handler now starts with the load
settled in its own copy of the CPU; the saved copy keeps it pending for
the interrupted program.

## Tools added on the way

`--trace-from F` records executed addresses from frame F and prints the
last of them, with sp, ra, gp, fp, s0 and s1, when the run stops;
`RUGRATS_WATCH=ADDR` stops the run when 0 is written over a code address
there; the hardware model traces each DMA start at `rugrats_hle::hw=trace`.

**Still unknown:**
- whether other oracle faults remain on paths not yet run (the front end, play)
