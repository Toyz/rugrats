---
title: Level 13, Cookie Race
status: partial
discs: US
covers: US SLUS_006.50:0x80079600, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80077070, US SLUS_006.50:0x80077ff0, US SLUS_006.50:0x80082c78, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80083818, US SLUS_006.50:0x80049e18, US SLUS_006.50:0x80083e2c, US SLUS_006.50:0x80083efc, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x80063d3c, US SLUS_006.50:0x80064838, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x80076c24, US SLUS_006.50:0x80076b94, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x8004a218, US SLUS_006.50:0x8004a318, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80077864, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x80094af0, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x800818e0, US SLUS_006.50:0x80081474, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8005e1f4, US SLUS_006.50:0x80086ef0, US SLUS_006.50:0x80018598, US /DATA/DB13/DB13LP.BIN, US /DATA/DB13/LEVEL13.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/DB01/DB01LP.BIN, US /DATA/HINTS
worklog:
---

# Level 13, Cookie Race

Everything level 13 needs to be played from its start to its end: Tommy
races Angelica from the upstairs of the Pickles house down to the
kitchen, and five cookies on the way are worth two more Reptar bars. It
is a bonus level reached by a hub door, and an Activity. Read from the
pseudo-code and the disassembly; the start, Angelica's whole run and her
win (the loss), Tommy's win with and without all five cookies, the
"already earned" note, a forced cry and activity play were run in the
oracle (see Checked in the oracle). Builds on
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md),
[screens](../engine/screens.md), [level 10](level-10.md) (a race along
posts, GoalPost) and [level 17](level-17.md) (the fourth bar, activity
play).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default case of 0x80018598; LP 65, id
1492). Cine 6 "Race Angelica Intro", then card HINT35 (file 706): race
Angelica down to the kitchen; beat her to the finish line while
collecting the five cookies to earn bonus Reptar bars. When the cine
ends, Angelica (id 600) starts running a chain of posts from post 50
upstairs to post 3 in the kitchen (room 20). She stops at six posts and
waits for Tommy. Tommy entering room 20 first plays cine 21 "Race Ang
Ending 2": won, 2 Reptar bars, 4 when all five cookies were eaten, none
when the bars were won before. Angelica reaching post 3 first posts
GoalPost: speech 723 and cine 20 "Race Ang Ending 1": lost. Both go to
the hub. There is no clock and no HUD of its own. A cry (health 0) ends
the level at once: no cry animation, the hub, nothing recorded.

## How level 13 is reached

