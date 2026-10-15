---
number: 56
title: Doors: hinged and sliding
date: 2026-10-15
area: play
files: crates/rugrats/src/game.rs crates/rugrats/src/level12.rs crates/rugrats/examples/play_script.rs
started: 2026-10-15T15:09:23-06:00
took: 26m
---

# 56. Doors: hinged and sliding

Class 3 and class 4 doors are now ported, from docs/engine/doors.md (traced and checked in the oracle). Before, they were fixed scenery boxes, so the hub's yard (Spike, the toys, a bar) could not be reached.

- **Game::doors:**
  - A Door is created for every class 3/4 placement with a model, except 843 (level 5's own doors) and 770 (golf's automatic door).
  - Doors are taken out of the scenery and the fixed boxes; their boxes and triangles are rebuilt each frame from Door::pose.
  - A hinged door turns: the placed matrix times a turn by +0xa8 about the origin.
  - A sliding door moves: its position is the start minus the placed matrix times the offset.
- **Use:**
  - The door is the one Tommy touches: its box covers him within RADIUS + 150, with height overlap. This replaces the original's touched-object pointer.
  - A use is refused while the door moves.
  - Hinged doors follow the facing rule: cos(h_p - h_d). One-way doors open only when Tommy faces against them; either-way doors (218, 1194) swing away from his facing. Closing needs him to face from the side the door swung to.
- **Thinks:**
  - Hinged: step trunc(speed x 136 / 4096). It stops on passing the target and snaps to it; closing to 0 stops on a sign change.
  - Sliding: 2 x step, stopping on the sum test with no snap.
  - The playpen gate's endless 1249/1250 comes out the same, as in the oracle: it stays open and refuses further uses.
- **Blocking:** if the next pose's box would cover Tommy or an NPC, the door does not move that frame. Level 16 shows message 24.
- **Sounds:** 14 open and 15 close (170 in level 16); 54 at the end of a hinged close (not 1216/672); 16/17 for sliding doors.
- **Level 12:** the refrigerator doors' use is turned off, so the cine stays the use.
- **Checked:**
  - Back door 337: Tommy is stopped at z 23020 while it is shut (oracle 23134). Triangle opens it, and he walks on into the yard (z 32030).
  - The gate opens only when faced against, then keeps moving (the quirk).
  - Yard door 218 opens in about 53 frames facing it. Facing away, it is blocked by Tommy.
  - No panics in any level. Doors found: DB5 2, DB10 4, DB12 8, DB13 7, DB15 8, DB16 4, hub 6.

**Still unknown:**
- The blocking test is a box against the body (radius 160/200), not the original's 12 edges; pickables are not tested.
- DB15's tilted doors turn about the placement's own y axis (guess; the doc leaves it open).
- Class 8 swinging doors (DB2, DB3, DB6), golf's automatic door 770 and the Triangle use animation (state 12 with its event) are not ported.
