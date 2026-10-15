---
title: Level 2, Grandpa's Teeth
status: partial
discs: US
covers: US SLUS_006.50:0x8007914c, US SLUS_006.50:0x8007c08c, US SLUS_006.50:0x8007c6e0, US SLUS_006.50:0x8007eb68, US SLUS_006.50:0x8007ec48, US SLUS_006.50:0x8007ecc4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80077864, US SLUS_006.50:0x800772e0, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80082264, US SLUS_006.50:0x80082c78, US SLUS_006.50:0x80082ee0, US SLUS_006.50:0x80083634, US SLUS_006.50:0x8008357c, US SLUS_006.50:0x80059328, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x80049870, US SLUS_006.50:0x8004d1a8, US SLUS_006.50:0x8004ea34, US SLUS_006.50:0x8008c5e0, US SLUS_006.50:0x80080648, US /DATA/DB02/DB02LP.BIN, US /DATA/DB02/LEVEL02.BIN
worklog:
---

# Level 2, Grandpa's Teeth

Everything level 2 needs to be played to its end: the three phases, the
characters and objects that matter, the NPC movement they use, damage,
crying and respawn, the 130 s shuffleboard timer, and the win and loss
paths. Read from the pseudo-code and the disassembly. The phase changes
and the NPC set-up values were checked in the oracle (see Checked in the
oracle). Builds on [gameplay](gameplay.md) and
[objectives](objectives.md). That page's "Level 2" section has two
errors, corrected here: the 200 health in phase 3 is Chuckie's, not the
goose's, and the goose is registry id 11, not character 11.

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (object +0x38, LP +0x2a). "char" is the
character, the hierarchy number (+0x15c = template +0x10).

## The goal in one paragraph

A goose (id 11) has Grandpa's teeth: model 190 sits on its part slot
0x1f (0x8007914c). Phase 1: Tommy walks through the yard (a maze, hungry
geese) to room 69, the gap in the fence. Entering it plays cine 3 "Goose
Transition": the goose takes Chuckie. Phase 2: Tommy rides Spike and
chases the goose along a fixed route of posts to room 44, the
shuffleboard court. Entering it plays cine 4 "Shuffle Board Transition".
Phase 3: Tommy stands still at the end of the court and kicks pucks at
the goose, which chases Chuckie round the court. 130 s on the clock. The
goose at 0 health plays cine 5 and the level is won. The words come from
the cards HINT03-05 (files 674-676, seen in the oracle).

## The phase and the in-level respawn

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1, 2, 3 (0 before EpisodeStart) |
| 0x800e5830 | s32[3] | respawn position for ForceCry |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |

0x80078d60(pos, angles, phase) writes all of them (null pointers keep
the old ones) and always sets 0x800e583c = 1.

The progress checkpoint is separate: the CheckPoint event (0x800772e0)
stores the player's position in progress +0x24..+0x2c, its health in
+0x20 and sets +0x30 bit 2. That one is used when the level is loaded
again (restart, or back from the hub): the LP loader puts the player
there (0x800454dc, see [gameplay](gameplay.md)).

## Data in DB02

### Characters (LP kind 1, hierarchy templates)

All built by 0x800497dc (1696 bytes) and 0x80081bac. Template values
come from TEMPLATE.BIN (56 bytes each).

| LP | template | char | id | position | health (+0x18) | range +0x68c (+0x1a) | wait +0x68e (+0x1c) | speed index (+0x1e) | +0x6d (+0x32) | trigger +0xa0 (+0x34) | what |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 181 | 11 | 1 | 1492 | (-2867, -392, 11392) | 5 | 2000 | 0 | 4 | 50 | 15000 | Tommy, the player |
| 175 | 2 | 2 | 1776 | (-3342, -502, 12524) | 250 | 5000 | 0 | 3 | 50 | 15000 | Chuckie |
| 177 | 20 | 21 | 11 | (52565, -527, -28341) | 250 | 4000 | 400 | 5 | 60 | 25000 | the goose; LP +0x40 = 88 (first post) |
| 179 | 9 | 6 | 14 | (-8167, -688, 23550) | 5 | 6000 | 0 | 1 | 0 | 30000 | Spike |
| 152-156 | 443 | 12 | 900-904 | see below | 5 | 9000 | 400 | 2 | 10 | 15000 | the hungry geese (guess from HINT03) |
| 81 | 443 | 12 | 1972 | (52237, 0, -213) | 5 | 9000 | 400 | 2 | 10 | 15000 | a goose never armed (idle) |
| 176 | 3 | 13 | 16 | (4036, -1490, 11448) | | | | | | | not used by level code |
| 178 | 5 | 7 | 13 | (-9658, -1249, 23443) | | | | | | | not used by level code |
| 180 | 10 | 8 | 15 | (3936, -1118, 13301) | | | | | | | not used by level code |

