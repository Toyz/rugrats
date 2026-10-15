---
title: Level 17, Gold Rush
status: partial
discs: US
covers: US SLUS_006.50:0x80079af0, US SLUS_006.50:0x80079870, US SLUS_006.50:0x80079b38, US SLUS_006.50:0x80079bec, US SLUS_006.50:0x80079c10, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x80048a44, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8004fd20, US SLUS_006.50:0x8004a34c, US SLUS_006.50:0x800585b0, US SLUS_006.50:0x8005928c, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x8007f920, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x80078dd8, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x80076ba8, US SLUS_006.50:0x80076b94, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80077c88, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x8005d030, US SLUS_006.50:0x800813a0, US SLUS_006.50:0x800818e0, US SLUS_006.50:0x8008168c, US SLUS_006.50:0x80081758, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80018598, US SLUS_006.50:0x8008a1f8, US /DATA/DB17/DB17LP.BIN, US /DATA/DB17/LEVEL17.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 17, Gold Rush

Everything level 17 needs to be played from its start to its end: Phil
picks up 62 nickels in the park against a 170-second clock, and if he
runs out of time Lil gets one more round. A gold bonus coin is worth a
fourth Reptar bar. It is also an Activity. Read from the pseudo-code and
the disassembly; the start, collecting all 62 coins by walking, the
gold coin, the win, Phil's loss and the swap to Lil, Lil collecting, Lil's
win and Lil's loss, the way back to the hub and activity play were run in
the oracle (see Checked in the oracle). Builds on
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md),
[screens](../engine/screens.md), [level 15](level-15.md) (spawn groups, the
level timer) and [level 20](level-20.md) (the body swap).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Phil (char 5, 0x80018598 case 17; LP 38, id 608). Gold
Rush is a bonus level: 0x80076d6c starts it when the seventh main level
is completed (+0x22 = 7, printf "should trigger LVL_GOLD_RUSH next",
0x80076ff0). Cine 12 "Gold Rush Intro", then card HINT36 (file 707):
Angelica wants all the nickels in the park; gather as many coins as you
can before time runs out; look out for a gold bonus coin. The level
places 62 coins (template 446, kind 2), one at each spot of spawn group
1, and the gold coin (template 1218, kind 15, id 1000) at one of four
spots. A 170 s clock runs. Coins are taken by walking into them; the
count left shows top left. Taking the 62nd sets the clock to one tick, so
the clock's end checks the count: all taken plays cine 26 "Win Bonus 5",
3 Reptar bars, 4 when the gold coin was taken, and the hub. The clock
running out with coins left is a ForceCry. For Phil there is no cry: cine
27 "Lose Bonus 5" plays, the player becomes Lil (char 4, id 609) and the
coins and the clock start again. For Lil it is a real cry, the last try,
and the hub with nothing won. Nothing hurts (no Damage is posted in level
17).

## How level 17 is reached

- Main game, once: 0x80076d6c (a main level of 2-6 or 8-12 completed for
  the first time) counts +0x22; at 7, SwitchEpisode(17) (0x80076ff0,
  0x80077028), +0x30 bits 1-2 cleared (0x80077000). Won or lost, the main
  game does not offer it again: no DB01 record has LP +0x42 = 17 (no hub
  door, from the data).
- The Activities menu: ACTIVITY_LEVELS 13, 17, 23, 18 (0x80013030),
  +0x21 = 1. See Activity play below.
- The loading screen: "Bonus Level" / "Gold Rush" / "Loading" in the main
  game, "Activity" / "Gold Rush" / "Loading" in activity play
  ([screens](../engine/screens.md)).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 Phil's round, 2 Lil's round (0x80078d60) |
