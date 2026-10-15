---
title: In-level screens, the puzzle board, bars, restart and loading
status: partial
discs: US
covers: US SLUS_006.50:0x8009480c, US SLUS_006.50:0x800943bc, US SLUS_006.50:0x800930c8, US SLUS_006.50:0x80093048, US SLUS_006.50:0x80092f5c, US SLUS_006.50:0x80092ed4, US SLUS_006.50:0x80092c68, US SLUS_006.50:0x80092090, US SLUS_006.50:0x80092040, US SLUS_006.50:0x800929f8, US SLUS_006.50:0x800925ec, US SLUS_006.50:0x8009254c, US SLUS_006.50:0x80092754, US SLUS_006.50:0x800924d0, US SLUS_006.50:0x80092450, US SLUS_006.50:0x800921bc, US SLUS_006.50:0x80091f7c, US SLUS_006.50:0x80091ed4, US SLUS_006.50:0x80091e4c, US SLUS_006.50:0x80092d9c, US SLUS_006.50:0x800945d0, US SLUS_006.50:0x80094978, US SLUS_006.50:0x80094178, US SLUS_006.50:0x80094ab0, US SLUS_006.50:0x80094794, US SLUS_006.50:0x80093b6c, US SLUS_006.50:0x800946b0, US SLUS_006.50:0x800936ec, US SLUS_006.50:0x80094728, US SLUS_006.50:0x8009392c, US SLUS_006.50:0x80094bc0, US SLUS_006.50:0x8008f8f4, US SLUS_006.50:0x8008f5d8, US SLUS_006.50:0x800895ac, US SLUS_006.50:0x80035e80, US SLUS_006.50:0x80035f4c, US SLUS_006.50:0x8003f350, US SLUS_006.50:0x800403b4, US SLUS_006.50:0x80035798, US SLUS_006.50:0x80043968, US SLUS_006.50:0x80043d58, US SLUS_006.50:0x8004418c, US SLUS_006.50:0x8003b168, US SLUS_006.50:0x8003eefc, US SLUS_006.50:0x8003f008, US SLUS_006.50:0x8008fc0c, US SLUS_006.50:0x8008fb20, US SLUS_006.50:0x8004d3f0, US SLUS_006.50:0x800bb2e4, US SLUS_006.50:0x800bb32c, US SLUS_006.50:0x800bb38c, US SLUS_006.50:0x800b7d60, US /DATA/DB00/BURST2.CMD, US /DATA/MENU/TEXPOS.DAT, US /DATA/DB01/DB01LP.BIN
worklog:
---

# In-level screens, the puzzle board, bars, restart and loading

The full-screen pages a level can stop for: the puzzle board and its
messages, "You did it!", the Reptar bar award, the "already earned"
notes, the golf over-par box, the pause's "Returning to" pages, the
Select restart menu and the loading screen between levels. Read from the
pseudo-code and the disassembly; each screen was reached in the oracle
and screenshotted (see Checked in the oracle). Builds on
[objectives](objectives.md) (progress block, piece and bar rules, pause,
hint cards), [front end](front-end.md) (backdrop, button cubes, camera),
[text](text.md) (text items) and [golf](golf.md).

