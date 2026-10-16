---
number: 160
title: GRAB reaches as the original; a box put down does not stack
date: 2026-10-16
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/tests/play.rs
started: 2026-10-16T11:10:49-06:00
took: 47m
---

# 160. GRAB reaches as the original; a box put down does not stack

Box stacking, worklog 158's last unchecked item, run on the oracle in
level 6 as Tommy (no character hook; the level then plays as him).

**A box put down by another does not stack.** Box LP 198's object
(0x800f7dc4, the one after 197's at 0x800f7d1c) moved to (42741, -1280,
14263), 1400 beyond box 197; Tommy picks up 197 and, from z 13100, 13400
and 13700 facing 198, puts it down. Each time 197 comes down on the floor
(y -1280) overlapping 198, at z 13796, 14219, 14160 (Tommy at 13700 was
held at 13601 by 198). The port does the same: its drop flight meets
`solid`, which leaves out things at rest.

**GRAB reaches as the original's (0x800631d8).** On the oracle Tommy
takes box 197 from 600, 800, 1000, 1200, 1320 and 1324 straight in front,
not from 1330, 1450 or more; from 1000 with the box 512 or 768 off his
heading, not 1024 off; never from behind (400 and 600). The code: the
objects of the nearby list (0x80062ed4: his cylinder, 3 x the larger of
his body box's half-width and depth across, 3 x its half-height tall,
meeting each object's, radius +0x86 = its farthest model box corner,
577 for the box: 0x8006abbc), 511 below to 4095 above him, within his
box's depth + 1800 (D_800c4308), strictly within a quarter turn of his
heading (D_800c430a = 1024), the nearest, not the one he stands on. 750 +
577 = 1327 is the edge seen. The port took the nearest within 520 any way
faced; `Game::touched` now holds the original's rule
(`Player::body_box` gives the header's box). `grab_reaches_as_the_original`
holds the six oracle cases.

**The put-down lets go at its animation's event.** State 8 (0x80060910)
waits for +0xe0 bit 2048 before the drop (0x80062ac4), the box carried
out in front by then (the oracle's land in line with Tommy, 560 to 820
ahead). The port dropped it at the press, from the hand; now
`PendingThrow::Down` lets go at the event, as a throw does at its
release. `a_box_put_down_by_another_does_not_stack_as_the_original`
holds the floor landing ahead.

The put-down's room test (0x80063888: the carried thing's box centre
against the nearby objects' model boxes grown by its larger half-width)
runs over the nearby list, which the oracle showed empty (+0x100 = 0)
while standing: no refusal was seen. The port has no such test.

Level 6's walk faces each box before Circle, from 900 off (nearer on a
shelf without the room), now that a box behind is not taken.

**Still unknown:**
- The drop's landing exactly: the port's come down 600 ahead, the
  oracle's 560 to 820 (the put-down animation's hand path).
- When the nearby list is filled (empty while standing in the oracle),
  and so when 0x80063888 can refuse a put-down.
