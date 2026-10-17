---
number: 202
title: The tools compare a character's height: the original's has the root's lift in it, the port's place is the floor and the feet; Mr. Friend's posed parts are the original's to the unit
date: 2026-10-17
area: test
files: crates/rugrats/examples/npc_check.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T16:07:41-06:00
took: 1h 7m
---

# 202. The tools compare a character's height: the original's has the root's lift in it, the port's place is the floor and the feet; Mr. Friend's posed parts are the original's to the unit

The oracle's tools had never compared a character's height. `npc_check` now does (`YCHECK`): the original's height (+0x1c) against the port's place raised by its root curve's height, which is how the port draws and poses a body. It can also print the first posed parts beside the original's (`PARTS`: +0x158, 32 bytes a part, the place at +0x14).

**What a character's height is.** The original's +0x1c already has the animation's root lift in it. The port keeps its place on the floor (the floor, +0x148, plus the feet, the root part's y offset) and adds the lift where the body is drawn and posed. Level 9's Mr. Friend shows it, given the original's state at frame 0: his floor 8192, feet -500, so a place of 7692; the lift 6, 3, -1 by the frame. His first four parts are then the original's to the unit, x, y and z. Level 13's runner held the same rule frame by frame (place plus lift is the original's height).

**The tools' sync** gave a character the original's height as its place, so the lift was counted twice (Mr. Friend's parts were 8 low). The sync (`npc_check` and `sync_npc`) now sets the place to the original's floor plus the feet. A character the original never stepped keeps its own height there (its +0x148 is its height), and so does a paired thing that is no body.

**The sweep with heights** (levels 2, 6, 9, 10, 11, 20, 300 frames from frame 0): no character is off but those below.

**Still unknown:**
- Level 11's runner on its posts is 1 off on about half its frames. Its lift is the original's; the floor under a runner differs by one, a rounding of the slope not found.
- Level 20's babies draw 21 below the original's (lift 16 for 37). Their base track begins its first frame again every frame (the stand's re-ask, worklog 198), and the original takes the frame's whole lift there, where the port blends from where the body was.
- Level 3's brothers at their posts (worklog 201): the wait (table 31, a state whose word has 32) moves the original's place by the lift's changes, and an adult's lift there (+0xde 1325) is not the place's. Not settled.
- Level 2's follower Chuckie is off on 74 frames and level 10's hiders on 3; not looked at.
