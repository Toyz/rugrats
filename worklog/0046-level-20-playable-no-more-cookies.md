---
number: 46
title: Level 20 playable: No More Cookies
date: 2026-10-15
area: play
files: crates/rugrats/src/level20.rs, crates/rugrats/src/game.rs, docs/levels/level-20.md
started: 2026-10-15T10:36:22-06:00
took: 18m
---

# 46. Level 20 playable: No More Cookies

A delegated map, docs/levels/level-20.md, traced No More Cookies and
checked it in the oracle. Two pointers in the brief were wrong:

- The target picker (0x80059b34) is not off in levels 20-21. There it
  takes only the four babies.
- The hand swap (template 886) is level 21's.

The port now plays the level.

**Level 20** (`rugrats::level20`):

- The round (0x80079cb0): Tommy holds the cookie box (template 442, kept
  in hand) with 15 cookies. The babies wait at home; Angelica follows.
- Circle or Square standing eats one: animation 42, sound 123. At 0,
  cine 64, won (3 bars).
- Triangle with a baby in front lobs the box at it. The target is the
  nearest within 512 of the heading. The port's lob is aimed to land on
  the baby; the original's speed comes from the distance (0x80057bc4).
- The box within 1400 of the baby (x and z) is the catch (0x80057518).
  The player becomes that baby (0x80057570): the bodies, places and ids
  trade, and the box goes to the new holder.
- Angelica's lunge (sound 121) within reach of the holder, or of a box
  on the floor, loses the round:
  - no cry, a try;
  - cine 65;
  - card 709 and the round again, or the hub with none left.
- GRAB never puts the box down (levels 5, 20, 21; 0x8005d11c).

Checked in scripted runs:

- Two cookies eaten.
- A lob from Tommy, and the player becoming Phil (id 608).
- Angelica reaching a standing Tommy, the knock at frame 69, cine 65,
  one try left, the round again.

**Still unknown:**

- The lob's true speed (0x80057bc4's range formula) and the arrow sprite
  (0x80031a28; a small marker is drawn instead).
- The kitchen's fixed camera with the box as subject.
