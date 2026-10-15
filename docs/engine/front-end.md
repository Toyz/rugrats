---
title: The front end
status: partial
discs: US
covers: US SLUS_006.50:0x80030158, US SLUS_006.50:0x8003eb28, US SLUS_006.50:0x8003d884, US SLUS_006.50:0x8003d5a0, US SLUS_006.50:0x80035ab8, US SLUS_006.50:0x80034fdc, US SLUS_006.50:0x8003528c, US SLUS_006.50:0x80034e80, US SLUS_006.50:0x8008b100, US SLUS_006.50:0x8002fb7c, US SLUS_006.50:0x80040194, US SLUS_006.50:0x8003f200, US SLUS_006.50:0x8003f7f0, US SLUS_006.50:0x8001dfb8, US SLUS_006.50:0x80020ee0
worklog: 18, 19
---

# The front end

## Main's chooser (0x80030158)

Main runs it with mode -1. It cycles: the front end (0x8003eb28(30)),
then a demo, then the opening cinematic (0x80074958(-1, 0)), and back. It
returns the mode the front end picks. A demo is movie 3, 4 or 5
(DEMO01-03.STR, 0x8008b100/14c/198), chosen by `rand() % 3`. `rand`
(0x8002fb7c) is x = x * 214013 + 2531011 from 0x2dcf1a5, returning bits
16-23.

## The front end (0x8003eb28)

It loads MENU.TEX (file 108) to VRAM (0, 256), 576 x 256 (the rectangle
at 0x800c40d8), and database 0's models, animations and hierarchies.
Hierarchies 1-37 get copies, and model 1038 is the sign. 0x8001dbcc marks
the six backdrop CLUT rows (688, 506-511) semi-transparent. Then the title
loop (0x8003d884) runs at 30 frames a second, two vertical blanks a frame.

Each frame:

- every 30 frames (0x800c4080 % 30) a backdrop fade starts;
- the pads: Up, Down or Select move between the three items (sound 3 or
  4), X picks (Start Game: the submenu 0x8003cf80; Load Game: the card
  menu 0x800398bc(8, 1, 0, 0); Audio: 0x80041970), Triangle leaves (-1);
  holding L1, R1, L2 and R2 together plays sound 5;
- the menu draw (0x80035ab8): a fade step, the backdrop, the items, the
  3D camera (0x80040268), the babies (0x8003f41c), the sign;
- CANCEL and ACCEPT with their buttons (0x80035550, 0x80035674);
- a countdown from 300 x 30 loses 5 a vertical blank (0x800c3f1c). At 0,
  or on Triangle, the front end returns -1 and the chooser moves on. Any
  move resets it.

## The backdrop (0x80034fdc, 0x8003528c, 0x80034e80)

Six 128 x 128 4-bit patterns in CHARACTR.RAW, three across and two down
from (576, 256) (the third column on the next page). Pattern (c, r) has
its CLUT at (688, 511 - (c + 3r)). Two layers, each two 256 x 256 quads
(x -256..256, y -128..128, texels 0-255) through a 128 x 128 texture window
on the pattern:

- A (0x8003528c): opaque, pattern 0x800c406c/70, grey 0x800c408d -
  0x800c408c;
- B (0x80034fdc): added, pattern 0x800c4064/68, grey 0x800c408c.

A fade step (0x80034e80): with the level at its full value, A moves to
the next pattern; the level falls 16; at 0, B moves to the next pattern
and the level returns to full. Both start on pattern (2, 0) at 0x80.

## The title menu's items (0x8003d5a0)

"Start Game", "Load Game", "Audio" (pointers at 0x800bb6c0, 0x800bb51c,
0x800bb694), font 1, spacing 1, centred across. They start at y 200, 250,
300 and slide at 12 a frame to 25, 50, 75, then wobble 4 times. They are
drawn white, the chosen one yellow (0x05ffff), each over a black shadow 2
down and right.

## The sign, the buttons, the cameras

The front end's camera (0x800e24c0, reset by 0x80040194): field of view
512 (0x8001dfb8: H = 0xa0000 / (rsin(fov/2) << 12 / rcos(fov/2)), kept as
16 bits; 386 here, 178 for the opening's 955), at (0, 30, -500), angles
0. The sign is object 0x800c40a0, model 1038 at the origin. Its state
(+0xd0) 1 flies the camera in (z + 600 a frame to -500), 2 out (z - 600 to
-17048), 0 holds. Holding, 0x8003f200 nudges the model's vertices every
other frame: odd vertices by step k and even ones back by step 4 - k of
the five (x, y) steps at 0x800b7dd4, k counting on (0x800c40e8). The steps
sum to nothing over five. The trademark (0x8003f0f0) is a raw POLY_FT4,
page 0x1e, CLUT 0x64fb, screen (210, -75)-(230, -69), texels
(192, 148)-(212, 160), drawn while the sign holds.

CANCEL and ACCEPT (0x80035550, 0x80035674; strings at 0x800bb504, 508)
are font-2 text at (-192, -93) and (115, -93) over shadows at +2, +2.
Beside each is a cube, a menu block (0x8003f444: model 334 for 0, 332,
342, 343 for 1-3) at (-600 or 600, -400, 1298). It is drawn with the
second camera (0x800e2580, 0x80040010: (0, 0, -500), angles 0). Each frame
the label's code sets the cube's target angle to 2048 (CANCEL) or 1024
(ACCEPT) at 128 a frame. 0x8003f7f0 eases each angle toward its target
and reads a whole turn as none; the loop start (0x8003ffe8) zeroes them,
so the cubes turn to show the triangle and the cross. The block's matrix
comes from its angles (flag 0x20, 0x80020ee0; checked against the
original before its MatrixNormal pass).

## Unknown

- The babies' motion, the submenus, the sounds' ids.
- Whether MatrixNormal (0x8009f398) changes the cubes' matrices.
