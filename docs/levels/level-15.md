---
title: Level 15, Mirrorland
status: partial
discs: US
covers: US SLUS_006.50:0x8007ceb8, US SLUS_006.50:0x8007cc00, US SLUS_006.50:0x8007cc34, US SLUS_006.50:0x8007ccb4, US SLUS_006.50:0x8007d02c, US SLUS_006.50:0x8007d0f4, US SLUS_006.50:0x8007d1bc, US SLUS_006.50:0x8007d250, US SLUS_006.50:0x8007d4e4, US SLUS_006.50:0x8007d52c, US SLUS_006.50:0x8004d434, US SLUS_006.50:0x8004cdfc, US SLUS_006.50:0x80054894, US SLUS_006.50:0x80048a44, US SLUS_006.50:0x8004d570, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8004fd20, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x80049eec, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x80078dd8, US SLUS_006.50:0x80078df8, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x800811c0, US SLUS_006.50:0x80018598, US SLUS_006.50:0x800919b4, US /DATA/DB15/DB15LP.BIN, US /DATA/DB15/LEVEL15.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/DB01/DB01LP.BIN, US /DATA/HINTS
worklog:
---

# Level 15, Mirrorland

Everything level 15 needs to be played from its start to its end: Tommy
in an upside-down copy of the Pickles house, six purple "?" boxes, six
colours of balloons against a 90-second clock. Read from the pseudo-code
and the disassembly; the start, a box opened, a full balloon run, all six
colours, the win (cine 72) and the way back to the hub, the clock running
out, the respawn and what a failure undoes were run in the oracle (see
Checked in the oracle). Builds on [gameplay](../engine/gameplay.md),
[objectives](../engine/objectives.md), [screens](../engine/screens.md) and
[level 12](level-12.md) (the same house, spawn groups).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"slot" is an index into the loader's spawn lists (0x800e2710 objects,
0x800e26e0 group, 0x800e2740 copies). A VSync is 1/60 s; the oracle's
frame counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default case of 0x80018598). Mirrorland
is a bonus level: 0x80076d6c starts it after the third main level is
completed (state +0x22 = 3). Cine 73 "MirrorLand Intro", then card HINT13
(file 684): find the six purple ? boxes and press Triangle to release the
balloons; collect all of them before the timer runs out to earn a Reptar
balloon of that colour; earn all six. Each box (ids 207-212) sits in its
own room. Using one hides it and scatters 25 or 30 balloons of one colour
over that floor of the house, starts a 90 s clock and a counter. Taking
the last balloon stops the clock and lights that colour's icon. While a
colour runs, the other boxes cannot be used. The clock at 0 is a cry:
Tommy goes back to the last checkpoint and the colours not yet secured
by a checkpoint are undone. Tries are 0, which means unlimited: the level
cannot be lost. The sixth colour plays cine 72 "MirrorLand Win" and
0x80076d6c(15, 1): the level's puzzle piece, +0x30 bit 16, and the hub.

## How level 15 is reached

- First time: 0x80076d6c (a main level of 2-6 or 8-12 completed for the
  first time) counts +0x22; at 3 it prints "should trigger
  LVL_MIRROR_LAND next" and calls SwitchEpisode(15) (0x80076fc8,
  0x80077028), +0x30 bits 1-2 cleared.
- Again, from the hub: with +0x30 bit 16 set, the hub's mirror in
  Tommy's room becomes a door to level 15 (below). It plays no cine.
