---
number: 163
title: A put-down box slides on
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T13:35:12-06:00
took: 6m
---

# 163. A put-down box slides on

A put-down box slides on after it comes down. The oracle, level 6, Tommy
at (42741, 12263) facing +z puts box LP 197 down: let go at (42731,
-1211, 12892) at frame 4170 (VSyncs), on the floor at -1280 the next, and
on along it 17, 14, 12, 10, 7, 5, 2 a frame to rest at z 12959. The port
let it rest where it landed (12912).

A put-down thing now slides (`Thing::slides`, set at the put-down's
release): landing on a floor, it keeps its speed along it less 16 a frame
in the flight's units (64 to 10 a frame: 17 less 2.5), and rests once
that runs out; the port's comes to rest at 12960. Thrown things still
stop where they land (worklog 77).

**Still unknown:**
- The original's rule for the slide (which flight flag, which code); the
  loss here is fitted to the one oracle run.
- The put-down's release frame: the port lets go a frame before the
  oracle (its x 42772 to the oracle's 42731), reading a frame's events
  as it ends.
