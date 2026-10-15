---
title: Level 20, No More Cookies
status: partial
discs: US
covers: US SLUS_006.50:0x80079ff0, US SLUS_006.50:0x80079cb0, US SLUS_006.50:0x80079ee8, US SLUS_006.50:0x80079f1c, US SLUS_006.50:0x80079f5c, US SLUS_006.50:0x8007a06c, US SLUS_006.50:0x80057950, US SLUS_006.50:0x80057570, US SLUS_006.50:0x80057518, US SLUS_006.50:0x800574cc, US SLUS_006.50:0x80057468, US SLUS_006.50:0x80057bc4, US SLUS_006.50:0x80059b34, US SLUS_006.50:0x8008046c, US SLUS_006.50:0x8005f3a4, US SLUS_006.50:0x8004df94, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80082478, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x8005e1f4, US SLUS_006.50:0x8005be24, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x80060b44, US SLUS_006.50:0x80061f88, US SLUS_006.50:0x8003205c, US SLUS_006.50:0x80031a28, US SLUS_006.50:0x800494d8, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80018598, US /DATA/DB20/DB20LP.BIN, US /DATA/DB20/LEVEL20.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 20, No More Cookies

Everything level 20 needs to be played from its start to its end: the
babies keep a box of cookies away from Angelica in the Pickles kitchen
and eat all fifteen. The player throws the box from baby to baby and
becomes whoever catches it. Read from the pseudo-code and the
disassembly; the start, eating, a throw and the body swap it causes, two
swaps in a row, Angelica's catch, the retry, the last-try loss and the
win were run in the oracle (see Checked in the oracle). Builds on
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md),
[screens](../engine/screens.md), [level 12](level-12.md) (the same
house) and [level 9](level-09.md) (aimed throws).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player starts as Tommy (char 1, the default case of 0x80018598).
Level 20 is a bonus level: 0x80076d6c starts it when the second main
level is completed (+0x22 = 2, printf "should trigger
LVL_NO_MORE_COOKIES next", 0x80076fb4). Cine 63 "No More Cookies Start",
card HINT07 (file 678): Angelica got sick eating too many cookies; eat
all the cookies before she gets any more; run from her, press Triangle to
throw the box to the baby with the arrow over the head, press Circle to
eat while standing still. The box (template 442, id 33) starts in hand
with 15 cookies. Circle (or Square) while standing eats one. Triangle
throws the box in a lob to the targeted baby (Tommy 1492, Chuckie 1776,
Phil 608, Lil 609); when the box comes within 1400 of that baby the
player becomes the baby. Angelica (600) runs after the box and lunges;
if she touches the player while the box is held, the box flies up and it
is a loss (cine 65 "No More Cookies Lose"). Tries are 2: the first loss
starts the round again with 15 cookies, the second sends the player to
the hub. The fifteenth cookie plays cine 64 "No More Cookies Win": 3
Reptar bars and the hub. Nothing in the level hurts; there is no health
meter.

## How level 20 is reached

- Only once: 0x80076d6c (a main level of 2-6 or 8-12 completed for the
  first time) counts +0x22; at 2, SwitchEpisode(20) (0x80076fb4,
  0x80077028), +0x30 bits 1-2 cleared.
- No hub door leads to 20 (the door list in
  [gameplay](../engine/gameplay.md#doors-0x8004f78c)) and it is not an
  activity (0x80013030: 13, 17, -2, 18). Won or lost, it is not played
  again in that game (from the code).
- The loading screen is the "Bonus Level" one
  ([screens](../engine/screens.md)).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4bd6 | s16 | cookies left; 15 at each round's start |
| 0x800c42a4 | ptr | the box (template 442 or 259 in levels 20-21, 0x800494d8) |
| 0x800c42a8 | ptr | the baby the box was thrown to; 0 after a swap |
| 0x800c4420 | ptr | set to the player when the box reaches 0x800c42a8; the swap runs on it |
| 0x800c4648 | ptr | the counter's HUD item (model 313 and "%d") |
| 0x800c4298 | ptr | level 21's gauge; 0 in level 20 |
| 0x800c4410 | s32 | last camera kind; 1 written by cine 65 so kind 3 is set again |
| 0x800e5778 | object | a dummy target; its position +0x18..+0x20 is set by the round start |
| player +0x1f0 | ptr | the box while held |
| player +0x1f4 | ptr | the throw target: a baby, or the dummy |
| player +0x200..+0x20c | s32[4] | the stored aim: direction (4096 unit) and speed |
| player +0x188 | u32 | bit 4 throw requested, bit 1 frozen during it (levels 20-21) |
| 0x800c56a8 (+0x30) | u32 | bit 64: Angelica knocked the box (the cry follows) |
| progress +0x23 (0x800c569b) | u8 | tries: 2 (0x800b2920[20]) |
| 0x800e5830, 0x800e583c | | respawn, phase 1, set at the first round's start only |

Level 20 has no level timer (0x8007f5a8 has no case), no room change
case, no CheckPoint room, no room cine and no Gameplay script. Its
per-frame ambience is 0x800b9618[20] = 0x8008046c (also level 21's).

## Data in DB20

### The house

49 rooms (LEVEL20.BIN; boxes read from RAM, room +0x10..+0x28), the same
Pickles house as [level 12](level-12.md#rooms-level12bin-lp-kind-6). The
kitchen is room 19 (x 21504..32768, z 6144..20480), with 20 (x
13312..21504, z 6144..22528) and 21 beside it. Kind 6 records give rooms
19, 20 and 21 camera kind 3 on post 1 (+0x5c = 1), rooms 19 and 20 room
flag 128 with song byte 1, and kinds 1, 8 and 0 elsewhere. Post 1 (LP
126, kind 2) is at (14147, -1080, 18587), flag 8. Three kind 3 records
(LP 30-32) and one kind 99 record are not traced.

### The characters (LP kind 1)

| LP | template | char | id | LP position | what |
|---|---|---|---|---|---|
| 107 | 11 | 1 | 1492 | (16848, 1001, 16390) | Tommy, the first player; cine 63 leaves him at (25993, 779, 17711), room 19 |
| 111 | 2 | 2 | 1776 | (17499, 686, 8796) | Chuckie; health 250, range 5000, speed index 3 |
| 108 | 7 | 5 | 608 | (28389, 671, 15900) | Phil; health 5, range 2000, speed index 4 |
| 109 | 6 | 4 | 609 | (27863, 674, 9567) | Lil; health 5, range 2000, speed index 4 |
| 110 | 1 | 3 | 600 | (23055, 546, 13985) | Angelica; health 250, range 2000 (+0x68c), reach 1024, touch 9 |

Values in RAM at 2350. NPC init (0x80081bac case char 3, at 0x8008204c)
sets Angelica's speed row +0x670 = 1 in levels 20-21, so her speed index
2 gives 3 (table 0x800b9698 row 1) where row 0 gives 8.

### The box (template 442, id 33)

LP 112 at (18035, 789, 15553). Template 442: class 0, pickable (+0x28 =
1), shape 760; object flags 0x00010000 (throwable, not class 13). Object
init (0x800494d8 at 0x80049538) keeps it in 0x800c42a4 in levels 20-21.
Its draw +0x40 is 0x80031b84 (not 0), so a cine start hook or a cry puts
it down if it is held.

### Reptar bars (LP kind 7)

Two spots: (29789, 615, 19340) group 0 and (27569, 615, 7152) group 1,
threshold 100, for the two bars (template 835, LP 106 and 122, one copy
each). Not taken in the oracle.

## Start (EpisodeStart and the round start)

0x8007871c case 20 installs 0x80079ff0. The level start, in order: player
init on LP 107; cine 63 (0x80074958 case 20; no start hook, the default
end hook 0x80074ec0(0, player id)); CineFinish (0x800783d8 case 62):
card 678 (HINT07, arg 5); EpisodeStart 0x80079ff0; 0x8007737c case 62:
0x80079ee8.

0x80079ff0: 0x800c4420 = 0; ids 609, 608, 1776 moved to their homes
(0x800825b0 with +0x648..+0x650); song 0 (0x8006eac4(0, 0),
MidiPlaySong).

0x80079ee8(player): the round start 0x80079cb0, then the respawn at the
player's position and angles, phase 1 (0x80078d60).

0x80079cb0(player), the round start:

1. Cookies 0x800c4bd6 = 15. 0x800c4648 +0x78 = 1 (with 0x800c4648 = 0 at
   the first round this writes address 0x78; seen 0 at 2300).
2. 0x800c4420 = 0, +0x30 bit 64 cleared.
3. The box in hand: player +0x1f0 = box, the box out of the room lists
   (0x80078b00), the grip (0x8005ff88(player, 1)). The hand model is not
   changed in level 20.
4. The dummy's position (0x800e5790..98) = (22086, 775, 12864).
5. Ids 1776, 608, 609: idle (0x80083918, mode 8), +0x688 |= 66 (2: go
   home when more than range / 2 from it; 64: face the player),
   scheduled (0x80049e18).
6. Id 600, Angelica: speed index 2, follow (0x80083764, mode 4) with
   +0x678 = the box, +0x688 |= 128 (may lunge), scheduled, +0x1f4 = the
   player.
7. The counter: model 313 (a green cookie box) at scale (128, 128, 1024)
   and the text "%d" (15) in white, both at (-160, 65 - (0x800c4a06 >>
   4)) (0x8008f8f4, 0x80090004), kept in 0x800c4648: bottom left.
8. Song 0.

Then RoomChange for room 19 (flag 4, kind 3) posts CameraSwitch(player,
3) (0x80077b44): the fixed camera on post 1.

Seen: cine 63 from 1819 to 2305 (486 VSyncs); at 2305 card 678 (arg 5),
0x80079ff0, 0x80079ee8, 0x80079cb0; CameraSwitch(3) at 2306; Tommy at
(25993, 779, 17711) room 19 with the box, respawn there phase 1, tries
2, +0x30 0x100, cookies 15, Angelica mode 4 after the box, the babies in
mode 8. Screenshot: the kitchen from a fixed camera, the green box with
"15" bottom left.

## Level 20 in the player code

Only for the non-AI player (+0x00 bit 0x4000 clear), in levels 20-21:

- Every player update runs 0x80057950 (from 0x8005a068 at 0x8005b14c):
  eating, the catch test, the floor-box loss, the counter text.
- The stand / walk input (0x8005be24) is skipped while +0x188 bit 1 is
  set (the throw).
- The walk modifier is off (0x8005c060: s6 = 0); Square eats instead.
- GRAB while carrying never puts the box down (0x8005d11c step 1 skips
  levels 5, 20, 21).
- Entering state 28 (0x8005b2d8 at 0x8005b99c) plays sound 121 at the
  character in levels 20-21 (Angelica's lunge).
- Characters drawn (0x8003205c at 0x80032440): the one in player +0x1f4
  gets 0x80031a28, a sprite above it: the arrow. Seen in screenshots
  over Lil and Phil.
- NPC idle reset (0x80082478) uses animations 35, 17 and 47 in levels
  20-21 (0, 17, 0 elsewhere).
- 0x8006f2fc uses sound bank 13 in level 20 (14 elsewhere); no level 20
  caller traced.

## Eating (0x80057950)

With the box held (+0x1f0 not 0), Circle (0x800c4988) or Square
(0x800c4984) pressed and state 0 (standing):

1. Level 20: sound 123 (0x8006f26c, not positional), animation 42, state
   19. (Level 21: a random positional sound, animation 40.)
2. Cookies - 1. Above 0, nothing more.
3. At 0: cookies 0, +0x1f4 = the dummy, throw requested (+0x188 |= 4,
   animation 18), the hand part model = the character's empty hand
   (0x80057468: char 1 7, 2 22, 3 37, 4 54, 5 69), then 0x8007a06c:
   +0x1f0 = 0 and PostCine(64). With +0x1f0 cleared the throw never
   happens.

Each frame, held or not, the counter text is rewritten with the cookies
left (0x800c42ac "%d", 0x80090214) and 0x8008ca80(0x800c4298, cookies)
is called (0x800c4298 is 0 in level 20).

Seen: Circle pressed every 24 VSyncs: one cookie every about 48 (state 19
lasts about 40 VSyncs), 15 -> 10 between 2333 and 2504, sound 123 each.
As Lil after a swap: sound 123, animation 42, 15 -> 14. A full natural
eat takes at least 15 x 40 VSyncs, longer than Angelica needs to reach
the player from the start (about 200 VSyncs), so the player has to keep
throwing.

## Throwing to a baby

### The target (0x80059b34)

Every NPC think runs 0x80082230: 0x80059b34(npc, player) then the
character update. In levels 20-21 only templates 2, 6, 7 and 11
(Chuckie, Lil, Phil, Tommy; never Angelica) are candidates; any other
NPC that is the current target is dropped. A candidate becomes +0x1f4
when the direction to it is within 512 of the player's heading and within
the pitch limit, and it is nearer than the current target (the same cone
as [level 9](level-09.md#aim-0x80059b34-and-0x80061c98)); out of the cone
a current target is dropped. Then, in levels 20-21 with a target,
0x80057bc4(player, +0x200) stores the aim:

- Direction: (0, -2048, 2048), or (0, -4096, 100) with +0x30 bit 64,
  turned by the horizontal heading from the player to the target and
  normalised: a 45-degree lob (or nearly straight up).
- Speed +0x20c: the square root of a term in (d - 1024), d the
  horizontal distance, over sin 512 x cos 512 (a ballistic range
  formula; guess).

### The throw

ACTION (Triangle) pressed with the box held:

- Standing (0x8005d11c, animation at 0x8005d24c): only with a target; +0x188 |= 5
  (throw, frozen), animation 18 blend 0x2800, speech 1159 or 1167 (rand
  & 1).
- Walking (0x8005e1f4 at 0x8005e6a0): the same without the speech.

At the release (0x80060b44, the levels 20-21 branch before 0x80060d4c): the direction
and speed are the stored aim; +0x188 bit 1 cleared; 0x800574cc(player):
when +0x1f4 is an AI character, its mode = 8, its +0x688 bit 2 cleared
(it stops walking home) and 0x800c42a8 = it. The launch (0x80061f88)
gives the box flags 0x09010700 (other objects 0x09070700). The cookie
count is not changed (step 5 needs 0x800c4298). +0x1f0 = 0, animation
17, jump lock 300.

### The catch and the swap

0x80057950, each player frame: with 0x800c42a8 set and 0x80057518 true
(the box in flight, +0x90 not 0, and within 1400 of 0x800c42a8 in x and
z), 0x800c4420 = the player. The ambience 0x8008046c then calls
0x80057570(player, 0x800c42a8) and clears 0x800c4420. So the catch is
when the box comes near, not when it lands.

0x80057570(player, baby), levels 20-21 only: the two objects swap
contents. The player object (0x800e2b48) takes the baby's first 392
bytes, the rest zeroed (so +0x1a4 health is 0 after a swap); the baby's
object takes the old player:

1. Room lists fixed when the rooms differ.
2. The baby slot (the old player's body): AI (+0x00 |= 0x4000), +0x688
   |= 2, idle (0x80083918), scheduled, its own parts (+0x158), camera
   +0x224 = 0, +0x1f4 = 0, registered by its id (0x800462d4), think
   0x8005d11c, state 0, animations 35 and 17.
3. The player slot: the baby's home (+0x648..+0x654) kept, AI cleared,
   game state +0x08 = the new char, camera +0x224 = 0x800de460
   (0x800de4f4 = the player), registered by id, +0x44 = 0, +0x1f0 = the
   box, the box's draw = 0x800307dc(1), the box into the player's lists,
   the grip (0x8005ff88(player, 0)), a running flight cancelled (+0x6b =
   255), state 20, animations 0 and 20 (not 20 in level 21), then 36;
   +0x23c..+0x244 = the position.
4. 0x800c42a8 = 0; the baby's +0x688 |= 2.

The ids move with the bodies: after Tommy throws to Lil the player
object has id 609 and char 4, and id 1492 is an NPC. Angelica's
follow target is the box, so she goes after the new holder.

Seen: Triangle at 2330 (Tommy, target Lil): state 0, +0x188 5, speech
1167 at 2333; release at 2353 (0x800574cc, 0x800c42a8 = Lil); the box
from (25674, 505, 17671) up to y -2047 and down; at 2402 at (27393, -966,
10295), within 1400 of Lil (27863, 9567): 0x800c4420 set; 0x80057570 at
2403; the player Lil (char 4, id 609) at (27863, 630, 9567) with the box,
state 20 for about 24 VSyncs, health 0, target now Phil; Tommy an NPC.
Then Lil ate one, Triangle at 2502 (speech 1159), release 2509, swap to
Phil (char 5, id 608) at 2552. Cookies unchanged by the throws.

A throw that never gets within 1400 leaves the box on the floor with
empty hands (not seen). GRAB on it picks it up (generic pick-up; from the
code, not checked); see the floor-box loss below.

## Angelica

Mode 4 (follow, 0x80084084 at 0x80084df0) with +0x678 = the box: she
runs to the box. Within her range (2000, +0x68c) with a clear line
(0x8002cd7c) and +0x688 bit 128: mode 8, state 28 (the lunge), +0x688 |=
4. State 28 (0x8005f3a4, animation 43, sound 121) moves her until the
animation ends (+0xe0 bit 0x8000000: mode 0, state 0, +0x688 bit 4
cleared; she goes back to following). While she moves, if she touches
(+0xf4) the player holding the box, or the box while the player holds
it:

1. +0x30 |= 64; the counter text hidden (0x800c4648 +0x78 = 0).
2. Player +0x1f4 = the dummy, state 19, +0x188 |= 4, animation 8: the
   player throws the box, with the bit 64 aim (nearly straight up).

The box's think (0x8004df94) with +0x30 bit 64 ends its flight early
(+0x6b = 255 when the flight record +0x04 is under 6) and on the next
think posts ForceCry(player) and clears bit 64.

A second way to lose (0x80057950 at 0x80057afc): the player not holding
the box, the box not flying (+0x90 = 0), Angelica (600) in state 28:
ForceCry and bit 64 cleared. So a box on the floor is lost as soon as she
lunges (from the code; not seen).

Seen: Angelica reached Tommy standing at the start spot at about 2500
(state 28), the knock at 2514 (bit 64), the throw at 2516, ForceCry
posted by 0x8004df94 (0x8004e080) at 2518.

## The loss (ForceCry, cine 65)

ForceCry (0x8007fd0c), generic part: a held box is put down; tries
counted down (0x800c569b - 1 when not 0). Level 20 is case 19
(0x8007ffd4): no cry animation (s1 = 0), health 255, with the respawn set
(always) the respawn pointers and the callback 0x80079f5c:

0x80079f5c(player): +0x1f0 = 0; speech 0x800b9220[(rand & 255) mod 5]
(535, 536, 578, 579, 580); PostCine(65); the counter text freed,
0x800c4648 = 0.

Tries left: back to the respawn (0x8007fbf8). None: nothing (level 20 is
skipped by the SwitchEpisode at 0x800802b4).

Cine 65: start hook none (0x80072ef0 case 65 does nothing beyond the
generic put-down); end hook (0x800732d4 case 65): with 0 tries
SwitchEpisode(player, 1), then the default. CineFinish (0x800783d8 case
64): card 709 (HINT38 "Now try again.", arg 1) only with tries left.
0x8007737c case 64: 0x800c4410 = 1, 0x80076d6c(20, 0) (only +0x34 bit
20 and music off), then 0x80079f1c: with tries left the round start
0x80079cb0 and CameraSwitch(player, 3); with none nothing.

So a retry gives the box with 15 cookies to whoever is the player, where
cine 65 leaves them. The respawn is not set again.

Seen: first catch: ForceCry at 2520, speech 578, cine 65 from 2520 to
2993 (473 VSyncs), card 709, 0x80076d6c(20, 0), 0x80079cb0, CameraSwitch
at 2993; tries 2 -> 1; Tommy at (28451, 779, 9500) after the cine,
cookies 15. As Lil (after a swap): ForceCry at 2800, the player still
Lil at the respawn, then at (29837, 634, 11506) after the cine with the
box. Second catch (Tommy put beside Angelica at 3400): tries 1 -> 0,
ForceCry at 3411, no 0x8007fbf8, cine 65 to 3884, no card,
SwitchEpisode(player, 1) from 0x800737f0 at 3884, 0x80076d6c(20, 0), the
hub loaded (level 1 at 3900).

## Winning (cine 64)

The fifteenth cookie posts cine 64 (above). Start hook and end hook: the
defaults; no card (0x800783d8 case 63). 0x8007737c case 63: exit code
0x800c4918 = 1, 0x80076d6c(20, 1).

0x80076d6c(20, 1): +0x34 |= bit 20; bars 3 (table 0x800152cc index 7):
0x80094b10(20, 3) sets bits 31, 30, 29 of +0x48[20] and 0x80094ab0 shows
"Great Job! You have earned 3 Reptar bars!" (0x80093b6c); then
0x80094bfc(20): level 20 owns no piece (0x800919b4), so no puzzle screen,
only the 15-bar check (0x80094924); +0x38 |= bit 20 the first time (not
a main level, +0x22 not counted); the hub.

Back in the hub, 0x80078e7c puts the player at 0x800b8718[20] = (5775,
779, 19403).

Seen (cookies poked to 1 at 2320, Circle at 2330): sound 123 and
0x8007a06c at 2333, cine 64 from 2335 to 2677 (342 VSyncs),
0x80076d6c(20, 1), 0x80094b10(20, 3), the bars screen until Cross at
2800, 0x80094bfc(20) and 0x80094924 at 2859, level 1 loaded; after cine 7
and its board (a fresh game) EpisodeStart at 3458 put Tommy at (5775,
779, 19403).

## Camera (0x80077a54, 0x80077c88)

- RoomChange with room flag 4: in levels 20-21 a room's camera kind is
  taken only when it is 3 (0x80077a54 at 0x80077b34). So kind 3 on post 1, set in the
  kitchen, stays when the player walks into rooms of other kinds.
- CameraSwitch (0x80077c88): the usual "same kind as last time" skip on
  0x800c4410 (cine 65 writes 1 so its kind 3 is applied again). In levels
  20-21 the camera call is 0x800291f8(camera, 0, kind, 1500, 3000, 0,
  box, 0): no player, the box as the followed object (guess from the
  arguments: the camera keeps the box in view).

Seen: camera +0x4a 3 from 2306 through every round.

## Per-frame ambience (0x8008046c)

1. A pending catch (0x800c4420): the swap (above).
2. Countdown 0x800c4cc0 minus dt; below 0 speech 0x800b92f8[(rand & 255)
   mod 31] (1148, 1150, 1152, 1170, 1172, 1175, 1180, 294, 307, 323, 330,
   38, 42, 43, 498, 499, 548, 549, 550, 554, 556, 557, 559, 564, 565,
   569, 582, 66, 768, 781, 979), then countdown (rand & 255) x 8.

0x8008052c is the same with table 0x800b9338 but no level uses it
(0x800b9618[21] is 0x8008046c too).

Seen: speech 548 at 2307, 1148 at 2506, 38 at 2646, 569 at 2507.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT07 | 678 | after cine 63 (arg 5) | Angelica got sick eating too many cookies. Eat all the cookies before she can get anymore. Run away from her and press @ to throw the box to the baby with the arrow above his or her head. Press # to eat the cookies while standing still. |
| HINT38 | 709 | after cine 65 with tries left (arg 1) | Now try again. |

@ is Triangle, # Circle (Square eats too). The port's `card_after` has 63
-> 678 and 65 -> 709 (the latter without the tries test).

## Cines

| cine | name | posted by | start hook | end hook | end (0x8007737c) |
|---|---|---|---|---|---|
| 63 | No More Cookies Start | the level start | none | default | 0x80079ee8 (round start, respawn) |
| 64 | No More Cookies Win | the last cookie (0x8007a06c) | none | default | won, 0x80076d6c(20, 1) |
| 65 | No More Cookies Lose | ForceCry's callback (0x80079f5c) | none | SwitchEpisode(1) when tries are 0 | 0x800c4410 = 1, 0x80076d6c(20, 0), 0x80079f1c |

Cine table flags (0x800b8bd0 +4) are 0x200 for all three. Seen lengths
(oracle VSyncs): 63 486, 64 342, 65 473.

## Checked in the oracle

The level 15 probe copied to the scratch folder as l20probe, with a level
20 line (the box, 0x800c42a8, 0x800c4420, ids 600, 608, 609, 1776, 1492).
Presses start 300, cross 500, 650, 800, 1650, 1800; main's level argument
patched (0x80018824 = `li a0, 20`), 0x800c5678 = 20 at the level runner,
cards skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 1
(its own answer for 20). The start cine was kept.

- Start: cine 63 1819-2305; card 678 (arg 5); 0x80079ff0, 0x80079ee8,
  0x80079cb0; CameraSwitch(3); box in hand, cookies 15, tries 2, respawn
  phase 1; Angelica following the box; babies idle; the HUD counter.
- Eating: Circle presses, sound 123, state 19, 15 -> 10; as Lil too.
- Throw: target picked (609), speech 1167 / 1159 from standing, the lob,
  0x800c42a8, the catch within 1400 in flight, 0x80057570 from
  0x8008046c; Tommy -> Lil -> Phil; ids and chars swapped; cookies kept.
- Angelica: the lunge (state 28), the knock (+0x30 bit 64), ForceCry
  posted by the box's think 0x8004df94.
- Loss: cine 65, card 709, the round start again, tries 2 -> 1 -> 0, the
  second loss to the hub through cine 65's end hook.
- Win: cookies poked to 1, one eat, cine 64, 0x80076d6c(20, 1), 3 bars,
  no piece, the hub spot (5775, 779, 19403).
- Rooms and objects read from RAM; screenshots of the kitchen, the
  counter and the arrow.
- Not checked: a natural full eat of 15 (Angelica always arrives first
  without throws); a missed throw and the floor-box loss; picking the box
  up from the floor; Chuckie as a target or as the player; walking out of
  the kitchen; the bars; Pause and Quit Level; what the fixed camera
  does with the box as its subject.

## For the port

The port has the Script trait (frame, cine_finished, cry, draw), Tommy
as the level's character, carrying and aimed throws, things kept in
hand, NPC follow (mode 4) and idle (mode 8), scripted body swaps (levels
6 and 11), ForceCry with tries, cards from level code, the kind 3 fixed
camera, the room camera rule for 20-21 (game.rs `flags & 4`), `start_cine`
20 -> 63, `card_after` 63 -> 678 and 65 -> 709, `ENDINGS` 64 (won) and
the 3-bar award. Level 20 needs, in order:

1. A `level20` script. EpisodeStart 0x80079ff0 (1776, 608, 609 at home,
   song 0) and the round start 0x80079cb0 after cine 63 and after each
   retry: cookies 15, the box in the player's hand, babies idle facing
   the player, Angelica following the box with the lunge allowed, the
   HUD counter, the respawn (first round only).
2. Eating: Circle or Square, standing, holding the box: sound 123,
   animation 42 (state 19, about 40 VSyncs), cookies - 1; at 0 cine 64.
   Square must not be the walk modifier in this level.
3. The baby target and the arrow: each frame the nearest of templates 2,
   6, 7, 11 (not the player's own body) within 512 of the heading; a
   sprite above it. The port's `aim_at` takes any NPC and gives level 9's
   flat throw; level 20 needs the candidate filter and the 45-degree lob
   whose speed comes from the distance (0x80057bc4), Triangle only with a
   target, speech 1159 / 1167 from standing, the player frozen during
   the throw, the target set idle.
4. The runtime body swap (0x80057570): when the box in flight comes
   within 1400 (x, z) of the target, the player takes that baby's body,
   character, id, place and home and gets the box (state 20, animation
   36); the old body becomes an idle NPC with the old id. Any pair of
   Tommy, Chuckie, Phil and Lil, any number of times. Ids must move with
   the bodies (cine 65 and the round start find the babies by id).
5. Angelica: follow a thing (the box), not the player or an NPC (the
   port's `Target` has only Player and Npc(k)); speed row 1 (index 2 ->
   3); within 2000 with a clear line the lunge (animation 43, sound 121,
   until the animation ends); touching the holder: the box goes almost
   straight up and ForceCry follows two VSyncs later; a lunge while the
   box lies on the floor is also a loss.
6. `Script::cry` for 20: no cry animation, health 255, back to the
   respawn while tries remain, empty hands, speech 535 / 536 / 578 / 579
   / 580, cine 65, counter freed. Cine 65's finish: with tries left card
   709 (the port shows 709 always), the round start and camera kind 3
   again; with none the hub (from cine 65's end, not from the cry), with
   the level recorded played (0x80076d6c(20, 0)) and no card.
7. The HUD counter: model 313 and the number at (-160, 65 - (0x800c4a06
   >> 4)), hidden by Angelica's knock.
8. Small things: the ambience speech list (31 lines, every (rand & 255) x
   8 dt); the camera following the box (guess); the box's flight flags
   0x09010700; GRAB never puts the box down; the babies' idle animations
   35 / 17 / 47 and 53; the two bars at the group 0 and 1 spots; the
   level is played once per game; the original's write to address 0x78 at
   the first round needs nothing.

## Unknown

- What 0x80031a28's sprite is exactly (the arrow; colour 208 / 128
  pulse, guess).
- How the kind 3 camera uses the box as its subject.
- The exact lob speed formula's constants (0x80057bc4) and whether the
  pitch limit of the cone matters in the flat kitchen.
- What happens on a miss in practice (the floor-box loss is from the
  code), and whether Angelica can be outrun for 15 eats.
- What state 20 is for the new player after a swap (handler 0x8005cc24 is
  empty) and why the swap gives health 0.
- The kind 3 records (LP 30-32) and the kind 99 record.
- Card args 5 and 1.
- Where cines 63, 64 and 65 leave each baby from a natural approach (the
  oracle's are from teleports or standing play).
