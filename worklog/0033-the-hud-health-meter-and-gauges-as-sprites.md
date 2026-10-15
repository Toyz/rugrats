---
number: 33
title: The HUD: health meter and gauges as sprites
date: 2026-10-15
area: render
files: crates/rugrats/src/hud.rs, crates/rugrats/src/game.rs, crates/rugrats/src/level2.rs, crates/rugrats/examples/play_script.rs
started: 2026-10-15T02:52:13-06:00
took: 10m
---

# 33. The HUD: health meter and gauges as sprites

The level's HUD sprites now draw where the port showed numbers.

**Sprites** (`rugrats::hud`, 0x8008f8f4 drawn by 0x8008f5d8):

- A texture from the database's TEXPOS.DAT: 428 12-byte entries, read
  through 0x80089fa4 (n mod 428).
- The file is 0x800b9b20[db + 1]: 0x8008a328 is called with the level
  plus one, and entry 0 is the menu's. The first try used [db] and got
  the previous database's file. Its meter happened to match; sprite 278
  was empty.
- A quad of half-size (w, h) at depth z. Its x is scaled for the 512-wide
  screen (512 / 320) and it is projected with the game's focal length.

**Health meter** (0x8008ba70, drawn by 0x8008bdf4):

- Fill 260 (32 x 60) under the frame 258 (32 x 24), depth 768, at
  (192, 76) from the centre.
- The fill's top comes down as health falls.
- It shows for 1500 dt after a change, on levels 2-6, 9, 12, 14 and 16.

**Gauges** (0x8008c5e0, drawn by 0x8008c394):

- Back 282, then the fill, half-size 64 x 64 at depth 1024.
- Level 2's court: the goose's (278) at (-200, 88) and Chuckie's (279)
  at (200, 88).

Checked in pictures: the bottle meter at reduced health, and the
goose's and Chuckie's portraits on the court in phase 3.

**Still unknown:**

- Where 0x8008fa9c puts each piece of a multi-part sprite. The meter's
  cap sits on the fill's full top here.
- How a gauge's fill shows damage; at full health it cannot be seen.
