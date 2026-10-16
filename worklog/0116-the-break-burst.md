---
number: 116
title: The break burst
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs crates/rugrats/src/levels/mirrorland.rs
started: 2026-10-15T23:59:49-06:00
took: 2m
---

# 116. The break burst

From 114's list of callers: use callback 2 (0x8004d434, level 15's
breakable hiding spots) throws a burst of five (0x80054894) as it hides
the object:

- kind 5, flags 0x10505 (gravity, the world collides);
- radius 300, half-size 240 x 300, 1000 ms;
- spin 3391, bounce 2048, face 0xc0800000;
- speed rand mod 400 from its own seed (0x800c41d0, 48732 in the
  executable);
- directions within 1024 of up.

The sprite comes from the object's room byte (+0x3b): 2 303, 5 300, 19
304, 21 299, 25 301, 45 302, otherwise entry 0.

`Flights::break_burst` builds it; mirrorland's opening of a spot calls
it next to the sound 114 (112).

**Still unknown:**
- the shards (0x8004cdfc, 100) that go with it: model pieces, a flight kind the port does not draw (kinds 1, 2, 4)
- the cone about the object's own matrix (+0x04): world up here
