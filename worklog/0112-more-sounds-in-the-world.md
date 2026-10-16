---
number: 112
title: More sounds in the world
date: 2026-10-15
area: audio
files: crates/rugrats/src/levels/aisle_seven.rs crates/rugrats/src/levels/box_toss.rs crates/rugrats/src/levels/golf.rs crates/rugrats/src/levels/let_there_be_light.rs crates/rugrats/src/levels/mirrorland.rs crates/rugrats/src/levels/mr_friend.rs crates/rugrats/src/levels/outer_space.rs crates/rugrats/src/levels/toy_palace.rs crates/rugrats/src/levels/training.rs crates/rugrats/src/world/game/mod.rs
started: 2026-10-15T23:21:28-06:00
took: 3m
---

# 112. More sounds in the world

69.1 asked which other sounds the original plays positionally. All 67
call sites of 0x8006f644 (a sound at a position) were listed with their
sound numbers and set against the port's centred `play` calls. These
are positional in the original and now go through `Game::play_at`:

| sound | original | at |
|---|---|---|
| 44 | 0x800511a8 (a hit) | the one hit: level 3's Larry or Steve and lobsters, training's Angelica and lobster |
| 63 | 0x800511a8 | the stunned robot (level 5) |
| 110 | 0x800511a8 | the character the light catches (level 12) |
| 68 | 0x80059d3c | the player, the zap (level 5) |
| 70 | 0x80090fac | the beam where it bounces (level 5) |
| 71 | 0x8004d078 / 0x8004d0b0 | the sliding door (level 5) |
| 72 | 0x8004dba0 | the scary toy (level 6) |
| 79 | 0x80053ef8 | the burst (level 6) |
| 173 | 0x8005a068 | the player (level 6's breakables) |
| 114 | 0x8004d434 | the broken object (level 15) |
| 98 | 0x800833a4 | Mr. Friend, kicked |
| 117 | 0x8007f5a8 | the player, the timer's end |
| 121 | 0x8005b2d8 (state 28) | Angelica lunging (levels 20-21) |
| 137 | 0x80086ef0 | the golf ball |

Level 5's beam bounce also lost its guess. The wall's normal now comes
from the segment test (0x8002cd7c, 0x80090554's world part), and the
reflection is 0x8007666c's (r = u - (c N >> 11)).

**Still unknown:**
- Reptar's building sounds (90-96, 0x8004d7c4) and the rumble landing (93, 0x8005cba8) are positional too; reptar_2010.rs is being reworked with the effects, so they wait for that
- the call sites with a variable sound (0x80053344, 0x80053604, 0x8004f868, 0x80050140 and others) were not each traced to a port call
