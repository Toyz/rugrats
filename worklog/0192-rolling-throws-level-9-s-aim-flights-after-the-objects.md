---
number: 192
title: Rolling throws, level 9's aim, flights after the objects
date: 2026-10-17
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/src/levels/golf.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs, crates/rugrats/examples/throw_check.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T05:45:01-06:00
took: 55m
---

# 192. Rolling throws, level 9's aim, flights after the objects

Rolling throws, level 9's aim and the order of a frame's flights now follow the original's code. The level 9 ball is proven on every frame.

**The aim target (0x80059b34).** Each character's update (0x80082230, in all of 0x80084084's exits) runs it for itself. It keeps the player's +0x1f4 target:
- The target is the nearest character within 512 of the heading and 768 of the pitch. The yaw is ratan2 of the way less the heading. The pitch is minus the asin of the height difference, cut to 16 bits.
- Distance is the squared distance by 0x8007681c, or -1 when an axis is past 32767.
- A held target stays until a nearer one passes the bounds, or until it fails them itself.
- In levels 20-21 only the babies count (templates 2, 6, 7, 11).

The port worked this out afresh at each throw, with a float atan2 and no pitch. Now `Game::consider_aim` runs for each character after the NPCs step, in their order. The box toss reads the same target and clears it on the swap, as 0x80057570 does.

**Level 9's throw (0x80061c98).** The way to the target is made a unit. Its angles are ratan2 and minus asin (0x800760f4). (0, -384, 1024) is turned by those angles when they lie within 512 of the heading, else by the player's own. The result is made a unit, its height set to -300, at speed 170. It is all integer now.

**Rolling (0x80086ef0).** A template whose +0x30 is 1 sets the object's flag 0x800000. Its launch flags are then 0x190f0700: it rolls (0x10000000). After the same move (0x8004fbb4):
- The segment runs from where it was to its leading edge (reach = radius, at least 10). It is bent along the floor it was on (0x80086b58: -v along the plane by 0x800763d0, as long as the segment by 0x80075f58, plain >> 12).
- The segment meets things as a flight does.
- Off a surface met head on, it backs off and goes back along the reflection at |v| bounce >> 12. There is no -95 and no stop.
- Then it rolls (0x80086cd0): the centre `reach` over the floor, the speed less the friction (98 dt >> 6 = 15) along its unit, rounded toward zero, plus a slope's push (0x80086a20, now by the game's asin). With no floor, or too slow on the level, it stops.

The port used to fly balls as projectiles. Golf's ball (flags 0x19030000, a roll without gravity) now uses the same `roll_on`. Its shot list was searched again under it.

**The flights step after the objects.** In the original's frame (0x80018c38) the player comes first (0x8005a068), then the objects (0x80072bf0), then the flights (0x8008851c). The port flew before the NPCs moved. It now flies after them, with the characters' solids moved to where they now are; their pose stays as last drawn. With the original's Friend given to the port, the roll's meeting with him matches the original's point and normal.

**Oracle.** `level_9_ball_rolls_as_the_original` boots level 9 with its cinematics, runs past cine 17 (where the port starts), picks up ball 3 and throws it away from Mr. Friend. The ball is on the original's place on all 107 frames of the roll and where it stops. The other 11 scripts still pass.

throw_check gains LIST, AIM, SYNCNPC (each NPC given the original's animation and, after its frame, its place), MEETS (the original's meeting in the roll), CINES and SETTLE. npc_check gains CINES, SETTLE, PUT and MOVES.

**Still unknown:**
- Mr. Friend's chase: the original keeps him in state 19, then 0, for four frames and then walks him at 39 a frame (speed index 4 at +0x184, the way +0x178 a short (0, 0, 3)). The port starts him at once at double speed (2 x base past twice his range). Both the move (0x80084084's follow, the shared body step) and its timing need porting.
- What +0x18c == 8 is for the player (0x80061c98 then throws without the aim).
- Golf's ball still meets objects by the port's boxes and the world by its own segment, not 0x80086668.
- The put-down still slides by the port's stand-in; 0x80062ac4 launches through the same 0x80061f88, so a rolling template's put-down rolls.
