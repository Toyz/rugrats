---
title: Level 9, The Mysterious Mr. Friend
status: partial
discs: US
covers: US SLUS_006.50:0x8007b078, US SLUS_006.50:0x8007cac4, US SLUS_006.50:0x8007cc00, US SLUS_006.50:0x80083be0, US SLUS_006.50:0x800834b0, US SLUS_006.50:0x800833a4, US SLUS_006.50:0x80083764, US SLUS_006.50:0x80083728, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80061c98, US SLUS_006.50:0x80061f88, US SLUS_006.50:0x8008864c, US SLUS_006.50:0x80086ef0, US SLUS_006.50:0x80087594, US SLUS_006.50:0x80059b34, US SLUS_006.50:0x80082230, US SLUS_006.50:0x800628a4, US SLUS_006.50:0x8005f584, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x80077864, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x80080e08, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8006fe04, US SLUS_006.50:0x80018598, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x8002953c, US SLUS_006.50:0x800562bc, US SLUS_006.50:0x8008a668, US SLUS_006.50:0x80078bc4, US SLUS_006.50:0x80078c68, US SLUS_006.50:0x800825b0, US SLUS_006.50:0x800307dc, US /DATA/DB09/DB09LP.BIN, US /DATA/DB09/LEVEL09.BIN, US /DATA/HINTS
worklog:
---

# Level 9, The Mysterious Mr. Friend