| 0x800e2740 | s32 | coins in slot 0: 62; the total the clock end compares with |
| 0x800e2744 | s32 | copies in slot 1 (the gold coin): 1 |
| 0x800e26e0, 0x800e26e4 | s32 | the slots' spawn groups: 1 (coins), 0 (gold coin) |
| 0x800e2710 | ptr | slot 0's first coin; coins are 232 bytes apart |
| 0x800c4b94 | u8 | also 62 (written by 0x80079870, not read by level 17's code) |
| player +0x1a9 | u8 | coins taken this round; 0 at each round start |
| player +0x220 | ptr | the coin counter HUD item |
| 0x800c4658 | ptr | the gold coin icon, 0 when none |
| 0x800c4430 | s32 | the level timer (dt); 51000 at a round start |
| 0x800e5830, 0x800e583c | | respawn; set at each round start (phase 1, then 2) |
| progress +0x23 (0x800c569b) | u8 | tries: 2 (0x800b2920[17]) |
| progress +0x30 (0x800c56a8) | u32 | bit 8 set by 0x80079870 (a minigame phase) |
| 0x800c4018 | u32 | 6500 for level 17 (0x80078944; 2600 for most levels, 3100 for 23) |
| object 1000 +0xab | u8 | bit 2: the gold coin was taken (0x80076b94) |

Level 17 has a TimerExpired callback (0x8007f5a8 table 0x800158cc index
15 = 0x80079b38), no room change case (0x8007f498 covers 3-15), no room
cine, no Gameplay script, no CheckPoint room. Its per-frame ambience is
0x800b9618[17] = 0x800813a0 (activity play uses another, below).

## Data in DB17

### The park

10 rooms (LEVEL17.BIN; boxes read from RAM, room +0x10..+0x28: x0, y0,
z0, -, x1, y1, z1).

| room | x | z | flags (+0xc) | +0x5d | +0x64 |
|---|---|---|---|---|---|
| 0 | -16384..4096 | 0..28672 | 0x90 (16, 128) | 0 | 1 |
| 9 | 4096..26624 | 0..28672 | 0x90 (16, 128) | 0 | 1 |
| 1 | -32768..-20480 | 13312..25600 | 0x14 (4, 16) | 8 | 0 |
| 2 | 13312..26624 | 28672..38912 | 0x14 | 8 | 0 |
| 3 | -28672..-16384 | 4096..9216 | 0x14 | 8 | 0 |
| 4 | -28672..-16384 | 28672..33792 | 0x14 | 8 | 0 |
| 5 | -6144..13312 | 33792..39936 | 0x14 | 8 | 0 |
| 6 | -16384..-6144 | 28672..38912 | 0x14 | 8 | 0 |
| 7 | -28672..-24576 | 25600..28672 | 0x14 | 8 | 0 |
| 8 | -28672..-24576 | 9216..13312 | 0x14 | 8 | 0 |

Rooms 0 and 9 are the open park; 1-8 the paths and corners around it
(screenshots: grass, trees, a fence, a sandbox, spring riders, a hedge
path). Flag 128 with +0x64 = 1 starts song 0 (0x80077a54 at 0x80077c20).
Flag 4 with kind 8 is a CameraSwitch (below). Flag 16 is not traced. LP
kind 6 records 1, 29-34, 52, 53 and 55 set these bytes; four LP kind 3
records (2-5) are not traced; one kind 99 record.

### The characters (LP kind 1)

| LP | template | char | id | LP position | what |
|---|---|---|---|---|---|
| 38 | 7 | 5 | 608 | (-6603, -1269, 2655) | Phil, the first player; cine 12 leaves him at (-5531, -398, 4480), room 0, heading 676 |
| 37 | 6 | 4 | 609 | (-8293, -1286, 4856) | Lil; hidden after cines 12 and 27; the player in phase 2 |
| 36 | 1 | 3 | 600 | (22827, -3375, 4542) | Angelica; only in the cines (hidden by their end hooks) |
| 39 | 104 | | 35 | (22618, -3448, 4806) | cine prop, hidden by the end hooks of 12 and 27 |
| 40 | 105 | | 34 | (1117, -926, 26798) | cine prop, hidden by the same |
| 64 | 799 | | 117 | (-2425, -481, 6082) | cine prop, hidden by the same |

The loader puts the player on the kind 1 record whose shape is char 5
(0x800454dc; see [level 5](level-05.md)). Ids 40-42 and 125 (templates
775, 524, 776, 798) are not read by level 17's code. The rest of LP kind
1 is scenery (templates 269-313, 391, 408-411, 437, 446, 787, 805).

### The coins (template 446)

LP 54: class 12, kind (+0x26) 2, 62 copies (LP +0x46), group 1 (+0x48),
+0x4a 1, shape 761, draw byte 3. Init 0x80048a44 case 2 (0x80048be0): box
-375..375, +0xa9 = 2, +0xb0 = 0xc0800000, +0xab |= 1, +0xc0 =
0x8008a2a4(0x1011), size +0xb4 = +0xb6 = 300, grey 128. Seen: silver
"5c" nickels drawn as sprites, still (no bobbing).

### The gold coin (template 1218, id 1000)

LP 65 at (1956, 0, 27889): class 12, kind 15, 1 copy, group 0, +0x4a 1,
draw byte 3. Init case 15 (0x800491f8): the same box, +0xa9 = 15, +0xc0 =
0x8008a2a4(0x1022). 0x80076ba8 looks it up by id 1000 for the fourth bar.

### Spawn groups (LP kind 7)

66 records: x, y, z at record +4, group at +0x10, threshold at +0x11.

| group | spots | thresholds | what |
|---|---|---|---|
| 0 | LP 82-85: (20997, -741, 3193), (-22518, -1318, 15415), (24604, -673, 27919), (-28186, -343, 33229) | 25, 50, 75, 100 | the gold coin: one spot in four |
| 1 | LP 86-147, 62 spots from (-24567, -343, 31717) to (-10793, -396, 1543) | 1 .. 100 | one coin each |

0x8004fe14(list, group, n), per object: r = (rand & 255) mod 100; the
first free spot of the group (+0xf bit 0 clear) whose threshold is above
r; up to five tries, the later ones with r = 0 (so the first free spot).
The object is moved there, put in that room (0x8004a34c), its spot index
kept in +0xaa and the spot marked used. 0x8004fd20 takes a list out of
the rooms and frees its spots (only objects still in a room with +0xaa
set). So 62 coins fill all 62 spots and the gold coin lands at a random
one of four. Seen: (24604, -673, 27919) at the first start, (-28186, -343,
33229) after a re-placement.

There are no Reptar bars (template 835) in DB17.

## Start (EpisodeStart, 0x80079af0)

The level start, in order: player init on LP 38; cine 12 (0x80074958);
its start hook does nothing (0x80072ef0 case 12 is the default); its end
hook (0x800732d4 case 12, 0x800734b4) hides 609, 600, 117, 35, 34, then
the default 0x80074ec0(0, player id); CineFinish (0x800783d8 case 11,
0x80078634): card 707 (HINT36, arg 6) only outside activity play;
EpisodeStart 0x80079af0 (installed by 0x8007871c case 17, which also
writes 0x800c4018 = 6500); 0x8007737c case 11 does nothing.

0x80079af0(player): 0x80079870(player); player +0x26 = +0x28 = 0;
respawn = the player's position and angles, phase 1 (0x80078d60).

0x80079870(player), the round setup:

1. The counter (+0x220) and the gold icon (0x800c4658) are freed if
   present.
2. All slot 0 coins out (0x8004fd20) and placed at group 0x800e26e0[0]
   (0x8004fe14).
3. Outside activity play (0x800c5699 = 0): the slot 1 copy (the gold
   coin, 0x800e2714) out and placed at group 0.
4. 0x800c4b94 = 62; 0x8008a1f8(first coin +0xc0, 62) (sets that handle's
   +0x18; not traced).
5. Camera: 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 612) and
   0x80026ddc(camera, player, 0). 1500 in level 17 gives the follow
   height 0x800c3f5c = 1024 (0x8002927c).
