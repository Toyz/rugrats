---
number: 185
title: Root motion across for every state that has it
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/throw_check.rs
started: 2026-10-17T03:55:41-06:00
took: 18m
---

# 185. Root motion across for every state that has it

Root motion across is no longer the bump's alone. Any base state whose word has 32 moves the body by its root keys' x and z (0x80056834 adds them into +0xd8 and +0xdc, turned by the body's angles, 0x80075ee8), as long as the body is on the ground and not climbing (the climb carries itself). Chuckie's pick-up (state 15, word 33) steps him in and back; his put-down (16, word 97) steps him back. Tommy's root keys across are zero, so none of this showed in level 1.

- The move is turned by the integer turn (`Trig::turn_by`), no longer by the float rotation.
- When a state begins (`Animator::began`), the one that ran out in that step first gives what it still had, if it was showing its last frame. Chuckie's pick-up used to end 8 short.

`examples/throw_check.rs` now boots the oracle as the level's own player (level 6 is Chuckie's). It takes the floor at the player's own spot and can put the thing down instead of throwing it (PUTDOWN=1). An eleventh oracle script, `level_6_chuckie_box_as_the_original`, boots level 6 as Chuckie and picks up box 1016 off its shelf, carries it and puts it down. His places match the original's on every frame, and the box matches it on all 55 frames in his hands.

**Still unknown:**
- The put-down's release in that scene. The original builds no flight: neither 0x80061f88 nor 0x8008864c is called (hooked), and the box goes onto the shelf's edge at (42820, -1651, 12411). The port still lets it slide.
- The rolling flight (0x80086ef0) is read but not ported: the step projected onto the floor's plane (0x80086b58), a bounce without the floor stop, then 0x80086cd0's roll (friction 98 dt >> 6, pinned its radius above the floor, the slope's push from 0x80086a20: 1190 under a tilt of 768, then 99 a unit less). The oracle's level 6 box, booted as Tommy, slid 94, 79, 64, 49, 34, 19, 4 at a radius of 256.
