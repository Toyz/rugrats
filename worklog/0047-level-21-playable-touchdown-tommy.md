---
number: 47
title: Level 21 playable: Touchdown Tommy
date: 2026-10-15
area: play
files: crates/rugrats/src/level20.rs, crates/rugrats/src/game.rs, docs/levels/level-21.md
started: 2026-10-15T10:53:58-06:00
took: 16m
---

# 47. Level 21 playable: Touchdown Tommy

A delegated map, docs/levels/level-21.md, traced Touchdown Tommy and
checked it in the oracle. It runs on level 20's code; the port does the
same with a table of differences.

| | level 20 | level 21 |
|---|---|---|
| held | cookie box (442), 15 | bottle (259), 20 |
| eating | animation 42, sound 123 | animation 40, sound 43 + (rand & 15) |
| a throw | free | costs one (the bottle's +0x50 effect, 0x80060e3c) |
| win | cookies at 0, cine 64 | a drink to 0, cine 50 |
| loss | cine 65 | cine 51 |
| round start | keeps the holder | swaps back into Tommy (0x8007a0ec) |
| HUD | box sprite 313 and "%d" | gauge 366 over 365, max 20 |

Level 21's EpisodeStart (0x8007a444) gives Lil a new home at (10030,
779, 10051) and the end of cine 49 hides 13, 15 and 16.

Checked: two drinks, 20 down to 18; the football field with Angelica
after the bottle and the gauge bottom left.

**Still unknown:**

- The bottle drawn as part 1238 / model 1245.
- The effect 0x80054708.
- Whether twenty drinks are reachable before Angelica in natural play.