6. 0x800e2740 = 62; +0x30 |= 8; player +0x1a9 = 0.
7. The counter: icon model 286 at scale (64, 64, 1024), at (-192,
   ((0x800c4a06 x 171) >> 11) - 100) = (-192, -80) (0x800c4a06 is 240,
   seen), with the text "%d" (62) at (-205, -93), white, flags (1, 1)
   (0x8008f8f4, 0x80090004); kept in +0x220.
8. Song 0 (0x8006eac4(0, 0)); timer 170 s (0x80078dd8(170): 51000 dt,
   the clock at the top).

Seen: cine 12 from 1819 to 3145 (1326 VSyncs); at 3145 card 707 (arg 6),
0x80079af0, 0x80079870, 0x80078dd8(170), 0x80078d60(phase 1); Phil
(id 608, char 5) at (-5531, -398, 4480) room 0, heading 676; timer
51000; tries 2; +0x30 0x108; the counter "62" on a silver coin top left
and "167" by the clock top centre (screenshot at 3300); 62 coins placed;
the gold coin at (24604, -673, 27919) room 9; Lil, Angelica and ids 34,
35, 117 out of the rooms.

## Collecting (0x800514b0)

Coins are touched by walking into them (calls from 0x8005d5cc and
0x8005e34c, as in [level 15](level-15.md)). The generic part: three
sparks, the coin leaves its room, +0xab |= 2, its spot freed, +0xaa = 0.
Then by kind:

