---
title: Level 12, Let There Be Light
status: partial
discs: US
covers: US SLUS_006.50:0x8007b7c8, US SLUS_006.50:0x8007bb84, US SLUS_006.50:0x8007bc24, US SLUS_006.50:0x80083330, US SLUS_006.50:0x80083764, US SLUS_006.50:0x8004f83c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x80078dcc, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005f4cc, US SLUS_006.50:0x80059d3c, US SLUS_006.50:0x80059cdc, US SLUS_006.50:0x80091834, US SLUS_006.50:0x800908e8, US SLUS_006.50:0x8009134c, US SLUS_006.50:0x8009173c, US SLUS_006.50:0x80091590, US SLUS_006.50:0x80090e70, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80083be0, US SLUS_006.50:0x80083b4c, US SLUS_006.50:0x80083918, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x80048a44, US SLUS_006.50:0x8004cf58, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x800601e4, US SLUS_006.50:0x80060910, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x8004822c, US SLUS_006.50:0x800596e8, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004860c, US SLUS_006.50:0x80058da4, US SLUS_006.50:0x80053344, US SLUS_006.50:0x800812c8, US SLUS_006.50:0x80078e3c, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x80018598, US SLUS_006.50:0x8008c7f0, US SLUS_006.50:0x8008ca80, US SLUS_006.50:0x8008c9f4, US /DATA/DB12/DB12LP.BIN, US /DATA/DB12/LEVEL12.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 12, Let There Be Light

Everything level 12 needs to be played to its end: Tommy alone in the
dark Pickles house with a flashlight. He goes down to the kitchen past
the Shadow monsters, tries the refrigerator, then searches the house and
yard for Spike. Read from the pseudo-code and the disassembly; the start,
the light on a monster and its return, a bite, a battery, the
refrigerator, the light picked up again, the win (Spike found, cine 68),
a cry and the last-try loss were run in the oracle (see Checked in the
oracle). Builds on [level 5](level-05.md) (the remote: a carried class 13
object with a gauge and a beam), [level 10](level-10.md) (the same house,
spots from spawn groups), [level 11](level-11.md), [gameplay](../engine/gameplay.md)
and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c = template +0x10). A VSync is 1/60 s; the
oracle's frame counter counts VSyncs. The player update runs once a game
frame (every second VSync).

## The goal in one paragraph

The player is Tommy (char 1, 0x80018598 default case). Cine 66 "Let There
Be Light Intro", card HINT32 (file 703): get the light out of the
refrigerator, find the kitchen safely, shine the flashlight on the Shadow
monsters with Triangle, collect batteries. EpisodeStart puts the
flashlight (id 512) in his hand with 120 charge. Holding Triangle shines
it: one charge a game frame. Eight Shadow monsters (char 43) guard their
spots and bite for 20; the light makes one vanish and come back at a
random spot. Batteries give 30. Using the refrigerator door (id 601 or
602) in the kitchen plays cine 67 "Let There Be Light Mid": the fridge
light will not come out; Tommy drops the flashlight. Card HINT34 (file
705): search the house and yard for Spike (id 14), press Triangle to get
his help; do not forget the flashlight. Spike now waits at a random spot
of six. Using Spike plays cine 68 "Let There Be Light End" and the level
is won. Each cry costs a try; Tommy stays where he is (the level has no
respawn). The third cry ends the level.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: never written in level 12 (no 0x80078d60 call); 0 |
| 0x800e583c | u32 | respawn set: 0 the whole level (cleared by the runner, 0x80017c0c -> 0x80078dcc) |
| 0x800c4df8 | ptr | the flashlight's gauge (0x8008c7f0); 0 after cine 67 or 68 starts; a freed pointer after a put-down |
| player +0x1f0 | ptr | the carried object: the flashlight (id 512) or 0 |
| flashlight +0x6d | u8 | charge, template 933 +0x32 = 120 |
| flashlight +0xa2 | u8 | cost a frame, template 933 +0x36 = 1 |
| ids 601, 602 +0x54 | fn | the refrigerator's use: 0x8004f83c (PostCine(+0x6a)) until cine 67, then 0 |
| ids 601, 602 +0x6a | u8 | 67; 602's becomes 68 at the win (0x8007bc24) |
| id 14 +0x54 | fn | Spike's use: 0 until cine 67, then 0x80083330 |
| progress +0x23 (0x800c569b) | u8 | tries; 3 at the start |
| 0x800c4cc0 | s32 | countdown (dt) to the next ambience line (0x800812c8) |

