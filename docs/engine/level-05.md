---
title: Level 5, Visitors from Outer Space
status: partial
discs: US
covers: US SLUS_006.50:0x8007b214, US SLUS_006.50:0x8007b280, US SLUS_006.50:0x8007b3d0, US SLUS_006.50:0x8007b4a0, US SLUS_006.50:0x8007b624, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80018598, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80083b4c, US SLUS_006.50:0x80084084, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005f4cc, US SLUS_006.50:0x80059d3c, US SLUS_006.50:0x80091834, US SLUS_006.50:0x800908e8, US SLUS_006.50:0x80090fac, US SLUS_006.50:0x8004845c, US SLUS_006.50:0x80051d44, US SLUS_006.50:0x8004e7fc, US SLUS_006.50:0x8004d078, US SLUS_006.50:0x8004d0b0, US SLUS_006.50:0x80048804, US SLUS_006.50:0x8004ee00, US SLUS_006.50:0x8004ed00, US SLUS_006.50:0x8004f228, US SLUS_006.50:0x800627a0, US SLUS_006.50:0x800487b8, US SLUS_006.50:0x8005db28, US SLUS_006.50:0x800782d4, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x80025c64, US SLUS_006.50:0x80025e90, US SLUS_006.50:0x80026ddc, US SLUS_006.50:0x80029194, US SLUS_006.50:0x80080ba8, US SLUS_006.50:0x8005ff88, US SLUS_006.50:0x80078e3c, US SLUS_006.50:0x8008c7f0, US /DATA/DB05/DB05LP.BIN, US /DATA/DB05/LEVEL05.BIN, US /DATA/HINTS
worklog:
---

# Level 5, Visitors from Outer Space

Everything level 5 needs to be played to its end: Angelica as the
player, the TV remote that zaps, the TV robots, the batteries, the
sliding doors, the elevator down to the lower deck, the float through the
air ducts and the win in the alien garage. Read from the pseudo-code and
the disassembly; the start, a zap, a robot's bite and a death, a battery,
the elevator ride, the three phases, the float, the win and the last-try
loss were run in the oracle (see Checked in the oracle). Builds on
[level 4](level-04.md), [level 3](level-03.md) and [level 2](level-02.md)
(NPC modes, posts, events, ForceCry, cines, cards, spawn groups, movers),
[gameplay](gameplay.md) and [objectives](objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). "char" is the character (+0x15c =
template +0x10). A VSync is 1/60 s; the oracle's frame counter counts
VSyncs. Heading 0 faces +z, 1024 faces +x (seen in the oracle).

## The goal in one paragraph

