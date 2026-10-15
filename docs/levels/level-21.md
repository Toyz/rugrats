---
title: Level 21, Touchdown Tommy
status: partial
discs: US
covers: US SLUS_006.50:0x8007a444, US SLUS_006.50:0x8007a094, US SLUS_006.50:0x8007a314, US SLUS_006.50:0x8007a348, US SLUS_006.50:0x8007a388, US SLUS_006.50:0x8007a504, US SLUS_006.50:0x80057950, US SLUS_006.50:0x80057570, US SLUS_006.50:0x80057518, US SLUS_006.50:0x800574cc, US SLUS_006.50:0x80057468, US SLUS_006.50:0x80059b34, US SLUS_006.50:0x80060b44, US SLUS_006.50:0x800601e4, US SLUS_006.50:0x80054708, US SLUS_006.50:0x80046110, US SLUS_006.50:0x8006f2fc, US SLUS_006.50:0x8005f3a4, US SLUS_006.50:0x8004df94, US SLUS_006.50:0x8008046c, US SLUS_006.50:0x800494d8, US SLUS_006.50:0x800596e8, US SLUS_006.50:0x8008c7f0, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80018598, US /DATA/DB21/DB21LP.BIN, US /DATA/DB21/LEVEL21.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 21, Touchdown Tommy