Level 12 has no level timer (0x8007f5a8 table 0x800158cc index 10 is the
default 0x8007f6c8), no room change case (0x8007f498 table 0x80015894
index 9 is the default 0x8007f598), no CheckPoint room, no room cine and
no Gameplay script (no kind 6 record has those bytes). The progress of
the level is held only by the use callbacks of 601, 602 and 14 and by
who carries the flashlight.

## Data in DB12

### The characters (LP kind 1)

The loader (0x800454dc) puts the player on LP 202, the first kind 1
record whose shape is char 1.

| LP | template | char | id | position | what |
|---|---|---|---|---|---|
| 202 | 11 | 1 | 1492 | (30898, -5381, 14662) | Tommy, the player; cine 66 leaves him at (29611, -5365, 15428), heading 1024, room 3 |
| 173 | 9 | 6 | 14 | (29973, 320, 11157) | Spike, in the kitchen; hidden by cine 66's end, placed by 0x8007bb84 |
| 194, 203-209 | 934 | 43 | 3333-3340 | below | the Shadow monsters |
| 161, 160 | 10, 3 | 8, 13 | 15, 16 | (21384, -1279, -13700), (19660, -1490, -14781) | no level code (guess: cine 66 actors, as in level 10) |

Level 12 swaps two bodies by level number: the player's hierarchy is 44,
not its character (player init 0x800596e8 at 0x8005981c), and an NPC of
char 6 gets hierarchy 45 (0x8004822c at 0x80048284). Guess: Tommy in his
night clothes and a dark Spike. The hand part's model is 1690 (holding the
flashlight) or 1310 (empty) (0x80044b74 into the part named by hierarchy
byte +0x23, +0x58).

### The flashlight (template 933, id 512)

LP 193 at (30342, -5337, 13180). Template 933: class 13 (+0x00 bit
0x1000000, carried but not thrown), pickable (+0x00 0x10000, template
+0x28 = 1), +0x32 = 120 (charge, +0x6d), +0x36 = 1 (cost, +0xa2),
trigger 5000. Seen object flags 0x01010000.

### The Shadow monsters (template 934, char 43)

Template 934: class 1, health 150 (+0x18), range 10000 (+0x1a), speed
index 2 (8), bite 20 (+0x32), touch 9 (+0x58 = 0x800511a8). NPC init
(0x80081bac case 41, char 43): +0x688 |= 0x8000 and template flag 8 (no
object collision). EpisodeStart sets each to guard. Seen in RAM at 2300:
mode 11, +0x688 0x800a, reach (+0x690) 1024, speed 8, health 150.

| id | LP | position (RAM) | room |
|---|---|---|---|
| 3333 | 194 | (10477, -848, 9192) | 25 |
| 3334 | 203 | (16914, -848, -3706) | 16 |
| 3335 | 204 | (25611, -848, 12293) | 19, the kitchen |
| 3339 | 205 | (28661, -848, 9209) | 19, the kitchen |
| 3336 | 207 | (28163, -6992, 12) | 13 |
| 3337 | 208 | (18436, -9791, 1845) | 8 |
| 3338 | 209 | (16479, -6992, 6945) | 7 |
| 3340 | 206 | (37389, -6992, -2049) | 2 |

They hover 1872 above the floor: their feet (hierarchy +0x4c) are -1872,
and each is settled at the floor under its LP place plus that
(0x800452f8: LP y -1872 over the downstairs floor 1024 is -848, -7919
over the upper floor -5120 is -6992). Four guard downstairs, four
upstairs.

### The refrigerator (templates 226, 227)

| LP | template | id | position | class |
|---|---|---|---|---|
| 116 | 226 | 602 | (30720, -611, 14314) | 4, hinged door (shape 351), trigger 1000 |
| 117 | 227 | 601 | (30721, -82, 12293) | 4, hinged door (shape 352) |
| 230, 231 | 226, 227 | 510, 511 | (30720, 7574, 14314), (30721, 8103, 12293), room 44 | copies below the house (guess: for a cine) |

Class 4 (0x8004860c) gives use +0x54 = 0x8004f4a0 (swing open);
EpisodeStart replaces it on 601 and 602 (below).

### Spawn groups (LP kind 7)

36 records (0x800c4bac, 16 bytes; x +0x04, y +0x08, z +0x0c, group +0x10,
threshold +0x11, post +0x12 all 0).

