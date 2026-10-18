---
number: 212
title: Level 5's ambience proven, the guards' attack test the original's, and the unused hint cards settled
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/npc_check.rs, docs/levels/level-05.md, docs/levels/level-12.md
started: 2026-10-17T21:18:34-06:00
took: 44m
---

# 212. Level 5's ambience proven, the guards' attack test the original's, and the unused hint cards settled

Level 5's ambience proven in the oracle, the guards' attack test made the original's, and the unused hint cards settled.

**Level 5's ambience.** `level_5_ambience_as_the_original`: Angelica beside a robot the original holds still (+0x699 240), the rand and the countdown given the original's, 2000 frames standing. Every line comes on the original's frame: 729, 731, 684, 640, 663, 684, 641. With worklog 207's frame order and the level 12 run, the ambience is the original's in both levels where it was compared.

**The guard's attack test (mode 11, 0x80084084 at 0x80085220).** The port bit when the player came within reach by straight distance, with a height check of its own (1200, or reach + 600). It is now the original's:
- within reach by the game's distance (+0x674, 0x8001f57c);
- the way to the target clear;
- the two bodies level (0x80083ff0: the target's box heights, +0x154 + 4 for a character, against its own, at their heights with the lift). That is the test mode 4's chase already used.

A robot hovering 2233 over Angelica was refused by the old height check, and the original's bites her.

**Robots are never settled.** Template flag 0x10000000 (+0x688 0x2000, worklog 204): a robot's place stays at its LP height (-7824) and its hover (-8000 to -8600) is its lift. The tests' sync (`sync_npc`, `npc_check`) put every character on its floor plus its feet; it now keeps an unsettled one's own height.

**What the attack does next, seen and not yet ported.** Traced a frame at a time, a guarding robot 1200 from Angelica:
- it holds still 4 frames, turns 672 a frame until it faces her, and only then attacks;
- +0x688 bit 512 snaps the heading and asks animation 13 first (mode 1), then the bite (mode 5);
- the damage lands two frames after, mode 1 lasts 20 frames (track state 49), then mode 0 and mode 11;
- one attack every 23 frames.

The port's guard attacks on reaching, without the wait or the turn, its damage 3 frames early, and its idle is track state 0 where the original's is 63. A first port of the cycle (mode 4's Strike / Recover / Resume) came out slower than the original's and was set aside; the level 5 doc's unknowns have the trace.

**Hint cards never shown.** Every call of 0x80044434 passes a constant or shows the last card again (worklog 209). So level 5's HINT17-21 (688-692) and level 12's HINT33 (704), "no caller found" in the docs, are never shown.

**Level 11's runner in `npc_check`.** The tool syncs only with SYNCAT. With it, the runner after the sync is the original's frame before on every frame: the tool's timing, not the runner's. Bisected: the same at worklogs 199, 202 and 204.

**Still unknown:**
- The guard's turn and wait before an attack, its mode 1 (animation 13) and the bite cycle's 23 frames, ported (traced above).
- The robot's idle track state (63).
- Why `npc_check`'s sync leaves level 11's runner a frame behind.
