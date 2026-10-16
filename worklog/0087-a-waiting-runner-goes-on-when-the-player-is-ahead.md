---
number: 87
title: A waiting runner goes on when the player is ahead
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/npc.rs crates/rugrats/tests/play.rs
resolves: 49.1, 55.1
started: 2026-10-15T21:20:50-06:00
took: 2m
---

# 87. A waiting runner goes on when the player is ahead

Mode 7, a route walker waiting at a flag 2 post (0x800843f0), resumes (0x80083efc) on any of three rules (docs/levels/level-13.md, The waits). The port had only the first two:

1. 400 VSyncs have passed (+0x68e);
2. the player is within +0x68c (2000);
3. the bearing to the player is within 768 of the bearing to her next post (0x800760f4): he is ahead of her. Seen in the oracle: Angelica at post 34 leaving after 2 VSyncs with speech 611.

- **Rule 3 added** to npc's Mode::Wait.
- **The lines move:** 611/612 (level 13's Resumes event) are now said for rule 3 only, as 0x800b96e0 is used. Before, the port said them when the player came near.
- **Cookie Race:** Angelica no longer dawdles at a wait when Tommy is past her.
- **Level 16's pie toss:** Angelica circles her ring and Tommy throws from the booth, so he is often ahead; she leaves her waits sooner, as the original's generic mode 7 does. The toss test now checks the win itself, Angelica stopped with her health back when the clock goes off, not that it ends before frame 1000. It is still won.

**Still unknown:**
- nothing
