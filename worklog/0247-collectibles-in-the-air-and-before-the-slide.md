---
number: 247
title: Collectibles in the air, and before the slide
date: 2026-10-23
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/goodies.rs, crates/rugrats/tests/play.rs, docs/engine/objectives.md
started: 2026-10-23T06:37:12-06:00
took: 37m
---

# 247. Collectibles in the air, and before the slide

Collectibles in the air are now taken as the original's air move meets them, not within a radius. On the ground the collectible now comes before any slide along it.

**The air** (0x8005ee78, worklog 245's open item):
- The air move's test (0x80066014) meets the objects of the nearby list. When it answers anything but clear and +0xf4 is set, 0x8005ed2c runs.
- For an object of class 11 (or template 711, not 1177) that clears +0xf4 and +0xf8 and runs its +0x58 (level 11's standing jump keeping its bones, 0x800c4bc4). It answers 1, and the move is then had as if clear.
- The port's `air_move` meets the collectibles after the other objects (`touchable`, as the walk's probes do). Met, one is `touched` and the answer becomes Clear, across and down alike.
- `Game::touch` no longer has a radius test at all.

**The ground's order.**
- 0x80064a34 only answers held; the walk handler (0x8005e1f4) looks at +0xf4's class first. Class 11 is the touch, with no step. Only otherwise does it slide or bump (0x80065120 at 0x8005e3c4).
- The port had slid along a collectible met at a slant before looking, so the touch was lost and the player slid round it. `ground_move` now looks at the first probe's hold first.

**The play tests' collecting.**
- Where no walk led to a pickup, the harness puts the player on it. Its probes then start inside its box, as on the original.
- It now puts him 600 off on his own floor, the first of eight ways from the one he came, and he walks at it from there.
- Worklog 245's walk back had run off a ledge in level 15 and could not turn away in level 18.

**Checked:**
- every test file, and the hle suite;
- levels 15, 17, 18 and the whole game won by play.

**Still unknown:**
- An air touch has not been seen against the original, only taken from the code.
- Template 711 (counted with the collectibles by 0x8005ed2c) and 1177 (not) are not in the port's touchable list.
