---
number: 242
title: The nearby test, and no flight outside the rooms
date: 2026-10-23
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-23T01:28:20-06:00
took: 1h 18m
---

# 242. The nearby test, and no flight outside the rooms

The nearby test now measures as the original's (0x8006abbc). A thing flight now starts only inside a room (0x8008864c), and a put-down with no flight is set on the floor (0x8005fdb4).

**0x8006abbc.** The test is against the object's bound box middle (+0x78..+0x84, turned, plus its place) and its +0x86. The original passes when:
- across: d² is no more than (+0x86 + cylinder radius)², in 32-bit unsigned;
- up and down: within +0x86 of the cylinder's height.

**What +0x78 and +0x86 hold.** stack_check OBJBOUNDS=1 printed every object in levels 1, 3, 6 and 16. Model objects mostly have a cube of the model's radius, with +0x86 that radius (154 of 178 in level 6). Every thing is such a cube (BOUNDS=1: level 3, 6 and 12's templates), so a thing's bound middle is its place. The ones that are not:
- characters (a cube of 1773, +0x86 8192);
- collectibles (class 11, 649);
- a few big objects: 140, 185, 231 and 573 in level 1; 494, 623 and 883 in level 3; 1028 in level 6.

**The port's change** (`Game::nearby`). The port had measured from a thing's place with its farthest box corner, in floats. It now uses the model's radius and the integer compares, for things, doors and placed objects. For level 6's 1164 the corner was 484 and the radius is 483.

**No room, no flight.** `start_flight` finds the first room holding the start, as 0x8008864c does with no hint. Outside every room there is no flight, and the thing stays where it is at rest; a throw does nothing more (0x80060b44). A put-down then sets it at the player's floor (+0x148) plus the least top of its boxes (0x80062ac4 at 0x80062c64, 0x8005fdb4). With the lookup inclusive and unhinted, the clamp into the room's box never moves a start.

**Level 1 in stack_check.** Two differences came from the boot, not the port:
- the original refusing a put-down;
- the port not picking up a block.

The hle boot without cinematics leaves the hub's blocks (ids 55 to 59) shown. The port hides them at the start, as cine 7's end hook and EpisodeStart do (0x800732d4, 0x80078e7c).

**Checked:** every test file; the hle suite (40), and the thing tests after the room change.

**Still unknown:**
- The nearby list's order: the original goes over the player's room and its portal rooms, newest first. The port goes down its things from every room.
- The bound of the few big objects above. Their template's model is not what their bound was made from; it may come from a hierarchy or another model.
- A flight from outside every room has not been seen on the original.