- Not an activity (0x80013030: 13, 17, -2, 18).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cb0 | u8[6] | state by slot: 0 not started, 1 running, 2 done (0x8007cc34, 0x8007ccb4, 0x8007d250) |
| 0x800c441c | s8 | the running slot, -1 none |
| 0x800c4cb8 | s8[6] | rooms whose box was used since the last checkpoint |
| 0x800c441d | s8 | count of 0x800c4cb8; 0 means "set a checkpoint at the next box" |
| 0x800c4cd0 | u8 | phase: 1 at the start, 2 after one colour, 3 after three (0x80078d60) |
| 0x800e5830, 0x800e583c | | respawn (0x80078d60); set by EpisodeStart, so always set |
| 0x800c4430 | s32 | the level timer (dt); 27000 at a box |
| 0x800c4b94 | u8 | balloons of the running colour (25 or 30) |
| player +0x1a9 | u8 | balloons taken of the running colour |
| player +0x220 | ptr | the counter's HUD item, 0 when none |
| 0x800e6ed8 | ptr[6] | the six balloon icons by slot |
| progress +0x23 (0x800c569b) | u8 | tries: 0 (0x800b2920[15] = 0) |
| progress +0x30 (0x800c56a8) | u32 | bit 16 set at cine 73's start and at the win |

Level 15 has a level timer callback (0x8007f5a8 table 0x800158cc index
13 = 0x8007d4e4), no room change case (0x8007f498 table 0x80015894 index
12 is the default 0x8007f598), a per-frame ambience (0x800b9618[15] =
0x800811c0), no CheckPoint room, no room cine and no Gameplay script (no
kind 6 record has those bytes).

## Data in DB15

### The house, upside down

47 rooms (LEVEL15.BIN; boxes read from RAM, room +0x10..+0x28). It is
the Pickles house turned over: the bedroom floor is at y about 8587
(rooms 0-14, y 3712..8832), the ground floor at about 2420 (rooms 15-31,
33-36, 42, 44, y -2432..2688), the yard rooms 24 and 37-41, and rooms 32
and 45 at y -9600..-4480 (the box 212 floor at about -4745; guess: the
attic or the basement turned over). Screenshots: Tommy walking on
ceilings, the furniture hanging above him. Kind 6 records set camera kinds
1, 6 and 0, flag 16 on many rooms, bytes +0x5c 1-4 on rooms 9, 17, 21, 27
and 32 (not used; no kind 3 room) and song 1 in room 5. Posts (LP kind
2): 1-4 and 19 camera posts (flag 8) that no room uses; 5-18 a ring at y
-9134 (flags 0; no level code uses it, guess: a cine path).

### The characters (LP kind 1)

| LP | template | char | id | position | what |
|---|---|---|---|---|---|
| 208 | 11 | 1 | 1492 | (44881, 8586, 12510) | Tommy; play starts at (44863, 8587, 12281), room 5, beside the mirror |
| 293 | 9 | 6 | 14 | (1, -688, -187) | Spike; seen at (43032, 4378, 10956) room 5 after cine 73, +0x688 0x2000; no level code moves him |
| 223 | 3 | 13 | 16 | (3668, 7643, 987) | cine actor, hidden by the end hooks |
| 219 | 1145 | 46 | 136 | (3643, 0, 1886) | cine actor, hidden by the end hooks |
| 220-222 | 1146-1148 | | 137-139 | (45536..46366, 0, 14496..17455) | cine props, hidden by the end hooks |

