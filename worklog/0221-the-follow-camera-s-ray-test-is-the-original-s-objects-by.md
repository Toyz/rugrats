---
number: 221
title: The follow camera's ray test is the original's: objects by their exact test, a start inside a box meeting it; the wall push's divide exact; the shelf's last camera difference is a stale stack normal
date: 2026-10-18
area: engine
files: crates/rugrats/src/world/game/camera.rs
started: 2026-10-18T07:18:31-06:00
took: 41m
---

# 221. The follow camera's ray test is the original's: objects by their exact test, a start inside a box meeting it; the wall push's divide exact; the shelf's last camera difference is a stale stack normal

The follow camera's ray test is the original's: objects by their exact test, a start inside a box meeting it, characters and flags 16 and 8 passed over, the world kept only when nearer on every axis. The wall push's divide is exact. Traced to its end on level 3's shelf, the camera's last difference there is a normal the original takes from its uninitialised stack.

**The camera's ray (0x80090554).** The follow camera tested its rays against the objects' float boxes (`Box3::segment_hit`), where a ray starting inside a box meets nothing. The original's line test runs as follows:
- It goes through the objects in the rooms along the segment (0x80090340 -> 0x8006a9cc with 0x8000), passing over template flags 16 and 8 and characters.
- It tests each by its exact boxes (0x8006a714 -> 0x80069c40 -> 0x80068e68), where a start inside a box meets it at the start.
- Then the world's segment (0x8002cd7c) is kept over the object only when nearer on all three axes.

`Game::blocked` now does that over the frame's solids (`Solid::meet`), as the characters' casts already did. On level 3's shelf Tommy stands inside the shelf's box (template 156, y -1761..7). The camera's ray up from him (the eye's height goal, 0x80027e80) now meets it as the original's does, where the port's missed. `Game::camera_ray` gives the test to tools.

**The wall push's divide (0x800263c4).** A wall aside pushes by (2048 - near 2048/768) << 16 times 0x4a90be59, high word >> 19. The port divided by 1800000; it now does the original's multiply.

**What is left on the shelf: a stale normal.** The camera's state now matches the original's on the run's first frame, flags and all, but for one push.
- The view test (0x80026914) asks the line test from Tommy, inside the box, to the eye. The box is met at the start, so its normal is never written.
- The original pushes 18 along whatever its buffer held. That buffer is a stack slot of 0x8006a9cc and 0x8006a714 that earlier calls left. It was turned by the object's matrix (0x80010c14) and set to length 18 by 0x80076944, whose squares overflow on such values: (-16, 6) here.
- The port has no stack to read and pushes nothing.

Traced with the original's accel at each call (0x80028080: 0, then 23 after the walls as the port's, then 7, 6 after the view test).

**Still unknown:**
- The view test's push from inside an object's box (the stale stack normal): the follow camera on level 3's shelf parts by a few units from the run's first frame, so the crouch over an edge stays uncompared there.
- Levels 20 and 21's Angelica through her modes 4, 8 and the lunge.
- Level 5's robot 17 after the original's lag frame: the sweep tool's pairing.