Geese 900-904: (-2014, -527, -53597), (24051, -527, -53624),
(-1960, -527, -24704), (24712, -527, -41078), (9310, -527, -9763).

The player's own health is 255 at init (0x80059a88), not the template's.
The meter's max is that 255.

### Other objects that matter

| LP | template | class | id | position | what |
|---|---|---|---|---|---|
| 186 | 438 | 7 | | (37355, -97, 23424) | the puck: model 756, +0x6d 50, speed +0xbe 200 (template +0x1a), sound 36 (+0x24) |
| 83 | 1200 | 0 | 335 | (33265, -2040, -17396) | hidden at start, shown by cine 3 (room 43, next to the gap; guess: a fence board) |
| 109 | 111 | 0 | 1313 | (-5193, -1602, 15799) | always hidden at start (0x8007914c) |
| 188, 275-280 | 835 | 12 | | (23829..25717, 0, -1097) | Reptar bars 0-6 (kind 3 from template +0x26; bar number LP +0x48); optional |
| 108 | 863 | 12 | | (26625, 0, -1071) | cookies, LP +0x46 = 5 copies (kind 4); optional |
| 274, 281 | 1177 | 12 | | (15001, -1731, -54281), (33133, 5, -4974) | kind 0; the geese's attack ray ignores this template |

Class 12 (0x80049af4) makes LP +0x46 copies of a 232-byte class-11
object (0x80048a44, touch 0x800514b0, kind = template +0x26). See
[objectives](objectives.md) for kinds 3 and 4.

### Rooms (LEVEL02.BIN, LP kind 6)

Room boxes from LEVEL02 (+0x10 min, +0x20 max). Room flags come from
LP kind 6 (record +4.. bytes, see RoomChange below).

| room | x | z | LP kind 6 | role |
|---|---|---|---|---|
| 0 | -15360..26624 | 0..38912 | camera 1, sound 1 | start yard; the respawn (1720, -245, 5608) is here |
| 43 | 32768..34816 | -19456..-15360 | | next to the gap; id 335 stands here |
| 69 | 34816..38912 | -20480..-13312 | cine 3, checkpoint | the gap in the fence |
| 68 | 37888..54272 | -20480..-12288 | camera 2, fixed camera post 125, sound 3 | |
| 45 | 36864..56320 | -39936..-19456 | sound 3 | the goose's yard (post 88) |
| 44 | 30720..46080 | 2048..36864 | cine 4, sound 4 | the shuffleboard court |

Every kind 6 record also sets room flag 16 (meaning unknown).

### Posts (LP kind 2, the 28-byte table at 0x800c4b9c)

250 records. The same table holds the fixed cameras
([gameplay](gameplay.md)) and the NPC waypoints, called posts here.
Index n is 1-based: post n is entry n-1.

```
+0x00 u32      record +0x04 (= n in DB02)
+0x04 s32[3]   position (y is 0 in every DB02 post)
+0x10 u8[4]    neighbour posts, 0 = none
+0x14 u32      flags (record +0x18)
+0x18 u8       kind 7 back link (set after loading)
+0x19 u8       record +0x1d
```

Post flags read by 0x80082c78 and 0x80082ee0:

| bit | effect when the post is reached |
|---|---|
| 1 | end of the route: the think returns -1 (taken off the scheduler) |
| 2 | wait: 0x80083e2c, animation 9, mode 7 |
| 4 | next post is a random neighbour (rand & 3, 40 tries) |
| 16 | loop counter: after +0x697 passes go to neighbour +0x11 |
| 32 | GoalPost event (0x80077ff0); no level 2 handler |
| 64 | jump: NPC state 5 |
| 128 / 256 | speed index +1 / -1 (0..5) |

