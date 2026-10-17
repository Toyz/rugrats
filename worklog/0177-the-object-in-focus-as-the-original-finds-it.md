---
number: 177
title: The object in focus as the original finds it
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/chuckies_glasses.rs, crates/rugrats/tests/hle.rs
started: 2026-10-16T23:52:54-06:00
took: 9m
---

# 177. The object in focus as the original finds it

The object in focus (player +0x1e8) is the original's, and level 10's
playpen catch with it.

**Focus** (0x8006af4c, `Game::focus`): each placed object of template
flag 8 (the ones the collision tests pass through: rugs, the playpen)
whose ball, its model's radius (+0x86) about its bound's middle, the
body's cylinder overlaps (0x8006abbc: the body box's widest across as
radius, its tallest as half height), and whose model the segment from
the body's part named by the hierarchy (header +0x24) down that half
height meets (0x8006a714). The port tested float boxes from the body's
top to its feet (worklog notes said "the boxes stand in for both").

`hub_run_across_the_playpen_as_the_original`: a run across level 1's
playpen (template 1032); the object in focus is the original's on every
frame of the run (`FOCUS=1` logs it), and the positions are the same.

**Level 10's playpen catch** runs while the playpen is the object in
focus (0x8005ae98: +0x1e8 of template 1032, then 0x8007f2e8), not inside
a rectangle fitted to the oracle's catches (a guess, gone).

The landing's sound on template 505 and the walk's touches (rugs 1160 to
1163, touches 12, 13, 15) take the object in focus the same way.

**Still unknown:**
- The part's joint comes from the port's float pose (0x8004a5dc's
  integer posing is not ported); a joint a unit off could change a
  focus at an edge.
