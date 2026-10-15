---
number: 32
title: Level 2 playable: geese, the Spike chase, the shuffleboard, the win
date: 2026-10-15
area: play
files: crates/rugrats/src/npc.rs, crates/rugrats/src/level2.rs, crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats/src/progress.rs, crates/rugrats/src/main.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/lib.rs, crates/rugrats-data/src/placement.rs, crates/rugrats/examples/play_script.rs, docs/engine/level-02.md, docs/README.md
started: 2026-10-15T02:14:23-06:00
took: 38m
---

# 32. Level 2 playable: geese, the Spike chase, the shuffleboard, the win

A third delegated map, docs/engine/level-02.md, traced Grandpa's Teeth
to its end. It checked the phase changes and NPC set-up in the oracle.
The level now plays through in the port.

**Framework:**

- Cinematics inside a level. The game asks (`Game::cine`), main plays
  the cine on the level's database, shows its card, and then resumes
  the same game (`Then::Resume`) with `cine_finished(n)`.
- Room flags from LP kind 6: flag 2 plays the room's cine once (+0x5b),
  flag 32 is a checkpoint.
- `rugrats::npc`: posts from LP kind 2 (neighbours at record +0x14,
  flags at +0x18), the speed table 0x800b9698, steering at most 512 a
  step with a quarter speed on wide turns (0x800820e0, 0x80082264).
  Modes:
  - Idle, Follow (mode 4), Posts and Wait (6, 7: next post 0x80082c78).
  - Guard (11: bites within range).
  - Hit (1), and Ride (0x80083634: the carrier's position less a height,
    its heading plus a turn).
- Characters take health, range, wait, speed index and damage from
  template +0x18..+0x1e and +0x32, and their first post from LP +0x40.
- The player has health (255), a frozen state, and `set_body` (the
  0x80059328 swap, with Spike's turn rate 298).
- Progress records a finished level (0x80076d6c: played, won, the
  main-level count).

**Level 2** (`rugrats::level2`):

- Phase 1 (0x8007914c): Chuckie follows Tommy; geese 900-904 guard
  their spots and bite for 10; ids 335 and 1313 hidden.
- Room 69 plays cine 3, which leads to phase 2 (0x8007c08c):
  - Tommy runs as Spike, with Tommy riding 250 up.
  - The goose (speed index 5, range 4000) carries Chuckie from post 88
    along its route, waiting at the wait posts.
- Room 44 plays cine 4, which leads to phase 3 (0x8007c6e0):
  - Tommy is frozen at (37355, -236, 23544) facing 2048. Spike is at
    (36000, y, 25000).
  - The goose (index 2, range 2000) chases Chuckie, who loops posts from
    114 with 200 health.
  - The 130 s clock is 39000 dt.
- Triangle kicks a puck along the facing at speed 200: 50 damage on
  whoever it reaches. The goose at 0 plays cine 5, then the hub, won.
  Chuckie at 0, the clock at 0, or Tommy's health at 0 cost a try.
- Tries (3, 0x800b2920[2]) put the player back at the phase's respawn;
  a lost phase 3 falls back to phase 2. With no tries left: cine 16 and
  the hub in phase 3, else the hub.
- Health, the clock and the two characters' health show as numbers
  where the original's meters sit.

Checked in scripted runs:

- Phase 1: Chuckie follows.
- Room 69 asks for cine 3, then phase 2. The goose waits at post 88
  (2000 dt, its 400 VSync ticks), then runs posts 88, 87 .. 77 (wait),
  70 (wait), 116, 61 ..., as the doc's route.
- Room 44 asks for cine 4, then phase 3. Kicks hit Chuckie (200 -> 150
  -> 100) and the goose. Five goose hits ask for cine 5.
- Pictures show Tommy riding Spike, and the court with the clock and
  health numbers.

**Still unknown:**

- The original's move test for NPCs (they run the player's state
  machine and collision). The port moves them along the COL floor.
- The meters (HUD sprites, TEXPOS); the puck's touched-object rule; the
  cry animation and its speech.