The mirror: LP 295 (template 836, id 602, speech line 266 at LP +0x4c)
and LP 296 (template 881, id 135) at (45713, 5369, 12603), in room 5 above
Tommy. In level 15 they have no use (the 836 / 881 case of 0x8004691c is
level 1's).

### The ? boxes (template 1102)

Template 1102: class 0, flags 0x442 (climbable), use byte 2, +0x32 = 10,
trigger 5000, shape 1512. Use byte 2 is table 0x800b7e90[2] = 0x8004d434.

| LP | id | position | room | group | slot | colour |
|---|---|---|---|---|---|---|
| 284 | 207 | (38234, 2136, 5634) | 25 | 0 | 3 | green |
| 285 | 208 | (21856, 2136, 13314) | 19 | 1 | 1 | red |
| 286 | 209 | (7520, 2136, 9738) | 21 | 2 | 2 | yellow |
| 287 | 210 | (40314, 8315, 12328) | 5 | 3 | 0 | blue |
| 288 | 211 | (8508, 8315, -4128) | 2 | 4 | 4 | orange |
| 289 | 212 | (19298, -5021, -5644) | 45 | 5 | 5 | purple |

Rooms and slots seen in RAM; colours from the HUD in screenshots.

### Room to box (0x8007d02c)

0x8007d02c(room) returns (group, slot, id) by table 0x800157e4 (room -
2, rooms 2..45): room 25 -> group 0, id 207; rooms 19 and 20 -> group 1,
id 208; room 21 -> group 2, id 209; room 5 -> group 3, id 210; room 2 ->
group 4, id 211; room 45 -> group 5, id 212; any other room -> group -1,
id 0. The slot is the index k with 0x800e26e0[k] == group, -1 if none.

### The balloons (templates 845-850)

Class 12 collectibles (0x80048a44 kinds 6-11). LP +0x46 copies, LP +0x48
group, LP +0x4a copies placed at load (0 for all six, so none is placed
at load; 0x800454dc at 0x80045f34).

| LP | template | kind (+0xa9) | group | copies | slot | colour |
|---|---|---|---|---|---|---|
| 215 | 847 | 8 | 0 | 30 | 3 | green |
| 213 | 850 | 11 | 1 | 30 | 1 | red |
| 214 | 845 | 6 | 2 | 30 | 2 | yellow |
| 212 | 846 | 7 | 3 | 25 | 0 | blue |
| 216 | 849 | 10 | 4 | 30 | 4 | orange |
| 217 | 848 | 9 | 5 | 25 | 5 | purple |

Seen at load: 0x800e26e0 = (3, 1, 2, 0, 4, 5), 0x800e2740 = (25, 30, 30,
30, 30, 25); slots 6-10 are the bars. Init (0x80048a44 at 0x80048df8):
box -375..375, +0xab |= 1, +0xb0 = 0xc0800000, +0xac = a sprite by group
(0x800c4484 + 3612, 3648, 3588, 3600, 3636, 3624), +0xb4 240, +0xb6 300,
grey 128, and per axis a random speed (+0xd0..+0xd4), a random phase
(+0xd6..+0xda) and amplitude 128 (+0xdc..+0xe0). Think 0x8004d570 kinds
6-11: each axis phase += speed x dt >> 6, offset = 128 x sin(phase); the
position is the spot (+0xc4..+0xcc, kept on the first think) plus the
offsets. So each balloon bobs up to 128 on every axis around its spot.

### Spawn groups (LP kind 7)

175 records. Groups 0-5 hold exactly one spot per balloon (30, 30, 30,
25, 30, 25), so every spot gets one. Groups 6-10 hold one spot each for
the five Reptar bars (template 835, LP 227-231, bar numbers 6-10), placed
at load (seen: 0x8004fe14(.., 6..10, 1) at 1820).

| group | spot y | where |
|---|---|---|
| 0 | 1961 (all 30) | ground floor; seen in rooms 23, 25, 26 |
| 1 | 1985 (29), 1050 (1) | ground floor |
| 2 | 1981 (28), 1203, 1194 | ground floor; seen in rooms 15, 16, 28 |
| 3 | 8144 (23), 7209 (2) | bedroom floor, rooms 0, 5, 6, 9, 14, 43 |
| 4 | 8157 (all 30) | bedroom floor; seen in rooms 1, 2, 7, 12 |
| 5 | -5184 (all 25) | room 45 |

The bar spots: groups 6 and 10 at y 8144, 7 and 9 at 2020, 8 at -5192.

A colour's spots spread over its whole floor, not only the box's room.

## Level 15 in shared code

- Camera: EpisodeStart calls 0x800291f8(camera, player, 1, 1500, 3000,
  0, 0, 612); with 1500 in levels 4 and 15 the follow height 0x800c3f5c
  is 512 (0x8002926c). Seen camera +0x4a 1.
- Generic object init (0x8004691c, table 0x8001369c index 14 =
  0x80047a54), each copy 168 bytes with its draw +0x40 = 0 (invisible,
  guess: collision only):
  1. Template 223 (LP 93): six copies at x 19100, 20600, 22100, 23600,
     25100, 26100, y 2443, z 6800, room from position (hint 22).
  2. Template 248 (LP 24): template flag 64 cleared (not climbable);
     copies at (its x, 2443, its z) and (6800, 2443, its z).
  3. Template 54 (LP 195): copies at (its x, its y, 3000) and (22000,
     its y, 3000).
- Collecting (0x800514b0, kinds 6-11 at 0x80051c08): the generic burst
  and removal, then sound 113, player +0x1a9 += 1, the counter text =
  0x800c4b94 - +0x1a9 ("%d", 0x800c4194, 0x80090214); when they are
  equal, 0x8007d250(player). Taken by walking into them (calls from
  0x8005d5cc and 0x8005e34c); a teleport onto one did not take it.
- The ambience (0x800811c0): countdown 0x800c4cc0 minus dt; below 0
  speech 0x800b94b8[rand & 3] = 155, 266, 267 or 25 (0x8008da4c), then
  countdown (rand & 255) x 8.

## Start (EpisodeStart, 0x8007ceb8)

Installed by 0x8007871c (case 15). The level start, in order: player
init on LP 208; cine 73 (0x80074958 case 15); its start hook (0x80072ef0
case 73, 0x800731ec): Spike +0x688 |= 0x2000, progress +0x30 |= 16; its
end hook (0x800732d4 case 73): hide 16, 136, 137, 138, 139, then the
default 0x80074ec0(0, player id); CineFinish (0x800783d8 case 72): card
684 (HINT13, arg 6), no callback; EpisodeStart; 0x8007737c case 72 does
nothing.

0x8007ceb8(player):

1. 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 612) (above).
2. 0x8007cc34(k) for slots 0-5: the slot's copies taken out of the world
   (0x8004fd20), state 0.
