---
title: Level 6, Toy Palace
status: partial
discs: US
covers: US SLUS_006.50:0x800794a0, US SLUS_006.50:0x8007b670, US SLUS_006.50:0x8007b71c, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80018598, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80083918, US SLUS_006.50:0x80083b4c, US SLUS_006.50:0x80084084, US SLUS_006.50:0x800585b0, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x80059570, US SLUS_006.50:0x800596b0, US SLUS_006.50:0x80049e5c, US SLUS_006.50:0x800495d0, US SLUS_006.50:0x80048364, US SLUS_006.50:0x8004e2ac, US SLUS_006.50:0x80048400, US SLUS_006.50:0x8004e0f4, US SLUS_006.50:0x80049d54, US SLUS_006.50:0x80048974, US SLUS_006.50:0x8004dba0, US SLUS_006.50:0x80049d98, US SLUS_006.50:0x8004d174, US SLUS_006.50:0x800522ec, US SLUS_006.50:0x80051db4, US SLUS_006.50:0x800539b4, US SLUS_006.50:0x80053ef8, US SLUS_006.50:0x80054ae0, US SLUS_006.50:0x8004cdfc, US SLUS_006.50:0x80058da4, US SLUS_006.50:0x80056e44, US SLUS_006.50:0x80062cf4, US SLUS_006.50:0x8005f310, US SLUS_006.50:0x800834b0, US SLUS_006.50:0x800833a4, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x80080a20, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x80077864, US SLUS_006.50:0x8004f83c, US SLUS_006.50:0x80057950, US SLUS_006.50:0x8005be24, US /DATA/DB06/DB06LP.BIN, US /DATA/DB06/LEVEL06.BIN, US /DATA/DB06/LEVEL06.COL, US /DATA/HINTS
worklog:
---

# Level 6, Toy Palace

Everything level 6 needs to be played to its end: Chuckie through the dark
toy store to Tommy, the scary toys, the clowns, the hand-over to Tommy in
cine 46, the Reptar boxes, the climb to the switch and the win. Read from
the pseudo-code and the disassembly; the start, the scares, the bites,
the timed bursts, the phases, cine 46 and the swap, a box pick-up and a
climb, the switch and the win, a cry and the last-try loss were run in the
oracle (see Checked in the oracle). Builds on [level 5](level-05.md),
[level 4](level-04.md), [level 3](level-03.md) and [level 2](level-02.md)
(NPC modes, posts, events, ForceCry, cines, cards, spawn groups),
[gameplay](../engine/gameplay.md) and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). "char" is the character (+0x15c =
template +0x10). A VSync is 1/60 s; the oracle's frame counter counts
VSyncs.

## The goal in one paragraph

