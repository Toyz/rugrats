---
number: 240
title: Which rests on which, and GRAB's choice, as the original's
date: 2026-10-23
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/boxhit.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/flights.rs, crates/rugrats/examples/stack_check.rs, docs/engine/objectives.md
started: 2026-10-22T23:19:33-06:00
took: 57m
---

# 240. Which rests on which, and GRAB's choice, as the original's

The stacking rule (0x8006c7a0) is decoded, ported and checked against the original. A thrown thing now rests on what its flight met, and GRAB now chooses its target as the original's 0x800631d8 does.

**The flight's +0x98.**
- It is the last object the flight met that is not a character (object flag 0x4000). 0x80087594 and 0x80086ef0 set it at each meet.
- The port's `Thing::last_hit` kept the opposite: the NPC or the player hit. Nothing read it, and `bounced` was never read either.
- Both are replaced by `met` (+0x98) and `on` (+0x70). `Game::loads` holds the +0x74 lists.

**0x8006c7a0** (objectives.md, "Which rests on which"; `things::stacking`). For each of a's boxes:
1. The box's middle goes into the world, then into b's frame.
2. A b box holds it, grown across by the larger of the box's half-width and half-depth, and up and down by 3/4 of its height.
3. The two half-heights then decide: 2 (b on a) or 1 (a on b).

The original's quirk is kept: once one box was inside, a later box is judged by b's last box.

**Checked.** `things_stack_as_the_original` in tests/hle.rs moves thing a about thing b on a grid and compares each answer with the original's. It calls 0x8006c7a0 through `call_nested`.
- Level 3: planks and tipped pillars, 5 pairs.
- Level 6: boxes, 2 pairs.
- 34587 calls in all, 5460 answers not 0, none off.

**The think's answers** (0x8004df94 at 0x8004e018 and 0x8004e030):
- 1: the thing rests on what it met and joins its load.
- 2: the object's +0x74 is made of the thing's list plus itself, and its +0x70 is the thing's. That list is empty, so the object ends up bearing itself and can never be touched again (`come_to_rest`).

**The pick-up** (0x800601e4): the thing leaves what it rested on, and what rested on it falls oldest first (0x80062928: (0, 0, 4096) unturned, at speed 10).

**GRAB** (0x800631d8, `touched`):
- It skips a thing bearing a load.
- Going down the list, a thing whose first box rests on the nearest so far is taken over it at any distance. The nearest is forgotten even when the new one is out of reach. A thing under the nearest is passed over.
- The reach is now in integers:
  - the floor square root across (0x8001f4ec);
  - the way's VectorNormal and ratan2 (0x8009f278, 0x800760f4) against the heading, strictly within D_800c430a.
- The port had used a float hypot and atan2.
- `touched_any` skips doors and placed objects bearing a load.

**Also:**
- tests/flights.rs had not built since worklog 238, when `Flights::step` gained its meets argument; the suites only ran lib, play and hle. It builds again. Its `a_thrown_thing_knocks_parts_off` had set a thing flying with no flight record, which the record flights (`flown`) never move; it now starts the flight with `start_flight`. Every test file passes, and the hle suite (39) passes too.
- examples/stack_check: one thing put down onto another on both, with the oracle's think and the 0x8006c7a0 calls hooked.

**Still unknown:**
- The nearby list's order (0x8006add8 over the rooms' lists). The port goes down its things in their own order, which decides ties and the stacking preference.
- Teleported next to a plank while carrying (stack_check, level 3), the original's player moves back 155 over two frames at any distance. At the pillar, the port's player rises 214 instead. The tool's put leaves the two overlapping in different ways, and neither push is traced.
- A put-down through stack_check could not be lined up on both, because the cameras turn Up differently after the put. The rest link is checked only through the function (above), not through a drop.