Angelica (char 3, 0x80018598) is on the alien spaceship. Cine 37 "Space
Case Intro", then card HINT16 (file 687): guide Angelica to a
shuttlecraft, zap the TV robots with Triangle, collect batteries to
recharge the remote, find the elevator. At the end of cine 37 she holds
the TV remote (id 115) with 120 charge; each zap costs 3, a battery gives
30. She starts on the upper deck (rooms 0-16, 41, 43, y about -5590).
Fourteen TV robots (char 35) guard their spots, chase her within 8000 of
home and bite for 42; a zap stuns one for 20 s. Sliding doors open as
she comes near. The elevator (id 1969) in room 15 takes her down when she
lands on it; room 42 at the bottom is phase 2 (the lower deck, rooms
17-29, 39, 51-59, y about 6700). Entering room 30 plays cine 38 "Space
Case Fish" and card HINT22 (693): she now floats through the air ducts
(player state 21; Cross rises, Circle sinks), with one try left (phase
3). Entering room 38, the alien garage, plays cine 40 "Space Case Ending"
and wins the level. The words come from HINT16 and HINT22.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 0 until cine 37 ends, 1 upper deck, 2 lower deck (room 42), 3 the float (cine 38) |
| 0x800e5830 | s32[3] | respawn position for ForceCry (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| 0x800c4df8 | ptr | the remote's charge gauge (0x8008c7f0) |
| progress +0x23 (0x800c569b) | u8 | tries; set to 1 by 0x8007b4a0 (phase 3) |
| player +0x18c | u8 | 21 while floating (0x8005db28) |
| player +0x65c | s32 | the float's depth anchor; the camera aims at it in state 21 |
| player +0x660 | s16 | the float's bob phase |
| 0x800c4a50 | s32 | camera hold flag for the float (below) |
| 0x800c4170 | u16 | next robot id, 13000 at load (0x800454dc) |

The phase is only written through 0x80078d60: 1 by 0x8007b280 (cine
37's end), 2 by 0x8007b3d0 (room 42), 3 by 0x8007b4a0 (cine 38's end).
EpisodeStart sets none. There are no CheckPoint rooms in DB05 (no kind 6
record has byte +4 set) and no Gameplay scripts (no kind 6 script byte).
Level 5 has no TimerExpired callback (0x8007f5a8 table 0x800158cc has no
level 5 case) and no level timer.

## Data in DB05

### The player and the characters (LP kind 1)

Level 5's playable character is 3, Angelica (0x80018598 case 5). The
loader (0x800454dc) moves the player onto the first kind 1 record whose
template's shape is the character (state +0x08, 0x800c5680), when called
with a1 = 0 (the runner, 0x80017c0c). That is LP 14 (template 1, id 600).
Angelica has the normal turn rate (0x80062eac: slow only for chars 6 and
21) and the camera aims 768 above her (0x80025e90: 400 only for char 6).

| LP | template | char | id | position | what |
|---|---|---|---|---|---|
| 14 | 1 | 3 | 600 | (47842, -5812, 31997) | Angelica, the player |
| 106 | 2 | 2 | 1776 | (-1088, -5812, 34024) | Chuckie (guess: cine 37 actor) |
| 107 | 11 | 1 | 1492 | (-4050, -5812, 29217) | Tommy (guess: cine 37 actor) |
| 108, 111 | 6, 7 | 4, 5 | 609, 608 | x -5084, 4065 | babies (guess: cine 37 actors) |
| 109, 110, 112, 113 | 16, 17, 19, 18 | 17, 18, 20, 19 | 130, 132, 134, 131 | x about 0 | aliens (guess: cine actors) |
| 115 | 831 | 36 | 142 | (86414, 5288, 52649), room 30 | (guess: cine 38 actor) |
| 114, 116, 117, 176-186 | 826 | 35 | 13000-13013 | see below | the TV robots |

LP 106-113 stand outside every room (x below 8192). The end of cine 37
hides 1776, 608, 609, 1492, 130, 132, 131, 134, 514 and 524 (0x800732d4
case 37); the end of cine 38 hides 131, 142, 134 (case 38). Chars 1-4,
17-20 and 36 get nothing from 0x80081bac.

### The TV robots (template 826, char 35)

Template 826: health 255, range 8000 (+0x1a), speed index 2 (8), +0x32 =
42 (bite), flags 0x10002001. The generic init gives each a running id
(0x8004691c level 5 case, 0x80046e90: +0x38 = 0x800c4170++, from 13000)
in LP order, overwriting the LP id. 0x80081bac case 33 (char 35) then
tests +0x38 against 2000-2002: never true here, so every robot gets
0x80083b4c (mode 11, guard, target the player, +0x688 |= 2), +0x690 =
1536 (attack reach), enter callback +0x48 = 0x80049e18, +0x688 |= 8, and
template flag 8 (no object collision). Template flag 0x10000000 adds
+0x688 0x2000. Seen in the oracle: mode 11, +0x688 0x200a, range 8000,
+0x6d 42.

| id | LP | home | room |
|---|---|---|---|
| 13000 | 114 | (25841, -7824, 16110) | 11 |
| 13001 | 116 | (35128, -7824, 63550) | 1 |
| 13002 | 117 (LP id 2000) | (71403, -7824, 7737) | 41, by the elevator |
| 13003-13009 | 176-182 | y -7824, rooms 1, 4, 2, 3, 14, 9, 8 | upper deck |
| 13010-13013 | 183-186 | (96613, 5155, 8575), (82862, 5155, 8766), (99853, 5155, 36440), (91139, 5155, 29227) | lower deck |

The robots hover (seen y -8000 to -8600 above a floor at -5590). Only LP
ids are registered by the loader for characters (0x8004584c: LP +0x2a,
here 0 for most robots and 2000 for two), so 0x800461f4(13000..) finds
nothing: the disarm loops of 0x8007b3d0 and 0x8007b4a0 do nothing (seen:
no 0x80049e5c call from them). 0x800461f4(2000) returns LP 117's robot.

### The remote (template 828, id 115)

LP 48 at (37672, -5193, 32020). Template 828 is class 13 (0x80049c04:
generic init, object +0x00 |= 0x1000000), pickable (+0x00 0x10000),
+0x32 = 120 (charge, +0x6d), +0x36 = 3 (cost a zap, +0xa2), trigger
15000. Template 933 is the same kind of object in level 12 (the
flashlight, see [objectives](objectives.md)).

### Collectibles and spawn groups (LP kind 7)

30 kind 7 records (0x800c4bac): groups 0-6 one spot each (threshold 100),
group 7 sixteen spots, group 8 seven spots.

| LP | template | kind (+0x26) | +0x6d | copies | group | what |
|---|---|---|---|---|---|---|
| 221, 272-277 | 835 | 3 | | 1 each | 0-6 | Reptar bars |
| 169 | 87 | 12 | 30 | 16 | 7 | batteries |
| 222 | 863 | 4 | 63 | 7 | 8 | health (guess: food, kind 4 heals, see level 4) |

All 16 battery spots get a battery. Seen placed at load (0x8004fe14 calls
(7, 16), (0, 1), (8, 7), (1..6, 1)): batteries at (32483, -5936, 26015),
(48282, -5838, 1391), (9228, -5846, 38313), (47097, -5850, 65366) and 12
more, up to y 6735 on the lower deck; they float about 300-450 above the
floor (Angelica jumps for them).

### Sliding doors (template 843, class 3)

16 records (LP 135-150), shape 1189, y -7637 on the upper deck and 4658 on
the lower. Class 3 is 0x8004965c -> 0x8004845c. For template 843 (except
an id 8801, which no 843 has) it sets enter +0x48 = 0x8004d078, think
+0x44 = 0x8004e7fc, leave +0x4c = 0x8004d0b0, use +0x54 = 0. Template:
step (0, -256, 0) (+0xb0..+0xb4), travel 5120 (+0xb6), trigger 5000
(+0xa0), touch +0x58 = 0x80051d44 (template +0x0a 11).

- Enter (the trigger pass, 2D distance max + min / 2 under 5000,
  0x80072d38): 0x8004d078: sound 71 at the door, then 0x80051d44.
- 0x80051d44 (also what a zap calls): unless already moving (+0xcc), think
  = 0x8004e7fc, scheduled every frame, +0x00 |= 1.
- 0x8004e7fc: offsets -= 2 x step, position = home - offsets (so up 512 a
  frame); when the travel reaches 5120 it sets +0x3c bit 2 and leaves its
  rooms (0x80078b00). Seen: -7637 to -8661 in 2 frames.
- Leave (out of 5000): 0x8004d0b0: sound 71, think 0x8004e908 (guess:
  slides back).

A door (843) first in a robot's attack ray stops the bite (mode 11 in
0x80084084, with template 1177).
Id 8800 (LP 143 at (91642, 4658, 41408), room 29) is the door to the tank
rooms; cine 38 handles it (below).

