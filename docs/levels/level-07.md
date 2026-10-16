---
title: Level 7, Reptar 2010
status: partial
discs: US
covers: US SLUS_006.50:0x8007b748, US SLUS_006.50:0x80053d28, US SLUS_006.50:0x8005202c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x8005cc2c, US SLUS_006.50:0x8005ce74, US SLUS_006.50:0x8005cd74, US SLUS_006.50:0x8005cba8, US SLUS_006.50:0x8005e1f4, US SLUS_006.50:0x80059008, US SLUS_006.50:0x800590d4, US SLUS_006.50:0x8005be24, US SLUS_006.50:0x80049c50, US SLUS_006.50:0x8004d7c4, US SLUS_006.50:0x8006baa8, US SLUS_006.50:0x8005202c, US SLUS_006.50:0x80053d28, US SLUS_006.50:0x80049e18, US SLUS_006.50:0x80049e5c, US SLUS_006.50:0x80048804, US SLUS_006.50:0x800495d0, US SLUS_006.50:0x80048364, US SLUS_006.50:0x8004e2ac, US SLUS_006.50:0x8004cdfc, US SLUS_006.50:0x80080cd8, US SLUS_006.50:0x80080318, US SLUS_006.50:0x800819a4, US SLUS_006.50:0x80078e3c, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x80074958, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80077864, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x8008b1e4, US SLUS_006.50:0x8008b2c8, US SLUS_006.50:0x8008b448, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80018598, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x80025e90, US SLUS_006.50:0x800290a4, US SLUS_006.50:0x8004d3f0, US /DATA/DB07/DB07LP.BIN, US /DATA/DB07/LEVEL07.BIN, US /DATA/DB07/LEVEL07.COL, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS, US /DATA/OPEN/CREDITS.STR
worklog:
---

# Level 7, Reptar 2010

Everything level 7 needs to be played from its start to the game's end:
the player is Reptar, stomping through a city at night to City Hall,
knocking buildings down on the way. It is the final level, reached once
all twelve puzzle pieces are placed. Reaching City Hall plays cine 85
"Reptar Dance"; the game then clears the progress, plays the credits
movie and starts again in the hub. Read from the pseudo-code and the
disassembly; the start, smashing by walking, the chomp, the fire, the
rumble jump, a street object taken, the walk from the start to City Hall,
cine 85, the credits and the hub after them, and a forced cry were run in
the oracle (see Checked in the oracle). Builds on
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md)
and [screens](../engine/screens.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c, state +0x08 at 0x800c5680). A VSync is
1/60 s; the oracle's frame counter counts VSyncs.

## The goal in one paragraph

The player is Reptar (char 9: 0x80018598 case 7). Cine 79 "Reptar Intro"
plays (the babies at home, then Reptar walking out of City Hall), then
card HINT37 (file 708): stomp through the city to Reptar's goal, City
Hall; run into buildings to knock them down; Triangle chomps, Circle
breathes fire, Cross is a rumble jump. There is no clock, no counter, no
enemy, no collectible to count and no HUD. Buildings (96 class 14
objects) break when Reptar's body meets them. The city's streets do not
link the start plaza to City Hall's plaza directly: the way goes north
and west along an avenue, over a raised strip on the west side, and back
south-east (see The route). Entering room 6 (City Hall) plays cine 85;
its end sets exit -1 and gp+0 (0x800c3e80); the runner's teardown clears
the progress, plays CREDITS.STR and returns level 1, so the game goes on
in the hub with an empty save state. Nothing in level 7 was found that
hurts Reptar; a forced ForceCry ends the level at once (one try) and goes
to the hub with nothing recorded.

## How level 7 is reached

