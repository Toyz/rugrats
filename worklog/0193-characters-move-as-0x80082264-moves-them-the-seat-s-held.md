---
number: 193
title: Characters move as 0x80082264 moves them; the seat's held probe; post cameras
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/camera.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/grandpas_teeth.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T06:40:28-06:00
took: 55m
---

# 193. Characters move as 0x80082264 moves them; the seat's held probe; post cameras

The characters now move as the original moves them. Mr. Friend's chase in level 9 is on the original's place and heading on every frame of 280.

The NPC update (0x8005a068 with object flag 0x4000) takes 0x80082264 where the player takes the stick (0x8005be24). The rest of the body (the animation, the state's handler, the turn) is shared. The port's `Npc::go` now follows these steps:

1. **The steer (0x800820e0).** The heading wanted (+0x2c) is ratan2 of the way to the goal (+0x67c). The heading turns toward it by at most 512 (D_800c4444) and its matrix is made again. It skips this with +0x688 bit 4, or in mode 8 without bit 64.
2. **The look ahead (0x800825fc).** It runs only with a speed and without +0x688 bit 0x4000. There are three casts from the body to the goal, each at the goal's place and the body's height, 384 over the floor less the body's height over it: one from the left side (the hierarchy box's half width clamped 500-1000 out), one from the right, and one from the middle. If the sides are not both blocked and the middle is clear, the result is 1 if the left is blocked, 3 if the right is, else 0. Otherwise two ways round are cast. Each goes from a side to the goal's side, the half width times 8 out (2 in levels 10, 12, 13) and 5 times the half depth (5000-10000) back. If the left way is open, the result is 4 if the right cast was blocked, else 0. If not, a blocked right way gives 5, an open one 2. A result of 1-2 turns the heading 512 one way, 3-4 the other, and the way's length is half again.
3. **A cast (0x80082b60, 0x80090554, 0x8006a9cc).** It takes the nearest object met (t >= 0; not itself, not template flag 16), then the world's segment, which is kept over the object only when nearer on all three axes. The world blocks. An object of template flag 72 or 8, or the character's own +0x1f4, does not.
4. **The way (+0x178).** It is (0, 0, speed), or a quarter of it while the heading is over 1024 from the one wanted. Both are taken as 0..4095, not wrapped. It is turned by the heading and set to the speed's length (0x80076944). These are small integers, so the step's direction keeps to what they can point to: Mr. Friend at heading 61 walks straight along z.
5. **The step.** It is the way set to speed x dt. Then the body's turn (0x8005bc80) toward the heading wanted takes 1025 dt >> 6 (Spike 298, 0x8005bbf4).

**The speed.** The table at 0x800b9698 has two rows (+0x670). Row 1, (0, 2, 3, 6, 11, 13), is set by 0x80081bac for every character in levels 9 and 10 and for character 3 in levels 20-21. Mr. Friend walks at 2, doubled past twice his range. The port knew only row 0, so he ran at double the original's speed.

**The distance.** The ranges are measured by the game's 0x8001f57c: |dx| + |dz| less half the smaller. The port measured them as Euclidean, so his walk slowed eight frames early.

The game builds the casts' objects each frame (`npc::Obstacle`): this frame's solids with their template flags, the characters and the player.

**Oracle.** `level_9_friend_chases_as_the_original` puts Tommy far off past cine 17. Mr. Friend is given the original's place once his first wait is over, then chases for 280 frames on the original's place and heading, through the ways round. npc_check gains SYNCAT, CASTS (the original's casts and their results) and the look ahead's result.

**The seat (a probe held from a standstill).** On the first frame with the stick pushed, the stand handler (0x8005d11c) probes the step (0x80064a34, mode 1). At the seat the probe holds (1). The way then turns along the face (0x80065120: (-2909, 0, 2887)), and that starts the run (state 2). The handler still puts the speed by at its end (+0x184 into +0x176, 0) because the probe held. The port zeroed the speed only when the step stayed held, so its camera moved on that frame. `ground_move` now reports whether its first probe held, and the stand's first step zeroes the speed on that.

**Late frames.** On one frame of the seat script the original ran late. Its camera's update (0x80028d4c) finished in the next frame's vertical blanks, so the harness read the camera mid-update. The frames after it match. The harness now counts the original's camera returns (0x8002953c into 0x80018db4) and does not compare the camera of a frame that has none. The seat script now asserts the camera heading.

**A room's post camera (kind 3, 0x800249cc).** The look-at (0x800de454) goes halfway to the player across each frame. Its height is left alone within 512 of the body's; otherwise it moves at 298 dt >> 6 a frame (0x800c4a3c bit 1024) until it reaches it. In level 3 the goal is the aim lower. The rows are the game's integer look-at (0x800211e8) from the post to it. The port aimed at its aim point in floats, so it was 6 off. `Game::sync_post_look` gives the port the original's look-at for tools. The harness's put uses it after its first frame, and the pick-up and level 9 scripts now press R1 first and assert the camera. Eleven of the thirteen scripts now hold the camera heading.

**Still unknown:**
- Mr. Friend's first frames: the original holds him in mode 1 (state 19, +0x69c a random 300 to 1200 down by dt) after cine 17, then mode 0 and 4. The port starts him at once.
- The follow mode's other exits (0x80084cd0): the lunge (mode 8, state 28) within range with +0x688 bit 128, and the bite (mode 5) within half the range with the boxes overlapping (0x80083ff0). The port keeps its own bite rule.
- The NPCs' floor and walls on the step: the port's own query, not the shared body's probes (0x80064838).
- The casts take every object; the original walks only the rooms along the segment (0x8002ade0).
- Level 11's post camera leads with the player's speed (0x800249cc's own branch); not ported.
- Why the original ran late on that seat frame (the emulator's timing or the game's own load); whether a real PlayStation does.
- Level 6's camera (Chuckie's box script holds no camera assert).