### The elevator (template 855, id 1969)

LP 155 at (65540, -4578, 8708), room 15 (the shaft top), speech line 641.
Class 11 is a mover (0x80049ab0 -> 0x80048804). Template 855: step (0,
50, 0), range (0, 12288, 0), flags 0x40000002; level 5's init sets the
step to 60 (0x80046ed8: template +0x1a = 60). Flag 0x40000000: no enter
callback, +0xbe |= 64 (moves only while ridden); 0x80048804 adds +0xbe |=
256 for template 855. Its collision box is (-2560, -512, -2048)..(2560,
512, 2048).

- Standing on it: the floor query 0x800627a0 sees player +0xf8 = a class
  10 object (+0xa6) with +0xbe bit 64 and no think: 0x8004f228 (schedule
  the think, flip the directions). Teleporting onto it does not set +0xf8;
  landing from a jump does (seen).
- The think 0x8004ee00 moves y by 60 a think (30 a VSync) toward +0xaa =
  12288 and turns at the ends. A level 5 case (0x8004efdc): for template
  855, when the rider is off it and 0x8004ed00 returns 1, y moves only
  while +0xaa is at least half the range (guess: it does not come back up
  past the middle).
- 0x8004ed00 (movers with +0xbe bit 256), rider off and 0x8004ec34 0:
  other movers turn toward the player's height (0x8004ed78); template 855
  in level 5 just returns 1 (0x8004ed58).
- Seen: armed at 3642, from y -4578 down to about 7710 at 4052 (410
  VSyncs), Angelica at y 6738 on it; she walked off into room 42.
- ForceCry before phase 2 resets it (0x8007b624, below).

### Other objects

| LP | template | class | id | what |
|---|---|---|---|---|
| 292 | 1231 | 0 | 524 | registered as 524 by the level 5 init (0x80046ebc), also D_800c4168 (only golf reads it); hidden at cine 37's end |
| 296 | 1268 | 0 | 8801 | at door 8800's spot; hidden at cine 37's end, shown at cine 38's end with a box (guess: the door shut behind her) |
| 38 | 97 | 2 | | pickable (shape 725), speech line 617; no level code |
| 173, 174, 231-234 | 1139 | 10 | | timed scenery (0x80048400), not drawn, sound 50; LP 173 speech 1056 |
| 223-226 | 1156 | 11 | | movers: y step -32, range -1024 (up and back), armed by the trigger (15000) |
| 219 | 1040 | 11 | | a mover with y step 40 and only a z range (guess: does not move) |
| 220, 293-295 | 1041, 1267 | 11 | | never loaded: the loader skips templates 1242, 1041, 1267, 834, 382, 625 (0x800455d0) |
| 190, 191 | 866, 867 | 3 | | class 3 with an angle step of 16 (guess: spinning decoration) |
| 218, 235 | 968, 827 | 0 | 501, 525 | no level code (guess: cine props) |

