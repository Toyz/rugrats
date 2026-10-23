---
number: 241
title: The put-down's way ahead, and the object lists' order
date: 2026-10-23
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-23T00:16:55-06:00
took: 1h 11m
---

# 241. The put-down's way ahead, and the object lists' order

The put-down now tests the way ahead as the original does (0x80060810). A thing put down in front of an object comes back toward the player, as it does in the original; the port had set it down where the object stood. A level's or a cine's drop now goes through the same put-down and flies.

**What worklog 240's divergences were.**
- **The original's 155 push-back.** It came from the tool: teleporting the player leaves the original's carried object (+0x18) where it was until its draw, and 0x80067ba4 stretches the body's box to it. Once stack_check moves the carried object with the player, neither side moves, and both refuse the plank's put-down.
- **The port's 214 rise.** The tool had put the player inside the plank's footprint (the plank lies along z, 860 long).
- **The real difference showed up with both lined up.** Put down 700 before a plank, the pillar came to rest at z -24185 on the original and -23700 on the port.

**0x80060810** (objectives.md, the put-down's step 1; `Game::put_way`):
1. The far end is the greatest z of the body's box together with the carried object's boxes in the player's frame (0x80060594).
2. The way runs from the player's place (with the animation's root) straight ahead to that far end.
3. 0x8006c438 tests the way against:
   - the world (0x8002cd7c);
   - the first object of the nearby list it crosses (0x8006c334: list order, not flag 8, not the player nor the carried object).

   The object's point wins when nearer.
4. Met, the carried object moves by the hit less the far end, in x and z.

It was hooked in the oracle at 0x800608b0: the way from [104222, 838, -24180] to 778 ahead met plank 651 at -23887, pulling the pillar back by 485, and the drop flight's 96 did the rest.

**The put-down** (`Game::put_down`, 0x80062ac4) is now shared by:
- the player's put-down;
- `drop_carried`, which the levels' and cines' drops use (0x80072ef0 and 0x8007fd0c call 0x80062ac4(player, 1)). It had set the thing 100 above the player's feet with no flight.

Spike's put-down, not thrown, goes 1024 further ahead with line 190, as 0x80062ac4 has it.

**Checked.** `things_put_down_as_the_original` (tests/hle.rs), level 3: a tipped pillar picked up and put down 700 and 800 before a plank and 1500 off in the open. Its resting place is the original's in all three ([104211, 717, -24185], [104211, 717, -24184], [104212, 717, -24500]).

**Room lookup.** 0x8008864c's room lookup takes the first room holding the start (0x8002b660 with no hint, inclusive), so the clamp into that room's box never moves it. The clamp only matters for a hinted lookup, which the flights do not make.

**Worklog 240's lists, corrected.** 0x8001a1c0 is no ring:
- It puts a node {prev, next, item} at a list's head and returns the node as the new head; the head's prev is the tail.
- 0x8001a640 and 0x8001a698 go down the nexts from the head, newest first, and 0x8001a534 pops the head.

So:
- What rested on a thing picked up falls newest first.
- In the think's case 2, the thing's own list is left as it was; the port had put the met object in it too.
- stack_check reads the lists that way.

The nearby list's build is also written down (objectives.md, Unknown):
- 0x8006add8 goes over the player's room's lists +0x48 and +0x4c, then each portal's room.
- 0x8006abbc tests each object's bound box middle and +0x86.
- Each object it passes goes in at the head.

The port does not build it yet.

**Checked after.** The suites passed before the list fix: every test file, and hle 40. After it, the default tests and the two things_ hle tests were rerun and passed.

**Still unknown:**
- An object put down outside every room: 0x8008864c makes no flight and 0x80062ac4 sets it on the floor (0x8005fdb4, +0x148). The port flies it.
- Spike's put-down is ported but unchecked. Level 4's things (105) could not be picked up by either side from stack_check's place. The part slot 0x24 back to model 79 is not ported.
- A stacked drop could not be set up on both. The way at mid-body height meets anything taller than about 160, so only a low thing can be dropped onto:
  - in level 18 the original would not pick up from the tool's place;
  - in level 13 the original's player ends 4800 higher after the put.
- The nearby list's order and test (0x8006add8, 0x8006abbc). The port measures from a thing's place with its farthest box corner and goes down the things in their own order. The original measures from the bound box's middle with +0x86, in its room lists' order, which the port does not keep.