| entries | group | thresholds | positions | use |
|---|---|---|---|---|
| 0-5 | 0 | 16, 33, 50, 66, 83, 100 | (49124, 75, 26645), (-971, -5905, -4472), (19440, -5905, -2999), (37916, -5905, 11), (27678, -5905, 11251), (3, -5905, 8240) | Spike's spot after cine 67 (one in the yard, five upstairs) |
| 6-12 | 1-7 | 100 | (6643, -6755, 3143), (41496, 602, -3070), (29878, 602, 20116), (16393, -5556, -8720), (49698, 581, 19453), (25584, -5518, 17414), (52653, 965, 24417) | Reptar bars (template 835, LP 244-250) |
| 13-17 | 8 | 20 .. 100 | (28067, 602, 14821), (36919, 602, 9048), (15919, 602, -6088), (37085, 602, 816), (5083, 602, 18104) | health (LP 243, template 863, kind 4, +0x6d 63, 5 copies) |
| 18-22 | 9 | 20 .. 100 | (4995, 602, 9594), (24526, 602, -9188), (39134, 602, -7136), (46383, 602, 7386), (17997, 602, 21034) | batteries (LP 242, template 87, kind 12, +0x6d 30, 5 copies) |
| 23-35 | 10 | 7, 15, 23, .. 100 | upstairs y -5518: (6145, 9110), (14921, 6126), (33761, 2084), (25528, -2009), (7997, -2533), (31722, 12293); downstairs y 533: (2010, -9200), (35830, 2077), (14393, 20447), (23534, -9253), (35832, 16030), (11238, 21537), (12322, 25647) | where a Shadow monster comes back |

Seen at load (0x8004fe14 from 0x80046008 at 1818): groups 9 (5), 8 (5),
7..1 (1 each). Groups 0 and 10 are filled later. In level 12 a group 10
spot is never marked used (0x8004fe14 at 0x80050004), so two monsters can
come back on the same spot.

### Rooms (LEVEL12.BIN, LP kind 6)

45 rooms; boxes read from RAM (room +0x10..+0x28). The same house as
level 10.

| rooms | y | role |
|---|---|---|
| 0-14, 43 | -10240..-5120 | upstairs; Tommy starts in room 3 (x 24576..40960, z 10240..18432) |
| 15-23, 25-31, 33-36, 42 | -4096..1024 | downstairs; room 19 (x 21504..32768, z 6144..20480) is the kitchen with the refrigerator |
| 24, 37-41 | -11264..1536 | the yard (z 23552..46080) |
| 32 | -11264..-6144 | x 44032..53248, z 13312..23552 (not traced) |
| 44 | 4096..9216 | below the house; holds 510 and 511 (guess: a cine set) |