Everything level 9 needs to be played to its end: Tommy in the basement
workshop, Mr. Friend chasing him, balls and cans thrown at him, the kick,
cine 18 and the three Mr. Friends, the one-try finish and the win. Read
from the pseudo-code and the disassembly; the start, the bites, an aimed
ball and a can hitting Mr. Friend, the kick, each death, cine 18 and phase
2, phase 3, cine 19 and the win, a cry and the last-try loss were run in
the oracle (see Checked in the oracle). Builds on [level 6](level-06.md),
[level 5](level-05.md), [level 3](level-03.md) and [level 2](level-02.md)
(NPC modes, events, ForceCry, cines, cards, spawn groups),
[gameplay](../engine/gameplay.md) and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). "char" is the character (+0x15c =
template +0x10). A VSync is 1/60 s; the oracle's frame counter counts
VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default of 0x80018598). Cine 17 "Mr.
Friend Intro", then card HINT28 (file 699): watch out for Mr. Friend;
Circle picks up balls and cans, Triangle throws them. Mr. Friend (id 46,
char 16, 100 health) chases Tommy and bites for 15. Throws aim at him by
themselves (0x80061c98): a ball (template 777) takes 50, a can (716) 30,
a third toy (390) 25. Triangle with him touched is a kick (state 12): 50,
Tommy's own +0x6d. A counter 0x800c4cd8 starts at 4 and drops at each
Mr. Friend death (0x80083be0). The first death plays cine 18 "Mr. Friend
Mid Pt." and card HINT39 (710): "three against one"; then Mr. Friend 46
comes back with two more (ids 91 and 800, 150 health each), all chasing
Tommy (phase 2). When one is left Tommy has one try (phase 3). The last
death plays cine 19 "Mr. Friend Ending" and wins the level.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd8 | s32 | Mr. Friends left to beat: 4 at EpisodeStart (0x8007b078), minus 1 at each death (0x80083be0), not below 0 |
| 0x800c4cd0 | u8 | phase: 1 EpisodeStart, 2 after cine 18's card (0x8007cac4), 3 at one left (0x8007cc00) |
| 0x800e5830 | s32[3] | respawn position for ForceCry (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| progress +0x23 (0x800c569b) | u8 | tries; 3 at the start, set to 1 by 0x8007cc00 |
| player +0x1f4 | ptr | the aim target; Mr. Friend 46 at EpisodeStart, then 0x80059b34 |

The phase is only written through 0x80078d60, and nothing in level 9 reads
it: ForceCry is the generic case and the room change has no level 9 case
(table 0x80015894 index 6 is the default 0x8007f598). The counter drives
the level. There is no level timer (0x8007f5a8 table 0x800158cc index 7 is
the default 0x8007f6c8), no CheckPoint room, no room cine and no Gameplay
script (no kind 6 record has those bytes).

## Data in DB09

### The player and the characters (LP kind 1)

Level 9's playable character is 1 (0x80018598 default). The loader
(0x800454dc) puts the player on LP 30 (template 11, id 1492). Tommy's
+0x6d is 50 (template 11 +0x32; seen 50), which is what his kick takes.

| LP | template | char | id | position | health | range | speed index | +0x6d | what |
|---|---|---|---|---|---|---|---|---|---|
| 30 | 11 | 1 | 1492 | (18500, 7954, -9585) | | | 4 | 50 | Tommy, the player |
| 36 | 15 | 16 | 46 | (17286, 7679, -7627) | 100 | 2000 | 1 | 15 (template 30) | Mr. Friend, phases 1-3 |
| 37 | 733 | 16 | 91 | (18159, 7684, 373) | 150 | 2000 | 1 | 15 | a second Mr. Friend, phase 2 on |
| 38 | 732 | 16 | 800 | (19057, 7684, -444) | 150 | 2000 | 1 | 15 | a third Mr. Friend, phase 2 on |
| 52 | 3 | 13 | 16 | (13718, -1490, 766) | 5 | 2000 | 3 | 0 | outside every room; no level code (guess: cine actor) |
| 53 | 10 | 8 | 15 | (13577, -1279, 883) | 5 | 2000 | 2 | 0 | outside every room; no level code (guess: cine actor) |

NPC init (0x80081bac, char 16, at 0x80081ff0): +0x58 = 0x800834b0 (hit),
+0x54 = 0x800833a4 (use), +0x6d = 15 (over the template's 30 and 15),
+0x688 |= 264 (8 may attack, 256 death effect). Seen: +0x688 0x108, range
+0x68c 2000, reach +0x690 1024. Templates 732 and 733 have flag 0x2000
(meaning not traced). Speeds (table 0x800b9698 row 0): index 1 = 4, 3 =
12.

Mr. Friend's footsteps (0x800562bc, walk and run animations): sound 132 for
char 16 in levels 9 and 14 (seen 132 every 40 VSyncs).

### Things to throw

| LP | template | count | flags | pickable | +0x6d (damage) | +0x30 | flight sound | where |
|---|---|---|---|---|---|---|---|---|
| 60-65 | 777 | 6 | 0x202 | yes | 50 | 1 (object 0x800000) | 0; 163 at the throw (below) | (31204, 8052, -11085), (30003, 8052, -3772), (31063, 8052, 2026), (21732, 8052, -10805), (20360, 8052, -9736), (20154, 8052, -2066) |
| 48 | 716 | 1 | 0x0a | yes | 30 | 0 | 21 | (31559, 8060, -2546) |
| 49 | 390 | 1 | 0x02 | yes | 25 | 0 | 20 | (26533, 7916, -10560) |

HINT28 calls them "balls and cans" (guess: 777 are the balls, 716 a can).
Template +0x36 (bounce, +0xa2): 777 2000, 716 3200, 390 3500.

### Cine props

| LP | template | id | position | what |
|---|---|---|---|---|
| 31, 32, 33 | 433, 432, 440 | 48, 47, 49 | (23250, 7624, 1957), (22904, 5822, 1724), (20986, 7419, 1985) | cine 17 actors (their faces are set every cine 17 frame, 0x80074ec0) |
| 39-43 | 741-745 | 94, 93, 802, 97, 804 | x 16638..17238, y 6481, z -721..-303 | hidden by cine 17's start hook; 94 and 93 act in cine 18; all shown for cine 19 |

Id 95 is hidden and shown by the hooks but no LP record has it
(0x800461f4(95) finds nothing).

### Collectibles and spawn groups (LP kind 7)

Seven kind 7 records (x at +0x04, y +0x08, z +0x0c, group +0x10,
threshold +0x11):

| entry | group | threshold | position | room |
|---|---|---|---|---|
| 0 | 0 | 100 | (33693, 7740, -1113) | 3 |
| 1 | 1 | 100 | (19456, 6357, 1999) | 3 |
| 2 | 2 | 100 | (15372, 7740, -6564) | 4 |
| 3-6 | 3 | 25, 50, 75, 100 | (24920, 5629, -7822), (33788, 6710, -9220), (15189, 6659, -13950), (15060, 6543, -3066) | 4, 4, 4, 3 |

Reptar bars (template 835) LP 50, 57, 58 go to groups 0, 1, 2; health
(template 863, kind 4, heal 63) LP 51 with LP +0x46 = 4 copies goes to
group 3, one per spot. Placed by 0x8004fe14 as in
[level 4](level-04.md) (not checked in the oracle).

### Sound sources (LP kind 3, 0x8006fe04)

Four kind 3 records. The loader (0x800454dc at 0x80045b24, only when
called with a1 = 0) gives each to 0x8006fe04(sound LP +0x06, position, LP
+0x18, LP +0x1c, LP +0x20), which fills a 40-byte record at 0x800e3218 +
40 x 0x800c434b. All have sound 0 and level 127:

| position | +0x18 | +0x1c |
|---|---|---|
| (22043, 6920, -14094) | 1024 | 3072 |
| (17363, 6920, -14277) | 1024 | 3072 |
| (16321, 6315, -5799) | 534 | 2654 |
| (33034, 6988, -3330) | 534 | 2654 |

(guess: positional ambient loops, inner and outer radius.) The one kind 99
record (words 2, 7) has no case in the loader.

### Other objects

| LP | template | class | what |
|---|---|---|---|
| 22 | 52 | 2 | spins (template +0x18 = 32), trigger 5000 |
| 35 | 708 | 2 | spins (+0x18 = 32) |
| 13-21, 23-29, 34, 44, 45, 47 | 40-58, 167, 168 | 0 | scenery (crates, shelves; flags 2 or 0x42, 64 climbable) |
| 46 | 788 | 0 | (16512, -707, 4088), room 1 |
| 54-56, 59, 66, 67 | 54, 709, 710, 1224, 1244 | 0 | scenery |

The generic object init (0x8004691c) has no level 9 case: table 0x8001369c
index 8 is the default 0x800481c8.

### Rooms (LEVEL09.BIN, LP kind 6)

Five rooms; boxes read from RAM in the oracle (room +0x10..+0x28).

| room | x | y | z | kind 6 | role |
|---|---|---|---|---|---|
| 0 | 14336..26624 | 1024..2048 | 3072..5120 | camera post byte 3, no flag | |
| 1 | 14336..24576 | -3072..1024 | 2048..5120 | camera post byte 3, no flag | template 788 |
| 2 | 14336..34816 | 2048..8192 | 2048..5120 | camera post byte 4, no flag (four records) | template 1244 |
| 3 | 14336..34816 | 3072..8192 | -5120..2048 | camera 3 (flag 4) post 1, song 1 | the north half of the workshop; phase 2 respawn |
| 4 | 14336..34816 | 3072..8192 | -15360..-5120 | camera 3 (flag 4) post 1, song 1 | the south half; the start |

Rooms 3 and 4 have +0x0c = 0x84 (flags 4 and 128). Seen: Tommy stands at
y 7947 on the floor, Mr. Friend at about 7692. Posts (LP kind 2): 1 at
(24571, 3526, -5418) and 2 at (15384, 3991, -1005), both flag 8 (camera
posts). Rooms 3 and 4 use post 1; no room uses post 2, and posts 3 and 4
named by rooms 0-2 do not exist (those rooms have no camera flag).

## Start (EpisodeStart, 0x8007b078)

Installed for level 9 by 0x8007871c (case 9). The level start, in order:
player init on LP 30; cine 17 (0x80074958, always: no "seen" flag for
level 9). Cine 17's start hook (0x80072ef0 case 17) hides ids 800, 91, 94,
93, 802, 97, 804 and 95 (0x80078bc4). Its end hook (0x800732d4 case 17)
is the default (0x80074ec0(0, player id)). CineFinish (0x800783d8 case
16): card 699 (HINT28, arg 4). Then EpisodeStart, then 0x8007737c case 16
(nothing).

0x8007b078(player):

1. Mr. Friend = 0x800461f4(46).
2. Tommy's animation overrides: 0x8008a668(1, {234, 235, 233, 232},
   logical 49, 1) and 0x8008a668(1, {131, 130, 129, 128}, logical 6, 1)
   (logical 6 is the hurt animation of state 9; 49 is not traced).
3. 0x80083764(46, player): mode 4 (follow) on the player, +0x69b = 4,
   think 0x80084084, speed from the table, +0xf4 = 0. Then scheduled
   (0x80049e18).
4. Counter 0x800c4cd8 = 4.
5. 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 712): the follow
   camera, field of view 712. Room 4 is a kind 3 room, so the fixed camera
   at post 1 takes over (seen: camera +0x4a = 3).