### Rooms (LEVEL05.BIN, LP kind 6)

Room boxes read from RAM in the oracle (room +0x10..+0x28). Kind 6 bytes
from +4: room, cine (flag 2, +0x5b), fixed camera post (+0x5c), camera
kind (flag 4, +0x5d), checkpoint, flag 16, script, song (+0x64).

| room | x | y | z | kind 6 | role |
|---|---|---|---|---|---|
| 0-14, 45-50 | 8192..74752 | -11264..-5120 | -2048..67584 | 0 song 1 | the upper deck; Angelica starts in room 0 |
| 15 | 63488..67584 | -11264..-4096 | 6144..11264 | camera 3 (post 4) | the elevator's top |
| 40 | 63488..67584 | -4096..8192 | 6144..11264 | camera 3 (post 4) | the shaft |
| 41, 43 | 67584..73728 | -11264..-5120 | 4096..22528 | camera 1; 41 song 1 | beside the shaft top; robot 13002 |
| 42 | 67584..74752 | 1024..8192 | 6144..11264 | camera 1, song 2 | the shaft's foot: phase 2 (0x8007b3d0) |
| 17-29, 39, 51-59 | 74752..105472 | 0..7168 | -14336..41984 | 19 song 2 | the lower deck |
| 44 | 87040..101376 | -5120..7168 | 41984..49152 | song 1 | before the tank |
| 30 | 86016..101376 | -5120..7168 | 49152..63488 | cine 38, song 1 | cine 38; the float starts here |
| 32, 35, 60-75 | 74752..99328 | -10240..0 | 57344..75776 | 32 camera 8 song 3; 60, 62, 68, 70-72 camera 3; 61, 64-67 camera 8; 73-75 camera 7 | the air ducts |
| 33, 34, 36, 37 | 51200..78848 | -11264..-5120 | 49152..61440 | 33 camera 8 | ducts toward the garage |
| 38 | 52224..73728 | -11264..1024 | 54272..77824 | cine 40 | the alien garage: the win |

Fixed camera posts (LP kind 2, flag 8): posts 1-9 (y -8616..-1824, around
the ducts) are cameras; posts 10-15 are two small loops (10-11-12,
13-15-14) no robot uses (robots are in mode 11). EpisodeStart's room
sound bytes (0x80078e3c(0, 2), (30, 1), (44, 1)) change nothing: those
rooms already have songs (0x80078e3c only fills an empty +0x64).

## Start

EpisodeStart (0x8007b214, installed by 0x8007871c case 5):

1. 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 712): follow camera,
   0x800c3f5c = 1024 (level below 8, not 4 or 7), field of view 712
   (0x8001dfb8; the default is 612).
2. Room sound bytes 0x80078e3c(0, 2), (30, 1), (44, 1) (no effect).

The level start: player init on LP 14, cine 37 "Space Case Intro"
(0x80074958; no start hook in 0x80072ef0). Its end hook hides the cine
actors (above). CineFinish shows card 687 (HINT16, arg 6) (0x800783d8
case 36). Then the end handler 0x8007737c case 36 runs 0x8007b280:

1. Hide id 8801 (0x80078bc4).
2. The player's current animation slot (+0x154 + 40 x hier +0x23, +0x58)
   = animation 1209 (0x80044b74; guess: standing with the remote; level
   12's flashlight uses 1690 the same way and 1310 to undo it).
3. The remote (id 115): out of its rooms (0x80078b00), player +0x1f0 =
   it, 0x8005ff88(player, 1) (to bone 15, the hand), its draw +0x40 = 0
   (guess: drawn with her).
4. The gauge: 0x8008c7f0(360, scale (36, 64, 1024), pos, 361, scale (36,
   54, 1024), pos, max = the remote's +0x6d (120), 1), pos = (((0x800c4a04
   x 5) >> 7) - 192, 88 - (0x800c4a06 >> 4)); kept in 0x800c4df8. Seen:
   the battery icon bottom left.
5. Respawn = the player's position and angles, phase 1.

Seen: Angelica at (38395, -5590, 28281), heading 1895, room 0, tries 3,
+0x30 0x100.

## The remote and the zap

In the player update (0x8005a068, at 0x8005a6f4): carrying an object with
+0x00 bit 0x1000000 that is not template 933, Triangle pressed
(0x800c49d4 `& 3 == 1`): animation 39 (blend 0x2000), +0xe0 |= 2048, and
0x8005f4cc, which with bit 2048 calls 0x80059d3c(player, the remote's
position, 2048) and clears the bit.

0x80059d3c: with charge (+0x6d) above 0 and template 828:

