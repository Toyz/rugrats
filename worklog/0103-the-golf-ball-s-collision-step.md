---
number: 103
title: The golf ball's collision step
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/golf.rs crates/rugrats/src/world/collide.rs crates/rugrats/tests/play.rs
started: 2026-10-15T22:50:51-06:00
took: 5m
---

# 103. The golf ball's collision step

With the segment test ported (99), the golf ball's collision step
(0x80086ef0, docs/levels/golf.md "Collision step") comes across as
documented, in place of the per-axis stand-in. Each frame:

- p' = p + (v dt) >> 6.
- The leading edge e = p' + unit(v) R >> 12, with R = 104.
- The segment from the old position, bent along the floor's plane
  there: e = s + P d >> 12. P is 0x800763d0(-v, n), the direction along
  the plane; d is 0x80075f58, 16384 when any axis is that far.
- The segment test on it. A stop (1) plays 137 and bounces off the
  normal (0x8007666c): with c = N . unit(v) < 0, r = u - (c N >> 11);
  p' = H - u R >> 12; v = r (|v| 3000 >> 12) >> 12.
- Then the roll as before (0x80086cd0).

`collide` gains the vector routines it needs:

- `unit` (0x8007652c: VectorNormal after the 12-bit pre-shift);
- `length` (0x80076784);
- `fix` (x y >> 12 toward zero);
- `along_plane` (0x800763d0's direction);
- `floor_normal`.

`normalize` (0x8009f278) is public now.

The doc's one oracle sample, a wall at x 39900 turning (771, 0, 71)
into (-551, 0, 49): by hand, the step plus a friction step gives about
(-548, 0, 49). The old per-axis bounce gave (-549, 0, 69), so z now
agrees. The shot list for golf_course_is_played was searched again
(golf_course_search, 68 s). Every hole is played at or under par:
2, 2, 3, 2, 3, 2, 3, 4 and 1 strokes.

**Still unknown:**
- the 3 left in that sample's x (an order of friction and bounce, or the wall's normal); an oracle run at that wall would settle it
- objects along the segment (0x80086340) still use the port's boxes, with the old per-axis turn