- Kind 2, a coin (0x80051ba4): sound 31 (0x8006f26c); player +0x1a9 += 1;
  the counter text = 0x800e2740 - +0x1a9 ("%d", 0x80090214); when they
  are equal, the timer 0x800c4430 = 1. Then 0x8008a1b8(+0xc0).
- Kind 15, the gold coin (0x80051a14): sound 31; in level 17
  (0x80051a98), if 0x800c4658 is 0, an icon: model 240 at scale (32, 32,
  1024), at (-140, ((0x800c4a06 x 171) >> 11) - 80) = (-140, -60), kept
  in 0x800c4658.
  It does not count towards the 62.

With one tick left, the next heartbeat ends the clock: sound 117 and
TimerExpired (0x8007f5a8). So the win plays the clock's buzzer too.

Seen: the 62 coins taken by teleporting Phil 600 from each spot and
holding Up (sound 31 each); "60" with a gold coin icon right of it after
the gold coin and two coins (3300); the 62nd at 7819; sound 117 and
TimerExpired at 7821.

## The clock runs out (0x80079b38)

Every end of the clock calls 0x80079b38(player):

1. +0x1a9 below 0x800e2740: ForceCry (0x8007fd0c).
2. Else, in phase 2: 0x800585b0(player, id 608) and 0x8005928c(5,
   player): back into Phil's body (below). Then PostCine(26).
3. The gold icon (0x800c4658) is freed.

### ForceCry, level 17 (0x8007fd0c case 16, 0x800800e0)

The generic part counts the tries down (2 -> 1 -> 0). Then:

- Phase 1 (Phil): no cry (s1 = 0), health 255, with the respawn set
  (always) the respawn pointers, and the callback 0x80079bec =
  PostCine(27). With a try left, back to the respawn (0x8007fbf8).
- Phase 2 (Lil): the cry (0x8007f920), health 255, no callback, no
  respawn. With no try left, SwitchEpisode(player, 1) (0x800802b4): the
  hub. No cine and no 0x80076d6c: nothing is recorded by this loss.

The tries are 2, so Phil's loss always leaves one and Lil's always
leaves none (from the code). The cry handler in levels 2 and 17
(0x8005d030, state 9) also makes model 1008 particles at body part 16
raised 1024, at random half the frames when nothing is touched (guess:
tears).

## Phil's loss and Lil's round (cine 27, 0x80079c10)