0x8007914c sets bit 64 on posts 6, 9, 19, 22, 25, 51, 116 and 250 (the
words at table +0xa0, +0xf4, +0x20c, +0x260, +0x2b4, +0x58c, +0xca8,
+0x1b50). All lie on the goose's route.

Without a special flag the next post is the first neighbour that is not
the previous post (+0x694), else neighbour 0.

Routes in DB02 (from the neighbour lists):

- Goose, from post 88 (53284, 0, -28596), room 45: 88, 87 .. 62, 116,
  61 .. 26, 24 .. 16, 250, 15 .. 1, 89, 90. Post 90 (40826, 0, 8145) in
  room 44 ends it. Waits at 88, 77, 70, 61, 42, 28, 18, 8. Jumps at the
  eight posts above.
- Chuckie in phase 3, from post 114: 113 .. 92, 91, 115, 114 and round
  again, a loop around the court (x 32466..42132, z 7307..20128).
- Posts 117-249 carry flag 8 (not read by the AI code; guess: camera
  posts).

### Kind 7 (17 records at 0x800c4bac, 16 bytes)

Position, then bytes +0x0c (group 0..9), +0x0d, +0x0e (a post to link
back to), +0x0f = 0. Only read by 0x80082ee0 for NPCs with +0x688 bit
16, which no level 2 NPC has.

## Phase 1: start (EpisodeStart, 0x8007914c)

Posted before the first frame (0x8007871c, case 2). The EpisodeStart
event itself clears +0x30 bit 8 and the timer 0x800c4430 (0x80077070);
level 2 also gets +0x30 bit 256 (all levels but 1). Then the callback:

1. Bit 64 on the eight jump posts.
2. Without progress +0x30 bit 2 (no checkpoint yet):
   - Chuckie (1776) follows Tommy: 0x80083764(chuckie, player), speed
     index 4, scheduled (0x80049e18).
   - Animations for char 12 (geese): 0x8008a668(12, {1187, 1188, 1186,
     1185}, slot 1..4, 1).
   - Geese 900-904: 0x80083b4c (mode 11, target the player, +0x688 |= 2),
     enter callback +0x48 = 0x80049e18, added to the trigger list
     0x800c4c24, +0x690 = 2000, health 255, +0x688 |= 10.
   - id 335 hidden (0x80078bc4).
3. With it: id 335 shown (0x80078c68). Chuckie stays idle.
4. The goose (id 11) gets model 190 on its part slot 0x1f.
5. id 1313 hidden.
6. Respawn (1720, -245, 5608), the player's angles, phase 1.
7. Room sound bytes (0x80078e3c): room 0 = 1, rooms 68 and 45 = 2 (only
   when the room has none yet).

Level start, in order: player init at 0x800b2940[2] (-7248, -1590,
19713), moved onto LP 181 by the loader; cine 1 "Episode 1 Intro"
(0x80074958, only without +0x30 bit 2); its CineFinish shows card 674
(HINT03, arg 6) at once (0x800783d8); then EpisodeStart. In the oracle
Tommy stands at (-5334, -315, 10864) after cine 1.

The goose is not scheduled in phase 1: it stands at its LP position.

What can hurt Tommy in phase 1: the geese (10 a bite) and falls.

## Phase 1 to 2: room 69

RoomChange (0x80077a54) on entering room 69:

- room flag 2: PostCine(player, room +0x5b = 3), flag cleared (once);
- room flag 32: CheckPoint (0x80077f98, handled by 0x800772e0).

Cine 3 start (0x80072ef0, case 3) shows id 335. Its end
(0x8007737c case 2) resets the camera behind the player (0x80026ddc).
Its CineFinish (0x800783d8) shows card 675 (HINT04, arg 4) and queues
0x8007c08c as the event callback.

## Phase 2: the chase (0x8007c08c)

Also called by ForceCry to fall back from phase 3; the first block only
runs then.

