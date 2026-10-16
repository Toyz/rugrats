---
title: The front end
status: partial
discs: US
covers: US SLUS_006.50:0x80030158, US SLUS_006.50:0x8003eb28, US SLUS_006.50:0x8003d884, US SLUS_006.50:0x8003d5a0, US SLUS_006.50:0x80035ab8, US SLUS_006.50:0x80034fdc, US SLUS_006.50:0x8003528c, US SLUS_006.50:0x80034e80, US SLUS_006.50:0x8008b100, US SLUS_006.50:0x8002fb7c, US SLUS_006.50:0x80040194, US SLUS_006.50:0x8003f200, US SLUS_006.50:0x8003f7f0, US SLUS_006.50:0x8001dfb8, US SLUS_006.50:0x80020ee0
worklog: 18, 19, 21
---

# The front end

## Main's chooser (0x80030158)

Main (0x800185f8) first shows the legal still, the THQ, Nickelodeon and
n-Space logos, and the opening cinematic (0x80074958(-1, 0)), then runs
it with mode -1. It cycles: the front end (0x8003eb28(30)), then a demo,
then the opening, and back. A demo skipped with a button (the movie
player 0x8008b448 returns 1; 0 when it ran to its end) goes straight back
to the front end, with no opening. It
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

## The Audio menu (0x80041970)

The title runs it on X at Audio, after sound 2 and the items sliding away
(0x8003de08). It takes no arguments. Checked in the oracle (probe runs,
frames 884-1300).

### What it edits

Three words of the game state (0x800c5678):

| word | what | range | boot value (0x800b27e4) |
|---|---|---|---|
| 0x800c5684 (+0x0c) | music volume | 0-100 | 90 |
| 0x800c5688 (+0x10) | effects volume | 0-100 | 100 |
| 0x800c568c (+0x14) | 0 stereo, 1 mono | 0, 1 | 0 |

It copies all three at the start (sp+0x19c, 0x1a0, 0x1a4) for Triangle.
The cursor (fp) is 0 Music, 1 Sfx, 2 the stereo item, and starts on Music
each time. The pads (port 1 at 0x800cda18; the release test reads both
ports), a game frame:

- Triangle pressed (0x800422b0): sound 1, the three copies go back, the
  result is 0.
- Cross or Start pressed (0x800422f8): sound 2, the result is 1.
- Up pressed (0x80042338): sound 4, the cursor goes back one, 0 to 2.
- Down or Select pressed (0x80042364): sound 3, on one, 2 to 0.
- Left or Right released (0x8001bda4(pad, 8 or 6, 1): history & 3 is 2)
  with the cursor on Sfx: sound 5. Each one released plays it.
- Left or Right pressed on the stereo item (0x800423f4): the flag flips.
  No sound.
- Left held (0x80042540) or Right held (0x800425dc), on Music or Sfx: the
  value goes down or up 1, clamped to 0-100. Both held: down, then up.

