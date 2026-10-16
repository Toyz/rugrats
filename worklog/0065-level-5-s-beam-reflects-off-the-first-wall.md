---
number: 65
title: Level 5's beam reflects off the first wall
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/outer_space.rs docs/levels/level-05.md
started: 2026-10-15T18:01:16-06:00
took: 2m
---

# 65. Level 5's beam reflects off the first wall

Angelica's zap in level 5 now reflects off the first wall it hits, with sound 70, and stops at the second, as 0x80090554 / 0x8007666c do (docs/levels/level-05.md). Before, it stopped at the first wall.

- **0x8007666c** mirrors the direction about the wall's normal: dir - 2 (n . dir) n when the beam moves into the wall. It reflects when the returned angle is 1025 or more, so nearly every hit.
- **The port** takes the normal from which axis of the step the COL blocks:
  - x blocked: the x direction flips;
  - z blocked: the z direction flips;
  - both: it turns back.
- **Checked** with play_script in DB05 from the start, zapping in the four headings: each beam turns at the first wall (for example dir (0, 4096) to (0, -4096)) and ends at the next.

**Still unknown:**
- The wall's true normal. The original tests the room's polygons (0x80090554), not the COL grid. A slanted wall would reflect differently.
- The reflection's effects 0x80090acc and 0x80090c0c are not drawn.