1. Phase 3 before (fall back only): +0x30 bit 8 off, player unfrozen
   (+0x188 bit 1 off), room 44 flag 2 set again (cine 4 re-armed), the
   goose and Chuckie stopped (template flag 8 off, 0x80049e5c), timer off
   (0x80078df8), both gauges freed (0x8008c994).
2. Ride animations: 0x8008a668(6, ids, slot, 1) for slots 3, 4, 28, 5,
   6, 7, 8, 1, 2, 0, 23, 24 (ids 1076-1123), and the same for char 1
   (ids 1036-1079). Player animation 0.
3. Spike (14) rides the player: 0x8008357c(spike, player) (Spike +0x00
   bit 0x80000, AI stopped, appended to player +0x74), health 255/255.
4. If the player is char 1: 0x80059328(player, spike) swaps +0x154
   (hierarchy) and +0x15c (char) between them, sets state +0x08 to the
   player's new char. The player now runs as char 6 (Spike's turn rate
   298, camera 400 high) and the rider shows Tommy.
5. Chuckie: +0x188 bit 16, AI stopped (0x80082534), health = max.
6. The goose: +0x68c = 4000, speed index 5, +0x6d = 0 (no bite damage),
   +0x688 bit 8 off (no attack), health = max (250); Chuckie rides it
   (0x8008357c(chuckie, goose)); placed at (52232, y, -27468)
   (0x800825b0, snapped to the floor); hierarchy +0x0e = 600; posts from
   +0x698 (88) with 0x80083818 (mode 6); scheduled.
7. From phase 3: phase = 2, nothing else. Otherwise: camera position
   (28506, -1980, -17613), respawn = the player's position and angles,
   phase 2.
8. 0x8006eac4(2, 0) (music change, guess).

Riders (0x80083634, run in every player and NPC update): each object in
the carrier's +0x74 list with +0x00 bit 0x4000 takes the carrier's
heading (Chuckie: heading + 2048, he faces backwards), speed and
position with y - 250 (Chuckie: y - 200), then is animated and moved
between rooms.

The goose cannot be hurt in phase 2: Damage on id 11 while level 2 and
phase 2 returns at once (0x80077864).

What Tommy does: follow the goose. It waits at the eight wait posts (see
mode 7) and jumps at the eight jump posts. Card HINT04 says Cross jumps
benches and fences.

## Phase 2 to 3: room 44

RoomChange into room 44: PostCine(4). Cine 4 start (0x80072ef0 case 4)
swaps the player and Spike back (0x80059328). CineFinish shows card 676
(HINT05, arg 4) and queues 0x8007c6e0.

## Phase 3: the shuffleboard (0x8007c6e0)

1. +0x30 bit 8 on; 0x8008a7ec(-1) (unknown).
2. Tommy at (0x91eb, -236, 0x5bf8) = (37355, -236, 23544), heading 2048,
   frozen (+0x188 bit 1: no displacement, turning still runs).
3. Camera: a fixed record at 0x800e5778 (the player's matrix and
   position, y = -1800), then 0x800291f8(camera, player, 4, 1500, 952,
   &(0x92ae, -3510, 0x6b6c), 0x800e5778, 612).
4. The puck spot: (0, 0, 512) turned by the player's angles, plus his
   position: 0x80049818 writes its x, z into all six pucks' home
   (+0xac, +0xb4) and position. In the oracle: (37355, -97, 23032).
5. Riders dropped: 0x800835d0 clears bit 0x80000 of everything on the
   goose and on the player.
6. The goose: +0x68c = 2000, follows Chuckie (0x80083764(goose,
   chuckie), mode 4), speed index 2, +0x688 |= 8, scheduled, health =
   max (250).
7. Chuckie: speed index 4, posts from 114 (0x80083818, mode 6, target
   the player), +0x688 |= 0x1800, scheduled, health 200, max 200.
8. Spike: heading 2048, +0x188 bit 16, placed at (0x8ca0, y, 25000) =
   (36000, y, 25000), mode 8 (0x80083918, idle), scheduled.
