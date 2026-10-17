---
number: 188
title: The NPCs step before they ask; Spike held to the original
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats/src/actors/animator.rs
started: 2026-10-17T04:43:31-06:00
took: 7m
---

# 188. The NPCs step before they ask; Spike held to the original

The NPCs now take the original's order as the player does: the animation steps first, its root curve with it, and then the mode asks for what plays from the next step (0x8005a068 runs every character the same way). The play tests all still pass.

`examples/npc_check.rs LEVEL [FRAMES]` boots both. It matches each port NPC to the original's object at its place in the rooms' lists (0x800c4a8c, +0x48 and +0x4c) and gives the port's animator the original's state (`Animator::load_record`, then `mark_asked` with the held animation). It then plays both with no input and prints where an NPC's place, heading or tracks differ.

In level 1 from the oracle's boot, Spike (id 14, held in animation 26, table states 39 and 40) matches the original's place, heading and animation tracks on all 60 frames. The other objects the port has as NPCs there (ids 16, 15, 600, 1776, 13, 609, 608) hold no animation state in the original: their track records are whatever the memory held, so the original has never stepped them. Their rooms have not been active since the level began.

**Still unknown:**
- The original steps a character only while its room is active (its record shows nothing has run). The port steps every NPC from the level's start, so one met later may be at a different point of its animation.
- The NPCs' own modes (routes, bites, swats) are not compared with the oracle; no scene was set up for them.