Kind 6 records set camera kinds 1, 6 and 8 (follow 1500, 200 and 1600;
see [level 5](level-05.md#camera-in-level-5)), flag 16 on many rooms and
songs 1-3. Bytes +0x5c 2 and 3 on rooms 9 and 27 are not used (no kind 3
room). The three posts (LP kind 2: 2, 3, 19) are camera posts (flag 8)
that no room uses. The one kind 99 record has no loader case.

EpisodeStart's room sound bytes (0x80078e3c fills an empty +0x64 and sets
+0x0c bit 128): (27, 1) no effect (27 has song 2), (46, 2) no effect (no
room 46; the byte there is not 0), (10, 2) and (17, 1) take (seen: room
10 song 2, room 17 song 1, both +0x0c 0x80).

### Level 12 in shared code

- Generic object init (0x8004691c, table 0x8001369c index 11 =
  0x800470bc):
  1. Template 1070 (LP 91, id 116, in the yard, room 24): its second
     model box becomes (-900, -339, -1200)..(1500, 1463, 1200).
  2. Template 381: box top +0x42 = -200, template flag 64 cleared (not
     climbable). No template 381 record in DB12.
  3. Template 1271 (LP 255 at (24130, -1312, 161)): four 168-byte copies
     at (21900, -2380, 161), (19650, -3440, 161), (17400, -4500, 161),
     (15200, -5550, 161), room from position 27 (guess: the stairs'
     steps).
  4. Template 960 (LP 24 at (36606, 521, 14886)): template flag 64
     cleared, first box min y -3000.
  5. Templates 954 (LP 112, id 80) and 678 (LP 170): template +0x24 = 20
     and 167 (meaning not traced).
- Batteries (0x80048a44 case 12 at 0x80049168): in level 12 the enter
  callback +0x48 = 0x8004cf58 and the trigger +0xa0 = 1000: a battery is
  taken when Tommy comes within 1000 (the trigger pass 0x80072d38), not
  by touch. 0x8004cf58 calls its +0x58 (0x800514b0) and leaves the
  trigger list.
- 0x800514b0 kind 12 (0x80051c68): sound 109 in level 12 (69 in level 5);
  carried +0x6d += 30, then = 0x8008ca80(0x800c4df8, value), which clamps
  to the gauge's max. No test that anything is carried (below).
- Use callback 14 (0x80053344) returns at once in level 12 (no sounds;
  no level 12 object uses it).
- Climbing (0x80058da4 at 0x80058e04): in levels 1 and 12, in room 4 every
  climb is refused.
- GRAB with something carried puts it down in level 12 (only levels 5, 20
  and 21 skip that, 0x8005d308).

## Start (EpisodeStart, 0x8007b7c8)

Installed by 0x8007871c (case 12). The level start, in order: player init
on LP 202; cine 66 (0x80074958; no start hook, 0x80072ef0 case 66 does
nothing); its end hook (0x800732d4 case 66): hide 14 (Spike), the hand
part's model = 1690, then the default 0x80074ec0(0, player id);
CineFinish (0x800783d8 case 65): card 703 (HINT32, arg 6); EpisodeStart;
0x8007737c case 65 does nothing.

0x8007b7c8(player):

1. The flashlight (id 512): player +0x1f0 = it, its draw +0x40 = 0 (the
   hand model shows it). It is not taken out of its room list and has no
   grip computed (level 5's 0x8007b280 does both).
2. The gauge: 0x8008c7f0(360, scale (36, 64, 1024), pos, 361, scale (36,
   54, 1024), pos, max = the flashlight's +0x6d (120), 1), pos =
   (((0x800c4a04 x 5) >> 7) - 192, 88 - (0x800c4a06 >> 4)); kept in
   0x800c4df8. Seen: the battery icon bottom left, as in level 5.
3. Ids 602 and 601: use +0x54 = 0x8004f83c, +0x6a = 67.
4. Ids 3333-3340: 0x80083b4c(npc, player) (mode 11 and back mode 11,
   target the player, +0x688 |= 2, speed from the table), enter +0x48 =
   0x80049e18, put on the trigger list 0x800c4c24, +0x688 |= 8.
5. Room sound bytes (27, 1), (46, 2), (10, 2), (17, 1) (above).

There is no respawn, no phase and no camera call (the runner's follow
camera stays; seen camera +0x4a 1).

Seen: cine 66 from 1819 to 2274; at 2274 card 703 (arg 6), EpisodeStart,
flashlight in hand, gauge 120 of 120; Tommy at (29611, -5365, 15428), room
3, heading 1024, health 255, tries 3; Spike in room -1.

## The flashlight

### Shining (0x8005a068 at 0x8005a6f4)

In the player update, carrying an object with +0x00 bit 0x1000000:

- Template 933 and Triangle held (0x800c49d4 bit 0): on the press (`& 3
  == 1`) sound 161 (0x8006f26c, not positional); every game frame while
  held, 0x8005f4cc -> 0x80059d3c(player, the flashlight's position, 100).
  No animation is played.
- Any other class 13 object (level 5's remote): on the press only,
  animation 39 and the zap.

0x80059d3c with charge (+0x6d) above 0 and template 933 (at 0x80059ef8):

1. The direction player +0x200 (0x80059cdc): toward the aim target
   +0x1f4 (0x80075e4c with 1: the target's +0x18 less the player's, made
   a unit by 0x8007652c) while the target's size +0x36 is above 0, else
   from the facing (0x80076040: the angles +0x24). The aim target is
   kept by 0x80059b34 as in every level (the nearest character within
   512 of the heading and 768 of the pitch; see
   [objectives](../engine/objectives.md)), so a Shadow monster ahead
   draws the light up toward it.
2. The matrix of hand bone 15 (0x800bb2e2, 0x80044fec).
3. 0x80091834(player, matrix, +0x200, 100 + the z of the body's step
   +0xd8..+0xdc turned by its angles), beam type 1: start at bone 15 +
   (0, 0, 350) (table 0x800bb28c entry 1, +0x24), speed +0x44 = the last
   argument. The step's z is 0 on the ground (seen walking: only +0xda,
   the lift, moves). On a frame whose animation event 2048 fired
   (+0xe0 bit 2048), 0x8005f4cc calls it with 2048 instead of 100 and
   clears the bit: that shot is about 84,000 long (seen walking, three
   such frames in 24).
   The handler runs in the player's update, which comes before the
   characters' (0x80018c38): the facing is the heading before the body's
   turn this frame, the target and its size (+0x36) as they were before
   the characters' step; the shot is stepped after them (0x8009173c).
4. Charge -= +0xa2 (1); gauge = 0x8008ca80(gauge, charge). The result is
   not stored back.

With 0 charge nothing happens (sound 161 still plays on the press).

### The light (beam type 1)

Beam pool: 8 records of 92 bytes at 0x800e6ef0. The update loop 0x8009173c
calls table 0x800baedc by the type's byte +0x09 (type 0 0x80090fac, type
1 0x8009134c).

- Move (0x8009134c): step = direction (made a unit again, 0x8009f278) x
  ((+0x44 << 10) / 100) >> 10: the unit, 4096 long, when standing. The
  segment from the old to the new position is tested (0x80090860 ->
  0x80090554): the nearest object or character it crosses in the rooms
  along it (0x80090340 -> 0x8006a9cc over room lists +0x48 and +0x4c:
  not the owner, nor a template of flag 16; flag 8 does not matter
  here; the boxes as 0x8006a714 has them, not grown), unless the
  world's segment (0x8002cd7c) is nearer on all three axes. The one
  met, if it has a +0x58 callback (and is not a character with health
  0 or less), gets +0x58(object, hit point). One without a callback
  still stops the shot. The shot's place is not moved to the hit.
- The start (0x80091834): hand slot 15's part matrix as last drawn
  (+0x158 + 32 x part, 0x80044fec; transposed by 0x80010b54) turning
  (0, 0, 350), plus the part's joint.
- Draw (0x80091590 type 1): 0x80090e70 draws mesh 12/13 at the midpoint
  of the two points, scaled 15000 across and the length along
  (0x8002474c), turned by 0x8008e6c8 of the way (end less start), which
  points its +z back at the hand; then the shot is freed (0x80090280).
  So the light is a new 4096 shot each game frame, drawn once.

Seen (the port against the oracle, `level_12_light_shot_as_the_original`,
`level_12_light_strikes_as_the_original`): standing at the start, every
shot's start and end are the original's for 20 frames; facing a kitchen
monster 2000 off, the light goes up toward it and strikes it on the
first shot.

Seen: Triangle held 80 VSyncs: charge 120 -> 80 (40 game frames), sound
161 once. Held 60 VSyncs: 120 -> 90.

### Hitting a Shadow monster (0x800511a8 case 41)

Char 43: mode 10 (dying), state 19, the hierarchy's first record +0x04
..+0x10 zeroed, sound 110 at the monster. Health is not touched.

The zeroed hierarchy record +0x04..+0x10 is the box the collision tests
read (0x8006a008), so a struck monster meets nothing.

Mode 10 (0x80084084 at 0x80084314) with +0x688 bit 0x8000: +0x36 -= 250
a frame while above 0 (4096: 17 frames, to -154); +0x36 is the draw
scale (0x8003205c: each part's matrix times the scale's, the joints
kept), so the monster shrinks to nothing where it was struck. The
frame +0x36 is found at 0 or less: speech 143 or 21 (rand & 1,
0x8008da4c) and 0x80083be0 case 43:

1. Mode 0 (+0x699), +0x36 = 4096; 0x80083b4c (guard the player again),
   enter callback 0x80049e18, +0x188 |= 0x4000, +0x688 |= 8.
2. The hierarchy record +0x04..+0x10 copied back from 0x800c4b70 +0xac.
3. 0x8004fe14(&npc, 10, 1): a random group 10 spot (not marked used);
   home (+0x648..+0x650) = the new position; scheduled; state 0. The
   spot is settled by 0x800452f8: the floor under the spot itself
   (0x8002b240 at +0x18), and the height that floor plus the feet
   (hierarchy +0x4c, -1872 for the monsters): spot (6145, -5518, 9110)
   puts one at -6992 over the upper floor (-5120).

Seen: a hit at 2443, speech 143 at 2479 (36 VSyncs), the monster at
(31722, -6992, 12293) in room 3 (entry 28), mode 11 with its new home.
Another hit at 2563 / 2565 on both kitchen monsters: speech 143 and 21 at
2599 and 2601.

## Shadow monsters: mode 11 (0x80084084)

Mode 11 as for level 5's robots (see
[level 5](level-05.md#tv-robots-mode-11-0x80084084)): chase the player
within range (10000) of home, bite (mode 5) within reach (1024), else go
home. Speed index 2: 8. The bite: sound 111 at the monster (0x80085808),
Damage 20 through the event queue (0x80077864 from 0x800771b4).

Seen: Tommy in the kitchen: bites at 2439 and 2535 (96 VSyncs apart),
health 255 -> 235 -> 215; with both kitchen monsters at him, two Damage
calls 2 VSyncs apart each time (40 a bite). 13 bites of 20 empty 255
health.

## Batteries

Taken within 1000 (above): 0x8004cf58 from the trigger pass
(0x80072ec8), 0x800514b0, sound 109, charge + 30 clamped to the gauge's
max. Seen at 2500: charge 90 -> 120 at the spot (39134, 602, -7136); the
same at (4995, 9594) and (46383, 7386). Teleporting to (24526, -9188) and
(17997, 21034) pushed Tommy away from the spot (walls) and nothing was
taken.

Without the flashlight in hand (after cine 67, before it is picked up)
the battery is still taken: the original adds 30 to the byte at address
0x6d (carried pointer 0, +0x6d) and calls 0x8008ca80(0, 30). Seen: no
crash in the oracle, byte 0x6d = 30, the battery gone.

## The refrigerator: cine 67

ACTION (Triangle) with a refrigerator door touched: the flashlight is
class 13, so ACTION uses the touched object (0x800631d8, the nearest of
the nearby list within reach and a quarter turn: the door upstairs above
it is no use; state 12, animation 50); at
the animation event (0x8005f584 -> 0x8005f614) +0x54 = 0x8004f83c(door,
player): PostCine(player, +0x6a = 67, 0) (0x80078350). The light also
shines while Triangle is held. Circle would put the flashlight down
first (GRAB step 1); with empty hands Circle calls the door's +0x54 at
once (objectives, GRAB step 4; not checked).

Cine 67's start hook (0x80072ef0 case 67, at 0x80073240), when carrying:
the hand model = 1310, the flashlight's draw = 0x800307dc(1), the gauge
freed (0x8008c9f4), 0x800c4df8 = 0, player +0x1f0 = 0. The hook does not
move the flashlight and gives it no flight. Seen in hand at (30025, -366,
12947) just before, and on the kitchen floor after the cine (below;
guess: the cine moves it).

Its end hook is the default. CineFinish (0x800783d8 case 66): card 705
(HINT34, arg 3) and the callback 0x8007bb84, run after the card:

1. Ids 602 and 601: use +0x54 = 0 (the refrigerator is done).
2. Id 14 (Spike): +0x678 = id 1492 (the player), use +0x54 = 0x80083330,
   0x80083918 (mode 8 and back mode 8, idle, speed 0), scheduled,
   0x8004fe14(&spike, 0, 1): a random group 0 spot, marked used; speed
   index +0x66e = 3.

0x8007737c case 66 does nothing.

Seen: Triangle at 2440 at (30000, 425, 13300) facing +x: state 12, door
601's use at 2459, PostCine(67); cine 67 from 2488 to 3193; the flashlight
dropped at 2488 (gauge freed) and seen after the cine at (29693, 578,
14231) in room 19; Tommy at (28657, 779, 13322), heading 1570; at 3193
card 705 (arg 3), 0x8007bb84, Spike at entry 2 (19440, -5808, -2999),
room 1, mode 8. Spike stayed there (seen at 4000 and 6000).

## Picking the flashlight up again

GRAB (Circle) with the flashlight touched and empty hands: pick up (state
7, 0x800601e4). Template 933 (at 0x800602d8):

1. A new gauge: 0x8008c7f0(360, .., 361, .., max 100, 1), then
   0x8008ca80(gauge, charge). The max is 100, not 120.
2. The hand model = 1690; the flashlight's draw +0x40 = 0.

Seen: Circle at 3340 beside it: state 7 from 3344, the gauge at 3370;
charge 118, gauge value 100 of 100. The charge is not clamped: it stays
118 until a battery clamps it to 100.

Put down (GRAB while carrying, state 8, 0x80060910 template 933): gauge
freed (0x800c4df8 is not cleared), hand model 1310, draw 0x800307dc(1),
then the generic put down (0x80062ac4). Seen: Circle at 2400 in room 3,
the gauge freed at 2431.

ForceCry and the cine start hook 0x80072ef0 only put down a carried
object whose draw +0x40 is not 0; the flashlight's is 0, so a cry never
drops it.

## Winning: Spike (cine 68)

ACTION (or GRAB with empty hands) with Spike touched: +0x54 =
0x80083330(spike, player):

1. 0x80083764(spike, +0x678 = the player): mode 4 and back mode 4
   (follow the player), speed from the table, think 0x80084084.
2. Scheduled (0x80049e18).
3. Level 12: 0x8007bc24: id 602's +0x6a = 68; 0x8004f83c(602, player):
   PostCine(68).

Cine 68's start hook (0x80072ef0 case 68) drops the flashlight like cine
67's. Its end hook is the default; no card. 0x8007737c case 67: exit
code 0x800c4918 = 1, 0x80076d6c(12, 1) (level 12 played and won, its
puzzle piece; see [objectives](../engine/objectives.md)). The port's `ENDINGS`
already has 68.

Seen: Tommy put at (18740, -5400, -2999) facing +x beside Spike in room
1; Triangle at 3540: 0x80083330 at 3556, 0x80083764, 0x8007bc24,
PostCine(68); cine 68 from 3559 to 4269; at 4269 0x80076d6c(12, 1),
0x80094bfc(12), the puzzle screen 0x8009480c(12, 1).

## ForceCry (0x8007fd0c, level 12 case)

Generic first part: a carried object with draw +0x40 not 0 is put down
(not the flashlight); tries counted down (0x800c569b - 1 when not 0).
Level 12 is case 11 (0x8007ffc0): the cry (0x8007f920) always; health
255; no callback. Tries left: to the respawn only if one is set
(0x800e583c bit 0); level 12 never sets one, so Tommy stays where he
cried. None: SwitchEpisode(player, 1) (0x800781f8, the hub).

Seen: health poked to 30 near the kitchen monsters: ForceCry at 2441
(from 0x80077a40), the cry, tries 2, no 0x8007fbf8, the same position;
health 0 during the cry (0x800c56a8 bit 0x20 set) and 255 at 2900. With
tries poked to 1: tries 0, SwitchEpisode(player, 1) at 2861, level 1
loaded at 2923; no 0x80076d6c.

What hurts: the Shadow monsters' bites (20). Nothing else in level 12
code.

## How the level ends

- Won: cine 67 done, Spike used, cine 68, 0x80076d6c(12, 1), exit code 1.
- Lost: the third cry, SwitchEpisode(1). There is no "lost" cine.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT32 | 703 | after cine 66 (arg 6) | Tommy needs to get the light out of the refrigerator. Help him find the kitchen safely. Press @ to shine his flashlight on the Shadow monsters. Collect batteries to recharge the flashlight. |
| HINT33 | 704 | never (no call of 0x80044434 passes 704; see level 3) | Press @ to try to open the refrigerator |
| HINT34 | 705 | after cine 67 (arg 3) | Search the house and backyard for Spike. Press @ to get him to help. Beware of the Shadow monsters! Don't forget your flashlight! |

@ is Triangle. The port's card table (front.rs `card_after`) has 66 ->
703 and 67 -> 705.

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 66 | Let There Be Light Intro | the level start | none | hide 14, hand model 1690 | nothing |
| 67 | Let There Be Light Mid | the refrigerator's use | drop the flashlight | default | nothing (CineFinish: card 705, 0x8007bb84) |
| 68 | Let There Be Light End | Spike's use (0x8007bc24) | drop the flashlight | default | won |

Cine table flags (0x800b8bd0 +4): 66 and 67 0x200, 68 0x10000200 (guess:
bit 0x10000000 marks a level's end). Seen lengths (oracle VSyncs): 66
455, 67 705, 68 710. 0x8007737c(n, 1) also runs about 2 VSyncs after each
PostCine (seen 2461, 3558), with no level 12 effect.

## Per-frame ambience (0x800812c8)

0x800b9618[12]. The countdown 0x800c4cc0 minus dt; below 0: while id
602 still has a use (before cine 67) speech 0x800b94c8[rand & 1] (256,
258); after it 0x800b94cc[(rand & 255) mod 5] (188, 234, 235, 236, 259);
then countdown (rand & 255) x 32. Speech only.

Seen: 258 at 2455; 236, 234, 234, 236 at 4768, 4954, 5076, 5742.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, 16-bit pokes, hooks with
arguments, object and room dumps, screenshots): presses start 300, cross
500, 650, 800, 1650, 1800; main's level argument patched (0x80018824 =
`li a0, 12`), 0x800c5678 = 12 at the level runner, cards skipped
(0x8004472c = 0x1000004a), 0x80018598 made to return 1 (Tommy). The start
cine was not skipped.

Moving Tommy between floors needs more than his position: poking only
+0x18..+0x20 under the upstairs floor left him pushed up to y about
-4000. Poking the room byte (+0x3b) and the floor heights +0x13c and
+0x148 to the new y as well put him on the kitchen floor.

- Start: cine 66 1819-2274; card 703 (arg 6); EpisodeStart; flashlight
  in hand, gauge 120; monsters in mode 11; Spike hidden; tries 3; no
  respawn.
- The light: charge 1 a game frame, sound 161 on the press; hits on
  3335 and 3339 (0x800511a8 from 0x80091504), sound 110, mode 10, speech
  143 / 21 and the return at a group 10 spot 36 VSyncs later.
- Bites: sound 111, Damage 20 each; two monsters at once.
- Batteries: taken within 1000 by 0x8004cf58, sound 109, 90 -> 120. One
  taken with no flashlight in hand (byte 0x6d written).
- Refrigerator: Triangle use of door 601, cine 67, the drop, card 705,
  Spike placed at a group 0 spot in mode 8.
- Pick up again: gauge max 100, charge 118 kept.
- Put down with Circle: gauge freed.
- Win: Spike used, cine 68, 0x80076d6c(12, 1), the puzzle screen.
- Cry: no respawn, Tommy stays; the last try: SwitchEpisode(player, 1).
- Ambience lines before and after cine 67. Rooms and objects read from
  RAM; screenshots of the bedroom and the kitchen.
- Not checked: walking the house (all moves were teleports); Circle on
  the refrigerator with empty hands; the light's exact reach and aim;
  the climb refusal in room 4; a battery after a put-down (stale gauge
  pointer); the health pickups and bars; Spike at the yard spot; cines
  played naturally from where a player would stand.

## For the port

The port has a Script per level, the playable character by level (Tommy
for 12 already), climbing, hiding objects, cinematic-object use, doors,
room flags, the follow camera, NPC modes 0, 1, 3, 4, 6, 7, 9, 11 and
riders, carrying, things kept in hand (class 13) with `give`, hazards,
healing collectibles, collect events, spawn groups for collectibles and
NPCs, ForceCry with tries, gauges, cards from level code and the result
screens; level 5's remote, zap, beam and charge live in `level5.rs`.
Level 12 needs none of level 11's swaps, meters or jumps. It needs, in
order:

1. A `level12` script: EpisodeStart 0x8007b7c8 (the flashlight given with
   120 charge and the gauge, models 360 / 361, max 120; doors 601 and
   602 use = cine 67; ids 3333-3340 guard the player; room songs 10 -> 2
   and 17 -> 1). Cine 66's end: hide 14, hand model 1690. Cines 67 and
   68 start: drop the flashlight where it is (draw on, gauge off, hand
   model 1310). Cine 67's finish (0x8007bb84): doors' use off; Spike at a
   random group 0 spot, idle (mode 8), use = win. Spike's use: follow
   (mode 4) and cine 68 (won, already in `ENDINGS`). Cards are in
   `card_after`.
2. The flashlight's light: while Triangle is held and charge is above 0,
   each game frame one ray from hand bone 15 + (0, 0, 350) along the
   facing, about 4096 long (guess), drawn for that frame; charge - 1 a
   frame; sound 161 on the press. It calls the first hit object's touch.
   Level 5's zap is a press, 3 charge and a travelling beam; share the
   charge and gauge code, not the beam.
3. Shadow monsters (char 43, template 934): guard (mode 11, range 10000,
   reach 1024, bite 20, speed 8), hovering about 1500 above the floor.
   The light: sound 110, a death (mode 10: shrink over about 18 frames),
   speech 143 or 21, then back in guard at a random group 10 spot with a
   new home (spots never marked used). The port needs NPC mode 10 and
   NPC placement into group 10 at runtime.
4. The flashlight as a pickable class 13 thing: Circle puts it down (gauge
   off, hand 1310) and picks it up (gauge max 100, hand 1690, its own
   draw off); a cry never drops it. `give` covers the start; the port
   needs the pick-up and put-down of a kept thing and the hand model
   swap.
5. Batteries in level 12: taken within 1000 (trigger pass), not by touch;
   sound 109; + 30 clamped to the gauge's max. With nothing in hand the
   battery is used up and nothing else happens (the original writes to
   address 0x6d).
6. `Script::cry` for level 12: the cry, health 255, a try; no respawn
   (stay in place); at 0 tries the hub.
7. Bodies: in level 12 the player uses hierarchy 44 and Spike hierarchy
   45 (0x800596e8, 0x8004822c).
8. Small things: the ambience lines (0x800812c8, two lists split by cine
   67); climbing refused in room 4 (also level 1); the object init tweaks
   (template 1070's box, template 1271's four copies, template 960 not
   climbable with box min y -3000); speech 143 / 21 on a monster's
   return.

## Unknown

- What hierarchies 44 and 45 look like.
- What templates 1070, 1271, 954, 678 and 960 are (the stairs? the
  doghouse?) and what template +0x24 means.
- What rooms 32 and 44 are; why 510 and 511 copy the refrigerator doors.
- Whether the battery's write to address 0x6d matters on hardware.
- Card args 6 and 3.
- What 0x800e2b48 +0x1f8 (the grip) is for the flashlight given at the
  start (no grip computed).
- Where cines 67 and 68 really leave Tommy, Spike and the flashlight from
  a natural approach (the oracle's are from teleports).
- How long the cines really are (the oracle: 455, 705 and 710 VSyncs).