9. Two gauges (0x8008c5e0, flag 1): the goose's with model 278 at x =
   (0x800c4a04 * 5 >> 7) - 200, and Chuckie's with model 279 at x = 200 -
   (same), both y = 88 - (0x800c4a06 >> 4), scale (64, 64, 1024), second
   model 282 (which one is the fill is a guess). Each follows its object's +0x1a4: max +0x2a/+0x3a, value
   +0x32/+0x42, object at +0x48. In the oracle: bottom left and bottom
   right.
10. Timer 130 s (0x80078dd8: 0x800c4430 = 39000), phase 3.

### Kicking a puck

The puck pool: six 196-byte objects at 0x800e2770[0..5], cloned from LP
186 by class 7 (0x80049870): use +0x54 = 0x8004d1a8, think +0x44 =
0x8004ea34, home +0xac..+0xb4, +0xbc = template +0x18 (15), +0xbe =
template +0x1a (200). Only puck 0 is in a room list after loading; the
index of the puck in play is 0x800c4134.

1. ACTION (Triangle) in the stand state with the puck as the touched
   object (+0xf4, from the nearby list +0x100 by 0x800631d8: within the
   player's size + 1800 and +-1024 of his heading) enters state 12.
2. State 12 (0x8005f584) uses animation 7 in level 2 with +0x30 bit 8
   (0x800628a4; else 50). At the animation's event (+0xe0 bits 2048 or
   16) it calls puck +0x54(puck, player, 1).
3. 0x8004d1a8: sound 36, +0xa8 = player, home = position, a flight
   (0x8008864c) with flags 0x19070000 (no gravity; ends on the floor;
   collides with objects), start = position, radius = the larger of the
   model box's |min y|, |max y|, direction = the player's facing
   (0x80076040), speed 200, damage +0x6d = 50 (so flag 0x80000 is added),
   lifetime 0, then think 0x8004ea34 every frame (period 68).
4. 0x8004ea34: on its first run marks the puck (+0x00 bit 0x8000) and
   brings the next one (0x800c4134 + 1, mod 6) to its home (0x800493f8)
   and into its room (0x8004a34c). When the flight ends (+0x6b = 255)
   the kicked puck leaves its room lists (0x80049380) and the think
   stops.
5. A puck flight that touches an AI character (0x80086ef0): its +0x58
   touch callback (flag 0x40000), then Damage(player, character, 50)
   (flag 0x80000).

How Tommy gets the puck into +0xf4 while frozen is not confirmed (guess:
pushing the stick still runs the move test, which fills the nearby
list; see Unknown).

Touch callback 0x800511a8 (template +0x0a = 9): saved speed to +0x176,
then by char: the goose (21) five 0x800509d8 (guess: feathers), sound
34, animation 8, mode 1; Chuckie (2) speech 185 in level 2, animation 8
(blend 0x1800), mode 1; Spike (6) and geese (12) animation 8, mode 1.

### Win, lose a try (0x80077864 then 0x8007eb68)

Damage events with +0x30 bit 8 in level 2 go to 0x8007eb68(player,
character) after the health change:

- character health 0, id 11 (the goose): PostCine(player, 5, 0);
- character health 0, anyone else (Chuckie): ForceCry(player);
- else: the character's gauge shows the new health (0x8008ca4c).

250 / 50 = five goose hits win; 200 / 50 = four Chuckie hits cost a try.
The goose's own attack on Chuckie (mode 5 in mode 4) does 0 damage: its
+0x6d was zeroed in phase 2.

Timer: 0x8007f5a8 (from HeartBeat 0x8007733c while 0x800c4430 is set
and nothing is frozen) counts down by dt and draws the seconds. At 0:
sound 117 and TimerExpired with 0x8007ecc4, which is ForceCry(player).

## Damage (0x80077864)

Event (20+ bytes): +0x04 the player, +0x10 the character hit (only kept
when it has +0x00 bit 0x4000, 0x800780ac), +0x14 the amount. Not posted
in levels 1, 11, 17, 23-31.

1. A character: id 11 in level 2 phase 2 is skipped. Char 33 takes no
   health. Else health -= amount, not below 0.
2. No character: the player. Standing on an object (+0xf8) with a
   positive amount: no damage. Else health -= amount, clamped 0..255, the
   health meter shown (0x8008c124). A negative amount heals (cookies).