1. The matrix of bone 15 (D_800bb2b6, 0x80044fec).
2. 0x80091834(player, matrix, player +0x200, 2048): beam type 0 from the
   bone (offset D_800bb2b0), direction from player +0x200; speed (+0x44)
   = table 0x800bb28c[0] +4 (10) + 2048; player +0x00 |= 8.
3. Sound 68 at the player (0x8006f644).
4. Charge -= +0xa2 (3); gauge = 0x8008ca80(gauge, charge).

With 0 charge nothing fires (the animation still plays).

The beam (pool of 8, 92 bytes each at 0x800e6ef0, 0x80090298; update
0x80090fac): moves along its direction each frame and dies when its life
(+0x32, from the table) runs out (0x80090280). It tests the room geometry
and the room object lists (0x80090554, 0x80090340):

- The first wall hit (+0x2e = 0): reflects when 0x8007666c allows (result
  1025 or more), effects 0x80090acc and 0x80090c0c, sound 70. Later wall
  hits do not reflect.
- An object with a +0x58 callback: effects, then +0x58(object, hit
  point), unless it is a character (+0x00 bit 0x4000) with health 0 or
  less; the beam is done (+0x00 |= 4).

Robot hit (+0x58 = 0x800511a8, char 35, case 33): sound 63 at the robot,
animation 48 (blend 2048), mode 8, state 19, +0x69c = 6000, +0x688 |= 32.
Mode 8 with bit 32 counts +0x69c down by dt (0x80084fc4); at 0 it goes
back to +0x69b (11), state 0, its speed back, sound 65 for char 35. So a
zap stuns a robot for 20 s; another zap restarts the 20 s. Health is not
touched. A door hit: 0x80051d44 (it opens).

Battery (kind 12, touch 0x800514b0, at 0x80051c68): sound 69 (109 in
level 12), carried +0x6d += the battery's +0x6d (30), then = 0x8008ca80
(gauge, value), which clamps it (seen 10 -> 40).

The remote can be put down and picked up with Circle like any pickable
(guess from the generic code; not checked). Nothing level 5 specific
handles it on the ground.

## TV robots: mode 11 (0x80084084)