6. Respawn = the player's position and angles, phase 1 (0x80078d60).
7. Player +0x1f4 = Mr. Friend (the aim target).

Seen: cine 17 from 1818 to 3205; at 3205 card 699, EpisodeStart, respawn
(30113, 7947, -11549) in room 4, tries 3, health 255. Mr. Friend 46 left
by the cine at (29306, 7694, -11911), right next to Tommy; 91 and 800 in
room -1 (hidden).

## Mr. Friend

### Chasing and biting (mode 4 and mode 5, 0x80084084)

Mode 4 as in [level 2](level-02.md#modes-used-in-level-2): goal = the
target (+0x678, the player); the line of sight test 0x8002cd7c; beyond
twice the range (4000) speed = 2 x table, beyond range / 2 (1000) the
table speed; closer with heights overlapping (0x80083ff0), or touching
(+0xf4 = the player): with +0x688 bit 8, mode 5. Mode 5 (0x800846e4):
state 19, animation 37, Damage(player, +0x678, +0x6d = 15) (0x800780ac),
then mode 1.

Seen: Tommy standing still lost 15 every 36 VSyncs (255 to 0 in 17 bites,
ForceCry at 4038).

### Hit and kick callbacks (char 16 in level 9)

Hit, +0x58 = 0x800834b0(npc), called by a flight that reaches him:

1. Speech 0x800b96c8[(rand & 255) mod 3] (153, 187, 146) (0x8008da4c;
   seen 153).
2. +0x54 = 0 and +0x58 = 0: no more hits or kicks until he recovers.
3. Unless already in mode 1 with animation 29 (+0xa8): animation 8 (blend
   2048), mode 1.

Use, +0x54 = 0x800833a4(npc, user), Tommy's kick:

1. Sound 98 at him, the same speech.
2. +0x54 = 0 (+0x58 stays).
3. Unless mode 1 with animation 29: health (+0x1a4) -= the user's +0x6d
   (Tommy 50), not below 0; animation 8 (blend 2048), mode 1.

Mode 1 (0x800841cc): health 0 or less: mode 10. Else state 19; with the
animation's event bit 0x2000 a wait of 300 + (rand & 3) x 300 dt; at the
animation's end (+0xe0 bit 0x8000000) mode 0, state 0, speed back, and for
char 16 the callbacks come back (+0x54 = 0x800833a4, +0x58 = 0x800834b0,
after 0x800842d8). Mode 0 goes to +0x69b (4, the chase). Seen: after a
kick Mr. Friend stayed in mode 1 (state 19) for about 350 VSyncs, then
chased and bit again.

The callbacks are cleared only by a hit or a kick, not by his own bite, so
a kick works while he recovers from a bite (seen: the kick at 3468 landed
with him in mode 1 after biting).

### Death (mode 10, then 0x80083be0)

Mode 10 (0x80084314) for a char other than 6: with +0x688 bit 256, sound
100 at him (level 9; 172 in level 6) and 0x800858d4(npc, 24, 100) (the
death effect); then 0x80083be0(npc). Seen: death 2 VSyncs after the
killing kick (3468 kick, 3470 death).

0x80083be0, char 16 (at 0x80083c34):

1. Counter 0x800c4cd8 -= 1 (not below 0).
2. Counter 3 and level 9: PostCine(18) (0x80078350(player, 18, 50)).
3. Hide him (0x80078bc4(id)): out of the room lists, room -1, his leave
   callback; he is no longer scheduled (seen +0x9c = 0).
4. Counter 1 and level 9: 0x8007cc00(player): respawn = the player's
   position and angles, phase 3, tries (0x800c569b) = 1.
5. Counter 0 and level 9: PostCine(19).

So: 46 dies (3: cine 18), any two of the next three die (2, then 1: one
try left), the last dies (0: cine 19). Which one dies in which order does
not matter.

## Throwing at him

### Aim (0x80059b34 and 0x80061c98)

Every NPC think runs 0x80082230: 0x80059b34(npc, player) then the
character update. 0x80059b34 (outside levels 20-21): with the direction
from the player to the NPC within 512 of the player's heading (+0x24,
0x80075d7c) and within 768 of its pitch (+0x26, 0x80075de8), the NPC
becomes the target (+0x1f4) when there is none or it is nearer than the
current one; out of that cone, a current target is dropped. Dead
Mr. Friends are not scheduled, so they never drop themselves (guess: the
target can stay on a hidden one until another comes into the cone).

