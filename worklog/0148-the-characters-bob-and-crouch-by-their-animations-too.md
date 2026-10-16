---
number: 148
title: The characters bob and crouch by their animations too
date: 2026-10-16
area: render
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/levels/grandpas_teeth.rs
started: 2026-10-16T02:41:48-06:00
took: 2m
---

# 148. The characters bob and crouch by their animations too

Worklogs 137, 138 and 147 gave the player's body the height its
animations' root keys put it at on the ground. The other characters run
the same code (0x80056834 into 0x80055904) and were drawn flat.

`RootCurve` (actors/animator.rs) is the rule of worklog 147 on its own:

- the base track's root curve so far;
- plus where the body was as the animation began, blended away over its
  first frame;
- a finished animation's rest goes into the next one's start.

Each `Npc` keeps one, stepped after its animator. It is drawn under the
character's position.

A body other than the player's (0x800c5680's character) moves by its
root offset (hierarchy +0x4c) over 0x800c41dc. That is -561, peeked in
levels 1, 4 and 6. The NPCs' curves are scaled so: a baby with a root
offset of -354 moves 0.63 of its keys.

All tests pass.

**Still unknown:**

- Not checked on the oracle. An NPC's object could not be told from the
  others without its address, and only the player's (0x800e2b48) is
  fixed.
- The player is still stepped by its own code (`Player::root_step`, with
  the jump's crouch and take-off), not by `RootCurve`.
