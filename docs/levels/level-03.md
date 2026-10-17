---
title: Level 3, Incident on Aisle 7
status: partial
discs: US
covers: US SLUS_006.50:0x800793c8, US SLUS_006.50:0x8007bc60, US SLUS_006.50:0x8007e3c0, US SLUS_006.50:0x8007e308, US SLUS_006.50:0x8007ece4, US SLUS_006.50:0x8007dfe0, US SLUS_006.50:0x80077ee4, US SLUS_006.50:0x80077070, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80077864, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80084084, US SLUS_006.50:0x8008395c, US SLUS_006.50:0x800839f8, US SLUS_006.50:0x80082c78, US SLUS_006.50:0x80082ee0, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x8007ebd4, US SLUS_006.50:0x8004997c, US SLUS_006.50:0x8004eb10, US SLUS_006.50:0x8005282c, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x80061c98, US SLUS_006.50:0x8005fe44, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004f83c, US SLUS_006.50:0x80080330, US SLUS_006.50:0x80063d3c, US SLUS_006.50:0x8006eac4, US /DATA/DB03/DB03LP.BIN, US /DATA/DB03/LEVEL03.BIN, US /DATA/HINTS
worklog:
---

# Level 3, Incident on Aisle 7

Everything level 3 needs to be played to its end: the walk to the
bakery, the pie toss against Larry and Steve, the lobster aisles, the
boss lobster and the switch. Read from the pseudo-code and the
disassembly; the phase changes, the pie toss, both of its outcomes, the
lobster bite, the boss's swat and the win were run in the oracle (see
Checked in the oracle). Builds on [level 2](level-02.md) (NPC modes,
posts, events, damage, ForceCry, cines, cards), [gameplay](../engine/gameplay.md)
and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). "char" is the character (+0x15c =
template +0x10). Heading 0 faces +z, 2048 faces -z (0x80075ee8 turns
(0, 0, 512) to (0, 0, -512) at 2048).

## The goal in one paragraph

