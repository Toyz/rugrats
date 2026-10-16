---
number: 166
title: The Load Game pages' button cubes
date: 2026-10-16
area: ui
files: crates/rugrats/src/ui/front.rs, crates/rugrats/src/ui/card_menu.rs
started: 2026-10-16T15:47:05-06:00
took: 6m
---

# 166. The Load Game pages' button cubes

The Load Game pages now show their button cubes. 0x800398bc makes three
cube objects when it opens (0x8003f444: CANCEL's 0x800c4b14, ACCEPT's
0x800c4b18, PRESS X's 0x800c4b40) and each frame draws either PRESS X TO
CONTINUE with its cube (0x80035798: at (-60, 414, 1298), turning to a
quarter turn at 128 a frame) or CANCEL and ACCEPT with theirs
(0x80035550, 0x80035674: (-600 or 600, -400, 1298), to a half turn and a
quarter), by the second camera (0x80040010); 0x8003f934 steps a cube as it
draws it. The port's front end drew only the texts, so the line read
"PRESS   TO CONTINUE" with nothing in the gap. `Title` keeps the page's
cubes (`card_cubes`, new at each opening) and draws the shown ones as it
draws the title's (`draw_cube`); `CardMenu::footer` says which.

**Still unknown:**
- The in-game pause's card pages (the same 0x800398bc) still draw no
  cubes: the game's stage has no second camera for them yet.
