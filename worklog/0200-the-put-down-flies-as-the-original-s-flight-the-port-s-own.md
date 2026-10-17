---
number: 200
title: The put-down flies as the original's flight, the port's own flight is gone; frame events, the hierarchy box, the movie frame size and the cine record length named
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, crates/rugrats-data/src/anim.rs, crates/rugrats-data/src/hierarchy.rs, crates/rugrats-data/src/model.rs, crates/rugrats-data/src/movie.rs, crates/rugrats-data/src/cine.rs, crates/rugrats-data/src/level.rs
started: 2026-10-17T14:24:41-06:00
took: 18m
---

# 200. The put-down flies as the original's flight, the port's own flight is gone; frame events, the hierarchy box, the movie frame size and the cine record length named

The put-down flies as the original's flight does, and the port's own flight with it is gone. Several file fields the port named unknown are known now.

**The put-down (state 8, 0x80060910, 0x80062ac4).** It is a flight like a throw (0x80061f88, 0x8008864c): (0, 0, 4096) turned by the player and made a unit, at speed 10, and a template that rolls rolls (0x80086ef0). The slide the oracle showed (17, 14, 12, 10, 7, 5, 2 a frame) is the roll's friction, 98 a 64th of dt. The port had kept a float flight of its own for it, with a slide of 16 less a frame and a guessed 450 for how near it struck a character. That flight had nothing else left to fly; it, its radius and its slide are gone. Every flight is now `fly_exact` or `roll_exact`, started by `Game::start_flight`.

**Oracle.** `level_9_ball_put_down_as_the_original`: ball 3 picked up and put down again, on the original's place every frame of its 76 to rest. The port's old put-down was off on all 76. `a_thrown_thing_lands_on_an_object` starts its flight as a throw does now. The flight stops once a step under its radius meets something, so it rests a few units over the top; the test allows that.

**Fields.**
- FRM +0x00 is the frame's events (`Frame::events`), or'd into the body's +0xe0 as the frame ends (0x80055d40).
- HIE header: +0x04 is the body's box, s16 min[3], pad, max[3], pad. 0x80067ba4 hands it to the character probe (0x80064838), which takes its depth from |min z| and |max z| and its height from min y. +0x14 is the channel map. A part's +0x00 is 0 in every file.
- MOD +0x04 is run-time: the loader writes the model's index there.
- STR video sector +0x0c is the frame's size in bytes, to within the last zero halfword; the frame's chunks are padded past it.
- A cinematic frame's u16 at +0x02 is its record's length past those four bytes, in every frame of every cinematic. An actor's +0x16 is 0 in every record.
- The ENDS table's bit 32 is ported (the root's move across, `RootCurve::across`); its comment said not.

**Still unknown:**
- HIE header +0x02 (2 to 7) and a part's +0x20, which is 0 but in DB08, where its high half is depth + 1.
- A room blob's u16s at +0x0a and +0x12: they vary by blob, 0 to 0x8172, and the room renderer (0x80010e14) does not read them.
- The pies (level 3's bakery, level 16's booth) still roll by the port's own model; they are flights of flags 0x0a070400 in the original (gravity 1, ending on a wall).