3. +0x30 bit 8: level 2 0x8007eb68, 3 0x8007e308, 16 0x8007eafc.
4. The player (no bit 0x4000) at 0 health: ForceCry(player) directly.

Sources in level 2: NPC attacks (mode 5: Damage(player, target, +0x6d):
geese 10 on Tommy), pucks (50 on the goose or Chuckie), falls (state 10).

## ForceCry (0x8007fd0c, level 2 case)

Posted once until handled (0x800782d4, latch 0x800c4414). Does nothing
when an exit code is set or +0x30 bit 32 is set.

1. A carried object is put down (0x80062ac4(player, 1)).
2. Tries: progress +0x23 (set to 0x800b2920[2] = 3 at each level start,
   0x80018138) is counted down; the new value is the "tries left" s7.
   Zero to start with counts as 1.
3. By phase, with a respawn set (0x800e583c bit 0):
   - 1: cry; move [player, Chuckie] to the respawn.
   - 2, or 3 with tries left: move [player, Spike]; the goose back to
     (0xcc08, y, -27468), +0x698 = +0x695 = 88, mode 7 (waiting at post
     88).
   - 3 with no tries left: no cry; 0x8007ec48 runs (below).
4. The cry (0x8007f920, not in level 1): speech 2040 + char for chars
   1-5, state 0, speed 0, +0x188 bit 64, +0x30 bit 32, animation 16
   (blend 2560), then its own loop of frames (camera, player draw, sound,
   flip) for up to 2100 dt or until the animation ends; then bits 64 and
   32 off and animation 0.
5. Health = 255.
6. Tries left: phase 3 first runs 0x8007c08c (back to phase 2) and the
   camera reset; the list is moved by (respawn - player position) each
   (0x8007fbf8: position += delta, angles = respawn angles, floor snap).
7. No tries left: SwitchEpisode(player, 1), back to the hub.

0x8007ec48 (the phase 3 loss): timer 0, +0x30 bit 8 off, PostCine(16
"Episode 1 2nd Ending"), both gauges freed. After cine 16 the handler
records the level as played, not won: 0x80076d6c(2, 0) (0x8007737c case
15).

## How the level ends

Cine 5 "Episode 1 Endiing": its start hides the goose (0x80072ef0 case
5); its end (0x8007737c case 4):

- progress respawn +0x24..+0x2c = (0x901c, 1001, 0x2839);
- timer 0, exit code 0x800c4918 = 1, +0x30 |= 1;
- 0x80076d6c(2, 1): level 2 played and won, its puzzle piece, the
  main-level count (see [objectives](objectives.md)).

Level 1's EpisodeStart then puts Tommy at 0x800b8718[2] = (37002, 779,
9866).

Restart (Select, 0x80043d58) or a return later reloads level 2. With
progress +0x30 bit 2 the player spawns at the room 69 checkpoint, cine 1
is skipped, and the RoomChange into room 69 plays cine 3 again (seen in
the oracle).

## NPC movement (0x80084084 and friends)