The throw (0x80060b44 -> 0x80061f88 -> 0x80061c98). Level 9's case
(0x80061e70), unless the player is in state 8:

1. The direction and angles from the player to +0x1f4 (0x80075e4c,
   0x800760f4).
2. turn = aim heading - player heading, wrapped to -2048..2048.
3. |turn| under 513: (0, -384, 1024) (0x800b7fe0) turned by the aim
   angles; else turned by the player's angles. Normalised (0x8007652c).
4. Direction y replaced by -300; speed 170 (0x800b7fec).

Seen: a ball thrown with Mr. Friend about 2500 behind and 300 to the side
of the facing: direction (-3652, -300, -442), speed 170; it struck him 12
VSyncs later. A ball thrown with him behind the player flew straight
ahead and missed.

### The flight (0x80061f88, 0x8008864c)

- Flags: 0x190f0700 for objects with bit 0x800000 (the balls, template
  +0x30 = 1), else 0x09070700. Level 9 then ORs 0x40000 (0x8006205c);
  both words already have it. Seen 0x190f0700 for a ball, 0x09070700 for
  the can.
- Template 777 in level 9: sound 163 at the ball as it leaves (0x8006203c;
  seen). (Level 3 plays 53, level 16 171.)
- 0x8008864c: record +0x8e = the object's +0x6d; with it nonzero, record
  flags |= 0x80000 (just before 0x800886e8). So every thing with a
  damage value hurts the characters it reaches.