Everything level 21 needs to be played from its start to its end: the
babies keep Tommy's bottle of chocolate milk away from Angelica on a
football field laid over the Pickles living room, and drink it all. It
runs on [level 20](level-20.md)'s code: the same throws to a baby with
the arrow, the same body swaps, the same Angelica. What differs is the
bottle, the milk gauge, the count (20, and throws cost milk), the swap
back to Tommy at each round, and the cines. Read from the pseudo-code and
the disassembly; the start, drinking, a throw and its swap, a throw that
empties the bottle, the floor-bottle loss, the retry after a swap, both
natural losses and the win were run in the oracle (see Checked in the
oracle). Builds on [gameplay](../engine/gameplay.md),
[objectives](../engine/objectives.md), [screens](../engine/screens.md)
and [level 20](level-20.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player starts as Tommy (char 1, the default case of 0x80018598).
Level 21 is a bonus level: 0x80076d6c starts it when the fifth main level
is completed (+0x22 = 5, printf "should trigger LVL_TOUCHDOWN_TOMMY
next", 0x80076fdc). Cine 49 "Touchdown Tommy Start", card HINT25 (file
696): Angelica wants Tommy's chocolate milk; drink it all before she gets
any; run from her, press Triangle to throw the bottle to the baby with
the arrow, press Circle to drink while standing still. The bottle
(template 259, id 700) starts in Tommy's hand with 20 units of milk,
shown by a bottle gauge bottom left. Circle (or Square) while standing
drinks one. Triangle lobs the bottle to the targeted baby and costs one
too; when the bottle comes within 1400 of that baby the player becomes
the baby. Angelica (600) runs after the bottle and lunges; touching the
holder knocks it up (one more unit gone) and it is a loss (cine 51
"Touchdown Tommy Lose"). Tries are 2: the first loss starts the round
again with 20 and the player back in Tommy's body; the second sends the
player to the hub. A drink that brings the milk to 0 (or a drink at 0)
plays cine 50 "Touchdown Tommy Win": 3 Reptar bars and the hub. Nothing
hurts; there is no health meter.

## How level 21 is reached

- Only once: 0x80076d6c (a main level of 2-6 or 8-12 completed for the
  first time) counts +0x22; at 5, SwitchEpisode(21) (0x80076fdc,
  0x80077028), +0x30 bits 1-2 cleared.
- No hub door leads to 21 and it is not an activity (as level 20). Won or
  lost, it is not played again in that game (from the code).
- The loading screen is the "Bonus Level" one, name "Touchdown Tommy"
  ([screens](../engine/screens.md)).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4bd6 | s16 | milk left; 20 at each round's start |
| 0x800c4298 | ptr | the milk gauge (0x8008c7f0, models 366 / 365); 0 after a loss |
| 0x800c42a4 | ptr | the bottle (template 259, 0x800494d8 at 0x80049538) |
| 0x800c42a8 | ptr | the baby the bottle was thrown to; 0 after a swap |
| 0x800c4420 | ptr | set to the player when the bottle reaches 0x800c42a8; the swap runs on it |
| 0x800c4648 | ptr | level 20's counter text; 0 in level 21 |
| 0x800c4410 | s32 | last camera kind; 1 written by cine 51's end |
| 0x800e5778 | object | the dummy target; position +0x18..+0x20 = (6825, 775, 10814) |
| player +0x1f0 | ptr | the bottle while held |
| player +0x1f4 | ptr | the throw target: a baby, or the dummy |
| player +0x200..+0x20c | s32[4] | the stored aim (0x80057bc4) |
| player +0x188 | u32 | bit 4 throw requested, bit 1 frozen during it |
| 0x800c56a8 (+0x30) | u32 | bit 64: Angelica knocked the bottle |
| progress +0x23 (0x800c569b) | u8 | tries: 2 (0x800b2920[21]) |
| 0x800e5830, 0x800e583c | | respawn, phase 1, set after cine 49 only (0x8007a314) |

Level 21 has no level timer case (0x8007f5a8's switch covers levels
2-18), no room change case (0x8007f498 covers 3-15), no room cine, no
Gameplay script and no CheckPoint room (no kind 6 record has those
bytes). Its per-frame ambience is 0x800b9618[21] = 0x8008046c, the same
as level 20's. Player init (0x800596e8 at 0x800599e8) clears 0x800c42a8
in levels 20-21.

## Data in DB21

### The house as a football field

49 rooms (LEVEL21.BIN; boxes read from RAM, room +0x10..+0x28), the
Pickles house. The play is in room 25 (x 0..12288, z -2048..14336), the
living room with green carpet, yard lines and goal posts (screenshots).
Room 25 has flags 0x94 (4, 16, 128), camera kind 3 on post 1 (+0x5c =
1, +0x5d = 3) and song byte +0x64 = 1. Rooms 16, 20, 21, 29 and 42 also
have kind 3 on post 1; rooms 23 and 26 beside room 25 have flags 0x90.
Post 1 (LP 114, kind 2) is at (11291, -2098, 17624), flag 8. One kind 99
record is not traced.

### The characters (LP kind 1)

| LP | template | char | id | LP position | what |
|---|---|---|---|---|---|
| 92 | 11 | 1 | 1492 | (2494, 1001, 14918) | Tommy, the first player; cine 49 leaves him at (4225, 779, 4176), room 25, heading 734 |
| 90 | 2 | 2 | 1776 | (2503, 686, 4895) | Chuckie; health 250, range 5000, speed index 3 |
| 88 | 7 | 5 | 608 | (9332, 671, 4261) | Phil; health 5, range 2000, speed index 4 |
| 89 | 6 | 4 | 609 | (10744, 674, 15628) | Lil; moved to (10030, 779, 10051) by EpisodeStart |
| 93 | 1 | 3 | 600 | (5526, 546, 7787) | Angelica; health 250, range 2000, wait 400, reach 1024, touch 9 |
| 100 | 5 | 7 | 13 | (-1337, -1271, 4975) | cine 49 actor, hidden by its end hook |
| 98 | 10 | 8 | 15 | (-1365, -1279, 2963) | cine 49 actor, hidden by its end hook |
| 99 | 3 | 13 | 16 | (-1407, -1490, 4011) | cine 49 actor, hidden by its end hook |

Values in RAM at 2000 and 3125. Cine 49 shows Grandpa Lou and Chas
Finster handing over the bottle (screenshots); which of ids 13, 15, 16 is
which is a guess (16, char 13, is Grandpa in [level 15](level-15.md)).
NPC init (0x80081bac at 0x8008204c) gives Angelica speed row 1 in levels
20-21, as in level 20.

### The bottle (template 259, id 700)

LP 91 at (3809, 731, 13903). Template 259: class 0, pickable (+0x28 =
1), shape 521, draw byte 1, byte +9 = 16. Object flags 0x00040000 (not
0x10000). Object init keeps it in 0x800c42a4 in levels 20-21 (0x800494d8
at 0x80049538: template 259 or 442).

Byte +9 picks the object's +0x50 from table 0x800b7eb0 (0x80046110,
called by 0x8004691c at 0x80046b8c, which also sets object flag
0x40000): 16 gives 0x80054708, a particle effect at the object (sprite
0x800c4484 + 0x1368; guess: a milk splash). Level 20's box (442) has
none. This +0x50 is why throws cost milk (below).

