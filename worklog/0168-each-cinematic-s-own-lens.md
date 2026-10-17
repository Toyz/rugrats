---
number: 168
title: Each cinematic's own lens
date: 2026-10-16
area: render
files: crates/rugrats/src/media/cine.rs, crates/rugrats/tests/play.rs, crates/rugrats/examples/cine_objects.rs, crates/rugrats/examples/play_script.rs
started: 2026-10-16T16:02:22-06:00
took: 2h 17m
---

# 168. Each cinematic's own lens

Each cinematic now has its own lens. The port drew every cinematic with
the projection distance it measured on the opening (178 on the 512 x 240
screen), and the new game's cinematic (7, Tommy and the blocks before the
TV) came out about twice as wide as the original's: the oracle (level 1
booted with its cinematics) opens on the car and the blocks filling the
screen, the port showed the room, with Chuckie by the plant in his
hierarchy's rest pose (arms out) - in the original he is off screen there
and the stream does not pose him until the camera pulls back.

0x8007424c sets the cinematic camera's lens as it starts (0x8001dfb8 on
0x800e5428): the first half-word of the cinematic's row in the table at
0x800b8bd4 (16 bytes a row; byte 3 is the music flags the port already
read) is a field of view, and the projection is 160 over the tangent of
half of it (0xa0000 / tan in 4.12), handed to SetGeomScreen
(0x8009fd78). The opening's is 955 (178, as measured); most others' 512
(386). `cine::focal` is that sum; `Cine` keeps it for its camera. The
port's cine 7 now matches the oracle's pictures shot for shot.

**Still unknown:**
- Whether any cinematic changes its lens while it plays (the table is
  read once, at its start).
