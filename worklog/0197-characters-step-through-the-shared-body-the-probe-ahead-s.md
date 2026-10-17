---
number: 197
title: Characters step through the shared body; the probe ahead's fall; posts and waits as the original's
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/ground.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/levels/cookie_race.rs, crates/rugrats/src/levels/aisle_seven.rs, crates/rugrats/src/levels/training.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/src/levels/chuckies_glasses.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T09:38:52-06:00
took: 1h 20m
---

# 197. Characters step through the shared body; the probe ahead's fall; posts and waits as the original's

Characters now step through the player's own body code, as the original's shared body (0x8005a068) steps them. The probe ahead's fall is wired in for every body. Post walkers walk their routes as the original's do. Angelica's whole race in level 13 matches the original: place, heading and animation tracks on every frame of 940.

**The shared body.** `Player::frame` is now the stick plus `Player::drive`, the rest of the frame from the way on: the ground move, the jump, the air, the floor, the state and its animation. A character keeps a body of its own (`Npc::body`, `Player::character_body`) and drives it with its own stick (0x80082264: the steer, the look ahead, the small way, the turn).
- Its animator is lent to the body for the step, and its own solid is left out.
- A character walks under its row's fifth speed (`walk_below`) and has no bump damage, hints or falls that hurt.
- In the air it is pushed on 12 a dt (0x80061330's object flag 0x4000 branch); the stick's push it has none of.
- Through a landing (state 10) the body takes no step and plays it out, the speed kept.
- A bite or stagger playing out (mode 1, state 19) only turns.

**A character's probe (0x80064838).** 0x80064a34 hands an object of flag 0x4000 to it: the middle probe alone (`Ground::probe_middle`), at the body's depth past the step. A held probe lets the character by when its +0x688 has bit 0x1000, with the floor delta 0 (`Npc::passes`, `Player::passes`): the post runners of levels 13, 3 (75, 76), 16, 2 (Chuckie on post 114) and 10's hiders, and character 33. Such a runner stands on the floor where it now is, however high.

**The probe ahead (0x800645d8).** Walking or running with the way open, it classifies the step from here (`Ground::ahead`). A drop past the step limit is the fall (state 4) for any body. The port had judged a fall by a 384 drop, which a 384 stair missed. The fall's launch is the running jump's: the way set to 40 x speed in the body's frame. The original's character leaves at (-13, 0, 680) for speed 17; the player had (0, 0, speed << 6). Its `Ground::ahead` was proven against the original in worklog 171 but never called.

**Posts (mode 6, 0x80082c78, 0x80082ee0).**
- The frame a post is reached, the body still steps, aimed at the post reached: mode 6 takes its place for +0x67c before moving on.
- A random post (flag 4) draws its slot as rand & 3 until one is set (40 draws), else slot 0. The port drew rand % count.
- Otherwise it takes the first neighbour not come from, else slot 0.

**The wait at a post (0x80083e2c, mode 7, 0x80083efc).**
- On entry the speed is put by and the wait (logical 9, table state 31, which moves the body by its root) is asked at 4096, with the line. That frame has no body update at all, since mode 6 leaves once it is mode 7.
- Each wait frame the stick turns her to the player, with no step.
- She resumes once VSync passes the wait (+0x68e; 2 a frame, against the entry's), the player is within range, or he lies within 768 of the way to the next post (both as 0..4095).
- Then state 0 and mode 0 (`Mode::Resume`): one frame with no body update.

**The root across a loop.** A loop's next round carries what the round before still had, as a new state does. It used to throw the body back by a round's across.

npc_check's SYNCAT gives the body its state and a post walker its posts, and pairs characters by the original's id (+0x38). It also prints the original's air velocity and posts (POSTS compares the post tables: all equal). Spike's give-up reads the body's held step (`Player::held_up`), not a step that did not move: the stand's first step never moves.

**Oracle.** `level_13_angelica_races_as_the_original` (`sync_npc`): past cine 6, Angelica given the original's state once, 940 frames to the goal post. Her place, heading and tracks are the original's on every frame: the stairs' fall, the landings, the random post, both waits. All 16 scripts pass.

**Still unknown:**
- +0x688 bit 16's post choice (a neighbour whose post's room flag has bit 1, D_800c4bac): no port character has it.
- The player's own falls now come from the probe ahead too; no oracle script has the player walk off an edge yet.
- The landing on a character's body: the port's floor snap, not the original's 0x800627a0.
