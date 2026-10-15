---
title: Level 11, Circus Angelicus
status: partial
discs: US
covers: US SLUS_006.50:0x8007d8f0, US SLUS_006.50:0x8007d6f8, US SLUS_006.50:0x8007d98c, US SLUS_006.50:0x8007d9c4, US SLUS_006.50:0x8007da2c, US SLUS_006.50:0x8007da6c, US SLUS_006.50:0x8007dab0, US SLUS_006.50:0x8007dadc, US SLUS_006.50:0x8007db34, US SLUS_006.50:0x8007db6c, US SLUS_006.50:0x8007db90, US SLUS_006.50:0x8007dbb4, US SLUS_006.50:0x8007dbf4, US SLUS_006.50:0x8007dc44, US SLUS_006.50:0x8007dc68, US SLUS_006.50:0x8007dccc, US SLUS_006.50:0x8007dd3c, US SLUS_006.50:0x8007df08, US SLUS_006.50:0x8007df2c, US SLUS_006.50:0x8007df50, US SLUS_006.50:0x8007df74, US SLUS_006.50:0x8007df98, US SLUS_006.50:0x8007dfd8, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x800585b0, US SLUS_006.50:0x8005878c, US SLUS_006.50:0x80058860, US SLUS_006.50:0x800588e0, US SLUS_006.50:0x80058998, US SLUS_006.50:0x800589f4, US SLUS_006.50:0x80058a78, US SLUS_006.50:0x80058cb8, US SLUS_006.50:0x80060f1c, US SLUS_006.50:0x80061330, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x8005ed2c, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x8005e1f4, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x8005bbf4, US SLUS_006.50:0x80048a44, US SLUS_006.50:0x80049cc0, US SLUS_006.50:0x8004da50, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8008d3b8, US SLUS_006.50:0x8008cef0, US SLUS_006.50:0x80025b88, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80078dd8, US SLUS_006.50:0x80078df8, US SLUS_006.50:0x800810c8, US SLUS_006.50:0x80018598, US SLUS_006.50:0x80075f58, US /DATA/DB11/DB11LP.BIN, US /DATA/DB11/LEVEL11.BIN, US /DATA/HINTS
worklog:
---

# Level 11, Circus Angelicus

Everything level 11 needs to be played to its end: Angelica's backyard
circus in three rings, each with its own player, its own rules and its own
three tries. Ring 1 is Spike eating 12 bones in 110 s; ring 2 is Phil and
Lil on a tricycle jumping a ramp into a net; ring 3 is a timing meter that
throws Chuckie off a seesaw. Read from the pseudo-code and the
disassembly; the start, a bone, the ring 1 win and timer loss, a ring 2
win and both kinds of ring 2 miss, a ring 3 win, a ring 3 miss, the ring 1
loss and the ring 3 loss at the last try were run in the oracle (see
Checked in the oracle). Builds on [level 10](level-10.md),
[level 9](level-09.md), [level 6](level-06.md), [level 5](level-05.md),
[level 4](level-04.md) and [level 2](level-02.md) (NPC modes, posts, spawn
groups, ForceCry, cines, cards, the player swap), [gameplay](../engine/gameplay.md)
and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c = template +0x10). A VSync is 1/60 s; the
oracle's frame counter counts VSyncs. The player update runs once a game
frame (every second VSync).

## The goal in one paragraph

