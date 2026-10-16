---
number: 104
title: Thrown things bounce; the can comes back
date: 2026-10-15
area: play
files: crates/rugrats/src/world/game/things.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/tests/play.rs
resolves: 40.1
started: 2026-10-15T22:55:29-06:00
took: 4m
---

# 104. Thrown things bounce; the can comes back

40.1 left the can bouncing back into Tommy. The oracle (level 9 doc,
"The flight") shows the can hit Mr. Friend (100 -> 70), bounce off him
and come back into Tommy (Damage(player, player, 30), 255 -> 225). The
port's flights stopped dead on any hit.

What the original does (0x80087594, the world path of flights without
0x10000000):

- An object or world hit reflects off the surface's normal
  (0x8007666c), unless the flight was moving away from it.
- The speed afterwards is (|v| - 95) * bounce >> 12. The bounce is the
  object's +0xa2, from template +0x36 clamped to 4095: the can 3200, the
  ball 2000, the pie 1500, most things 0.
- Slower than 95 + 382 with a floor under it (0x80087414), it stops.
- An object hit on this path damages any class 1 object, the player
  included (0x80087924), which is how the can hurts Tommy.

In the port (`things.rs`):

- `rebound` is the reflection and speed.
- A wall hit takes its normal from the segment test (99), or from the
  axis moved along when that does not stop.
- A body's normal is the face of its bounds cube the thing came at.
- Each thing remembers its last hit (record +0x98), so it does not
  strike the same body twice, and whether it bounced.
- A bounced thing of a template with +0x30 not 1 (not a ball) that
  reaches Tommy hurts him by its damage.
- Templates with no bounce drop as before.

The test `level_9_a_can_bounces_back` throws a can at Mr. Friend 1500
away. He loses 30, the can comes straight back, and Tommy loses 30, as
in the oracle.

**Still unknown:**
- the object test 0x8006a714 itself (the port's bounds-cube face is a reading of what it tests against)
- the spin rate's damping by the bounce (record +0x16)
