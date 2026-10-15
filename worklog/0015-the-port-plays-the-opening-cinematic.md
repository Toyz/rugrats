---
number: 15
title: The port plays the opening cinematic
date: 2026-10-14
area: engine, render, play, decomp, test, input
files: crates/rugrats/src/cine.rs, crates/rugrats/src/main.rs, crates/rugrats/src/input.rs, crates/rugrats/src/movie.rs, crates/rugrats/src/disc.rs, crates/rugrats/examples/cine_objects.rs, crates/rugrats/examples/find_texture.rs, crates/rugrats-data/src/model.rs, crates/rugrats-render/src/models.rs, crates/rugrats-render/tests/against_original.rs, crates/rugrats-viewer/src/main.rs, crates/rugrats-hle/examples/cine_shot.rs, crates/rugrats-hle/src/gpu.rs, crates/rugrats-hle/src/lib.rs
started: 2026-10-14T21:19:13-07:00
took: 1h 10m
---

# 15. The port plays the opening cinematic

After the logos the port now plays the opening (cine 0 on database 22):
`rugrats::cine` reads OPEN.BIN whole the way the movies are read, keeps
the stream's schedule (a data sector every 5 sectors at double speed, so
30 frames a second), plays the XA that the filter picks (file 1, channel
1), draws the rooms into a 4:3 target with the renderer, and moves the
objects each frame. The viewer's `--cine` now goes through the same code,
and its picture is byte for byte what it was.

Three frames (199, 600, 1000) were set against the oracle's pictures.
Frame 1000 lacked the television, which led to four corrections:

- A model id the search misses comes back as offset 0, the first model
  stored, and the class-0 constructor (0x800494d8) then clears the
  object's draw. `Models::get` now follows 0x80044b74 exactly, including
  its upper bound at the count, its final look at two candidates and the
  word before the table. A test calls the original for every id 0-2047 of
  every MOD file and agrees.
- Placed objects take LP's quaternion inverted (0x8001f684), as angles
  (0x800202b8), back into a matrix (0x80020ee0, or 0x80020d10 for a jointed
  object's base). The object transform transposes object +0x04, so placed
  models draw as quat_matrix(q) transposed, while cinematic actors
  (0x8001f9fc) draw as quat_matrix(q). The port had placed objects the
  other way, which turned the television to face the wall. A test pins
  the three chains for single-axis turns. Two-axis turns do not survive
  the angles' round trip unchanged; DB22's LP has none.
- The loader passes over six templates, and the constructor is picked
  by template byte +0x04.
- Jointed objects the stream does not pose are drawn at rest.

The television was found by tracing the oracle's polygons in that part of
the screen (cine_shot's CINE_RECT, now confined to the frame asked for),
which gave its texture page and CLUT, and `find_texture` named model 420.

play_cine samples the pads once a frame, after drawing, and stops on the
first port's Triangle going down. The port samples there too. The logo
movies' skip sampled every tick but tested every other, so it could miss
a press; they now sample once a movie frame.

The oracle cannot check the skip: the game drives the pads through
libpad's direct serial driver (0x80099e08), which the oracle does not
model, so its receive buffers never fill.

**Still unknown:** the Euler round trip for turns about more than one axis; whether Triangle skips (the oracle cannot take input: libpad drives the serial port directly); the title card (text system 0x8002e144); wall shading
