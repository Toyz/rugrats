---
title: Level 10, Chuckie's Glasses
status: partial
discs: US
covers: US SLUS_006.50:0x8007ee54, US SLUS_006.50:0x800831a4, US SLUS_006.50:0x80083818, US SLUS_006.50:0x800838b4, US SLUS_006.50:0x8007f2e8, US SLUS_006.50:0x8007f150, US SLUS_006.50:0x8007f068, US SLUS_006.50:0x8007f004, US SLUS_006.50:0x8007f0e0, US SLUS_006.50:0x8007f0a8, US SLUS_006.50:0x8007f118, US SLUS_006.50:0x8007effc, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x80077070, US SLUS_006.50:0x80077ff0, US SLUS_006.50:0x80082c78, US SLUS_006.50:0x80082ee0, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80080fe0, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x80058da4, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8006af4c, US SLUS_006.50:0x800825fc, US SLUS_006.50:0x8004860c, US SLUS_006.50:0x8004f4a0, US SLUS_006.50:0x8004f868, US SLUS_006.50:0x80049754, US SLUS_006.50:0x8006fe04, US SLUS_006.50:0x80018598, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x800628a4, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x8005f584, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x8007fbf8, US SLUS_006.50:0x8007f920, US SLUS_006.50:0x80074958, US /DATA/DB10/DB10LP.BIN, US /DATA/DB10/LEVEL10.BIN, US /DATA/HINTS
worklog:
---

# Level 10, Chuckie's Glasses

