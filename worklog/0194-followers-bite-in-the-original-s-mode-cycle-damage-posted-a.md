---
number: 194
title: Followers bite in the original's mode cycle; damage posted a frame late
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/camera.rs, crates/rugrats/src/levels/mr_friend.rs, crates/rugrats/tests/hle.rs, crates/rugrats/examples/npc_check.rs
started: 2026-10-17T07:35:12-06:00
took: 36m
---

# 194. Followers bite in the original's mode cycle; damage posted a frame late

The followers' bite is now the original's mode cycle, and damage to the player arrives a frame late, as the original posts it. Mr. Friend's bites in level 9 match the original on every frame of 120: his animation tracks, Tommy's tracks, place, health and camera.

**The bite (0x80084084, modes 4, 5, 1, 0).**
- **Mode 4 (0x80084cd0).** Within half the range it stops. That needs the way to the target clear (0x8002cd7c) and the two bodies level (0x80083ff0: the hierarchy boxes' heights overlapping, each from its +0x1c). With +0x688 bit 8 it goes to mode 5. The speed is then twice the table's past twice the range, the table's past half of it, or as it was.
- **Mode 5 (0x800846e4).** State 19. The bite (logical 37, table state 49) is asked at 4096 and the Damage posted, then mode 1. There is no body update that frame (0x80084084 leaves without 0x80082230).
- **Mode 1 (0x800841cc).** The bite plays out with the body updated; it still turns toward where it aimed. When the base completes (+0xe0 0x8000000) it is mode 0 with its speed back. The wait on event 0x2000 never came in these bites.
- **Mode 0.** Mode 4 again, again without a body update. The state set back to 0 by mode 1 is entered anew at the next update (0x8005b2d8), so the stand is asked afresh (phase 0). The port's `Npc::reenter` starts it again rather than leaving it be.

The port had its own rule: a bite within range / 2 every `bite_wait`, with a 1200 height window. `Npc::bite` (`Bite::Chase`, `Strike`, `Recover`, `Resume`) now holds the cycle. The NPC loop skips the animation step on the two frames without a body update.

**Damage (0x800780ac).** Damage is posted as event 8 on D_800c4408 and taken at the next frame's start (0x8007871c, before the player's update). Pickles' Home, the circus, Gold Rush and golf take none. `Game::post_damage` queues it. The eight levels' bite handlers now post rather than hurt at once, which put Tommy's health a frame ahead before.

A note on the 225-frame "waits" seen earlier: they were Tommy's death after the bites. The original then updates no one.

**Oracle.**
- `level_9_friend_bites_as_the_original` gives Mr. Friend the original's mode and animation once past cine 17, and gives Tommy his animation and the camera's look-at (`Game::set_camera_look`). It then plays 120 frames and asserts Friend's tracks, Tommy's health, tracks, place and camera.
- `level_9_friend_chases_as_the_original` now runs 520 frames, and Chuckie's box script asserts the camera after R1. All 14 scripts hold the camera heading.
- npc_check gains CALLS (the original's update, body and animation calls for one object), and its MOVES line shows the mode, the wait (+0x69c), the events (+0xe0) and the base track.

**Still unknown:**
- Mode 1's wait on event 0x2000 (300 + (rand & 3) x 300 dt with no body update): which animations carry 0x2000, and whether `Rand` is 0x8002fb7c's.
- Mode 5's sounds by template (495, 882, characters 41 and 43), still left to the levels.
- The other player damage that goes through 0x800780ac (a thing run into, a flight, the slip and fall) still hurts at once.
- The lunge (mode 8 within the range with +0x688 bit 128).