Two hit paths:

- Flag 0x10000000 (the balls): 0x80086ef0. An object hit: with flag
  0x40000 and a +0x58, call it; then with 0x80000 and a character (+0x00
  bit 0x4000), Damage(player, it, +0x8e) (0x80087234). With +0x58 cleared
  (Mr. Friend recovering) there is neither, so a ball does nothing to a
  staggered Mr. Friend.
- Without it (the can, the toy 390): 0x80087594. With 0x40000 and a +0x58,
  call it; then with 0x80000 and a class 1 object (+0xa6 = 1),
  Damage(player, it, +0x8e) (0x80087924), with or without the callback.
  That includes the player: Damage on a non-AI object goes to the
  player's health (0x800780ac drops a source without bit 0x4000).

Damage (0x80077864) with a target character: its health -= the amount,
not below 0 (char 33 excepted); no level 9 routing.

Seen: a ball hit Mr. Friend (0x800834b0 from 0x80087208, Damage(player,
46, 50), health 100 -> 50, mode 1, hit sound 19, speech 153). The can hit
him (0x800834b0 from 0x800878b8, Damage(player, 46, 30), 100 -> 70), then
bounced back into Tommy: Damage(player, player, 30) from 0x8008792c,
Tommy 255 -> 225.

## The kick (state 12)

The stand handler (0x8005d11c): ACTION (Triangle) not carrying: the
touched object (0x800631d8, +0xf4) with a +0x54 puts the player in state
12. Mr. Friend touching Tommy (he comes to bite) is the touched object.
GRAB (Circle) on a touched object that is not a pickup calls its +0x54,
except templates 15, 732 and 733 (0x8005d470): Circle does nothing to a
Mr. Friend.

State 12's animation is 0x800628a4(): level 9 picks 37 or 50 at random
(rand & 1), other levels 50 (level 2 with +0x30 bit 8: 7). The handler
0x8005f584 calls +0x54(obj, player, 1) at the animation event (+0xe0 bits
2064) or at the end of animation 61, then +0xf4 = 0.

Seen: Triangle at 3420 with Mr. Friend touching: state 12, Tommy rose to y
7659 and back, 0x800833a4 from 0x8005f614 at 3468, health 100 -> 50. Cross
(JUMP) near him only jumps (state 5, no call). HINT39's "press @ near Mr.
Friend to grab him and press % for a kick" (guess: describes the two
state 12 animations; one Triangle does it all).

## Cine 18 and phase 2 (0x8007cac4)

The first death posts cine 18. Cine 18's start hook (0x80072ef0 case 18):
ids 94 and 93 draw +0x40 = 0. Its end hook (0x800732d4 case 18):
0x8001dfb8(camera, 712), id 800 placed at (19913, 7500, 251)
(0x800825b0), 94 and 93 hidden, 800, 91 and 46 shown (0x80078c68), then
the default 0x80074ec0(0, player id). CineFinish (0x800783d8 case 17):
card 710 (HINT39, arg 5) and the event's callback +0x0c = 0x8007cac4, run
after the card. 0x8007737c case 17 does nothing.

0x8007cac4(player), for each of 46, 91 and 800:

1. 0x80083728: +0x694 = 0, +0x695 = +0x698, health = max (+0x66c),
   0x800825b0(npc, home +0x648..+0x650): back home, its room and floor.
2. 0x80083764(npc, player): mode 4 on the player. Scheduled (0x80049e18).
3. Draw +0x40 = 0x800b7d2c[2] (0x8003205c, the character draw;
   0x800307dc(2)).
