---
number: 162
title: No room to put it down; the put-down's own arms
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T13:16:55-06:00
took: 18m
---

# 162. No room to put it down; the put-down's own arms

Worklog 160 left the put-down's room test (0x80063888) unported, its
nearby list having peeked empty, and the drop's landing off by a hand.

**The put-down's room test, on the oracle.** The nearby list is built and
cleared within a frame (0x800452f8: 0x80062ed4 ... 0x800631a4), so a peek
between frames sees it empty; it is there for the test. It is filled from
the room lists of the player's room and its portals' rooms, and a poke
of an object's place does not move it between them: worklog 160's box LP
198, poked beside Tommy, was still in its own room 6's list, not the
player's room 9, so neither the put-down test nor the drop could meet it.
That run's "a box put down by another does not stack" showed nothing.
Linked in properly on the oracle (its +0x3b set to 9, then the loader's
own 0x8004a440 injected for it): 198 at z 13100 or 13400 with Tommy at
12263 holding 197, Circle does nothing (state 0, still carried); at 13500
he puts it down, 197 resting at (42731, -1280, 12959) short of 198. The
edge is 198's half-depth 256 + the held box's half-width 450 (its larger
of half-width and half-depth) from the held box's middle, z 12744: 706,
between 656 (refused) and 756 (put down). So a put-down never stacks:
one that would overlap is refused.

`Game::room_to_put` holds 0x80063888: the carried thing's first model
box's middle, where it is held, against every body's model box in its
frame grown by that half-extent (a character's by its height); the pie is
never put down (0x8005d11c). Test:
`a_box_held_into_another_is_not_put_down_as_the_original`.

**The arms of the put-down.** The put-down (logical 14, table state 16)
and the pick-up (15) both list animations [138, 139, 137, 136] for tracks
0-3; the pick-up's 2048 (frame 755) is on track 1's 139. Carrying, track
1 plays the carry's pose (state 19, over the base), which loops; the
port kept it there through the put-down, so the box rode a still hand
and no 2048 came (it let go at the animation's end, 600 ahead and 127 to
the side). In the original, each frame 0x80056708 answers 1 for a base
state whose word (0x800b9c30) has 64, state 16 alone, and track 1 then
follows the base (0x800561b0). The animator now does so; the box swings
back, comes forward and down to y -1209 and in to the middle, and is let
go at frame 755's 2048, as the oracle's (per frame against its: 12653 /
12651, 12862 / 12859, 12895 / 12891 in z; -1209 / -1211 at the bottom).
`a_box_put_down_comes_down_in_front_as_the_original` holds the landing.

**Still unknown:**
- The drop's slide: the oracle's box goes on 67 along the floor after it
  lands (17, 14, 12 a frame); the port's rests where it lands (12912).
- Thrown things against other things at rest (a thrown box onto a box).
