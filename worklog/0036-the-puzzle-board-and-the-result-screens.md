---
number: 36
title: The puzzle board and the result screens
date: 2026-10-15
area: ui
files: crates/rugrats/src/screens.rs, crates/rugrats/src/main.rs, crates/rugrats/src/game.rs, crates/rugrats/src/golf.rs, crates/rugrats/src/hud.rs, crates/rugrats/src/progress.rs, crates/rugrats/examples/screen_shot.rs, docs/engine/screens.md
started: 2026-10-15T04:26:46-06:00
took: 33m
---

# 36. The puzzle board and the result screens

A sixth delegated map, docs/engine/screens.md, traced the screens that
run with the level frozen and checked them in the oracle with
screenshots. The port now shows them.

**The message pages** (the message loop 0x80093048):

- The front end's backdrop with a fade every 30 frames.
- The burst (BURST2.CMD, file 240, a MOD model) at the origin. The
  camera flies from z -17048 to -500 at 600 a frame; the lines show from
  the arrival. Cross flies it back out.
- Three centred lines at y -65, -40, -15 in font 1, with shadows.
  PRESS X TO CONTINUE in font 2 with its cube turning to the cross face.
- Kinds:
  - "Congratulations! You earned a puzzle piece!", with the lowest
    earned, unplaced piece blinking at (0, 45).
  - "Help Tommy win twelve puzzle pieces" after Reptar Puzzle.
  - "Great Job! You have earned n Reptar bars!" with icon 291 and the
    bars available.
  - "You did it!" with twelve placed; the golf over-par box.
- Icon 291 differs per database and is empty in MENU. Its texels and
  CLUT are copied from the VRAM the level left.

**The board** (0x80092c68):

- The board 350, the twelve pieces: 314 + i in colour, 326 + i the
  shadow drawn under, 338 + i grey. Tray and slot tables at 0x800bb32c
  and 0x800bb38c.
- 35 cover tiles (351) open one a frame in the order the game's rand
  makes, and close the same way after the wait.
- Select: any direction to the next earned piece, Cross picks. Place:
  Left/Right 7 a frame, Up/Down 3. Cross within d2 <= 100 (x scaled
  320 / 512) snaps it and writes +0x46; farther sends it back.
- "Terrific!" for 150 dt when all earned are placed. Twelve placed is
  Reptar 2010.

**Where they show**:

- After a finished level: the bars, then the piece and the board.
- Mid-level when 15 bars buy a piece; after Reptar Puzzle.
- From the board object in Pickles' Home (template 581).
- The golf box. Select's "Restart Level?": Yes restarts the level (golf
  from Hole One), checkpoint cleared.

Checked in pictures from screen_shot: the burst with the blinking piece,
the board with a piece placed and "Terrific!", the bar page with
Reptar's icon, the restart menu.

**Still unknown:**

- The loading screen between levels (the line table is in the doc) and
  the "already earned" notes at a level's start.
- The "Returning to Main Menus" page after Quit Game.
- Whether 326 + i is really a shadow; the speech under the screens.
- Select in activity play is not ignored yet.