An NPC is a 1696-byte character. +0x00 bit 0x4000 makes the player
update (0x8005a068) take its stick from 0x80082264 instead of the pad, so
NPCs run the player's state machine, physics and collision. The think
0x80084084 runs from the scheduler (0x80072bf0) once a frame after
0x80049e18 (period 136, the frame's 0x800c3f20 at 30 fps). It calls
+0x50 first (the goose: 0x800537c4, a particle effect), then the mode.
Most modes end in 0x80082230 = 0x80059b34(npc, player) and the player
update 0x8005a068(npc).

### Fields

| offset | type | meaning |
|---|---|---|
| +0x184 | s32 | speed (player units); +0x176 keeps it while stopped |
| +0x1a4 | u16 | health |
| +0x648 | s32[3] | home, the LP position |
| +0x654 | s16 | saved heading (mode 3) |
| +0x66c | u16 | max health (template +0x18) |
| +0x66e | u16 | speed index 0-5 (template +0x1e, clamped to 5) |
| +0x670 | u8 | speed row: 1 in levels 9-10 (and char 3 in 20-21), else 0 |
| +0x674 | u32 | distance to the goal (2D, 0x8001f57c) |
| +0x678 | ptr | target object |
| +0x67c | s32[3] | steering goal |
| +0x688 | u32 | flags, below |
| +0x68c | s16 | range (template +0x1a) |
| +0x68e | s16 | wait time, in VSync(-1) ticks (template +0x1c) |
| +0x690 | u16 | post reached radius / attack reach (1024 at init) |
| +0x694, +0x695 | u8 | previous post, current post (1-based) |
| +0x696, +0x697 | u8 | loop counter, its reload (LP +0x42) |
| +0x698 | u8 | first post (LP +0x40) |
| +0x699 | u8 | mode; +0x69b the mode to go back to |
| +0x69c | u32 | timer or VSync stamp |

Speed table 0x800b9698, s32[6] per row: row 0 = 0, 4, 8, 12, 17, 25;
row 1 = 0, 2, 3, 6, 11, 13. The player's full stick speed is 17, so
index 4 runs like Tommy and the goose (5) is faster.

+0x688 bits seen: 1 goal = +0x678 in mode 7 (else the player), 2 leash
to home, 4 no steering, 8 may attack (mode 5), 16 prefer linked posts,
32 keep thinking out of range / timed idle, 64 aim at the player in mode
8, 128 stop at range (mode 8), 256 death effect, 512 face and animation
13 before an attack, 0x4000 no obstacle probe, 0x8000 death speech.
0x800 and 0x1000 are set (goose, Chuckie) but not read in this code.

Char 21 (the goose) gets +0x688 |= 0x1801 at init; char 12 |= 2
(0x80081bac). Templates 2, 20, 443 call no mode at init.

### The stick (0x80082264)

1. Unless +0x688 bit 4 (or mode 8 without bit 64): steer (0x800820e0):
   +0x2c = heading to +0x67c; +0x24 turns toward it by at most 512
   (0x800c4444) a call.
2. Moving and no bit 0x4000: 0x800825fc probes ahead; results 1-2 turn
   +512, 3-4 turn -512, both with speed x 1.5.
3. Facing vector (+0x178..+0x180) = (0, 0, s) turned by the angles and
   scaled to s (0x80076944): s = speed when |heading - target| <= 1024,
   else speed / 4. +0x1b0 = 4096 when moving.

### Modes used in level 2

| mode | who | does |
|---|---|---|
| 0 | all | mode = +0x69b |
| 1 | all (hit) | health <= 0: mode 10. Else state 19; with animation event 0x2000 (not Chuckie) wait 300 + (rand & 3) * 300 dt; when the animation ends: mode 0, speed back |
| 4 | Chuckie (phase 1), goose (phase 3) | goal = target; LOS test (0x8002cd7c); dist > 2 x range: speed 2 x table; dist >= range / 2: table; closer with height overlap (0x80083ff0) or touching: stop and, with bit 8, mode 5 |
| 5 | geese, goose | state 19, animation 37, Damage(player, +0x678, +0x6d), then mode 1 |
| 6 | goose (phase 2), Chuckie (phase 3) | goal = post +0x695; 0x80082c78 advances posts while dist <= +0x690; at the end of the route returns -1 |
| 7 | goose at a wait post | goal = the player; resume (0x80083efc, mode 0) when (VSync - +0x69c) >= +0x68e, or the player is within +0x68c, or the player lies within 768 of the direction to the next post |
| 8 | Spike (phase 3) | idle; with bit 2 walk home when beyond range / 2 |
| 10 | (dead) | char 6: health 255, mode 0; else effects and 0x80083be0 |
| 11 | geese 900-904 | guard: speed = table; player within +0x68c of home and |dy| < 2800: chase him; attack (bit 8, mode 5) when the ray 0x80090554 reports an object (not templates 1177, 843), he is within +0x690 (2000) and heights overlap. Else with bit 2 go home when he is beyond +0x690 of home |

The geese are armed by the trigger pass: within +0xa0 (15000) their
enter callback 0x80049e18 schedules the think; leaving calls +0x4c =
0x800820ac, which stops it (no bit 32).

## Per-frame ambience (0x80080648)

From 0x800818e0 through the table 0x800b9618[level]. A countdown
0x800c4cc0: with +0x30 bit 8 a random sound 32-35 or speech from
0x800b937c; otherwise, while the goose is in mode 6 or 7, goose sounds
at its position (0x8006f644) or speech 0x800b93c4; else speech
0x800b9398. Speech only, nothing that changes play.

## Checked in the oracle

Driven with `rugrats-hle`: main's level argument patched (0x80018824 =
`li a0, 2`), 0x800c5678 = 2, cards skipped (0x8004472c = 0x1000004a).

