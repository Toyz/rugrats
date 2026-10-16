---
number: 134
title: Climbs move by their root keys
date: 2026-10-16
area: play
files: crates/rugrats-data/src/anim.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/player.rs
started: 2026-10-16T01:44:15-06:00
took: 5m
---

# 134. Climbs move by their root keys

The ANM key with bit 29 (`Key::Lift`) is the root's move over its frame,
not a lift alone. 0x800557ec reads three fields:

- x: w0's bits 0-13;
- y: bits 14-27;
- z: w1's bits 0-13.

0x80055904 takes the frame's key in proportion to the time run. 0x80056834
adds the result into +0xda (y, always) and +0xd8 / +0xdc (x, z, for states
whose ENDS word has 32). 0x80066e60 turns that vector by the body's matrix
and moves the body by it (0x80066014). For bodies other than character
0x800c5680 (1, Tommy) the move is scaled by hierarchy +0x4c over
0x800c41dc. 0x800c41dc is -561, peeked in the hub.

The climbs' keys add up to their rises. For Tommy, by the climb picked:

| climb band | y | z (forward) |
|---|---|---|
| under 300 | -345 | 308 |
| 500 | -511 | 371 |
| 700 | -768 | 401 |
| 900 | -904 | 403 |
| 1030 | -1013 | 470 |

So in the original the body starts at the foot of the ledge and is
carried up and on to it by the animation.

The port puts the body on top at once. It used to play the climb there,
hovering over the ledge. The physics stay as they are. Now
`Animator::root_progress` gives the base track's root motion so far, and
its whole, and `Player::drawn_at` draws the body, while the climb plays:

- back along the heading by the forward move still to come;
- down by the part of the rise still to come, in proportion to the
  animation's own y.

`parts` uses the same place, so a carried thing follows.

Checked in level 6's box setup (play test level_6_box_stack): the drawn
root goes -2402, -2441, -2596, -2733, -2787, -2818, -2860 over the climb.
The physics y is -2914 throughout.

**Still unknown:**

- The last frame is cut short when the action ends, so the body pops
  about 50 into place.
- Not compared with the original's picture of a climb.
- The +0x4c / 0x800c41dc scale for the other bodies is not applied.
  Tommy's root offset in the port is -245, which does not fit the -561
  ratio; it needs a look.
- Other states with root keys (jump 3, land 4, fall 15, moving jump 28)
  add their y in the original. Their part (crouches) is not drawn.
