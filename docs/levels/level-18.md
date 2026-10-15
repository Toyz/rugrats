---
title: Level 18, Egg Hunt
status: partial
discs: US
covers: US SLUS_006.50:0x80079670, US SLUS_006.50:0x800797ec, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x80049af4, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x80076bf0, US SLUS_006.50:0x80076b94, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x80074958, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80078dd8, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x80094af0, US SLUS_006.50:0x800818e0, US SLUS_006.50:0x80081474, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80018598, US /DATA/DB18/DB18LP.BIN, US /DATA/DB18/LEVEL18.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/DB01/DB01LP.BIN, US /DATA/HINTS
worklog:
---

# Level 18, Egg Hunt

Everything level 18 needs to be played from its start to its end: Tommy
picks up 35 coloured eggs around the downstairs of the Pickles house
against a 170-second clock, and a gold egg behind the living-room couch
is worth a fourth Reptar bar. It is a bonus level reached by a hub door,
and an Activity. It is level 17's first round with eggs instead of coins,
one try and no second player. Read from the pseudo-code and the
disassembly; the start, all 35 eggs taken by walking, the gold egg, the
win with 3 and 4 bars, the way back to the hub, the clock running out
(poked and natural), a forced cry, the "already earned" note and activity
play were run in the oracle (see Checked in the oracle). Builds on
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md),
[screens](../engine/screens.md), [level 17](level-17.md) (the coin round,
spawn groups, the fourth bar, activity play) and [level 13](level-13.md)
(the hub door, the "already earned" note).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default case of 0x80018598; LP 67, id
1492). Cine 25 "Bonus 6 Intro", then card HINT26 (file 697): Angelica
wants all the coloured eggs; gather as many as you can by running through
them before time runs out; look out for a gold bonus egg. The loader
places 35 eggs (template 496, kind 1) at 35 of the 66 spots of spawn
group 0, and the gold egg (template 1219, kind 17, id 1001) at the one
spot of group 1, behind the couch in the living room. A 170 s clock runs.
Eggs are taken by walking into them; the count left shows top left.
Taking the 35th sets the clock to one tick, so the clock's end checks the
count: all taken plays cine 24, 3 Reptar bars, 4 when the gold egg was
taken, none when the bars were won before, and the hub. The clock running
out with eggs left is a ForceCry with no cry, which uses the one try:
cine 23 plays and the hub follows, with only +0x34 recorded.

## How level 18 is reached