Its model +0x88 is set to 0x80044b74(1245) by the round start and by the
start hooks of cines 49-51 (0x80072ef0 at 0x8007317c).

### Template 886

Pick-up (0x800601e4 at 0x800603d8) in levels 20-21 gives template 886
special handling: the player's part (+0x154 + 40 x +0x23, +0x58) =
0x80057468(char) and the object hidden. No LP record in DB20 or DB21
uses template 886, so it never runs in the shipped levels (from the
data).

### Other objects

Ids 54 (template 581, use byte 1), 56-58 (391-393), 60 (401), 80 (896)
sit in rooms 25 and 0; templates 1192 (draw byte 0, six copies) are
invisible. No level 21 code reads them (0x8007a444 and 0x8007a094 use
only 600, 608, 609, 1776, 1492 and 0x800c42a4). Not traced further.

### Reptar bars (LP kind 7)

Two spots, both in room 25, threshold 100: (551, 615, 5335) group 0 and
(11253, 615, 3798) group 1 (record +4 x, +8 y, +0xc z), for the two bars
(template 835, LP 106 and 107, one copy each). Not taken in the oracle.

## Start (EpisodeStart and the round start)

0x8007871c case 21 (0x800788c4) installs 0x8007a444. The level start, in
order: player init on LP 92; cine 49 (0x80074958, switch on level + 1,
case 22 at 0x80074ae8); start hook 0x80072ef0 case 49: the bottle shown
(draw 0x800307dc(1), model 1245); end hook 0x800732d4 case 49
(0x80073684): hide ids 15, 16, 13, then the default 0x80074ec0(0, player
id); CineFinish (0x800783d8 case 48): card 696 (HINT25, arg 6);
EpisodeStart 0x8007a444; 0x8007737c case 48: 0x8007a314.

0x8007a444(player):

1. 0x800c4420 = 0. Camera 0x800291f8(camera, player, 3, 0, 0, 0, 0, 0).
2. Lil (609) moved to (10030, 779, 10051) (0x800825b0) and her home
   +0x648..+0x650 set there.
3. Phil (608) and Chuckie (1776) moved to their homes.

0x8007a314(player): the round start 0x8007a094, then the respawn at the
player's position and angles, phase 1 (0x80078d60).

0x8007a094(player), the round start:

1. +0x30 bit 64 cleared.
2. Tommy back: if id 1492 is not the player object (0x800e2b48),
   0x80057570(player, Tommy) swaps them (below). So every round is
   played from Tommy's body, wherever his NPC stands.
3. Milk 0x800c4bd6 = 20; 0x800c4420 = 0.
4. The bottle: model +0x88 = 0x80044b74(1245); player +0x1f0 = bottle,
   out of the room lists (0x80078b00), the grip (0x8005ff88(player, 1));
   the player's part (+0x154 + 40 x +0x23, +0x58) = 0x80044b74(1238);
   the bottle's draw +0x40 = 0. So Tommy holds the bottle as part of his
   own model and the bottle object is hidden (guess: 1238 is Tommy's
   arm with the bottle).
