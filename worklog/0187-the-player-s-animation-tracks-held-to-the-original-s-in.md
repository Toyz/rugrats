---
number: 187
title: The player's animation tracks held to the original's in every script
date: 2026-10-17
area: test
files: crates/rugrats/tests/hle.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/player.rs
started: 2026-10-17T04:21:00-06:00
took: 22m
---

# 187. The player's animation tracks held to the original's in every script

Every oracle script now also holds the player's animation tracks (+0xa8, four of 12 bytes: state, frame before, frame showing, time left, phase) to the original's on every frame.

- **The bump's state.** 0x80056e10 gives table state 28 when 0x800590d4 says the body is running, else 32. Running means speed above 3933/4096 of the character's top speed (D_800c4200; Spike D_800c4210, Reptar D_800c421c), with +0x1b0 set. The port asked 32 always. The animator now has `running`, set as an action is asked from the speed then (the bump keeps its speed for that frame), and `table_state` takes it.
- **The oracle's boot.** Before a script's first press the oracle stood mid-fidget (state 24) from its long boot, while the port stood in the idle, so the records could not be compared until the first ask. The harness now gives the port's animator the original's state at open and at every `put`: the four track records, the speed (+0xe4), the states after (+0xe6, +0xe7), the fidget (+0xf0, +0xf2) and each part's kept quaternion, through `Animator::load_record`. From there both play on alike.
- `Both::script_with` counts the frames whose records differ. Each of the 11 scripts asserts none.

With 184's height and 181's turns, the eleven scripts now hold the player's x, y (root included) and z, heading, state, camera heading, animation tracks and, where they carry or throw, the thing's every place. The pick-up script also holds every part's quaternion.

**Still unknown:**
- +0x1b0 (part of 0x800590d4's test) is not read by the port.
- The NPCs' animators still ask before they step, and no script compares an NPC's records.
