---
number: 40
title: Level 9 playable: Mr. Friend
date: 2026-10-15
area: play
files: crates/rugrats/src/level9.rs, crates/rugrats/src/game.rs, crates/rugrats/src/npc.rs, docs/engine/level-09.md
started: 2026-10-15T07:08:05-06:00
took: 25m
---

# 40. Level 9 playable: Mr. Friend

A tenth delegated map, docs/engine/level-09.md, traced The Mysterious
Mr. Friend and checked it in the oracle (deaths forced by poking
health). The port now plays the level.

**Framework:**

- Aimed throws (level 9's case of 0x80061c98, the target from
  0x80059b34). The nearest visible character within 512 of the facing is
  aimed at. The direction is toward it with y replaced by -300 after
  normalising, speed 170.
- Follow bites use the character's bite wait (Mr. Friend 36 VSyncs, as
  seen).

**Level 9** (`rugrats::level9`):

- Tommy and Mr. Friend (46) where cine 17 leaves them. 91, 800 and the
  cine props are hidden. He follows and bites for 15.
- Hits: a thrown thing's +0x6d (ball 50, can 30), or the kick: Triangle
  touching him (within 1100: he stops at range / 2 to bite), Tommy's 50,
  animation 37 or 50.
- A hit staggers him (mode 1) with his callbacks off: no hits or kicks
  until 300 + (rand & 3) x 300 dt after the hit animation.
- At 0 health he dies (sound 100, hidden) and the counter 0x800c4cd8
  goes down from 4:
  - 3: cine 18; after card 710 all three come home at full health and
    chase, phase 2, song 0;
  - 1: phase 3 with one try;
  - 0: cine 19, won.

Checked in scripted runs: a kick for 50, a second kick ignored while he
staggers, the kill after recovery asking for cine 18, then all three
chasing Tommy in phase 2.

**Still unknown:**

- A can bouncing back into Tommy (the 0x80087594 path).
- 46 waiting in state 19 after his return (the oracle's quirk is not
  ported).
- Tommy's animation overrides (logical 49 and 6); the camera's return to
  the room kind after a cry.
