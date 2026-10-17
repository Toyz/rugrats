---
number: 176
title: Letting go stops the turn; a hazard hurts once a contact
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T23:39:53-06:00
took: 13m
---

# 176. Letting go stops the turn; a hazard hurts once a contact

Two of the code's guesses replaced by what the original does, and an
eighth script played against it.

- **Letting go of the stick stops the turn** (0x8005c8d8): with the
  stick's magnitude under 70 the target is set to the heading, in every
  state. The port did so only while held in place (a guess there) and
  otherwise turned on to the last target. A tap of Up and Left from the
  hub's start: the original turns 2048 to 1888 and stays; the port went
  on to 1536. Now the same (`hub_tap_turn_as_the_original`). The tests'
  turning helper holds its gentle push until the heading arrives, as a
  player would.
- **A hazard hurts once a contact** (template flag 0x200000, its touch
  0x8004e0f4): touching sets the player's +0x188 bit 256 (0x80059570),
  through which no touch hurts, and leaving it (its untouch, 0x80049e5c)
  clears the bit. The port waited 300 dt between hurts (a guess). Damage
  itself (0x800780ac) only posts an event (none in levels 1, 11, 17 or
  the golf holes), and the event (0x80077864) takes the health but
  never while standing on a thing.

`tests/hle.rs`: eight scripts, all the same as the original's across on
every frame.

**Still unknown:**
- The touch's timed hurts of types 969 and 982 (within 2048 or 3000, on
  their own clock, 0x8004e0f4) and the objects' touch and untouch events
  generally are the port's own contact test.