Mode 11 as for the geese of [level 2](level-02.md#modes-used-in-level-2):
with the player within range (8000) of home and |dy| under 2800, chase;
attack (mode 5) when the ray 0x80090554 toward the player first meets an
object (not a wall, not nothing) that is not template 1177 or 843, the
player is within +0x690 (1536) and heights overlap (0x80083ff0); else
(bit 2) go home. Speed index 2: 8. Mode 5:
state 19, animation 37, Damage(player, +0x678, +0x6d = 42) (0x800780ac),
then mode 1. The attacking robot is kept in 0x800c443c for the ambience.

Seen: a robot 1400 from Angelica bit at 3528 and 3574 (46 VSyncs apart),
42 each. One 4000 away crept up (x +80 every 20 VSyncs, stops and
starts), stopped about 2500 away and never bit (guess: something in the
ray's way).

Damage (0x80077864) has no level 5 routing. 255 health is six bites.

## Phase 2: room 42 (0x8007b3d0)

Level 5's room change (0x8007f498, table 0x80015894 index 2):

- Room 42, every entry: 0x8007b3d0: respawn (73246, 6697, 9170) (words
  0x800156d8), angles (1015, 0, 0), phase 2; then for ids 13000-13009
  except 13002: disarm (0x80049e5c) and off the trigger list (0x8001a810)
  (finds nothing, above).
- Room 38, every entry: player +0x18c = 0, 0x8008a7ec(-1) (the animation
  slots back to the character's own).

The lower deck has robots 13010-13013, batteries, the health pickups and
door 8800 to the tank (room 29 -> 44 -> 30).

## Phase 3: the float (cine 38, 0x8007b4a0)

Room 30 has cine 38 (flag 2): RoomChange posts Cine(38) (0x80077aa8 ->
0x80078350). Its start hook (0x80072ef0 case 38) hides id 8800. Its end
hook (0x800732d4 case 38) hides 131, 142, 134 and shows 8801
(0x80078c68). CineFinish: card 693 (HINT22, arg 4) (0x800783d8 case 37).
0x8007737c case 37:

1. Player +0x18c = 21 (the float).
2. 0x8007b4a0: door 8800 off the trigger list, +0x00 bit 1 cleared,
   hidden; id 8801's box = (-2500, -4000, -1000)..(2500, 3000, 1000);
   the player at (92984, -27, 57036) (+0x65c = -27), heading 441; respawn
   here, phase 3; tries = 1 (0x800c569b); ids 13010-13013 disarmed (finds
   nothing).
3. Camera reset (0x800290a4), +0x188 &= ~128, 0x8006eac4(2, 0) (song 2,
   the call the room songs use).

### Player state 21 (0x8005db28)

- In room 28: PostForceCry (0x800782d4) and nothing else.
- Triangle: carrying something without bit 0x1000000: throw request; else
  use the touched object (state 12) as when standing.
- Moves with the stick (0x800590d4 run / 0x80059008 walk, animations 2 /
  1, else 1 at idle); the ground speed is halved each frame before
  0x8005caac; the move is 0x8005d978; no gravity.
- Not AI: bob phase +0x660 += (218 x dt) >> 6 (34 a frame, mod 4096).
  - Cross (JUMP, 0x800c4994) held: dy = -((262 x dt) >> 6) (-40 a frame),
    anchor +0x65c = y.
  - Circle (GRAB, 0x800c4988) held: dy = (131 x dt) >> 5 (+40 a frame),
    anchor = y.
  - Else, with the stick and speed: dy = 0; at rest: dy = anchor +
    (sin(+0x660) x 256 >> 12) - y (a bob of 256 around the anchor).
- Touching a collectible (+0xa6 11) calls its +0x58 (0x8005de74), so
  batteries and bars still work.

Seen: Cross held 40 VSyncs: y 153 to -647 (-80 per 4 VSyncs); Circle: up
again by 80 per 4 VSyncs; then the bob.

### Camera in level 5

CameraSwitch (0x80077c88) maps the room's camera kind to 0x800291f8
arguments (table 0x80015484): 1 follow 1500; 2, 3, 4 kinds 2-4; 5 kind 5;
6 follow 200; 7 follow 500; 8 follow 1600; 9 follow 2000; 10 follow 1500
with 2000; 11 follow 500 with 952; 13 follow 1500 with -1. The height
(fourth argument) goes to 0x800c3f5c. Level 5 cases:

- The "same kind as last time" skip (0x800c4410) is not applied in state
  21.
- In state 21, kind 7 becomes 13: 0x800291f8 kind 1 with -1 only sets
  0x800c4a50 = 1 (the camera stays where it is). Rooms 73-75.
- Kind 3 (0x800291f8 case 2, level 5 at 0x800293c0): not floating:
  0x80025c64(camera, player, 0, 13): camera +0x4a = 12 (types 12 and 13;
  else 5), eye offset (0, -(0x800c3f58 + 0x800c3f5c), 5800), target offset
  (0, -0x800c3f58, 0), 0x800c4a3c = 0x20000 (rooms 15 and 40, the shaft). Floating: only when 0x800c4a50 = 1, the fixed
  camera 0x800259a0(camera, player, 0x80029194(room)): rooms 60, 62, 68,
  70, 71, 72 -> posts 5, 3, 8, 2, 4, 6; others 1.
- 0x80025e90 (follow setup) in state 21: 0x800c4a50 = 0; with a3 = 0
  keeps the camera position (0x800c4a3c |= 0x2000); the goal's y is
  +0x65c, not the player's y.
- R1 (0x80026ddc with 0): ignored in state 21 while 0x800c4a50 = 1.

## Winning: room 38

Room 38 has cine 40 (flag 2). Entering it: room change case (state 0,
0x8008a7ec(-1)), PostCine(40). Cine 40 "Space Case Ending" has no hooks
and no card. After it (0x8007737c case 39): exit code 0x800c4918 = 1,
0x80076d6c(5, 1) (level 5 played and won, its puzzle piece, the
main-level count; see [objectives](objectives.md)). The port's `ENDINGS`
already has 40. Seen: cine 40 from 5148 to 5672, then 0x80094bfc(5) and
the puzzle screen (0x8009480c).

## ForceCry (0x8007fd0c, level 5 case)

Case 4 (0x8007ff18), when a respawn is set:

1. Cry (0x8007f920) always.
2. Tries (+0x23) counted down.
3. Health 255; then 0x8007b624: in phases 0-1, the elevator (1969) is
   disarmed (0x80049e5c) and reset to its start (0x800487b8).
4. Tries left: to the respawn (0x8007fbf8). None: SwitchEpisode(player,
   1) (0x800781f8, the hub); no level record, no cine.

Respawns: phase 1 where cine 37 left her; phase 2 (73246, 6697, 9170);
phase 3 (92984, -27, 57036) with one try, so the first cry in the float
ends the level.

What hurts: robot bites (42). Nothing else in level 5 code; falls: the
shaft has no fall (she rides the elevator).

## How the level ends

- Won: room 38, cine 40, 0x80076d6c(5, 1), exit code 1.
- Lost: the last try (ForceCry), SwitchEpisode(1). In phase 3 that is the
  first cry.

## Hint cards

| card | file | shown | text (abridged) |
|---|---|---|---|
| HINT16 | 687 | after cine 37 (arg 6) | Guide Angelica through the alien spaceship to a shuttlecraft; zap the TV robots with @; collect batteries to recharge the remote; search for the elevator |
| HINT22 | 693 | after cine 38 (arg 4) | Float through the twisting air ducts to the alien garage; % to float higher, # to float lower |
| HINT17-21 | 688-692 | no caller found | "Alien baby's room", "Alien parent's room", "Holding area", "Control area", "This way to the elevator!" |

HINT22 confirms % = Cross and # = Circle (the guess in
[objectives](objectives.md)). The port's card table (front.rs
card_after) has 37 -> 687 and 38 -> 693.

## Per-frame ambience (0x80080ba8)

0x800b9618[5]. Countdown 0x800c4cc0 minus dt:

- A robot attacking (0x800c443c) within 5000 (+0x674) and the countdown
  run out: speech 0x800b9438[rand & 3] (522, 585, 621, 629), countdown
  (rand & 255) x 8.
- Else at 0: floating: 0x800b940c[(rand & 255) mod 5] (545, 546, 637, 638,
  639); else 0x800b9418[rand & 7] (684, 619, 663, 729, 731, 732, 641,
  640); countdown 900 + (rand & 255) x 16.

0x800c443c is cleared each call. Seen: speech 641, 619, 663 every 430-900
VSyncs.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, 16-bit pokes, hooks with
arguments and memory or object dumps, call redirects, screenshots):
presses start 300, cross 500, 650, 800, 1650, 1800; main's level argument
patched (0x80018824 = `li a0, 5`), 0x800c5678 = 5 at the level runner,
cards skipped (0x8004472c = 0x1000004a).