3. 0x800c441d = 0. Six icons: model 311 (empty balloon) at scale (36,
   48, 1024), x 50 + 24k, y 88 - 15 (k & 1) - (0x800c4a06 >> 4)
   (0x8008f8f4, 0x80090004), kept in 0x800e6ed8[k].
4. Player +0x220 = 0, 0x800c441c = -1.
5. Respawn = the player's position and angles, phase 1 (0x80078d60).

Seen: cine 73 from 1820 to 3075 (1255 VSyncs); at 3075 card 684 (arg 6),
EpisodeStart, respawn (44863, 8587, 12281) phase 1, room 5; +0x30 0x110
(16 from the cine, 256 from the EpisodeStart post 0x8007871c); tries 0;
six grey balloon icons bottom right.

## Opening a box (0x8004d434)

ACTION (Triangle) with a box touched (+0xf4) and Tommy standing beside
it: state 12, and at the animation event (0x8005f584 -> 0x8005f614) the
box's +0x54 = 0x8004d434(box, player):

1. 0x8004cdfc(box, box position, 100) (an effect) and sound 114 at the
   box (0x8006f644).
2. 0x80054894(box): a particle burst coloured by the box's room (table
   0x80013964; guess: the balloon colour).
3. Hide the box (0x80078bc4(id)).
4. 0x8007d0f4(player).

0x8007d0f4(player): (group, slot, id) = 0x8007d02c(player room +0x3b);
0x8007ccb4(player, group, slot); every box 207-212 but this room's gets
use +0x54 = 0; if 0x800c441d is 0, respawn = the player's position and
angles at the current phase; 0x800c4cb8[441d] = the room, 441d += 1.

0x8007ccb4(player, group, slot):

1. 0x800c441c = slot. A counter still shown (+0x220) is freed.
2. The slot's copies (0x800e2710[slot], 232 bytes apart, 0x800e2740[slot]
   of them) are taken out (0x8004fd20) and placed at the group's spots
   (0x8004fe14(list, group, count)).