Units: positions are (x, y) in pixels from the centre of the 512 x 240
screen, y down. "frame" is a game frame at 30 fps (two vertical blanks;
the oracle's frame numbers below count vertical blanks). dt is 1/300 s
(10 a frame). Pad words are port 0's histories at 0x800cda18 + 4k
([input](input.md)): +0x10 Start, +0x14 Up, +0x18 Right, +0x1c Down,
+0x20 Left, +0x34 Triangle, +0x3c Cross. "Pressed" is `& 3 == 1`, "held"
is `& 1`. Colours are text item words (R | G << 8 | B << 16): yellow
0x13cdec (0x800c4688), pink 0x5d17c6 (0x800c4684), light blue 0xe4b96a,
white 0xffffff, shadow 0x010101 (0x800c468c).

## What is behind these screens

None of them draws the level. Each runs its own frame loop inside the
caller (a cine-end handler, the teardown, a pickup touch, a use callback
or the ball's think), so the level is frozen and not shown. The screen is
the front end's backdrop layer B (0x80034fdc, see
[front end](front-end.md#the-backdrop-0x80034fdc-0x8003528c-0x80034e80))
with a pattern fade started every 30 frames (counter 0x800c4660 in
0x800924d0; 0x800c4080 in the golf box and the restart menu; 0x800c4640
in the loading redraw). Seen: a full-screen pattern, green, orange, blue
or yellow by the pattern of the moment.

## Shared parts

### The burst

The panel behind the messages is model file 240, /DATA/DB00/BURST2.CMD:
one model in the [MOD](../formats/mod.md) layout, loaded once by
0x80035f4c (load_file_alloc(240), 0x80044c98) into 0x800c409c. Its object
is 0x800c40a0 (212 bytes, the front end's sign object): 0x80035e80(model,
0) puts the model in +0x88, identity matrix and origin (0x800b4578),
state +0xd0 = 1. It is drawn by 0x8003f184 (0x80031b84 with the front-end
camera 0x800e24c0).

The camera moves, not the burst. 0x80040194(n) resets the front-end
camera: field of view 512 (0x8001dfb8, H about 386), position (0, 30, z)
(0x800b7d40..48), angles 0, and

- n = 1: z = -17048 (0xffffbd68), burst state 1 (fly in);
- n = 0: z = -500, burst state 2 (fly out);
- n = -1: camera only, the burst state is left alone.

0x800403b4(n) = 0x80035e80(burst, 0) then 0x80040194(n). Each frame
0x8003f350 (through 0x80040408 or 0x8003f41c) moves it:

- state 1: z += 600 while z < -500; the move that reaches -499 or more
  sets z = -500. The next call sets state 0 and returns 1 ("arrived").
  So the fly-in takes 28 frames and arrives on the 29th.
- state 2: z -= 600 until z <= -17048; then z = -17048, state 0, returns
  1. Also 28 frames plus one.
- state 0: the wobble (0x8003f200); returns 0.

Seen: camera z -16448, -15848, ... one step of 600 every two vertical
blanks; -500 about 56 blanks after the start; out the same.

### HUD sprites (0x8008f8f4)

0x8008f8f4(id, &size, &pos, angle, semi) takes a sprite from the free
list (0x8008f4e4), sets corners (-w, -h, d), (w, -h, d), (-w, h, d), (w,
h, d) from size = (w, h, d) (0x8008f808), position +0x08 = (x, y), the
TEXPOS entry +0x38 (0x80089fa4: entry id % 428 of the loaded TEXPOS.DAT,
or the empty 0x800b9ba4), angle +0x0e, semi +0x16. Every screen here
passes angle 0 and semi 0. 0x8008fa10 draws one (0x8008f5d8: x scaled by
0x800c4a04 / 320 = 1.6, RotTransPers, then the position added in screen
pixels); 0x8008fa9c moves it; 0x8008fafc resizes it; 0x8008f990 frees it.

So the drawn half size is (w * 1.6 * H / d, h * H / d) pixels with the
front-end H of about 386 (0x80040194 sets it before every screen here).
Measured: the board (128, 169, 1024) spans x 179..330 and y 58..184 on
screen, half size 76 x 63.5 (formula: 77 x 64). The port's
`hud::Sprites::draw` already does this with `focal`.

TEXPOS entries are POLY_FT4 texture words: uv0, CLUT, uv1, tpage, uv2,
uv3. Entries 314-351 are the same in MENU/TEXPOS.DAT and every
DBnn/TEXPOS.DAT but DB08's (a 108-byte stub):

| id | what | texels | page | CLUT |
|---|---|---|---|---|
| 314 + i | piece i, in colour | 64 x 64, see below | (896, 256) 4-bit | (688, 501 - i) |
| 326 + i | piece i, drawn 2 down and right (guess: a shadow) | same | same | (944, 396) for all |
| 338 + i | piece i, greyed (not earned) | same | same | (944, 384 + i) |
| 350 | the board | (128, 0)-(251, 163) | (640, 256) 4-bit | (688, 489) |
| 351 | a cover tile | (160, 164)-(255, 255) | (896, 256) 4-bit | (688, 488) |

Piece texels (u, v of the top left, 64 x 64): 0 (0, 0), 1 (64, 0), 2
(128, 0), 3 (0, 64), 4 (64, 64), 5 (128, 64), 6 (0, 128), 7 (64, 128), 8
(192, 0), 9 (0, 192), 10 (64, 192), 11 (192, 64).

Sprite 291 (the Reptar bar icon) differs per database and is empty in
MENU, DB00, DB19 and DB22.

### Text

Every text here is a 44-byte item ([text](text.md)) with spacing 1 and
no slide flags, drawn over a shadow item 2 right and 2 down in 0x010101
(shadow first). x is centred with 0x8002e144(item, 1) unless given.
Fonts: 0 (16 high), 1 (24), 2 (12).

### PRESS X TO CONTINUE, CANCEL and ACCEPT

- 0x80035798: "PRESS     TO CONTINUE" (string 67) font 2 white at (-96,
  80), shadow (-94, 82); in the gap a button cube (0x800c4b40, a menu
  block from 0x8003f444(0)) at (-60, 414, 1298) turned to angle 1024 at
  128 a frame: the cross face. Drawn with the second camera
  (0x80040010).
- 0x80035550 / 0x80035674: CANCEL and ACCEPT (strings 7 and 8) with
  their cubes 0x800c4b14 / 0x800c4b18, as in the
  [front end](front-end.md#the-sign-the-buttons-the-cameras): font 2
  white at (-192, -93) and (115, -93), cubes at (-600, -400, 1298) and
  (600, -400, 1298) turned to 2048 (triangle) and 1024 (cross).

### The message loop (0x80093048)

Used by the piece messages, "You did it!", the bar screen and the
already-earned notes. The texts are three items at 0x800e7300, +0x2c,
+0x58 (shadows 0x800e7390, +0x2c, +0x58); 0x80091ed4 draws up to six
items through the pointer lists 0x800e72c0 (text) and 0x800e72e0
(shadow), plus the bar count (0x800e7570 then 0x800e7540) while
0x800c4e04 is set. 0x80091e4c clears both lists, the x of the six items
and the count's string.

1. 0x800403b4(1): the burst flies in.
2. Each frame 0x80092f5c(1), the speech stream (0x8008d654), the draw
   (0x80092ed4):
   - 0x80092f5c: the backdrop fade (0x800924d0); the burst move
     (0x80040408). On the arrival frame the three item pointers go into
     the lists, so the texts appear on the 29th frame and stay. The bar
     icon 0x800c4e04, if any, is drawn every frame.
   - Cross pressed: sound 126 (0x8006f26c) and the loop ends, at any
     time, even while the burst is still flying in.
3. 0x80091e4c (texts off), 0x800403b4(0): the burst jumps to z -500 and
   flies out; each frame 0x80092f5c(0) and the draw until it is out (29
   frames).

The draw (0x80092ed4): frame start (0x8001d440(-1)), backdrop B
(0x80092178), the burst (0x80040428 -> 0x8003f184), the texts
(0x80091ed4), the blinking piece (0x80092d9c), PRESS X TO CONTINUE
(0x80035798), flip at 30 fps with the pads sampled (0x8001d3e8(1)), then
the controller check (0x80039720: with no pad it holds a "No Controller"
page until one is back).

0x80092d9c: the lowest earned, unplaced piece (sprite 0x800e71d0[i] and
its shadow 0x800e7200[i]) at (0, 45) and (2, 47), size (64, 64, 1024),
drawn only while VSync(-1) % 30 < 15: a quarter second on, a quarter
second off. Only the piece screen made those sprites, so only its
messages show it. Seen: piece 11 blinking under the burst while it flew
in.

## The puzzle screen (0x8009480c)

### Who calls it

| caller | argument | when |
|---|---|---|
| 0x80094bfc(level) | level | a level's piece or a bought piece was just awarded ([objectives](objectives.md#puzzle-pieces)); from 0x80076d6c and the teardown |
| 0x80094a58(level, bar) | level | a Reptar bar picked up made 15 unspent bars (0x80094924) |
| 0x8004d3f0 (use callback 1) | 0 | Triangle at the board object, template 581 (model 905), id 54: DB01 at (3103, 711, 514) in Pickles' Home; also placed in DB07, DB13, DB15, DB18, DB20-22. With +0x30 bit 512 it posts SwitchEpisode(7) instead |

Template 657 (model 984) also has use callback 1; DB23 places it at
(45572, -6903, -3336), id 0 (not checked).

0x80094bfc pauses the speech (0x8008dbec) and, when the level owns a
piece, stops the effects (0x8006f828, guess) and pauses the music
(0x8006ec40); after the screen the music resumes (0x8006ed30).

### Steps (0x8009480c)

1. Save the GTE state (0x8009fd08); sample the pads twice (0x8001acc0).
2. Three button cubes (0x8003f444(0), state 2): 0x800c4b40 (PRESS),
   0x800c4b18 (ACCEPT), 0x800c4b14 (CANCEL).
3. 0x800943bc(level): the sprites, then the message (0x800930c8).
4. Level not 0: the message loop (0x80093048). Level 0: 0x80040194(-1)
   (camera only) and 0x80091e4c (no texts).
5. Sample the pads twice. Unless level is 1 and exactly one piece is
   earned (0x80094b8c): the board (0x80092c68).
6. Free the sprites (0x800945d0) and the cubes (0x800219d8), restore the
   GTE state (0x8009fd78).

### The sprites (0x800943bc)

Clears 0x800c4660 (frame counter), selected 0x800c4e11 = -1, cursor
0x800c4e12 = -1, state 0x800c4e08 = 0. Then:

- 0x800c4e00: sprite 350, the board, size (128, 169, 1024) at (-1, 2).
- 0x800c4e0c: sprite 351, the cover tile, size (96, 92, 1024), moved
  per tile when drawn.
- For each piece i (0-11), with tray position T[i] (0x800bb32c) and board
  slot S[i] (0x800bb38c), 8 bytes each (x, y, 1024, 0):
  - earned (+0x44 bit i, 0x80091ca8): sprite 314 + i into 0x800e71d0[i]
    and 326 + i into 0x800e7200[i] (at the position + (2, 2)). Placed
    (+0x46 bit i, 0x80091cd4): at S[i], size (64, 64, 1024)
    (0x800c4674); else at T[i], size (64, 64, 1536) (0x800c467c).
  - not earned: sprite 338 + i into 0x800e7230[i] at T[i], size (64, 64,
    1536).
  - The position goes to 0x800e7260[i] (the current position).

| i | tray T (0x800bb32c) | slot S (0x800bb38c) |
|---|---|---|
| 0 | (150, -2) | (-52, -44) |
| 1 | (-200, -54) | (-1, -44) |
| 2 | (-150, -28) | (48, -44) |
| 3 | (150, -54) | (-52, -14) |
| 4 | (200, 26) | (-1, -14) |
| 5 | (-150, 26) | (49, -14) |
| 6 | (200, 76) | (-52, 18) |
| 7 | (200, -28) | (-1, 16) |
| 8 | (-200, -2) | (48, 18) |
| 9 | (150, 50) | (-52, 46) |
| 10 | (-150, 76) | (-1, 46) |
| 11 | (-200, 50) | (48, 46) |

The slots make a 3 x 4 grid on the board (columns about -52, -1, 48;
rows -44, -14, 17, 46). Drawn sizes: 77 x 48 pixels at depth 1024, 51 x
32 at 1536 (from the formula). Seen: the earned pieces in colour on two
trays left and right of the board, the others grey.

### The messages (0x800930c8 and its siblings)

All font 1 with shadows, x centred, at y -65, -40, -15 (shadows -63,
-38, -13). Strings are indices into the table at 0x800bb4e8.

| function | when | lines (string) | colours |
|---|---|---|---|
| 0x800930c8 | level 1 and one piece earned | "Help Tommy win" (88), "twelve puzzle pieces" (89), "by completing levels!" (90) | yellow, yellow, yellow |
| 0x800930c8 | otherwise | "Congratulations!" (91), "You earned a" (92), "puzzle piece!" (93) | yellow, pink, pink |
| 0x80094178 | all twelve placed | "You did it!" (228), "You've found all the" (230), "Reptar Puzzle Pieces!" (231) | yellow, pink, pink |
| 0x80093b6c(n) | bar award | "Great Job!" (100), "You have earned" (101), "n Reptar bar(s)!" (101 + n, n 1-4) | yellow, pink, pink |
| 0x800936ec | piece already given | "You have already" (97), "earned the puzzle" (98), "piece in this level." (99) | yellow x 3 |
| 0x8009392c | bars already won | "You have already" (97), "earned the reptar bars" (223), "for this level." (224) | yellow x 3 |

0x800930c8 also sets the board's three titles (font 1, pink, centred,
y -90, shadow y -88): "Select Piece" (94) at 0x800e7420, "Place Piece"
(95) at 0x800e7450, "Terrific!" (96) at 0x800e7480; shadows at
0x800e74b0, 0x800e74e0, 0x800e7510.

### The board (0x80092c68)

1. 0x80092090(1): all 35 cover tiles on (0x800e75a0[35] = 1), a fresh
   order (0x800e75d0), counter 0x800c4e10 = 0.
2. Each frame until 0x800929f8(0) returns 0: speech stream, draw
   (0x80092450). This uncovers one tile a frame (35 frames) and runs the
   state machine below at the same time.
3. 0x80092090(0) (tiles off, a fresh order). Each frame: speech stream;
   Cross pressed (sound 126) or Triangle pressed (no sound) ends the
   wait; backdrop fade, ACCEPT and CANCEL, draw.
4. Each frame until 0x800929f8(1) returns 0: one tile covered a frame
   (35 frames), no state machine.
5. All twelve placed (0x80091d28): SwitchEpisode(player 0x800e2b48, 7)
   (0x800781f8): Reptar 2010 next.
6. Music on (0x8006ed30).

The order: for k = 0..34, draw r = (rand() & 255) % 35 (rand 0x8002fb7c)
until order[r] is free, then order[r] = k. Step n (0x80092040) sets
tile[order[n]]. The port needs the game's rand to match it.

The tiles (0x80091f7c): tile 5c + r at (-267 + 89 c, -104 + 52 r), c
0-6, r 0-4, sprite 351, drawn 116 x 69 pixels, so they overlap like
jigsaw pieces. Seen: black jigsaw shapes covering everything, opening in
a random order, then closing.

### The state machine (0x800929f8(0), state 0x800c4e08)

Each frame, before the state: backdrop fade (0x800924d0), ACCEPT
(0x80035674), CANCEL (0x80035550), one tile step.

- 0: selected = cursor = -1, state 1, title "Select Piece".
- 1, select (0x800925ec):
  - Exactly one earned piece unplaced: it is selected at once (0x8009254c
    from 0), state 2, title "Place Piece". So "Select Piece" shows for
    one frame.
  - Else Up, Down, Left or Right pressed: sound 124, cursor + 1. The
    cursor is then moved to the next earned, unplaced piece from there,
    wrapping to 0 (0x8009254c). Every direction goes forward.
  - Cross pressed: sound 126, selected = cursor, cursor = -1, state 2.
  - Triangle pressed: sound 125; with twelve earned, hint card 713
    (HINT42, 0x80044434(713, 0)); the board ends (to step 3, whose wait
    sees the same Triangle press and ends at once).
- 2, place (0x80092754): on the selected piece's position P:
  - Up held: y - 3; Down held: y + 3; Left held: x - 7; Right held: x +
    7 (a frame). Sound 124 on each press, not while held.
  - Clamped to x -200..200, y -76..76.
  - d2 = (trunc((S.x - P.x) * 320 / 512))^2 + (S.y - P.y)^2.
  - Triangle pressed: sound 125, the piece back to its tray; with one
    piece left unplaced the board ends, else state 0 (back to select).
  - Cross pressed, d2 <= 100: sound 126, P = S, +0x46 |= bit i
    (0x8009195c on 0x800c56be), state 0.
  - Cross pressed, d2 > 100: sound 124, the piece back to its tray; it
    stays selected in state 2.
- 3, "Terrific!": counter 0x800c4e14 -= dt each frame while it is at
  least dt; below dt the board ends. Set to 150, so 15 frames.
- After any state: every sprite pair is moved to 0x800e7260 (the shadow
  2 right and down). If pieces earned == placed and the state is not 3:
  state 3, counter 150, title "Terrific!".

The title stays during the wait (step 3). With several pieces the player
places them one by one; each success returns to select.

### The board's draw (0x80092450)

Frame start, backdrop B, the board 350 (0x80092198), the titles
(0x80091ed4), then 0x800921bc:

1. The covered tiles (0x80091f7c).
2. The selected piece, if any: piece and shadow at size (64, 64, 1024).
   Else the cursor piece, if any, at (64, 64, 1024), only while
   VSync(-1) % 30 < 15 (it blinks).
3. The other pieces: earned and placed at size 1024, earned and not
   placed at 1536 (each piece then its shadow); not earned, the grey
   sprite at 1536.

Then the flip and the controller check.

### From Pickles' Home (argument 0)

No message: 0x80040194(-1) sets the camera only, the texts are cleared,
and the board starts with its dissolve at once. With one earned piece
unplaced it is selected at once; with none earned or none unplaced the
board shows "Terrific!" for 15 frames and waits (state 3 is entered
because earned == placed). Seen: a new game in the house, Triangle at
the board: the dissolve, "Terrific!" over an empty board.

## "You did it!" (0x80094978)

Runs only when all twelve are placed (0x80091d28 == 12); returns 1 then.

1. Sample the pads twice, the PRESS cube (0x8003f444(0)).
2. The texts (0x80094178, table above).
3. Speech 2046 (0x8008da4c).
4. The message loop (0x80093048).
5. Sample the pads twice, free (0x800945d0, 0x800219d8).

No board. Callers then go to level 7: 0x80076d6c (SwitchEpisode(7) unless
+0x30 bit 512; also sets +0x38 bit level), the teardown (0x80018354) and
the bar pickup (0x800514b0: 0x800781f8(player, 7)). Placing the twelfth
piece on the board already posted SwitchEpisode(7) (step 5 of the board).

## The bar screen (0x80094ab0 -> 0x80094794 -> 0x80093b6c)

0x80094794(n): pads twice, the PRESS cube, 0x80093b6c(n), the message
loop, pads twice, free.

0x80093b6c(n): the three lines above for n 1-4 (other n: no strings are
set). Then:

- sprite 291 (the bar icon from the level's TEXPOS) into 0x800c4e04, size
  (52, 64, 1024) at (192, 76): about 63 x 48 pixels, bottom right;
- the count: "%d" (0x800c4698) of the bars available (0x80094a0c) into a
  buffer in 0x80093b6c's own stack frame, font 1, white, at (120, 76),
  not centred; shadow at (122, 78) (items 0x800e7540 and 0x800e7570).

The count's string pointer outlives the function: it points into a dead
stack frame (0x801ffe00 in the oracle), below the message loop's frame.
It survived in the oracle (the count drew as "3" and "15"). The port
keeps its own string.

The icon and count show while the burst flies in and holds; they go when
it flies out (0x80092f5c(0) does not draw the icon and 0x80091e4c drops
the string).

Callers: 0x80076d6c with n = the bonus bars of the level (see
[objectives](objectives.md#reptar-bars)), and the teardown.

### The teardown award (0x800182b0)

In the runner (0x80017c0c), after the frame loop and the exit code:

1. 0x8008a7ec(-1).
2. +0x30 bit 1024 set (a golf hole in one, [golf](golf.md#holing-out-0x8007a9d8)):
   clear it, 0x80094b10(level, 1) (bar bit 31 of the level's word), the
   bar screen with n = 1.
3. Fewer than twelve earned: 0x80094bfc(level) (which buys a piece with
   15 bars, and gives the level's own piece if it has one and it was not
   given), then 0x80094978.
4. When that returned non-zero and +0x30 bit 512 is clear: +0x38 |= bit
   level, SwitchEpisode(7).

With twelve already earned, step 3 is skipped and s2 keeps a leftover
(1, or 32 after a quit), so step 4 still posts SwitchEpisode(7) (guess
from the register use; not run).

## The "already earned" notes

At a level's start, after loading and before the start cinematic
(0x80017c0c):

- Level not 1, 0x80094bc0(level) (level 25 is read as 8: the level owns a
  piece and +0x40 has it), and the previous level (+0x04) is another
  level: 0x80094ad0(1) -> 0x800946b0 -> 0x800936ec, "You have already
  earned the puzzle piece in this level." A restart (same level) does not
  show it.
- Level 13 with +0x38 bit 13, or level 18 with +0x38 bit 18, not in
  activity play: 0x80094af0 -> 0x80094728 -> 0x8009392c, "You have already
  earned the reptar bars for this level."

Both use the message loop with the PRESS cube; no blinking piece.

## The golf over-par box (0x80043968)

Called by holing out over par ([golf](golf.md#holing-out-0x8007a9d8)).

- "You were over par." (214), font 1, pink, centred, y -50; "Try Again"
  (232), font 0, pink, centred, y -15; shadows (+2, +2).
- "PRESS     TO CONTINUE" (67), font 2, white, (-96, 80), shadow (-94,
  82); its cube (a new 0x8003f444(0) block) at (-60, 410, 1298), angle
  1024 at 128.
- 0x800403b4(1): the burst flies in; the four texts are drawn from the
  arrival frame on (0x80040408 latched).
- Each frame: Cross pressed: sound 126 and leave after this frame (even
  during the fly-in). Backdrop fade every 30 frames (0x800c4080), backdrop
  B, the PRESS line, the burst, the cube, flip.
- Leaves with the cube freed and no fly-out; the next frame is the
  level's. Seen: the box until Cross, then the level at once.

## The pause's "Returning to" pages (0x8003e05c)

After the pause menu ([objectives](objectives.md#pause-0x8003e05c))
returns -1 (Quit Game, or Quit Level in level 1, 16 or activity play):

1. The burst flies out (state 2) and back in (state 1), one frame at a
   time with backdrop B, until it holds.
2. Activity play: "Returning to the" (71), font 0, light blue, y -55;
   "Activities Menu" (72), font 1, pink, y -35; "Loading" (24), font 0,
   pink, y -5. Else: "Returning to" (73) / "Main Menus" (74) / "Loading"
   (24), same fonts and colours. Centred, shadows (+2, +2).
3. Drawn for two frames once the burst holds; then the pause returns -1.

Result -2 would show "Returning to" (73), font 0, light blue, y -55, and
"Pickles' House" (75), font 1, pink, y -35, the same way. 0x800398bc
only returns 0, 1 and -1 (its result variable, set at 0x80039cb4 and
nearby, takes no other value), so this page is not reached (guess: left
from an older menu). Quit Level in
other levels returns 1 and gets the loading screen of level 1 instead.

On -1 the runner clears the progress (+0x22, +0x23, +0x30..+0x46, the
bar words +0x48..+0x147) and returns -1 to main, which runs the front end
(0x80018264). In activity play the runner forces the exit to -1 (so the
clear happens too); when the round ended without a quit it shows the
loading screen with level -1 (0x800183ac).

Seen: Quit Game in level 2: the burst out and in, "Returning to / Main
Menus / Loading" for one sampled frame, then the title's sign.

## The Select restart menu (0x80043d58)

Started from the frame loop when Select is pressed (`*0x800c49d8 & 3 ==
1`) outside activity play (0x800190c0); its result is the exit code
0x800c4918.

1. 0x80090234(0) as the pause does (guess: HUD texts off), save the GTE
   state, camera field of view 512, two cubes: CANCEL 0x800c4b14 and
   ACCEPT 0x800c4b18.
2. Items (0x8004418c(items, offset, 3)), twice, the second set with offset
   2 for the shadows:
   - "Restart Level?" (229), font 1, centred, y -55;
   - "Yes?" (11), font 0, at (-90, -20);
   - "No?" (12), font 0, at (45, -20).

   With no pad (0x8001b6b4(0) < 0) the items are "No Controller" (211) or
   "Unsupported Controller." (210) at y -60, "Please insert a supported
   controller" (212) at y -30, "into controller port 1." (213) at y -10.
3. The burst (0x80035e80) runs until it holds, without frames: the
   camera is already at -500, so it shows at once (no fly-in seen).
4. Cursor starts at 2 (No?). Cross already pressed on entry: result 0.
5. Each frame:
   - Left pressed and cursor > 1: cursor - 1, sound 124. Right pressed
     and cursor < 2: + 1, sound 125. Up pressed and < 2: + 1, sound 124.
     Down pressed and > 1: - 1, sound 125.
   - Start or Triangle pressed: cubes freed, 0x80090234(1), GTE
     restored, sound 124, result 0.
   - Colours: the title white; the chosen item yellow 0x13cdeb, the other
     white.
   - Draw: the texts (shadows first), backdrop fade every 30 frames,
     frame start, backdrop B, the burst, CANCEL and ACCEPT, flip.
   - Cross pressed: leave the loop.
6. Cross: cubes freed, 0x80090234(1), sound 126. Yes: levels 23-31 give 25
   (the whole golf course again from Hole One); other levels give the
   level itself with +0x30 bit 2 (checkpoint) cleared. No: 0.

A level number as the exit code ends the frame loop; the runner reloads
it (with the loading screen). Seen: level 2, Yes gave "Grandpa's Teeth /
Loading"; level 27 (Hole Nine), Yes gave "Ice Cream Mountain / Hole One /
Loading" and level 25.

## The loading screen (0x8003b168, 0x8003eefc)

The runner, after the teardown (0x80018368): previous level (+0x04) =
level; if the exit code is 0 or more, level = exit code,
0x8003b168(items, shadows, 2, 2) and 0x8003eefc(items, shadows, count).
The front end's Start Game path does the same (0x8003df20) for the first
level.

### The lines (0x8003b168(items, shadows, dx, dy))

By level (switch 0x800133f0 on level + 1). Up to three lines; line k has
string L[k], font F[k] and y Y[k]; x centred. Colours: every line pink
but the last, which is light blue; for the bonus case (s6) line 0
yellow, line 1 pink, line 2 light blue. Shadows at (+dx, +dy) = (2, 2) in
0x010101. Returns the line count.

| level | lines (string) | fonts | y |
|---|---|---|---|
| -1 | "Returning to the" (71), "Activities Menu" (72), "Loading" (24) | 0, 1, 0 | -55, -35, -5 |
| 0 | "Animation Testing" (235), "Database" (234), "Loading" | 1, 1, 0 | -65, -35, -5 |
| 1, +0x30 bit 4 clear | "Search for Reptar" (183), "Beginning" (182), "Loading" | 1, 0, 0 | -35, -55, -5 |
| 1, bit 4 set | "Pickles' Home" (151), "Returning to" (73), "Loading" | 1, 0, 0 | -35, -55, -5 |
| 2, 3, 6, 7, 8, 10, 11, 12, 16; 13 and 18 outside activity | name, "Loading" | 1, 0 | -50, -20 |
| 4 | "7 Voyages of" (188), "Cynthia" (189), "Loading" | 1, 1, 0 | -60, -30, 0 |
| 5 | "Visitors from" (190), "Outer Space" (191), "Loading" | 1, 1, 0 | -60, -30, 0 |
| 9 | "The Mysterious" (192), "Mr. Friend" (141), "Loading" | 1, 1, 0 | -65, -35, -5 |
| 14 | "The Return of" (193), "Mr. Friend" (141), "Loading" | 1, 1, 0 | -65, -35, -5 |
| 13, 18 in activity | "Activity" (187), name, "Loading" | 0, 1, 0 | -60, -40, -10 |
| 17 in activity | "Activity" (187), name, "Loading" | 0, 1, 0 | -55, -35, -10 |
| 15, 20, 21; 17 outside activity (bonus, s6) | "Bonus Level" (184), name, "Loading" | 0, 1, 0 | -55, -35, -10 |
| 23-31 | "Ice Cream Mountain" (158), hole name (173 + hole index 0x80057d50), "Loading" | 1, 1, 0 | -65, -35, -5 |
| 23-31 in activity | "Player 1" (233), "Hole One" (173), "Loading" | 1, 1, 0 | -65, -35, -5 |
| 19, 22, others | "Unknown" (236), "Database" (234), "Loading"; prints "PrintLoadingDatabase: unknown db" | 1, 1, 0 | -65, -35, -5 |

"name" is the level's name: string 0x800b7d60[level] (s16 a level): 1
"Pickles' Home" (151), 2 "Grandpa's Teeth" (152), 3 "Incident on Aisle
7", 6 "Toy Palace", 7 "Reptar 2010" (157), 8 "Ice Cream Mountain", 10
"Chuckie's Glasses", 11 "Circus Angelicus", 12 "Let There Be Light", 13
"Cookie Race" (111), 15 "Mirrorland", 16 "Training Level", 17 "Gold
Rush" (112), 18 "Egg Hunt" (114), 20 "No More Cookies", 21 "Touchdown
Tommy", 23-31 "Hole One" .. "Hole Nine" (173-181). For golf the name
comes from the hole index, so level 25 shows "Hole One".

+0x30 bit 4 is set at the runner's end after cine 85 "Reptar Dance" (its
end, 0x8007737c case 84, sets 0x800c3e80 and exit -1; the runner then
plays movie 11 with 0x8008b1e4 and returns 1; guess: the game's ending).
Before that every return to the house says "Beginning / Search for
Reptar" (seen after Quit Level).

### The screen (0x8003eefc)

1. 0x800403b4(1): the burst flies in.
2. Each frame: frame start, backdrop B (no fade step here), camera, the
   burst (0x8003f41c), sounds (0x8006f170); once it holds, the lines
   (shadow then text); flip.
3. After two frames with the burst holding: 0x800c4098 = 1, return. So
   29 + 2 frames.

During the load the runner's start (0x8008fc0c(level)) builds the same
lines into 0x800e6e50 / 0x800e6d78 (count in 0x800c4638) and installs
0x8008fb20 as 0x800c3ecc (guess: called while the loader waits). It
redraws, whenever the last frame is done (0x8001d3c4): backdrop B with a
fade every 30 calls (0x800c4640), the burst held (0x8003f008), the
lines. 0x8008fbdc removes it after the load. The next camera update
(0x80040268) sees 0x800c4098, resets the camera (0x80040194(0)) and sets
the burst back to state 1.

Seen: level 7 "Reptar 2010" (pink, font 1, y -50) over "Loading" (light
blue, font 0, y -20); level 2 "Grandpa's Teeth / Loading"; level 1
"Beginning / Search for Reptar / Loading"; golf "Ice Cream Mountain /
Hole One / Loading". The burst took about 56 vertical blanks to arrive.

## Sounds

| call | number | where |
|---|---|---|
| 0x8006f26c | 124 | board: cursor move, piece move press, wrong place; restart: Left, Up, Start/Triangle |
| 0x8006f26c | 125 | board: Triangle in select or place; restart: Right, Down |
| 0x8006f26c | 126 | Cross: message dismissed, piece chosen, piece placed, board wait; golf box; restart |
| 0x8008da4c | 2046 | "You did it!" (speech) |

No screen here uses 0x8006f644 (sounds at a position).

## Checked in the oracle

A scratch probe on rugrats-hle (pokes, one-shot pokes, hooks with
arguments, a call injection that redirects a per-frame function to a
chosen one with chosen arguments, a puzzle state trace, screenshots):
presses start 300, cross 500, 650, 800, 1650, 1800; main's level
argument patched (0x80018824 = `li a0, N`); 0x800c5678 = N at the
runner's entry; cards skipped (0x8004472c = 0x1000004a); start cines
skipped (0x80074958 returns 0). Injections at 0x800818e0 (the per-frame
ambience call). Frame numbers are vertical blanks.

- Level 2, 0x80094bfc(2) at 2001 (new game): piece 11 earned, +0x40 bit
  2. "Congratulations! / You earned a / puzzle piece!" with PRESS X TO
  CONTINUE; the burst arrived at about 2057, the texts at 2064; piece 11
  blinking at (0, 45). Cross at 2150: sound 126, burst out by 2209. Board:
  "Place Piece" at once (one piece), 35 tiles opened one per frame. Right
  held: x -200, -186, ... +7 a frame, 52; Cross: sound 126, snapped to
  (48, 46), +0x46 = 0x800, "Terrific!", counter 150 down by 10 a frame
  for 15 frames. Cross at 2420: tiles closed over 35 frames, back to the
  level at 2496.
- 0x8009480c(0) with three earned: no message, "Select Piece" with
  piece 0 blinking; Right: cursor 1 (sound 124); Cross: piece 1 selected
  and enlarged (126); a Cross far from the slot: sound 124, back to the
  tray, still "Place Piece"; Triangle: sound 125, back to select;
  Triangle in select: sound 125 and the tiles closed at once.
- Eleven placed, 0x80076d6c(2, 1): piece 5 given; placed with Right+Up
  and Right; "Terrific!"; after the wait 0x800781f8(player, 7) from
  0x80092d7c; then "You did it! / You've found all the / Reptar Puzzle
  Pieces!" with speech 2046; then SwitchEpisode(7) from 0x80077030 and
  the loading screen "Reptar 2010 / Loading".
- 0x80094ab0(3) with three bars: "Great Job! / You have earned / 3
  Reptar bars!", the bar icon and "3" bottom right; items 0x800e7540 at
  (120, 76) white, string at 0x801ffe00.
- +0x30 = 0x500, 14 bars, Select restart: teardown 0x80094b10(2, 1), "1
  Reptar bar!" with "15", then 0x80094924 bought piece 9, "Congratulations",
  board; Triangle in place mode with one piece left closed the board;
  0x80094978 returned 0; the loading screen.
- Level 1, 0x80094bfc(1): "Help Tommy win / twelve puzzle pieces / by
  completing levels!" all yellow, no board after it.
- Level 1, Tommy poked to (3103, 779, 1300) facing the board, Triangle:
  0x8004d3f0(id 54) from 0x8005f614, then 0x8009480c(0), the board.
- +0x40 bit 2 before level 2 loads: "You have already earned the puzzle
  piece in this level." (0x80094ad0 from 0x8001809c). +0x38 bit 13 before
  level 13: "You have already earned the reptar bars for this level."
- 0x80043968 in level 2: "You were over par." (font 1) / "Try Again"
  (font 0), pink; Cross: sound 126, the level on the next frame.
- Select in level 2: "Restart Level?" with No? yellow; Left: Yes? yellow
  (124); Right (125); Cross on No (126) went back; Yes reloaded level 2.
  In level 27 Yes loaded level 25 with "Ice Cream Mountain / Hole One".
- Pause, Quit Game, Yes: the burst out and in, "Returning to / Main
  Menus / Loading", then the front end. Pause, Quit Level: loading screen
  "Beginning / Search for Reptar" (bit 4 clear).
- Not checked: activity play pages, the 12-earned Triangle (HINT42),
  bar counts 2 and 4, the no-controller pages, the -2 page, the board in
  DB07 and template 657 in DB23, a bar pickup that buys a piece, the
  load-time redraw's caller.

## For the port

Order (port code: crates/rugrats/src/front.rs has the backdrop, the
cubes and the sign; hud.rs the sprites; text.rs the items; progress.rs
the piece and bar rules):

1. A screen runner that takes over from the level: its own 30 fps loop
   with backdrop B and the fade every 30 frames, the front-end camera
   (H about 386) and the pads.
2. The burst: load BURST2.CMD as a MOD model; the camera fly-in and
   fly-out (600 a frame between z -17048 and -500, 28 frames plus one).
3. The message page: three centred lines with shadows, PRESS X TO
   CONTINUE and its cube, Cross to leave at any time; the blinking piece.
   Then the six message kinds and the bar page (icon 291 from the
   level's TEXPOS, the count).
4. The board: sprites 314-351 from TEXPOS at the tray and slot tables,
   the 35-tile dissolve with the game's rand, the four states, the 7 / 3
   pixel moves, the snap test d2 <= 100 with the 320 / 512 x scale,
   "Terrific!" for 150 dt, the wait, +0x46, SwitchEpisode(7) at twelve.
5. The callers: 0x80094bfc after a level, the teardown bar award, the
   house board's use callback, "You did it!", the start-of-level notes.
6. The golf box, the restart menu (golf restarts at 25) and the pause's
   Returning page.
7. The loading screen: the line table, then the two held frames and the
   redraw while loading.

## Unknown

- Whether sprite 326 + i is a shadow (one dark CLUT for all twelve, drawn
  2 down and right) and the colours of the CLUTs at (944, 384..396).
- The VRAM page (896, 256) and (640, 256): which file puts the puzzle
  art there.
- The pause's -2 page: dead or reached from somewhere not found.
- The teardown's leftover s2 with twelve earned (step 4 posting
  SwitchEpisode(7)); whether a posted SwitchEpisode survives the level
  change.
- Template 657 in DB23 (use callback 1, id 0) and the board in DB07 and
  the bonus databases: whether they are reachable.
- What calls 0x800c3ecc during a load; 0x8008a7ec(-1) at the teardown.
- 0x8006f828 (guess: stops the object sounds) and 0x8008dbec (guess:
  pauses speech).
- Movie 11 and cine 85's end path (+0x30 bit 4) were read, not run.