The player starts as Spike (char 6, 0x80018598 case 11). Cine 52 "Circus
Angelicus R1 Intro", card HINT29 (file 700): guide Spike over the hurdles,
collect 12 bones before the 110 s clock runs out. One bone floats at a
time above a stand; each one eaten moves it to the next of six spots. The
twelfth bone plays cine 54 "R1 Win", and the player becomes the tricycle
(id 508, char 34) with Phil and Lil on it (cine 55, card HINT30, file
701): ride the cone path up the ramp and press Cross on the last red
boards to land in the net (id 7777). The jump length depends on how far
up the ramp Cross was pressed; only the middle band lands in the net.
Landing in it plays cine 57 "R2 Win"; the player becomes Angelica (id
600) on a tower above a seesaw with Chuckie (id 1776) on its far end (cine
58, card HINT31, file 702). Three Triangle presses on a meter set the
launch; cine 39 "Circus Angelica" shows her jump onto the seesaw, the
player becomes Chuckie and flies. Both meter readings in their zones: cine
62 "R3 Win", the level is won. A miss in any ring plays that ring's "Lose"
cine and costs a try (ForceCry); the ring then starts again. No try left:
back to the hub. A ring win sets the tries to 3 only after ring 1.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 from the player init (0x8007d98c), 0 at EpisodeStart, 2 from the ring 2 bar (0x8007db34), 3 at the ring 3 set-up (0x8007dd3c) |
| 0x800c4bd0 | u8 | ring step, the switch in the player update (0x8005a068 at 0x8005a7cc, table 0x80013f84, index step - 1): see below |
| 0x800c4bd4 | u8 | ring 2: landed on the net; ring 3: the meter was good |
| player +0x1a9 | u8 | bones eaten (ring 1) |
| 0x800c4430 | s32 | the level timer, dt; 110 s (33000) in ring 1 (0x80078dd8(110)); 0 stops it (0x80078df8 also frees the clock sprite 0x800c464c) |
| 0x800c42a0 | ptr | the ring 2 speed bar (0x8008d3b8) |
| 0x800e31f8 | s32[3] | ring 2: where the tricycle entered the take-off zone |
| 0x800c4bd8 | u16 | ring 2: distance from that point when Cross was pressed (0x80075f58, capped at 16384) |
| 0x800e31e8, 0x800e31ec, 0x800e31f0 | s32 | the launch of player state 26: ring 2 uses +4 (a speed factor), ring 3 +8 (forward speed); +0 is always 0 |
| 0x800c429c | ptr | the ring 3 meter (0x8008cef0) |
| 0x800c4bd2 | s16 | ring 3 meter, first reading (0..220) |
| 0x800c4bce | s16 | ring 3 meter, second reading |
| 0x800c4bca, 0x800c4bcc | s16 | ring 3: first / second reading running |
| 0x800c4bc8 | u8 | ring 3: first reading on its way down |
| 0x800e5830 | s32[3] | respawn position (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| progress +0x23 (0x800c569b) | u8 | tries; 3 at the level start; set to 3 at the ring 1 win (0x8005aa60) |

Ring steps (0x800c4bd0):

| step | ring | what the player update does |
|---|---|---|
| 8 | 1 | +0x1a9 = 12: 0x8007d9c4 (timer off, PostCine(54)), tries = 3 |
| 1 | 2 | 0x800588e0 (camera behind, the speed bar, phase 2), step 9 |
| 9 | 2 | bar = speed; touch sounds; on the take-off zone: remember the spot, step 2 |
| 2 | 2 | bar = speed; Cross: the jump, step 3; off the zone: falling step 3, else step 9 |
| 3 | 2 | when the player is back in state 0: on the net PostCine(57), else PostCine(56) and step 1 |
| 4 | 3 | 0x80058860, step 5, tries = 3 (nothing sets step 4; guess: unused) |
| 5 | 3 | the meter (0x80058a78); done: meter removed, PostCine(39), step 6 |
| 6 | 3 | 0x80058cb8 (Chuckie's launch), state 26, step 7 |
| 7 | 3 | when the player is back in state 0: good meter PostCine(62), else by tries 3 -> 59, 2 -> 60, else 61 |

It only runs for the player object (0x800e2b48) in level 11. There is no
CheckPoint room, no room cine, no Gameplay script (no kind 6 record has
those bytes) and no room change case (0x8007f498 table 0x80015894 index 8
is the default 0x8007f598).

## Data in DB11

### The characters (LP kind 1)

The loader (0x800454dc) puts the player on LP 56, the first kind 1 record
whose shape is char 6.

| LP | template | char | id | position | what |
|---|---|---|---|---|---|
| 56 | 9 | 6 | 14 | (34588, 667, 38444) | Spike, the ring 1 player; EpisodeStart moves him to (25252, 751, 995), heading 1095 |
| 13 | 814 | 34 | 508 | (10698, 996, 43470) | the tricycle with Phil and Lil (seen; HINT30), the ring 2 player; hidden at EpisodeStart |
| 55 | 1 | 3 | 600 | (23462, -2469, 40485) | Angelica: the ringmaster in ring 1, the ring 3 player on the tower |
| 22 | 2 | 2 | 1776 | (15664, 888, 40434) | Chuckie, on the seesaw; hidden until ring 3 |
| 15 | 11 | 1 | 1492 | (41647, 1352, 50774), post 5 | Tommy, runs round ring 1; hidden from ring 2 on |
| 60, 61 | 6, 7 | 4, 5 | 609, 608 | (5775, 1001, 17355), (5775, 1001, 15307) | Lil and Phil records; no level code (guess: cine actors) |

Char 34 turns slowly like Spike (298, 0x8005bbf4), plays footstep sound
108 on its running animation events (0x8005e1f4 at 0x8005e85c) and has its
own wall case in 0x80068e68 (not traced). Seen top speed (+0x184): 20.

Player init (at 0x800599b8): in level 11 it calls 0x8005878c: step 8, +0x1a9
= 0, 0x800c4bd4 = 0, char 2's animation overrides (0x8008a668(2, ..))
logical 6 -> 432-435, 7 -> 436-439, 8 -> 440-443 (guess: Chuckie's flight
and landing), then 0x8007d98c (respawn here, phase 1).

### Ring 1: the bone, its spots, the stands, the hurdles

One bone: LP 114, template 1017 (kind 14, +0x6d 63, box +-375), spawn
group 2. Kind 7 records (x +0x04, y +0x08, z +0x0c, group +0x10, threshold
+0x11, all 100, post 0):

| group | position | use |
|---|---|---|
| 0, 1 | (24555, 982, 47795), (1874, 982, 33541) | Reptar bars LP 65, 66 (template 835) |
| 2 | (32438, -1402, 456) | bone spot |
| 3 | (39134, -1402, -13724) | bone spot |
| 4 | (27443, -1402, -16195) | bone spot |
| 5 | (14906, -1402, -16237) | bone spot |
| 6 | (7380, -1402, -2922) | bone spot |
| 7 | (20211, -1402, 456) | bone spot |

Under each bone spot stands a template 1211 object (class 16, shape
1620): LP 149 (32373, 884, 598), 151 (38650, 884, -13520), 150 (27377,
884, -15674), 152 (14841, 884, -15715), 129 (7900, 884, -2993), 116
(20145, 884, 598). Class 16 (0x80049cc0): generic init, object +0x00 |=
0x4000000, +0xa4 = 0, +0x6d = template +0x18 (2), armed by distance. Its
top is at y -317 (seen: Spike stood there, +0xf8 = the stand). All six
LP records carry id 202.

The hurdles: templates 501 and 502 (LP 122-127, 131-140; y 1096, at z 3749
and z -19011), template 509 (LP 16, 17, 128, 142-146), flags 0x42
(template flag 64: climbable). Spike jumps them (HINT29).

The ring: 28 posts (LP kind 2, flags 0) in one loop round (23455, -7616),
x 15484..31425, z -12091..-3141; each links to the posts before and after
it. Tommy runs it.

### Ring 2: the path, the ramp, the net

| LP | template | what |
|---|---|---|
| 67-113, 117-121 | 813 | 52 traffic cones (shape 1123, y 1049) lining the path (seen) |
| 153 | 878 | the take-off zone: shape 1235, draw 0, flags 0x1a (template flag 8), at (18427, -733, 34307) |
| 58, 59 | 879 | (18460, 43, 32928), (18460, 43, 35684), beside the ramp (guess: its sides) |
| 14 | 812 | id 7777, the net at (34198, 1041, 34949); hidden at EpisodeStart, shown at the end of cine 54 |
| 30, 32, 34 / 31, 33 | 204 / 380 | +0x6d 15 / 10 (walk-into hazards, guess; not checked) |
| 45, 46 / 47, 48 / 49, 50 | 874 / 875 / 876 | scenery near the path (touch sounds below) |

The ramp rises along +x at z 34307: seen y 935 at x 14400, -1066 at x
22400; the tricycle leaves its top at about x 22700. The zone (template
878, a flag 8 object; player +0x1e8 as for level 10's playpen) starts at
about x 15000 (seen: step 2 at x 15200).

### Ring 3: the tower and the seesaw

| LP | template | id | position | what |
|---|---|---|---|---|
| 40 | 838 | 141 | (18687, 432, 40395) | the seesaw board (shape 1170); 0x8007dc68 sets its angles to (2048, 0, 182) |
| 55 | 1 | 600 | the tower top (23400, -2480, 40500) | Angelica |
| 22 | 2 | 1776 | (15664, 860, 40434) | Chuckie on the board's far end |

### Rooms (LEVEL11.BIN, LP kind 6)

17 rooms; boxes read from RAM (room +0x10..+0x28). Every kind 6 record
sets flag 16; camera rooms have +0x0c 0x94 (flags 4, 16, 128).

| rooms | x | z | camera | song | role |
|---|---|---|---|---|---|
| 1, 2, 3 | -2048..49152 | -23552..8192 | 1 | 1 | ring 1 (Spike starts in 3) |
| 4, 16 | 29696..50176 | 21504..45056 | 3, post 12 | 1 | the net side; the level's first player spot |
| 5, 6, 7, 11, 12 | -4096..30720 | 24576..53248 | 8 | 2 | the ring 2 path; ring 3 (tower in 5, seesaw in 6) |
| 0, 8, 13, 14, 15 | -10240..29696 | 22528..59392 | 8 | - | edges of the path |
| 9, 10 | 4096..19456 | 23552..24576 | - | - | small (y -3072..1024; not traced) |

Post 12 is a ring post without flag 8 (not traced what the kind 3 rooms do
with it). The one kind 99 record (0x63, 0x1c, 8) has no loader case.

### Level 11 in shared code

- 0x800514b0, kind 14 (and 4, 15) in level 11 (0x80051a40): sound 29 (31
  for kind 15), +0x1a9 + 1; below 12 the bone's group +0xbc becomes +0xbc
  + 1, or 2 after 7, and 0x8004fe14(&bone, group, 1) places it there; +0xab
  &= ~2. At 12 it stays gone. No healing, no HUD count. Its effect handle
  (+0xc0) is kept while +0x1a9 is below 12 (0x80051cd4).
- 0x8005ed2c (touch while in the air), level 11, phase below 2, kind 14:
  the touch is skipped when 0x800c4bc4 bit 1 is set, that is after a
  standing jump (0x80060f1c case 0 sets it). A standing jump never eats a
  bone.
- The stand and walk handlers (0x8005d594, 0x8005d6a0): moving into a
  touched class 11 object calls its +0x58. Seen: the bone was eaten from
  the stand top this way (0x800514b0 from 0x8005d5cc).
- The jump launch (0x80060f1c): the moving jump (case 25) and state 26
  (case 35) give char 6 vy -1400, -1700 in level 11 (others -1000); the
  horizontal factor when fast is 40, 50 for char 6, 20 for char 6 in level
  11 (12 when slow). The standing jump (case 0) is vy -1000 for all.
- The stand handler (0x8005d11c, just before 0x8005d54c) and the walk
  handler (0x8005e1f4, just before 0x8005e7c0): in level 11, for the player
  at step 2, Cross does not jump (the ring 2 take-off uses it).
- 0x8005b14c: in level 11 at steps 5 and 6, any template 2 object running
  0x8005a068 is held at (15664, 860, 40434).
- 0x80061330 (the air): at step 7 the speed drag is a quarter of the usual
  (just before 0x800617dc: << 3 instead of << 5).
- CameraSwitch (0x80077c88 at 0x80077e54): in level 11 at phase 2 room
  cameras are ignored.

## Start (EpisodeStart, 0x8007d8f0)

Installed by 0x8007871c (case 11). The level start, in order: player init
on LP 56 (0x8005878c); cine 52 (0x80074958, no hooks in 0x80072ef0 /
0x800732d4); CineFinish (0x800783d8 case 51): card 700 (HINT29, arg 6);
EpisodeStart; 0x8007737c case 51 does nothing.

0x8007d8f0(player):

1. Hide 1776, 508, 7777 (0x80078bc4).
2. 600: health and max 255, 0x80083918 (mode 8, idle), +0x688 |= 64
   (turns to watch the player), scheduled. Seen at (31294, -471, 30389),
   room 4.
3. 0x8007d6f8 (ring 1 set-up, below).
4. Phase 0.

### Ring 1 set-up (0x8007d6f8)

1. 1492: health and max 255, placed at (26113, 833, -4384) (0x800825b0),
   speed index 2, 0x80083818(npc, +0x698 = 5): mode 6 on the posts, target
   the player; scheduled. Speed table row 0: 8 (seen).
2. 14 (Spike) at (25252, 751, 995), health 255, heading 1095.
3. Ids 200-203: +0xa4 = 0 (only 202 exists here; guess: left over).
4. Timer 110 s (0x80078dd8(110)).
5. The kind 14 collectible (from the list 0x800e2710 with its flag at
   0x800e2740 set and +0xbc in 2..7): player +0x1a9 = 0, out of its rooms
   (0x8004fd20), +0xbc = 2, placed in group 2, +0xab &= ~2.
6. Camera reset (0x800290a4).

Seen: cine 52 from 1819 to 2236; at 2236 card 700, EpisodeStart, timer
33000, the bone at (32438, -1402, 456); Spike in room 3, tries 3.

## Ring 1: Spike and the bones

Spike climbs onto a stand (a moving jump onto it; the stand top is at y
-317) and jumps or walks into the bone above it (y -1402). Each bone moves
the next one to the next spot in group order 2, 3, 4, 5, 6, 7, 2, ...
(0x800514b0). Tommy runs round the post loop and stops when Spike is in
his way (seen: mode 6, state 0, speed 0 while blocked); he does no damage
(no Damage call seen). Angelica watches from room 4.

At 12 bones (step 8 in the player update): 0x8007d9c4: timer off
(0x80078df8), PostCine(54), tries = 3.

The clock: 0x8007f5a8 counts down the level timer and draws it; at 0:
sound 117 at the player and PostTimerExpired with level 11's callback
0x8007dc44 (table 0x800158cc index 9): PostCine(53) "R1 Lose".

- Cine 53's end (0x8007737c case 52, 0x8007da2c): ForceCry; tries left:
  0x8007d6f8 again (timer 110 s, the bone back to group 2, count 0,
  Spike, Tommy and the camera back).
- Cine 54's end (case 53): 0x8007da6c (show 7777, PostCine(55)); the
  player swaps with 508 (0x800585b0); hide 14; camera kind 1 follow 200
  (0x800291f8).
- Cine 55's end (case 54, 0x8007dadc): hide 1492; 508 health and max 255,
  shown; 0x8007dab0 (song 1, step 1). CineFinish case 54: card 701
  (HINT30, arg 7).

Seen: a bone eaten from the stand at 2423, the bone to group 3 (0x8004fe14
from 0x80051a84); +0x1a9 poked to 11, the 12th bone at 2451; 0x8007d9c4
at 2453; cine 54 2455-2599; the swap at 2599; cine 55 2599-3146; card 701,
step 1, 0x800588e0, the bar, phase 2 at 3146.

## Ring 2: the tricycle

### The run-up (steps 1, 9, 2)

0x800588e0 (step 1): camera behind the player (0x80026ddc(camera, player,
0)); the bar, if none: 0x8008d3b8(356, scale (64, 8, 1024), pos (170, 85),
355, scale (64, 32, 1024), pos (170, 76)) in 0x800c42a0 (bottom right,
seen: a purple half circle); 0x8007db34 (phase 2 and respawn here, once).
Step 9.

Step 9 (at 0x8005a850): the bar shows the speed (0x8008d500(bar,
+0x184)). Touch sounds, once per touch (0x800c42c0 re-arms with nothing
touched): template 813 sound 103, 204 sound 104, 874 or 875 sound 106.
Player +0x1e8 template 878: 0x800e31f8 = the player's position, step 2.

Step 2 (0x8005a924): the bar as above; 0x80058998: 0x800c4bd8 =
distance(player, 0x800e31f8), returns Cross pressed (0x800c4994).

- Cross: +0x188 |= 512, bar removed (0x8008d514), 0x800589f4, step 3,
  player state 26.
- No Cross, still on 878: stay.
- Off 878 and in state 4 (driven off the ramp's end): bar removed, step 3.
- Off 878 otherwise: step 9.

0x800589f4: heading 1050; 0x800e31e8 = 0 and by 0x800c4bd8: 1..5999 ->
0x800e31ec = 4; 6001..7499 -> 10; 7501 and more -> 16 (+8 = 0); exactly
0, 6000 or 7500 leave it as it was.

### The jump (state 26, 0x80060f1c(player, 38))

Velocity: y -1000 (char 34); along the facing factor x speed x
0x800e31ec, factor 40 when fast (speed above 0x800c4204), else 12. So
the later Cross is pressed on the ramp, the longer the jump.

Seen (speed 20, Cross with up held): at distance 5360 (x 4) the tricycle
landed at x 26977 (short); at 7009 (x 10) it landed on the net at (32671,
87, 34407) (+0xf8 = 7777); at 8040 (x 16) it landed at x 38690 (long).
Cross with the stick released gives speed 0: straight up, back onto the
ramp.

### The landing (step 3)

When the player is in state 0 again (+0x188 &= ~512): standing on
template 812 (+0xf8, the net): 0x800c4bd4 = 1, PostCine(57) "R2 Win"
(step stays 3); else 0x800c4bd4 = 0, PostCine(56) "R2 Lose", step 1.

- Cine 56's end (case 55, 0x8007dbb4): ForceCry; tries left: 0x8007dab0
  (song 1, step 1). The respawn is where cine 55 left the tricycle:
  (10551, 950, 44176), heading 1343 (seen).
- Cine 57's end (case 56, 0x8007dbf4): the player swaps with 600
  (Angelica); hide 508; PostCine(58).
- Cine 58's end (case 57, 0x8007dccc): 1776 health and max 255, shown,
  mode 8, scheduled; 0x8007dc68 (ring 3 set-up); song 0. CineFinish case
  57: card 702 (HINT31, arg 8).

The tries are not reset after ring 2: ring 3 starts with what ring 2
left.

## Ring 3: the meter and the seesaw

### Set-up (0x8007dc68)

1. Id 141 (the board): angles (2048, 0, 182), its matrix (0x80020ee0).
2. 0x80058860: meter state cleared; the player swaps with 600 (already
   the player after ring 2: it swaps with itself; after a miss the player
   is Chuckie and becomes Angelica again); hide 508; heading 3072; +0x188
   |= 1 (frozen); field of view 512 (0x8001dfb8); 0x8007dd3c.
3. Step 5.

0x8007dd3c: the player at (23400, -2480, 40500), out of its rooms; 1776
health 255 at (15664, 860, 40434), angles (3072, 0, 0); the fixed camera
0x800291f8(camera, player, 4, 1500, ..): kind 4 (0x80025b88) with the eye
at (27221, -3810, 39229) and the matrix from angles (3146, -203, 0) in
0x800e577c; 0x800e5790 = Chuckie + (0, -1000, 0); phase 3 and respawn
here (once); song 0.

### The meter (step 5, 0x80058a78)

The bar, if none: 0x8008cef0(358, scale (1, 8, 512), pos (127, 76), 357,
scale (110, 12, 512), pos (0, 76)) in 0x800c429c (seen: a purple bar
bottom centre with two small marks). Each game frame:

1. First reading running: up 6 a frame to 220, then down 6 a frame; at 0
   everything resets (the meter idles).
2. Second reading running: up 6 a frame; at 220 everything resets.
3. Triangle pressed (0x800c49d4 & 3 == 1):
   - nothing running: sound 124, the first reading starts (6);
   - first running, second not: sound 125, the first stops, the second
     starts at 220 - first;
   - second running: sound 125, done.
4. The bar shows the first reading, or 220 - second.

Done: meter removed (0x8008d06c), PostCine(39) "Circus Angelica", step 6.
Cine 39 (no hooks; CineFinish case 38 sets a callback 0x8007dfd8 that does
nothing) shows Angelica jumping onto the board. Its end (0x8007737c case
38): the player swaps with 1776: Chuckie is now the player.

### The launch (step 6, 0x80058cb8)

- Heading: second reading 1..171 -> 2944; 172..195 -> 3072; 196 and more
  -> 3200.
- Forward speed (0x800e31f0): first reading 1..170 -> 2176; 171..194 ->
  2816; 195 and more -> 3328.
- Good (0x800c4bd4 = 1) only with 2816 and 3072.

Then state 26 (+0x188 |= 512), step 7. At step 7 the launch is (0,
2 x -1000, 0x800e31f0) turned by the heading, with no speed term
(0x80060f1c after 0x80061158).

### The landing (step 7)

When the player is in state 0 again: good: PostCine(62) "R3 Win"; else by
tries: 3 -> PostCine(59), 2 -> 60, else 61. The landing spot is not
tested. Cines 59-61 end in 0x8007df98 (0x8007737c cases 58-60): ForceCry;
tries left: 0x8007dc68 again.

Seen: presses at 5300, 5360, 5410: readings 186 and 178 (detected 3
VSyncs after each press), good; cine 39 5415-5457, swap, Chuckie flew
heading 3072, forward 2816 and landed at (4056, 699, 40420); PostCine(62)
at 5542. With readings 186 / 166: heading 2944, cine 59. With 96 / 184:
forward 2176, cine 59 (tries 3) or 60 (tries 2).

## Winning: cine 62

0x8007737c case 61: exit code 0x800c4918 = 1, 0x80076d6c(11, 1) (level 11
played and won, its puzzle piece; see [objectives](../engine/objectives.md)). Cine
62 has no hooks and no card. The port's `ENDINGS` already has 62. Seen:
cine 62 5544-6464, 0x80076d6c(11, 1), 0x80094bfc(11), the puzzle screen
0x8009480c(11, 1).

## ForceCry (0x8007fd0c, level 11 case)

Level 11 is case 10 (0x80080044): no cry animation (0x8007f920 not
called); health 255; tries counted down; tries left: to the respawn
(0x8007fbf8); none: SwitchEpisode(player, 1) (0x800781f8, the hub). No
level callback. The ring's cine end then sets the ring up again (above),
which moves the player anyway.

Respawns: ring 1 the player-init spot (34588, 720, 38444), at once
replaced by 0x8007d6f8; ring 2 where cine 55 left the tricycle; ring 3 the
tower top.

Nothing in rings 1 and 3 hurts the player (seen health 255 throughout).
The ring 2 hazards (templates 204, 380) were not tested.

## How the level ends

- Won: the ring 3 meter good, Chuckie lands, cine 62, 0x80076d6c(11, 1),
  exit code 1.
- Lost: the last try in any ring: that ring's Lose cine, ForceCry,
  SwitchEpisode(1). Ring 1 runs out by the clock; rings 2 and 3 by misses.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT29 | 700 | after cine 52 (arg 6) | Ring One! Guide Spike over hurdles by pressing %. Collect 12 bones before time runs out to make Angelica happy enough to allow you to move on to Ring two. |
| HINT30 | 701 | after cine 55 (arg 7) | Ring Two! Phil and Lil race their tricycle around the path and up the ramp pressing % to jump from the last red board to land safely in the net. If Angelica doesn't think you were close enough to the center then you'll have to try again. |
| HINT31 | 702 | after cine 58 (arg 8) | Ring Three! Chuckie has unwillingly become the star of the center ring. Press @ to start the jump meter. Press @ a second time when it reaches the smaller left line. Press @ a third time when the meter moves back down to the smaller right line. |

The port's card table (front.rs `card_after`) has 52 -> 700, 55 -> 701
and 58 -> 702.

## Cines

| cine | name | posted by | end (0x8007737c) |
|---|---|---|---|
| 52 | Circus Angelicus R1 Intro | the level start | nothing |
| 53 | R1 Lose | TimerExpired 0x8007dc44 | 0x8007da2c |
| 54 | R1 Win | 0x8007d9c4 | 0x8007da6c, swap to 508 |
| 55 | R2 Intro | 0x8007da6c | 0x8007dadc |
| 56 | R2 Lose | 0x8007db90 | 0x8007dbb4 |
| 57 | R2 Win | 0x8007db6c | 0x8007dbf4, swap to 600 |
| 58 | R3 Intro | 0x8007dbf4 | 0x8007dccc |
| 39 | Circus Angelica | step 5 done | swap to 1776 |
| 59, 60, 61 | R3 Lose1-3 | 0x8007df2c, 0x8007df50, 0x8007df74 | 0x8007df98 |
| 62 | R3 Win | 0x8007df08 | won |

No start or end hooks (0x80072ef0, 0x800732d4) for any of them. Seen
lengths (oracle VSyncs): 52 417, 53 456, 54 144, 55 547, 56 182, 57 194,
58 1680, 39 42, 59 209, 60 209, 61 649, 62 920.

## Per-frame ambience (0x800810c8)

0x800b9618[11]. The countdown 0x800c4cc0 minus dt; below 0: at step 9
speech 0x800b94a0[(rand & 255) mod 3] (743, 766, 804); at step 5
0x800b94a8[(rand & 255) mod 7] (395, 474, 533, 605, 606, 672, 677); then
countdown (rand & 255) x 8. Other steps: nothing (the countdown stays
below 0, so the first line comes at once). Not checked in the oracle.

## Other notes

- The player swap (0x800585b0(player, other)): the two 1644-byte objects
  trade places, so the player stays at 0x800e2b48; ids re-registered
  (0x800462d4); the old player becomes a character (+0x00 |= 0x4000, mode
  8 idle, scheduled, +0x4c = 0x800820ac, stand state); the new
  player gets the camera (+0x224, 0x80029078), 0x800c5680 = its char,
  +0x1f0 = 0, its body set (0x8008a42c), state 0. Seen camera kind after
  the ring 3 swap to Chuckie: 1 (follow).
- 0x8004da50 (+0xa4 steps of ids 200-203 by a collectible's +0xbc, model
  template shape + step) has no caller and no table entry found.
- 0x8007737c(n, 1) also runs about 2 VSyncs after each PostCine (seen),
  with no level 11 effect.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, pad presses, hooks with
arguments, object hooks, room dumps, screenshots): presses start 300,
cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 11`), 0x800c5678 = 11 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 6 (Spike).
The start cine was not skipped.

- Start: player init and 0x8005878c at 1817; cine 52 1819-2236; card 700
  (arg 6), EpisodeStart, 0x8007d6f8, timer 33000, phase 0, tries 3; Tommy
  mode 6 speed 8 on posts 5 -> 4 -> ..; Angelica mode 8, +0x688 0x40;
  508, 1776, 7777 in room -1.
- A standing jump at the bone's spot from the stand: no touch. Moving off
  the stand top into the bone: 0x800514b0 from 0x8005d5cc at 2423, +0x1a9
  1, the bone to group 3. +0x1a9 poked to 11, the 12th bone at 2451: the
  ring 1 win as listed.
- Timer poked to 600: TimerExpired at 2421, cine 53 2421-2877, ForceCry
  without the cry, tries 2, 0x8007d6f8 again. With tries poked to 1:
  SwitchEpisode(player, 1) at 2877, level 1 loaded.
- Tommy blocked by Spike standing on his path: stopped in state 0; no
  Damage.
- Ring 2: the tricycle set at (12000, 700, 34307) heading 1024, R1, then
  up held: the ramp and the zone as listed. Cross at three distances: the
  three landings listed; the short and long ones cine 56, ForceCry, tries
  2, respawn (10551, 950, 44176) heading 1343, step 9. No Cross: off the
  ramp's end, step 3, cine 56.
- Ring 2 win at 3389: cine 57 3391-3585, swap to 600, cine 58 3585-5265,
  card 702 (arg 8), the ring 3 set-up, camera kind 4, phase 3, step 5.
- Ring 3 meter readings and outcomes as listed; the win as listed. Tries
  poked to 2: cine 60, ForceCry, tries 1, 0x8007dc68 (swap back to
  Angelica, step 5); the next miss: cine 61 5904-6553, ForceCry, tries 0,
  SwitchEpisode(player, 1) at 6553.
- Rooms and objects read from RAM; screenshots of the three rings.
- Not checked: twelve natural bones and hurdle jumps (the count was
  poked); the bone spots 4-7; the ring 2 hazards and touch sounds; the
  ambience; the kind 3 rooms 4 and 16; a ring 2 fall (state 4) inside the
  zone; the reset of the meter at 220; char 34's wall case.

## For the port

The port has a Script per level, the playable character by level (Spike
for 11 already), climbing, hiding objects, cinematic-object use, doors,
room flags, the follow camera and kind 3 fixed cameras, NPC modes 0, 1, 3,
4, 6, 7, 9, 11 and riders, posts, carrying and throwing, hazards,
healing collectibles, collect events, spawn groups, movers, COL floor
flags, floating, mid-level player swaps, the level timer, gauges and the
meter, ForceCry with tries, cards from level code and the result screens;
level 10 adds NPC use callbacks, NPC spawn spots and speed row 1. Level 11
needs none of level 9's or level 10's additions. It needs, in order:

1. A `level11` script: the ring step (0x800c4bd0) and phase, tries,
   respawn. EpisodeStart 0x8007d8f0 (hide 1776, 508, 7777; Angelica idle
   watching; the ring 1 set-up 0x8007d6f8; phase 0). The cine ends of
   52-62 and 39 (table above), TimerExpired = PostCine(53), and the ring
   step machine run in the player update. Cards are in `card_after`, 62
   in `ENDINGS`.
2. Player swaps between four bodies with their own characters: Spike ->
   tricycle (char 34, template 814) after cine 54, -> Angelica after cine
   57, -> Chuckie after cine 39, -> Angelica again on a ring 3 miss. The
   old player stays as an idle character where it was; the camera follows
   the new one. Level 6's swap is one fixed pair; this needs any pair, and
   char 34's body, slow turn (298) and footstep sound 108.
3. Ring 1: the bone as a kind 14 collectible that in level 11 counts
   (+0x1a9) instead of healing and moves itself to the next group (2..7,
   wrapping) until 12; no touch after a standing jump; the touch when
   moving into it from a stand (the move test's touched object). The
   stands (class 16, solid tops at y -317). Spike's level 11 jump: moving
   jump vy -1700 with factor 20 (the port always uses -1000 and 40 / 12;
   Spike elsewhere is -1400 and 50). Tommy on the post loop (mode 6
   without a goal post, speed 8), stopping when blocked. The 110 s clock
   (timer exists).
4. Ring 2: the speed bar HUD (models 356 / 355, value = speed); the
   take-off zone (template 878 as a flag 8 volume under the player, like
   level 10's playpen); the distance from the zone entry to the Cross
   press choosing x 4 / 10 / 16; Cross not jumping while in the zone;
   player state 26 (a jump with the factor, heading 1050); landing on the
   net (template 812 under the player) or not; the respawn (10551, 950,
   44176), heading 1343; room cameras off in phase 2.
5. Ring 3: the timing meter (models 358 / 357, Triangle three times, 6 a
   game frame, 220 limit, sounds 124 / 125) and its two readings; cine 39
   then the swap to Chuckie; the launch (0, -2000, 2176 / 2816 / 3328)
   turned by 2944 / 3072 / 3200; the quarter drag in the air at step 7;
   the win test on the readings only; Chuckie held at his spot during
   steps 5 and 6; the fixed camera kind 4 (eye (27221, -3810, 39229)) and
   field of view 512; the board's tilt (id 141).
6. `Script::cry` for level 11: no cry animation, health 255, tries - 1,
   the respawn or the hub; then the ring's cine end resets the ring.
7. Small things: the ambience lines by step (0x800810c8); the ring 2
   touch sounds (103, 104, 106); Chuckie's animation overrides (logical 6,
   7, 8 -> 432-443); the ring 2 hazards (templates 204, 380) if they
   matter.

## Unknown

- What the take-off zone's extent is exactly (model 1235; seen entering
  at about x 15000 on the ramp).
- Why the distance bands skip exactly 6000 and 7500 (left unchanged).
- What the tricycle's top speed is in the real game and the speed bar's
  scale (seen 20).
- Whether the ring 2 hazards (204, 380) hurt the tricycle and what a cry
  in ring 2 by damage would do to the step.
- What the kind 3 rooms 4 and 16 do with post 12 (no flag 8).
- What Chuckie's logical animations 7 and 8 are; what ids 200, 201, 203
  were and what 0x8004da50 was for.
- What char 34's wall case (0x80068e68) does.
- What rooms 9 and 10 are.
- Where the camera kind 4's angles (3146, -203, 0) point exactly and what
  0x800e5790 (Chuckie + (0, -1000, 0)) is used for.
- Card args 6, 7, 8.
- How long the cines really are (the oracle's lengths above).
- Whether Lil and Phil's records (LP 60, 61) and the template 1211 id 202
  matter outside the cines.