The character needs more than level 4's poke. With only 0x800c5680 = 3
poked, the loader still saw char 1 (main writes it in the same frame), so
the player took Tommy's record (LP 107, id 1492) and LP 14 became an NPC;
cine 37's end hook then hid id 1492, the player, whose room became -1,
and the room clamp (0x8005b018) threw it to (67110912, 4065280, 0). Making
0x80018598 return 3 fixes it: the player is LP 14, id 600. Skipping the
level-start cine (0x80074958 returns 0) hung level 5 after the loading
screen, so these runs play cine 37.

- Cine 37 from 1819 to 3477; at 3477 CineFinish (card 687, arg 6),
  EpisodeStart (0x800291f8 kind 1, 1500), 0x8007737c(37) and 0x8007b280:
  remote in hand, gauge, phase 1, respawn (38395, -5590, 28281), tries 3.
- Robots: only one armed near the start (13000, mode 11, +0x688 0x200a,
  range 8000, +0x6d 42, think 0x80084084). Angelica 1400 from it with
  health 50: Damage(player, player, 42) at 3528 and 3574, ForceCry and
  the cry at 3576, at 3996 0x8007b624 (elevator disarmed and reset) and
  the respawn at (38395, -5590, 28281), tries 2, health 255.
- Zap: Triangle at 3560 and every 60 VSyncs, facing the robot (heading
  3072): 0x80059d3c, beam 0x800908e8, 0x800511a8 on the robot each time;
  sounds 68 and 63; charge 120 -> 111 -> 102; the robot in mode 8, state
  19, +0x69c 4870 228 VSyncs after the last zap.
- Battery: charge poked to 10; a jump on the spot (48282, -5838, 1391):
  0x800514b0 at 3546, gauge 40.
- Elevator: a jump on it at (65540, 8708): 0x8004f228 from 0x80062894 at
  3642, the elevator from y -4578 down 30 a VSync to about 7710, Angelica
  riding (+0xf8 = it); walking +x: room 42 at 4198, 0x8007b3d0, phase 2,
  respawn (73246, 6697, 9170). No 0x80049e5c from 0x8007b3d0.
- Room 30: PostCine(38) at 3522, cine 38 from 3548 to 4973, hooks as
  listed, card 693 (arg 4), 0x8007b4a0: state 21, (92984, -27, 57036),
  phase 3, tries 1.
- The float: Cross and Circle held as above.
- Room 38 while floating: 0x8008a7ec(-1), PostCine(40), cine 40 from 5148
  to 5672, 0x80076d6c(5, 1), the puzzle screen.
- Room 28 while floating: PostForceCry, ForceCry and the cry at 5124,
  tries 1 -> 0, at 5544 SwitchEpisode(player, 1), level 1 loaded; no
  0x80076d6c.
- A zap on a door (LP 141 from 3457 away): 0x80051d44, the door rising 512
  a frame; Angelica had been pushed back to 6858 away, so the trigger
  pass then closed it (0x8004d0b0).
- Not checked: a door opening by approach, the remote put down, a phase
  2 respawn, robots 13010-13013, the health pickups (template 863), the
  beam's reflection, movers 1156, the duct cameras.