3. 0x800c4b94 = count. The counter: icon model by group (0 301, 1 304, 2
   299, 3 300, 4 303, 5 302) at scale (48, 64, 1024), x -192, y 88 -
   (0x800c4a06 >> 4), with the number "%d" (count) beside it; kept in
   player +0x220. Player +0x1a9 = 0.
4. State[slot] = 1; timer 90 s (0x80078dd8(90): 27000 dt, the clock
   model 375 at the top).

The box is climbable: walking into it climbs onto it (state 6), and
Triangle on top of it does nothing (the stood-on object is skipped).
Seen: Tommy put 586 to the box's +x side was pushed to touching it
(41076); Triangle at 3170, state 12, 0x8004d434 at 3188, 0x8007ccb4(3,
0), timer 26990, respawn (41076, 8342, 12328) phase 1, the counter "25"
with a blue balloon bottom left.

Note: 0x8007d0f4 uses the player's room, not the box's. Used from a room
the table does not map, it would start slot -1 (reads before
0x800e2740). Every box sits inside its mapped room, so this does not
happen in play (guess).

## A colour completed (0x8007d250)

Called from 0x800514b0 when +0x1a9 reaches 0x800c4b94. Only with a slot
running and its state 1:

1. A running timer stops (0x80078df8: 0x800c4430 = 0, the clock item
   freed); 0x800c4430 = 0.
2. State[slot] = 2. The slot's icon is freed and replaced by the full
   balloon of the group (0 306, 1 309, 2 310, 3 305, 4 307, 5 308) at
   the same place. The counter (+0x220) is freed.
3. 0x800c441c = -1; boxes 207-212 all get use 0x8004d434 again (the
   used ones stay hidden).
4. Count the slots in state 2: 1 -> respawn = the player's position,
   phase 2, 0x800c441d = 0; 3 -> the same with phase 3; 6 -> PostCine(72)
   (0x80078350(player, 72, 0)); other counts nothing.

So checkpoints come only after the first and the third colour, where the
last balloon was taken.

Seen: blue: 25 balloons taken 3302..4310 (sound 113 each), 0x8007d250 at
4310, 0x80078df8, the icon and counter freed, respawn (34499, 8587, 7834)
phase 2; the first icon blue. In the full run: blue 5184 (phase 2), green
7888 (no checkpoint), red 10548 (phase 3), yellow 13026, orange 16134,
purple 18692 and PostCine(72).

## The clock runs out (0x8007d4e4, ForceCry)

At 0 the timer plays sound 117 and calls 0x8007d4e4(player): the counter
(+0x220) is freed, then ForceCry (0x8007fd0c). Level 15 is case 14
(0x800800a8): the cry (0x8007f920), health 255; with the respawn set
(always) the respawn pointers and the callback 0x8007d52c. Tries: the
byte is 0, so it is not counted down and the try test (s7 = 1) always
passes: Tommy always returns to the respawn (0x8007fbf8). Level 15 can
never send him to the hub by crying.

0x8007d52c(player), run after the cry:

1. For each room in 0x800c4cb8[0 .. 441d): (group, slot, id) =
   0x8007d02c(room); 0x8007cc34(slot) (copies out, state 0); its icon
   freed and an empty one (model 311) made in its place; the box shown
   again (0x80078c68(id)).
2. 0x800c441d = 1, 0x800c441c = -1; all six boxes get use 0x8004d434.
3. The box of 0x800c4cb8[0] shown again (it is kept as the list's first
   entry).

So a cry undoes every colour started since the last checkpoint, done or
not: done colours after the first and third are lost again until the
next checkpoint. The clock is not stopped by 0x8007d52c; a cry while it
runs from another cause leaves it running (from the code; not seen).