Tommy is in the grocery store after the Reptar cereal. Part 1: walk to
the back of the store (card HINT08). A balloon (id 81, use plays cine 8
"Grocery Store Balloon") or room 19 (cine 9 "Grocery Store Cart Ride")
brings him to the bakery. Part 2, the pie toss (0x8007bc60): Tommy stands
on a counter, frozen, and throws pies (Triangle, turning to aim) at Larry
(id 75) and Steve (id 76), who walk rings of posts that close in on him.
They share 200 health; four pie hits (50 each) win and play cine 13
"Mellon Rush". If either reaches post 39 first, cine 15 "Larry and Steve
Win" plays and Tommy cries back to the start. Part 3 (phase 2): the
lobster aisles; lobsters (char 23) chase and bite (15). Cans (template
777) thrown at them kill them. Room 22 or 36 start phase 3; room 36 plays
cine 86 "Boss Lobster". The blue boss lobster (id 601) guards the end of
the stock room and swats Tommy back; a can stuns it. Tommy climbs the can
stacks to the switch (LP 278) and uses it: cine 14 "Episode 2 Ending",
the level is won. The words come from HINT08, HINT10-HINT12 (files 679,
681-683).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 store and pie toss, 2 lobster aisles, 3 boss area |
| 0x800e5830 | s32[3] | respawn position for ForceCry (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| progress +0x30 bit 8 | | the pie toss is on (0x8007bc60 sets, 0x8007e3c0 clears) |
| progress +0x30 bit 128 | | set at cine 15's start (0x80072ef0 case 15), cleared at its end (0x800732d4); while set no character shadow is drawn (0x8003205c) |
| 0x800c4cd4 | ptr | the pie Tommy holds: pie 0, 0x800e2770[0] (0x8007bc60) |
| 0x800c440d | u8 | 1 after cine 8, 0 after cine 9 (0x800783d8); nothing reads it |
| 0x800c4ba0 | s32 | 1000 at EpisodeStart; a shared countdown for object sound effects (0x80050140, 0x80053604) |

The phase is only written through 0x80078d60: 1 by EpisodeStart, 2 by
0x8007ece4, 3 by Gameplay script 1 (0x8007dfe0). Level 3 has no
CheckPoint rooms (no kind 6 record has byte +4 set), so progress +0x24
and +0x30 bit 2 are never set here.

## Data in DB03

### Characters (LP kind 1, hierarchy templates)

Built by 0x800497dc and 0x80081bac. Template values from TEMPLATE.BIN.

| LP | template | char | id | position | health (+0x18) | range +0x68c | wait +0x68e | speed index | +0x6d | first post (LP +0x40) | loop reload (LP +0x42) | what |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 84 | 11 | 1 | 1492 | (-797, -484, -2808) | 5 | 2000 | 0 | 4 | 50 | | | Tommy |
| 153 | 590 | 10 | 75 | (39917, -265, 1236) | 200 | 2000 | 200 | 5 | 0 | 12 | 4 | Larry (from HINT10, guess which is which) |
| 154 | 591 | 11 | 76 | (51413, -263, 1789) | 200 | 2000 | 200 | 5 | 0 | 7 | 4 | Steve |
| 237 | 495 | 23 | 83 | (66849, 354, -4996) | 50 | 4500 | 0 | 1 | 20 (15 set at init) | 65 | | lobster |
| 238 | 495 | 23 | 84 | (62587, 349, 2574) | 50 | 4500 | 0 | 1 | 15 | 67 | | lobster |
| 240 | 763 | 23 | 85 | (95529, 349, 7669) | 50 | 4500 | 0 | 1 | 15 | 99 | | lobster |
| 239 | 764 | 23 | 86 | (70233, 349, -21384) | 50 | 4500 | 0 | 1 | 15 | 45 | | lobster |
| 241 | 495 | 23 | 2007 | (87604, 349, 12554) | 50 | 4500 | 0 | 1 | 15 | 75 | | lobster |
| 242-244 | 495 | 23 | 2008-2010 | (117808, 349, 14117), (112488, 349, 17146), (113602, 349, 19930) | 50 | 4500 | 0 | 1 | 15 | 105, 121, 115 | | the boss's "bullies" (HINT12) |
| 245 | 495 | 23 | 2011 | (69809, 349, -13043) | 50 | 4500 | 0 | 1 | 15 | 52 | | lobster |
| 246 | 711 | 33 | 601 | (114090, -261, 22169) | 5 | 2000 | 0 | 2 | 85 | | | the blue boss lobster |
| 83 | 5 | 7 | 13 | (-2876, -251, -2809) | 5 | 2000 | 0 | 2 | 0 | | | hidden after cine 2 (0x800732d4); not used by level code |

Templates 590, 591, 495, 763, 764 and 711 have touch +0x0a = 9
(0x800511a8) and flags 0x2001 (the 0x2000 bit: unknown). The player's
health is 255 (0x80059a88), not the template's.

0x80081bac by char:

- 10, 11 (Larry, Steve): 0x80083818(npc, +0x698), mode 6 on its posts,
  +0x688 bit 1 off. Not scheduled: no enter callback.
- 23 (lobsters): 0x8008395c(npc, +0x698, player), mode 9; enter callback
  +0x48 = 0x80049e18; +0x6d = 15; +0x688 |= 0x4c08.
- 33 (boss): 0x800839f8(npc, player), mode 3 with saved heading +0x654 =
  its heading; +0x48 = 0x80049e18; touch +0x58 = 0x800511a8; +0x688 |=
  0x1c04.
- 7: nothing.

The LP loader adds every object with an enter callback to the trigger
list 0x800c4c24 (0x800454dc), so the lobsters and the boss are armed by
distance (template +0x34 = 15000) from the start, not by level code.

+0x688 bit 0x800 is read after all: an AI object with it skips the
object collision pass (0x80063d3c). Larry, Steve (0x1800) and the
lobsters (0x4c08) have it.

### Other objects that matter

| LP | template | class | id | position | what |
|---|---|---|---|---|---|
| 111 | 564 | 9 | 1111 | (45615, -2168, 18846) | the pie: class 9 makes six pies (below); model 885, pickable, +0x6d 50, +0x0b = 1 (flight end 0x8005282c) |
| 248 | 567 | 0 | 81 | (29989, -812, 14633), room 25 | LP +0x44 = 8: use plays cine 8 (the balloon, guess from the cine name); 0x8007e3c0 clears its use on a win |
| 128 | 598 | 0 | 79 | (45838, 173, -5322), room 19 | hidden after cine 9 (guess: the cart) |
| 278 | 873 | 0 | | (107630, -3516, 20608), room 35 | the switch: LP +0x44 = 16, which 0x8004691c turns into +0x6a = 14 with use 0x8004f83c |
| 283 | 1198 | 0 | 333 | (113686, -2586, 5630), room 36 | hidden at the start; shown after cine 86 (0x800732d4); hidden again by 0x8007ece4 in phase 3 |
| 100 | 769 | 0 | 74 | (56714, 0, -4111) | hidden after cine 13 |
| 108, 109 | 489 | 0 | 1931, 1932 | (42994, -296, 2506), (48126, -296, 2517) | template flag 8 on during the pie toss |
| 118, 119 | 566 | 0 | 1934, 1933 | (40947, -29, 6672), (50206, -29, 6611) | same |
| 253, 254 | 488 | 0 | 1935, 1936 | (47258, -109, 10254), (43805, -109, 10254) | same |
| 255-277 (20) | 777 | 0 | | y 885, the lobster aisles and room 26 | unopened cans: pickable, +0x6d 50 (a lobster's whole health); thrown in level 3: sound 53 (0x80061f88) |
| 204-206, 207-210 | 651, 650 | 0 | | (104222..121349, 811..881, -20495..-28159) | pickable, +0x6d 8 (guess: the opened cans of HINT11) |
| 286-292 | 835 | 12 | | (60470..62672, 0, 10708..13141) | Reptar bars (kind 3), optional |
| 285 | 863 | 12 | | (63793, 0, 12720) | cookies (kind 4), optional |

Template flag 8 is skipped by the object collision tests (0x80063d3c,
0x80065210, 0x8006592c, 0x8006a824, 0x8006baa8, 0x8006c334). Setting it
on templates 489, 566, 488 during the pie toss (0x8007bc60) and clearing
it after (0x8007e3c0) makes those props non-solid (guess: so the pies
roll past them).

Ids 61-71, 77 and 78, hidden by cine 13's end, are not in DB03
(0x80078bc4 prints a warning and goes on).

Hazards: walking into an object that is not pickable, not AI, with +0x6d
> 0 is Damage(player, 0, +0x6d) (walk state, 0x8005e1f4). In DB03:
templates 197 and 575 (10), 526 and 738 (4), 611 (12), 160, 159, 595,
602 (class 8, 15).

### Rooms (LEVEL03.BIN, LP kind 6)

Kind 6 record bytes from +4: room, cine (flag 2, +0x5b), camera (flag 4,
+0x5d), +0x5c, checkpoint (flag 32), flag 16, Gameplay script (flag 64,
+0x5f), song (flag 128, +0x64) (0x800454dc).

| room | x | z | kind 6 | role |
|---|---|---|---|---|
| 3 | 1024..20480 | -19456..-8192 | song 1 | the start: (6671, 779, -14463) |
| 19 | 34816..48128 | -8192..-2048 | cine 9, camera 8 | the cart ride |
| 25 | 20480..32768 | 12288..20480 | | the balloon (id 81) |
| 9, 18 | 34816..56320 | -2048..24576 | camera 9, song 2 | the bakery: Larry and Steve's rings, the pie spot (room 18) |
| 22 | 111616..115712 | 1024..5120 | camera 6, script 1 | phase 3 |
| 36 | 111616..115712 | 5120..6144 | cine 86, camera 3 (+0x5c 122), script 1 | Boss Lobster |
| 26 | 107520..118784 | 6144..13312 | camera 3 (+0x5c 122) | the stock room, cans |
| 35 | 107520..118784 | 13312..30720 | camera 3 (+0x5c 122) | the boss, the switch |

Other kind 6 records: rooms 0, 2, 3 song 1; 7 camera 1; 10, 20 camera 9
song 2; 12 camera 9; 14 camera 8 song 4; 16 camera 8; 23 camera 9; 24
song 4; 27 camera 9 song 3; 28 song 4; 32 camera 9 song 3; 34 camera 7.
Post 122 (117402, -4423, 25115) is the fixed camera for rooms 26, 35, 36
(seen in the oracle: the view from the far end of room 35).

### Posts (LP kind 2, 123 records)

Post flags as in [level 2](level-02.md#posts-lp-kind-2-the-28-byte-table-at-0x800c4b9c).
y is 0 in all but 122 and 123 (flag 8, camera posts).

Larry and Steve, three rings and the goal:

| ring | posts | wait (flag 2) | loop posts (flag 16), branch to |
|---|---|---|---|
| 1 | 1-12 (1 → 2 → .. → 12 → 1) | 1, 7 | 5 → 15, 11 → 20 |
| 2 | 13-22 (13 → 14 → 15 → 16 → 17 → 18 → 19 → 20 → 21 → 22 → 13) | 13, 18 | 17 → 27, 22 → 35 |
| 3 | 23-38 (23 → .. → 37 → 38 → 23) | 23, 31 | 29 → 39, 37 → 39 |
| goal | 39 (45699, 0, 13374), no neighbours | | flag 32: GoalPost |

The rings get nearer Tommy's counter (z 19446): ring 1 z -753..3604,
ring 2 z 3130..5871, ring 3 z 5590..10370. Larry starts at post 12,
Steve at 7.

Flag 16 (0x80082ee0): with a reload (+0x697 = LP +0x42 = 4) the counter
+0x696 counts down at each loop post; when it is 0 there, it is reloaded
and the next post is neighbour 1 (+0x11). Both loop posts of a ring share
one counter, so a walker branches on the fifth loop post it reaches,
about two and a half laps. 0x8007bc60 does not reset the counter.

A post with no neighbours (39) gives next post 0: 0x80082c78 clears the
think (+0x9c = 0) and returns -1, the walker stops. Its flag 32 posted
GoalPost first (0x80077ff0).

Lobster posts 40-121 are closed loops of 8-10 posts, one per lobster
(e.g. 102-108 for 2008, 109-115 for 2010, 116-121 for 2009), no special
flags. Kind 7: 18 records, only read for NPCs with +0x688 bit 16 (none
here).

## Phase 1: start (EpisodeStart, 0x800793c8)

Level start, in order: player init at 0x800b2940[3] (-2101, 0, -4685),
heading 1022; moved onto LP 84; cine 2 "Episode 2 Intro" (always, no
seen flag, 0x80074958); its end hides ids 75, 76, 13 (0x800732d4 case
2); CineFinish shows card 679 (HINT08, arg 6) (0x800783d8); then
EpisodeStart. The callback:

1. 0x800c4ba0 = 1000.
2. Player x = 0x1a0f (6671), z = 0xffffc781 (-14463), room update
   (0x8004a318).
3. Hide id 333.
4. Respawn = the player's position and angles, phase 1 (0x80078d60). In
   the oracle: (6671, 779, -14463).
5. Room sound bytes (0x80078e3c): rooms 0, 2, 3, 32, 27 = 1; 9, 18 = 2;
   26, 35 = 3.
6. Camera reset behind the player (0x80026ddc).

Room change in level 3 (0x8007f498 case 0): with player +0x188 bit 128,
reset the camera (0x800290a4) and clear the bit. Who sets bit 128 is not
found.

Nothing is scheduled at the start. The lobsters and the boss arm
themselves by distance.

## Phase 1 to the pie toss: cine 8 or 9

- Use on id 81 (GRAB or ACTION on it, 0x8004f83c): PostCine(8).
- Entering room 19: PostCine(9) (room flag 2, once). Cine 9's end hides
  id 79 (0x800732d4 case 9).

CineFinish (0x800783d8): cine 8 sets 0x800c440d = 1, cine 9 sets it to
0; both show card 681 (HINT10, arg 4) and queue 0x8007bc60. Cine 8 and 9
have no start hook in 0x80072ef0.

## The pie toss (0x8007bc60)

Prints "starting pie toss...". The phase stays 1.

1. Progress +0x30 |= 8.
2. Template flag 8 on for ids 1931, 1933, 1935 (templates 489, 566, 488).
3. Tommy at (0xb22f, -2300, 0x4bf6) = (45615, -2300, 19446), +0x148 =
   -2048, room update, heading and target heading 2048, frozen (+0x188
   bit 1).
4. Camera: the fixed record 0x800e5778 (the player's matrix and position,
   y = -3500), then 0x800291f8(camera, player, 4, 1500, 952, &(0xb1b5,
   -4200, 0x58c0), 0x800e5778, 512).
5. Pie spot: (0, 0, 512) turned by the player's angles plus his position;
   0x80049818 writes its x, z into the six pies' home and position.
6. Larry (75) and Steve (76): +0x694 = 0, +0x688 |= 0x1800, +0x695 =
   +0x698 (first post), health = max (200), placed at home (0x800825b0,
   which also puts them back in a room after cine 2 hid them), +0x688
   bit 1 off; 75 +0x678 = 76 and 76 +0x678 = 75; both scheduled
   (0x80049e18). They keep mode 6 from init.
7. One gauge (0x8008c5e0) on Larry: model 280 with 282, both at x =
   (0x800c4a04 * 5 >> 7) - 168, y = 88 - (0x800c4a06 >> 4), scale (64,
   64, 1024), flag 1. In the oracle: bottom left, a portrait of the two.
8. 0x800c4cd4 = pie 0 (0x800e2770[0]); reset (0x80049464); player +0x1f0
   = pie 0 (Tommy holds it); pie 0 into its room's list.
9. MidiPlaySong(2, 0) (0x8006eac4).

No timer is set. In the oracle at frame 4200 (900 frames in): Tommy at
(45615, -2300, 19446), +0x188 0x401, holding pie 0; Larry and Steve in
mode 7 at posts 1 and 7, +0x688 0x1800, speed 25.

### The pies (class 9, 0x8004997c)

Six 196-byte objects in 0x800e2770[0..5] cloned from LP 111: pickable
(+0x00 |= 0x10000), enter +0x48 = 0x8004d01c, +0xc0 = VSync, +0xbc /
+0xbe = template +0x18 / +0x1a (0, 0), home = position, reset with
0x80049464 (position = home, room from position, bit 0x8000 off).

Throwing (Tommy is frozen; turning still runs, so left and right aim):

1. ACTION (Triangle) in the stand state (0x8005d11c): holding pie 0 (no
   bit 0x1000000): throw requested, +0x188 |= 4, animation 18 (blend
   0x2000). Empty hands in level 3 with +0x30 bit 8: +0x1f0 = 0x800c4cd4
   again (a new pie in hand). So it is two presses a pie.
2. At the release point the throw (0x80060b44) builds the flight
   (0x80061f88). For level 3 with +0x30 bit 8, 0x80061c98 replaces it:
   flags 0x0a070400 (gravity 1, collides, ends on a wall), desc +0x46 =
   284, +0x66 = 1, +0x68 = 900, +0x67 = 0, direction (0, 35, 1024)
   turned by the player's angles and normalised, speed 225 when Larry
   (75) is within 0x32c8 (13000, 3D, 0x80075f58), else 450.
3. Think: +0x44 = 0x8004df94, then +0x48 = 0x8004d01c replaces it with
   0x8004eb10.
4. 0x8004eb10: first run marks the pie (+0x00 bit 0x8000), advances
   0x800c4134 (mod 6), resets the next pie to the pie spot, puts it in
   its room and sets 0x800e2d38 to it. When the flight ends (+0x6b 255)
   the pie leaves its room lists (0x80049380) and the think stops.
5. Flight end: +0x5c = 0x8005282c (template +0x0b = 1) throws five splat
   particles (0x8008864c each). In the oracle: a throw at frame 4327,
   five splats at 4393. The pie is template 564 (model 885) here and in
   level 16; its +0x0b is 1. Each splat's descriptor:
   - flags 0x30705: kind 5, gravity 13, world collision;
   - start at the pie, radius 200, lifetime 2000, bounce 1000;
   - texture TEXPOS + 0x10d4 (sprite 359), half-size 200 x 200, face
     flags 0xd0800000;
   - speed min(rand mod 150, 50) from the LCG 0x800c41a4;
   - direction from 0x80088bac(pie, (4096, 0, 4096), 0, 2048): a tilt of
     rand mod 2048 and a spin of rand mod 4096 (LCG 0x800c445c).
   Kind 5 flights are drawn by 0x80088388 through 0x8008afc4, the
   camera-facing quad queue the box toss's arrow uses.
6. A pie flight that touches Larry or Steve (0x80086ef0): touch callback
   0x800511a8, then Damage(player, character, 50).

Aim marker: in the carry draw, level 3 with +0x30 bit 8 runs 0x8005fe44:
the point (0, 1024, 24000) turned by the heading plus the player's
position (0x800c42f8 = 1024, 0x800c42fc = 24000) into +0x210..+0x218,
pulled in to the hit point when the ray 0x80090554 hits the world or
templates 590/591; +0x188 |= 1024. The player draw (0x80031db8) then
draws a sprite there and clears the bit. In the oracle: a small star on
the floor ahead.

Pie hit on Larry or Steve (0x800511a8, chars 10 and 11): in mode 7 they
resume (0x80083efc) with animation 6, else animation 8; mode 1 (blend
2048); speech 1065 (char 10) or 1076 / 1079 at random (char 11).

### Damage routing (0x80077864 then 0x8007e308)

Damage with +0x30 bit 8 in level 3 goes to 0x8007e308(player, hit)
after the health change:

- hit health 0: prints "Dead NPC (%d)", 0x8007e3c0(player): the pie
  toss is won.
- hit 76: Larry's health = Steve's; gauge (on 75) shows it (0x8008ca4c).
- hit 75: Steve's health = Larry's; gauge shows it.

So the two share 200 health: four pies. (0x8007e308 reads a1->0x1a4
without a null test; a Damage on the player while bit 8 is set would
read address 0x1a4. Nothing hurts Tommy during the toss: Larry and
Steve have +0x6d 0 and no +0x688 bit 8.)

### How the pie toss ends (0x8007e3c0)

Called by Damage at 0 health (above) and by GoalPost in level 3 (event
11, 0x80077070). Also the TimerExpired callback for level 3 (0x8007f5a8
table), but level 3 never sets the timer.

1. Template flag 8 off for ids 1931, 1933, 1935 (and the same on 75's and
   76's templates); Larry and Steve unscheduled (0x80049e5c).
2. Timer 0, +0x30 bit 8 off, player unfrozen (+0x188 bit 1 off), the
   gauge freed (0x8008c994(75)).
3. A held pie goes to the room under it (0x8002b660, else the player's)
   and into its list; +0x1f0 = 0; +0x188 bit 4 off.
4. Larry's or Steve's health 0: PostCine(13 "Mellon Rush"), room 19
   flag 2 off (cine 9 not re-armed), id 81 +0x54 = 0 (no balloon).
5. Else: PostCine(15 "Larry and Steve Win"), room 19 flag 2 on (cine 9
   re-armed), ForceCry(player) at once (not posted).
6. All six pies leave their rooms (0x80049380); 0x8008e878 frees the
   list at 0x800c4da0 (guess: the splats).

Cine 15: start sets +0x30 bit 128, resets the player (0x80059488) and
sets player +0x00 bit 0x80 (0x80072ef0 case 15); end hides 75 and 76 and
clears +0x30 bit 128 (0x800732d4); after it (0x8007737c case 14) the
player is placed at (0x1a0f, 779, 0xffffc781) = (6671, 779, -14463) and
the camera reset (0x800290a4). No card. The level carries on in phase 1:
the balloon or the cart ride starts the toss again.

## Phase 2: the lobster aisles (cine 13, 0x8007ece4)

Cine 13 "Mellon Rush": start resets the player (0x80059488) and sets
+0x00 bit 0x80 (0x80072ef0 case 13); end hides 75, 76, 61-71, 77, 78, 74
(0x800732d4); then (0x8007737c case 12) player +0x188 bit 128 off and
MidiPlaySong(1, 0). CineFinish shows card 682 (HINT11, arg 5) and calls
0x8007ece4(player) directly.

0x8007ece4 (also ForceCry's phase 3 respawn):

1. Phase 3 only: hide id 333; room 36 flag 2 on (cine 86 re-armed).
2. For ids 83-86 and 2007-2011 that exist: placed at home (+0x648),
   0x8008395c(npc, +0x698, player) (mode 9 from the first post), added to
   the trigger list 0x800c4c24 again, health = max (50).
3. Tommy at (0xb1a4, 779, 0x386e) = (45476, 779, 14446), heading and
   target heading 2048, matrix (0x80020d10), room update, +0xfc = +0xfd =
   255, +0x148 = 1024, +0x1e8, +0x1cc, +0xf4, +0xf8, +0x144 = 0, +0x00 bit
   0x80 off.
4. Respawn = there, phase 2 (0x80078d60). Camera reset (0x800290a4).
5. MidiStopSong(2) (0x8006ebb4).

### Lobsters: mode 9 (0x80084084)

Goal post +0x695; target +0x678 (the player). s4 = the larger of |hier
+0x08| and |hier +0x10| (the hierarchy's half size, +0x154).

- dist(npc, target) <= 2 x s4 and heights overlap (0x80083ff0): with
  +0x688 bit 8, mode 5 (attack).
- Else, when the current post is within 2 x range (9000) of the lobster
  and the target is within range (4500): goal = the target (chase).
- Else: goal = the post; 0x80082c78 advances posts within +0x690 (1024).

Speed stays the table value set at init (index 1: 4). Each think of a
char 23 not in mode 5, with its animation finished (+0xe0 bit
0x8000000): sound 51 at its position.

Mode 5 for template 495 also calls training message 13 (shown only in
levels 16 and 1, so nothing here) and sound 52 when the animation ends;
templates 763 / 764 (ids 85, 86) make no sound. Damage(player, player,
15): the bite. Then mode 1.

Hit by a thrown can (0x800511a8, char 23): with health > 0, animation 8
(blend 4096), mode 1, sound 44 (0x8006f26c). Mode 1 at 0 health goes to
mode 10; mode 10 for char 23 is 0x8007ebd4: when health is 0, effect
0x80050fd0, unscheduled, off the trigger list, moved to (0, -50000, 0).
A can (+0x6d 50) kills a lobster (50) in one hit; an opened can (+0x6d
8, guess) needs seven.

### Phase 2 to 3: rooms 22 and 36 (Gameplay script 1)

Room flag 64 posts Gameplay (0x800789e4) with the room's +0x5f; the
handler (0x80077ee4) runs script 1 = 0x8007dfe0 and clears the room's
flag 64. Rooms 22 and 36 both have script 1 (36 also cine 86).

0x8007dfe0: phase 3 (respawn = the player's position and angles,
0x80078d60); for ids 2008, 2009, 2010, hierarchy (+0x154) +0x10 and
+0x0c = 1000 (guess: a larger box, so their mode 9 bite reach 2 x s4
becomes at least 2000).

Entering room 36: PostCine(86 "Boss Lobster") once. Cine 86 has no
start hook; its end shows id 333 and MidiPlaySong(2, 0) (0x800732d4
case 86). CineFinish: card 683 (HINT12, arg 6), Tommy placed at (x, y,
8000) (0x800825b0, room 26).

## Phase 3: the boss and the switch

### The boss lobster: mode 3 (0x80084084)

Target +0x678 (the player), saved heading +0x654 (its LP heading, 2048:
facing -z).

- Target within range (2000, 2D) and not standing higher (player y +
  player hier +0x0e >= boss y + boss hier +0x06): the swat. 0x800c4268 =
  -1750, 0x800c426c = 2500, 0x800c4264 = 0, the player enters state 16
  with the boss's heading as his heading and target heading; boss
  animation 37 (blend 3072), sound 148, mode 1. No Damage; the landing
  may cost 5 (fall, 0x8005b734).
- Else: speed = saved speed; heading to the target; if the target lies
  to one side (difference outside 4071..4122) it turns +-1024 for the
  stick, steps sideways (0x80082230) and takes back its saved heading;
  else it stops. It strafes to stay between Tommy and the switch (guess
  from the behaviour).

Hit by a thrown object (0x800511a8, char 33): sound 44, mode 8 with
+0x69c = 600 and +0x688 |= 32: stunned 600 dt (2 s), then back to mode 3
(mode 8 with bit 32 counts +0x69c down by dt, then mode = +0x69b). Char
33 takes no health from Damage (0x80077864), so it cannot be killed.

### Winning: the switch

LP 278 (template 873) at (107630, -3516, 20608) has use 0x8004f83c with
+0x6a = 14: from LP +0x44 = 16, remapped by the generic init
(0x8004691c: +0x44 of 16 becomes 14). Tommy reaches it by the can stacks
in room 35 (in the oracle he stands at y -3275 next to it). ACTION
(Triangle) in the stand state with it touched enters state 12; the use
posts Cine 14 "Episode 2 Ending". After cine 14 (0x8007737c case 13):
exit code 1, 0x80076d6c(3, 1) (level 3 played and won, its puzzle piece,
the main-level count; see [objectives](../engine/objectives.md)). Cine 14 has no
start or end hook. Level 1's EpisodeStart then places Tommy at
0x800b8718[3].

## ForceCry (0x8007fd0c, level 3 case)

1. Cry (0x8007f920): in phases 2 and 3 always; in phase 1 only when the
   camera's +0x4a is 1 (in the oracle it was 4 after cine 15: no cry).
2. Tries: progress +0x23 (3 at level start, 0x800b2920[3]) counted down.
3. With a respawn set: the list is just the player, to the respawn.
4. Health 255.
5. Tries left: phase 3 first runs 0x8007ece4 (lobsters back, Tommy at
   (45476, 779, 14446), phase 2, room 36's cine re-armed, id 333
   hidden); then the player is moved to the respawn (0x8007fbf8).
6. No tries left: SwitchEpisode(player, 1).

Respawns: phase 1 the start (6671, 779, -14463) (and cine 15 puts him
there again after its cry); phase 2 (45476, 779, 14446). Script 1 sets
the respawn to where Tommy entered room 22 or 36, but in phase 3
ForceCry runs 0x8007ece4 first, which sets it back to (45476, 779,
14446) and phase 2, so that respawn is never used.

What hurts Tommy: lobster bites (15), the hazard props (4-15 on touch),
falls (5). The boss only throws him back.

## How the level ends

- Won: the switch, cine 14, 0x80076d6c(3, 1), exit code 1.
- Lost: the last try (ForceCry): SwitchEpisode(1). There is no "lost"
  ending cinematic for level 3.

## Hint cards

| card | file | shown | text (abridged) |
|---|---|---|---|
| HINT08 | 679 | after cine 2 (arg 6) | Work your way to the back of the store; Reptar posters; press ^ on slippery areas; hop the fruit bins into the bakery |
| HINT09 | 680 | no caller found | Press @ for a fun ride to the bakery |
| HINT10 | 681 | after cine 8 or 9 (arg 4) | Watch out for Larry and Steve; press @ to toss pies, left and right to aim |
| HINT11 | 682 | after cine 13 (arg 5) | Lobsters; unopened cans tame them, opened cans make Tommy cry; # picks up, @ throws; get to the soapy area |
| HINT12 | 683 | after cine 86 (arg 6) | Get past the boss lobster and his bullies to throw the switch; a can stuns the boss |

The port's card table (crates/rugrats/src/ui/front.rs card_after) already
has 2, 8, 9, 13 and 86.

## Per-frame ambience (0x80080330)

0x800b9618[3]. A countdown 0x800c4cc0: during the pie toss one of eight
speech lines from 0x800b92dc every 2000 dt; in rooms 26 and 35 one of
six from 0x800b92ec every 2000; else 1087 + rand % 22 every 5000.
Speech only.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, hooks with arguments,
object dumps): presses start 300, cross 500, 650, 800, 1650, 1800 to
reach Main Game; main's level argument patched (0x80018824 = `li a0,
3`), 0x800c5678 = 3, cards skipped (0x8004472c = 0x1000004a). Pad input
does reach the level in these runs (Triangle threw pies).

- Load at 1816, cine 2 to 2967, EpisodeStart at 2967: phase 1, respawn
  (6671, 779, -14463) flag 1, tries 3, +0x30 0x100.
- Tommy poked into room 19 at 3000: cine 9 at 3003, 0x8007bc60 at 3309.
  State as listed under the pie toss.
- Triangle at 4300 and 4500: flights at 4327 and 4527 (from the throw),
  five splats 66 frames after each.
- Left alone: Steve reached post 39 at 7993, GoalPost, 0x8007e3c0, cine
  15, ForceCry (no cry: camera +0x4a 4), tries 2; after cine 15 Tommy at
  (6671, 779, -14463), Larry and Steve out of the rooms.
- Health of 75 and 76 poked to 0, GoalPost at 7996: cine 13; after it
  0x8007ece4, phase 2, respawn (45476, 779, 14446). Lobster 83: mode 9,
  +0x688 0x4c08, +0x6d 15, speed 4; boss: mode 3, +0x688 0x1c04.
- Tommy poked into room 22 (9200): Gameplay 1, phase 3. Into room 36
  (9500): cine 86 and Gameplay 1; after it id 333 shown, Tommy at
  (113853, 779, 8000).
- Next to lobster 2008 with health 20: two bites of 15 (Damage(player,
  player, 15)), ForceCry with a cry, 0x8007ece4, phase 2, tries 2.
- At (114090, 0, 20500) facing the boss: sound 148, Tommy state 16 with
  0x800c4268 -1750, 0x800c426c 2500, thrown to z 16075, landing
  Damage(player, 0, 5); the boss in mode 1 then mode 3.
- Tommy poked onto the can stack at (107630, -3275, 19900), heading 0,
  Triangle at 10300: use 0x8004f83c, cine 14, then 0x80076d6c(3, 1).
- Not checked: a pie hit (Damage on Larry or Steve), a can hit on a
  lobster or the boss, cine 8 (the balloon).

## For the port

Order (the existing port code is crates/rugrats/src/world/game/,
levels/grandpas_teeth.rs, actors/npc.rs):

1. Data: already read: LP kinds 1, 2, 6 (room flags 2, 32, 128 in
   `Game::room_change`), templates, posts with flags. Add: kind 6 bytes
   +2 (camera switch, flag 4), +6 (Gameplay script, flag 64); post flags
   16 (loop, LP +0x42 reload) and 32 (GoalPost); template flag 8 (no
   collision); +0x688 bit 0x800 (NPC skips object collision); the
   0x8004691c remap of LP +0x44 16 to cine 14.
2. Events: RoomChange flag 64 calling the level's Gameplay script 1;
   GoalPost (an `npc::Event` like `RouteEnd`, posted before the stop at
   a flag-32 post); the cine start hooks (13, 15) and end hooks (2, 9,
   13, 15, 86) of 0x80072ef0 / 0x800732d4; CineFinish callbacks (8 and 9
   the pie toss, 13 0x8007ece4, 86 the placement at z 8000); 0x8007737c
   cases 13 (cine 14 won: already in `ENDINGS`), 14 (cine 15 placement)
   and 12 (cine 13). Cards: already in `card_after`.
3. Use callbacks on cinematic objects (0x8004f83c: id 81 cine 8, LP 278
   cine 14). The port's Triangle use only knows doors (LP +0x42 > 0).
4. A `level3` module like `level2`: phase (0x800c4cd0), respawn, tries;
   EpisodeStart 0x800793c8; ForceCry's level 3 case (phase 3 runs
   0x8007ece4 first; phase 1 no cry).
5. NPC modes beyond npc.rs (which has 0/Idle, 1/Hit, 4/Follow, 6/Posts,
   7/Wait, 11/Guard, riders): mode 9 (posts with a leashed chase and a
   bite reach of twice the hierarchy half size), mode 3 (strafing guard
   and the swat: player state 16 with -1750 / 2500), mode 8 with bit 32
   (timed stun, 600 dt), mode 10 for char 23 (0x8007ebd4). Mode 7's
   y = -384 for ids 75 and 76 and its resume tests (time, range, the
   768 direction test). Loop counters on posts.
6. The pie toss: frozen player with turning, fixed camera, pie pool of
   six with ACTION re-arming pie 0, the aimed roll (0x80061c98: flags
   0x0a070400, speed 225 / 450), splats, the aim marker, object hits on
   NPCs (touch 0x800511a8 and Damage 50), the shared health and the
   single gauge (model 280 / 282, hud::gauge exists), 0x8007e3c0's two
   outcomes.
7. Thrown cans: the port's carry and flight (`Game::carry`, `fly`) need
   object hits on NPCs (Damage with the can's +0x6d) and the level 3 can
   sound 53.
8. Player state 16 (launched) and the hazard Damage on touch
   (0x8005e1f4).

## Flags settled since

- Template flag 0x2000 (templates 590, 591, 495, 711): the object tests
  meet the body's posed parts (0x8006a714 at 0x8006a7bc, 0x8006a318),
  not its hierarchy's one box (0x8006a008).
- +0x688 bit 0x400: a character never climbs (0x80058da4 at 0x80058dd4).
- +0x688 bit 0x1000: a character's held probe (0x80064838, kind 5) lets
  it by with the floor delta 0 (0x800649c0).
- The brothers' height through a wait is -384 (worklog 203).

## Unknown

- Which of 75 / 76 is Larry and which is Steve; what ids 13, 74, 79, 81,
  333 look like (79 the cart, 81 the balloon and 333 something to climb
  are guesses).
- HINT09 (680): no caller found.
- Why "opened cans make Tommy cry": templates 650 / 651 are pickable with
  +0x6d 8 and no callbacks; no code found that makes them hurt.
- Who sets player +0x188 bit 128 (cleared on level 3 room changes and
  after cine 13); what player +0x00 bit 0x80 with +0x148 / +0x14c does
  (0x80067084 eases y), and why cine 15 leaves bit 0x80 set.
- The pie flight's desc +0x46 (284), +0x66, +0x67, +0x68 (900): the
  record's +0x2a, +0x8a, +0x8c, read by 0x8008652c at a hit (a mark of
  sprite 284, 900 square, 0x8008e72c / 0x8008e76c, and sound 46 at the
  player); the mark's drawing is not traced.
- A brother's height at a post (the wait, table 31). Seen, Steve (76)
  through the pie toss, a frame at a time: +0x1c is -384 (his floor
  1024 plus feet -1289, less 119) plus +0xda, the y displacement. +0xda
  goes 141 while the wait blends in from the walk, then 54, 0 (1206 to
  1207), -8, -18 (to 1208), -9, 0 (to 1209), and 0 through 1213. The
  wait's lift keys are 525 (1206), 0, -78 (1208), 0 ... 78 (1214). So
  the displacement follows the frame's own key at about a quarter of it
  (141 for 525, -18 for -78), eased across the frame, not the keys'
  running total scaled by +0x4c over -561 (2.3 for an adult) that a
  runner's height takes. +0xde holds 1325 throughout (the gap at the
  wait's first frame: 0x80055904 at 0x80055b3c). 0x80055904's y is not
  read through, and the port poses its waiting brothers about 1100 low.
- Whether 0x8007dfe0's hierarchy write is per object or shared by every
  char 23 (+0x154 copies).
- The camera's +0x4a values (the phase 1 cry test).
- The cart ride and balloon cines' content; whether the balloon and the
  cart can both be taken.
- Re-adding lobsters to the trigger list in 0x8007ece4 makes duplicate
  entries; whether that matters.
