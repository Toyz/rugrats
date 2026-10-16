---
number: 130
title: Hole 26's Reptar roars
date: 2026-10-16
area: audio
files: crates/rugrats/src/levels/golf.rs, crates/rugrats/tests/play.rs
resolves: 55.2
started: 2026-10-16T01:23:07-06:00
took: 3m
---

# 130. Hole 26's Reptar roars

Hole 26 (DB26) has a Reptar, id 9999 (template 8, hierarchy 9). The
hole's setup puts him in mode 8 (0x80083918) and schedules him.

Mode 8's handler (0x80084fc4 in 0x80084084) goes after the player with
+0x688 bit 64, or keeps to home with bit 2. With neither he stands, as
the port's `Mode::Idle` already has him. At its end (0x80085168) there is
level 26's own part. The countdown 0x800c4ce4, shared with the hub's
line timer, goes down by dt. At 0 it draws rand % 3:

- 0: sound 144 at him;
- 1: sound 145 at him;
- 2: line 1134.

Then the countdown goes back to 1000.

The golf script now does this while he is shown. Test:
hole_26_reptar_roars hears at least two noises in 250 frames.

**Resolved from earlier entries:** 55.2, the prowler's sounds 144/145
and line 1134.

**Still unknown:**

- The countdown's value when the hole starts; the port starts at 0.
- 0x800c4614's pending line (the other half of 55.2).
