---
number: 204
title: A first frame begun again takes its lift's gap anew, characters of template flag 0x10000000 are put unsettled; settled unknowns out of the docs
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/grandpas_teeth.rs, docs/levels/level-03.md, docs/levels/level-13.md, docs/levels/level-15.md, docs/engine/objectives.md, docs/engine/gameplay.md
started: 2026-10-17T17:41:00-06:00
took: 11m
---

# 204. A first frame begun again takes its lift's gap anew, characters of template flag 0x10000000 are put unsettled; settled unknowns out of the docs

Two engine rules from 0x80055904 and 0x800452f8, and the docs' unknowns that the last worklogs settled.

**A first frame begun again takes its gap anew.** At the start of a state's first frame, 0x80055904 sets +0xde to the frame's lift key less where the body is now over its floor and feet (0x80055b3c). Across that frame the body moves by +0xde times the part of the frame done. A state asked again every frame begins its first frame every frame (level 20's babies, whose stand re-asks their idle, worklog 198), and so closes the gap a part at a time. The port took its blend's base only when the state changed or looped, so a body re-begun every frame stayed where the first blend left it (level 20's Phil drawn at a lift of 16, the original's 37). `RootCurve::step` now takes the base again whenever the base track begins its first frame. The babies' lift now goes 16, 25, 30, 33, 35, 36 and stays: the original's way, truncated from below where the original's came from above (37).

**A character put where it is not settled.** 0x800452f8 places an object somewhere. For a character with +0x688 bit 0x2000 (its template's flag 0x10000000, 0x80081d64: level 4's rats, level 15's Spike) it skips the room and floor (0x80045350). `Npc::unsettled`, kept by `Npc::place`.

**Docs.** Settled unknowns moved out of the lists:
- level 3: template flag 0x2000 (the posed parts), +0x688 bits 0x400 (never climbs) and 0x1000 (a held probe lets by);
- level 13: bit 0x1000;
- level 15: Spike's bit 0x2000;
- objectives: +0xe0 bits 16 and 2048 (frame events), and the flight's object meet;
- gameplay: +0x130 (the slant: the climb under 384, the slide to 1024).

**Still unknown:**
- Where level 4's rats and level 15's Spike are put (the placement that would show `unsettled` in play) is not compared against the original.
- The convergence point of a re-begun first frame depends on where the body came from; the oracle's babies (37) came from above, the port's from the sync (16).