Cine 27 "Lose Bonus 5": start hook nothing; end hook (0x800732d4 case
27) hides 609, 600, 117, 35, 34 like cine 12's; CineFinish (0x800783d8
case 26, 0x800786d0): card 709 (HINT38 "Now try again.", arg 1) only
with tries left; 0x8007737c case 26 (0x800774f4): 0x80076d6c(17, 0)
(+0x34 bit 17, music off; nothing in activity play), then
0x80079c10(player):

1. The counter and the gold icon freed.
2. 0x800585b0(player, Lil): the body swap. The player gets Lil's first
   392 bytes (position, angles, id 609, model); Lil's object gets the old
   player (id 608) and becomes an idle NPC (+0x00 |= 0x4000, 0x80083918);
   the ids are registered again (0x800462d4); the camera follows the new
   player. Level 20's swap (0x80057570) does the same 1644 / 392-byte
   exchange in its own code; 0x800585b0 is also used by levels 6 and 11.
3. 0x8005928c(4, player): char 4, Lil's parts and hierarchy.
4. Hide id 608 (Phil, now the NPC).
5. 0x80079870: coins and the gold coin placed again, the counter at 62,
   +0x1a9 = 0, 170 s.
6. Respawn = the player's position, phase 2.

Taken coins keep +0xab bit 2 when placed again. Only 0x80076b94 reads
that bit, so they can be taken again (from the code). A gold coin taken
by Phil stays "taken" for 0x80076ba8 (seen).

Seen (timer poked to 1 at 3300): TimerExpired 3301; 0x80079b38,
ForceCry, no 0x8007f920, 0x80079bec, PostCine(27) and 0x8007fbf8 at 3303;
tries 2 -> 1; cine 27 from 3303 to 3986 (683 VSyncs; Angelica between
Phil and Lil, screenshot); at 3986 the hides, card 709 (arg 1),
0x80076d6c(17, 0), 0x80079c10, 0x800585b0, 0x8005928c(4), hide 608,
0x80079870, 170 s, respawn phase 2. The player Lil (id 609, char 4,
health 0: the swap copies only 392 bytes) at (-3657, -342, 10523) room 0,
where cine 27 left Lil; Phil an NPC, out of the rooms, mode 8; the gold
coin at (-28186, -343, 33229); the counter "62", the clock 168. Lil took
coins as Phil did (5 between 4260 and 4420 in another run, counted from
0).

## Lil's loss

