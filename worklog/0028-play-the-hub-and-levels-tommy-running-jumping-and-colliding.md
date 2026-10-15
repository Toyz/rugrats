---
number: 28
title: Play: the hub and levels, Tommy running, jumping and colliding
date: 2026-10-15
area: play, engine
files: crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats/src/main.rs, crates/rugrats/src/input.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/disc.rs, crates/rugrats-data/src/model.rs, crates/rugrats-data/src/placement.rs, crates/rugrats-data/src/cine.rs, crates/rugrats/examples/level_shot.rs, crates/rugrats/examples/play_script.rs, crates/rugrats/examples/isf_channels.rs, docs/engine/gameplay.md, docs/README.md
started: 2026-10-15T01:27:09-06:00
took: 18m
---

# 28. Play: the hub and levels, Tommy running, jumping and colliding

The user's goal is a playable port. This entry takes the port from the
title into play.

The map: a delegated agent wrote docs/engine/gameplay.md, covering the
level runner, the frame, input, the player object, the states, the
camera, objects, doors, events, pause and HUD.

What is ported:

- **Into a level.** Start Game, then Main Game (or Start), opens game
  mode 1, and the mode is the database number. Level 1 is the hub,
  Pickles' home.
- **Level setup.** `rugrats::game::Game` loads the rooms, models,
  hierarchies, LP, COL and animations.
  - The player starts where the original's placements loader puts it:
    on the LP record whose template is the character's hierarchy
    (Tommy, hierarchy 1). In DB01 that is (5775, 779, 19403), the
    oracle's own value.
  - The player's y is the floor (1024) plus the root's offset (-245),
    which also matches.
- **The player** (`rugrats::player`):
  - The stick turns the camera-relative direction into a heading. Speed
    is magnitude * 17 >> 12 running, or 8 walking with Square, and moves
    170 units a frame, per 0x8005be24.
  - The turn rate is 1025 * dt >> 6.
  - The jump (0x80060f1c) launches at -1000, with along-speed 40 or 12
    times speed. In the air: gravity 14 (5 less while Cross is held going
    up), drag 32/4096, displacement (v dt + a dt^2/2) >> 6. It lands on
    the floor.
  - The floor comes from the ported COL query, with a step-down of 384.
    The player goes back to the frame's start out of bounds, and back
    across into walls.
- **Animation.** The character's state table (0x800b9cb4) and entries
  (0x800ba824) feed four tracks, each slerping from the previous frame's
  keys into the current frame's. Stand, run, jump, fall and land are
  hooked up.
- **Objects.** Each placed model's collision boxes (model +0x0a/+0x0c,
  now parsed) are kept with their placement's rotation.
  - A step that goes deeper into a box is undone.
  - A box whose top is within 384 of the feet is floor.
  - Running at a playpen panel stops there.
- **The camera** follows as 0x80027304 does: it aims 768 above the
  player from 4096 behind and 1024 up, eased 1/8 a frame, with fov 612.
  It matches the oracle's camera position for the hub start (1776,
  -1013, 18486 behind heading 877). L2/R2 turn it.
- **Doors.** LP +0x42 is the level and +0x44 the cine. Triangle near a
  door goes to its level. Select returns to the hub.

Checked:

- The oracle reaches the hub: Start, then Cross in the Start Game menu,
  the Reptar Puzzle cine, two text cards, and play. Its player and
  camera positions match the port's.
- `play_script` (scripted pad input, pictures every N frames) shows
  running, the jump arc (about 560 units up, lands in 15 frames), and
  being stopped by a panel.
- `level_shot` loads levels 2-6, and Tommy stands in each.

Not yet:

- The level-start cinematics and text cards.
- NPC animation: Lil stands at rest in the hub.
- Pickups and throwing, the HUD, pause.
- Level music: levels below 22 skip MidiPlaySong, and DBnn.ISF is eight
  interleaved XA speech channels.
- The original's exact move test: 0x80066014 casts probes at three
  offsets against COL and object boxes; the port's box rule is simpler.

**Still unknown:**

- The hub's starting heading: LP says 2048, but the oracle shows 877
  after the start cinematic.
- Where levels below 22 get their music.
