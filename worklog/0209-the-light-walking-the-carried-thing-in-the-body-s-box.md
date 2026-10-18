---
number: 209
title: The light walking, the carried thing in the body's box, ACTION's touched object, and running into things as the original
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/ground.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/doors.rs, crates/rugrats/src/levels/let_there_be_light.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, docs/engine/objectives.md, docs/engine/gameplay.md, docs/levels/level-12.md, docs/levels/level-03.md
started: 2026-10-17T19:15:52-06:00
took: 1h 18m
---

# 209. The light walking, the carried thing in the body's box, ACTION's touched object, and running into things as the original

Level 12's light proven walking, and with it three engine rules: what a carried thing does to the body's box, when the carried thing is placed, and what ACTION's use takes.

**The light walking.** In the oracle, Up and Triangle held from where the boot leaves Tommy (LP 202), with 0x80091834's speed read at its call:
- **The speed term is 0.** The body's step +0xd8..+0xdc turned by the angles has z 0 on the ground; only +0xda, the lift, moves.
- **Event 2048 lengthens the shot.** On a frame whose animation event 2048 fired, 0x8005f4cc calls 0x80059d3c with 2048 instead of 100 and clears the bit, so the shot is about 84,000 long. Three such frames came in 24.
- **The facing is the handler's.** It is the heading before the body's turn this frame (0x8005bc80 comes after the handler): `Player::handler_heading`.
- **The aim sees the characters as before their step.** The main loop (0x80018c38) runs the player's update before the objects' (0x80072bf0), and 0x80059b34 runs in each character's own update. So the aim target, its place and its size are as they were before the characters' step: `Game::npcs_before`, `Game::aim_before`.
  - The strike test showed it: aimed at the monster's new place, the original's shot reads the place before the step.
  - After the monster comes back, the original still aims at it from behind for a frame.
- The shot is made in the update and stepped after the objects. `Game::shots` holds a frame's shots (the pool 0x800e6ef0), and the tests read them.

**The carried thing widens the body (0x80067ba4).** For the player carrying, the body box every probe and test uses is the hierarchy's grown to take in each of the carried object's model boxes. Each box is moved by the object's place less the player's, turned into his frame (the matrix +4 transposed). The original's box was read at 0x800e3208; with the flashlight at the hand it reaches a wall upstairs a frame sooner than the bare body's. The port ignored the carried thing. `Player::carry_box` is now set each frame by `Game::carry_box`.

**The carried thing is placed from this frame's pose.** The main loop runs the update, then the draw. The character draw poses the parts, then places the carried object (0x8005ff88 mode 1), the grab's own frame included. The port placed it from the frame before's pose and skipped the grab frame (`grabbed_on`).
- The hub test had matched the old order because a hub frame runs past the oracle's sample, so the sample saw the draw before's place. It now reads the place where the draw leaves it (a hook after the call, 0x800323b0), and matches over all 40 carried frames.
- The level 12 light, whose frames finish before the sample, matches the new order directly.

**ACTION's use takes the touched object (0x800631d8).** That is the nearest of the nearby list within reach and a quarter turn: things, characters (their hierarchy's box) and doors. `Game::touched_any`, sharing `touched`'s reach test (`reaches`).
- Level 12's refrigerator was a 2D check within 1500, so Triangle upstairs above the kitchen played cine 67 and dropped the light. The original ignores the door there (4700 below).
- Spike (1200 across) was the same kind of check. Both are now the touched door or character.
- `level_12_let_there_be_light_is_won` still wins.

**Running into things, carrying (0x8005e1f4).** Proved in level 6 as Chuckie, with box 9 in hand run at box 8:
- **Without the carried box in the body's box** the port met box 8 two frames late and ended 340 off. With it, the bumps match.
- **The bump needs a run.** 0x800590d4: the stick held and the speed over the run's factor times 3933 >> 12. The port's test was "over the walk's factor", so a run at 16 bumped.
- **The bump or the stand.** Held by a thing with no ledge in reach (0x80058da4's answer), it bumps when the thing met is none (a wall) or one that cannot be picked up. A thing that can be picked up (+0x00 bit 0x10000, template +0x28) stands him (state 0). A ledge he cannot climb bumps.
- **A climb runs no ground step.** It is state 6 (0x8005f310), so it never falls. The port's climb ran as the stand and could fall mid-climb.
- **A climb starts at the face (0x80062cf4).** The body moves to the probe record's point (+0x104) plus its normal (+0x114) times the hierarchy box's half depth (not the carried thing's), and the wanted heading turns square to the face.
  - For a height-field wall that point is the COL segment's (0x80064178 -> 0x800671f8 -> 0x8002cd7c). `ground::wall_met` now keeps it, as `wall` kept only the normal.
  - Carrying, the box ahead finds a ledge from farther back; without the move to the face the port's climb began there and fell off.
- Level 6's box route (`level_6_boxes_on_the_steps_reach_the_platform`, `level_6_toy_palace_is_won`) needed the walk's arrival within 400 while carrying, since the box held keeps the body back from a face. With the original's climb it plays through.

**HINT09 (level 3).** Every call of 0x80044434 passes a constant (0x800783d8's after cines and four others) or shows the last card again (0x800448c8). No call passes 680, so the card is never shown: off the level 3 doc's unknowns.

**Oracle.**
- `level_12_light_walking_as_the_original`: Tommy's place every frame for 26, and every one of the 24 shots, long ones included.
- `level_12_light_strikes_as_the_original` now holds with the in-game shots, through the monster's comeback.
- `hub_pick_up_as_the_original`: the held toy at each draw, 40 frames.
- `level_6_chuckie_box_as_the_original`: now with 30 frames of Up into the step with the box in hand. Bumps on the original's frames; the box at each draw.
- `level_6_box_into_a_box_as_the_original`: Chuckie with box 9 run at box 8: the bump, the run again, the stand. Places and states the original's every frame.

**Still unknown:**
- The other levels' uses still checked by distance (`*_REACH`: levels 1, 3, 5, 6, 7, 9, 10, 16) against the touched object.
- A bump's back-step drifts across from the original's by up to 5. The original holds one frame without motion between keys 428 and 429, and once turns a unit further.
- Turning in place after the stand, the heading and the camera are a unit off on a few frames (the follow camera's rounding).
- A climb onto a ledge while carrying, compared in the oracle: the route plays, but no oracle run climbs with a box.