Everything level 10 needs to be played to its end: Chuckie without his
glasses plays hide and seek in the Pickles house. Tommy, Phil and Lil
hide at random spots; Chuckie tags each one, and they race him back to
the playpen. Read from the pseudo-code and the disassembly; the start, a
tag by Triangle and by Circle, a hider's whole run home, a hider beaten
home, a hider winning the race, the re-hide and its try, the win (all
three caught, cine 78) and the loss (three races lost) were run in the
oracle (see Checked in the oracle). Builds on [level 9](level-09.md),
[level 6](level-06.md), [level 5](level-05.md), [level 4](level-04.md)
and [level 2](level-02.md) (NPC modes, posts, spawn groups, events,
ForceCry, cines, cards), [gameplay](gameplay.md) and
[objectives](objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Chuckie (char 2, 0x80018598 case 10, as level 6). Cine 44
"Chuckies Glasses Intro" takes his glasses (its start hook swaps his head
model to 1237), then card HINT06 (file 677): help Chuckie get his glasses
back from Angelica by finding the babies' hiding places; Triangle tags a
baby, then race him or her back to the playpen. EpisodeStart (0x8007ee54)
puts Phil (id 608), Lil (609) and Tommy (1492) at three random spots out
of 22. A tag (Triangle, or Circle) sends that baby running along the posts
to post 120 by the playpen. Chuckie standing in the playpen (template
1032) while the baby runs catches it: cine 75, 76 or 77 "X Loses
HideSeek", and the baby stays caught. The baby reaching post 120 first
(GoalPost) wins: cine 41, 42 or 43 "X Win HideSeek", the baby hides again
somewhere new and Chuckie loses a try. All three caught: cine 78 "Chuckie
Wins HideSeek" and the level is won. A third race lost ends the level.
Only one baby runs at a time.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 at EpisodeStart; +1 at each race a baby wins (0x8007f150); 4 = the third win |
| 0x800e5830 | s32[3] | respawn position (0x80078d60); set once, at EpisodeStart |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| progress +0x23 (0x800c569b) | u8 | tries; 3 at the start, -1 at each race lost |
| baby +0x692 | s16 | 1 once the baby is caught (0x8007f2e8); 0 again when it hides anew (0x8007f004) |
| baby +0x698 | u8 | the post its hiding spot starts from (0x8004fe14 copies kind 7 +0x0e) |
| 0x800c4ce4 | s32 | countdown (dt) to a running baby's next taunt (0x80084084 at 0x800849c4) |
| 0x800c4cc0 | s32 | countdown (dt) to the next ambience line (0x80080fe0) |

The phase counts lost races only; catches do not touch it. Both
0x8007f150 paths call 0x80078d60(0, 0, phase + 1), which keeps the
respawn and only bumps the phase. There is no level timer (0x8007f5a8
table 0x800158cc is indexed by level - 2; index 8 is the default
0x8007f6c8), no room change case (0x8007f498 table 0x80015894 index 7 is
the default 0x8007f598), no CheckPoint room, no room cine and no Gameplay
script (no kind 6 record has those bytes).

## Data in DB10

### The player and the characters (LP kind 1)

The loader (0x800454dc) puts the player on LP 165 (template 2, id 1776),
the first kind 1 record whose shape is char 2.

| LP | template | id | position | health | range | speed index | what |
|---|---|---|---|---|---|---|---|
| 165 | 2 | 1776 | (5648, 676, 19228) | 250 | 5000 | 3 | Chuckie, the player; cine 44 leaves him at (3309, 670, 16563), heading 2025 |
| 167 | 7 | 608 | (5775, 1001, 15307) | 5 | 2000 | 4 | Phil (0x8007f150: 608 -> cine 76 "Phil Loses", printf "Phil Lost") |
| 166 | 6 | 609 | (5775, 1001, 17355) | 5 | 2000 | 4 | Lil (609 -> cine 75 "Lil Loses", "Lil Lost") |
| 158 | 11 | 1492 | (19715, 1001, -12602) | 5 | 2000 | 4 | Tommy (1492 -> cine 77 "Tommy Loses", "Tommy Lost") |
| 79 | 1 | 600 | (21733, -706, -13634) | 250 | 2000 | 4 | Angelica; hidden after cine 44 |
| 125, 149, 150, 155 | 5, 10, 3, 9 | 13, 15, 16, 14 | near (20000..21800, -1500..-500, -14800..-13700) | 5 | 2000 (14: 6000) | | hidden after cine 44 (guess: cine 44 actors) |

The babies' LP positions do not matter: EpisodeStart moves them to
hiding spots. NPC init (0x80081bac at 0x80082078) sets the speed row
+0x670 = 1 in levels 9 and 10, so the speed table 0x800b9698 row 1 (0, 2,
3, 6, 11, 13) applies: index 4 is 11 (seen: +0x184 = 11 while running).
Lil ran her 104286-unit route in about 2090 VSyncs, about 50 a VSync (11
x 5 dt).

### Hiding spots (LP kind 7)

29 kind 7 records (0x800c4bac, 16 bytes each). LP record: x +0x04, y
+0x08, z +0x0c, group +0x10, threshold +0x11, post +0x12. Group 0 holds
the 22 hiding spots; thresholds rise by about 4.5, so each spot is about
equally likely. Every spot sits on the first post of its route home
(byte +0x12, copied to the baby's +0x695 and +0x698 by 0x8004fe14 for AI
objects, +0x00 bit 0x4000).

| entry | position | post | route | entry | position | post | route |
|---|---|---|---|---|---|---|---|
| 0 | (40195, -5697, 17984) | 20 | 48 posts | 11 | (50352, 476, 20061) | 183 | 27 |
| 1 | (-1374, -5697, 17388) | 74 | 40 | 12 | (50549, 476, 6725) | 177 | 25 |
| 2 | (3, -5697, 4151) | 87 | 41 | 13 | (41500, 476, 19077) | 182 | 22 |
| 3 | (-1431, -5697, -9413) | 94 | 39 | 14 | (39441, 476, 19457) | 110 | 20 |
| 4 | (15958, -5697, -9497) | 44 | 45 | 15 | (29994, 476, 20006) | 216 | 18 |
| 5 | (16000, -5697, -1624) | 58 | 47 | 16 | (574, 476, -9214) | 217 | 11 |
| 6 | (28950, -5697, -9359) | 53 | 45 | 17 | (13661, 476, -9570) | 121 | 17 |
| 7 | (22681, -5697, 10958) | 66 | 41 | 18 | (40962, 476, -9749) | 146 | 25 |
| 8 | (40453, -5697, -9670) | 4 | 48 | 19 | (41333, 476, -2827) | 170 | 28 |
| 9 | (40396, -5697, 2511) | 14 | 47 | 20 | (39481, 476, 2633) | 145 | 24 |
| 10 | (40367, -5697, 10950) | 39 | 47 | 21 | (39223, 476, 6632) | 208 | 26 |

Entries 0-10 are upstairs (y -5697), 11-21 downstairs (y 476). Entries
22-28 are groups 1-7 (one spot each, threshold 100, post byte 39 unused):
the Reptar bars (template 835, LP 171-177, LP +0x48 the group) placed by
the loader at 1817 (0x8004fe14(.., group, 1) from 0x80046008, seen):
(6643, -5533, 3143), (32304, 602, -9695), (11964, 602, 8227), (11198,
-5556, -9217), (11751, 602, -9690), (25593, -5518, 17398), (41520, 602,
6700). There are no health pickups in DB10.

### Posts (LP kind 2, 217 records)

All post flags in the LP are 0. EpisodeStart sets post 120's flags to 32
(0x800c4b9c + 0xd18 = post 120 +0x14): the GoalPost. Post 120 (5627, 476,
19847) has no neighbours, so a runner stops there (0x80082c78: think
cleared, -1) after posting GoalPost.

The next post (0x80082ee0, no flag 4 or 16 here) is the first neighbour
that is not the previous post, else the first neighbour. Followed that
way, every route ends at post 120 (route lengths above). Upstairs routes
meet at post 1 (11999, -5697, 2027), go down the stairs 2 -> 107 ->
108 -> 109 (y -5697 -> -4423 -> -2573 -> -671) and on through 212-215;
downstairs routes join at 118-120 or 167-169 -> 120. Posts 3, 106 and 126
have no neighbours and no route reaches them.

### The playpen (template 1032)

LP 199, template 1032 (shape 1442, draw 0: invisible), flags 0x0a
(template flag 8), at (5926, -1387, 21212), room 23. Seen in RAM: object
+0x00 0x2000, +0x7e 2780, +0x86 4139. Each frame 0x8006af4c sets player
+0x1e8 to the flag 8 object whose model the player's vertical segment
meets (0x8006abbc, 0x8006a714; guess at the exact test), else 0. The
player update (0x8005a068 at 0x8005aea4): level 10, the player not AI,
+0x1e8's template 1032: 0x8007f2e8(player).

Seen: Chuckie at (5926, 500, 21212) and (6512, 670, 19990) is in the
playpen (0x8007f2e8 every 2 VSyncs); at (3309, 670, 16563) and (6343,
677, 17202) he is not.

### Rooms (LEVEL10.BIN, LP kind 6)

45 rooms; boxes read from RAM (room +0x10..+0x28). Upstairs: rooms 0-14
and 43 (y -10240..-5120). Downstairs: rooms 15-23, 25-27, 29-36, 42, 44 (y
-4096..1024). Rooms 24, 28 and 37-41 span z 23552..46080 (y -11264..1536;
guess: outside, the yard). Room 23 (x 0..12288, z 14336..22528) holds the
start and the playpen.

Kind 6 records set camera bytes only (room +0x5d, flag 4): kinds 1, 6, 8
and 9 (follow 1500, 200, 1600, 2000; see
[level 5](level-05.md#camera-in-level-5)) and flag 16 on most downstairs
rooms. Rooms 23 and 25 have +0x0c 0x94 (flags 4, 16, 128). Byte +0x5c is
2 or 3 on rooms 9, 17, 27 (no kind 3 room uses it). No songs other than
rooms 23 and 25 (song 1).

### Other objects

| LP | template | class | what |
|---|---|---|---|
| 88, 89, 151, 152 | 213, 214, 226, 227 | 4 | hinged doors (0x8004860c: use 0x8004f4a0 swings open, sound 15 or 14; think 0x8004f868 closes, sound 54); +0x6d 0, so no damage |
| 26, 92, 156 / 47 / 48 / 102 / 192 | 112 / 192 / 277 / 130 / 180 | 5 | 0x80049754: generic init, then +0x00 = 0x400000 (meaning not traced); template flags 66 or 74 (climbable) |
| 99 | 94 | 2 | spins (template +0x18 = 32) |
| 93 | 117 | 0 | use 3 (0x8004d49c): training message 33 (guess: nothing shows outside level 16) |
| 198 | 1192 | 0 | draw 0, at (20967, -8223, 17418) (not traced) |
| 40 | 403 | 0 | not loaded in level 10 (0x800454dc at 0x80045634) |

Level 10's case of the generic object init (0x8004691c, table 0x8001369c
index 9 = 0x80046ef0):

1. Template 126 (LP 98): a 168-byte copy at (15385, 512, 800), draw 0,
   room from its position (0x8002b660), placed (0x8004a440).
2. Template 527 at (15184, y, -135) (LP 217): x = 16184.
3. Template 381 (LP 33): model box top +0x42 = -200, template flag 64
   cleared (no longer climbable).
4. Templates 1244 and 61: object +0x00 |= 0x2000, template flag 8.

### Sound sources

The loader fills ten kind 3 sources (LP 1-10). EpisodeStart then sets the
count 0x800c434b = 0 and adds three (0x8006fe04(n, position, 993, 2990),
level 64): sound 0 at (30714, -611, 14314) (the spot of LP 151, a hinged
door), 1 at (629, -1, -5204), 2 at (37941, 76, 19831). Seen in RAM at
0x800e3218. What they play is not known.

## Start (EpisodeStart, 0x8007ee54)

The level start, in order: player init on LP 165; cine 44 (0x80074958,
always; seen 1819 to 2198). Cine 44's start hook (0x80072ef0 case 44):
the player's hierarchy part (+0x154)->+0x24 gets model 1237 (0x80044b74)
- Chuckie's head without glasses for the rest of the level (seen in
play). Its end hook (0x800732d4 case 44): hide 14, 16, 15, 13 and 600
(0x80078bc4), then the default 0x80074ec0(0, player id). CineFinish
(0x800783d8 case 43): card 677 (HINT06, arg 5). EpisodeStart is installed
by 0x8007871c (case 10). 0x8007737c case 43 does nothing.

0x8007ee54(player):

1. For each of 608, 609, 1492: speed index +0x66e = 4; 0x800838b4(npc,
   46, 2048) (0x80082534, mode 1, back mode +0x69b = 1, think 0x80084084,
   animation 46, blend 2048: the hiding pose (guess)); use +0x54 =
   0x800831a4 (the tag); scheduled (0x80049e18).
2. 0x8004fe14(&{608, 609, 1492}, 0, 3): each to a random free group 0
   spot (below).
3. Respawn = the player's position and angles, phase 1 (0x80078d60).
4. Sound source count 0x800c434b = 0; post 120 flags = 32; three new
   sound sources (above).

No camera call: the runner's follow camera stays (seen camera +0x4a 1).

Seen: at 2198 card 677, EpisodeStart; Phil at entry 6 (28950, -5462,
-9359), Lil entry 5 (16000, -5459, -1624), Tommy entry 7 (22681, -5322,
10958), all mode 1, state 19, health 5; Chuckie (3309, 670, 16563) in
room 23, tries 3, health 255. The spots depend on rand.

### Picking a spot (0x8004fe14(list, 0, n))

With group 0 the spots are entries 0 to the first entry with threshold
100 (21). For each object: r = (rand & 255) mod 100; the first spot with
r below its threshold and its used bit (+0x0f bit 0) clear takes it:
position, used bit set, class 1 objects placed by 0x800825b0, AI objects
+0x695 = +0x698 = the spot's post. If none, up to four more tries with r
= 0 (the first free spot). See
[level 4](level-04.md#kind-7-spawn-groups-0x800c4bac-16-bytes-each).

## Tagging (0x800831a4)

Triangle with a hiding baby touched: state 12 (animation 50, 0x800628a4
for levels other than 2 and 9); its handler (0x8005f584) calls +0x54(baby,
player, 1) at the animation event (seen from 0x8005f614, 37 VSyncs after
the press). Circle (GRAB) with a baby touched calls +0x54(baby, player, 0)
at once from the stand handler (0x8005d4a8; seen). Either way,
0x800831a4(baby, user):

1. If any of 608, 609, 1492 is in mode 6 (running): return 0, nothing
   happens (the use stays).
2. 0x80083818(baby, +0x698): mode 6 and back mode 6, state 0, previous
   post 0, current post = the spot's post, target +0x678 = the player,
   speed from the table (row 1: 11), think 0x80084084.
3. The spot is free again: kind 7 entry (post +0x698's back link, post
   +0x18) +0x0f &= ~1.
4. Animation 47, blend 3076 (0x800572a8); mode 1 (it plays out, then mode
   0 -> mode 6).
5. +0x688 |= 0x1800 (0x800 no object collision; 0x1000 not traced).
6. Level 10 and the user is template 2: Chuckie says 0x800b9728[(rand &
   255) mod 3] (158, 160, 165) (seen 158).
7. Scheduled; +0x54 = 0 (no second tag).

Seen: Lil tagged at 2507 by Triangle (state 12 from 2470), 0x80083818(Lil,
58); and at 2473 by Circle (from 0x8005d4a8). At 3000 Lil ran in mode 6,
state 2, speed 11, +0x688 0x1800, posts 32 -> 33.

## The race

### Running home (mode 6, 0x80084084 case 6)

Mode 6 as in [level 2](level-02.md#modes-used-in-level-2): post to post
by 0x80082c78; post 120's flag 32 posts GoalPost (0x80077ff0: event 11,
+0x04 the player, +0x10 the baby). The obstacle steering (0x800825fc)
uses a side factor of 2 in levels 10, 12 and 13 (8 elsewhere).

Taunts, level 10 only (0x800849c4): the shared countdown 0x800c4ce4
minus dt; at 0 or below: by the runner's template, 11 (Tommy)
0x800b9704[(rand & 255) mod 6] (178, 180, 181, 166, 170, 171); 7 (Phil)
0x800b9710[mod 6] (189, 190, 192, 166, 170, 171); 6 (Lil)
0x800b971c[mod 5] (197, 198, 166, 170, 171); then countdown = (rand &
255) x 16. Seen for Lil: 166, 198, 197, 171, 166 (2539-4277). Footstep
sound 11 every 36-38 VSyncs while she ran (0x800563ac).

### Caught: Chuckie in the playpen (0x8007f2e8)

Each frame Chuckie stands in the playpen: the first of 608, 609, 1492 in
mode 6 with +0x692 = 0 gets +0x692 = 1, mode 8, +0x688 |= 64 (in mode 8
it turns to watch the player), then 0x8007f150(baby, player), which with
+0x692 set:

1. 0x80083918 (mode 8, idle), scheduled.
2. PostCine by id: 609 -> 75, 608 -> 76, 1492 -> 77 (0x80078350(player,
   n, 0)), and the printf "Lil Lost" etc.
3. The camera behind the player (0x80026ddc(camera, player, 0)).

Back in 0x8007f2e8: when all three have +0x692 set, PostCine(78, 1).

Cines 75-77 have no hooks, no CineFinish case and no 0x8007737c case.
The baby stays in mode 8 where the cine leaves it (seen: Lil at (4520,
630, 21464) by the playpen, from 2839 on) and has no use callback any
more.

Seen: Lil tagged at 2507; Chuckie put in the playpen at 2600; at 2607
0x8007f2e8, 0x8007f150, PostCine(75); cine 75 from 2635 to 2839; phase
1, tries 3 after it.

### Beaten: the baby reaches post 120 (GoalPost, 0x8007f150)

The event queue (0x80077070 at 0x80077254) sends GoalPost in level 10 to
0x8007f150(baby, player) (levels 3 and 13 have their own cases). With
+0x692 = 0:

- Phase below 3: phase + 1 (0x80078d60(0, 0, phase + 1)); PostCine by id:
  609 -> 41 "Lil Win HideSeek", 608 -> 42 "Phil Win", 1492 -> 43 "Tommy
  Win". CineFinish (0x800783d8 cases 40, 41, 42) sets the event callback
  +0x0c = 0x8007f0e0 (Lil), 0x8007f0a8 (Phil), 0x8007f118 (Tommy); each
  runs 0x8007f068(player, baby) after the cine:
  1. 0x8007f004(baby): leave callback (0x80049e5c), 0x800838b4(baby, 46,
     2048) (hiding again), scheduled, 0x8004fe14(&baby, 0, 1) (a new
     random free spot), +0x692 = 0, use +0x54 = 0x800831a4.
  2. ForceCry(player) (below): a try lost, back to the start.
  3. The camera behind the player.
- Phase 3 or more: phase + 1 (4) and ForceCry(player) at once: no cine,
  no re-hide.

Either way the camera goes behind the player (0x80026ddc).

Seen: Lil (tagged at 2507, left alone) reached post 120 at 4597
(GoalPost), 0x8007f150 at 4599, phase 2, PostCine(41); cine 41 from 4625
to 4907; at 4907 0x8007f004, Lil to entry 4 (15958, -5459, -9497) in mode
1, ForceCry, respawn: Chuckie at (3309, 670, 16563), tries 2, health 255,
no cry.

## Winning: cine 78

The last catch posts the "Loses" cine and cine 78 together; they play
back to back (seen: Tommy caught at 3439, PostCine(77) and PostCine(78);
cine 77 from 3467 to 3672, cine 78 from 3672 to 4450). Cine 78 has no
hooks and no card. 0x8007737c case 77: exit code 0x800c4918 = 1,
0x80076d6c(10, 1) (level 10 played and won, its puzzle piece; see
[objectives](objectives.md)). The port's `ENDINGS` already has 78. Seen:
0x80094bfc(10) and the puzzle screen 0x8009480c(10, 1) at 4450.

## ForceCry (0x8007fd0c, level 10 case)

Level 10 is case 9 (0x80080058):

- Phase below 4: no cry animation (0x8007f920 not called); health 255;
  the callback 0x8007effc (does nothing); tries counted down; tries left:
  to the respawn (0x8007fbf8); none: SwitchEpisode(player, 1).
- Phase 4: the cry (0x8007f920); health 255; tries counted down; tries
  left: nothing more (no respawn; guess: not reached in play); none:
  SwitchEpisode(player, 1) (0x800781f8, the hub).

The first part (a carried object put down) is the generic one. The
respawn is never moved after EpisodeStart, so every lost race puts
Chuckie back where cine 44 left him.

Three lost races cost the three tries: 3 -> 2 (cine 41-43), 2 -> 1 (cine
41-43), and the third is the loss. Nothing else in level 10 hurts Chuckie
(no NPC bites, hazards or damage values found; seen health 255
throughout), so the phase and the tries stay in step (guess).

## How the level ends

- Won: all three babies caught, cine 78, 0x80076d6c(10, 1), exit code 1.
- Lost: a third race lost: phase 4, the cry, the last try,
  SwitchEpisode(1). There is no "lost" cinematic for the level; cines
  41-43 are per race.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT06 | 677 | after cine 44 (arg 5) | Help Chuckie get his glasses back from Angelica by finding all the Rugrats babies' hiding places around the house. Press @ to tag the baby and then race him or her back to the playpen. |

The port's card table (front.rs `card_after`) has 44 -> 677. No other
card in level 10.

## Per-frame ambience (0x80080fe0)

0x800b9618[10]. Only while 1492, 608 and 609 exist and none is in mode 6:
the countdown 0x800c4cc0 minus dt; below 0: speech 0x800b9480[rand & 15]
(293, 405, 406, 417, 421, 422, 424, 438, 439, 440, 441, 442, 408, 409,
410, 411), countdown 600 + (rand & 255) x 16.

Seen: speech 417 at 2379 and 293 at 2667. Speech only.

## Other notes

- Climbing (0x80058da4 at 0x80058e14): in level 10 type 14 (a ledge 906
  to 1033 above the feet) is refused, so Chuckie climbs at most 905.
- The tag works through the same use callback (+0x54) as level 9's kick;
  there the callback is Mr. Friend's, here 0x800831a4.
- 0x8007737c (cine end) and 0x800783d8 (CineFinish) switch on cine - 1;
  the hooks 0x80072ef0 and 0x800732d4 switch on the cine number. The cine
  table flags: 41-43 0x20000200, 75-77 0x20000200, 78 0x2000200.
- [Level 9](level-09.md#the-player-and-the-characters-lp-kind-1) reads
  Mr. Friend's speeds from row 0; NPC init (0x80081bac at 0x80082078)
  picks row 1 in levels 9 and 10 (index 1 = 2, 3 = 6).

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, hooks with arguments,
call redirects, object and room dumps, screenshots): presses start 300,
cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 10`), 0x800c5678 = 10 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 2 (Chuckie,
the level's own character). The start cine was not skipped.

- Cine 44 from 1819 to 2198; at 2198 card 677 (arg 5), EpisodeStart
  0x8007ee54, 0x8004fe14(.., 0, 3); the babies at entries 6, 5, 7 in mode
  1 (speed index 4, health 5); Chuckie at (3309, 670, 16563), room 23,
  tries 3, phase 1. Ids 14, 16, 15, 13, 600 hidden. Chuckie drawn without
  glasses.
- Chuckie put beside Lil upstairs, walked into her: Triangle at 2470,
  state 12, 0x800831a4 from 0x8005f614 at 2507, 0x80083818(Lil, 58),
  speech 158. Circle instead: 0x800831a4 from 0x8005d4a8 at 2473.
- Lil left alone: mode 6, speed 11, footsteps sound 11, taunts 166, 198,
  197, 171; GoalPost at 4597; phase 2; cine 41 4625-4907; re-hidden at
  entry 4; ForceCry without the cry; respawn (3309, 670, 16563); tries 2.
- Chuckie put in the playpen after the tag: 0x8007f2e8 at 2607, cine 75
  2635-2839; Lil in mode 8 at (4520, 630, 21464); tries and phase
  unchanged.
- The win: Lil caught as above; Phil tagged by a redirected call
  (0x800831a4 at 2900) with Chuckie in the playpen: cine 76 2960-3164;
  Tommy at 3401: cine 77 3467-3672, cine 78 3672-4450, 0x80076d6c(10, 1),
  0x80094bfc(10), the puzzle screen.
- The loss: each tagged baby put on post 119 -> 120 near the playpen:
  GoalPost at 2545 (cine 41, tries 2, phase 2), 3334 (Phil: cine 42,
  tries 1, phase 3), 4139 (Tommy: phase 4, ForceCry at 4141 with the cry
  0x8007f920, tries 0); SwitchEpisode(player, 1) at 4561, level 1 loaded;
  no 0x80076d6c.
- Ambience speech 417, 293. Rooms and the playpen object read from RAM.
- Not checked: a natural tag of Phil or Tommy (redirected calls only);
  cine 43; the hinged doors; the class 5 objects; climbing type 14; the
  bars; the sound sources; what each race costs when Chuckie runs (the
  races were set up by pokes, not raced).

## For the port

The port has a Script per level, the playable character by level
(Chuckie for 10 already), climbing, hiding by id or placement,
cinematic-object use, doors, room flags, the follow camera with room
heights, NPC modes 0, 1, 3, 4, 6, 7, 9, 11, posts with flags 1, 2, 4, 16,
32 and the `Goal` event, spawn groups, ForceCry with tries, cards from
level code and the result screens; level 9 adds a use callback on NPCs
(the kick). Level 10 needs none of level 9's health, death or aimed
throws. It needs, in order:

1. A `level10` script: phase, respawn, tries, and per baby "caught"
   (+0x692). EpisodeStart 0x8007ee54: 608, 609, 1492 hiding (mode 1 with
   animation 46, speed index 4) with the tag as their use, each at a
   random free group 0 spot; respawn phase 1 where cine 44 leaves Chuckie
   (3309, 670, 16563), heading 2025; post 120 becomes a goal post (flag
   32). Cine 44's start: the player's head part becomes model 1237 for
   the level. Cine 44's end: hide 13, 14, 15, 16, 600. The card (677) and
   the ending (78) are already in `card_after` and `ENDINGS`.
2. The tag: Triangle (state 12, animation 50, the use at the animation
   event) or Circle (at once) on a touched baby. Refused while a baby
   runs. Otherwise: the baby's spot freed, animation 47, then
   `Posts { prev: 0, cur: spot post }` with the player as target, no
   object collision, Chuckie's line 158 / 160 / 165, the use taken away.
   Generalize level 9's NPC use so Circle reaches it too (0x8005d4a8).
3. Spots: kind 7 records need their post byte (+0x12) and the used bit;
   the pick of 0x8004fe14 (r = rand mod 100 against the rising
   thresholds, the used bit, retries with r = 0) for NPCs, setting the
   NPC's first post. The port's spawn groups place collectibles only.
4. The race each frame: Chuckie in the playpen (template 1032's volume;
   the oracle's test is a vertical segment against its model; a box
   around (5926, 21212) is a guess that fits the points seen) with a
   runner not yet caught: caught (mode 8 watching the player), cine 75 /
   76 / 77 by id; all three caught: cine 78 (won). `Goal` from post 120
   with the runner not caught: phase below 3: phase + 1, cine 41 / 42 /
   43 by id, and after it the baby hides again at a new spot and a try is
   lost; phase 3: phase 4 and the losing cry.
5. `Script::cry` for level 10: below phase 4 no cry animation, health
   255, tries - 1, back to the respawn (or the hub at 0 tries); at phase 4
   the cry then the hub.
6. NPC speed row 1 in levels 9 and 10 (`npc::SPEEDS` is row 0): the
   babies run at 11, not 17.
7. Mode 8 with +0x688 bit 64: the idle NPC turns to face the player (also
   level 6's Chuckie).
8. Climbing: in level 10 the highest ledge is 905, not 1033 (type 14
   refused, 0x80058da4).
9. Small things: the taunts while a baby runs (shared countdown, lines by
   template); the ambience lines (0x80080fe0, only while nobody runs); the
   babies' footsteps (sound 11); the three sound sources replacing the
   kind 3 ones; the object init tweaks (template 126 copy, 527's x, 381
   not climbable with box top -200, 1244 and 61 flag 8; template 403 not
   loaded); the hinged doors (class 4) if they matter; the side factor 2
   in the runners' steering (0x800825fc).

## Unknown

- The exact playpen test (0x8006abbc and 0x8006a714 on model 1442) and
  its extent.
- What animations 46 and 47 are (guess: the hiding pose and the "found"
  reaction).
- What object bit 0x400000 (class 5 objects) and NPC bit +0x688 0x1000 do.
- Whether the hinged doors (LP 88, 89, 151, 152) close off any hiding
  spot; none sits near them.
- What the three sound sources play (sounds 0, 1, 2) and what template
  1192 and the template 126 copy are for.
- Whether template 117's training message (33) shows outside level 16.
- What phase 4 with tries left would do (no respawn); only reachable if
  something else cost a try, and nothing in level 10 seems to.
- Where cines 41-43 and 75-77 leave the babies in general (seen once for
  Lil after cine 75) and Chuckie after cines 41-43 (then the respawn moves
  him anyway).
- Card arg 5.
- What ids 13, 14, 15 and 16 are (cine 44 actors, guess).
- How long cines 44, 41-43, 75-78 really are (the oracle: 379, 282, 204
  and 778 VSyncs).
