---
title: Level 16, Training Level
status: partial
discs: US
covers: US SLUS_006.50:0x80079090, US SLUS_006.50:0x800790f8, US SLUS_006.50:0x8007e680, US SLUS_006.50:0x8007e904, US SLUS_006.50:0x8007ea10, US SLUS_006.50:0x8007ea60, US SLUS_006.50:0x8007eafc, US SLUS_006.50:0x80097788, US SLUS_006.50:0x800974bc, US SLUS_006.50:0x80097584, US SLUS_006.50:0x80097664, US SLUS_006.50:0x80097344, US SLUS_006.50:0x80097564, US SLUS_006.50:0x80097314, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80077ee4, US SLUS_006.50:0x800789e4, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80077864, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x80018c38, US SLUS_006.50:0x800185f8, US SLUS_006.50:0x800398bc, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x8004d49c, US SLUS_006.50:0x80049eec, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x80058da4, US SLUS_006.50:0x80056e44, US SLUS_006.50:0x800590d4, US SLUS_006.50:0x8005dedc, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x80061330, US SLUS_006.50:0x800601e4, US SLUS_006.50:0x80060910, US SLUS_006.50:0x80060b44, US SLUS_006.50:0x80061c98, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x80084084, US SLUS_006.50:0x8004f868, US SLUS_006.50:0x8004e458, US SLUS_006.50:0x8004f4a0, US SLUS_006.50:0x8004eb10, US SLUS_006.50:0x80081230, US SLUS_006.50:0x80080308, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x8008a668, US SLUS_006.50:0x8008a7ec, US /DATA/DB16/DB16LP.BIN, US /DATA/DB16/LEVEL16.BIN, US /DATA/DB16/LEVEL16.COL, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 16, Training Level

Everything level 16 needs to be played from its start to its end: Tommy
walks a small house of seven rooms, and the game stops him with short
training messages the first time he runs, climbs, picks something up,
meets a lobster and so on. A pie booth in room 4 runs Angelica's pie
toss on a 30 s clock. Nothing can be lost: at 0 health the health just
fills again. The level ends at the door in room 6 (to the hub, as the
start of a main game) or by Pause, Quit Level (back to the title). It is
reached from the title's Start Game, Training. Read from the pseudo-code
and the disassembly; the menu path, the start card, 16 of the 17 shown
messages, Cross closing a message, the timeout, the pie toss won and lost,
the lobster, the slippery floor, a forced health 0, the door to the hub
and Quit Level were run in the oracle (see Checked in the oracle). Builds
on [gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md),
[screens](../engine/screens.md) and [level 3](level-03.md) (the same pie
toss code).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
A VSync is 1/60 s; the oracle's frame counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default of 0x80018598; LP 24). There is
no start cine (0x80074958 has no case 16). Card HINT40 (file 711) shows at
once: "Make your way through the training level to learn the helpful
skills that are needed to complete the Search for Reptar." There is no
goal object, counter or clock outside the pie toss. The house has six
Reptar bars and five cookies on fixed spots, two climbable boxes, soda
cans, a lobster walking a ring in room 0, a slippery corner, doors, two
sparkling hub-door objects, and Angelica in a carnival booth (room 4).
Walking into room 4 starts the pie toss (0x8007e680): five pie hits on
Angelica (250 health, 50 a pie) before the clock ends; nothing is
awarded either way and the toss starts again on every entry. Training
messages (0x80097788) are the level's teaching: each of 17 messages
shows once a level start, freezes the player and the world, and goes
after 5 s or on Cross. The door in room 6 (LP 55, template 121, LP +0x42
= 1) posts SwitchEpisode(1): the hub, with the progress kept.

## How level 16 is reached

