---
number: 245
title: Collectibles met by the walk's probes
date: 2026-10-23
area: play
files: crates/rugrats/src/world/game/goodies.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/ground.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-23T04:11:50-06:00
took: 1h 14m
---

# 245. Collectibles met by the walk's probes

On the ground a collectible is now taken as the original takes it: the walk's probes meet its box, rather than the player coming within the model's radius plus his own. The walk takes no step on that frame.

**What the original does.**
- **The box.** A collectible's set-up (0x80048a44) makes its model's first box the cube of 375 (worklog 244).
- **The probes.** The walk and the run (0x8005e1f4) call the move (0x80064a34). Its object probes (0x80063d3c) go over the nearby list and put the first object met in +0xf4.
- **The touch.** When the move answers 1 (held) and +0xf4 is of class 11 with a +0x58, it runs that (0x800514b0, the taking) and goes on to the end of the handler, at 0x8005e31c. No 0x8005caac: the step is not taken that frame.
- **Seen on the oracle.** A hook on 0x800514b0 in level 1, running at goodie 3:
  - the touch is called from 0x8005e34c;
  - the player stands at x 29055 two frames running, the take's and the next.

**The port.**
- Each frame `Game::touchable` gives the player the collectibles not taken (not kind 16) as solids: the model's boxes turned by their angles, kind 12's model 0, sized as their set-up left them.
- `Player::ground_move` meets them after the other objects (`ground::objects` over the list).
- Held by one, the player keeps it in `touched`, its move is (0, 0), and it is not held for anything else (no bump, no slide).
- `Game::touch` takes that one.
- In the air the old test is kept: within the model's radius plus the body's across, 1200 up or down.

**Tools.**
- The probes keep what they tried this frame (`Player::object_probes`).
- tests/hle.rs `goodies_taken_as_the_original` (GOODIE=level,k,away,side,heading,frames) prints the frame each side takes it, with GDBG=1 both sides' probes and GREPORT=1 the frame-by-frame report.

**Checked:**
- Level 1's goodie 3, approached along x from 1500: the original takes it on frame 6 and the port on frame 7, and each stands still on the frame after.
- The frame between them is the camera. There the original's camera swings off the player's back as he runs (its heading 1024 to 585 over 11 frames, flags 0x9c0 to 0x2100000 and 0x300000, and its eye 200 above the look-at rather than 1024 even at rest). The port's camera stays behind him. So the original's player turns with the camera, its probes reach further across, and its last one meets the cube a frame sooner.
- **The play tests' collecting** (`collect` in tests/play.rs) had, where no walk led to a pickup, put the player on it. Its probes then start inside the cube and never meet it, on the port as on the original (0x80067284: met past the start). After such a put it now goes 900 back the way it came, then walks at the pickup again. Levels 15, 17, 18 and the whole game are won again.
- Every test file, and the hle suite (41).

**Still unknown:**
- The follow camera there (0x80027304): why its eye sits 200 above the look-at in that part of level 1, and why it swings as the player runs.
- The air's touch (0x8005ee78: 0x80066014's object met, a class 11 touched and the move then not held), kept as the radius test.
- The nearby list's order also decides which object a probe meets first; the port meets the collectibles after the rest.