The player is Chuckie (char 2, 0x80018598 case 6) for the first half.
Cine 45 "Toy Palace Intro", then card HINT23 (file 694): be brave, work
through the darkened store to find Tommy; getting too close to certain
toys frightens Chuckie; pass the clown area. Scary toys (template flag
0x200000) frighten him when he comes within their trigger radius: his
walk animations change and he loses 2 or 5 health, once per approach.
Clowns (char 41, ids 920-922) guard room 4 and bite for 8. Two more
guards bite for 15 (id 46) and 6 (id 506). Timed bursts in room 4 hurt
for 18. Tommy waits in room 11. Entering it plays cine 46 "Toy Palace
Trans" (and sets the checkpoint); at its end the player becomes Tommy
(0x800585b0), Chuckie becomes an NPC who watches him, tries go back to 3
and play goes on in room 9, the Reptar room. Card HINT24 (695): collect
the Reptar boxes, put them on the stairs, climb up to the switch and turn
on Reptar; watch out for Thorg. Eight boxes (template 1016) lie around
rooms 6-10 and 12. Tommy picks them up with Circle, puts them down, and
climbs onto them (player state 6). ACTION (Triangle) on the switch (LP
163, template 355) on the top platform plays cine 47 "Toy Palace End" and
wins the level.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 EpisodeStart, 2 room 2, 3 after cine 46 |
| 0x800e5830 | s32[3] | respawn position for ForceCry (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| progress +0x23 (0x800c569b) | u8 | tries; 3 again after cine 46 (0x8007b670) |
| progress +0x30 bit 2 | | set by the room 11 checkpoint (0x800772e0) |
| player +0x188 bit 256 | | frightened (0x80059570 sets, 0x800596b0 clears) |

The phase is only written through 0x80078d60: 1 by EpisodeStart
(0x800794a0), 2 by 0x8007b71c (every entry into room 2), 3 by 0x8007b670
(cine 46's end). Nothing in level 6 reads the phase; it only travels with
the respawn, and level 6's ForceCry case is the generic one. Room 2 has no
phase test, so Tommy walking into room 2 in phase 3 sets phase 2 and moves
the respawn there (from the code; not seen). There is no level timer
(0x8007f5a8 table 0x800158cc index 4 is the default 0x8007f6c8) and no
Gameplay script (no kind 6 script byte in DB06).

## Data in DB06

### The player and the characters (LP kind 1)

Level 6's playable character is 2 (0x80018598, also level 10). The
loader (0x800454dc) moves the player onto the first kind 1 record whose
template's shape is the character: LP 26 (template 2, id 1776). Chuckie
has the normal turn rate (0x80062eac: slow only for chars 6 and 21) and
the camera aims 768 above him (0x80025e90: 400 only for char 6).

| LP | template | char | id | position | health | range | speed index | +0x6d | what |
|---|---|---|---|---|---|---|---|---|---|
| 26 | 2 | 2 | 1776 | (3051, -1376, -468) | 250 | 5000 | 3 | 50 | Chuckie, the player until cine 46 |
| 15 | 11 | 1 | 1492 | (5806, -1225, -35271), room 11 | 5 | 2000 | 4 | 50 | Tommy; the player after cine 46 |
| 172 | 15 | 16 | 46 | (-12604, -1538, 24717), room 0 | 100 | 2000 (5000 after EpisodeStart) | 1 | 15 | a guard near the start (guess: Thorg, or a store adult) |
| 42 | 14 | 22 | 506 | (14260, -2338, -30282), room 6 | 5 | 5000 (15000 after EpisodeStart) | 1 | 6 | a guard near the box rooms (guess: Thorg) |
| 70, 71, 72 | 925 | 41 | 920, 921, 922 | (-12583, -1697, -36188), (-36685, -1697, -36075), (-36830, -1697, -15922), room 4 | 5 | 7000 | 1 | 8 | clowns (guess, from HINT23 "the clown area") |
| 41 | 8 | 9 | 507 | (51589, -2776, 7810), room 9 | 5 | 2000 | 2 | 0 | no level code (guess: cine 47 actor) |
| 128 | 10 | 8 | 15 | (49393, -2311, -35494) | 5 | 2000 | 2 | 0 | no level code; cine 45 leaves it in room 1 (guess: cine actor) |
| 164 | 0 | | 334 | (-4642, 0, -35777) | | | | | no model (guess: a cine marker) |

NPC init (0x80081bac, switch on char - 2, table 0x80015a38):

- Char 16 (index 14): +0x58 = 0x800834b0 (hit), +0x54 = 0x800833a4 (use),
  +0x6d = 15, +0x688 |= 264 (8 may attack, 256 death effect).
- Char 41 (index 39): +0x688 |= 8.
- Char 9 (index 7): +0x688 |= 64.
- Chars 1, 2, 8 and 22 get nothing.

Speeds (table 0x800b9698 row 0): index 1 = 4, 3 = 12, 4 = 17.

### Scary toys (template flag 0x200000)

Any object whose template has flag 0x200000 and whose think runs
frightens the player:

- The think tests player +0x188 bit 256. When clear: 0x80059570(player)
  and Damage(player, object, object +0x6d) (0x800780ac; the source is
  dropped because it is not a character, so the player's health goes
  down).
- 0x80059570: when the player's template is 2 (Chuckie), five animation
  slots get scared versions: 0x8008a668(2, {258, 259, 257, 256}, slot 3),
  ({262, 263, 261, 260}, 4), ({274, 275, 273, 272}, 1), ({278, 279, 277,
  276}, 2), ({286, 287, 285, 284}, 0). Then +0x188 |= 256 for any player.
- The leave callback 0x80049e5c (the player out of the trigger radius)
  of a 0x200000 template calls 0x800596b0(player): 0x8008a7ec(-1)
  (animations back) and +0x188 &= ~256.

So each approach costs one hit; the next scare needs a leave first (any
scary object's leave clears the bit). Tommy can be frightened too (the
bit and the damage, not the animations).

Which objects:

| template | class | LP | position | trigger | damage | what |
|---|---|---|---|---|---|---|
| 1227 | 2 | 183, 184, 185 | (-1608, -1849, 4508), (-5338, -1848, 1085), (-13706, -1833, 11576), room 0 | 2800 | 5 | spinning toys near the start |
| 1028 | 2 | 140, 141, 142 | (-25328, -1576, -3459) room 14, (-37614, -1576, 1614), (-19700, -1576, 5713) room 1 | 3000 | 5 | spinning tops (seen: striped tops with faces) |
| 1155 | 15 | 153 | (-37364, -2014, 10800), room 1 | 6000 | 2 | a walking robot toy on posts 17-18 (seen) |
| 453, 454, 457 | 17 | 10, 93, 11 | (-11415, -2768, -21129), (-34198, -2391, -25064), (-28354, -2961, -27608) | 4096, 4096, 15000 | none | sound only (below) |

- Class 2 (0x800495d0 -> 0x80048364): think 0x8004e2ac, enter 0x8004cfa0
  (schedule), leave 0x80049e5c, object +0x00 |= 32, angle speeds +0x2c,
  +0x2e, +0x30 = template +0x18, +0x1a, +0x1c (1028 and 1227: 64, 0, 0).
  The think: for templates 1028-1031 and 1227, sound 72 at the object,
  the template's flag 0x200000 set (in the shared template table) and
  +0x6d = 5; then the scare test; then the spin (angles += speed x
  VSyncs since the last think). The flag is set on the first think, so
  the template data alone (flags 2) does not show it.
- Class 15 (0x80049d54 -> 0x80048974): a post walker. +0xa8 = LP +0x46
  (first post, 17), +0xaa = LP +0x48 / 300 (20000 -> 66), +0xac = LP
  +0x4a; think 0x8004dba0, enter 0x80049e18, leave 0x80049e5c. The think:
  sound 72 for template 1155; the scare test (flag 0x200000); a
  collision test (0x8006baa8), with extra tests in level 6 after looking
  up ids 1776 and 506 (purpose not traced); then it walks to its post
  and turns by at most 512 a think.
- Class 17 (0x80049d98): enter 0x8004d174 (sound template +0x24 = 79, 80
  or 81 at the object, +0x00 |= 1), leave 0x80049e5c (clears the scare).
  No think, so no scare and no damage.

### The breakable toys (template 1102, flag 1024)

In the player update (0x8005a068, at 0x8005ac98), level 6 only: when the
player stands on an object (+0xf8), is in state 0, and the object's
template has flag 1024:

1. Template 1102: sound 173 at the object.
2. 0x8004cdfc(object, position, 100) (the model breaks into pieces),
   0x80054ae0(object) (debris: models 421-425, random), the object off
   its room list (it is gone).
3. +0xfd = 0, +0xf8 = 0; 0x800c4268 = -2000, 0x800c4264 = 0x800c426c =
   0; state 16 (thrown up, 0x8005ec9c).
4. Damage(player, 0, 5).

In DB06 only template 1102 has flag 1024: LP 132-136 in room 4 (the
clown area), at y about -1537 and one at (-16917, -3586, -37356). Flags
0x442: also 64 (climbable). The level 6 init clears its use (+0x54 = 0,
0x8004691c). Its box is 1772 on each side (+0x78..+0x84 = -886..886), so
its top is about 1290 above the floor: too high to climb from the floor
(see Climbing). How the player gets onto one is not known (guess: from a
higher spot). Objectives and gameplay pages say level 6 "collects" flag
1024 templates on touch and counts 0x800c4bd6; that is wrong: the code
above is the only level 6 use, and the counter 0x800c4bd6 / HUD
0x800c4298 belong to levels 20-21 (0x80057950 runs only there, from
0x8005b14c; set up by 0x80079cb0 and 0x8007a094).

### Timed bursts (class 10)

Class 10 (0x80049620 -> 0x80048400): think 0x8004e0f4, enter 0x8004cfd8,
leave 0x80049e5c, +0x98 = a 16-byte block from the template (+2 period
= template +0x18). Each think, when more VSyncs than the period passed
since the last burst: for templates 969 and 982, Damage(player, object,
+0x6d) when the player is within 3072 (969: squared 3D distance up to
0x900000) or 2048 (982: 0x400000); then the burst effect +0x50.

| template | LP | position | period | damage | effect (+0x50) |
|---|---|---|---|---|---|
| 969 | 160, 161, 162 | (-29351, -2321, -15776), (-21541, -2321, -16131), (-11634, -2321, -16098), room 4 | 100 | 18 within 3072 | 0x80053ef8: sound 79, particles (models 381-38x) |
| 982 | 6, 7 | (-20228, -2453, -19569), (-25355, -2321, -19536), room 4 | 50 | 18 within 2048 | 0x800539b4: sound 81, particles |
| 905-908 | 43-50 | x -45458..-56793, y -2440..-2680, rooms 2-3 | 50 | none | 0x80054238 / 0x8005444c (particles) |
| 1048 | 127, 154-159 | ids 960-966 at (48021, -2987, -16832..-23154), room 12 | 0 | none | none (draw 0; the piano's note sources, below) |

Template 900 (LP 129-131) sits under the three 969 emitters; the level 6
init sets its y to -1750 (guess: the emitters' bases).

### The piano and the xylophone (touch 15, 0x800522ec)

Templates 999-1006 (ids 3000-3007, x 34315..41472 step 1024, y -1035, z
-18432) and 1007-1011 (ids 3008-3012, y -1079) in room 12 are piano keys;
1046 and 1047 (LP 119-126, x 45568) a second instrument. Trigger 100.
Touch (template +0x0a = 15) for 999-1006, 1046, 1047: 0x800522ec: sound
(template - 999) at the key; for 1046 and 1047 also a burst of note
particles (models 417-420) from a random one of ids 960-966. Ids
3008-3012 have no touch. No level code reads the keys: decoration
(guess).

### The Reptar boxes and the switch

Template 1016 (shape 1422), class 0, flags 0x42 (64: climbable),
pickable (template +0x28 = 1 -> object +0x00 0x10000), +0x30 = 1 (object
0x800000), +0x36 = 2000 (+0xa2). Eight copies:

| LP | position | room |
|---|---|---|
| 94 | (46657, -3324, -28052) | 6 |
| 95 | (34766, -4376, 4074) | 8 |
| 96 | (35255, -4378, -5666) | 12 |
| 97 | (23548, -3863, -23998) | 7 |
| 151 | (20998, -5401, 393) | 8 |
| 152 | (12290, -4376, 11929) | 10 |
| 197 | (42741, -1280, 12863) | 9 (next to the first step) |
| 198 | (25377, -1322, -34822) | 6 |

No level code names template 1016 (the put-down only tests it in level
16). The boxes work through the generic carry and climb code: GRAB
picks one up (state 7), GRAB again puts it down (state 8, 0x80062ac4), and
walking into one climbs onto it (state 6, +0xf8 = +0xf4). Seen: Tommy's y
went from -1269 on the floor to -1775 on a box, so a box is about 506
high.

The switch: LP 163, template 355 (shape 750) at (44063, -6092, 16202) in
room 9, LP +0x44 = 47. The generic init gives it use +0x54 = 0x8004f83c
with +0x6a = 47 (0x80046914 returns 1). ACTION on it: PostCine(47).

Template 1033 (LP 112, 113 at (-4090, -2567, 4596), (-7166, -2567, 4596),
room 0) is climbable (flags 0x42) and not pickable (guess: fixed blocks
on Chuckie's way).

### Other objects

| LP | template | class | id | what |
|---|---|---|---|---|
| 177 | 1027 | 0 | 527 | registered as 527 by the generic init (0x80046e08, also +0x00 0x2000 and template flag 8); 0x8007b670 sets its +0x0c (matrix m11) to -4096 (guess: turned over); cine 46 moves it to (10221, -4072, -35347) |
| 186 | 390 | 0 | | pickable, flight sound 20, +0x6d 25, +0xa2 3500 (guess: a toy to throw) |
| 187 | 1164 | 0 | | pickable; the generic init sets template +0x24 = 20 (0x80046db8) |
| 62-65 | 926 | 0 | | touch 13 (0x80051db4): a burst of particles (models 376-379); flags 0xa |
| 89-92 | 990, 991 | 8 | | class 8 (0x80049708, 0x80048714): +0x18 = 64 / -64, +0x20 = 1024 / -1024; two pairs at the same spots (guess: swinging double doors) |
| 82-88 | 984-989 | 2 | | class 2 spinning, +0x1c = 32 / -32, room 7 (guess: gears); no flag 0x200000 |
| 149 | 464 | 2 | | class 2, spin +0x18 = 15 |
| 98 | 924 | 15 | | class 15 walker on post 10 with LP +0x48 = 100 (speed 100 / 300 = 0), flags 0x2042; outside every room (room 0 by the clamp) (guess: a cine prop) |
| 165-171 | 835 | 12 | | Reptar bars, spawn groups 0-6 |
| 150 | 863 | 12 | | health (kind 4, heal 63), 6 copies, spawn group 7 |
| 13, 39 | 535, 923 | 0 | | LP speech lines 285 and 434 (+0x4c): said on approach (0x8004d0f0; seen at 3352) |

### Collectibles and spawn groups (LP kind 7)

13 kind 7 records (0x800c4bac): entries 0-6 are groups 0-6 with one spot
each (threshold 100); entries 7-12 are group 7 with thresholds 16, 33,
50, 66, 83, 100. The loader places the seven Reptar bars (template 835,
LP 165-171, one per group 0-6) and the six health pickups (template 863,
LP 150, LP +0x46 = 6, group 7) with 0x8004fe14 (see
[level 4](level-04.md#kind-7-spawn-groups-0x800c4bac-16-bytes-each)).
Six pickups for six spots: every group 7 spot gets one.

| entry | group | position |
|---|---|---|
| 0 | 0 | (-13341, -1791, 21509) |
| 1 | 1 | (-22513, -1791, -9229) |
| 2 | 2 | (-16827, -3427, -37315) |
| 3 | 3 | (12338, -1791, -4401) |
| 4 | 4 | (33990, -5652, -1441) |
| 5 | 5 | (31738, -2524, -26627) |
| 6 | 6 | (21098, -1791, 14305) |
| 7-12 | 7 | (-42016, -1657, -23504), (32741, -3658, -23541), (44578, -3459, -36698), (18752, -2667, 4), (-7164, -1657, -28675), (41966, -1657, 3086) |


### Rooms (LEVEL06.BIN, LP kind 6)

15 rooms. Boxes read from RAM in the oracle (room +0x10..+0x28); y is
-7168..-1024 except room 4 (-7168..0) and room 9 (-9216..-1024).

| room | x | z | kind 6 | role |
|---|---|---|---|---|
| 0 | -15360..5120 | -2048..26624 | song 1 | the start; spinners 1227; guard 46; blocks 1033 |
| 1 | -40960..-15360 | -3072..13312 | song 1 | the walking robot 1155, tops 1028 |
| 2 | -57344..-40960 | -21504..4096 | song 1 | phase 2 (0x8007b71c) |
| 3 | -51200..-39936 | -27648..-21504 | song 1 | |
| 4 | -39936..-9216 | -37888..-14336 | song 1 | the clown area: clowns, breakables 1102, bursts 969 and 982 |
| 5 | -9216..0 | -33792..-24576 | song 2 | |
| 6 | 9216..48128 | -37888..-24576 | song 3 | guard 506; boxes 94, 198 |
| 7 | 3072..29696 | -24576..-5120 | song 3 | box 97; gears |
| 8 | 20480..35840 | -5120..15360 | song 3 | boxes 95, 151 |
| 9 | 35840..54272 | 2048..16384 | song 3 | the Reptar room: stairs, the switch, box 197 |
| 10 | 10240..20480 | -4096..13312 | none | box 152 |
| 11 | 0..9216 | -37888..-24576 | cine 46, checkpoint, song 2 | Tommy |
| 12 | 29696..48128 | -24576..-5120 | song 3 | the piano; box 96 |
| 13 | 11264..18432 | -5120..-4096 | none | |
| 14 | -39936..-16384 | -12288..-3072 | none | top 1028 (LP 140) |

Room 11's record sets flag 2 (cine 46) and flag 32 (CheckPoint); room +0x0c
is 0xa2. EpisodeStart sets no room sound bytes.

### The Reptar room floor (LEVEL06.COL)

Floor heights in room 9 by 1024 tile (COL; Tommy stands about 245 above
them: -1269 on the -1024 floor):

| z \ x | 41984..43007 | 43008..46079 | 46080..48127 | 48128..51199 | 51200..54271 |
|---|---|---|---|---|---|
| 14336..16383 | -4736 | -4736 (x up to 46079) | -6144 | -4608 | -4608 |
| 12288..14335 | -1024 | -2048 | -3072 | -2560 | -3584 |
| 4096..11263 | -1024 | -1024 | -1024 | -1024; -2048 from x 50176 (z 5120..11263) | -2048 (Reptar's stage, z 5120..11263) |
| 3072..4095 | -2304 | -2304 | -2304 | -2304, then -1024 from x 49152 | -1024 |
| 2048..3071 | -3584 | -3584 | -3584 | -3584, then -2304 from x 50176 | -2304, -1024 at the east end |

The switch platform is the -4736 block (x 41984..46079, z 14336..16383);
Tommy stood at y -4747 there. The front row at z 12288 is a set of steps
of 1024 and 512; the row at z 2048-4095 has steps of 1280. Seen: Tommy
climbed the first 1024 step (-1024 to -2048) empty-handed. From the code
no single step reaches the -4736 block: from -2048 it is 2688 up, and the
-6144 block (1536 above -4608) separates the -4608 ledge from it. So
boxes are needed as extra steps (guess: stacked boxes at the foot of the
switch platform, or boxes as half steps so Tommy can climb while carrying
the next one; see Climbing). The exact route was not played.

### Posts (LP kind 2, 18 records)

| posts | links | flags |
|---|---|---|
| 1-4 | a line at x 44031, z -6141..-45057 | 0x200 on 1 and 4 |
| 5-7 | a line at x 38905, z 23556..-4624 | 0x200 on 5 and 7 |
| 8-11 | a line at z -14355, x 3098..55280 | 0x200 on 8 and 11 |
| 12-16 | a line at x 12287, z 20479..-45056 | 0x200 on 12 and 16 |
| 17-18 | a line at z 10715, x -36616..-27671 | none; the walking robot |

No NPC in level 6 walks posts 1-16 (the guards are in mode 11; template
924 starts on post 10 but its speed is 0). Flag 0x200 is not explained.

### Level 6 setup in the generic object init (0x8004691c case 5)

0x8004691c switches on level - 1 (table 0x8001369c); level 6 goes to
0x80047874:

1. Template 1028: model box top +0x02 = -1000 (object +0x88 -> +0x0c).
2. Template 1102: use +0x54 = 0.
3. Template 900: y = -1750.

## Start (EpisodeStart, 0x800794a0)

Installed for level 6 by 0x8007871c (case 6). It runs once cine 45 and
card 694 are done:

1. Tommy (1492): 0x80083918 (mode 8, idle; not scheduled).
2. Id 506: 0x80083b4c(npc, player) (mode 11, guard, target the player,
   +0x688 |= 2), range +0x68c = 15000, scheduled (0x80049e18), +0x688 |=
   8.
3. Ids 920, 921, 922: 0x80083b4c(npc, player), scheduled. (920 gets the
   handler's argument, the others 0x800e2b48; both are the player.)
4. Id 46: 0x80083b4c(npc, player), range 5000, scheduled.
5. Respawn = the player's position and angles, phase 1 (0x80078d60).
6. 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 712): the follow
   camera, field of view 712 (as level 5).

The level start, in order: player init on LP 26; cine 45 (0x80074958
case 7, always: no "seen" flag for level 6). Its start hook (0x80072ef0
case 45) does nothing level-specific. Its end hook (0x800732d4 case 45):
0x80074ec0(0, 1776) and hide 1492 (0x80078bc4): Tommy is hidden for phase
1. CineFinish (0x800783d8 case 44): card 694 (HINT23, arg 6). Then
EpisodeStart. 0x8007737c case 44 does nothing.

Seen: Chuckie at (-4154, -1378, 9795) in room 0, heading 3134, health
255, tries 3, +0x30 0x100. Tommy at (187, -1252, 34722), room -1
(hidden).

## Phase 1: Chuckie

What hurts Chuckie, all through Damage (0x80077864, no level 6 routing):

- Scary toys: 5 (1227, 1028) or 2 (1155), once per approach.
- Clowns (mode 11, bite 8), guard 46 (bite 15), guard 506 (bite 6).
  Mode 11 as in [level 2](level-02.md#modes-used-in-level-2): chase
  within +0x68c of home, bite (mode 5, Damage(player, +0x678, +0x6d))
  within +0x690 (1024) with heights overlapping.
- Bursts 969 and 982: 18 each period within range.
- Breakable toys 1102: 5 when stood on.

Throwing things at guard 46 (char 16): its +0x58 0x800834b0 plays
animation 8 (blend 2048) and mode 1 (a stagger); its use +0x54
0x800833a4 also takes the object's +0x6d from its health. In level 6 the
generic hit 0x800511a8 does nothing for char 16 (case 14 at 0x8005138c
returns in level 6). Clowns have no hit callback (template 925 touch 0).
Not checked in the oracle.

## Phase 2: room 2 (0x8007b71c)

Level 6's room change (0x8007f498, table 0x80015894 index 3): room 2,
every entry: 0x80078d60(player position, angles, 2). Nothing else.

## Cine 46: Tommy takes over (0x8007b670)

Room 11 has kind 6 flags 2 and 32. Entering it: PostCine(46) (0x80077aa8
-> 0x80078350) and CheckPoint (0x800772e0: progress +0x24 = position,
+0x20 = health, +0x30 |= 2). Cine 46's start hook (0x80072ef0 case 46)
shows Tommy (0x80078c68(1492)). Its end hook (0x800732d4 case 46):
0x80074ec0(0, 1492). CineFinish (0x800783d8 case 45): card 695 (HINT24,
arg 7). Then 0x8007737c case 45 runs 0x8007b670:

1. 0x8008a7ec(-1): animation slots back to the character's own.
2. 0x800585b0(player, Tommy): the player becomes Tommy (below).
3. Chuckie (1776, now in Tommy's old object): 0x80083918 (mode 8),
   +0x688 |= 64 (in mode 8: the goal is the player, so he turns to watch
   him), scheduled.
4. Respawn = Tommy's position and angles, phase 3.
5. Tries = 3 (0x800c569b).
6. Id 527: +0x0c = -4096.
7. 0x8006eac4(2, 0): song 2.

### The swap (0x800585b0(player, other))

1. A 1644-byte buffer is cleared and gets the first 392 bytes of the
   other object. The other object gets all 1644 bytes of the player. The
   player gets the buffer (the other's first 392 bytes, zeros after).
2. The registry ids are exchanged (0x800462d4 both ways), so 1492 names
   the player and 1776 the NPC.
3. The other (now the old player): its timer freed (0x80072bac), +0x00
   |= 0x4000 (AI), 0x80083918 (mode 8), scheduled (0x80049e18), +0x158 =
   own +0x248, +0x224 = 0, +0x1f4 = 0, leave +0x4c = 0x800820ac, state 0,
   handler 0x8005d11c, animations 0 / 17 reset (0x80057244, 0x80057214,
   0x80057274).
4. The player: +0x00 &= ~0x4000, timer freed, 0x800c5680 = its char
   (1), +0x158 = own +0x248, +0x224 = the camera 0x800de460,
   0x800de4f4 = player, +0x44 = 0, +0x1f0 = 0, 0x8008a42c(char, +0xe8),
   state 0, handler 0x8005d11c, +0x188 &= ~16, animations reset, and the
   camera re-targets (0x80029078).

Seen: health read 0 right after the swap and 255 by the next sample (20
VSyncs later; what sets it is not traced). Tommy stands where cine 46
left him, (40738, -1267, 8097) in room 9, char 1; Chuckie at (39612,
-1378, 7197), mode 8, +0x688 0x40.

## Phase 3: Tommy in the Reptar room

Tommy collects boxes and climbs. The guard 506 (range 15000 around room
6, bite 6 every 16 VSyncs, seen) is near two box spots (LP 94, 198). The
scary toys still cost health (no animation change for Tommy). The level
ambience switches to Tommy's lines (below).

### Climbing (state 6)

Generic code, needed here for the first time in the port:

- The stand and walk handlers (0x8005d11c at 0x8005d764, 0x8005e1f4 at
  0x8005e4cc) call 0x80058da4 then 0x80062cf4; both true: state 6
  (handler 0x8005f310, animation 5), which climbs onto the touched object
  (+0xf8 = +0xf4) or the ledge.
- 0x80056e44 turns the ledge height (+0x144, absolute) into a type: under
  394: 9; under 522: 10; under 650: 11; under 778: 12; under 906: 13;
  under 1034: 14; else -1 (too high).
- 0x80058da4: AI characters with +0x688 bit 1024 never climb; levels 1
  and 12 in room 4 and level 10 type 14 are refused. Carrying an object
  without +0x00 bit 0x1000000 allows only types 9 and 10 (under 522).
  With a touched object (+0xf4): it must have template flag 64. Without
  one: the ledge alone. Type -1: only the level 16 training messages.
- 0x80062cf4 tests +0x130 (under 384; under 220 for char 6) and the
  hierarchy's size.

So Tommy climbs up to about 1033 empty-handed and up to 521 with a box in
his arms. A box is about 506 high.

## Winning: the switch

ACTION (Triangle) with the switch touched (state 12, 0x8005f584 ->
+0x54 = 0x8004f83c): PostCine(47). Cine 47 has no start or end hook
(0x80072ef0 / 0x800732d4 case 47) and no card. After it, 0x8007737c case
46: exit code 0x800c4918 = 1, 0x80076d6c(6, 1) (level 6 played and won,
its puzzle piece, the main-level count; see [objectives](../engine/objectives.md)).
The port's `ENDINGS` already has 47. Seen: the puzzle piece screen
followed.

Nothing checks the boxes: reaching the switch any way wins.

## ForceCry (0x8007fd0c, level 6 case)

Level 6 is case 5 (0x8007ff90), the generic one:

1. A carried object is put down (0x80062ac4(player, 1)).
2. Cry (0x8007f920) when a respawn is set.
3. Tries (+0x23) counted down; health 255.
4. Tries left: to the respawn (0x8007fbf8). None: SwitchEpisode(player,
   1) (0x800781f8, the hub). No level record, no cine.

Respawns: phase 1 where cine 45 left Chuckie; phase 2 where he entered
room 2; phase 3 where cine 46 left Tommy (or room 2's entrance if he
walks there). Cine 46 gives Tommy 3 fresh tries.

The checkpoint (room 11) is for a restart: the loader puts the player
there (0x800454dc); cine 45 still plays first, and entering room 11 again
replays cine 46 (guess from the code; not run).

## How the level ends

- Won: ACTION on the switch, cine 47, 0x80076d6c(6, 1), exit code 1.
- Lost: the last try (ForceCry), SwitchEpisode(1). There is no "lost"
  cinematic for level 6.

## Hint cards

| card | file | shown | text (abridged) |
|---|---|---|---|
| HINT23 | 694 | after cine 45 (arg 6) | Be brave and work your way through the darkened store to find Tommy; getting too close to certain toys will frighten Chuckie; pass safely through the clown area to meet up with Tommy |
| HINT24 | 695 | after cine 46 (arg 7) | Collect the Reptar boxes and place them on the stairs to climb up to the switch and turn on Reptar; the boxes are hidden high and low; # (Circle) to pick up a box and # again to place it; watch out for Thorg |

The port's card table (front.rs `card_after`) has 45 -> 694 and 46 -> 695.

## Per-frame ambience (0x80080a20)

0x800b9618[6]. A countdown 0x800c4cc0 minus dt; at 0:

- Player char 1 (Tommy): with a guard (0x800c4438, the last mode 11 NPC
  to think, 0x80085220) between 1 and 5999 away (+0x674): speech
  0x800b9460[rand & 3] (311, 323, 469, 471), countdown (rand & 255) x 8.
  Else 0x800b9440[(rand & 255) mod 7] (106, 204, 222, 24, 69, 93, 497),
  countdown 1800 + (rand & 255) x 16.
- Player char 2 (Chuckie): 0x800b9450[(rand & 255) mod 7] (278, 282, 286,
  402, 436, 468, 399), countdown 1800 + (rand & 255) x 16.

Seen with Chuckie: speech 278, 286, 402 at 3512, 4497, 5353. Speech only.

## Other notes

- The stick speed factors 20 / 10 (0x800c4210 / 0x800c4214) in 0x8005be24
  and 0x80059008 / 0x800590d4 are for character 6 (Spike, 0x800c5680 =
  6), not level 6; character 9 uses 0x800c421c / 0x800c4220. Chuckie and
  Tommy use 17 / 8.
- 0x8007737c (cine end) and 0x800783d8 (CineFinish) switch on cine - 1;
  the hooks 0x80072ef0 and 0x800732d4 switch on the cine number.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, 16-bit pokes, hooks with
arguments, object dumps, room dumps, screenshots): presses start 300,
cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 6`), 0x800c5678 = 6 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 2 and
0x800c5680 = 2. The start cine was not skipped.

- Cine 45 from 1819 to 3332; at 3332 card 694 (arg 6), EpisodeStart
  0x800794a0, respawn (-4154, -1378, 9795), phase 1, tries 3. Chuckie is
  the player (char 2). Tommy hidden (room -1). Guards in mode 11: 46
  (+0x688 0x10a, range 5000, +0x6d 15), 506 (0x0a, 15000, 6), 920 (0x0a,
  7000, 8); 507 and 15 idle.
- Scares: 1227 at 3416 (0x80059570 from 0x8004e384, Damage 5), cleared
  at 3701 on leaving (0x800596b0 from 0x80049edc); 1028 at 3708 (5); the
  walking robot 1155 at 3400 (2, from 0x8004dc38), cleared when it walked
  away (3643). A class 17 object played sound 81 on approach.
- Room 2 at 4002: 0x8007b71c, phase 2, respawn there.
- Clown 920: Damage(player, player, 8) every 46 VSyncs from 4352. Guard
  46: 15 every 36 VSyncs. Guard 506 (Tommy near it): 6 every 16 VSyncs.
- Bursts 969: Damage(player, emitter, 18) every 104 VSyncs (6408, 6512 ..)
  with Chuckie about 1250 away.
- Health at 0 (guard 46) at 5698: ForceCry, tries 3 -> 2, the cry, at
  6118 the phase 2 respawn with health 255.
- Tries poked to 1, a clown bite at 3 health: ForceCry at 4354, tries 0,
  at 4774 SwitchEpisode(player, 1); no 0x80076d6c.
- Room 11 at 3402: PostCine(46) and CheckPoint (+0x30 0x102). Cine 46
  from 3429 to 4640; card 695 (arg 7); 0x8007b670, 0x800585b0(0x800e2b48,
  0x800ecb74); Tommy (char 1) at (40738, -1267, 8097) room 9, phase 3,
  tries 3, song 2; Chuckie mode 8, +0x688 0x40.
- Tommy picked up box LP 197 (state 7, 5060-5110), put it down (state 8,
  0x80062ac4 at 5230), and walking into it climbed onto it (state 6, +0xf8
  = the box, y -1269 -> -1775). Empty-handed he climbed the first step
  (-1024 to -2048, state 6).
- Tommy poked onto the switch platform (y -4747), Triangle: 0x8004f83c on
  the switch (0x800f6298) at 5118, PostCine(47), cine 47 from 5120 to
  5164, 0x80076d6c(6, 1), exit 1, the puzzle piece screen.
- Ambience speech 278, 286, 402 (Chuckie's table).
- Not checked: standing on a breakable toy (two drops onto 1102 objects
  landed on the floor or the shelf, never on the object); the full box
  route to the switch; a throw at guard 46; bursts 982; Tommy's cry and
  respawn in phase 3; a restart from the checkpoint; what ids 507, 15,
  527 and template 924 do in the cines.

## For the port

The port has a Script per level, hiding by id, cinematic-object use,
doors, room flags 2/4/32/64/128, the follow camera with heights and kind
3 fixed cameras, NPC modes 0, 1, 3, 4, 6, 7, 9, 11 and riders, posts,
carrying and throwing with strikes, hazards, healing collectibles, spawn
groups, movers, COL floor flags, the level timer, gauges and the meter,
ForceCry with tries, cards, the result screens. Level 6 needs, in order:

1. Chuckie as the player: `player_character(6) = 2` (0x80018598; also
   10). Today the port picks Tommy, so the player lands on LP 15 in room
   11 and room 11's cine 46 plays at once (this is the cine 46 seen at
   level start in the port's runs). Play starts where cine 45 leaves
   Chuckie: (-4154, -1378, 9795), heading 3134.
2. A `level6` script: phase, respawn, tries. EpisodeStart 0x800794a0
   (guards 46, 506, 920-922 into mode 11 with their ranges, Tommy mode 8,
   fov 712); cine 45's end hides 1492; room 2 (phase 2); cine 46's start
   shows 1492; cine 47 won (already in `ENDINGS`). The cards are already
   in `card_after` (694, 695). ForceCry is the default.
3. The player swap at cine 46's end (0x8007b670 / 0x800585b0): the
   player body, animations and character become Tommy's at Tommy's
   position; Chuckie becomes an NPC in mode 8 that turns to face the
   player; ids exchanged; respawn there, phase 3, tries 3, song 2, the
   camera re-targeted.
4. Climbing (player state 6): ledges up to 1033 high (types 9-14 by 128
   steps from 394), only under 522 while carrying; onto objects with
   template flag 64 (boxes 1016, blocks 1033, toys 1102), which then
   carry the player (+0xf8). Needed for the stairs and the boxes.
5. Scary toys: template flag 0x200000 (set at run time for 1028-1031 and
   1227 by their think) arms with the trigger radius; the first think
   with player +0x188 bit 256 clear: Chuckie's scared animation slots
   (0x80059570), the bit, Damage +0x6d (5 or 2); leaving the radius of
   any such object clears the bit and restores the animations. Class 2
   spinning toys (spin from template +0x18..+0x1c, sound 72), class 15
   post walker 1155 (speed LP +0x48 / 300, sound 72), class 17 approach
   sounds 79-81.
6. Timed bursts (class 10, 0x8004e0f4): every template +0x18 VSyncs a
   particle burst; 969 and 982 hurt 18 within 3072 / 2048 (3D).
7. Breakable toys: standing on a template with flag 1024 in level 6:
   sound 173 (1102), break and debris, gone, state 16 bounce (-2000),
   Damage 5.
8. Guards: mode 11 exists; set the ranges from EpisodeStart (506 15000,
   46 5000), bites from +0x6d (8, 15, 6), +0x688 bits (8 attack). Guard
   46's hit callback (stagger, mode 1) if throws at it matter.
9. Small things: the ambience speech by character (0x80080a20); LP
   speech lines on approach (+0x4c); the piano keys' notes (touch 15) and
   note particles; touch 13 particles; id 527's flip at cine 46's end;
   template 900's y; template 1028's box top -1000.

## Unknown

- Which guard is Thorg (46, char 16, or 506, char 22) and what chars 9,
  8, 22 and 41 look like; the clowns are a guess from HINT23.
- How the player gets onto a breakable toy 1102 (its top is about 1290
  above the floor) and whether walking into its side does anything.
- The intended box route to the switch platform (the COL table above;
  how many boxes and where).
- What sets the player's health to 255 just after the swap.
- What ids 507, 15, 334 and 527 and template 924 are for (cine props,
  guess), and what posts 1-16 and their flag 0x200 are for.
- The extra level 6 collision tests of the class 15 walker against ids
  1776 and 506 (0x8004dc4c..0x8004dccc).
- The class 8 objects 990 / 991 and the class 10 objects 905-908: what
  they look like.
- How long cines 45, 46 and 47 really are (the oracle: 1513, 1211 and 44
  VSyncs).
- Card arg 7 (HINT24) against arg 6.