## For the port

The port has a Script per level, hiding by id, cinematic-object use,
doors, room flags 2/4/32/64/128, the follow camera with heights and kind
3 fixed cameras, NPC modes 0, 1, 3, 4, 6, 7, 9, 11 and riders, posts,
carrying and throwing, hazards, the level timer, gauges and the meter,
ForceCry with tries, the result screens; level 4 adds a player other than
Tommy, spawn groups, COL floor flags, heal on collect (and movers with
riders, `game.rs`). Level 5 needs, in order:

1. Angelica: `player_character(5) = 3` (0x80018598); she starts on LP 14
   (template 1, hierarchy 3) but play starts where cine 37 leaves her
   (38395, -5590, 28281), heading 1895. Normal turn rate, camera aim 768.
2. A `level5` script: phase, respawn, tries; EpisodeStart 0x8007b214 (fov
   712); cine 37's end (0x8007b280); room 42 (phase 2, fixed respawn and
   angle 1015); room 38 (state 0); cine 38's start and end hooks and
   0x8007b4a0 (door 8800 hidden and off the trigger list, 8801 shown and
   solid, the player placed, phase 3, tries = 1); cine 40 won (already in
   `ENDINGS`); ForceCry's case (0x8007b624). Cards are in `card_after`.
3. The remote: a carried object from a script (no pick-up), in the hand
   (bone 15), animation 1209; the charge gauge (models 360 / 361, max
   120, bottom left); Triangle while carrying a class 13 object: animation
   39, sound 68, charge - 3, and a beam.
4. The beam: a projectile pool (8) along the facing; first wall hit
   reflects (sound 70); an object hit calls the object's +0x58. The port's
   throw and flight code is the closest thing (guess: this one is straight
   and fast; speed 10 + 2048 and life from table 0x800bb28c, to measure).
5. TV robots: char 35 in mode 11 (`Mode::Guard`, range 8000, reach 1536,
   bite 42 from template +0x32, speed 8); hit by the beam: a 6000 dt stun
   (`Mode::Hit { left: 6000, then: Guard }`), sound 63, animation 48;
   wake sound 65.
6. Batteries: collectible kind 12 adds its +0x6d (30) to the carried
   object's charge, clamped to the gauge's max, sound 69. Kind 4 heals
   (template 863, 63). Spawn groups 0-8 as above (level 4 has them).
7. Sliding doors (template 843): open on entering 5000 (sound 71, up 512
   a frame to 5120, then gone), close on leaving, open on a beam hit.
   A door first in a robot's attack ray stops the bite.
8. The elevator: the mover must arm only when the player lands on it
   (0x800627a0), step 60 in level 5, and hold at the top half when the
   rider is off (0x8004efdc). Reset on a cry before phase 2.
9. Player state 21, the float (0x8005db28): no gravity, Cross up 40 a
   frame, Circle down 40, a bob of 256 around the anchor, stick movement
   at half speed, room 28 = ForceCry, touch collectibles.
10. The camera rules: kind 3 rooms as mode 13 (0x80025c64) on the deck;
    in the float the kind 7 hold (0x800c4a50), the fixed cameras from
    0x80029194, no same-kind skip, the aim at +0x65c, R1 off; follow
    heights 500 (kind 7) and 1600 (kind 8).
11. Small things: the loader's skipped templates (1041, 1267); robot ids
    13000+ are not registered (lookups by them find nothing); the
    ambience speech (0x80080ba8); movers 1156; scenery 1139 sound 50.

## Unknown

- Where cine 37 leaves the player in the real game. The oracle's
  (38395, -5590, 28281) is with the loader fix above (guess: right).
- Animation 1209 and slot +0x58 of the hierarchy table (guess: the
  standing animation with the remote).
- The beam's speed and life in world units (table 0x800bb28c entry 0:
  +4 10, +0xc 1000, +0x10 297, +0x12 300, +0x14 1000) and what
  0x8007666c's 1025 bound means.
- What happens to the remote if put down and how the gauge follows it.
- Why room 28 makes a floating Angelica cry (it is on the lower deck, far
  from the ducts; guess: a safety net).
- Sound 64 heard 60 VSyncs after each robot hit (source not traced).
- HINT17-21 (688-692): no caller found (guess: unused room signs).
- 8801 (template 1268): what it looks like; whether it blocks the way
  back to the lower deck (guess).
- Who uses posts 10-15 (the robots do not).
- Templates 1139, 1156, 866, 867, 97: what they look like and do.
- Whether 0x8007b3d0 resetting the phase to 2 on a later room 42 entry can
  happen (guess: 8801 stops it).
- How long cines 37, 38 and 40 really are (the oracle: 1658, 1425 and 524
  VSyncs).
