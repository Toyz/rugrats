---
number: 201
title: Level 16's pies are six objects thrown and flown as the original's, the held player kept to the camera and on his spot; a pie matches the original every frame
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/boxhit.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T14:42:37-06:00
took: 1h 25m
---

# 201. Level 16's pies are six objects thrown and flown as the original's, the held player kept to the camera and on his spot; a pie matches the original every frame

Level 16's pie toss is the original's: the six pies are objects thrown by the player's own throw, and each flies the original's flight. A pie thrown matches the original on every frame of its flight, with the player's animation tracks the original's throughout.

**The pies (class 9, 0x8004997c).** A placement of template class 9 is six objects (0x800e2770), at its place.
- The toss's start (0x8007e680) puts all six on the pie spot: (0, 0, 512) turned by the player, plus his place (0x80049818). Pie 0 goes in his hand by +0x1f0 alone, with no grab: the grip stays as it was and the arms play on (`Game::hold`).
- A pie is thrown by the player's throw (Triangle, animation 18, the release at +0xe0 bit 16).
- The release builds 0x80061c98's flight. The way is (0, 35, 1024) turned by him and made a unit. The speed is 225 with the target within 13000, else 450: level 16's Angelica measured across (0x8001f57c), level 3's Larry in 3D (0x80075f58).
- The flags are 0x4a070400: gravity 1 (`Flown::gravity`); done at a hit (0x2000000, `Flown::ends_on_hit`); what it meets met by a segment of its radius (0x40000000, `Flown::grows`).
- The thrown pie's first think (0x8004eb10) puts the next pie (0x800c4134 + 1, mod 6) on the pie spot and in his hand, the same frame.
- A pie done at a hit leaves its room. Its template's +0x0b of 1 throws five splats (0x8005282c), and the record's mark sounds 46 at the player (0x8008652c).
- The port's own pies for level 16 (`levels::pie_toss`, a quarter-step roll with guessed radii) are gone from it.

**A segment with a radius.** 0x80087594 hands 0x80086668 the record's radius when its flags have 0x40000000 (0x800877dc), and 0x8006a318, 0x8006a008 and 0x80069c40 grow each box by it on every side before the slab test (0x8006a4bc). The bound's cube is not grown. `Solid::meet_grown`.

**The player held in place (+0x188 bit 1).**
- The stick keeps the heading wanted within 512 of the camera's (0x8005c3dc, both as 0..4096), even with the stick let go. This is not done in levels 20 and 21, where 0x8005be24 takes the bit as 0. The toss's Tommy, put facing 2048 by a camera along 1024, turns 1888, 1728, 1568, 1536, as the original does.
- Standing, he takes no step and no floor follows: the original's Tommy stays as put, 31 into the counter.

**Message 27 (and 6)** shows only on the frame the carry's animation is done (+0xe0 bit 0x8000000, 0x80060b44 at 0x80060ef8). The port showed it at once, and its hold of the world stopped his turn.

**Oracle.** `level_16_pie_flies_as_the_original`: Tommy put in room 4, both messages closed, a pie thrown, the throw's message closed. The pie is the original's on every frame of its 90, and its flight ends the frame the original's does (+0x6b 255), with the pie leaving its room. The hand's pie is the original's every frame; the animation tracks are the original's throughout. The play test of the toss (`level_16_toss_is_won`) still wins it. `npc_check`'s `MOVES` shows a character's root height.

**Still unknown:**
- Level 3's toss keeps the port's own pies. Its pies fly the same flight, and the original's met Steve 2000 short of him, from the bounds cube's edge with the radius grown by 405 (0x80086340, seen). The port's brothers wait with their root curve's height 1024 to 1202 under their place (the wait, table 31, starts at a lift key of 525, scaled 2.3 for an adult). The original keeps the same field (+0xde 1325) but its place is 119 over the port's. How a body's root height enters its posed parts and its place for adults is not settled, so the port's pies pass over them.
- Toward that, level 3's toss sets template flag 8 on templates 489, 566 and 488 (0x8007bc60) and clears it after (0x8007e3c0); `Game::set_template_solid` does that, and level 3 does not use it yet.
- The mark a pie leaves (sprite 284, 900 square, 0x8008e72c / 0x8008e76c) is not drawn.
- What the toss's end does with the pies left (the port puts them away).