- The hub door: DB01 LP 83, template 497 (an egg basket) at (525, 659,
  -2713), LP +0x42 = 18, no cine (+0x44 0). Its use is 0x8004f78c; the
  +0x30 bit 16 gate only covers levels 3, 4 and 6, so the door always
  works ([gameplay](../engine/gameplay.md#doors-0x8004f78c)). Seen in the
  hub: a sparkling basket by the TV with the label "Easy" / "Egg Hunt".
  0x80076d6c never switches to 18.
- The Activities menu: ACTIVITY_LEVELS 13, 17, -2, 18 (0x80013030),
  +0x21 = 1. See Activity play below.
- The loading screen: "Egg Hunt" (114) / "Loading" in the main game,
  "Activity" / "Egg Hunt" / "Loading" in activity play
  ([screens](../engine/screens.md)).
- Back to the hub: 0x80078e7c puts Tommy at 0x800b8718[18] = (2121, 779,
  -2837), room 26, by the basket (seen).

## State

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 at the start (0x80078d60); nothing in level 18 reads it |
| 0x800c4b64 | s32 | spawn slots: 2 |
| 0x800e2710, 0x800e2714 | ptr | slot 0 (the gold egg), slot 1 (the eggs, 232 bytes apart) |
| 0x800e2740, 0x800e2744 | s32 | copies: 1, 35; 0x800e2744 is the total the counter and the clock end use |
| 0x800e26e0, 0x800e26e4 | s32 | the slots' spawn groups: 1, 0 |
| 0x800c4b94 | u8 | copies of the last collectible record loaded (0x80049af4): 35; 0x80079670 makes the counter only when it is not 0 |
| player +0x1a9 | u8 | eggs taken (kind 1) |
| player +0x220 | ptr | the egg counter HUD item |
| 0x800c4658 | ptr | the gold egg icon, 0 when none |
| 0x800c4430 | s32 | the level timer (dt); 51000 at the start |
| 0x800e5830, 0x800e583c | | respawn, set at the start; level 18's ForceCry case does not use it |
| progress +0x23 (0x800c569b) | u8 | tries: 1 (0x800b2920[18]) |
| progress +0x30 (0x800c56a8) | u32 | \|= 256 by the EpisodeStart post (0x8007871c); \|= 8 by 0x80079670 (camera switches, below) |
| progress +0x38 bit 18 | | won before: no bars, and the "already earned" note |
| object 1001 +0xab | u8 | bit 2: the gold egg was taken (0x80076b94) |

Level 18 has a TimerExpired callback (0x8007f5a8 table 0x800158cc index
16: 0x800797ec), no room change case (0x8007f498 covers 3-15), no room
cine, no Gameplay script, no CheckPoint room, no GoalPost case and no
per-frame ambience in the main game (0x800b9618[18] = 0). 0x800c4018 is
the default 2600 (0x8007871c).

## Data in DB18

### The house

DB18 is the Pickles house again (the hub's rooms, as in DB13), dressed
for Easter. 45 rooms (LEVEL18.BIN; boxes read from RAM, room
+0x10..+0x28: x0, y0, z0, -, x1, y1, z1). Flags at +0xc; +0x5c the camera
post, +0x5d the camera kind, +0x64 the song. No room has flag 2 or a room
cine (+0x5b is 0 everywhere).

| rooms | where (x / y / z) | flags | +0x5d | notes |
|---|---|---|---|---|
| 0-7, 10-14, 43 | upstairs, y -10240..-5120 | 0 | 0 | |
| 8, 9 | upstairs, x 8192..25600, z 0..9216 | 0x4 | 1 | 9: +0x5c 2 |
| 15 | 31744..41984 / -4096..1024 / -10240..-2048 | 0x4 | 9 | the living room: the gold egg |
| 16 | 13312..28672 / -4096..1024 / -10240..-1024 | 0x4 | 7 | |
| 19, 20 | x 13312..32768, z 6144..22528 | 0x14 | 6 | 20 is level 13's kitchen |
| 21, 44 | x 32768..39936, z -1024..13312 | 0x4 | 8 | |
| 22, 30 | x 33792..39936 | 0x4 | 1 | |
| 23 | 0..12288 / -4096..1024 / 13312..22528 | 0x14 | 8 | |
| 25 | 0..12288 / -4096..1024 / -2048..13312 | 0x94 | 8 | the start, song (+0x64 1) |
| 26 | 0..12288 / -4096..1024 / -10240..-2048 | 0x94 | 8 | song (+0x64 1); the hub spot |
| 24, 37-41 | x -6144..54272, y -11264..1536, z 23552..46080 | 0x14 | 8 | the long rooms (level 13's 2, 5, 6, 12, 14, 24) |
| 35, 36 | z 22528..24576 | 0x14 | 1 | |
| 28, 29, 42 | | 0x10 | 0 | |
| 17, 27, 32 | | 0 | 0 | +0x5c 3, 3, 4 |
| 18, 31, 33, 34 | | 0 | 0 | |

Flag 128 with +0x64 1 starts a song on entry (0x80077a54; see
[level 13](level-13.md#the-house)). Flag 4 posts CameraSwitch(+0x5d)
(below). Flag 16 is not traced. LP kind 6 has 34 records; one LP kind 2
record (a post) and one kind 99 record are not traced.

### The characters (LP kind 1)

| LP | template | char | id | LP position | what |
|---|---|---|---|---|---|
| 67 | 11 | 1 | 1492 | (5775, 1001, 19403) | Tommy; cine 25 leaves him at (2333, 779, -926), room 25 |
| 167 | 1 | 3 | 600 | (21733, -706, -13634) | Angelica; in the cines only, hidden by cine 25's end hook |
| 150 | 797 | | 99 | (29590, -30, -6166) | a basket (cine prop), hidden by the same hook; shown again in cines 23 and 24 |
| 148 | 796 | | 98 | (29751, -1384, -7376) | not read by level 18's code |
| 57 | 794 | | 100 | (525, 659, -2713) | at the hub door's spot; not read by level 18's code |
| 72, 107, 79, 78, 158 | 9, 5, 10, 3, 2 | | 14, 13, 15, 16, 1776 | z about -14000 | outside every room box, idle; not read by level 18's code |

Ids 54 (the puzzle board, template 581), 57 and 80 (templates 392, 896)
are not read by level 18's code either.

### The eggs (template 496)

LP 151 at (29589, -30, -6189): class 12, kind (+0x26) 1, 35 copies (LP
+0x46), group 0 (+0x48), +0x4a 1 (placed at load), shape 818, draw byte
3. A kind 1 collectible: taken by walking into it, sound 30. Seen:
striped eggs drawn as sprites; the counter icon is one.

### The gold egg (template 1219, id 1001)

LP 76 at (29942, 0, -5181): class 12, kind 17, 1 copy, group 1, +0x4a 1,
draw byte 3. Kind 17 has a touch like kind 15 (sound 31, an icon). It is
placed at load like the eggs; 0x80076bf0 looks it up by id 1001 for the
fourth bar.

### Spawn groups (LP kind 7)

67 records: x, y, z at record +4, group at +0x10, threshold at +0x11.

| group | spots | thresholds | what |
|---|---|---|---|
| 0 | LP 180-245, 66 spots from (3794, -429, 7358) to (34306, 665, -9408) | 1, 3, 4, 6 .. 98, 100 | the eggs: 35 of the 66 |
| 1 | LP 246: (41453, 665, -3081) | 100 | the gold egg, room 15, behind the couch |

All spots are downstairs (y 340 to 665 on the floor, about -430 on
furniture). The loader (0x800454dc at 0x80045f34) gives each copy a spot
with 0x8004fe14 ([level 17](level-17.md#spawn-groups-lp-kind-7)): r =
(rand & 255) mod 100, the first free spot whose threshold is above r.
With 35 copies and 66 spots, which spots get an egg depends on rand.
Slots come in LP order, so the gold egg (LP 76) is slot 0 and the eggs
(LP 151) slot 1.

There are no Reptar bars (template 835) in DB18, so the bottom-left bar
counter of levels 17-18 ([objectives](../engine/objectives.md)) never
shows here.

## Start (EpisodeStart, 0x80079670)

The level start, in order: player init on LP 67; cine 25 (0x80074958
case 18); its start hook does nothing (0x80072ef0 case 25); its end hook
(0x800732d4 case 25, 0x8007349c) hides 600 and 99, then the default
0x80074ec0(0, player id); CineFinish (0x800783d8 case 24, 0x80078650):
card 697 (HINT26, arg 6) only outside activity play; then the runner's
"already earned" note (below); EpisodeStart 0x80079670 (installed by
0x8007871c case 18); 0x8007737c case 24 does nothing.

0x80079670(player):

1. The gold icon (0x800c4658) freed if present.
2. If 0x800c4b94 is not 0 (35 here):
   - +0x30 |= 8;
   - the counter: text "%d" of 0x800e2744 (35) at (-208, ((0x800c4a06 x
     171) >> 11) - 110) = (-208, -90), white, flags (1, 1); icon model
     143 at scale (53, 64, 1024) at (-192, -80) (0x8008f8f4,
     0x80090004); kept in player +0x220;
   - timer 170 s (0x80078dd8(170): 51000 dt, the clock at the top).
3. Respawn = the player's position and angles, phase 1 (0x80078d60).
4. Song 0 (0x8006eac4(0, 0)).

No camera call, no egg placement (the loader did it), +0x1a9 is not
reset (0 from the player init).

Seen: cine 25 from 1819 to 2136 (317 VSyncs); at 2136 card 697 (arg 6),
0x80079670, 0x80078dd8(170), 0x80078d60(phase 1), CameraSwitch(8) from
RoomChange (room 25); Tommy (id 1492, char 1) at (2333, 779, -926) room
25; timer 50670 at 2200; tries 1; +0x30 0x108; the counter "35" on an
egg top left and "165" in the clock top centre (screenshot at 2400); 35
eggs in rooms, the gold egg at (41453, 665, -3081) room 15; Angelica and
id 99 out of the rooms.

## Collecting (0x800514b0)

Eggs are touched by walking or standing into them (calls from 0x8005d5cc,
0x8005e34c and 0x8005ee14). The generic part: three sparks, the egg
leaves its room, +0xab |= 2, its spot freed, +0xaa = 0. Then by kind:

- Kind 1, an egg (0x80051b50): sound 30; player +0x1a9 += 1; the counter
  text = 0x800e2744 - +0x1a9 ("%d", 0x80090214); when they are equal, the
  timer 0x800c4430 = 1. This case does not test the level.
- Kind 17, the gold egg (0x80051ab4): sound 31; if 0x800c4658 is 0, an
  icon: model 245 at scale (32, 32, 1024), at (-140, ((0x800c4a06 x 171)
  >> 11) - 80) = (-140, -60), kept in 0x800c4658. It does not count
  towards the 35.

With one tick left, the next heartbeat ends the clock (0x8007f5a8: sound
117, TimerExpired), so the win plays the clock's buzzer too.

Seen: all 35 eggs taken by putting Tommy 600 on the +x side of each and
holding Up (one every 25 VSyncs, 2213-3277); the 35th at 3277,
TimerExpired at 3281. The gold egg taken at 2207 from (41453, 565,
-3681) facing +z (it could not be reached from the +x side: a wall); the
egg icon right of the counter after it (screenshot at 2250).

## The clock runs out (0x800797ec)

Every end of the clock calls 0x800797ec(player):

1. +0x1a9 below 0x800e2744: ForceCry (0x8007fd0c), then PostCine(23).
2. Else PostCine(24).
3. The gold icon (0x800c4658) is freed.

### ForceCry, level 18 (0x8007fd0c case 17, 0x80080148)

The generic part counts the tries down (1 -> 0). Level 18's case: s1 =
0, so no cry animation (0x8007f920 not called), health 255, no callback,
no respawn. With no try left, SwitchEpisode(player, 1) (0x800802b4).

So the clock out posts SwitchEpisode(1) and then the cine. The cine is
queued (0x8007737c with a2 = 1 tests the exit code 0x800c4918, which the
SwitchEpisode event has not set yet), cine 23 "Lose Bonus 6" plays, its
end (0x8007737c case 22, 0x800774cc) calls 0x80076d6c(18, 0): +0x34 |=
bit 18 and nothing else; then the exit code takes the game to the hub.
Cine 23's hooks are the defaults; no card (0x800783d8 case 22).

Seen (timer poked to 1 at 2300): TimerExpired 2303; 0x800797ec,
0x8007fd0c, no 0x8007f920, 0x800781f8(1), PostCine(23) at 2303; tries 1
-> 0, exit 1; cine 23 2303-2617 (314 VSyncs; Angelica with the basket,
Tommy, screenshot); 0x80076d6c(18, 0) at 2617; the hub at 2679
(0x80078e7c), Tommy at (2121, 779, -2837) room 26; +0x34 = 0x00040000,
+0x38 and +0x48[18] 0. Natural (no eggs taken): TimerExpired at 12335
(10199 VSyncs after the start, 170 s), the same calls, cine 23
12335-12649, the hub at 12710.

### Other cries

Damage is posted in level 18 (0x800780ac drops it only in levels 1, 11,
17 and 23-31), so health 0 from a fall posts ForceCry: the same case,
no cry, the hub at once, with no cine and nothing recorded. Seen
(0x8007fd0c forced at 2301): no 0x8007f920, SwitchEpisode(1) at once, no
0x80076d6c, level 1 loading at 2340. What can hurt Tommy here was not
traced (guess: falls).

## Winning (cine 24)

PostCine(24) from 0x800797ec. Cine 24 "Win Bonus 6": start and end hooks
the defaults; no card (0x800783d8 case 23). 0x8007737c case 23
(0x800774b8): exit code 0x800c4918 = 1, 0x80076d6c(18, 1).

0x80076d6c(18, 1) outside activity play: +0x34 |= bit 18; bars (table
0x800152cc index 5, 0x80076df0): none when +0x38 bit 18 is set
(0x800c56b0 & 0x40000), else 3, or 4 when 0x80076bf0 is true (object
1001 found and +0xab bit 2 set; unlike 0x80076ba8 there is no class
test); 0x80094b10(18, n) sets bits 31 down in +0x48[18] (0x800c5708),
0x80094ab0(n) "Great Job! You have earned n Reptar bars!"; after Cross
0x80094bfc(18): level 18 owns no piece, only the 15-bar check
(0x80094924); +0x38 |= bit 18 the first time; not a main level, so +0x22
is not counted. The hub: Tommy at (2121, 779, -2837).

Seen (all 35 eggs): cine 24 from 3281 to 3588 (307 VSyncs; Angelica,
Tommy and two baskets, one full of eggs, screenshot), 0x80076d6c(18, 1),
0x80094b10(18, 3), 0x80094ab0(3), +0x34 0x00040000, +0x48[18] =
0xe0000000; after Cross 0x80094bfc(18) and 0x80094924 at 3758, +0x38
0x00040000. With the gold egg taken first: 0x80076bf0 true,
0x80094b10(18, 4), "4" on the bars screen (screenshot), Cross at 3758,
level 1 at 3800. The hub plays cine 7 next when +0x30 bit 4 is clear
(the oracle never saw the hub before); with it poked the hub starts at
once (the loss runs).

## The "already earned" note (0x80094af0)

The runner (0x80017c0c at 0x800180f4) shows it for level 18 with +0x38
bit 18, outside activity play, after the start cine and its card (as for
level 13). The win then gives no bars and no bars screen.

Seen (+0x38 bit 18 poked every frame): cine 25, card 697 and 0x80094af0
-> 0x80094728 at 2136, Cross, EpisodeStart at 2239; the win (count and
clock poked): 0x80076d6c(18, 1) with no 0x80094b10, 0x80094bfc at once.

## Activity play (+0x21 = 1)

- No card after cine 25 (0x800783d8 at 0x80078650).
- No "already earned" note, and nothing recorded: 0x80076d6c returns at
  once (no +0x34, +0x38, bars).
- The gold egg is not placed: the loader skips templates 835, 863, 1218
  and 1219 in activity play (0x800454dc at 0x80045fa0). It stays at its
  LP spot (29942, 0, -5181), in room 0's list (seen; guess: out of
  reach). It keeps slot 0, so the eggs are still slot 1 and 0x800e2744
  is still 35. The eggs are placed as in the main game.
- The ambience is picked by character (0x800818e0): Tommy uses
  0x800b9618[25] = 0x80081474 ([level 13](level-13.md#activity-play-0x21--1)).
- The runner forces the exit to -1: "Returning to the Activities Menu"
  and the Activities menu ([screens](../engine/screens.md)).
- A loss goes the same way: tries are 1 in activity play too
  (0x80018138), so ForceCry's SwitchEpisode ends it (from the code).

Seen (+0x21 poked at 1805 and 1815, not through the menu; a win forced
at 2300): no 0x80044434 at 2136; object 1001 at (29942, 0, -5181);
0x80081474 every other VSync, speech 1214 at 2137; cine 24 2303-2610;
0x80076d6c(18, 1) with no 0x80094b10; 0x80018264 and 0x800183ac at 2610;
the Activities menu (0x8003b7bc) at 2668.

## Camera (0x80077a54, 0x80077c88)

0x80079670 makes no camera call: the level's follow camera, mode 1.
Rooms with flag 4 post CameraSwitch(+0x5d) (0x80077a54 at 0x80077b34).
0x80079670 sets +0x30 & 8; while it is set 0x80077c88 drops every switch
except in levels 17-18, so here they go through: kind 8 rooms (15's
neighbours 21, 44, and 23-26, 37-41) height 1600, kind 6 (19, 20) height
200, kind 1 the default, kinds 7 (room 16) and 9 (room 15) not traced.
Seen: CameraSwitch(8) at 2136 in room 25 and camera +0x4a 1; the heights
were not checked.

## What does not happen in level 18

- No NPC moves: Angelica and the basket are hidden after cine 25; the
  other ids sit outside the house.
- No second round and no second player (unlike level 17): one try.
- No speech in the main game (no ambience, no NPC lines).
- No health meter unless Tommy is hurt.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT26 | 697 | after cine 25 (arg 6), not in activity play | Greedy Angelica wants all of the colored eggs for herself. Quickly search the house and gather as many eggs as you can by running through them before time runs out. Lookout for a gold bonus egg! |

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 25 | Bonus 6 Intro | the level start | none | hide 600, 99 | nothing (card 697) |
| 24 | Win Bonus 6 | 0x800797ec, all eggs | none | default | won, 0x80076d6c(18, 1) |
| 23 | Lose Bonus 6 | 0x800797ec, eggs left, after ForceCry | none | default | 0x80076d6c(18, 0); ForceCry's SwitchEpisode(1) leaves |

Cine table flags (0x800b8bd0 + 16 x cine + 4) are 0x200 for all three.
Seen lengths (oracle VSyncs): 25 317,
24 307, 23 314.

## Checked in the oracle

The level 13 probe copied to the scratch folder as l18probe, with a level
18 line (+0x1a9, 0x800e2740/44, 0x800c4b94, 0x800c4b64, +0x220,
0x800c4658, +0x30, +0x34, +0x38, +0x48[18], the exit code, ids 600, 14,
13, 15, 16, 1776, 1001, 99, and both spawn slots: count, taken, in a
room) and a `--collect` option (each step puts Tommy 600 on the +x side
of the first egg still in a room, facing it, Up held). Presses start
300, cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 18`), 0x800c5678 = 18 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1 (Tommy
anyway). The start cine was kept.

- Start: cine 25 1819-2136, card 697 (arg 6), EpisodeStart, 170 s, tries
  1, the counter, 35 eggs and the gold egg placed, 600 and 99 hidden.
- Rooms read from RAM (45).
- All 35 eggs taken by walking: the 35th at 3277, the clock's end, cine
  24, 3 bars, +0x34, +0x48[18], +0x38 after Cross, level 1 after it.
- The gold egg taken by walking: its icon; 4 bars.
- The clock out (poked, and natural at 12335): no cry, SwitchEpisode(1),
  cine 23, 0x80076d6c(18, 0), the hub at (2121, 779, -2837).
- A forced ForceCry: no cry, the hub at once, nothing recorded.
- The "already earned" note after the card, and no bars.
- Activity play (+0x21 poked): no card, no gold egg, the char 1
  ambience, no bars, the Activities menu.
- Not checked: the hub door into level 18 (from the data; its label was
  seen in the hub); the Activities menu itself; the loading screens;
  walking the house (eggs were reached by teleports); what hurts Tommy;
  the camera heights; Pause, Quit Level and Restart; ids 98 and 100.

## For the port

The port has the Script trait, Tommy as the default character, spawn
groups with the loader placing collectibles at their group's spots,
world billboard sprites, kind 1 / 17 sounds (30 / 31), collect events,
the level timer and `Script::timer_expired`, hiding by id, ForceCry with
tries and `cry_animation`, cards from level code, `start_cine` 18 -> 25,
`card_after` 25 -> 697, `ENDINGS` 24 (won), the bar award (3, or 4 by
`fourth_bar`, none with +0x38 bit 18), the "already earned the reptar
bars" note for 13 and 18, and activity play. Level 18 needs, in order:

1. A `level18` script (game.rs has no case for 18). At EpisodeStart
   (after cine 25 and its card): hide 600 and 99 (cine 25's end hook),
   the 170 s clock, the counter, respawn, song 0. No camera call and no
   re-placement: the loader's placement stands.
2. The count: a kind 1 egg adds one; the counter text is 35 (the slot's
   copies, 0x800e2744) minus the count; the 35th sets the timer to one
   tick, so the clock's end (with sound 117) decides the win next frame.
   The counter HUD: icon model 143 at scale (53, 64) at (-192, -80), the
   number at (-208, -90), top left. The gold egg (kind 17) does not
   count; it sets `g.fourth_bar` and adds icon model 245 at (-140, -60)
   once. Count the total from the goodies (template 496), not a constant
   in code.
3. `Script::timer_expired` (0x800797ec): all taken -> cine 24 (the
   existing won ending); else the loss below; free the gold icon.
4. The loss: no cry animation (`cry_animation` false), the one try gone,
   then cine 23, and after it the hub with the level recorded lost
   (+0x34 only). The port's `Game::cry` returns at once while a cine is
   set and `cried()` sets the exit before any cine could play, so the
   script must play cine 23 itself and leave after it: either `ENDINGS`
   gains (23, false) (only level 18 posts cine 23, and with tries 1 it
   always leaves), or `cine_finished(23)` sets the exit. A cry from
   Damage instead leaves at once with nothing recorded.
5. Activity play: no card 697 after cine 25 (the port's `card_after` 25
   is unconditional); the gold egg not placed at load (template 1219 is
   one of the four 0x800454dc skips; the port's loader places every
   collectible); the eggs are placed. The ambience 0x80081474 for Tommy
   (as level 13). A loss also returns to the Activities menu.
6. The "already earned" note after cine 25 and its card, not before the
   cine (main.rs puts every note before the start cinematic, as noted
   for level 13).
7. Small things: the camera switches of flag 4 rooms while +0x30 & 8 is
   set are allowed in 17-18 (kinds 7 and 9 are not traced); the hub
   return spot (2121, 779, -2837), room 26; song 0 at the start and the
   flag 128 rooms 25 and 26; Damage is on in level 18.

## Unknown

- What hurts Tommy here (the generic Damage posters; guess: falls).
- Camera kinds 7 (room 16) and 9 (room 15) and the camera posts of rooms
  9, 17, 27, 32; room flag 16; the LP kind 99 record and the one post.
- Ids 98 and 100 (templates 796 and 794) and the five ids outside the
  house (14, 13, 15, 16, 1776): not read by level 18's code; guess: props
  and actors for other uses of the house.
- Card arg 6.
- Whether the gold egg at (29942, 0, -5181) can be reached in activity
  play (guess: no).
- The cines' lengths from natural play.