- The title: Start Game (0x8003cf80), then the third item, Training,
  returns 16 with +0x21 = 0 ([gameplay](../engine/gameplay.md#from-the-front-end-to-a-level)).
  Main (0x800185f8) resets the progress only for result 1 (0x80018754),
  so training starts with whatever progress there is (zero after boot).
- The loading page: "Training Level" (string 0x800b7d60[16] = 166) /
  "Loading" ([screens](../engine/screens.md#the-loading-screen-0x8003b168-0x8003eefc));
  the front end draws it (0x8003b168 from 0x8003df20).
- No hub door leads here, and no ending cine, SwitchEpisode or menu other
  than Start Game names 16. Select's "Restart Level?" (0x80043d58) can
  restart it (from the code, not run).

Seen (menu path, no pokes): presses start 300, cross 500, 650, 800, down
900, 960, cross 1050; the Start Game menu (0x8003cf80) at 844, the front
end's loading lines at 1114, the runner 0x80017c0c(16) at 1174, card 711
at 1176.

## State

| address | type | meaning |
|---|---|---|
| 0x800c5680 | s32 | character: 1, Tommy |
| progress +0x23 (0x800c569b) | u8 | tries: 0 (0x800b2920[16] = 0, stored by 0x80018138) |
| progress +0x30 (0x800c56a8) | u32 | \|= 256 by the EpisodeStart post; \|= 8 during the pie toss (0x8007e680, cleared by 0x8007e904); seen 0x100 and 0x108 |
| progress +0x48[16] (0x800c5700) | u32 | bars found in level 16, a bit each (0x80094a58), like any level |
| 0x800c473c | s32 | the training message showing (0 none) |
| 0x800c4738 | s16 | its time left (dt), 1500 at the start |
| 0x800e7f08 | u8[36] | shows left per message, from 0x800bb4c4 (all 1, message 0 is 0) |
| 0x800e7e90 | u16[36] | per-message cooldown (dt), 1500 after a show |
| 0x800e7e60, 0x800e7ed8 | 44-byte text items | the message and its shadow ([text](../engine/text.md#a-text-item-44-bytes)) |
| 0x800c3eb9 | u8 | frozen: the result of 0x80097584 each frame |
| 0x800c4430 | s32 | the level timer (dt): 9000 during the pie toss |
| 0x800e2770 | ptr[6] | the six pies (class 9, template 564) |
| 0x800c4cd4 | ptr | pie 0, set by 0x8007e680 |
| 0x800c4134 | u16 | the pie in hand, 0..5 (0x8004eb10) |
| 0x800c4cc0 | s32 | ambience countdown: 900 at the setup (0x80080308) |

Level 16 has a TimerExpired callback (0x8007f5a8 table index 14:
0x8007ea10), a Gameplay script (3, 0x8007e680, from 0x80077ee4), a
Damage hook (0x80077864 calls 0x8007eafc in level 16 with +0x30 bit 8), a
ForceCry case (0x8007fd0c table index 15), an end-of-level hook
(0x800790f8 from the runner at 0x80018228) and the health meter (the
runner's switch 0x800128a0 index 14: 0x8008c094). No room change case
(0x8007f498 covers 3-15), no CheckPoint room, no GoalPost case, no room
cine, no song table entry. Damage is posted (0x800780ac drops it only in
levels 1, 11, 17 and 23-31).

+0x30 bit 64 is not a training flag: it is set by 0x8005f3a4 (state 28,
levels 20-21), cleared by 0x80079cb0 and 0x8007a094 (levels 20-21) and
at the loop's end, and read by 0x8004df94, 0x80060b44 and 0x80057bc4.
Level 16 never sets it (seen: +0x30 0x100 / 0x108 throughout).

## Data in DB16

LP (DB16LP.BIN): 60 kind 1, 19 kind 2 (posts), 11 kind 7 (spawn spots),
one kind 6 (room 4's script), one kind 99.

### Rooms (LEVEL16.BIN)

7 rooms, boxes read from RAM (room +0x10..+0x28: x0, y0, z0, -, x1, y1,
z1; all y -7168..-1024 but room 2 to -512). Flags 0 and camera kind 0
everywhere, but room 4: flag 64, script +0x5f = 3. No room song, no
fixed cameras: the follow camera throughout.

| room | x | z | what |
|---|---|---|---|
| 0 | 10240..30720 | -37888..-24576 | south-west: the lobster's ring (posts 10-19), soda cans, the slippery corner, two doors to room 3, template 441 |
| 1 | 31744..48128 | -24576..-12288 | east: the two boxes (1016), the burst emitter (969), table tops |
| 2 | 10240..24576 | -24576..-5120 | west: the wet trench, high corner (a bar), toys |
| 3 | 30720..48128 | -37888..-24576 | south-east: doors, template 111, high blocks (bars) |
| 4 | 29696..43008 | -11264..-5120 | the pie booth: Angelica (posts 1-9), the counter with the pie; flag 64, script 3 |
| 5 | 24576..31744 | -24576..-5120 | the start |
| 6 | 17408..21504 | -5120..-1024 | the corridor to the exit door |

Rooms 4 and 5 overlap at x 29696..31744: the counter (x 30720..31743)
belongs to room 5 by lookup, so the toss starts when Tommy drops off it
into room 4 (seen).

### LEVEL16.COL

Decoded with the COL layout of [gameplay](../engine/gameplay.md#collision-levelnncol)
(min (10240, -7168, -37888), max (48128, -512, -1024)). The main floor is
at y about -1024 (Tommy's origin at -1269). Measured from the file:

- Floor flag 2 (slippery: player +0x00 |= 0x100000 at 0x8005c96c) on 30
  cells, x 10240..16383, z -37888..-32769 (room 0's south-west corner).
- Floor flag 1 on 26 cells, x 10240..23551, z -15360..-13313, y -768 and
  -512: a sunken trench across room 2 (walking sound 176 in level 16,
  0x8005e1f4).
- Raised floors: the counter (x 30720..31743, z -11264..-6144, about 450
  up), blocks in room 0 (x 19456..23551 and 25600..29695, z
  -28672..-26624) and room 3 (x 37888..39935, z -37888..-35840, about
  3000 up), room 2's north-west corner (up to about 2200 up).
- Openings: room 5 to room 2 at x 23552..24575 for z -15360..-14336 and
  -24576..-21504; rooms 2/5 to rooms 0/3 at z -25600 for x 14336..18431
  and 33792..39935; room 5 to room 4 only over the counter or along z
  -11264; room 2 to room 6 at x 17408..21503.

### Characters and objects (LP kind 1)

| template | count | class | where | what |
|---|---|---|---|---|
| 11 | 1 | 1 | LP 24 (28041, -1376, -19515), room 5 | Tommy, the player |
| 1 | 1 | 1, char 3, id 600 | LP 34 (41053, -1690, -8080), room 4 | Angelica; first post 2 (LP +0x40); health 250 (+0x66c) |
| 495 | 1 | 1 | LP 39 (21317, -1304, -33845), room 0 | the lobster; first post 16; health 50; bite 15 (+0x6d) |
| 564 | 1 | 9, id 1111 | LP 54 (31189, -1709, -8141), on the counter | the pie (six copies, [level 3](level-03.md#the-pies-class-9-0x8004997c)); +0x6d 50 |
| 835 | 1 (6 copies) | 12, kind 3 | group 1 | Reptar bars |
| 863 | 1 (5 copies) | 12, kind 4 | group 0 | cookies (+0x6d 63) |
| 1016 | 2 | 0 | (45575, -1292, -13985), (39010, -1292, -22640), room 1 | boxes: pickable, climbable (flags 0x42) |
| 777 | 4 | 0 | room 0, y -1163 | soda cans: pickable, object bit 0x800000, +0x6d 50 |
| 526 | 3 | 2 | room 0, y -1166 | spinning soda cans (think 0x8004e2ac), +0x6d 4 |
| 218 | 2 | 4 | (30115, -3079, -28681), (30250, -3079, -32764) | double door between rooms 0 and 3 (think 0x8004f868, use 0x8004f4a0) |
| 226, 227 | 1 each | 4 | room 3 | doors |
| 441 | 1 | 0, use 3 | (23285, -1222, -30670), room 0 | a hub door object (in DB01 it leads to level 3) |
| 111 | 1 | 0, use 3 | (44266, -1122, -32077), room 3 | a hub door object (in DB01 to level 2) |
| 121 | 1 | 0 | LP 55 (19273, -3092, -1243), room 6 | the exit door: LP +0x42 = 1, +0x44 = 0 |
| 969 (900 under it) | 1 | 10 | (41164, -1993, -13773), room 1 | burst emitter: Damage 18 within 3072 every 100 VSyncs ([level 6](level-06.md#timed-bursts-class-10); not seen here) |
| 611 | 1 | 10 | (12414, -2794, -26120), room 0 | burst, no damage |
| 1160-1163 | 1 each | 0 | room 1, y -1046 | template flag 8 (no collision), guess: rugs |
| 1164 | 1 | 0 | (18437, -1313, -20201), room 2 | pickable, flag 0x400000, trigger 4096 (not traced) |
| 679, 680 | 1 each | 0 | room 2 | pickable (the dog's toys in the hub) |
| 391-395 | 1 each, ids 55-59 | 0 | x 45209..46753, z -32001..-30292 | not placed: 0x800454dc skips them in level 16 (test at 0x8004560c) |
| 29, 434, 1175, 1228, 228, 381, 112, 206, 132, 315, 318 | | | | scenery; 315 and 318 have touch 2 and 3 (0x800504a4, 0x800504c4; not traced) |

Templates 111 and 441 in level 16 get trigger radius 2500, enter
0x80049e18, leave 0x80049e5c and think 0x80049eec (0x8004691c at
0x800469a4); the template's own radius (+0x34, 15000) is then put over
the 2500 later in the set-up. In the player's room the think only
sparkles (0x80050fd0 with 0x800c416c cleared, so no sound), in a sphere
about the object's matrix (+0x04); the hub's labels are drawn only in
level 1. The spinners 526 run hit callback 1 (0x80053604) each think
while drawn: model 932 thrown along their turning angle, and sound 178
off the shared count. Character 23 sounds 51 at its animation event
0x8000000 unless biting (0x80084084).

### Spawn spots (LP kind 7)

Group 0, five spots (LP 62-66, thresholds 20, 40, 60, 80, 100): the five
cookies. Group 1, six spots (LP 67-72, thresholds 16, 33, 50, 66, 83,
100): the six bars. Every spot gets a copy, so the placement does not
depend on rand. Bars (seen in RAM): (11433, -3371, -6122) and (23712,
-4190, -23968) room 2; (28159, -3809, -27710) room 0; (38918, -4719,
-37527), (43012, -3204, -36810) room 3; (47542, -3795, -17998) room 1.
Cookies: (11773, -1830, -36946) room 0; (32601, -3552, -23778), (32619,
-1830, -14584) room 1; (20489, -1830, -14296) room 2; (44720, -1830,
-26616) room 3.

### Posts (LP kind 2)

Posts 1-9 (LP 73-81): a ring in room 4, x 36612..41988, z -10454..-5911,
y -2038 (Angelica). Posts 10-19 (LP 82-91): a ring in room 0, x
18876..28642, z -36847..-32681 (the lobster).

## Start

The level start, in order: player init on LP 24 (0x800596e8, which also
resets the training messages, 0x80097664 at 0x800599ec); the health
meter; no cine; EpisodeStart 0x80079090 (installed by 0x8007871c at
0x80078904); the setup's 0x800b9598[16] = 0x80080308 (ambience countdown
900).

0x80079090(player):

1. Card 711 (HINT40), 0x80044434(711, 4).
2. Angelica (600): idle (0x80083918), +0x688 |= 64, scheduled (0x80049e18).
3. Song 0 (0x8006eac4(0, 0)).

Seen: the runner at 1818 (level patched) or 1174 (menu), 0x80074958(16)
without a cine, 0x8007871c, 0x80079090 and card 711 (arg 4) in the same
frame. Tommy at (28041, -1269, -19515), room 5, health 255, tries 0,
+0x30 0x100; the bottle (health meter) bottom right (screenshot).

## Training messages (0x80097788)

### The calls

0x80097788(n) shows message n only in level 16, or in level 1 for n = 33
or 35, and only when none is showing (0x800c473c = 0). Then 0x800974bc(n):
nothing if its shows-left byte (0x800e7f08[n]) is 0 or its cooldown
(0x800e7e90[n]) is not 0; else time = 1500 (0x800c4738), shows-left -= 1,
cooldown = 1500, and 0x80097344(string 0x800bb89c[n]) (n of 36 or more
uses entry 0); 0x800c473c = n.

A call while another message shows is dropped: the shows-left byte is
not touched, so the next trigger shows it. Seen: a cookie taken during
message 2 showed no message 7; a later cookie did. With every count 1
each message shows once a level start; the cooldown has no visible effect
(guess: meant for counts above 1).

### The text (0x80097344, 0x80097664)

Two text items: 0x800e7e60 (colour 0x800c4740 = 0x00e4b96a, R 106, G 185,
B 228, light blue) and its shadow 0x800e7ed8 (0x800c4744 = 0x010101) at
(+2, +2). Level 16: font 1 (24 high), spacing 1, at (0, 0), centred both
ways (0x8002e144 flag 3). Level 1: font 0, centred across only (flag 1),
y = -105. Seen: message 2 at (-172, -24) after centring, two lines in the
screen's middle over the frozen world, no box (screenshot).

### Each frame (0x80097584 from 0x80018d18)

0x800c3eb9 = 0x80097584(dt):

- A message showing and time above 0: Cross pressed (0x8001bda4(port 0,
  word 15, mode 0); the physical button, not the action map): sound 126,
  time = 0; else time -= dt. Returns 1.
- Showing and time 0 or less: 0x800c473c = 0, returns 0.
- None showing: every cooldown -= dt (to 0), returns 0.

While it returns 1, outside level 1, the loop skips the player update
(0x80018d70) and the world update: scheduler, flights, NPC lists, shadows,
effects (0x80018e24). The camera, drawing, events and the ambience still
run; the pause is not skipped. The level timer stopped too (seen: 8990 for
the 300 VSyncs of message 28; guess: the heartbeat's timer step is skipped
while frozen). The message is drawn after the HUD (0x80097564 ->
0x80097314: shadow, then text) only while frozen. In level 1 nothing
freezes.

Seen: message 2 from 1902 to 2204 (302 VSyncs, time 1500 down by 10 a
game frame); Cross at 1960 closed it at 1962 with sound 126 and Tommy
went on running (no jump).

### The messages (strings 238-272)

| n | string | text | shown when | site | seen |
|---|---|---|---|---|---|
| 1 | 238 | Press forward to climb certain objects. | a climb is allowed: a climb height band (0x80056e44) and the touched object climbable (template flag 64) or no object | 0x80058da4 at 0x80058fe8 | 1904 (box 1016) |
| 2 | 239 | Press and hold Square button to walk. | the player not frozen at running speed (+0x184 above factor x 3933 >> 12: 16 for Tommy) | 0x800590d4 at 0x800591a4 | 1902 |
| 3 | 240 | Reptar Bars can be collected to earn Puzzle pieces. | a bar taken (kind 3, sound 28) | 0x800514b0 at 0x800517c4 | 2906 |
| 5 | 242 | Press Circle button again to put objects down. | the grab event of a pick up (+0xe0 bit 2048) | 0x800601e4 at 0x80060228 | 1940 |
| 6 | 243 | Press Triangle button while holding an object to throw it. | carrying, no throw under way, animation done (+0xe0 bit 0x8000000); not the pie | 0x80060b44 at 0x80060ef8 | 2254, after 5 closed |
| 7 | 244 | Collecting Cookie powerups increases your health. | a cookie taken (sound 29, then speech 216) | 0x800514b0 at 0x800519c0 | 2950 |
| 8 | 245 | Press Circle button to pick up and again to put down objects to help reach high places. | template 1016 put down (the event of state 8) | 0x80060910 at 0x8006097c | 2630 |
| 13 | 250 | Lobsters can hurt you. Watch out for those pinchy things. | template 495 bites the player | 0x80084084 at 0x80084734 | 1904 |
| 16 | 253 | Some objects are too tall to climb without help. | no climb band and the step is 1025..3071 up (+0x144), the touched object climbable or none | 0x80058da4 at 0x80058fe8 | 2462 |
| 24 | 261 | Be careful to not be too close to opening doors. | a class 4 door's swing meets the player (0x8006baa8) | 0x8004f868 at 0x8004fb48; class 3 0x8004e458 at 0x8004e7d0 (none in DB16) | no |
| 27 | 264 | Press Triangle button and a direction to throw a pie. | as 6, carrying template 564 | 0x80060b44 at 0x80060ef8 | 2916 |
| 28 | 265 | Try and hit Angelica 6 times before the timer runs out. | the pie toss starts | 0x8007e680 at 0x8007e8d8 | 2600 |
| 29 | 266 | Not all objects are climbable. | a climb band but the touched object lacks template flag 64 | 0x80058da4 at 0x80058fe8 | 1934 (template 441), 1910 (a can) |
| 30 | 267 | Be careful not to run on slippery surfaces. | the slip's fall: state 17 ends, state 18, Damage 5 | 0x8005dedc at 0x8005e0b4 | 2568 |
| 31 | 268 | Falling from high places can hurt you. | landing (state 10) after 193 dt or more in the air (0x800c428c, summed in 0x80061330), Damage 5 | 0x8005b2d8 at 0x8005b6f8 | 2020 (air time poked) |
| 32 | 269 | Press and hold Square button to walk carefully over slippery surfaces. | the slip handler (states 3, 17, 18) at running speed | 0x8005dedc at 0x8005df18 | 2226 |
| 34 | 271 | Some objects will launch a new episode. | use (Triangle) on template 111 or 441 | 0x8004d49c at 0x8004d4fc | 2338 |
| 33 | 270 | You can only open these after you have completed three levels and Mirrorland. | use on template 117, 715 or 720 (none in DB16; level 1 too) | 0x8004d49c | no |

There are 15 call sites in all (jal 0x80097788). No code passes 4, 9-12,
14, 15, 17-23, 25, 26 or 35, so those strings are never shown (35 "Need a
place to start? Head for the bathroom and find Grandpa's teeth." is
allowed in level 1 but has no caller). Message 0 is "No Hint Available."
(237).

Notes from the oracle:

- Message 1 shows on the first climb; a climb attempted while message 2
  shows gets none.
- Message 6 comes right after message 5 closes (the carry's animation
  finishes again).
- Message 2 is the first one in nearly every run: any run at full speed.
  Holding Square (WALK) avoids it.
- No natural fall in the house was long enough for message 31: a 3200
  drop from room 3's block gave about 170 dt in the air (guess: none in
  the level reaches 193 by walking off).

## Moving around (what the messages teach)

- Running and walking: the default gait (17, 8 with WALK).
- Climbing: onto the boxes (seen at (39010, -22640)) and the counter.
- Pick up and put down the boxes 1016 (Circle), throw (Triangle).
- Bars and cookies: generic collectibles; a bar is recorded under level
  16 (seen: +0x48[16] = 8 after the bar at (28159, -3809, -27710)).
- The slippery corner: running in gives state 3 (speech 212 on entry,
  0x8005b2d8 at 0x8005b3c4), then 17 and 18, Damage 5 (seen: health 255
  -> 250).
- The lobster walks posts 10-19 and bites for 15 every 36 VSyncs when it
  reaches Tommy (Damage from 0x80084724). A thrown soda can (template
  777: flight flags 0x190f0700, speed 170) kills it: Damage 50 on 50
  health, its think stopped (seen at 3512).
- Doors (class 4): Triangle opens one (0x8004f4a0, sound 170 in level
  16); each half of the double door opens on its own (seen: one half
  opened, the other still blocked).
- Templates 111 and 441 sparkle like hub doors; using one shows message
  34 and nothing else.

Seen besides: Damage 4 to Tommy from a flight (0x8008792c) at 1933, 2739
and 2758 in room 0 near template 441 and the spinning cans; its source
was not traced (guess: a spinning can, +0x6d 4).

## The pie toss (Gameplay script 3)

Entering room 4 posts Gameplay(3) (room flag 64, +0x5f = 3; 0x80077a54
at 0x80077bc0); 0x80077ee4 runs 0x8007e680 and then clears flag 64 of
room (event +0x11). 0x800789e4 never sets +0x11, so room 4 keeps its
flag: the toss starts again on every entry (seen: again at 3950 after a
win at 3350; message 28 does not repeat).

### The start (0x8007e680)

1. +0x30 |= 8.
2. Angelica (600): +0x694 = 0, +0x688 |= 0x1800, +0x695 = +0x698 (first
   post), health = max (+0x66c, 250), 0x80083818(angelica, +0x698), placed
   at home (0x800825b0, +0x648..+0x650), +0x688 bit 1 off, scheduled.
   Seen: she runs between posts (states 2 and 19), x 37400..40600, z
   -9900..-6800.
3. With pie 0 (0x800e2770): Tommy to (31189, -1750, -8141) (0x79d5,
   0xffffe033), room update, 0x800c4cd4 = pie 0, reset (0x80049464),
   +0x1f0 = pie 0 (in hand), pie 0 out of its room's list (0x8001a810).
   No pie is in a room's list: the one in
   hand is drawn with the player (0x8003205c calls +0x1f0's draw), the
   others are out of the world until each is next; a thrown one is
   listed while it flies and leaves when its flight ends (seen).
4. Heading and target heading 2048; +0x188 |= 1 (frozen: he turns but
   does not move).
5. 0x800e577c.. = the player's matrix, 0x800e5790.. = his position with
   y - 1570; the camera 0x800291f8(camera, player, 4, 1500) with stack
   (952, &eye, 0x800e5778, 512), eye = position + (-4000, -2240, 0).
6. The pie spot: (0, 0, 512) turned by his angles plus his position,
   into the six pies' home (0x80049818).
7. Message 28; the clock 30 s (0x80078dd8(30): 9000 dt, the clock at the
   top).

Seen: 2600 (walked in) and 1902 (teleported in): Tommy at (31189, -1750,
-8141), room 4, carrying 564, +0x188 0x21, +0x30 0x108, camera +0x4a 4,
timer 8990; a booth with Angelica behind it and targets, the clock top
centre (screenshot).

### Throwing

- Triangle in the stand state (0x8005d11c): carrying, throw requested
  (+0x188 |= 4, animation 18, blend 0x2000). A press while a throw is
  under way is ignored. No empty-hands case in level 16 (level 3 only).
- The release (0x80060b44) builds the flight; 0x80061c98 with level 16
  and +0x30 bit 8: flags 0x0a070400 (0x4a070400 with the pie bit),
  +0x46 = 284, +0x66 = 1, +0x68 = 900, +0x67 = 0, direction (0, 35, 1024)
  turned by his angles, speed 225 with Angelica within 13000 (2D,
  0x8001f57c) else 450. Seen: descriptor 4a070400, speed 0xe1.
- The thrown pie's enter (+0x48 = 0x8004d01c) makes its think 0x8004eb10:
  on its first run the next pie (0x800c4134 + 1 mod 6) goes to the pie
  spot and into his hand (0x800e2d38 = player +0x1f0). So one press a
  pie. When a flight ends the pie leaves its room.
- Aim: the d-pad turns him in place (camera-relative). Seen: left for 14
  VSyncs took the heading from 1536 to 913.
- The heading also moved from 2048 to 1536 with no input, between
  messages 28 and 27 (+0x2c became 1536; not traced). From 1536 the
  pies flew at (2900, 140, -2904) into the booth's side.

### A hit

The pie's flight touches Angelica (0x80086ef0): her touch (0x800511a8)
and Damage(player, Angelica, 50) (from 0x8008792c). 0x80077864 takes 50
off her health, then (level 16, +0x30 bit 8) 0x8007eafc(player, Angelica):

- health not 0: her animation slot 0 of char 3 = {658, 659, 657, 656}
  (0x8008a668(3, list, 0, 1));
- health 0: health = max, 0x8007ea60: won.

So five hits win (250 / 50), not six as the message says. Seen: hits at
2626, 2716, 2810, 3300, 3348 (heading 913, a press every 24 VSyncs);
0x8007ea60 at 3350.

### Won (0x8007ea60)

0x8007e904; Angelica +0x699 = 1, +0x69b = 8, +0xe0 bits 0x08000000 and
0x10000000 off, animation 16 (blend 2560), slot 0 = {686, 687, 685, 684}.
Nothing is recorded or awarded.

### The clock runs out (0x8007ea10)

0x8007e904; Angelica idle (0x80083918), scheduled, +0x688 |= 64. Seen:
TimerExpired at 5000 and 4302 (30 s of unfrozen time). Nothing is
recorded.

### The end (0x8007e904)

1. +0x30 &= ~8; the timer off (0x80078df8); +0x188 bit 1 off.
2. The follow camera: 0x800291f8(camera, player, 1, 1500, 3000, 0, 0,
   612).
3. Tommy to (31000, -1781, -8146) (0x7918, 0xffffe02e).
4. A pie in hand: its room from its position (else his), out of that
   room's list (0x8001a810); +0x1f0 = 0, so it is no longer drawn (it
   was drawn with the player); +0x188 bit 4 off. The other pies stay as
   they are: out of the world, or flying.
5. 0x8008e878 (guess: the splats' list, as in level 3).

Pies still flying can hit Tommy afterwards: seen Damage(player, player,
50) twice at 5000, health 255 -> 205.

## Health and the cry

Health meter on (bottle). At 0 health 0x80077864 posts ForceCry
(0x8007fd0c): level 16 is case 15 (0x80080148): no cry animation
(0x8007f920 not called), health 255, no callback. Tries are 0, and with
+0x23 = 0 the count is not decremented and s7 stays 1, so neither
SwitchEpisode nor a respawn follows (s2 = 0). A carried object is dropped
first (0x80062ac4). Training cannot be lost.

Seen (lobster bites from 1904): health down 15 at a time to 0 at 2780,
ForceCry at 2782, health 255 at once, Tommy where he was, the level going
on.

## Ambience (0x80081230)

0x800b9618[16], every frame (0x800818e0). Countdown 0x800c4cc0 - dt;
below 0: speech 0x800b94c0[(rand & 255) mod 3] = 114, 24 or 204
(0x8008da4c), then countdown = 1200 + (rand & 255) x 8. Seen: 24 at 1998
(180 VSyncs after the start), then 114 and 204 about every 300-400
VSyncs; it ran during a message.

## Level 16 sounds

Level 16's sound set is 0x800b82e8[16] = 0x800b81b0; these sites swap
in its numbers:

| site | level 16 | else |
|---|---|---|
| door opening, 0x8004f4a0 | 170 | 15 |
| sparkle, 0x80050fd0 (when 0x800c416c) | 179 | 48 |
| particles, 0x80052ed8 | 177 | 133 |
| 0x80053604 (0x8006f644) | 178 | 150 |
| walking on floor flag 1, 0x8005e1f4 | 176 | 27 (level 4: 57) |
| slip, 0x8005dedc (+0xe0 bit 256) | 169 | 56 in level 3 |
| state 3 entry, 0x8005b2d8 | speech 212 | none |

## The end

### The door to the hub

Use on template 121 (0x8004f78c, from state 12 at 0x8005f614): +0x6c = 1
is not 3-6, so no gate; 0x8006ee14; +0x6a = 0, no cine; SwitchEpisode(1).
Teardown: 0x800790f8 (0x8008a7ec(3) and 0x8008a668(3, {514, 515, 513,
512}, 0, 0): Angelica's animation slots back), +0x04 = 16, level 1, the
loading page for level 1 ("Search for Reptar / Beginning / Loading" while
+0x30 bit 4 is clear, [screens](../engine/screens.md#the-lines-0x8003b168items-shadows-dx-dy)).
The progress is not cleared (exit 1). The hub plays cine 7 when +0x30
bit 4 is clear; its EpisodeStart (0x80078e7c at 0x80078f34) does not put
Tommy at a door spot when the previous level is 16 (or -1).

So training through the door is a main game that keeps what training
found: bars found in level 16 stay in +0x48[16] (guess: they count
toward the 15 bars for a piece like any other, 0x80094a0c).

Seen: the walk from the start (1900) to the door (2547) by the route
below; Triangle at 2700; 0x8004f78c and 0x800781f8(player, 1) at 2718;
0x800790f8 and the loading page at 2720; level 1 at 2782 with cine 7;
+0x04 16. With +0x48[16] poked to 8 before: still 8 in the hub.

Route (waypoints steered with the d-pad): (26500, -22500), (22500,
-22800), (19500, -16000), (19400, -9000), (19300, -4000), (19280, -2200):
room 5 west into room 2 by the south gap, north through room 2 into room
6. About 650 VSyncs plus message 2's 300.

### Pause, Quit Level

0x800398bc at 0x80039cd8: Quit Level, Yes, in level 1 or 16 or in
activity play gives -1 (other levels 1). The pause shows "Returning to /
Main Menus / Loading"; the runner clears the progress (exit -1,
0x80018264) and main runs the front end.

Seen: Start at 2300, right, right (Quit Level), cross, left (Yes), cross:
"Quit Level?" Yes / No, the "Returning to" page, 0x800790f8 and the
chooser (0x80030158) at 2872, +0x30 0, exit -1, the title.

## What does not happen in level 16

- No start cine, no ending cine, no 0x80076d6c, no "already earned" note,
  no bar award screen, no puzzle piece of its own.
- No cry and no loss.
- No camera switches, fixed cameras or room songs.
- Not an Activity (ACTIVITY_LEVELS 13, 17, -2, 18).

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT40 | 711 | EpisodeStart (0x80079090), arg 4 | Make your way through the training level to learn the helpful skills that are needed to complete the Search for Reptar. |

## Checked in the oracle

The level 7 probe copied to the scratch folder as l16probe, with a level
16 line (the message showing, its time, the frozen byte, the text item's
string and position, the 36 shows-left bytes, the cooldowns, pie 0,
0x800c4cd4, the carried template) and room +0x5f in the room dump.
Presses start 300, cross 500, 650, 800, 1650, 1800; main's level
argument patched (0x80018824 = `li a0, 16`, restored at 2400),
0x800c5678 = 16 at the runner, cards skipped (0x8004472c = 0x1000004a).
Steering by `--path` / `--goto`, places by `--tp`.

- The menu path (Start Game, Training) without pokes: card 711 at once.
- Start: no cine, 0x80079090, Tommy's spot, tries 0, +0x30 0x100, 7
  rooms and their objects read from RAM, room 4's flag 64 and script 3.
- Messages 1, 2, 3, 5, 6, 7, 8, 13, 16, 27, 28, 29, 30, 32, 34 by play;
  31 with the air time poked. Their text, place, 302-VSync timeout, Cross
  closing (sound 126), the freeze, a dropped message.
- The pie toss: started by walking in and by teleport; the clock stopped
  during messages; five hits and the win; the clock running out; the
  restart on re-entry; pies hitting Tommy afterwards.
- The lobster's bites, the can that kills it, the slip with Damage 5, the
  health refilled at 0.
- The door to the hub, the hub's cine 7, progress kept; Quit Level to the
  title with the progress cleared.
- Not checked: message 24 (a door's swing was not made to hit Tommy);
  message 31 without a poke; Select's restart; the burst emitter 969;
  templates 315, 318, 1164; the class 2 cans' damage; the pause during a
  message; what turned Tommy to 1536 at the toss's start; whether
  training's bars buy a piece in the hub.

## For the port

The port has the Script trait, Tommy as the default character, gait,
climbing, carrying and throwing, doors, room flags (64 included, through
`Script::gameplay`), the follow camera, NPC modes and posts, hazards,
healing collectibles, collect events, spawn groups, the level timer and
`timer_expired`, NPC damage, the health meter, ForceCry with tries and
`cry_animation`, cards from level code, text items with the fonts, and
level 3's pie toss. main.rs already enters level 16 from Training without
a progress reset (as the original). Level 16 needs, in order:

1. The training message system, shared with level 1 (messages 33, 35):
   per level start, shows-left 1 for messages 1-35 and cooldowns 0
   (0x80097664); `train(n)` from each site in the table above, dropped
   when a message shows; 1500 dt or physical Cross (sound 126) to close;
   while one shows (not in level 1) no player update and no world update
   (NPCs, flights, scheduler, movers, effects), the level timer stopped,
   the camera, drawing, events and ambience going on; drawn after the HUD
   as two text items, font 1, light blue (106, 185, 228) over a (1, 1, 1)
   shadow at +2, +2, centred on the screen (level 1: font 0, centred
   across, y -105). The 17 hooks go into the player code (climb test,
   run test, grab, put down of 1016, carry animation end, slip, landing
   after 193 dt of air time), the collect code (bar, cookie), NPC bites
   by template 495, class 4 doors' swing, use of templates 111 / 441 (34)
   and 117 / 715 / 720 (33), and the pie toss start.
2. A `level16` script. Start: no start cine; card 711 from level code
   (arg 4); Angelica (600) idle; song 0; the ambience (first line after
   900 dt, then 1200 + (rand & 255) x 8; lines 114, 24, 204). The loader
   skips templates 391-395 in level 16; templates 111 and 441 get
   trigger 2500 and the sparkle think (no labels).
3. The pie toss as Gameplay script 3 (room 4), reusing level 3's: Tommy
   put on the counter at (31189, -1750, -8141), heading 2048, frozen but
   turning with the d-pad; camera mode 4 from 4000 behind and 2240 up;
   message 28; a 30 s clock; one Triangle a pie (the next pie goes into
   his hand by itself, no empty-hands press); flight (0, 35, 1024), speed
   225 within 13000 of Angelica else 450, gravity 1; Angelica walking
   posts 1-9; Damage 50 a hit through the level 16 hook: her hit
   animations, at 0 health reset and won. Won or timed out: the end
   (follow camera back, Tommy to (31000, -1781, -8146), the pie in hand
   dropped), no award; the room keeps its flag, so the toss starts again
   on the next entry (the port's room flag 64 handling may clear it).
4. The cry in level 16: `cry_animation` false and the script's `cry`
   handling it with health 255 and nothing else. The port's `cried`
   would otherwise go to the last checkpoint; tries are 0 here.
5. The exits: the door (template 121) to the hub keeping the progress,
   +0x04 = 16 so the hub does not place Tommy at a door spot, cine 7 if
   +0x30 bit 4 is clear, the "Beginning" loading page; Pause, Quit Level
   giving -1 in level 16 (progress cleared, "Returning to / Main Menus",
   the title).
6. Small things: the lobster's ring (posts 10-19, bite 15, health 50); the
   soda cans (object bit 0x800000 flight 0x190f0700); the slippery corner
   (floor flag 2) with speech 212 on the slip; floor flag 1's walking
   sound 176; the level 16 sound swaps above; the bars found recorded
   under level 16.

## Unknown

- What turned Tommy from 2048 to 1536 between messages 28 and 27 with no
  input (camera mode 4, guess).
- Whether a door's swing can hit Tommy in normal play (message 24), and
  whether any fall in the house is long enough for message 31.
- Where the flight that did Damage 4 in room 0 came from (guess: the
  spinning cans, template 526).
- Templates 315 and 318 (touch 2 and 3), 1164 (flag 0x400000), 611's
  burst, and whether 969 hurts here as in level 6.
- Why "6 times" when five hits win (guess: Angelica's health was 300 once).
- Whether bars found in training buy pieces in the main game that
  follows (from the code they count like any level's).
- Card arg 4.