5. The dummy's position (0x800e5790..98) = (6825, 775, 10814).
6. Ids 1776, 608, 609: idle (0x80083918, mode 8), +0x1f0 = 0, +0x688
   |= 66, scheduled (0x80049e18).
7. Id 600, Angelica: +0x1f0 = 0, speed index +0x66e = 2, follow the
   bottle (0x80083764, mode 4), +0x688 |= 128 (may lunge), scheduled,
   +0x1f4 = the player.
8. The gauge: 0x8008c7f0(366, (44, 64, 1024), (x, 78 - (0x800c4a06 >>
   4), 0), 365, (44, 96, 1024), (x, 68 - (0x800c4a06 >> 4), 0), max 20,
   0) with x = ((0x800c4a04 x 5) >> 7) - 192; kept in 0x800c4298. Seen:
   a brown chocolate-milk bottle with a teat, bottom left; its fill
   drops as the milk goes.
9. Song 0 (0x8006eac4(0, 0)).

There is no counter text (0x800c4648 stays 0). RoomChange for room 25
(flag 4, kind 3) gives camera kind 3.

Seen: cine 49 from 1819 to 3120 (1301 VSyncs); at 3120 card 696 (arg 6),
0x8007a444, 0x8007a314, 0x8007a094, respawn (4225, 779, 4176) phase 1;
at 3125 Tommy there in room 25 with the bottle (draw 0, part 1238), milk
20, gauge 20 / 20, tries 2, +0x30 0x100, camera kind 3, Angelica mode 4
after the bottle from (7983, 561, 8239) where the cine left her, the
babies in mode 8, Lil at (10030, 636, 10051).

## Drinking (0x80057950, level 21 branch)