Seen (Phil's loss as above, then the timer poked to 1 at 4400):
TimerExpired 4401; 0x80079b38, ForceCry, the cry 0x8007f920 at 4403;
tries 1 -> 0; +0x30 bit 32 during the cry; SwitchEpisode(player, 1) from
ForceCry at 4823 (420 VSyncs later); the hub at 4885 (0x80078e7c),
Tommy at (5775, 779, 19403) room 23.

## Winning (cine 26)

PostCine(26) from 0x80079b38. Cine 26 "Win Bonus 5": start and end
hooks the defaults; no card (0x800783d8 case 25). 0x8007737c case 25
(0x800774e0): exit code 0x800c4918 = 1, 0x80076d6c(17, 1).

0x80076d6c(17, 1) outside activity play: +0x34 |= bit 17; bars (table
0x800152cc index 4, 0x80076e48): 3, or 4 when 0x80076ba8 is true (object
1000 is class 11 and +0xab bit 2 is set; there is no "already earned"
test for 17, unlike 13 and 18): 0x80094b10(17, n) sets bits 31 down in
+0x48[17], 0x80094ab0(n) "Great Job! You have earned n Reptar bars!";
after Cross 0x80094bfc(17): level 17 owns no piece, only the 15-bar check
(0x80094924); +0x38 |= bit 17 the first time; not a main level, so +0x22
is not counted. The hub: 0x80078e7c puts the player at 0x800b8718[17] =
(5775, 779, 19403), the spot of levels 20 and 21.

Seen (Phil, all 62): cine 26 from 7823 to 8252 (429 VSyncs; Phil, Lil
and Angelica by a pile of nickels, screenshot), 0x80076d6c(17, 1),
0x80094b10(17, 3), 0x80094ab0(3). With the gold coin taken at 3203:
0x80076ba8 true, 0x80094b10(17, 4), "4" on the bars screen, after Cross
0x80094bfc(17) and 0x80094924 at 8558.

Seen (Lil's win: Phil took the gold coin and four coins, lost; Lil took
five; 0x800e2740 poked to 0 and the timer to 1 at 4500): 0x80079b38 at
4502, 0x800585b0(player, 608) and 0x8005928c(5): Phil again; cine 26
4502-4931; 0x80076d6c(17, 1) with 0x80076ba8 true (the gold coin Phil
took), 4 bars; Cross; 0x80094bfc at 5109; the hub at 5171, Tommy at
(5775, 779, 19403) room 23; +0x34 = +0x38 = 0x00020000; +0x48[17]
(0x800c5704) = 0xf0000000.

## Activity play (+0x21 = 1)

- No card after cine 12 (0x800783d8 at 0x80078634).
- The gold coin is not placed: the loader skips templates 835, 863, 1218
  and 1219 in activity play (0x800454dc at 0x80045fa0), and 0x80079870
  skips slot 1. It stays at its LP spot (1956, 0, 27889) (seen; guess:
  out of reach). The best is 3 bars' worth, but nothing is recorded.
- The ambience is picked by character, not by level (0x800818e0): char 5
  uses 0x800b9618[29] = 0x8008168c, char 4 0x800b9618[23] = 0x80081758,
  golf's functions. They play speech from 0x800b9568 (Phil: 742, 1217,
  1219, 1205, 1210, 1200, 1214) or 0x800b9550 (Lil: 1217, 878, 1219, 941,
  1205, 1210, 1200, 1214), 820 / 868 or 949 / 995 in state 15, every
  (rand & 255) x 16 + 2700 (Phil) or + 1800 (Lil) dt. 0x80081758 also
  moves a golf ball, which level 17 has not (0x800c42b0 = 0).
- 0x80076d6c returns at once: no +0x34 / +0x38, no bars.
- The runner forces the exit to -1: the "Returning to the Activities
  Menu" loading screen and the Activities menu
  ([screens](../engine/screens.md)).
- Restart (Select) is off in activity play (0x800190c0).

Seen (+0x21 poked to 1 at 1805 and 1815, not through the menu; a win
forced at 3300): no 0x80044434 at 3144; object 1000 at (1956, 0, 27889);
0x8008168c every frame and never 0x800813a0; speech 1205 at 3204; cine 26
3302-3731; 0x80076d6c(17, 1) with no 0x80094b10; 0x80018264 and
0x800183ac at 3731; the Activities menu (0x8003b7bc) at 3789; +0x34 and
+0x38 0.

## Camera (0x80077a54, 0x80077c88)

The round setup uses the follow camera, mode 1, height 1024, distance
3000. Rooms 1-8 have flag 4 and kind 8: RoomChange posts CameraSwitch(8)
(0x80077a54 at 0x80077b34). 0x80077c88 lets a switch through while +0x30
bit 8 is set only in levels 17-18; kind 8 is mode 1 with 1600 and 3000
(0x80077ddc), so the follow height becomes 1600 (0x800291f8 at
0x800292a8). Rooms 0 and 9 have no flag 4, so the height is not set back
(from the code). Seen: camera +0x4a 1 everywhere, room 1 included; the
height was not checked.

## Per-frame ambience (0x800813a0)

Countdown 0x800c4cc0 minus dt; below 0: r = rand & 255; with 0x800c4cc4
set (cleared here; set by 0x80080318), speech r mod 5 for char 5 or r & 3
otherwise; with it clear, r & 3 when r is odd, r mod 3 when even
(0x8008da4c); then countdown (rand & 255) x 8 + 900. The ids 0-4 are
looked up in the level's speech table (0x8008ddfc; DB17's SPEECHTB.DAT
lists 0-8). Seen: speech 0 at 3205, 1 at 3759 and 4133 (Phil), 0 at 4415
and 4747 (Lil). What they say is not traced.

