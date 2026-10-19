---
number: 233
title: The class 10 emitters as the original's: their clocks, schedule and callbacks
date: 2026-10-18
area: engine
files: crates/rugrats/src/world/game/emitters.rs, crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/toy_palace.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T18:55:22-06:00
took: 1h 57m
---

# 233. The class 10 emitters as the original's: their clocks, schedule and callbacks

The class 10 objects (think 0x8004e0f4) now run in the port as the original's: their clocks, their schedule and every callback the levels use, each launch checked against the oracle.

**The think.** A class 10 object is on the trigger list with its own enter (0x8004cfd8: record +0x00 = 1, scheduled) and leave (0x80049e5c). While scheduled, each think starts its clock (+0x64 from VSync(-1)) after an enter or a go, and goes when the count is past it by more than the period (template +0x18, VSyncs). 969 and 982 hurt the player first when near (0x900000 / 0x400000 squared), which replaces level 6's stand-in bursts. The class 10 enter is set after the set-up's speech line, so it wins over a talker (level 6's 905 at LP 46 had one, and went 25 frames early).

**The callbacks** (0x800b7eb0 by template +0x09), in `world::game::emitters`:
- 2 (577) and 10 (982): `count` drops at random speeds, the object's matrix less its third row;
- 3 (611): sound 149, a drop that sounds 133 (177 in level 16) and leaves mark 263;
- 4 (614): spinning model 931;
- 5 (615): one of six sprites;
- 6 (616, 1139): bubbles, and the shared count 0x800c4ba0's sounds;
- 7 (706): model 1040 down, sound 147, mark 218;
- 9 (682): model 1016, mark 212;
- 13 (969): six sprites, a random cone, the period as a delay;
- 14 (905) and 15 (906-908): a model along the object's matrix (0x8009f6f8);
- 18 (1215): model 1628, bounce 2500.

Each has its own seed (0x800c41a8..0x800c41c8, 0x800c41d8, all 48732). A launch now carries its sound (+0x64) and mark (+0x46, +0x65..+0x68). The flights play the sound where they meet something (0x80087594); the marks are kept but not drawn.

**What the oracle showed besides:**
- The update list (0x800c43b8) is head-inserted and walked from the head, so the latest scheduled thinks first. Level 6's 969 and 982s going in one frame draw their cones in that order.
- The think reads VSync(-1) during the frame. In npc_check's frames that is the frame-end count less one, so the port's think reads `Game::vcount` less one.
- `Flights::launched` is now cleared each frame; it grew without end outside the tools.

**Tools.**
- npc_check FLIGHTS=1 compares every launch, the model kinds without their unset sprite and size, and with sound and mark.
- It syncs the emitters' clocks, schedules and order, the trigger cursor and the seeds at the first sync frame (SYNCAT=1 avoids the boot's half frame).
- EMITDBG=1 prints both sides' clocks.

`emitters_launch_as_the_original` runs five places in levels 29, 1, 3 and 6: 156 launches, all the original's.

gameplay.md has "Emitters (class 10, 0x8004e0f4)". Levels 1, 4, 6 and 7 and objectives.md are updated.

**Still unknown:**
- The flights' marks (0x8008652c, drawn by 0x8008e72c / 0x8008e76c).
- What a flight meeting level 4's water calls (the splash 706's drops make).
- 706's flights' hit callback (+0x4c, its object's +0x5c).
- Which VSync count the think reads on the hardware: the oracle's frame-end count less one in npc_check, the count itself in the test harness.
- Seen beside the emitters and not theirs:
  - level 3's hinged door closing (sound 54, 0x8004f868) about 7 frames early;
  - level 3's character sound 51 (0x80084084);
  - a port-only sound (program 0, tone 5) some frames after a put;
  - level 5's and level 16's other launches near the emitters.