Every player update in levels 20-21 runs 0x80057950 (see
[level 20](level-20.md#eating-0x80057950)). With the bottle held, Circle
or Square pressed and state 0:

1. Level 21 (0x800579e0): sound 43 + (rand & 15) at the player
   (0x8006f2fc: bank 14, positional; bank 13 is level 20's), animation
   40, state 19.
2. Milk - 1. Above 0, nothing more.
3. At 0 (or below): milk 0, +0x1f4 = the dummy, throw requested (+0x188
   |= 4, animation 18), the player's part = 0x80057468(char) (the
   character's own: char 1 7, 2 22, 3 37, 4 54, 5 69), then 0x8007a504
   (0x80057ab8).

0x8007a504(player): speech 0x800b9244[(rand & 255) mod 6] (1164, 1166,
1176, 60, 594, 918); +0x1f0 = 0; PostCine(50). With +0x1f0 cleared the
requested throw never happens.

Each frame 0x80057950 also writes the "%d" text to 0x800c4648 (0, so
nothing) and calls 0x8008ca80(0x800c4298, milk): the gauge.

Seen: Circle every 10 VSyncs from 3130: drinks at 3133, 3163, 3213, 3263
(sounds 44, 47, 57, 46), state 19 about 48 VSyncs each, 20 -> 16; as Phil
after a swap: sound 46, 19 -> 18. At 20 x about 50 VSyncs a full drink
takes about 1000 VSyncs; Angelica reached the standing player in about
200.

## Throwing and the swap

The target, the arrow, the aim and the lob are level 20's
([level 20](level-20.md#throwing-to-a-baby)): candidates are templates 2,
6, 7, 11 within 512 of the heading (0x80059b34), the aim 0x80057bc4,
Triangle only with a target, speech 1159 or 1167 from standing, the
player frozen during the throw. Level 21 adds, at the release
(0x80060b44):

1. Level 21 only (0x80060ce0): the bottle shown (+0x40 =
   0x800307dc(1)) and the player's part back to 0x80057468(char).
2. After the launch, the bottle's +0x50 (0x80054708, the effect) runs,
   and because 0x800c4298 is set, milk - 1 (not below 0) and the gauge
   updated (0x80060e3c..0x80060e70). Level 20's box has no +0x50, so
   this never runs there.

So every throw, Angelica's knock included, costs one unit. A throw at 1
leaves 0 without a win; the next drink wins.

The catch and the swap are level 20's (0x80057518 within 1400 in x and z
while the bottle flies; 0x8008046c calls 0x80057570). In level 21 the
swap skips animation 20 (0x800578c4): the new player gets animation 0,
then 36. The bottle in the new holder's hand is the visible object
(0x80057570 sets its draw to 0x800307dc(1)); only the round start uses
part 1238.

Seen: Triangle at 3135 (target Phil, 608), speech 1167 at 3137 (from
0x8005d278), release at 3157 (0x800574cc, 0x800c42a8 = Phil), milk 20 ->
19, gauge 19 / 20, the bottle visible, Tommy's part back to his own; the
bottle from (4395, 484, 3887) up to about y -986 and down; 0x80057570 at
3193 (from 0x8008048c); the player Phil (char 5, id 608) at (9332, 626,
4261), state 20 for about 30 VSyncs, health 0, the bottle in hand; Tommy
(1492) an NPC in mode 8. Screenshots: the yellow arrow over the target,
the gauge's fill a little lower at 19.

Seen (milk poked to 1 at 3130): the throw at 3157 left 0, no cine; the
swap to Phil at 3193; Circle at 3240: a drink at 3243 at 0, 0x8007a504,
PostCine(50).

## Angelica

Level 20's: mode 4 after the bottle, the lunge (state 28, sound 121)
within her range with a clear line, the knock on touching the holder
(0x8005f3a4 at 0x8005f468: +0x30 |= 64, player +0x1f4 = the dummy, state
19, throw requested, animation 8), the bottle's think 0x8004df94 posting
ForceCry. See [level 20](level-20.md#angelica). In level 21 the knock
also writes 0 to 0x800c4648 +0x78 with 0x800c4648 = 0 (address 0x78;
harmless, guess).

The floor-bottle loss (0x80057950 at 0x80057afc) is the same: no bottle
held, the bottle not flying, Angelica in state 28: ForceCry
(0x800782d4).

Seen: from the start, Angelica in state 28 at about 3320, the knock
throw released at 3347 (milk 20 -> 19), 0x8004df94 at 3347 and 3349,
ForceCry at 3351. Seen (0x800c42a8 poked 0 after a release, so no
catch): the bottle on the floor at (9325, 448, 4112), Angelica walked to
it, lunged, 0x800782d4 from 0x80057b70 at 3317, ForceCry at 3319, cine
51.

## The loss (ForceCry, cine 51)

ForceCry (0x8007fd0c), generic part: a held object is put down only when
its draw +0x40 is not 0, so Tommy's hidden bottle stays; tries counted
down (0x800c569b - 1 when not 0). Level 21 is case 20 (0x8008000c): no
cry animation (s1 = 0), health 255, with the respawn set (always) the
respawn pointers and the callback 0x8007a388:

0x8007a388(player): +0x1f0 = 0; speech 0x800b922c[(rand & 255) mod 12]
(1152, 1161, 1157, 1156, 1162, 1169, 1171, 535, 536, 578, 579, 580);
PostCine(51); the gauge freed (0x8008c9f4), 0x800c4298 = 0; camera
0x800291f8(camera, player, 3, 0, 0, 0, 0, 0).

Tries left: back to the respawn (0x8007fbf8). None: nothing (level 21 is
skipped by the SwitchEpisode at 0x800802b4).

Cine 51: start hook (0x80072ef0 case 51): the bottle shown, model 1245.
End hook (0x800732d4 case 51, 0x800737cc): with 0 tries
SwitchEpisode(player, 1), then the default. CineFinish (0x800783d8 case
50): card 709 (HINT38 "Now try again.", arg 1) only with tries left.
0x8007737c case 50 (0x80077624): 0x800c4410 = 1, the bottle's draw = 0,
0x80076d6c(21, 0) (only +0x34 bit 21 and music off), then 0x8007a348:
with tries left the round start 0x8007a094 and CameraSwitch(player, 3)
(0x80078264); with none nothing.

So a retry always gives Tommy the full bottle: a player in another
baby's body is swapped back into Tommy, who stands where his NPC was.
The respawn is not set again.

Seen (no input): first knock, ForceCry at 3351, speech 580, the respawn,
cine 51 from 3379 to 3734 (355 VSyncs), card 709 (arg 1),
0x80076d6c(21, 0), 0x8007a348, 0x8007a094, CameraSwitch at 3734; tries 2
-> 1; Tommy at (4225, 779, 4176), milk 20, gauge 20, the bottle hidden
again (part 1238); Angelica starting from (5323, 948, 10983). Second
knock: release 4072, ForceCry 4076, speech 536, tries 1 -> 0, no
0x8007fbf8, cine 51 4104-4459, no card, SwitchEpisode(player, 1) from
0x800737f0 at 4459, 0x80076d6c(21, 0), 0x8007a348 doing nothing, the hub
loaded (level 1 at 4521).

Seen (as Phil): ForceCry at 3317 (speech 578), the player Phil sent to
the respawn (4225, 626, 4176); at the end of cine 51 (3700) 0x8007a094
called 0x80057570 (from 0x8007a0ec): the player Tommy at (2576, 809,
14047), where his NPC had walked (towards his LP spot), state 20, health
0, with the bottle; Phil an NPC again at the respawn spot, walking home.

## Winning (cine 50)

The drink that empties the bottle posts cine 50 (above). Start hook
(0x80072ef0 case 50): the bottle shown, model 1245. End hook: the
default. No card (0x800783d8 case 49). 0x8007737c case 49 (0x800775f0):
exit code 0x800c4918 = 1, the bottle's draw = 0, 0x80076d6c(21, 1).

0x80076d6c(21, 1): +0x34 |= bit 21; bars 3 (table 0x800152cc index 8,
0x80076e60): 0x80094b10(21, 3) and 0x80094ab0 "Great Job! You have
earned 3 Reptar bars!"; after Cross 0x80094bfc(21): no piece, only the
15-bar check (0x80094924); +0x38 |= bit 21 the first time (not a main
level, +0x22 not counted); the hub.

Back in the hub, 0x80078e7c puts the player at 0x800b8718[21] = (5775,
779, 19403), the same spot as level 20.

0x8007a504 does not free the gauge, so 0x800c4298 still points at it in
the hub (seen 0x801b6f8c at 4970). What that stale pointer does there is
not traced (it is read by 0x80060b44's counted pickup and 0x80057950).

Seen (milk poked to 1 at 3140, Circle at 3150): the drink at 3153 (sound
47), 0x8007a504 (speech 1176), PostCine(50), cine 50 from 3183 to 3834
(651 VSyncs), 0x80076d6c(21, 1), 0x80094b10(21, 3), 0x80094ab0(3), the
bars screen until Cross at 4000, 0x80094bfc(21) and 0x80094924 at 4058,
level 1 at 4121, 0x80078e7c; Tommy at (5775, 779, 19403), room 23; +0x34
and +0x38 0x00200000. (+0x30 bit 4 was poked at 3900 so the probe's
fresh game skipped cine 7.)

## Camera (0x80077a54, 0x80077c88)

Level 20's rules ([level 20](level-20.md#camera-0x80077a54-0x80077c88)):
RoomChange in levels 20-21 takes a room's kind only when it is 3
(0x80077a54 at 0x80077b34); CameraSwitch in levels 20-21 calls
0x800291f8(camera, 0, mode, 1500, 3000, 0, bottle, 0) (kind 3 gives mode
3). EpisodeStart and the loss callback call 0x800291f8(camera, player,
3, 0, 0, 0, 0, 0) instead.

Seen: camera +0x4a 3 throughout; the view changed after the swap to Phil
(screenshots at 3175 and 3225).

## Per-frame ambience (0x8008046c)

Shared with level 20 (0x800b9618[21]): the pending catch runs the swap;
a countdown 0x800c4cc0 plays speech from 0x800b92f8 (31 lines) every
(rand & 255) x 8 dt. Seen: speech 548 at 3122, 323 at 3321, 569 at 3738.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT25 | 696 | after cine 49 (arg 6) | Angelica wants to take Tommy's chocolate milk. Drink all the milk before Angelica can get any for herself. Run away from her and press @ to throw the bottle to the baby with the arrow above his or her head. Press # to drink it while standing still. |
| HINT38 | 709 | after cine 51 with tries left (arg 1) | Now try again. |

@ is Triangle, # Circle (Square drinks too). The port's `card_after` has
49 -> 696 and 51 -> 709 (the latter without the tries test).

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 49 | Touchdown Tommy Start | the level start | bottle shown, model 1245 | hide 15, 16, 13 | 0x8007a314 (round start, respawn) |
| 50 | Touchdown Tommy Win | the last drink (0x8007a504) | bottle shown, model 1245 | default | won, bottle hidden, 0x80076d6c(21, 1) |
| 51 | Touchdown Tommy Lose | ForceCry's callback (0x8007a388) | bottle shown, model 1245 | SwitchEpisode(1) when tries are 0 | 0x800c4410 = 1, bottle hidden, 0x80076d6c(21, 0), 0x8007a348 |

Cine table flags (0x800b8bd0 + 16 x cine + 4) are 0x200 for all three.
Seen lengths (oracle VSyncs): 49 1301, 50 651, 51 355.

## Checked in the oracle

The level 20 probe copied to the scratch folder as l21probe, with a level
21 line (the bottle and its +0x40, +0x48, +0x50, +0x88; the player's
part; the gauge's values; ids 600, 608, 609, 1776, 1492). Presses start
300, cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 21`), 0x800c5678 = 21 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1 (its own
answer for 21). The start cine was kept.

- Start: cine 49 1819-3120; card 696 (arg 6); 0x8007a444, 0x8007a314,
  0x8007a094; camera kind 3; bottle in hand and hidden, milk 20, gauge,
  tries 2, respawn phase 1; Angelica following the bottle; babies idle;
  Lil moved; the cine actors hidden.
- Drinking: Circle presses, animation 40, state 19 (about 48 VSyncs),
  random bank 14 sounds, 20 -> 16; as Phil too.
- Throw: target 608, speech 1167, the lob, milk 20 -> 19 at the release,
  the bottle shown, the catch, 0x80057570 from 0x8008046c, the player
  Phil.
- A throw at 1 leaves 0 with no win; the next drink (at 0) wins.
- Angelica: the lunge, the knock (milk - 1), ForceCry from the bottle's
  think; the floor-bottle loss after a miss.
- Loss: cine 51, card 709, the round start again, tries 2 -> 1 -> 0, the
  second loss to the hub through cine 51's end hook; a loss as Phil
  swapped back into Tommy at the retry.
- Win: cine 50, 0x80076d6c(21, 1), 3 bars, no piece, the hub spot
  (5775, 779, 19403), +0x34 / +0x38 bit 21.
- Rooms and objects read from RAM; screenshots of cine 49, the field,
  the gauge, the arrow, the swap.
- Not checked: a natural full drink of 20 (Angelica arrives first
  without throws); a natural miss (the miss was forced by a poke);
  picking the bottle up from the floor; Chuckie or Lil as a target or as
  the player; leaving room 25; the bars; Pause and Quit Level; the
  camera's use of the bottle; what 0x80054708 draws.

## For the port

The port has the Script trait, Tommy as the level's character, carrying
and aimed throws, things kept in hand, NPC follow and idle, ForceCry with
tries, cards from level code, the kind 3 fixed camera and the 20-21 room
camera rule (game.rs `flags & 4`), GRAB never putting down in 20-21,
`start_cine` 21 -> 49, `card_after` 49 -> 696 and 51 -> 709, `ENDINGS`
50 (won) and the 3-bar award. Level 20's doc lists what is being added
for level 20 (the baby targets and arrow, the lob, runtime body swaps,
NPCs following a thing, the counted pickup HUD); level 21 uses all of
it. Beyond that, level 21 needs, in order:

1. A `level21` script, best as level 20's script with a table of
   per-level values (guess: one "keep-away" script): bottle template 259
   (id 700) for box 442, 20 for 15, cines 49 / 50 / 51 for 63 / 64 / 65,
   the speech lists, animation 40 for 42. EpisodeStart 0x8007a444: Lil
   to (10030, 779, 10051) and her home set there, Phil and Chuckie home,
   camera kind 3. The respawn only after cine 49 (0x8007a314).
2. The swap back at each round start (0x8007a094): if the player is not
   in Tommy's body (id 1492), swap with Tommy's NPC first, so the round
   starts where that NPC stands. The port's level 20 round start keeps
   whoever is the player.
3. The milk rules: a drink (Circle or Square, standing, holding) costs
   one, with animation 40, state 19 for about 48 VSyncs and sound 43 +
   (rand & 15) from bank 14 at the player; every throw costs one at the
   release (Angelica's knock throw too), not below 0; the win only on a
   drink that leaves 0 or less (a throw to 0 does not win).
4. The gauge instead of level 20's counter: models 366 and 365, max 20,
   bottom left (0x8008c7f0 arguments above), updated each frame, freed
   by a loss and rebuilt by the round start; no counter text.
5. The bottle's look: at the round start Tommy holds it as part 1238
   with the object hidden; at the release, after a swap and at the
   starts of cines 49-51 the object is drawn with model 1245; Tommy's
   part goes back to his own at the release and at the winning drink.
6. The swap without animation 20 (animation 0 then 36).
7. `Script::cry` for 21 (case 20, 0x8007a388): no cry animation, health
   255, back to the respawn while tries remain, empty hands, speech from
   the 12-line list, cine 51, gauge freed. Cine 51's finish: with tries
   left card 709, the round start and camera kind 3; with none the hub
   from cine 51's end, no card; 0x80076d6c(21, 0) each time. Cine 50:
   speech from the 6-line list before it, won after it.
8. Small things: hide ids 13, 15, 16 at the end of cine 49; the two bars
   at (551, 615, 5335) and (11253, 615, 3798); Angelica's lunge, the
   floor-bottle loss, the ambience and the camera as in level 20; the
   level is played once per game; the stale gauge pointer and the write
   to address 0x78 need nothing.

## Unknown

- What 0x80054708 draws (a milk splash, guess) and its sprite.
- What models 1238 and 1245 are exactly (Tommy's arm with the bottle and
  the bottle, guess), and which of 366 / 365 is the gauge's frame and
  which its fill.
- How the kind 3 camera uses the bottle as its subject; why the view
  changed after a swap.
- Whether 20 drinks can be done in practice; how many throws a natural
  win needs.
- What the stale 0x800c4298 does in the hub after a win.
- Ids 54, 56-58, 60, 80 and the invisible template 1192 copies; the
  kind 99 record; room 25's flag 16.
- Which grown-up each of ids 13, 15, 16 is.
- Card args 6 and 1.
- Where cines 49, 50 and 51 leave each baby from natural play (the
  oracle's are from standing play and pokes).