- Placing the twelfth piece on the board: SwitchEpisode(7) (0x80092c68
  step 5; [screens](../engine/screens.md#the-board-0x80092c68)).
- "You did it!" (0x80094978) callers: 0x80076d6c (SwitchEpisode(7)
  unless +0x30 bit 512), the teardown (0x80018354) and the bar pickup
  ([screens](../engine/screens.md#you-did-it-0x80094978)).
- The hub's puzzle board (use 0x8004d3f0): with +0x30 bit 512 set it
  posts SwitchEpisode(player, 7) instead of the puzzle screen
  ([objectives](../engine/objectives.md)). Bit 512 is set by level 7's
  start (below), so after leaving level 7 any other way the board takes
  the player straight back.
- The loading screen: "Reptar 2010" (157) / "Loading"
  ([screens](../engine/screens.md)).
- 0x80076d6c is never called for level 7 (no ending cine records it, and
  no bonus case): level 7 owns no piece and gives no bars.

## State

| address | type | meaning |
|---|---|---|
| 0x800c5680 | s32 | character (state +0x08): 9, Reptar |
| progress +0x30 (0x800c56a8) | u32 | \|= 256 by the EpisodeStart post (0x8007871c); \|= 512 by 0x8007b748 ("final level reached"); seen 0x300 |
| progress +0x23 (0x800c569b) | u8 | tries: 1 (0x800b2920[7]) |
| 0x800c3e80 (gp+0) | u8 | set by cine 85's end; the teardown plays the credits when it is set |
| 0x800c4cc0 | s32 | ambience countdown (dt): 300 at the setup (0x80080318), + 1000 after each line |
| 0x800c4cc4 | s32 | 1 at the setup: the first ambience line is 1181 |
| 0x800c4ba0 | s32 | 100 at EpisodeStart (a cooldown also used by 0x80053344 and others; not traced here) |
| 0x800c41f6 | u16 | fire on: 0 when state 30 starts, 1 from the animation event |
| 0x800c4a4c | s32 | follow distance: 6400 in level 7 (0x80025e90) |
| 0x800c4a68 | s32 | 7200 in level 7, else 4800 (0x80025e90) |
| 0x800c3f5c | s32 | camera height: -1000 in level 7 for every switch (0x800291f8) |
| 0x800c3f60 | s32 | 0 in level 7 (0x80025e90 at 0x800261f4) |
| player +0x38 | u16 | 0: the runner's id switch (0x80017c0c at 0x80017f0c) has no case for char 9; the registry maps id 507 to the player |

Level 7 has no TimerExpired case (0x8007f5a8 table 0x800158cc index 5 is
the default 0x8007f6c8), no room change case (0x8007f498 table 0x80015894
index 4 is the default), no Gameplay script, no CheckPoint room and no
GoalPost case. Damage is posted in level 7 (0x800780ac drops it only in
levels 1, 11, 17 and 23-31) and 0x80077864 has no level 7 case.

## Data in DB07

LP (DB07LP.BIN): 270 kind 1, 31 kind 6, one kind 3, one kind 99. No kind 2
posts, no kind 7 spawn groups, no Reptar bars (template 835), no health
(863).

### The city: rooms (LEVEL07.BIN)

23 rooms, boxes read from RAM (room +0x10..+0x28: x0, y0, z0, -, x1, y1,
z1). Every room has flags 0x14 (4 and 16) and camera kind 6 (+0x5d); room
6 also has flag 2 and room cine 85 (+0x5b). After EpisodeStart rooms 5
and 9 also have flag 128 (0x94).

| room | x | z | what |
|---|---|---|---|
| 0 | -29696..2048 | 37888..52224 | far north-west; the mover (template 795) |
| 1 | -1024..25600 | 25600..34816 | the avenue (west part) |
| 2 | -18432..-10240 | 34816..37888 | north-west |
| 3 | -27648..-6144 | 6144..26624 | the raised west strip (south part) |
| 4 | -26624..-1024 | 26624..35840 | the raised west strip (north part), the avenue's west end |
| 5 | 0..10240 | 8192..25600 | north of City Hall; speech 161 on entry |
| 6 | -1024..9216 | -11264..6144 | City Hall and its plaza; flag 2, cine 85 |
| 7 | 25600..35840 | 16384..26624 | north-east street |
| 8 | 11264..18432 | 7168..17408 | |
| 9 | 10240..35840 | -21504..4096 | the start plaza; song 0 on entry |
| 10 | 10240..11264 | 6144..8192 | small, y -5120..-1024 (not traced) |
| 11 | -10240..-1024 | -8192..5120 | west of City Hall |
| 12 | 37888..60416 | -21504..-11264 | the babies' living room (a cine set, not part of the city) |
| 13 | 11264..21504 | 4096..7168 | |
| 14 | 7168..10240 | -18432..-9216 | south of City Hall |
| 15 | 18432..27648 | 7168..17408 | |
| 16 | 17408..27648 | 15360..23552 | |
| 17 | 24576..35840 | 7168..16384 | |
| 18 | 21504..35840 | 26624..34816 | the avenue (east part) |
| 19 | 21504..35840 | 4096..7168 | |
| 20 | -3072..10240 | 5120..9216 | the street into City Hall's plaza from the north |
| 21 | 3072..5120 | 25600..26624 | small, y -3584..-768 (not traced) |
| 22 | -7168..0 | 9216..25600 | between the strip and room 5 |

y spans are -12672..256 for most rooms and -8192..0 or -10240..0 for some.
The street floor is y 0; Reptar's origin stands at y -703 on it.

### LEVEL07.COL: the street map

Decoded with the COL layout of [gameplay](../engine/gameplay.md#collision-levelnncol)
(min (-27648, -10240, -21504), max (57344, 0, 49152)). Cells (1024 x
1024) with any record are streets; cells with none are building blocks
and cannot be stood in. Measured from the file:

- The start plaza and its streets (x 11264..30720, z -19456..4096, and
  the grid north of it up to the avenue z 28672..31743) and City Hall's
  side (x -6144..9216, z -6144..24576) share no floor cell at y 0. Of the
  cells reachable at street level from the start (788) and from City
  Hall's plaza (346) none are shared.
- A strip of floor at y -1024 (x -22528..-11264, z 18432..39936) links
  them: the avenue's west end meets it at x -10240, z about 30000, and
  City Hall's side meets it at x -10240, z about 18432.
- City Hall's plaza (x -6144..7168, z -6144..4096) has no floor east of x
  8191: the cine shows Reptar walking out that way, but the cine moves him
  without collision.

### The characters (LP kind 1)

| LP | template | char | id | position | what |
|---|---|---|---|---|---|
| 2 | 8 | 9 | 507 | (25279, -703, -1418) | Reptar, the player (template 8: class 1, shape 9, flags 0x2001) |
| 298 | 11 | 1 | 1492 | (19715, 1001, -27333); (43394, -1012, -15004) in RAM | Tommy, cine actor in room 12 |
| 300 | 2 | | 1776 | (22813, -876, -29668); (42946, -899, -14687) in RAM | Chuckie, cine actor |
| 301, 302 | 7, 6 | | 608, 609 | | Phil and Lil, cine actors |
| 296, 297 | 10, 3 | 8, 13 | 15, 16 | | cine actors (seen: two grown-ups in cine 79) |
| 299 | 581 | | 54 | (16011, 711, -28007) | the puzzle board (use 0x8004d3f0) in room 12, out of Reptar's reach |

All but Reptar are in room 12 with +0x00 bit 0x4000 and are not read by
level 7's code. The loader puts the player on LP 2 (the kind 1 record
whose template's shape is char 9); cine 79 leaves him at (25162, -703,
-9351), room 9 (seen).

### Buildings (class 14, template +0x04 = 14)

Class 14 (0x80049c50): enter +0x48 = 0x80049e18 (scheduled, period 136,
+0x00 bit 1), leave +0x4c = 0x80049e5c, think +0x44 = 0x8004d7c4, +0x00
|= 0x2000000 (breakable). Touch +0x58 is template byte +0x0a: 14
(0x8005202c, debris) for most, 0 for 1135 and 1071. Trigger radius 15000.
96 in DB07:

| template | count | rubble (t + 1) flags | rubble blocks |
|---|---|---|---|
| 910 | 24 | 0x2 | yes |
| 914 | 2 | 0xa | no |
| 916 | 10 | 0x2 | yes |
| 918 | 4 | 0x2 | yes |
| 927 | 3 | 0x2 | yes |
| 931 | 3 | 0x2 | yes |
| 971 | 2 | 0xa | no |
| 993 | 5 | 0xa | no |
| 995 | 2 | 0xa | no |
| 997 | 3 | 0xa | no |
| 1018 | 7 | 0x2 | yes |
| 1020 | 2 | 0x2 | yes |
| 1034 | 1 | 0xa | no |
| 1049 | 4 | 0x2 | yes |
| 1071 | 1 | 0xa | no |
| 1135 | 12 | 0xa | no (street lamps, guess: they stand in rows along the plaza) |
| 1158 | 1 | 0xa | no |
| 1165 | 6 | 0xa | no |
| 1173 | 3 | 0x2 | yes |
| 1240 | 1 | (removed) | no |

Template flag 8 is "no object collision" (0x8006baa8 returns at once, and
the player's object test skips it), so rubble of a flag 0xa template can
be walked through and rubble of a 0x2 template still blocks. Seen: the
rubble of 1049 at (24576, -2048, 28670) blocked the avenue's south lane.

### Other objects

| template | count | class | what |
|---|---|---|---|
| 401 | 2 | 0 | (30158, -167, 21532) room 7, (23628, -167, 19991) room 16; +0x00 bit 0x10000; taken by touch (below) |
| 795 | 1 | 11 | (-3837, -2490, 42990) room 0; a mover (0x80048804: think 0x8004ee00, enter 0x8004f228, leave 0x8004f3a8); +0x00 0x30000; taken by touch (below) |
| 512 | 6 | 2 | y -4217 / -5390 above room 9 and room 0; think 0x8004e2ac turns +0x24 by template +0x18 (16) a VSync; no damage (template flag 0x200000 clear); guess: the searchlights seen in the sky |
| 616 | 1 | 10 | (-20246, -5091, 21126) room 3; think 0x8004e0f4, +0x50 0x80053344; no damage; not traced |
| 688, 793, 872, 391, 403, 685 | 18, 17, 15, 1 each | 0 | +0x00 bit 0x10000 (pickable), but Circle is the fire in level 7, so nothing picks them up; they block (guess: stop signs and street furniture; a stop sign was seen) |
| 800-809 | 2 each | 0 | high up (y -2752..-5792) at the avenue's edges (guess: signs) |
| others | | 0 | scenery |

## Start

The level start, in order: player init on LP 2 (Reptar's hierarchy from
DB07); the setup's 0x800819a4 calls 0x800b9598[7] = 0x80080318
(0x800c4cc4 = 1, 0x800c4cc0 = 300); cine 79 (0x80074958 case 7; its start
and end hooks are the defaults, 0x80072ef0 / 0x800732d4 cases 79); the end
hook's 0x80074ec0(0, player +0x38 = 0); CineFinish (0x800783d8 case 78):
card 708 (HINT37, arg 6); EpisodeStart 0x8007b748 (installed by 0x8007871c
case 7); 0x8007737c case 78 does nothing.

0x8007b748(player):

1. 0x800291f8(0x800de460, player, 1, 1500, 3000, 0, 0, 712) (the last
   four on the stack, read by 0x800291f8 as its locals 0x30..0x3c): the
   follow camera; in level 7 the height is -1000 whatever the argument.
   What the stack arguments do for kind 1 was not traced.
2. +0x30 |= 512.
3. 0x80078e3c(5, 1185): room 5 +0x64 = 1185 & 255 = 161 (the field is a
   byte) and flag 128, so entering room 5 plays speech 161 (0x80077a54:
   +0x64 of 5 and up is a speech line). Guess: line 1185 was meant.
4. 0x80078e3c(9, 1): room 9 song 1 (0x8006efcc(0, 0, 30) on entry).
5. 0x800c4ba0 = 100.

No respawn, no phase, no timer, no HUD.

Reptar faces +z after cine 79 (heading 0 at 2890 and 2950).

Seen: cine 79 from 1819 to 2887 (1068 VSyncs): the babies and two grown-ups
in the living room (room 12), then Reptar walking from (916, -702, -435) in
room 6 to (25170, -702, -9032) in room 9 (the player object, moved by the
cine). At 2887 card 708 (arg 6), 0x80080318 from 0x800819dc, 0x8007b748,
0x800291f8(.., 1, 1500), 0x80078e3c(5, 0x4a1), 0x80078e3c(9, 1), then
RoomChange: CameraSwitch(6) and 0x800291f8(.., 1, 200). Reptar at (25162,
-703, -9351), room 9, health 255, tries 1, +0x30 0x300, 0x800c3f5c -1000,
0x800c4a4c 6400, 0x800c4a68 7200, camera +0x4a 1. Room 5 +0x64 161 and
room 9 +0x64 1, both flags 0x94. No HUD on screen.

## Reptar

### Movement

The stick's speed factor for char 9 is 0x800c421c = 10, and 0x800c4220 =
10 with WALK held (0x8005be24 at 0x8005bfe0); the walk and run tests use
the same (0x80059008, 0x800590d4). The turn rate is the default 1025
(0x8005bbf4 slows only chars 6 and 34). Seen: about 97 units a game frame
running (state 2). (gameplay.md lists 10 / 10 under "level 9"; the test
is on 0x800c5680, the character, so it is Reptar's.)

Walking into a wall: other levels go to state 9; level 7 goes to state 0
(0x8005e1f4 at 0x8005e658). Seen: Reptar stops and stands.

Reptar climbs (state 6) onto floor 1024 higher: seen at (-9243, -1072,
30152) onto the west strip, y -703 to -1870.

Reptar's y bobs while he runs (seen -700 to -930): the stomp animation.

### Level code in the player update (0x8005a068 at 0x8005ab60)

Every frame in level 7, before the state change:

1. Triangle pressed (raw button word 13, 0x800cda4c `& 3 == 1`): sound 89
   (0x8006f26c), state 29.
2. Circle pressed (word 14, 0x800cda50): sound 89, state 30.
3. A touched object (+0xf4) with +0x00 bit 0x10000:
   - template 401: sound 88 at it;
   - template 795: its leave +0x4c, off the trigger list (0x8001a810 on
     0x800c4c24), explosion 0x80053d28, sound 91 at it;
   - then for both: shards 0x8004cdfc(obj, pos, 200), out of its room's
     list, +0xf4 = 0.
   Any other 0x10000 object is left alone.

The presses are taken in any state (in the air too) and override the
stand handler's ACTION and GRAB, so Reptar never picks anything up.

### Chomp: state 29 (0x8005cc2c, animation 49)

At the animation's event (+0xe0 bit 2048), if the touched object (+0xf4)
has +0x00 bit 0x2000000 (a building) and its model is not yet shape + 1:
model = template shape + 1 (0x80044b74), template 1135 gets +0x00 |=
0x2000, its touch +0x58 (debris), template += 1, explosion 0x80053d28.
The state ends when the animation does (generic). Seen: state 29 for about
110 VSyncs (3280-3390); +0xf4 stayed 0 in every run (a building breaks on
contact first), so this smash was not seen.

### Fire: state 30 (0x8005ce74, animation 51)

Entering clears 0x800c41f6; the animation event sets it. While it is set,
each game frame: the matrix of bone 16 (0x80044fec), the facing into
+0x200 (0x80076040), the start = bone 16's position + (0, -1400, 512)
(s16 at 0x800c41f0) turned by the bone matrix (0x8009fbe8), then 0x8005cd74(start, +0x200): one
flight (0x8008864c) with flags 0x8030b: kind 11, no gravity, collision
bits 0x80000 only (damage, but neither the world 0x10000 nor objects
0x20000, so it hits nothing); radius 750 (0x800c41e0), lifetime 1500 ms
(0x800c41e4), speed 70 (0x800c41e8), colour 0x800c42d0 (grey 128) and
damage 1000 (desc +0x6a; not a speed). Kind 11 (0x80086114) runs sequence
0x1017 (five frames of 200 ms) on a quad of half size 750 that grows by
256 * dt / 64 a frame (0x800c41ec << 6), additive (face 0xd2a00000). The
direction is the head's yaw and pitch: the part's matrix as the game
keeps it is world to part; its transpose's (1, 2) gives -asin pitch,
(2, 0) and (2, 2) the yaw (0x800210b0), back to a unit vector
(0x80076040, 0x8007652c). docs/engine/objectives.md "Effect flights" has
the record and the step.

Seen: Circle at 3300: state 30 3310-3380, 21 flights 3345-3387 (one a
game frame) in one run, 12 (3344-3385, dt 15-20) in another: a flame
cone from the mouth, sweeping right to left. A building 2744 ahead did
not break: the flames break nothing (their flags have no collision).
The first flame: head part 2 (slot 16) with matrix rows (3514, 128,
-2120), (121, 4079, 447), (2119, -448, 3487) at (25225, -2236, -8706);
start (25448, -3687, -8423), direction (2118, -446, 3485), velocity (395,
-84, 650); its position, life, size and sequence frame then went as
crates/rugrats/tests/flights.rs pins (dt 20: life 1500, 1434, 1368;
size 750, 830, 910; frame 1 after 600 ms).

### Rumble jump

Cross is the ordinary jump (state 5, 0x80060f1c, no char 9 case). The
land handler (0x8005cba8) for char 9, every frame in state 10: sound 93
at the player and 0x8001e318(camera, player pos, 160, 450) (guess: a
camera shake). Seen: a standing jump at 3500: state 5 3510-3560, apex y
-1939 from -703, state 10 3570-3580, eight 0x8001e318 calls 3568-3588.
No Damage was posted.

## Buildings breaking (0x8004d7c4)

The think runs every game frame while Reptar is within the trigger
radius (15000; 0x80072d38 enter / leave). 0x8006baa8(building) scans the
objects around it (0x8006add8) and, for the player or an object with
+0x00 0x10000 or 0x4000, tests it against each of the building's model
boxes (0x8006b6d0); a template with flag 8 never hits. No hit: nothing.
On a hit:

- Template 1240: think off, +0x00 bit 0x2000000 cleared, touch (debris),
  explosion, sound 91, out of its room's list.
- Else, if the model is not already shape + 1: model = shape + 1; a sound
  by template (931 speech 1183; 793 and 688 sound 94; 914 and 1165 sound
  96; 971 sound 95; 1135 +0x00 |= 0x2000 and sound 90); template += 1;
  think off; bit 0x2000000 cleared; touch (+0x58) if any; explosion
  0x80053d28; sound 91 or 92 (bit 16 of the next rand 0x800c4174).
  Templates 793 and 688 are class 0 in DB07, so their sound case is not
  reached here.

Debris (0x8005202c): ten kind 5 flights (flags 0x30705: gravity 13, the
world and objects collide) of TEXPOS sprites picked by (rand mod 10,
0x800c419c) from 392, 393, 394, 395, 396, 397, 399, 401, 403, 405, at
speed rand mod 200 capped at 100, thrown within half a turn of up
(0x80088bac(up, identity, 0, 2048)), 1000 ms, bounce 2048, 200 x 200,
opaque (0xc0800000), colour 255, damage the object's +0x6d. Explosion
(0x80053d28): one kind 7 flight at the object's position, radius (and
half size) 3000, 5000 ms, still, white, sequence 0x1016 (150 ms a
frame), additive; its direction is drawn (0x80088bac(+z, identity, 0,
120): two draws of 0x800c445c) though the speed is 0. Seen: the lamps at
2931 and 2950 made explosions at (26004, -1701, -7911) and (26079, -1701,
-11540), the lamps' positions.

Seen: lamps (1135) broke 1670 to 2240 from Reptar's origin as he stood or
walked by (2931, 2950, 2966, 3052: sound 90 then 91, no debris); building
914 at (31744, -3072, -1026) broke as he ran past at 3238 (sound 96,
debris, explosion, sound 91); building 1018 at (31744, -3072, -5122)
broke at 3635 while he stood 2400 away (guess: his idle animation's tail).
27 breaks on the walk to City Hall. The broken model shows the building
gutted (screenshot).

## The route (walked in the oracle)

Steered with the stick from the start, by waypoints, no teleport:

1. The start plaza (room 9): east to x 28160, then north along x about
   28000 to z 9728 (room 17).
2. West to x 23040 (room 15), north to z 17920 (room 16), east to x 29184
   (room 7), north to the avenue at z about 29000 (room 18).
3. West along the avenue's middle (z 29900..30700; the lanes at its edges
   are blocked by the rubble of 1049 and by signs) through rooms 18, 1
   and 4 to x -10300.
4. Climb (state 6) onto the raised strip, west to x -15000, south on it
   to z about 19500 (room 3).
5. Down and east to (-6000, 18800) (room 22), north-east to (-4500,
   23400), east along z about 23800 to x 6600 (room 5; the building 916 at
   (0, 21502) and its solid rubble block z 21838).
6. South along x about 6800 to z 12800, west to x 3600, south through
   room 20 into room 6: RoomChange posts cine 85.

Seen: from 2960 to 6503 (3543 VSyncs, about 59 s). Other ways north
exist (seen: x about 12400 through rooms 13 and 8); by the COL every
street-level way from the start meets City Hall's side only over the
raised strip.

## The end: cine 85 and the credits

Entering room 6: 0x80077a54 sees flag 2, PostCine(85) (0x80078350 from
0x80077aa8), clears flag 2. Cine 85 "Reptar Dance": hooks the defaults
(0x80072ef0 / 0x800732d4 cases 85); no card (0x800783d8 case 84);
0x8007737c case 84 (0x80077838): 0x800c3e80 = 1, exit code 0x800c4918 =
-1. Nothing is recorded: 0x80076d6c is not called.

The teardown (0x80017c0c from 0x80018230) with exit -1:

1. +0x04 = 7 (the previous level).
2. Not activity play: the progress is cleared: +0x22, +0x23, +0x30, +0x34,
   +0x38, +0x3c, +0x40, +0x44, +0x46 = 0, and +0x48[32], +0xc8[32] = 0
   (0x80018264).
3. No loading screen (the result is negative).
4. Everything freed, then, with gp+0 set: 0x8008b1e4 (0x8008b2c8(11): the
   movie buffers and display mode 11; movie_play(6, 1): CREDITS.STR, 1875
   frames, any button skips; free; display mode 26), gp+0 = 0, +0x30 |=
   4, return 1.

Main then runs level 1 (0x80018854; no chooser, no new-game reset). With
+0x30 bit 4 set the hub plays no cine 7; its EpisodeStart (0x80078e7c)
puts Tommy at 0x800b8718[+0x04 = 7] = (5532, 779, -475), room 25. The game
goes on in the hub with no pieces, no bars and no levels done.

Seen: cine 85 from 2981 to 5317 (2336 VSyncs; Reptar dancing on City
Hall's steps between searchlights); at 5317 0x8007737c(.., 0x55, 0),
0x8008b1e4 from 0x8001854c; movie_play(6, 1) at 5326; the credits (names
over the Rugrats logo) until about 11387; at 11387 the runner's load
lines (0x8003b168 from 0x8008fc38), at 11390 0x80074958(1) without a cine
and 0x8007871c(.., 1); Tommy at (5532, 779, -475) room 25, +0x30 = 4,
+0x34 = +0x38 = 0. With progress poked before the end (+0x34, +0x38 0x1ffe,
+0x44 and +0x46 0xfff, +0x48[2] 0x80000000): all 0 after the teardown,
+0x04 = 7.

## The loss: ForceCry (0x8007fd0c)

Level 7 is case 6 (0x80080148): s1 = 0, so no cry (0x8007f920 not
called), health 255, no callback. The generic part counts the one try
down (1 -> 0); with none left, SwitchEpisode(player, 1) (0x800802b4).
Nothing is recorded and +0x30 bit 512 stays set.

What can post it: Damage to 0 health (0x80077864). Nothing in level 7 was
found that damages Reptar: no NPC, no hazard (no template with flag
0x200000 or 1024 in DB07), the class 2 and 10 objects have no damage
flag, and no Damage was posted on jumps or walks (0x800780ac hooked).
Guess: the level cannot be lost in normal play; Pause "Quit Level" and
Select "Restart" still leave it.

Seen (0x8007fd0c injected at 3001): no 0x8007f920, 0x800781f8(player, 1)
at 3001, the loading page (0x8003eefc) at 3001, tries 0, +0x30 0x300, no
0x80076d6c; the hub started at 3063 (it played cine 7 because the probe's
game had never set +0x30 bit 4).

## Camera

- 0x80025e90 (follow, kind 1) in level 7: 0x800c4a68 = 7200 (else 4800),
  0x800c4a4c = 6400 (else 4096), 0x800c3f60 = 0 (else 256 in level 15),
  and the target's y is the player's +0x148 (the floor under him) instead
  of +0x1c (at 0x800260c4 and 0x80026248), so the camera does not follow
  the stomp's bob.
- 0x800291f8: level 7 sets 0x800c3f5c = -1000 for every switch (with 1500
  or any other argument, at 0x80029288 / 0x80029298); 0x800290a4 resets it
  to -1000 too.
- Every room has flag 4 and kind 6, so each room change posts
  CameraSwitch(6) (0x80078264 from 0x80077b44) and 0x800291f8(.., 1,
  200), which in level 7 keeps -1000.

Seen: 0x800c3f5c -1000, 0x800c4a4c 6400, 0x800c4a68 7200 throughout;
camera at (25170, -470, -15427) for Reptar at (25162, -703, -9351): 6076
behind and 233 lower than his origin, looking up at him.

## Ambience (0x80080cd8)

0x800b9618[7], every frame (0x800818e0). Countdown 0x800c4cc0 - dt; below
0: if 0x800c4cc4, line 1181 and clear it; else rand & 1: 0 -> 1123 +
(rand & 255) mod 10 (1134 would become 1123); 1 -> 1182 + (rand & 255) mod
13, with 1183 and 1185 replaced by 1184 and 1186. Then countdown += 1000.
Speech only (0x8008da4c).

Seen: 1181 at 2949 (62 VSyncs after EpisodeStart), then a line every about
200 VSyncs: 1132, 1126, 1127, 1182, 1128 in one run; 1192, 1182, 1184,
1131, 1130, 1194 in another.

## What does not happen in level 7

- No NPC moves; the babies and grown-ups stay in room 12 for the cines.
- No timer, counter, health meter or other HUD.
- No pickups: Circle is the fire, and only templates 401 and 795 are taken
  (by touch).
- No piece, no bars, no 0x80076d6c, no "already earned" note.
- Not an Activity (ACTIVITY_LEVELS at 0x80013030 is 13, 17, -2, 18).

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT37 | 708 | after cine 79 (arg 6) | Stomp through the city to reach Reptar's goal, City Hall. Run into buildings to knock them down. Press @ to chomp. Press # to breathe fire. Press % for a rumble jump. |

@ is Triangle, # Circle, % Cross (matching states 29, 30 and the jump).
The port's `card_after` has 79 -> 708.

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 79 | Reptar Intro | the level start | none | default | nothing (card 708) |
| 85 | Reptar Dance | room 6 (flag 2) | none | default | gp+0 = 1, exit -1: the end of the game |

Cine table flags (0x800b8bd0 + 16 x cine + 4) are 0x200 for both. Seen
lengths (oracle VSyncs): 79 1068, 85 2336. The credits movie ran about
6060 VSyncs.

## Checked in the oracle

The level 18 probe copied to the scratch folder as l07probe, with a level
7 line (char, +0x30, +0x34, +0x38, +0x44, +0x46, gp+0, the ambience
countdown, 0x800c4ba0, the camera's position, 0x800c3f5c, 0x800c4a4c,
0x800c4a68, 0x800c41f6), a room object list dump, `--tp` (position, floor
heights, heading) and `--path` (waypoints steered with the d-pad from the
camera's heading). Presses start 300, cross 500, 650, 800, 1650, 1800;
main's level argument patched (0x80018824 = `li a0, 7`, restored at
2400), 0x800c5678 = 7 at the runner, cards skipped (0x8004472c =
0x1000004a), 0x80018598 made to return 9 (until frame 3000 or 4000 where
the hub followed). The start cine was kept.

- Start: cine 79 1819-2887, card 708 (arg 6), 0x80080318, 0x8007b748 and
  its calls, Reptar in room 9, tries 1, +0x30 0x300, rooms (23) and their
  objects read from RAM, the camera values.
- Breaking by contact: lamps and buildings, sounds 90, 96, 91; debris and
  explosion.
- Chomp (state 29), fire (state 30, 21 flights), the rumble jump (sound
  93 is from the code; the eight 0x8001e318 calls seen).
- Template 401 taken by walking into it (0x8004cdfc from 0x8005ac64).
- Walking into a wall: state 0.
- The walk from the start to City Hall by the route above; room 5's
  speech 161 (0x8008da4c(0xa1) from 0x80077c18).
- Cine 85 by walking in and by teleport; the teardown clearing the
  progress; CREDITS.STR; the hub at (5532, 779, -475) with +0x30 = 4.
- A forced ForceCry: no cry, the hub, nothing recorded.
- Ambience lines.
- Not checked: the chomp's own smash (+0xf4 a building); template 795 and
  1240 being taken / removed; Pause, Quit Level and Restart; skipping the
  credits with a button; the hub's board sending back to 7 after a loss;
  the loading page at the end (there is none at the teardown; what the
  load-time redraw shows was not looked at); template 616; room 10 and
  21; what the flames do when they hit.

## For the port

The port has the Script trait, the playable character by level, per-player
gait and jump, climbing, template flag 8 volumes, hiding objects, room
flags 2/4/32/64/128 (room 6's cine 85 is posted already), the follow
camera with level 7's -1000 and 6400, card 708 after cine 79, cards and
tries, ForceCry, `start_cine` 7 -> 79 and the switch to level 7. Level 7
needs, in order:

1. Reptar as the player: `player_character(7)` = 9 (the port's table has
   no 7, so Tommy loads); the LP record with shape 9 is LP 2 (template 8,
   id 507); gait (10, 10) (0x800c421c, 0x800c4220); player +0x38 left 0.
2. A `level7` script. Setup: the ambience (first line 1181 after 300 dt,
   then a line every 1000 dt from the two lists). EpisodeStart
   (0x8007b748): +0x30 |= 512; room 5 speech 161 on entry; room 9 song 0
   on entry; no HUD, no timer.
3. The three buttons, on raw Triangle / Circle presses in any state,
   with sound 89: state 29 (animation 49; at its event a touched building
   breaks) and state 30 (animation 51; from its event, one flame a game
   frame from bone 16 along the head's yaw and pitch, speed 70, hitting
   nothing). Done (world::flights, levels::reptar_2010); the port's
   animation 51 runs about twice as fast as the original's: its event 10
   frames (20 VSyncs) after the press and its end 19 frames (38 VSyncs)
   after, where the oracle's state 30 has the event 34 VSyncs in and ends
   at 70, so the port breathes 9 flames where the original breathes
   about 18 at dt 10. The
   rumble jump: in the land state for char 9, sound 93 and the camera
   shake (0x8001e318 with 160, 450) every frame. Walking into a wall goes
   to stand, not state 9. GRAB and ACTION do nothing else.
4. Buildings (class 14): break when Reptar's shape meets one of their
   model boxes, tested every frame within 15000: model = shape + 1,
   template += 1 (so the rubble's template flag 8 decides whether it
   still blocks), the per-template sound, debris (ten flights of
   particles), the explosion, sound 91 / 92; template 1240 goes away
   entirely; lamps (1135) get no debris and sound 90. The debris and the
   explosion are done (world::flights). The port needs a
   runtime template change for an object (collision and model from the
   new template).
5. Touch-collect in the player update: templates 401 (sound 88) and 795
   (sound 91, explosion) are removed with shards (0x8004cdfc, 200). The
   explosion is done; the shards are not.
6. The camera in level 7: keep -1000 on every CameraSwitch (the port's
   room change sets 200 for kind 6, which every DB07 room is); aim at the
   floor height (+0x148), not the bobbing origin; 0x800c4a68 7200.
7. The end: cine 85's finish must not go to "Returning to Main Menus"
   (the port's `quit`). It must clear the progress block fields listed
   above (not +0x00, +0x04, +0x21), set the previous level to 7, play
   CREDITS.STR (movie 6, skippable by any button), set +0x30 = 4 and start
   level 1 with no start cine, Tommy at (5532, 779, -475), room 25, and no
   loading page before the credits.
8. The loss: `cry_animation` false; with tries 1 any cry goes to the hub
   at once with nothing recorded, +0x30 bit 512 kept (the hub's board
   then switches to level 7; check that the port's board does that).
9. Small things: the class 2 objects' spin (heading + 16 a VSync); room
   12's actors stay hidden from play (they are only in the cines).

## Unknown

- Whether anything can hurt Reptar in normal play (none found; guess:
  no).
- Why animation 51 (and 49) take about 70 (110) VSyncs in the original
  and half that in the port (the animator's rate for char 9 or the
  state's own timing; not traced).
- The exact test in 0x8006b6d0 (the player's shape against a model box)
  and why a building broke 2400 from Reptar while he stood (guess: the
  tail).
- What templates 401 and 795 are (guess: a vehicle and a moving thing;
  795 is class 11, a mover, at the far north-west).
- Template 616 (class 10, think 0x8004e0f4) and rooms 10 and 21.
- Whether speech 161 in room 5 is the intended line (1185 was passed).
- What 0x800c4ba0 = 100 is for in level 7 (it also changed to 10 during
  play; not traced).
- The animation lengths that end states 29 and 30 (seen about 110 and 70
  VSyncs).
- The hub's load-time page after the credits ("Pickles' Home / Returning
  to" by the table in screens.md; not looked at).
- screens.md's "movie 11": 11 is the display mode 0x8008b2c8 sets
  (0x8001de04); the movie is 6, CREDITS.STR, and the runner returns 1,
  the hub, not the front end.