4. Speed index +0x66e = (rand & 2) | 1: 1 or 3 (speed 4 or 12).

Then respawn = the player's position and angles, phase 2; room sound bytes
0x80078e3c(0, 1) and (2, 1) (rooms 0 and 2 get song 1 if they have none);
0x8006eac4(0, 0) (song 0).

The end hook's placement of 800 is overwritten by step 1 (it runs first).

Seen: kill at 3470, PostCine(18), cine 18 from 3599 to 4222; at 4222 card
710 (arg 5), 0x8007cac4, respawn (24342, 7968, -2529), phase 2, tries
unchanged; all three in mode 4, speed index 1, health 100 / 150 / 150.

46 comes back home at (17713, 7692, -7673) still in state 19 (left from
his death; 0x80083728 does not reset it). In state 19 he does not walk:
seen standing there in mode 4 from 4222 to 6400 while 91 and 800 chased
and bit. Brought near Tommy he bites (mode 5, then mode 1 and state 0 at
the animation's end) and chases from then on (seen). (guess: in normal
play 46 always waits like this until Tommy comes to him.)

## Phase 3: one left (0x8007cc00)

At counter 1 (the second death after cine 18): respawn = Tommy's
position, phase 3, tries = 1. So the next cry ends the level.

Seen: 800 died at 4646 (counter 2), 91 at 4830: 0x8007cc00, phase 3,
tries 1.

## Winning: cine 19

The last death posts cine 19. Cine 19's start hook (0x80072ef0 case 19):
shows 94 and 93 with draw 0x800b7d2c[1] (0x80031b84), shows 802, 97, 804
and 95. Its end hook is the default. CineFinish (0x800783d8 case 18): no
card. 0x8007737c case 18: exit code 0x800c4918 = 1, 0x80076d6c(9, 1)
(level 9 played and won; its puzzle piece; see
[objectives](../engine/objectives.md)). The port's `ENDINGS` already has 19.

Seen: 46 died at 5002, PostCine(19), cine 19 from 5131 to 5765,
0x80076d6c(9, 1), 0x80094bfc(9), the puzzle screen 0x8009480c(9, 1).

## ForceCry (0x8007fd0c, level 9 case)

