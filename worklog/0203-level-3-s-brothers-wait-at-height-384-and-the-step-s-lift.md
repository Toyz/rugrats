---
number: 203
title: Level 3's brothers wait at height -384 and the step's lift, the toss's pies are the shared ones and its strikes are its own; a level 3 pie matches the original
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/levels/aisle_seven.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T17:14:59-06:00
took: 26m
---

# 203. Level 3's brothers wait at height -384 and the step's lift, the toss's pies are the shared ones and its strikes are its own; a level 3 pie matches the original

Level 3's pie toss is the original's too, on the same pies as level 16's (worklog 201). A pie matches the original on every frame up to the original's first lag frame, with Tommy's animation tracks the original's.

**Larry and Steve at their posts.** Mode 7 has a case of its own for registry ids 75 and 76 (0x80084428): their height (+0x1c) is put to -384 every wait frame, before the body's update adds the step's displacement (+0xda). Traced a frame at a time, Steve's height goes -243, -330, -384, -392, -402, -393, -384: -384 and this step's displacement (141, 54, 0, -8, -18, -9, 0). A step's displacement is the frame's lift key over the part of the frame it covered (0x80055904: the key times the fraction now less the fraction before). On a state's first frame it is the gap to the key instead (+0xde: the key less the body's height over its floor and feet, 0x80055b3c). The port drew a waiting brother at his place raised by the whole lift so far, about 1100 under the floor. `npc::step` now puts their place at -384 less the lift before the step, so they are drawn at -384 and the step's change of it.

**The toss** (0x8007bc60):
- It runs on `Game::start_pie_toss` with Larry's distance in 3D.
- It sets template flag 8 on templates 489, 566 and 488 and clears it at the end (0x8007e3c0, `Game::set_template_solid`), so the bakery's props no longer hold the brothers.
- Tommy stands where he is put on the counter (the original's +0x148 is -2048, 7 under his feet; held, he never falls to it).
- `levels::pie_toss`, the port's own pies, is gone.

**Strikes.** Both levels' scripts took the frame's strikes (`Game::struck`) for their thrown things before the toss saw them, so a pie on Angelica or a brother was never counted by the toss. Each toss now takes its pies' strikes first and leaves the rest.

**Oracle.** `level_3_pie_flies_as_the_original`:
- Tommy is put into room 19; the port's cine 9 is finished at once, as the boot skips the original's.
- After the toss starts, the port is given the original's heading and animation, which the skipped cine leaves apart.
- A pie is thrown at speed 450. It is the original's on every frame before the original's first lag frame (its time, 0x800c3e98, not 10 on), and Tommy's tracks are the original's to then.

The play tests that win both tosses still win them.

**Still unknown:**
- The pie's strike on a brother is not compared frame for frame: the original's lag frame comes before it in this run.
- The first-frame gap for a brother's wait as the port has it (its root curve's blend from where the body was) against 0x80055904's (+0xde).