## What does not happen in level 17

- Damage: 0x800780ac posts no Damage event in levels 1, 11, 17 and 23-31.
  No fall or touch hurts.
- The counter, the gold icon and the clock are the whole HUD; no health
  meter shows.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT36 | 707 | after cine 12 (arg 6), not in activity play | Angelica wants all of the nickels in the park and she's making Phil and Lil collect them. Quickly search the park gathering as many coins as you can before time runs out. Be on the lookout for a gold bonus coin! |
| HINT38 | 709 | after cine 27 with tries left (arg 1) | Now try again. |

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) | end (0x8007737c) |
|---|---|---|---|---|---|
| 12 | Gold Rush Intro | the level start | none | hide 609, 600, 117, 35, 34 | nothing (card 707) |
| 26 | Win Bonus 5 | 0x80079b38, all coins | none | default | won, 0x80076d6c(17, 1) |
| 27 | Lose Bonus 5 | ForceCry's callback 0x80079bec, phase 1 | none | hide 609, 600, 117, 35, 34 | 0x80076d6c(17, 0), 0x80079c10 (Lil's round) |

Cine table flags (0x800b8bd0 + 16 x cine + 4) are 0x200 for all three.
Seen lengths (oracle VSyncs): 12 1326, 26 429, 27 683.

## Checked in the oracle

The level 21 probe copied to the scratch folder as l17probe, with a
level 17 line (+0x1a9, 0x800e2740, 0x800e2744, +0x220, 0x800c4658, ids
600, 608, 609, 1000, 34, 35, 117, coins taken and placed). Presses start
300, cross 500, 650, 800, 1650, 1800; main's level argument patched
(0x80018824 = `li a0, 17`), 0x800c5678 = 17 at the level runner, cards
skipped (0x8004472c = 0x1000004a), 0x80018598 made to return 5 (only
before the hub loads, so the hub got Tommy). The start cine was kept.
Coins were taken by putting Phil 600 from a spot on each side in turn,
facing it, and holding Up.

- Start: cine 12 1819-3145, card 707 (arg 6), EpisodeStart, the setup,
  respawn phase 1, 170 s, tries 2, the counter, 62 coins placed, the
  gold coin at a group 0 spot, the cine actors hidden.
- Rooms read from RAM.
- All 62 coins taken by walking: the 62nd at 7819, the clock's end with
  sound 117, cine 26, 3 bars.
- The gold coin taken by walking: its icon; 4 bars.
- Phil's loss (clock poked): no cry, cine 27, card 709, tries 2 -> 1, the
  swap to Lil where cine 27 left her, coins and gold coin placed again,
  phase 2.
- Lil taking coins after the swap.
- Lil's win (count and clock poked): the swap back to Phil, cine 26, 4
  bars from Phil's gold coin, the hub at (5775, 779, 19403), +0x34,
  +0x38 and +0x48[17].
- Lil's loss (clock poked): the cry, tries 1 -> 0, the hub after 420
  VSyncs.
- Activity play (+0x21 poked): no card, no gold coin, the golf ambience,
  no bars, back to the Activities menu.
- Not checked: reaching 17 from the seventh main level (from the code);
  the Activities menu itself; the loading screens; a natural loss (the
  clock was always poked); the gold coin taken again by Lil; the camera
  height in rooms 1-8; the speech lines' content; Pause, Quit Level and
  Restart; what 0x800c4018 and the coins' +0xc0 handle do.

## For the port

The port has the Script trait, Phil as level 17's character, spawn
groups and collectibles held for runtime, world billboard sprites, kind
2 / 15 sounds (31), collect events, the level timer and
`Script::timer_expired`, hiding by id, level 20's runtime body swap,
ForceCry with tries, cards from level code, the follow camera with room
heights, `start_cine` 17 -> 12, `card_after` 12 -> 707, `ENDINGS` 26
(won), the bar award (3, or 4 by 0x80076ba8), the bottom-left bar
counter for 17-18 and activity play. Level 17 needs, in order:

