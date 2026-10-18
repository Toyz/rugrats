---
number: 206
title: Level 12's flashlight is the original's shot, and a struck Shadow monster shrinks where it was hit
date: 2026-10-17
area: play
files: crates/rugrats/src/levels/let_there_be_light.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/tests/hle.rs, docs/levels/level-12.md
started: 2026-10-17T17:58:25-06:00
took: 45m
---

# 206. Level 12's flashlight is the original's shot, and a struck Shadow monster shrinks where it was hit

Level 12's flashlight is the original's shot now, with no guessed width, and a struck Shadow monster shrinks where it was hit, as the original's does.

**The light is a shot (0x80091834, type 1).** Every frame Triangle is held, 0x8005f4cc -> 0x80059d3c makes one shot record (pool 0x800e6ef0). 0x8009134c steps it once, 0x80090e70 draws it, and the draw frees it.
- Start: hand slot 15's part matrix as last drawn (+0x158, transposed by 0x80010b54) turning (0, 0, 350) (table 0x800bb28c entry 1, +0x24), plus the joint.
- Way (+0x200, 0x80059cdc): toward the aim target +0x1f4 (0x80075e4c: target less player, made a unit) while the target's size +0x36 is above 0. Otherwise the facing (0x80076040). The doc's "+0x1f4 is 0 in level 12" was wrong: 0x80059b34 keeps the target here as in every level, so a monster ahead pulls the light up toward it.
- End: the way made a unit again (0x8009f278) times (100 << 10) / 100, >> 10: 4096 long.
- What it meets (0x80090554 -> 0x80090340 -> 0x8006a9cc): the nearest solid it crosses, not the owner, nor a template of flag 16 (template flag 8 does not count here), boxes not grown, unless the world is nearer on all three axes (`Game::shot_meets`).
- An object without a callback still stops it. The doc said otherwise; the code does not skip one.

The port had a cone: 4096 ahead of the body, 700 across (a guess). Now `let_there_be_light::shot` and `shine` follow the original. `Game` keeps the frame's solids for shots. The light mesh is drawn on the shot's two points. Its axis is now traced: 0x8008e6c8 of the way turns the mesh's +z back at the hand, so the old order was right.

**The flashlight in hand.** EpisodeStart (0x8007b7c8) sets +0x1f0 alone: no grip and no carry arms. The light's own draw is off (+0x40 0), and the hand part shows it (model 1690; 1310 bare, cine 67/68's start and the put-down).
- The port gave it with `give` (grip, the carry's arms 19 on tracks 1-3) and drew the object at the grip.
- It now uses `hold` and hides the object while in hand. The hand model is swapped each frame.
- With the original's arms, Tommy's parts are the original's to the unit (the oracle's +0x158).

**Mode 10 (0x80084314).** A struck monster:
- stays where it was (mode 10);
- has its hierarchy's box (+0x04..+0x10) zeroed, so the collision tests meet nothing;
- loses 250 of +0x36 a frame while above 0;
- once +0x36 is spent, says speech 143 or 21 and comes back at a group 10 spot with its box and size restored.

+0x36 is the draw scale: 0x8003205c multiplies each part's matrix by it and keeps the joints. So the monster shrinks to nothing where it was struck. `Npc::scale` is new and drawn. The port hid the monster at once and brought it back after 18 frames.

**Settling (0x800452f8).** The floor is found under the object's place itself (+0x18), and its height is that floor plus its feet. The port looked from its place less its feet. That is the same for most bodies, but the Shadows' feet are -1872 (they hover). A monster coming back to an upstairs spot (-5518) found the floor downstairs and stood at -848, where the original's is at -6992. `Npc::snap` now queries at the place.

**Oracle.**
- `level_12_light_shot_as_the_original`: standing at the start, every shot's start and end are the original's for 20 frames, and Tommy's tracks are too.
- `level_12_light_strikes_as_the_original`: a kitchen monster 2000 ahead, the game's rand given the original's.
  - The light goes up toward it and strikes it on the first shot.
  - Its size is the original's every frame (4096, 3846 .. -154), and it comes back on the original's frame.
  - Every shot is the original's for 24 frames, including those aimed at the shrinking monster and the one after it comes back, aimed at its new spot.

**Still unknown:**
- The light's speed term while walking (+0xd8..+0xdc turned by the angles) is not compared; standing it is 0.
- Which of the player's and a struck monster's updates comes first on the frame its size runs out: no frame where the order shows was seen.
- Monster 3333 put far from the camera: the original's hand matrix stayed where Tommy was last drawn (+0x158 is only made by a draw), so a teleported Tommy's shot starts at the old place. The port draws every frame and has no culling of the player.