Level 9 is case 8 (0x8007ff90), the generic one, as in
[level 6](level-06.md#forcecry-0x8007fd0c-level-6-case): a carried object
put down; the cry (0x8007f920) when a respawn is set; tries counted down;
health 255; tries left: to the respawn (0x8007fbf8); none: SwitchEpisode
(player, 1) (0x800781f8, the hub). No level record, no cine.

Mr. Friends keep their health and positions across a cry (nothing resets
them).

Respawns: phase 1 where cine 17 left Tommy (30113, 7947, -11549); phase 2
where he stood when card 710 ended; phase 3 where he stood at the second
phase 2 death, with one try.

## How the level ends

- Won: the counter reaches 0, cine 19, 0x80076d6c(9, 1), exit code 1.
- Lost: the last try (ForceCry), SwitchEpisode(1). In phase 3 that is the
  first cry. There is no "lost" cinematic for level 9.

## Camera in level 9

- Rooms 3 and 4 are kind 3: the fixed camera at post 1 (0x800291f8 ->
  0x800259a0(camera, player, room +0x5c)). Rooms 0-2 have no camera flag.
- CameraSwitch (0x80077c88): the "same kind as last time" skip does not
  apply to kind 3 in level 9 (0x80077d0c), so walking between rooms 3 and
  4 sets the same fixed camera again.
- 0x8002953c: when the camera hold of player +0x188 bit 64 ends
  (0x800c4a3c bit 0x40000000; bit 64 is set during the cry, seen 0x40),
  levels 9, 20 and 21 set the camera again with its kind (0x800291f8(camera,
  player, +0x4a, 0)) instead of the follow camera (0x800290a4). Seen:
  camera +0x4a = 3 before and after the cry.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT28 | 699 | after cine 17 (arg 4) | Watch out for Mr. Friend, he's no friend of yours! Press # to pick up balls and cans. Press @ to throw them and send him back to the drawing board! |
| HINT39 | 710 | after cine 18 (arg 5) | Three against one isn't playing fair. Tommy has more surprises in store for Mr. Friend. Press @ near Mr. Friend to grab him and press % for a kick. |

The port's card table (front.rs `card_after`) has 17 -> 699 and 18 -> 710.

## Per-frame ambience (0x80080e08)

0x800b9618[9]. A countdown 0x800c4cc0 minus dt; below 0:

- While the counter (0x800c4cd8) is above 0: the first of 46, 800, 91
  that exists and has a use callback (+0x54, not recovering) gives speech
  0x800b9468[(rand & 255) mod 10] (1136, 1137, 1138, 1139, 1140, 1141,
  1142, 1144, 1145, 1146); with none, 0x8008da4c(0) (guess: nothing).
- Countdown = (rand & 255) x 16 (0 to 4080 dt).

Seen: speech 1142, 1144, 1138 at 3386, 3688, 3856. Speech only.

## Other notes

- The stick speed factors 10 / 10 (0x800c421c / 0x800c4220) in 0x8005be24
  and 0x80059008 / 0x800590d4 are for character 9 (0x800c5680 = 9), not
  level 9 ([gameplay](../engine/gameplay.md) says level 9). Tommy uses 17 / 8.
- 0x8007737c (cine end) and 0x800783d8 (CineFinish) switch on cine - 1;
  the hooks 0x80072ef0 and 0x800732d4 switch on the cine number. The death
  posts the cine; 0x8007737c(18 or 19, 1) also runs about 100 VSyncs
  before the cine starts (seen at 3572 and 5104, from 0x8007715c), with no
  level 9 effect.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, 16-bit pokes, hooks with
arguments, object dumps, room dumps, screenshots): presses start 300,
cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 9`), 0x800c5678 = 9 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1 (Tommy,
the level's own character). The start cine was not skipped.

- Cine 17 from 1818 to 3205; at 3205 card 699 (arg 4), EpisodeStart
  0x8007b078, respawn (30113, 7947, -11549) room 4, phase 1, tries 3.
  Mr. Friend 46 in mode 4 (+0x688 0x108, range 2000, +0x6d 15, health
  100); 91 and 800 hidden (room -1, health 150).
- Bites: Damage(player, player, 15) every 36 VSyncs from 3460; health 0
  at 4038: ForceCry, tries 3 -> 2.
- A ball (LP 60) picked up with Circle (state 7), thrown with Triangle at
  3326 facing Mr. Friend: flags 0x190f0700, direction (-3652, -300, -442),
  speed 170, sound 163; at 3338 0x800834b0 and Damage(player, 46, 50):
  health 50, mode 1, speech 153, hit sound 19. Facing away: it missed.
- The can (LP 48): flags 0x09070700, at 3344 0x800834b0 and Damage(player,
  46, 30) (health 70); at 3378 Damage(player, player, 30) from the bounce.
- The kick: Triangle with 46 touching at 3420: state 12, 0x800833a4 at
  3468, health 100 -> 50. Cross near him: only jumps.
- Health poked to 50, the kick: mode 10 at 3470, 0x800858d4, 0x80083be0,
  PostCine(18). Cine 18 from 3599 to 4222; card 710 (arg 5), 0x8007cac4,
  three 0x80083728, respawn (24342, 7968, -2529), phase 2, song 0, room
  sound bytes 0 and 2.
- Phase 2: 46 stood at (17713, 7692, -7673) in state 19 through 6400; 91
  and 800 chased and bit (15 each). Tommy cried twice (tries 3 -> 2 -> 1)
  and came back at the phase 2 respawn. Tommy put next to 46: it bit, the
  kick hit it (4408, health 50).
- Deaths with health poked to 0 (they die at their next mode 1): 800 at
  4646 (counter 2), 91 at 4830: 0x8007cc00, phase 3, tries 1; 46 at 5002:
  PostCine(19), cine 19 from 5131 to 5765, 0x80076d6c(9, 1), exit 1, the
  puzzle screen.
- Phase 3 loss: Tommy's health poked to 5, a bite: ForceCry at 6158, tries
  1 -> 0, at 6578 SwitchEpisode(player, 1), level 1 loaded; no
  0x80076d6c.
- Ambience speech 1142, 1144, 1138; footsteps 132.
- Not checked: the toy 390 thrown; a ball hitting a staggered Mr. Friend;
  the spawn groups (bars and health); the kind 3 sound sources; speed
  index 3 (all three got 1); 46 waking by a throw from afar in phase 2;
  rooms 0-2; what ids 15, 16, 47-49, 93-97, 802, 804 do in the cines.

## For the port

The port has a Script per level, the playable character by level, hiding
by id, cinematic-object use, doors, room flags 2/4/32/64/128, the follow
camera with heights and kind 3 fixed cameras, NPC modes 0, 1, 3, 4, 6, 7,
9, 11 and riders, posts, carrying and throwing with strikes on
characters, hazards, healing collectibles, collect events, spawn groups,
movers, COL floor flags, floating, the level timer, gauges and the meter,
ForceCry with tries, cards from level code, the result screens; level 6
adds climbing, scares, timed bursts, breakables and the player swap.
Level 9 needs none of level 6's additions. It needs, in order:

1. A `level9` script: the counter (4), phase, respawn, tries.
   EpisodeStart 0x8007b078 (46 to `Follow(player)` and scheduled, aim
   target 46, fov 712, respawn phase 1); hide 800, 91 and the cine props
   (93, 94, 97, 802, 804) at the start (cine 17's start hook). On each
   Mr. Friend death (0x80083be0): counter - 1; at 3 post cine 18; at 1
   phase 3 and tries = 1 (0x8007cc00); at 0 post cine 19 (already won in
   `ENDINGS`). After cine 18's card (710, already in `card_after`):
   0x8007cac4 (all three back to full health at home, `Follow(player)`,
   shown, speed index 1 or 3 at random, respawn phase 2, song 0). ForceCry
   is the default. Play starts where cine 17 leaves Tommy: (30113, 7947,
   -11549), with Mr. Friend at (29306, 7694, -11911).
2. NPC health and death: Damage on a character takes its health
   (0x80077864); mode 1 at health 0 or less goes to mode 10 (sound 100,
   the death effect 0x800858d4 with +0x688 bit 256, hidden, no longer
   thinking) and calls the level's death hook. The port has no NPC death
   yet.
3. Mr. Friend's mode 1 (0x800841cc): state 19 until the hit animation (8)
   ends plus 300 + (rand & 3) x 300 dt on its event, then back to the
   chase; his hit and use callbacks are off from a hit or kick until then.
   Mode 5 after a bite also goes to mode 1. In phase 2, 46 does not walk
   until it bites or is hit (state 19 kept from its death; seen).
4. The kick: Triangle with an NPC touched that has a use callback: state
   12, animation 37 or 50 at random (0x800628a4), the callback at the
   animation event (0x8005f584): sound 98, speech 153 / 187 / 146, health
   - Tommy's +0x6d (50), animation 8, mode 1. Today Triangle only uses
   doors, cines and the puzzle board. Circle does nothing to templates 15,
   732, 733.
5. Aimed throws: the target picker (0x80059b34: an NPC within 512 of the
   heading and 768 of the pitch and nearest becomes the target, others
   drop out), and level 9's throw (0x80061c98): aim at the target when it
   is within 512 of the facing, else straight; y -300, speed 170. A hit on
   a character: its hit callback (speech, stagger) and Damage by the
   thing's +0x6d (ball 50, can 30, toy 25). Balls (object bit 0x800000,
   flight flag 0x10000000) do nothing while the callback is off; the
   other things damage any character they reach, Tommy too on the bounce.
   Sound 163 when a ball leaves.
6. Camera: after a cry the camera goes back to the room's kind (the fixed
   camera in rooms 3 and 4), not the follow camera (0x8002953c).
7. Small things: the ambience speech (0x80080e08, 1136-1146 while a Mr.
   Friend can be hit); Mr. Friend's footsteps (132); hit sound 19; Tommy's
   animation overrides at EpisodeStart (logical 49 -> 232-235, logical 6
   -> 128-131); the kind 3 sound sources; room sound bytes after cine 18;
   spinning scenery 52 and 708 (class 2).

## Unknown

- What Tommy's logical animation 49 is and when level 9 plays it (232-235
  replace it at EpisodeStart).
- Whether 46 always waits in state 19 after cine 18 in normal play, and
  whether that is intended.
- What templates 777, 716 and 390 look like (balls and cans, guess from
  HINT28), and what template flag 0x200 (777) and 0x2000 (732, 733) do.
- What speech 0 does (0x8008da4c(0) when no Mr. Friend can be hit).
- What the kind 3 sound sources play (sound 0) and what the kind 99 record
  (2, 7) is for.
- What rooms 0-2 are (above and behind the workshop) and whether Tommy
  can reach them; post 2 is never used by a room.
- Where cine 18's end hook means 800 to stand ((19913, 7500, 251) is
  overwritten by 0x80083728).
- Card args 4 and 5.
- How long cines 17, 18 and 19 really are (the oracle: 1387, 623 and 634
  VSyncs).
- What ids 15, 16, 47, 48, 49, 93-97, 802 and 804 are (cine actors and
  props, guess).