- Level start: phase 1, respawn (1720, -245, 5608) flag 1, tries 3,
  health 255, +0x30 = 0x100.
- Spawning in room 69 (progress +0x24 and +0x30 = 3 poked): CheckPoint
  and cine 3 at frame 1069, 0x8007c08c at 2238, phase 2. Player char 6,
  Spike riding 250 above. The goose at frame 4000: mode 7 at post 61
  (+0x694 61, +0x695 60), +0x688 0x1801, +0x68c 4000, +0x68e 400, speed
  index 5, health 250/250, Chuckie riding.
- Spawning in room 44: 0x8007c6e0 at 1349, phase 3, Tommy at (37355,
  -236, 23544) frozen, timer 39000 then counting, the goose in mode 4
  on Chuckie (+0x688 0x1809, +0x68c 2000, speed 2, health 250), Chuckie
  mode 6 on the post loop (health 200/200, +0x688 0x1800), Spike at
  (36000, -245, 25000), six pucks, two gauges and the clock on screen.
- Not checked: a kick. Pad presses in the level did not register in
  these runs (also Cross on in-level cards), so the puck path, the win
  and ForceCry are from the code only.

## For the port

Order:

1. Data: DB02 LP kinds 1, 2 (posts with neighbours and flags), 6 (room
   flags) and 7; TEMPLATE.BIN fields +0x04, +0x07..+0x0b, +0x10, +0x18,
   +0x1a, +0x1c, +0x1e, +0x26, +0x28, +0x32, +0x34; LEVEL02 room boxes.
2. The event queue with RoomChange room flags (cine, checkpoint), Cine /
   CineFinish with the cards 674-676 and the CineFinish callbacks, and
   the per-cine hooks of 0x80072ef0 (3 show 335, 4 swap, 5 hide the
   goose).
3. Phase state (0x800c4cd0, respawn), EpisodeStart 0x8007914c.
4. NPC core: scheduler, AI stick, modes 0, 1, 4, 5, 6, 7, 8, 11, posts,
   riders, the speed table; NPCs running the player state machine.
5. Phase 2 (0x8007c08c) with the hierarchy swap, Phase 3 (0x8007c6e0).
6. Pucks: state 12 with animation 7, the flight with object hits, the
   touch callbacks, Damage routing, gauges, the timer.
7. ForceCry with tries, the cry, the three respawns, 0x8007ec48, cine
   5/16 endings and 0x80076d6c.

## Unknown

- How the frozen Tommy gets the puck as touched object (+0xf4 needs the
  nearby list +0x100, which was empty in the oracle while standing).
- The flight's exact end for a puck: flags 0x19070000, lifetime 0, no
  gravity; whether it stops on the court floor at once or slides.
- Why pad input did not reach the level in the oracle runs (an oracle
  matter, not the game's).
- Post flag 8 (posts 117-249), room flag 16, kind 2 record +0x1d, and
  whether kind 2 posts double as cameras for room 68 (post 125).
- +0x688 bits 0x800 and 0x1000; 0x8008a7ec(-1); 0x8006eac4(2, 0).
- What ids 335 and 1313 are; what chars 13, 7, 8 (ids 16, 13, 15) are
  and whether anything in level 2 moves them.
- The order of cine 16 and SwitchEpisode(1) when the last try is lost in
  phase 3, and whether cine 16 plays before the exit.
- 0x80090554's results (1, 0, an object) and 0x800825fc's probe.
- The gauge offsets 0x800c4a04 / 0x800c4a06 (0 in the EXE).
- Cine 14 is "Episode 2 Ending" (level 3), not a level 2 cinematic.
