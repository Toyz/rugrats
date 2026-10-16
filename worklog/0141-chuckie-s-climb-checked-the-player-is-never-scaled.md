---
number: 141
title: Chuckie's climb checked: the player is never scaled
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T02:18:26-06:00
took: 7m
---

# 141. Chuckie's climb checked: the player is never scaled

**Booting the oracle as another character.** The probe harness booted
every level with Tommy. The main loop sets the player's character
(progress +0x08, 0x800c5680) from fn_80018598(level) before the probe's
level hook changes the level. So 0x800c5680 stayed 1.

The fix is a second hook, `--onhook 0x800454dc:0x800c5680=2`, at the
placement loader that builds the player. With it, level 10 starts with
Chuckie at (5648, 670, 19228). 670 is his root offset (-354) over the
floor.

**What 0x800c5680 is.** It is the player's own character, so 0x80055904's
"not 0x800c5680's character" test leaves the player unscaled whoever he
is. Worklog 135 scaled Chuckie's and the others' climbs by +0x4c over
-561. That was wrong for the player; it now applies to nobody. The
scale belongs to the NPCs' root motion, which the port does not draw.

**Checked against the original.** In level 10, Chuckie was put at (8945,
670, 12500) by the same seat (template 130) and walked into it. The
oracle climbs in table state 13:

661, 652, 639, 622, 605, 502, 398, 282, 154, 25, 6, -14, -22, -17, -12,
-23, -34, -50, -72, -95, -114, -133, -152, -172, -193.

The port, unscaled: 660, 651, 638, 621, 605, 501, 397, 281, 153, 25, 5,
-15, -22, ... -153. All within 1.

The forward distance differs by up to about 80 in the middle of the
climb. Both bodies were put inside the seat, and the original pushes out
of it first.

A climb's leftover root y no longer carries on to the ground after it.
It had put Chuckie 110 too high on top.

Test: chuckies_climb_moves_as_the_original holds 23 frames within 2.

**Still unknown:**

- The climb's last two frames: the oracle reaches -172 and -193 before
  state 0, while the port ends the action at -153 and draws the body at
  the top.
- The NPCs' root motion (scaled by +0x4c / 0x800c41dc) is not drawn.