1. A `level17` script. EpisodeStart 0x80079af0 and the round setup
   0x80079870: every coin taken out and placed at group 1's spots, the
   gold coin at a random group 0 spot (rand mod 100 against thresholds
   25 / 50 / 75 / 100), the counter, +0x1a9 = 0, 170 s, follow camera
   (1500 -> height 1024, 3000), song 0, respawn phase 1. End hooks of
   cines 12 and 27 hide 609, 600, 117, 35 and 34.
2. The count: a kind 2 coin adds one; the counter text is 62 minus the
   count; the 62nd sets the timer to one tick (so the clock's end, with
   sound 117, decides the win next frame). The counter HUD: icon model
   286 at (-192, -80) with the number at (-205, -93), top left. The gold
   coin (kind 15) does not count; in level 17 it adds icon model 240 at
   (-140, -60) once. Its "taken" must survive the swap and the
   re-placement (4 bars after Lil's win if Phil took it).
3. `Script::timer_expired` (0x80079b38): fewer than 62 -> ForceCry; else
   (Lil: swap back to Phil first) cine 26; free the gold icon.
4. Level 17's ForceCry: in phase 1 no cry animation at all (the port's
   `Game::cry` always plays the cry before `Script::cry`, and counts a
   try only when the script declines): health 255, a try gone, cine 27,
   back to the respawn. In phase 2 the normal cry, then the hub (tries
   at 0), with no cine and nothing recorded.
5. Cine 27's finish: `card_after` needs 27 -> 709 only with tries left
   (the port has 51 and 65 -> 709 only); 0x80076d6c(17, 0) (+0x34 bit 17
   outside activity play); then 0x80079c10: free the counter and the
   icon, swap the player into Lil (id 609, char 4, her own gait) where
   cine 27 left her, Phil becomes a hidden idle NPC, the round setup
   again, respawn phase 2.
6. Activity play: no card 707 after cine 12 (the port's `card_after` 12
   is unconditional); the gold coin neither placed at load nor by the
   setup (it stays at its LP spot); the ambience by character
   (0x800818e0: golf's 0x8008168c for Phil, 0x80081758 for Lil). The
   original returns to the Activities menu (0x8003b7bc), not the title.
7. Main game: the gold coin placed at load and the 62 coins too (LP
   +0x4a = 1); the setup places them all again, so the port may place
   them only in the setup. The win: no puzzle piece, +0x38 bit 17, the
   hub spot (5775, 779, 19403). The level is offered once.
8. Small things: the ambience 0x800813a0 (speech 0-4, (rand & 255) x 8 +
   900 dt); rooms 1-8 CameraSwitch kind 8 (height 1600), allowed in
   17-18 while +0x30 bit 8 is set; no Damage in level 17; the cry's tear
   particles (model 1008) in levels 2 and 17; Lil's health 0 after the
   swap needs nothing (nothing hurts); 0x800c4018 = 6500 (unknown).

## Unknown

- What 0x800c4018 is (6500 in level 17; no reader found, also not
  through gp 408); guess: a draw or fog distance for the open park.
- What the coins' +0xc0 handle is (0x8008a2a4(0x1011), the gold coin's
  0x1022; 0x8008a1f8 sets its +0x18 to 62, 0x8008a1b8 runs on each take);
  guess: a shared sparkle or sound.
- Room flag 16 on all ten rooms; the LP kind 3 and kind 99 records.
- What speech 0-4 say; what model 1008 is (tears, guess).
- Card arg 6 and 1.
- Whether the gold coin at (1956, 0, 27889) can be reached in activity
  play (it is in room 0's list; guess: under the ground).
- Where cine 27 leaves Lil from natural play (seen once: (-3657, -342,
  10523)), and the cines' lengths from natural play.