So a volume moves 1 a game frame while held, 2 vertical blanks. The music
volume is heard at once: every frame it is held, even at a limit, it goes
to the song playing (0x8006ffa8(0x800b803a, v, v), SsSeqSetVol
0x800a91a0; 0x800b803a is the menu set's song, +0x0a of 0x800b8030). The
effects volume is heard when sound 5 plays on the release, since
0x8006f26c keys every effect on at 0x800c5688.

Oracle: Left held 40 vertical blanks on Music, 90 to 70 (the song volume
0x59 to 0x46 one a frame); Down, then Left held 30 on Sfx, 100 to 85, and
sound 5 once on the release; Right on the stereo item, 0 to 1, again, 1 to
0. Right held on Music from 90 stops at 100; Triangle then gives sound 1
and 90 again.

Cross, Start or Triangle end the menu that frame (0x800428ac): the music
volume (kept or restored) goes to the song again, the boombox's target z
returns to 1000 at 50, and every model the menu made is freed
(0x800219d8, 0x80042cbc), the boombox too. There is no slide out. It
returns the result (sp+0x198), which the title ignores: it slides its
three items in again (0x8003d5a0), Audio still chosen.

The menu does not touch the card. The values stay in the game state,
which a save writes whole (in a record, +0x2c, +0x30, +0x34). 0x80070c24,
called from show_still (0x80030010) while the boot stills show, tries
both cards and copies the three back from the last record used
(0x800702d8). The mono flag is read by the positional sounds (0x8006f4c0,
0x8006f644, 0x8006f88c): with it set they get one volume for both sides
(0x8006fd68), otherwise a pan (0x8006fc40).

The positional sounds (0x8006f644(sound, position)):

- The listener is the camera (0x800de460's position, copied to 0x800e3498
  each frame by 0x8006f88c) and its heading, the bearing of its forward
  axis (0x800c4be4).
- Out of range when |dx| or |dz| exceeds 0xfffe or |dy| reaches 0x7fff.
- Distance d = dx^2 >> 14 + dy^2 >> 12 + dz^2 >> 14 (0x800c435c = 14).
- 0x8006fd68 with the table at 0x800b8024 (near 128, far 16384, max
  127): 127 under 128, 0 past 16384, else 127 - (d - 128) / 128; then no
  more than the effects setting (+0x10), and not below 0.
- 0x8006fc40, the angle a = the sound's bearing less the camera's: the
  volume loses (v * 89 >> 8) * max(0, -rcos a) / 4096 (up to about a third
  from behind); then the side away from the sound loses
  (v * 192 >> 8) * |rsin a| / 4096 (up to three quarters). rsin a > 0 (to
  the right) lowers the left. Both clamp to 0-127, as SsUtKeyOn's voll
  and volr.

### The frames

The first 20 frames (0x80042014, until 0x800c3f24, counted by the flip
0x8001d3e8, reaches 20) read no pads. They slide the text in and draw
CANCEL but not ACCEPT. The loop after it (0x80042228) does the pads, the
colours, the six labels, the needles' targets, frame start (0x8001d440),
the backdrop fade, the backdrop (0x80034fdc, the title's two layers), the
heading, CANCEL and ACCEPT (0x80035550, 0x80035674), camera 2 (0x80040010),
the meters and needles (0x80042958(0)), the boombox (0x8003f748), the
speakers and the arrow blocks (0x8003f934), sounds (0x8006f170), text
(0x8002ecd0(1)), the flip (0x8001d3e8(1)) and the controller check
(0x80039720). No sign and no babies.

A backdrop fade starts every 30 frames (0x800c4080) as on the title. The
loop steps it at its top (0x80042294) and again after the counter
(0x800427ec), so while it runs it steps twice a frame. Oracle: the level
went 0x80, 0x70, 0x50, 0x30, 0x10, full, where the title goes down 16 a
frame.

### The text

All spacing 1, white (0xffffff), each over a shadow of 0x010101 drawn
first. Positions are from the screen's centre. Strings come through the
table at 0x800bb4e8.

| item | string | font | start | target | flags, speed | shadow start |
|---|---|---|---|---|---|---|
| heading | 0x800bb634, "Audio" (83) | 1 | x centred (0x8002e144 flag 1: -35), y -180 | y -100 | 4 and 16 (slide y, wobble 4), 12 | (x + 2, -178) |
| Music | 0x800bb638, "Music" (84) | 0 | (280, -26) | x 116 | 2 (slide x), 12 | (282, -24) |
| Sfx | 0x800bb63c, "Sfx" (85) | 0 | (271, 44) | x 128 | 2, 12 | (273, 46) |
| stereo | 109 "STEREO" or 110 "MONO" by the flag | 0 | (40, 140) | y 77 | 4 (slide y), 12 | (42, 142) |

Each shadow has the same target as its text, not 2 more. The +0x24 or
+0x26 of the three lower items hold the slide's length (164, 143, 63), but
no bounce or wobble flag uses them. Seen at rest in the oracle:

- heading (-35, -103), shadow (-33, -103): the wobble leaves both 3 above
  the target;
- Music (116, -26), shadow (116, -24): 2 below only;
- Sfx (128, 44), shadow (129, 46): the loop slides a shadow only while its
  text's flag is set (0x80042270 tests the text's +0x18). The Sfx text,
  2 pixels nearer, arrives a frame first, so its shadow stops a pixel
  short with flag 2 still set;
- STEREO (40, 77), shadow (42, 77).

The stereo item's x is 40 at the start whatever the flag, so a menu
entered with mono shows MONO at 40 (oracle). A flip sets the string anew
and puts the text at (40 for STEREO or 60 for MONO, 75), the shadow at
(x + 2, 77), with no flags: the text sits 2 higher than before.

The chosen item is yellow (0x05ffff), the others white (0x80042678); the
heading never changes. The loop draws the three shadows, the three items,
then after the backdrop the heading's shadow and the heading.

### The models

All from database 0 (0x80044b74), drawn with the second camera (0x800e2580,
0x80040010), their matrix from their angles (flag 0x20, 0x80020ee0). Each
frame 0x8003f7f0 eases the position (+0x18, +0x1c, +0x20) toward its
target (+0xbc, +0xc0, +0xc4) at the speeds (+0xc8, +0xca, +0xcc) and the
three angles (+0x24, +0x26, +0x28) toward theirs (+0x2c, +0x2e, +0x30) at
+0xce. There is no scale: sizes come from depth.

- The boombox: model 335, 0x800c4b1c, made by 0x80035c1c. (-900, 0, 1000)
  to (-400, 0, 900), speeds 50, 50, 10 (the z target and speed set at
  0x80041ff0); the first angle 0 to 2048 at 150. Drawn by 0x8003f748.
  Which model is the boombox comes from the screenshot, with its
  microphone.
- Two meters: model 339 (0x80042b84(0), made by 0x80042958(1)),
  0x800c4b30 and 0x800c4b34. (900, -100, 500) to (174, -125, 500) and
  (900, 100, 500) to (174, 50, 500), speeds 50.
- Two needles: model 340 (0x80042b84(1)), 0x800c4b50 (Music) and
  0x800c4b54 (Sfx). (719, -52, 200) to (124, -69, 200) and (719, 93, 200)
  to (124, 53, 200), speeds 40, 50, 50. The third angle (+0x28, a turn in
  the screen's plane) starts at 1023. Each frame (0x80042014, 0x80042724)
  its target is 21 x volume - 1024, clamped to -1024..1024, reached at
  128 a frame. 0 points left, about 49 up, 98 and over right. Oracle: 70
  gave 446, 85 gave 761.
- The meters and needles are drawn by 0x80042d04, not the usual block
  draw. It sets the back colour from +0x3a (guess: 0x8009fd18 as
  SetBackColor) and stores its last argument at 0x800de3d4: 12 for the
  meters, 0 for the needles (guess: a depth bias that keeps the needles
  in front).
- Two arrow blocks: model 334 (0x8003f444(0)), sp+0x180 and sp+0x184.
  (0, 1000, 500) rising to y 220 and to y 150 at 50; angles (1024, 0,
  1024) and (1024, 2048, 1024), turn speed 0. The upper one (y 150) shows
  the up arrow (screenshot).
- Two speakers: model 343 (0x8003f444(3)), 0x800c4b00 and 0x800c4b04,
  state +0xd2 2. Both at (600, 1800, 4000) rising to y 740 at 150 (z
  speed 50, turn 128). Their first angle comes from the pair at
  0x800c4124: stereo 1536 and 2662, two speakers back to back; mono (the
  pair at 0x800c4128) 2048 and 2048, one face on. The start always uses
  the stereo pair, even with mono set (oracle); a flip (0x8004241c) sets
  the targets from the flag.
- The CANCEL and ACCEPT cubes are the title's: 0x8003ffe8 zeroes their
  angles at the start and they turn back.

### The sounds

Only 1 to 5, through 0x8006f26c (table 0x800b84fc: program, tone, note):
1 (0, 4, 28) cancel, 2 (0, 1, 25) accept, 3 (0, 2, 26) down, 4 (0, 3, 27)
up, 5 (0, 13, 37) the effects test.

## In the port

`rugrats::front::Title` runs the loop a frame at a time, sampling the pads
at each frame's end as the original does. It has the moves, the
shoulder-button sound, the items sliding away before an action (down to
150 at 8 a frame, the second and third waiting 5 and 10 frames), and the
countdown. The actions (the Start Game menu, Load Game, Audio) are not
ported yet: the port logs them and brings the items back, as if the
action had returned nothing. Sounds are recorded by number, not played.
Idle or Triangle ends the title, and the program runs the chooser's cycle:
a demo movie, the opening, the title again.

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

## The babies (0x80035d44, 0x80043098, 0x80042f34)

Two character objects (0x800c4b20, 0x800c4b24), each holding one of five
hierarchy copies (0x800e2440: DB00 hierarchies 1-5). Their parts' quaternions
stay zero, which 0x8001fc48 turns into the identity, so they are at rest,
turned by their base angles (+0x24; 0x80020d10, the transpose of 0x80020ee0's
matrix) and drawn with the second camera at z 4000. Each frame, in the menu's
draw: the first's AI (0x80043098), then its move (0x80042f34), then the
second's.

The AI, by state (+0x5b2):

- 0: corner = rand() & 7. Below 5, it takes a free corner (re-drawing
  rand() % 5 while the corner's bit in 0x800c4090 is set) and a free baby
  (rand() % 5 against 0x800c4094) and marks both. Angles (2176, 64, 512)
  with targets (2048, 64, z) by corner; speeds 80, 80; turn step 1024.
  Corners 0-3 start at (+/-3000, +/-2500) and head for (+/-1800, +/-1300)
  with z turns -1536, 1536, 512, -512. Corners 4-7 go to state 7.
- 1: speeds less 2 a frame; arrived: 2.
- 2: a giggle (0x80070064, two rand calls); back out from the inner point
  to the outer; speeds 0; 3.
- 3: speeds plus 2 a frame; arrived: 0, the corner and baby freed.
- 4-6: a lean (the y angle to -2304 and back), set from outside.
- 7: sitting out: the corner number counts to 31, then 0. What it marked
  stays marked.
- 8-9: leaving, set when the menu goes (Start), speeds plus 4 a frame.

The move eases each angle toward its target by the turn step, a whole turn
reading as none, and each coordinate by its speed, never past the target.

rand is drawn once before the title's first frame, by an object the front
end makes (0x80048a44, the call at 0x80048b78). From there the port's rand
and babies match the original's frame by frame (test, 150 frames; 860
compared by hand).

## Unknown

- The submenus, the sounds' ids.
- Whether MatrixNormal (0x8009f398) changes the cubes' matrices.
