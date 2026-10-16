---
number: 156
title: A cinematic in play leaves the player where it ends
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/app/flow.rs, crates/rugrats/src/media/cine.rs, crates/rugrats/tests/play.rs, crates/rugrats/examples/cine_last.rs
started: 2026-10-16T09:36:07-06:00
took: 10m
---

# 156. A cinematic in play leaves the player where it ends

The start cinematics leave the player where their last frame poses it
(worklog 155); one in play does the same. On the oracle, level 2 with its
start cinematic played and the player put into room 69 at frame 5000:
cine 3 (the gap) plays, and from then on the player object stands at
(45196, -688, -19249), heading 1331, with Spike's object under it. The
stream's last frame poses id 1492 at (45196, -911, -19249) with a root key
giving heading 1331; the y is the floor's. The port had left Tommy riding
Spike where he walked in, at (34975, -2710, -17454), ten thousand units
short.

`Game::cine_left` now holds what `open_from` did for the start cinematic:
the last frame's pose with the player's id moves the player (onto the
floor, heading from the root key) and those of characters in a room move
them. The app calls it when a cinematic that resumes the level ends,
before CineFinish, skipped or not (`Cine::last`); a level's own
CineFinish still places after it (level 5's float start after cine 38).
The tests' `Played` does the same. `a_cinematic_in_play_leaves_the_player_as_the_original`
holds level 2's: (45196, -688, -19249), heading within 16 of 1331.

`examples/cine_last.rs` prints a cinematic's last frames: its actors, and
each pose's id, place, root key and the heading it gives. Level 3's cine
86, level 9's 18 and level 12's 67 pose the player too (at (113853, 789,
6158), (24342, 7968, -2529) and (28657, 777, 13322)); the port now leaves
him there.

**Still unknown:**
- Whether a skipped cinematic (Triangle) also leaves the objects at its
  last frame, or where it was when skipped.
- The oracle after level 3's cine 86, level 9's 18 and level 12's 67 (not
  run; the stream says where).
- Whether a cinematic's actors also change the objects' rooms.
