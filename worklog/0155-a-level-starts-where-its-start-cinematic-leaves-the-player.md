---
number: 155
title: A level starts where its start cinematic leaves the player
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T08:39:55-06:00
took: 56m
---

# 155. A level starts where its start cinematic leaves the player

The oracle, with each level's start cinematic played (the probe's
`--ret 0x80074958=0` had skipped them), shows the player afterwards where
the cinematic's last frame posed it, not at the loader's place: level 13
(22739, -5365, -5925) heading 711, level 15 (44863, 8587, 12281) 2899,
level 18 (2333, 779, -926) 1647 (the port started it on the hub's spot),
level 20 (25993, 779, 17711), level 21 (4225, 779, 4176) 734, level 2 out
of the playpen. The stream's last frame holds exactly these (`Frame::poses`
by id): its actors are the level's own objects (0x80073c34) and stay
where it leaves them.

`Game::open_from` now plays that out before the level's EpisodeStart: with
the start cinematic the app plays (not the hub's, golf's, or level 2's from
its checkpoint), the last frame's pose with the player's id moves the
player (onto the floor; heading -2 atan2(y, w) of the root key, as the
oracle's headings come out), and the poses of characters the loader put in
a room move them (level 7's babies, in none, stay out of play). A level's
own EpisodeStart still moves the player after (level 3's respawn, level
11's Spike at (25252, 751, 995)). Level 2's hand-placed start is gone for
it. `the_start_cinematics_leave_the_player_as_the_original` holds five of
them.

Level 10 now starts with Chuckie outside the playpen, as cine 44 leaves
him; the test goes home to the pen's own place. Level 11's bones by walk:
up behind the run-up, so the jump needs no turn (92 frames a bone instead
of about 150), which keeps the ring inside its 110 s with Tommy where the
cine put him.

**Still unknown:**
- Levels 4, 5, 6, 7, 11 and 17 with their start cinematics in the oracle
  (the character hook leaves the player in the void, or moving).
- Whether a cinematic's actors also change the objects' rooms.
- Mid-level cinematics' poses (the port still places by hand where a level
  says).