Seen: blue done (checkpoint), green done (no checkpoint), red started at
8048 and left: timer at 0 at 13451, 0x8007d4e4, ForceCry, the cry; at
13871 (420 VSyncs later) 0x8007d52c: 0x8007cc34(3) (green) and
0x8007cc34(1) (red), boxes 207 and 208 shown (207 twice), 0x8007fbf8;
Tommy at (38996, 2443, 5634), the respawn set when box 207 was opened;
states (2, 0, 0, 0, 0, 0), tries 0; +0x30 bit 32 during the cry. In the
full run a timed-out orange (box 211, list (21, 2)) undid yellow too
(0x8007d52c at 19150) and Tommy went back to box 209.

What hurts: nothing in level 15's code. Health fell 255 -> 225 once
during the teleports in room 45 (cause not traced; guess: a fall).

## Winning (cine 72)

Cine 72 "MirrorLand Win": start hook (0x80072ef0 case 72): hide Spike
(id 14); end hook (0x800732d4 case 72): hide 16, 136-139 and the
default; no card (0x800783d8 case 71 nothing). 0x8007737c case 71: exit
code 0x800c4918 = 1, 0x80076d6c(15, 1).

0x80076d6c(15, 1): music off (0x8006ee14); +0x34 |= bit 15; no bonus
bars (table 0x800152cc index 2 is none); with fewer than 12 pieces the
level's piece (0x80094bfc(15); level 15 owns one, 0x800919b4) and the
puzzle screen, then the all-placed check (0x80094978); all placed (and
not +0x30 bit 512): SwitchEpisode(7). Otherwise +0x38 |= bit 15 the
first time (not a main level, so +0x22 is not counted), then +0x30 |=
16 and SwitchEpisode(1) (0x80077034).

Seen: sixth colour at 18692, PostCine(72), cine 72 from 18721 to 19138
(417 VSyncs), 0x80076d6c(15, 1), 0x80094bfc(15), 0x8009480c(15, 1)
"Congratulations! You earned a puzzle piece!", the board; after Cross
and Triangle, SwitchEpisode(player, 1) at 6095 (a shortened run, below)
and level 1 loaded.

The port's `ENDINGS` already has 72, `start_cine` 73 and `card_after` 73
-> 684.

## After Mirrorland: what +0x30 bit 16 changes

Set at cine 73's start (so as soon as the level is entered, won or not)
and again by 0x80076d6c(15). Cleared only for a new game (main,
0x80018754). Read in three places, all for the hub:

