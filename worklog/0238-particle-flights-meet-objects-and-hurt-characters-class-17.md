---
number: 238
title: Particle flights meet objects and hurt characters; class 17's enter sound
date: 2026-10-22
area: play
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, docs/engine/gameplay.md, docs/levels/level-06.md
started: 2026-10-22T22:06:28-06:00
took: 40m
---

# 238. Particle flights meet objects and hurt characters; class 17's enter sound

The particle flights now meet objects and characters as the thrown things do. Level 6's flying toys hurt the player where the original's do, and level 6's class 17 toys sound as the player comes near.

**Objects** (0x80086340). The class 10 callbacks launch flights of flag 0x20000 (in their 0xb0000 and 0xf0000 masks). The original tests those against the objects of the flight's room as well as the world, through 0x80086668. The port's flights tested the world only:
- the 905 toys' models, which carry damage 35 and flag 0x80000, flew through the player;
- the drops passed through the furniture.

Now:
- **The test.** `Flights::step` takes an object test, built from `things::meets`, the same 0x80086668 the thrown things fly by (objects first, the world kept over them only when nearer on all three axes; the boxes grown by the radius with flag 0x40000000). The stop test's floor goes through it too.
- **Damage.** What a flight meets comes back in `Flights::struck`. With flag 0x80000 a character takes the flight's damage (+0x8e, 0x800780ac): the player through `post_damage`, a character through `post_npc_damage`.
- **Bounce and marks.** It bounces off an object as off the world. Only world hits leave marks.

Checked: standing at (-56493, -2440, -3042) in level 6, 905's models hit the player for 35 at frames 35, 62, 89 and 116 on both sides; the flights are all the original's.

**Class 17** (0x80049d98, level 6's 79, 80 and 81 toys): its enter (0x8004d174) plays the template's +0x24 sound at it. The port's trigger enter had done nothing, so level 6 was missing sound 81 there.

**Still unknown:**
- A flight's object touch (flag 0x40000 calls the object's +0x58) and its stick (+0x98 = the object met): not done for the particles.
