---
number: 38
title: Level 5 playable: the remote, the robots, the float
date: 2026-10-15
area: play
files: crates/rugrats/src/level5.rs, crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats/src/npc.rs, crates/rugrats/src/level2.rs, crates/rugrats/examples/play_script.rs, docs/engine/level-05.md
started: 2026-10-15T05:21:47-06:00
took: 1h 6m
---

# 38. Level 5 playable: the remote, the robots, the float

An eighth delegated map, docs/engine/level-05.md, traced Visitors from
Outer Space and checked it in the oracle. The tracer found that poking
the character byte is not enough: main overwrites it during the load,
and the fix is to make 0x80018598 return 3. Skipping cine 37 hung the
level, so every run played it. The port now plays the level.

**Framework:**

- Angelica (char 3) is level 5's player, at the normal turn rate. Only
  chars 6 and 21 turn slowly (0x80062eac).
- Class 13 things stay in the hand: Triangle is the level's, not a
  throw. A level can put a thing in the hand (`give`).
- Single placements can be hidden (the sliding doors). Collectibles
  report what was taken (kind, +0x6d).
- Movers with template flag 0x40000000 move only while ridden (the
  elevator, +0xbe bit 64), and can be reset.
- A guard's bite reach (+0x690) and the time between bites. The height
  window grows with the reach: the robots hover 1600 up.
- State 21 (0x8005db28), floating:
  - the stick at half speed, no gravity;
  - Cross rises 40 a frame, Circle sinks 40;
  - else a bob of 256 about the anchor, 34 a frame.

**Level 5** (`rugrats::level5`):

- Cine 37's end (0x8007b280): the actors and 8801 hidden; Angelica at
  (38395, -5590, 28281) facing 1895 with the remote (template 828) in
  her hand.
- The charge is 120 on gauge 360/361 bottom left. A zap costs 3 (sound
  68); a battery (kind 12) gives 30 (sound 69).
- The beam runs along her facing. A robot it reaches is stunned 6000 dt
  (sound 63); a door it reaches opens.
- TV robots guard 8000 from home and bite for 42 within 1536.
- Doors (template 843) open within 5000 (max + min / 2) and shut beyond,
  with sound 71.
- The elevator (1969) steps 60 a frame while ridden. Room 42 is phase 2.
- Cine 38's end: the float at (92984, -27, 57036) with one try. Room 28
  while floating is a cry; room 38 ends the float, and cine 40 wins.
- ForceCry resets the elevator before phase 2.

Checked in scripted runs:

- The start with the gauge.
- A robot biting 42 then stunned by a zap (Hit 6000).
- The elevator carrying her from -4578 down past 7600.
- The float rising, bobbing, sinking and moving.

**Still unknown:**

- The beam's speed, life and look: the port draws a small marker, 600
  a frame for 40 frames. The wall bounce is not ported.
- Animation 1209 (standing with the remote); the remote drawn in hand.
- The float's cameras (0x800c4a50, posts by room) and the shaft's type
  13 camera.
- The elevator's half-range hold and its arming by landing only.