1. Doors (0x8004f78c, any door use): without bit 16 a door whose level
   (+0x6c) is 3, 4 or 6 shows card 712 (HINT41, "To go on this
   adventure you'll need more puzzle pieces.") and does nothing. Level
   5 and the others are never locked. With bit 16 every door works:
   music off, PostCine(+0x6a) if any, SwitchEpisode(+0x6c).
2. The hub mirror (0x8004691c level 1 case at 0x800478bc): templates
   836 and 881 (DB01 LP 282 and LP 283 id 135, both at (12054, -6799,
   14487) in Tommy's room) get +0x6c = 15 and use 0x8004f78c with bit
   16, use 0 without it. DB01 LP 283 already has LP +0x42 = 15.
3. The door sign (think 0x80049eec, given to every hub door with +0x6c
   by 0x8004691c at 0x800481c8, trigger 2500): it draws the level name
   and "Easy", "Medium" or "Hard" (0x800bb858-0x800bb860; level 15 is
   "Easy", green); for 836 and 881 only with bit 16.

Back in the hub, EpisodeStart 0x80078e7c puts Tommy at 0x800b8718[the
previous level, +0x04]: for 15 (9458, -5365, 13673), beside the mirror.
Seen: +0x04 = 15 after the switch, Tommy at (9458, -5365, 13673) room 5,
respawn there, phase 1; the mirror (id 135) with +0x54 0x8004f78c and
+0x6c 15. (In the probe's fresh game the hub first replayed cine 7 and
its board, because +0x30 bit 4 was clear; with it poked set the hub
started at once.)

## Unused: 0x8007d1bc and cine 74

0x8007d1bc(player) has no caller and no pointer to it (rugrats-re xrefs,
a search of the executable): with nothing running and the player's room
mapped to a slot in state 0, slot 0 starts at once (0x8007d0f4), any
other slot posts cine 74 "MirrorLand Disperse" (0x80078350(player, 74,
0)). Cine 74's 0x8007737c case (73) calls 0x8007d0f4: the cine would end
by starting the room's colour. Nothing else posts 74 (no LP +0x44 of 74
in DB15). Guess: an older design where entering a room started its
balloons with a cine. The port needs neither.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT13 | 684 | after cine 73 (arg 6) | Find the six purple ? boxes scattered around the house and press @ to release the balloons. Collect all the scattered balloons before the timer runs out to earn a Reptar balloon of that color. Earn all six to get to the mirror and get back to the floor. |
| HINT41 | 712 | hub doors to 3, 4, 6 before bit 16 | To go on this adventure you'll need more puzzle pieces. |

@ is Triangle.

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 73 | MirrorLand Intro | the level start | Spike +0x688 0x2000, +0x30 bit 16 | hide 16, 136-139 | nothing (card 684) |
| 72 | MirrorLand Win | the sixth colour (0x8007d250) | hide 14 | hide 16, 136-139 | won, 0x80076d6c(15, 1) |
| 74 | MirrorLand Disperse | only 0x8007d1bc (dead) | none | hide 16, 136-139 | 0x8007d0f4 |

Cine table flags (0x800b8bd0 +4) are 0x200 for all three, so level 12's
guess that 0x10000000 marks a level's end does not hold for 72. Seen
lengths (oracle VSyncs): 73 1255, 72 417.

## Checked in the oracle

The level 14 probe copied to the scratch folder as l15probe. Presses
start 300, cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 15`), 0x800c5678 = 15 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1. The start
cine was kept. Balloons were taken by putting Tommy 450 (and 1000) from a
spot on each side, turning him toward it and holding Up; a teleport onto a
balloon does not take it.

- Start: cine 73 1820-3075; card 684 (arg 6); EpisodeStart; respawn the
  start spot, phase 1; +0x30 bit 16 set by the cine; tries 0; the six
  icons; spawn lists (3, 1, 2, 0, 4, 5) and (25, 30, 30, 30, 30, 25).
- Rooms read from RAM; box rooms 25, 19, 21, 5, 2, 45.
- A box: walked into it climbs it; Triangle on top does nothing; Triangle
  beside it: state 12, 0x8004d434, 0x8007ccb4, 90 s, respawn there.
- Each colour: all 25 or 30 balloons taken by walking (sound 113),
  0x8007d250, the clock and counter freed, the icon filled; checkpoints
  after the first (phase 2) and third (phase 3) only.
- Win: all six in one run (blue, green, red, yellow, orange, purple,
  18692), cine 72, 0x80076d6c(15, 1), the piece and the board. A second
  run with five states poked to 2 at 3100 and purple played: cine 72,
  0x80076d6c, the board, SwitchEpisode(1), the hub at (9458, -5365,
  13673), the mirror a door to 15.
- Loss path: the clock at 0, 0x8007d4e4, ForceCry, 0x8007d52c 420
  VSyncs later, back to the respawn, uncheckpointed colours undone,
  tries still 0 (twice: green and red; yellow and orange).
- Not checked: walking the house (all moves were teleports); the
  ambience lines; the bars and their spots taken; a cry with the clock
  running from another cause; Pause, Quit Level; the hub doors to 3, 4
  and 6 before and after bit 16 (from the code); the door sign; what
  the invisible copies of templates 223, 248 and 54 block; Spike's role
  (+0x688 0x2000).

## For the port

The port has the Script trait, Tommy for 15, the follow camera at 512,
spawn groups for collectibles, collect events with sound 113 for kinds
6-11, hiding by id, the level timer and TimerExpired, ForceCry with tries
and a respawn, cards from level code, `start_cine` 73, card 684,
`ENDINGS` 72 and `Progress::finish` setting +0x30 bit 16 for 15. Level 15
needs none of the NPC, carry or rider work. It needs, in order:

1. A `level15` script. EpisodeStart 0x8007ceb8: every balloon out of the
   world, states 0, six empty icons, nothing running, respawn at the
   start with phase 1. Cine 73's start sets +0x30 bit 16 (the port must
   not wait for the win). End hooks of 72-74 hide 16 and 136-139; cine
   72's start hides 14.
2. Balloons not placed at load. The port places every collectible with a
   group at load; the original places only LP +0x4a copies (0 for
   templates 845-850; the bars have 1). Balloons go out at EpisodeStart
   and in at a box: all copies of the colour at all of its group's spots.
   They bob (128 a axis, random speeds and phases, 0x8004d570) and are
   taken only by walking into them.
3. The ? boxes (ids 207-212, use byte 2 = 0x8004d434): Triangle beside a
   box (not on it): effect, sound 114, burst, hide it, then 0x8007d0f4 on
   the player's room: start that colour (copies placed, counter from the
   colour's total with icon 299-304, +0x1a9 = 0, 90 s clock), all other
   boxes unusable while it runs, and a respawn at the box when the
   rooms-since-checkpoint list is empty. The boxes are also climbable.
4. Completion (0x8007d250) on the last balloon: stop and hide the clock,
   free the counter, the slot's icon becomes the colour (305-310), all
   boxes usable again, checkpoint (phase 2 / 3, list emptied) at one and
   three colours, cine 72 at six.
5. Failure: `Script::timer_expired` frees the counter and cries;
   `Script::cry` for 15: the cry, health 255, back to the respawn, then
   (after the cry) undo every colour in the list (state 0, balloons out,
   empty icon, box shown), list = its first entry. Tries 0 must mean
   unlimited (0x8007fd0c never counts a 0 byte down); check that the
   port's ForceCry does not send a 0-try level to the hub.
6. The HUD row: six balloon icons by slot (x 50 + 24k, y alternating by
   15) and the counter bottom left; slot order (3, 1, 2, 0, 4, 5) comes
   from LP order, so the port can derive it from the placements.
7. The hub after bit 16: door use 0x8004f78c must refuse levels 3, 4 and
   6 with card 712 while bit 16 is clear (the port's door use has no such
   gate); templates 836 / 881 in the hub are no door (use 0) without bit
   16 and a door to 15 with it; the door sign only with bit 16; the hub
   return spot for 15 (9458, -5365, 13673).
8. Small things: the ambience speech 155, 266, 267, 25 every (rand & 255)
   x 8 dt; the invisible collision copies of templates 223, 248 (and its
   climb flag cleared) and 54 (0x80047a54); Spike +0x688 0x2000; the
   unused 0x8007d1bc / cine 74 need nothing.

## Unknown

- What the invisible copies of templates 223, 248 and 54 are for
  (blockers, stair steps?).
- What Spike does in the level. (His +0x688 bit 0x2000, from his
  template's flag 0x10000000 (0x80081d64), keeps him unsettled where he
  is put: 0x800452f8 skips the room and floor for it.)
- What the 255 -> 225 health loss in room 45 was (a fall after a
  teleport, guess).
- What 0x80054894's colour table by room shows (the balloon colour,
  guess) and what 0x8004cdfc draws.
- Card arg 6 for HINT13.
- What post ring 5-18 at y -9134 is for.
- Whether the mirror (ids 602 / 135 in DB15) is meant to be used at the
  end ("get to the mirror"): no level 15 code reads it; the win is
  automatic on the sixth colour.
- Whether a player can stand in a room the table does not map while the
  box's use fires (slot -1).
- How long the cines really are from natural play (the oracle: 73 1255
  and 72 417 VSyncs).