- The hub door: DB01 LP 204, template 442 (the cookie box, id 33) at
  (22190, -5387, -7410) in room 1, LP +0x42 = 13, no cine (+0x44 0).
  Its use is 0x8004f78c; the +0x30 bit 16 gate only covers levels 3, 4
  and 6, so the door always works ([gameplay](../engine/gameplay.md#doors-0x8004f78c)).
  0x80076d6c never switches to 13.
- The Activities menu: ACTIVITY_LEVELS 13, 17, 23, 18 (0x80013030),
  +0x21 = 1. See Activity play below.
- The loading screen: "Cookie Race" (111) / "Loading" in the main game,
  "Activity" / "Cookie Race" / "Loading" in activity play
  ([screens](../engine/screens.md)).
- Back to the hub: 0x80078e7c puts Tommy at 0x800b8718[13] = (22563,
  -5365, -6729), room 1, beside the box (seen).

## State

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 at the start (0x80078d60); nothing reads it in level 13 |
| 0x800e5830, 0x800e583c | | respawn, set at the start; level 13's ForceCry case does not use it |
| 0x800c4418 | s32 | set to 1 by the ends of cines 20 and 21 (0x8007757c), cleared at once by 0x80076d6c (0x80076d9c) |
| 0x800c4b64 | s32 | spawn slots: 1 (the cookies) |
| 0x800e2710 | ptr | slot 0's first cookie; cookies are 232 bytes apart |
| 0x800e2740 | s32 | slot 0's copies: 5 |
| 0x800e26e0 | s32 | slot 0's group: 0 |
| progress +0x23 (0x800c569b) | u8 | tries: 1 (0x800b2920[13]) |
| progress +0x30 (0x800c56a8) | u32 | |= 256 by the EpisodeStart post (0x8007871c, not in level 1) |
| progress +0x38 bit 13 | | won before: no bars, and the "already earned" note |
| Angelica +0x690 | u16 | post reach 1536 (the default is 1024) |
| Angelica +0x688 | u32 | |= 0x1800 at the start |

Level 13 has no TimerExpired callback (0x8007f5a8 table 0x800158cc index
11 is the default), no room change case (0x8007f498 table 0x80015894
index 10 is the default), no per-frame ambience (0x800b9618[13] = 0), no
Gameplay script, no CheckPoint room and no level timer. 0x800c4018 is the
default 2600 (0x8007871c).

## Data in DB13

### The house

DB13 is the Pickles house again, with the kitchen as the goal. 33 rooms
(LEVEL13.BIN; boxes read from RAM, room +0x10..+0x28: x0, y0, z0, -, x1,
y1, z1). Flags at +0xc; +0x5b the room cine, +0x5c the camera post,
+0x5d the camera kind, +0x64 the song.

| room | x | y | z | flags | +0x5b | +0x5c | +0x5d | +0x64 |
|---|---|---|---|---|---|---|---|---|
| 0 | 12288..13312 | -4096..1024 | 15360..18432 | 0x10 (16) | 0 | 0 | 0 | 0 |
| 1 | 15360..29696 | -10240..-5120 | -10240..-1024 | 0x80 (128) | 0 | 0 | 0 | 1 |
| 2 | 11264..22528 | -11264..1536 | 24576..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 3 | 4096..8192 | -3072..1024 | 22528..24576 | 0x14 (4, 16) | 0 | 0 | 1 | 0 |
| 4 | 32768..33792 | -4096..1024 | 16384..18432 | 0 | 0 | 0 | 0 | 0 |
| 5 | 32768..41984 | -11264..1536 | 23552..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 6 | 22528..32768 | -11264..1536 | 24576..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 7 | 13312..40960 | -10240..-5120 | 5120..9216 | 0 | 0 | 0 | 0 | 0 |
| 8 | 13312..25600 | -10240..-4096 | 0..3072 | 0x4 (4) | 0 | 0 | 1 | 0 |
| 9 | 8192..13312 | -10240..-5120 | 0..9216 | 0x4 (4) | 0 | 2 | 1 | 0 |
| 10 | 39936..40960 | -3072..1024 | 16384..18432 | 0 | 0 | 0 | 0 | 0 |
| 11 | 15360..19456 | -3072..1024 | 22528..24576 | 0x14 (4, 16) | 0 | 0 | 1 | 0 |
| 12 | -6144..1024 | -11264..1536 | 24576..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 13 | 26624..29696 | -10240..-5120 | -1024..5120 | 0 | 0 | 0 | 0 | 0 |
| 14 | 1024..11264 | -11264..1536 | 24576..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 15 | 31744..41984 | -4096..1024 | -10240..-2048 | 0 | 0 | 0 | 0 | 0 |
| 16 | 13312..28672 | -4096..1024 | -10240..-1024 | 0x4 (4) | 0 | 0 | 1 | 0 |
| 17 | 25600..26624 | 1536..2048 | 3072..3072 | 0 | 0 | 3 | 0 | 0 |
| 18 | 33792..35840 | -3072..1024 | -2048..-1024 | 0 | 0 | 0 | 0 | 0 |
| 19 | 21504..32768 | -4096..1024 | 6144..20480 | 0x14 (4, 16) | 0 | 0 | 6 | 0 |
| 20 | 13312..21504 | -4096..1024 | 6144..22528 | 0x16 (2, 4, 16) | 21 | 0 | 6 | 0 |
| 21 | 33792..39936 | -4096..1024 | 5120..13312 | 0x4 (4) | 0 | 1 | 3 | 0 |
| 22 | 33792..39936 | -4096..1024 | 14336..20480 | 0x4 (4) | 0 | 0 | 1 | 0 |
| 23 | 0..12288 | -4096..1024 | 13312..22528 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 24 | 41984..54272 | -11264..1536 | 23552..46080 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 25 | 0..12288 | -4096..1024 | -2048..13312 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 26 | 0..12288 | -4096..1024 | -10240..-2048 | 0x14 (4, 16) | 0 | 0 | 8 | 0 |
| 27 | 13312..32768 | -4096..1024 | -1024..5120 | 0 | 0 | 3 | 0 | 0 |
| 28 | 35840..37888 | -3072..1024 | 3072..5120 | 0x4 (4) | 0 | 0 | 1 | 0 |
| 29 | 12288..13312 | -3072..1024 | -8192..-5120 | 0x10 (16) | 0 | 0 | 0 | 0 |
| 30 | 32768..39936 | -4096..1024 | -1024..3072 | 0x4 (4) | 0 | 0 | 1 | 0 |
| 31 | 27648..31744 | -4096..1024 | 5120..6144 | 0 | 0 | 0 | 0 | 0 |
| 32 | 44032..53248 | -11264..-6144 | 13312..23552 | 0 | 0 | 4 | 0 | 0 |

Upstairs (y about -5600): rooms 1 (Tommy's room, where play starts), 7,
8, 9, 13. The stairs: room 27 (Angelica jumps down them, states 4 and
10). Downstairs (y about 560): 16, 29, 26, 25, 23, 3, then the long rooms
12, 14, 2, 6, 5, 24 (z 23552..46080, floor about 940), the doorway room
11 and the kitchen, room 20. Room 20 is the finish: flag 2 with cine 21.
Room 1 has flag 128 with song 1 (+0x64 1: 0x8006efcc(0, 0, 30)). LP kind
6 records 14-45 set these bytes; LP kind 3 records 1-13 and the kind 99
record are not traced.

Seen (screenshots): Tommy's bedroom at the start; the kitchen's checked
floor, the back door and a cookie jar in cines 20 and 21.

### The characters (LP kind 1)

| LP | template | char | id | LP position | what |
|---|---|---|---|---|---|
| 65 | 11 | 1 | 1492 | (21959, -5190, -5902) | Tommy; cine 6 leaves him at (22739, -5365, -5925), room 1, heading 711 |
| 142 | 1 | 3 | 600 | (21733, -706, -13634) | Angelica: first post 50 (LP +0x40), range 2000 (+0x1a), wait 400 (+0x1c), speed index 4 (+0x1e), health 250; cine 6 leaves her at (23346, -5625, -4942), room 1 |
| 150 | 9 | | 14 | (21782, -542, -14483) | Spike; outside every room, idle; no level 13 code moves him |
| 200 | 442 | | 33 | (19655, 0, -3405) | the cookie box (the hub door's template); after cine 6 at (21489, -5234, -4822), room 1 |
| 140 | 581 | | 54 | (6682, 711, 514) | the puzzle board (use callback 1, [screens](../engine/screens.md)), room 25 |

Ids 56, 57, 60, 80 and 116 (templates 391, 392, 401, 896, 30) are not
read by level 13's code. Carryable objects (template +0x28 0x10000):
templates 29 (LP 47-50, 151, 152), 133 (LP 101) and 790 (LP 103).

### The cookies (template 863)

LP 208 at (30475, 0, -7302): class 12, kind (+0x26) 4, 5 copies (LP
+0x46), group 0 (+0x48), +0x4a 1 (placed at load), shape 1210, value 63
(template +0x32 to +0x6d). At runtime +0xa6 = 11, +0xa9 = 4 (seen). A
kind 4 collectible ([objectives](../engine/objectives.md#collectibles-touch-0x800514b0)):
taken by walking into it, sound 29, health += 63 (at most 255), the
health meter shown. There is no cookie counter on screen.

### Spawn groups (LP kind 7)

| LP | spot | group | threshold | room |
|---|---|---|---|---|
| 223 | (19793, -6411, 8410) | 0 | 20 | 7 (upstairs hall) |
| 224 | (50088, 208, 29801) | 0 | 40 | 24 |
| 225 | (5709, 208, 10664) | 0 | 60 | 25 |
| 226 | (15354, 208, -1770) | 0 | 80 | 16 |
| 227 | (20488, -2450, 1353) | 0 | 100 | 27 (the stairs) |

0x8004fe14 at load gives each of the five cookies one of the five spots
(see [level 17](level-17.md#spawn-groups-lp-kind-7)), so every spot gets
a cookie; only which copy goes where is random. Seen: all five placed,
in rooms 25, 27, 24, 16 and 7. There are no Reptar bars (template 835)
in DB13.

### Angelica's posts (LP kind 2)

81 posts, LP 228-308 (post n is LP 227 + n). Flags (record +0x18): 1 end,
2 wait, 4 random, 8 camera, 32 goal.

| post | position (x, z) | flags | neighbours |
|---|---|---|---|
| 1 | (10832, 7114) | 2 | 43, 44 |
| 2 | (16226, 19127) | | 51, 3 |
| 3 | (16231, 22251) | 32 (goal) | 4, 2 |
| 5 | (18224, 25736) | 2 | 4, 6 |
| 13 | (42042, 37590) | 2 | 12 |
| 20 | (6112, 33049) | 4 | 14, 66 |
| 32 | (12947, -6664) | 2 | 31, 33 |
| 34 | (23070, -4406) | 2 | 33, 35 |
| 46 | (28129, 7244) | 2 | 47, 45 |
| 50 | (24245, -5570) | | 49 |
| 51 | (16943, 15873) | 1 | 2 |
| 52-55 | y -6586 .. 5898 | 8 | none (camera posts) |

The others are plain links (y 0 in all route posts). From post 50 the
first neighbour not come from gives: 50, 49, 48, 47, 46 (wait), 45, 44,
1 (wait), 43 .. 34 (wait), 33, 32 (wait), 31 .. 21, 20 (random: 14 or
66), then 14-19 and 56-65, or 66-81, both to 13 (wait), 12 .. 6, 5
(wait), 4, 3 (goal), 2, 51 (end). Seen: the 66 branch in every run.

## Start (EpisodeStart, 0x80079600)

The level start, in order: player init on LP 65; cine 6 (0x80074958 case
13); its start hook does nothing (0x80072ef0 case 6 is the default); its
end hook is the default 0x80074ec0(0, player id) (0x800732d4 case 6);
CineFinish (0x800783d8 case 5, 0x800786a8): card 706 (HINT35, arg 5)
only outside activity play; then the runner's "already earned" note
(below); EpisodeStart 0x80079600 (installed by 0x8007871c case 13);
0x8007737c case 5 does nothing.

0x80079600(player):

1. Angelica = 0x800461f4(600). +0x690 = 1536. +0x688 |= 0x1800.
2. 0x80083818(Angelica, +0x698 = 50): mode 6 (+0x699 = +0x69b = 6),
   state 0, current post +0x695 = 50, target +0x678 = the player, think
   0x80084084, speed +0x184 = 0x800b9698[row +0x670][index +0x66e] = 17
   (row 0, index 4).
3. 0x80049e18(Angelica): scheduled (0x80072b48(obj, 136)), +0x00 |= 1.
4. Respawn = the player's position and angles, phase 1 (0x80078d60).

No camera call, no timer, no HUD. Seen: cine 6 from 1819 to 2386 (567
VSyncs); at 2386 card 706 (arg 5), 0x80079600, Angelica mode 6 post 49,
+0x688 0x1800, +0x690 1536, speed 17; tries 1; +0x30 0x100; follow
height 0x800c3f5c 1024, camera +0x4a 1.

## Angelica's run (0x80084084)

Mode 6 walks from post to post (0x80082c78) at speed 17 (85 a VSync,
seen). A post counts as reached within +0x690 = 1536. On reaching one:
flag 32 posts GoalPost (0x80077ff0, at 0x80082d38); flag 2 starts a wait
(0x80083e2c); the next post is the first neighbour that is not the one
come from, a random one at flag 4.

+0x688 bit 0x800: an AI-driven object (+0x00 bit 0x4000) skips object
collision (0x80063d3c at 0x80063db8, 0x80065210, 0x8006592c), so the
blockers and Tommy do not stop her. Bit 0x1000: in 0x80064838 a result
5 clears +0x144 instead of 0x800647b4 (guess: she does not stop at a
ledge). Seen: she jumps down the stairs (room 27) in steps of state 4 and
state 10.

### The waits (mode 7)

0x80083e2c(Angelica) at a flag 2 post: +0x69c = VSync, speed kept in
+0x176 and set to 0, animation 9, mode 7; in level 13 a line from
0x800b96d0[rand & 7] = 608, 609, 610, 643, 645, 646, 664, 723
(0x8008da4c). Each frame of mode 7 (0x800843f0): state 19, face the
player, then resume (0x80083efc: mode 0, speed back, then mode 6) when:

1. 400 VSyncs have passed (VSync - +0x69c >= +0x68e = 400), or
2. the player is within +0x68c = 2000 (2D, 0x8001f57c), or
3. the direction to the player is within 768 of the direction to her
   next post (0x800760f4, 0x8001f650): Tommy is ahead of her. In level
   13 this also plays 0x800b96e0[rand & 1] = 611 or 612.

So she waits for Tommy to catch up, and goes on at once when he is ahead
or near. Seen with Tommy idle in his room: waits of exactly 400 VSyncs at
posts 46 (2542-2942), 1 (3144-3544), 32 (4274-4674), 13 (5766-6166) and
5 (6616-7016); at post 34 (4144) a resume after 2 VSyncs by rule 3 with
speech 611 (Tommy upstairs lay in the direction of post 33). Tommy put
600 from her at post 46 (2700): resume at 2701 by rule 2. Lines seen at
wait starts: 723, 610, 646, 608, 645.

### Bumping or hitting her (0x800511a8, from the code)

Angelica's touch (+0x58, template +0x0a = 9, table 0x800b7e50) is
0x800511a8; char 3 takes case 1 (0x80051220): if in mode 7, resume
(0x80083efc) and animation 6, else animation 8 (blend 2048); mode 1 (hit,
+0x699 = 1); speech 556 or 557 (rand & 1). Called when the player walks
or runs into her (0x8005e1f4 at 0x8005e43c: the touched object's +0x58)
and when a thrown object reaches her (0x80086ef0 at 0x80087200, flight
flag 0x40000). Not seen: Tommy put on her path was pushed aside (+0x188
bit 0x20) and no touch came; a pickup try failed.

## The cookies and the fourth bar (0x80076c24)

0x80076c24() walks every spawn slot (0x800c4b64 of them; lists
0x800e2710, counts 0x800e2740) and counts the copies with +0xa6 = 11 and
+0xa9 = 4 (cookies) and those of them with +0xab bit 2 (taken,
0x80076b94). True when at least one is counted and all are taken. Taken
cookies keep bit 2 for the level.

Seen: the five cookies taken by walking (Tommy put 600 from each spot and
Up held; 2523, 2743, 2991, 3281, 3463; the one at y -6411 in room 7 was
taken with Tommy on the floor at -5383); crumbs (the three sparks).

## Winning (cine 21)

The player entering room 20 posts RoomChange; 0x80077a54 sees flag 2:
PostCine(+0x5b = 21) and clears the flag (0x80077aa8). Only the player
posts RoomChange (0x8004a218 returns -1 for other objects), so Angelica
entering the kitchen does nothing.

Cine 21 "Race Ang Ending 2": start and end hooks the defaults; no card
(0x800783d8 case 20). 0x8007737c case 20 (0x80077570): 0x800c4418 = 1,
exit code 0x800c4918 = 1, 0x80076d6c(13, 1).

0x80076d6c(13, 1) outside activity play: +0x34 |= bit 13; bars (table
0x800152cc index 0, 0x80076e18): none when +0x38 bit 13 is set
(0x800c56b0 & 0x2000), else 2, or 4 when 0x80076c24 is true;
0x80094b10(13, n) sets bits 31 down in +0x48[13], 0x80094ab0(n) "Great
Job! You have earned n Reptar bars!"; after Cross 0x80094bfc(13): level
13 owns no piece, only the 15-bar check (0x80094924); +0x38 |= bit 13 the
first time; not a main level, so +0x22 is not counted. The hub: Tommy at
(22563, -5365, -6729).

Seen (Tommy put in room 20): with all five cookies, RoomChange and
PostCine(21) at 3803, cine 21 3803-4083 (280 VSyncs; Tommy and Angelica
in the kitchen), 0x80076c24 true, 0x80094b10(13, 4), "4 Reptar bars!";
Cross, 0x80094bfc and 0x80094924 at 4458. With none: cine 21 3003-3283,
0x80094b10(13, 2), +0x48[13] (0x800c56f4) = 0xc0000000, +0x34 =
0x2000 at 3300 and +0x38 = 0x2000 after Cross, 0x800c4418 0 at 3300; the
hub at 3520 (0x80078e7c), Tommy at (22563, -5365, -6729) room 1.

## Losing (GoalPost, cine 20)

Angelica reaching post 3 (flag 32) posts GoalPost (0x80077ff0). The
event queue (0x80077070, type 11) has a level 13 case (0x8007722c):
speech 723 (0x8008da4c) and PostCine(event object = the player, 20, 0).

Cine 20 "Race Ang Ending 1": start and end hooks the defaults; no card
(0x800783d8 case 19). 0x8007737c case 19 (0x80077560): 0x800c4418 = 1,
exit code 1, 0x80076d6c(13, 0): +0x34 |= bit 13 and nothing else. The
hub.

The Cine event (type 4, 0x8007712c) drops a cine posted in level 13
while 0x800c4418 is set. 0x80076d6c clears it at once (0x80076d9c), so
the test never fires (from the code; 0 seen at 3300). A late cine after
a win or loss is dropped anyway by the exit code test in 0x8007737c (a2
= 1 with 0x800c4918 set).

Seen (Tommy idle): GoalPost at 7069 (4683 VSyncs, 78 s, after the
start), speech 723, PostCine(20) at 7071, cine 20 7071-7314 (243 VSyncs;
Angelica at the cookie jar, Tommy in the kitchen), 0x80076d6c(13, 0),
+0x34 = 0x2000, +0x38 and +0x48[13] 0; the hub at 7374, Tommy at (22563,
-5365, -6729) room 1. The same frames in three runs (the oracle is
deterministic).

## A cry (0x8007fd0c case 12)

Health 0 from any Damage (0x80077864; level 13 is not one of the levels
0x800780ac drops) posts ForceCry. Level 13's case (0x80080148): s1 = 0,
no cry animation (0x8007f920 not called), health 255, no callback, no
respawn. The generic part counts the tries down (1 -> 0), so the cry
always ends the level: SwitchEpisode(player, 1) (0x800802b4). No cine,
no 0x80076d6c: nothing is recorded.

Seen (ForceCry called at 3001): no 0x8007f920, SwitchEpisode(1) at once,
tries 1 -> 0, no 0x80076d6c; level 1 from 3065. What can hurt Tommy in
level 13 was not traced (the generic Damage posters; guess: falls).

## The "already earned" note (0x80094af0)

The runner (0x80017c0c) shows it for level 13 with +0x38 bit 13, outside
activity play, after the start cine (0x80074958 at 0x800180b8, the note
at 0x80018134), so after cine 6 and card 706. The puzzle-piece note
comes before the cine; this one after. The win then gives no bars and no
bars screen.

Seen (+0x38 bit 13 poked every frame): cine 6, card 706 and 0x80094af0 ->
0x80094728 at 2386, Cross at 2420, EpisodeStart at 2479; the win:
0x80076d6c(13, 1) with no 0x80094b10, 0x80094bfc at once, the hub.

## Activity play (+0x21 = 1)

- No card after cine 6 (0x800783d8 at 0x800786a8).
- No "already earned" note, and nothing recorded: 0x80076d6c returns at
  once (no +0x34, +0x38, bars).
- The cookies are not placed: the loader skips templates 835, 863, 1218
  and 1219 in activity play (0x800454dc at 0x80045fa0). They stay at LP
  208's spot (30475, 0, -7302) (seen; guess: out of reach). The race is
  the same.
- The ambience is picked by character (0x800818e0): char 1 uses
  0x800b9618[25] = 0x80081474: countdown 0x800c4cc0 minus dt; below 0,
  speech 270 when the player is in state 15, else 0x800b950c[(rand &
  255) mod 14] (80, 1217, 269, 1219, 75, 1205, 11, 1210, 1200, 1214, 269,
  80, 75, 11); then (rand & 255) x 16 + 1800 dt.
- The runner forces the exit to -1: "Returning to the Activities Menu"
  and the Activities menu ([screens](../engine/screens.md)).

Seen (+0x21 poked at 1805 and 1815; Tommy put in room 20): no card;
cookies at (30475, 0, -7302); 0x80081474 every other VSync, speech 80 at
2567; cine 21 3003-3281; 0x80076d6c(13, 1) with no bars; 0x80018264 and
0x800183ac at 3281; the Activities menu (0x8003b7bc) at 3340; +0x34 and
+0x38 0.

## Camera

The level's default: the follow camera, mode 1, height 0x800c3f5c 1024
(seen). Rooms with flag 4 post CameraSwitch(+0x5d) (0x80077a54 at
0x80077b34): kind 8 rooms (2, 5, 6, 12, 14, 23-26) height 1600, kind 6
(19, 20) height 200, kind 1 rooms (3, 8, 9, 11, 16, 22, 28, 30) the
default, room 21 kind 3 a fixed camera (+0x5c 1). Seen: room 19 at 2603
CameraSwitch(6), 0x800291f8(.., 1, 200), height 200; room 2 at 2803
CameraSwitch(8), height 1600.

## What does not happen in level 13

- No clock, no counter, no level HUD; only the health meter (cookies,
  damage).
- No tries to lose: one cry is the hub.
- Angelica never hurts Tommy: 0x80084084 posts Damage only in mode 5
  (0x800846e4), and she was seen only in modes 0, 6 and 7.
- Spike (id 14) and the box (id 33) do nothing.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT35 | 706 | after cine 6 (arg 5), not in activity play | Race Angelica down to the kitchen. Win the race and get the cookies. Watch out for areas that are blocked off. Beat Angelica to the finish line while collecting the five cookies to earn bonus Reptar bars! |

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 6 | Race Angelica Intro | the level start | none | default | nothing (card 706) |
| 20 | Race Ang Ending 1 | GoalPost (0x80077070 level 13 case) | none | default | lost, 0x80076d6c(13, 0) |
| 21 | Race Ang Ending 2 | room 20, flag 2 | none | default | won, 0x80076d6c(13, 1) |

Cine table flags (0x800b8bd0 + 16 x cine + 4) are 0x200 for all three.
Seen lengths (oracle VSyncs): 6 567, 20 243, 21 280 (278 in activity
play).

## Checked in the oracle

The level 17 probe copied to the scratch folder as l13probe, with a
level 13 line (0x800c4418, 0x800c4b64, the spawn lists, health, +0x30,
+0x38, ids 600, 14, 33, 54 with mode, post, speed, +0x690, +0x688, and
each cookie's place, room, +0xa6, +0xa9, +0x6d and taken bit). Presses
start 300, cross 500, 650, 800, 1650, 1800; main's level argument
patched (0x80018824 = `li a0, 13`), 0x800c5678 = 13 at the level runner,
cards skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1
(Tommy anyway). The start cine was kept. Moves were teleports (player
+0x18..+0x20 and the two y copies) and held Up.

- Start: cine 6 1819-2386, card 706 (arg 5), EpisodeStart, Angelica mode
  6 from post 50 with +0x690 1536 and +0x688 0x1800, tries 1, phase 1.
- Rooms read from RAM; cookies placed at all five spots.
- Angelica's whole run with Tommy idle: the posts, the 400-VSync waits,
  the early resume at post 34, the random branch (66), the stairs jumps,
  the wait lines, GoalPost at 7069, speech 723, cine 20, the hub, +0x34.
- Rule 2 of the wait (Tommy near).
- The win by entering room 20: cine 21, 4 bars with all five cookies, 2
  without, +0x34, +0x38, +0x48[13], the hub spot.
- The "already earned" note after the cine, and no bars.
- A forced cry: no cry animation, the hub, tries 1 -> 0, nothing kept.
- Activity play (+0x21 poked): no card, no cookies, the char 1 ambience,
  no bars, the Activities menu.
- The camera heights in rooms 1, 19 and 2.
- Not checked: the hub door into level 13 (from the data); the
  Activities menu itself; the loading screens; walking the route
  (everything was teleports); bumping or hitting Angelica (0x800511a8,
  from the code); the random branch to post 14; what hurts Tommy; the
  fixed camera of room 21; the speech lines' content; Pause, Quit Level
  and Restart; the board (id 54).

## For the port

The port has the Script trait, Tommy as the default character, NPC mode
6 posts with flags 1, 2, 4, 16 and 32 (GoalPost as `npc::Event::Goal`),
mode 7 waits (time or the player near), riders, hiding by id, healing
collectibles (kind 4) placed at load at spawn groups, room flag 2 cines,
room camera kinds, `ENDINGS` 20 (lost) and 21 (won), `start_cine` 13 ->
6, `card_after` 6 -> 706, the bar award (2, or 4 by `fourth_bar`, none
with +0x38 bit 13), the "already earned the reptar bars" note,
`cried()` sending a level out of tries to the hub, and activity play.
Level 13 needs, in order:

1. A `level13` script. At EpisodeStart (after cine 6 and its card):
   Angelica (id 600) put in mode 6 at post 50 (her LP +0x40), where cine
   6 leaves her ((23346, -5625, -4942), room 1), speed index 4 (17),
   moving from that frame on; she must stand still during cine 6 and the
   card. No timer, no HUD, no camera call.
2. A post reach per NPC: the port's `npc::REACH` is a constant 1024;
   Angelica's is 1536 (+0x690). It decides when she turns and when post
   3 posts GoalPost.
3. GoalPost in level 13: on `Event::Goal` from Angelica, speech 723 and
   cine 20 (the generic ending then records the loss and goes to the
   hub). Only the first one counts (the exit code stops later cines).
4. The win is data the port already reads: room 20, flag 2, cine 21. The
   script sets `g.fourth_bar` when all five cookies (template 863, kind
   4) are taken (0x80076c24), and not when none are counted (activity
   play).
5. The waits in level 13: on starting a wait, speech 0x800b96d0[rand &
   7]; the third resume rule (the player within 768 of the direction to
   her next post, 0x800843f0) with speech 0x800b96e0[rand & 1]. The
   port's `Mode::Wait` has only the time and the near rule.
6. ForceCry for 13: `cry_animation` false (no cry), and then the generic
   `cried()` with tries 1 goes to the hub with nothing recorded. Without
   a script the port would play the cry first.
7. The "already earned the reptar bars" note after cine 6 and its card,
   not before the cine (the port puts every note before the start
   cinematic; the puzzle-piece note does come first).
8. Activity play: no card 706 (the port's `card_after` 6 is
   unconditional); the five cookies not placed at load (template 863 is
   one of the four the loader skips); the ambience by character
   (0x80081474 for Tommy).
9. Angelica ignores object collision (+0x688 bit 0x800): the port's NPCs
   do not collide with objects, so nothing is needed unless that
   changes. She drops down the stairs: the port's `go` snaps down to a
   lower floor, without the jump states.
10. Small things: her touch (0x800511a8: walked into or hit by a throw,
    she stops in mode 1 with speech 556 / 557, from the code); song 0 in
    room 1 (flag 128); Spike out of every room; the hub return spot
    (22563, -5365, -6729).

## Unknown

- What +0x688 bit 0x1000 does exactly (0x80064838 result 5; guess:
  ledges).
- How long mode 1 lasts after a bump, and whether a bump or a throw is
  meant as a way to slow her (not seen).
- What hurts Tommy here, if anything (the generic Damage posters).
- "Areas that are blocked off" (HINT35): which objects block the way.
- Room flag 16; LP kind 3 records 1-13 and the kind 99 record; the
  camera post of room 21 (+0x5c 1) and posts 52-55.
- What the speech lines say (608-612, 643-646, 664, 723, 556, 557).
- Card arg 5.
- Whether the cookies at (30475, 0, -7302) in activity play can be
  reached (guess: no).
- The cines' lengths from natural play, and where cine 6 leaves Angelica
  with other timings.
