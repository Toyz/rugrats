---
title: Level 4, 7 Voyages of Cynthia
status: partial
discs: US
covers: US SLUS_006.50:0x8007e058, US SLUS_006.50:0x8007e198, US SLUS_006.50:0x8007e278, US SLUS_006.50:0x8007e2b0, US SLUS_006.50:0x8007e2e8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80078dd8, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80077ee4, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80018598, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8004fd20, US SLUS_006.50:0x80072d38, US SLUS_006.50:0x8004d138, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x8004f83c, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x8005e1f4, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x80050770, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80084084, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x80049af4, US SLUS_006.50:0x80048a44, US SLUS_006.50:0x80049ab0, US SLUS_006.50:0x80048804, US SLUS_006.50:0x8004ee00, US SLUS_006.50:0x80048974, US SLUS_006.50:0x80080934, US SLUS_006.50:0x8006fe04, US SLUS_006.50:0x8008a668, US SLUS_006.50:0x80078e3c, US /DATA/DB04/DB04LP.BIN, US /DATA/DB04/LEVEL04.BIN, US /DATA/HINTS
worklog:
---

# Level 4, 7 Voyages of Cynthia

Everything level 4 needs to be played to its end: Spike as the player,
the walk through the sewers to the garbage dump, the brown water that
hurts, the rats, the bones, the 130 s search for Cynthia at a random
spot, and the win and loss paths. Read from the pseudo-code and the
disassembly; the phase changes, the water damage, a rat's bite, a bone,
the timer running out, the win and the loss were run in the oracle (see
Checked in the oracle). Builds on [level 3](level-03.md) and
[level 2](level-02.md) (NPC modes, posts, events, ForceCry, cines,
cards), [gameplay](../engine/gameplay.md) and [objectives](../engine/objectives.md).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
"id" is the registry id (LP +0x2a). "char" is the character (+0x15c =
template +0x10). A VSync is 1/60 s; the oracle's frame counter counts
VSyncs.

## The goal in one paragraph

Spike looks for Angelica's doll Cynthia. The player is Spike (char 6) for
the whole level (0x80018598). Cine 28 "Episode3 Intro", then card HINT14
(file 685): search the sewers, watch out for the rats, stay out of the
icky brown water, eat bones to stay healthy. Spike walks from the start
(room 34) through the sewers. Room 26 and room 21 only move the respawn
on (phases 2 and 3). From phase 2 on, standing or walking in brown water
(floor flag 1) costs health. Rats (char 39, ids 444-448) patrol posts
and bite (3). Bones heal 63. Entering room 40, the garbage dump, starts
phase 4: card HINT15 (file 686, "Time's running out!"), a 130 s clock
and a Cynthia portrait on the HUD. Cynthia (id 34) waits at one of six
random spots in the dump (kind 7 group 0). Coming within 2000 of her
plays cine 48 "Cynthia End" and wins the level. When the clock runs out
Spike cries, loses a try, goes back to the dump entrance, the clock
starts again and Cynthia moves to another spot. No tries left: back to
the hub.

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd0 | u8 | phase: 1 start, 2 after room 26, 3 after room 21, 4 the dump (timer) |
| 0x800e5830 | s32[3] | respawn position for ForceCry (0x80078d60) |
| 0x800e583c | u32 | 1 when a respawn is set |
| 0x800c4cc8 | u16[3] | respawn angles |
| 0x800c4430 | s32 | the level timer, dt (0x80078dd8(s) = s * 300) |
| 0x800c4ba0 | s32 | 500 at EpisodeStart (an object sound countdown, see level 3) |
| 0x800c42dc, 0x800c42e8 | u32 | VSync of the last water damage, standing / moving |
| 0x800c42e4 | u32 | footstep counter for the water splash sound |
| 0x800c42c0 | u32 | 1 when nothing is touched (re-arms the touch sounds) |

The phase is only written through 0x80078d60: 1 by EpisodeStart, 2 by
0x8007e278 (only when below 2), 3 by 0x8007e2b0 (only when below 3), 4 by
Gameplay script 2 (0x8007e198). Each also sets the respawn to the
player's position and angles at that moment. There are no CheckPoint
rooms in DB04 (no kind 6 record has byte +4 set).

## Data in DB04

### The player and the characters (LP kind 1)

Level 4's playable character is 6 (0x80018598, also level 11). The player
init moves onto the LP record whose template's shape is the character
(0x800454dc), so the player is Spike, LP 260. The other character
records are NPCs built by 0x800497dc and 0x80081bac.

| LP | template | char | id | position | health | range | speed index | first post | what |
|---|---|---|---|---|---|---|---|---|---|
| 260 | 9 | 6 | 14 | (-114202, -688, 64250) | | | | | Spike, the player |
| 262 | 11 | 1 | 1492 | (-93887, 1130, 74057) | 5 | 2000 | 4 | | Tommy, idle in room 56 (guess: for cine 48) |
| 45 | 2 | 2 | 1776 | (-93831, 1222, 75517) | 250 | 5000 | 3 | | Chuckie, idle in room 56 |
| 29 | 1 | 3 | 600 | (-95832, 946, 74658) | 250 | 2000 | 4 | | char 3, idle in room 56 |
| 189 | 882 | 39 | 444 | (-78082, -1056, -17122), room 23 | 100 | 3000 | 1 | 25 | rat |
| 190 | 882 | 39 | 445 | (-135160, -1030, -4371), room 6 | 100 | 3000 | 1 | 10 | rat |
| 191 | 882 | 39 | 447 | (-140152, -1048, 68419), room 38 | 100 | 3000 | 1 | 5 | rat |
| 192 | 882 | 39 | 448 | (-224214, -1070, 68170), room 40 | 100 | 3000 | 1 | 4 | rat |

Rats are char 39 "rats" from HINT14 (guess from the card). 0x80081bac
case 37 (char 39): 0x8008395c(npc, first post, player), so mode 9, enter
callback +0x48 = 0x80049e18, +0x6d = 3 (the template's 15 is
overwritten), +0x688 |= 0x4c08. Template 882 has flag 0x10000000, which
adds +0x688 bit 0x2000 (0x80081d64); seen in the oracle: +0x688 0x6c08.
They arm themselves by distance (template +0x34 = 15000), like the
lobsters of level 3. Chars 1, 2, 3 get nothing from 0x80081bac and no
level code moves them.

### Cynthia and the other objects that matter

| LP | template | class | id | position | what |
|---|---|---|---|---|---|
| 64 | 105 | 0 | 34 | (-115121, -1490, 67041) | Cynthia (shape 661); moved to a group 0 spot at EpisodeStart |
| 304 | 105 | 0 | 52 | (-97270, -1490, 70958), room 56 | a second Cynthia model; no level code (guess: cine prop) |
| 63 | 104 | 0 | 51 | (-35057, -6432, -6567) | no level code (guess: cine prop) |
| 22-28 | 835 | 12 | | (-146393..-144505, 0, 25167) | Reptar bars 0-6, one each; LP +0x48 = spawn group 7, 1..6 |
| 71 | 1017 | 12 | | (-145484, 0, 27171) | bones: LP +0x46 = 7 copies, kind 14, heal +0x6d = 63; spawn group 8 |
| 59, 60, 61, 62, 188, 296, 297 | 1182, 1182, 1183, 1184, 653, 425, 1185 | 11 | | sewer rooms 22, 7, 23 | movers (guess: the "floating boxes and rafts" of HINT14) |
| 109-114 | 661 | 0 | | y -554, rooms 27-33 | hazard: +0x6d 15 |
| 46, 47 | 666 | 0 | | (-120833, -2313, 8181), (-134183, -2313, 9259) | hazard: +0x6d 15 |
| 115-122 | 1192 | 0 | | y -2435 | touch sound 60 only |
| 263 | 543 | 0 | | (-244131, -1047, 94470) | oil drum; cloned twice (below) |
| 91-93 | 1189 | 15 | | (-144790, -951, 38001), (-165429, -951, 32062), (-107243, -781, -8892) | not armed in level 4 (below) |

Class 12 templates (collectibles) are built by 0x80049af4: LP +0x46
copies (D_800c4b94), each with 0x80048a44 (object class 11, kind
template +0x26). Bars are kind 3, the bones kind 14 (0x80048cd0: +0xa9 =
14). Touching a kind 14 object runs 0x800514b0 case 13: sound 29, player
health += +0x6d (template 1017 +0x32 = 63), at most 255, meter update
(0x8008c124). Template 1141 (LP 73-81, kind 16) has no touch callback
(0x80049244 sets +0x58 = 0): decoration.

Template class 11 objects are movers, built by 0x80049ab0 with
0x80048804 (object class 10). The think 0x8004ee00 steps the object by
the template's per-axis step (+0x18, +0x1a, +0x1c) until it has gone the
template's range (+0x1e, +0x20, +0x22) and back; direction bits in
+0xbe. Template flag 0x10000000 sets object bit 0x20000 (guess: the
player rides it, see the 0x20000 test at 0x8004f130).

| template | step (x, y, z) | range (x, y, z) |
|---|---|---|
| 425 | (-40, 0, 0) | (-5120, 0, 0) |
| 653 | (0, 0, 40) | (0, 0, 10000) |
| 1182 | (0, 0, -40) | (0, 0, -4096) |
| 1183 | (-40, 0, 0) | (-7168, 0, 0) |
| 1184 | (0, 0, 40) | (0, 0, 7168) |
| 1185 | (0, 0, -40) | (0, 0, -5120) |

Template class 10 (template 706, LP 82-90) is 0x80048400: timed animated
scenery (think 0x8004e0f4).

### Level 4 setup in the generic object init (0x8004691c case 3)

0x8004691c switches on the level (table 0x8001369c); level 4 goes to
0x80047e30:

1. Id 34 (Cynthia): trigger radius +0xa0 = 2000, enter callback +0x48 =
   0x8004d138, +0x6c = 0. LP 64 has +0x42 = 1, which made it a door to
   level 1 a few lines earlier (+0x54 = 0x8004f78c, +0x6c = 1,
   0x80046cc4); +0x6c = 0 takes the level away again.
2. Template 203 at (-226738, y, 78666) (LP 123): y = -400.
3. Template 543 at (-244131, -1047, 94470) (LP 263): two copies at
   (-282981, -2313, 64739) and (-281974, -2318, 64044) ("Created 2
   oildrums").
4. Template 1189: object +0x00 |= 0x2000, template flag 8 (no object
   collision).
5. Templates 425 and 1185: box +0x02 = -666; templates 653, 1182, 1183,
   1184: box +0x02 = -450 (s3->0x88->0xc, guess: the top of the
   collision box, so Spike can stand on the movers).

Class 15 (template 1189, 0x80048974, object class 12) in level 4 gets no
enter or leave callback (+0x48 = +0x4c = 0, 0x80048a14), so its post
walker (0x8004dba0) is never scheduled.

Generic walk-into hazards (walk state, 0x8005e5b0): a touched object
that is not pickable and not AI with +0x6d > 0 is Damage(player, 0,
+0x6d). In DB04: templates 661 and 666 (15). Template 692 has +0x6d 2
but template flag 8 (guess: never touched).

### Kind 7: spawn groups (0x800c4bac, 16 bytes each)

20 records, copied by the loader (0x80045d80) to x, y, z, then byte +0x0c
group, +0x0d threshold (a running percentage; 100 ends a group), +0x0e
post link, +0x0f used flag.

| entries | group | spots | use |
|---|---|---|---|
| 0-5 | 0 | 16, 33, 50, 66, 83, 100 | Cynthia's six hiding places |
| 6-12 | 1-7 | one each | Reptar bars (one per group) |
| 13-19 | 8 | 14, 28, 42, 57, 71, 85, 100 | the seven bones |

Group 0 (y as in RAM after EpisodeStart):

| entry | position | room |
|---|---|---|
| 0 | (-255997, 73, 31430) | 46 |
| 1 | (-217081, 100, 17411) | 47 |
| 2 | (-247200, 1185, 63672) | 49 |
| 3 | (-209405, -1622, 90074) | 42 |
| 4 | (-276590, -2368, 60274) | 44 |
| 5 | (-217069, 825, 29620) | 48 |

In level 4 the loader sets entry 1's y to 900 and entry 5's y to 825
after each kind 7 record (0x80045e08); EpisodeStart sets entry 1's y to
100 (0x8007e0a4). The LP values are 1215 and 644.

0x8004fe14(list, group, count) places objects at spots:

1. Find the group: skip `group` runs of entries ending with threshold
   100; the group ends at the next 100.
2. Per object: r = (rand & 255) mod 100 (0x8002fb7c). Take the first
   unused entry of the group whose threshold is above r. If none, retry
   with r = 0; give up after five rounds (the object stays where it is).
3. The object takes the spot's x, y, z (class 1: 0x800825b0, snapped to
   the floor; else 0x8004a34c, the room update only). The entry gets used
   (+0x0f |= 1) except for group 10 in level 12. Class 11 objects keep
   the entry in +0xaa and the spot in +0xc4..+0xcc; AI objects take its
   post link as their first post.

The loader (0x80045f34) calls it at the end of loading for every
collected group of objects with a spawn group (LP +0x48), unless state
+0x21 (activity play) is 1 and the object is template 835, 863, 1218 or
1219. In the oracle: (7, 1 object), (1, 1) .. (6, 1), (8, 7). So the bars
and bones in DB04 do not stay at their LP positions (y 0, all in a row).

0x8004fd20(list, count) takes objects out of their room lists and, for
class 11 objects, frees their entry. Cynthia is class 0: her used entry
is never freed. Each re-hide moves her to a spot she has not used yet
(guess from the code: after six spots she stays put).

### Rooms (LEVEL04.BIN, LP kind 6)

Room boxes read from RAM in the oracle (room +0x10..+0x28).

| room | x | z | kind 6 | role |
|---|---|---|---|---|
| 34 | -116736..-111616 | 46080..80896 | song 1 | the start |
| 27-33, 52, 53 | -125952..-48128 | -7168..57344 | 28, 33 song 1 | the first sewer (pipes, grates; seen) |
| 26 | -68608..-49152 | -15360..-6144 | song 2 | phase 2 (0x8007e278) |
| 0-25, 35, 36, 38 | -215040..-68608 | -18432..69632 | 6 song 2, 36 song 2 | brown water floors (flag 1 where scanned) |
| 21 | -149504..-130048 | 23552..38912 | song 3 | phase 3 (0x8007e2b0) |
| 37, 39 | -221184..-215040 | 38912..69632 | flag 16 | between the water rooms and the dump |
| 40 | -237568..-221184 | 57344..70656 | script 2, song 4 | the dump entrance: phase 4 |
| 41-51 | -290816..-203776 | 8192..99328 | flag 16; 41 song 2, 50 and 51 song 4 | the garbage dump, Cynthia's spots |
| 56-58 | -100352..-74752 | 67584..83968 | | the babies and the second Cynthia (guess: the cine stage) |

All kind 6 records of the dump (37, 39-51) set room flag 16 (the light
and camera pass 0x80088c80, see [gameplay](../engine/gameplay.md)). Rooms 0-3, 6-25,
35, 38 have room flag 1 (meaning unknown). EpisodeStart's room sound
bytes (0x80078e3c(34, 1), (41, 2)) change nothing: both rooms already
have a song.

### Posts (LP kind 2, 36 records)

| posts | links | rat |
|---|---|---|
| 1-4 | a line 1-2-3-4 (ends turn back) | 448, from post 4 |
| 5-8 | a line 5-6-7-8 | 447, from post 5 |
| 9-23 | a loop | 445, from post 10 |
| 24-36 | a loop | 444, from post 25 |

No post has flags; y is about -1070 in all.

## Start (EpisodeStart, 0x8007e058)

Installed for level 4 by 0x8007871c (case 4). Level start: player init,
moved onto LP 260 (Spike); cine 28 "Episode3 Intro" (0x80074958, no
start or end hook in 0x80072ef0 / 0x800732d4); CineFinish shows card 685
(HINT14, arg 6) (0x800783d8 case 27); then EpisodeStart:

1. Id 34: use +0x54 = 0x8004f83c with +0x6a = 48 (PostCine(48) on
   ACTION); kind 7 entry 1's y = 100; placed by 0x8004fe14(&cynthia, 0,
   1) (group 0).
2. Respawn = the player's position and angles, phase 1 (0x80078d60).
3. Room sound bytes 0x80078e3c(34, 1) and (41, 2) (no effect, above).
4. Spike's animation slots 10, 11, 12 and 13 each get the ids 902, 903,
   901, 900 (0x8008a668(6, list, slot, 1); what the slots are is not
   known).
5. 0x800c4ba0 = 500.
6. A positional sound: 0x8006fe04(2, (-83212, -688, 42873), 1024, 10000)
   with volume 127 (an emitter slot at 0x800e3218; guess: running water).

Cine 28 leaves Spike at (-114036, -688, 61842) (oracle), not at LP 260.
Nothing is hidden and nothing is scheduled; the rats arm by distance.

## Phases 2 and 3: rooms 26 and 21 (0x8007f498 case 1)

Level 4's room change (0x8007f498, table 0x80015894 index 1):

- Room 26: 0x8007e278: phase 2 and respawn here, when the phase is 0 or
  1.
- Room 21: 0x8007e2b0: phase 3 and respawn here, when the phase is below
  3.

Nothing else changes. The phase matters for the water (below) and for
ForceCry's respawn.

## Phase 4: the dump (Gameplay script 2, 0x8007e198)

Room 40 has kind 6 flag 64 with script 2. The Gameplay handler
(0x80077ee4) runs script 2 = 0x8007e198 and clears the room's flag 64.

0x8007e198, only when the timer 0x800c4430 is 0 or less:

1. A HUD sprite: 0x8008f8f4(287, scale (32, 64, 1024), position (-60,
   ((0x800c4a06 * 171) >> 11) - 90, 0), 0), then 0x80090004(0, &item,
   sprite) with colour 0xffffff. In the oracle: Cynthia's portrait left
   of the clock, top of the screen. It stays for the rest of the level.
2. Timer 130 s (0x80078dd8(130): 39000 dt).
3. Card 686 (HINT15, 0x80044434(686, 0)).
4. Respawn = here, phase 4.

The timer runs in 0x8007f5a8 (each frame, minus dt) and draws the clock
(sprite 375 and the seconds, "%ld"). At 0: sound 117 at the player and
PostTimerExpired with level 4's callback 0x8007e2e8 (table 0x800158cc
index 2), which calls ForceCry (0x8007fd0c).

## The brown water

A floor with COL record flag 1 (+4 bit 0) is water. Level 4 code in the
player's state handlers:

- Standing (0x8005d11c, at 0x8005d8e0): on water, standing on no object (+0xf8 =
  0), level 4, state +0x30 bit 32 clear, phase not 1, and 36 VSyncs since
  the last (0x800c42dc): Damage(player, 0, 5).
- Walking or running (0x8005e1f4; state 2's 0x8005ec7c calls it): on
  water and on no object: splash particles 0x80050770 each frame
  (0x8005ea9c); then the same test with 0x800c42e8 (0x8005eab0):
  Damage(player, 0, 3).
- Running footsteps (0x8005e85c, animation event +0xe0 bit 256): Spike
  (state +0x08 = 6) on water in level 4: sound 57 when the counter
  0x800c42e4 is 0 (then counter = 4), else counter - 1.
- Landing (0x8005b2d8, state 10): on water in level 4: sound 57.
- The floor flag 16 dust (0x80050b5c) is skipped in level 4 (0x8005d7f8,
  0x8005e4b0, 0x8005e59c).

36 VSyncs is 0.6 s: 8.3 health a second standing, 5 walking. Phase 1
(the first sewer and the water rooms before room 26) is safe. In the
oracle the room 40 floor is 33 (flags 1 and 32).

Flags 32 and 64 (+4 bits 5 and 6) are how deep a character sinks into
its floor as drawn. The character draw (0x8003205c) lowers each part by
298 times (flags & 96) >> 5 while it draws, and puts the parts back
after. A sunk character gets no shadow (0x8003414c skipped). None sink
while standing on an object (+0xf8). Only level 4 has such tiles: 54
water tiles of depth 1 (flags 33), the deep water around (-229000, 256,
64000), where the babies wade 298 lower.

Damage (0x80077864) has no level 4 routing (only levels 2, 3, 16). With
+0xf8 set and a damage above 0 nothing is taken (0x80077900).

Falls: the land code (0x8005b700) skips fall damage for template 9, so
Spike never takes it.

## Other level 4 code in the player update (0x8005a068)

0x8005ad88: when the player touches (+0xf4) template 666 or 1192: sound
60 at the player; template 661: sound 61. Once per touch: 0x800c42c0 is
set again only when nothing is touched.

## Rats: mode 9 (0x80084084)

Mode 9 as for the lobsters ([level 3](level-03.md#lobsters-mode-9-0x80084084)):
chase the player while the current post is within 2 x range (6000) and
the player within range (3000), else walk the posts; bite within 2 x the
hierarchy half size with heights overlapping. No sound 51 (that is char
23 only).

The bite, mode 5 (0x800846e4): state 19, animation 37 (blend 4096),
Damage(player, +0x678, +0x6d = 3), sound 168 at the rat (template 882),
then mode 1. Mode 1 waits for the animation (with event 0x2000: 300 +
(rand & 3) * 300 dt), then mode 0 goes back to mode 9. In the oracle a
rat next to Spike bit every 16 VSyncs.

Spike has no attack in this level and nothing hurts the rats (template
882 touch +0x0a = 0).

## Winning: Cynthia

Two ways, both PostCine(48):

- The trigger pass (0x80072d38(0), five list entries a frame): Cynthia is
  on the trigger list (enter callback), not active (+0x00 bit 1 clear),
  and within 2000 of the player (2D, 0x8001f57c): +0x48 = 0x8004d138
  calls 0x8004f78c(cynthia, player). With +0x30 bit 16 clear and +0x6c =
  0 it stops the music (0x8006ee14) and posts Cine(+0x6a = 48); +0x6c = 0
  means no SwitchEpisode. Nothing sets bit 1, so this repeats every few
  frames while Spike is near (seen: at 2203, 2213, 2221, 2229); the first
  one plays the cine.
- ACTION (Triangle) with Cynthia touched: +0x54 = 0x8004f83c, also
  PostCine(48). Not needed: the radius is larger than a touch.

Cine 48 "Cynthia End" has no start or end hook. After it (0x8007737c
case 47): exit code 0x800c4918 = 1, 0x80076d6c(4, 1) (level 4 played
and won, its puzzle piece, the main-level count; see
[objectives](../engine/objectives.md)). The port's `ENDINGS` already has 48. In
the oracle the puzzle piece screen followed.

Cynthia is placed from the start, so reaching her before room 40 would
win without the timer. Whether the dump can be entered without passing
room 40 is not known.

## ForceCry (0x8007fd0c, level 4 case)

Called by Damage at 0 health and by the timer (0x8007e2e8).

1. Cry (0x8007f920) always (0x8007ff90, s1 = 1). In the oracle the rest
   of ForceCry ran 420 VSyncs later; state +0x30 bit 32 and player +0x188
   bit 64 are set meanwhile.
2. Tries: progress +0x23 counted down (3 at level start).
3. Health 255.
4. Tries left and a respawn set: in phase 4 (0x80080224): timer 130 s
   again (0x80078dd8(130)); Cynthia out of her rooms (0x8004fd20(&c, 1))
   and placed again (0x8004fe14(&c, 0, 1)). If the tries byte is 0 here
   (it was 0 before ForceCry): 0x80076d6c(4, 0) instead. Then the player
   goes to the respawn (0x8007fbf8).
5. No tries left: SwitchEpisode(player, 1) (0x800781f8, back to the hub).
   No level record, no cine.

Respawns: phase 1 the start; phase 2 where Spike entered room 26; phase
3 room 21; phase 4 room 40. Phases 1-3 only respawn; the timer is not
running there.

What hurts Spike: brown water from phase 2 (5 or 3 per 0.6 s), rat bites
(3), templates 661 and 666 (15). Bones heal 63.

## How the level ends

- Won: within 2000 of Cynthia, cine 48, 0x80076d6c(4, 1), exit code 1.
- Lost: the last try (ForceCry): SwitchEpisode(1). There is no "lost"
  cinematic for level 4.

## Hint cards

| card | file | shown | text (abridged) |
|---|---|---|---|
| HINT14 | 685 | after cine 28 (arg 6) | Search the sewers for Cynthia; watch out for rats; floating boxes and rafts keep Spike out of the icky brown water; crunchy bones keep him healthy |
| HINT15 | 686 | Gameplay script 2, room 40 (arg 0) | Time's running out! Race around the garbage dump and find Cynthia before the counter runs out |

The port's card table (front.rs card_after) has 28 -> 685. Card 686 is
shown by level code, not after a cine.

## Per-frame ambience (0x80080934)

0x800b9618[4]. A countdown 0x800c4cc0 minus dt; below 0: speech
0x800b9400[(rand & 255) mod 6] = one of 1110, 1112, 1116, 1117, 1119, 1122
(0x8008da4c), countdown + 1000. Seen every 200 VSyncs. Speech only.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, hooks with arguments,
object dumps, screenshots): presses start 300, cross 500, 650, 800, 1650,
1800; main's level argument patched (0x80018824 = `li a0, 4`),
0x800c5678 = 4 at the level runner, cards skipped (0x8004472c =
0x1000004a). The playable character must be poked too: main computes
state +0x08 from the old level (0x80018780), so 0x800c5680 = 6 (without
it the level loads Tommy and he falls forever). Level start cine skipped
(0x80074958 returns 0) except in the first run.

- With cine 28: cine at 1819, card 685 and EpisodeStart at 2010; Spike at
  (-114036, -688, 61842), phase 1, respawn there, tries 3, +0x30 0x100.
  Spawn group calls at load: (7, 1), (1, 1) .. (6, 1), (8, 7); EpisodeStart
  (0, 1) put Cynthia at entry 1 (-217081, 100, 17411), room 47.
- Rats: mode 9, +0x688 0x6c08, +0x6d 3, speed 4, range 3000, health 100.
- Spike poked into room 26 (2200): 0x8007e278, phase 2. Into room 21
  (2400): 0x8007e2b0, phase 3. Into room 40 (2600): script 2, sprite 287,
  timer 39000, card 686, phase 4; the floor there is 33 and Damage(player,
  0, 5) came every 36 VSyncs (2603, 2639, 2675 ..).
- Timer poked to 600 at 2800: TimerExpired posted at 2919, ForceCry and
  the cry at 2921, tries 2; at 3341 timer 39000 again, Cynthia moved to
  entry 2 (-247200, 1185, 63672) room 49, Spike at the room 40 respawn.
- Phase 1 floor scan (rooms 39, 37, 22, 6, 23, 0, 12, 15, 24, 19, 10):
  flag 1 floors in the water rooms, no Damage posts.
- Spike next to rat 445: Damage(player, player, 3) from 0x80084724 every
  16 VSyncs, health 255 to 144 by 2800; the rat in mode 1 / mode 9.
- A jump into the bone at entry 13 (-115290, -2105, -3590): 0x800514b0,
  sound 29, health 144 to 207.
- Running on a flag 1 floor in room 28: sound 57 from 0x8005e8b4.
- Spike poked 500 from Cynthia (2200): 0x8004f78c on her every 8-10
  VSyncs, PostCine(48); cine 48 from 2803 to 2903, then 0x80076d6c(4, 1),
  exit 1, the puzzle piece screen. Her +0x54 was 0x8004f83c, +0x6a 48,
  +0x6c 0.
- Tries poked to 1, timer run out in room 40: ForceCry, tries 0, at 2781
  SwitchEpisode(player, 1); no 0x80076d6c; level 1 loaded.
- Not checked: the movers (rafts) moving and carrying Spike, the
  templates 661 / 666 hazards (walking at LP 111 put Spike in state 9
  without a Damage post; not explained), ACTION on Cynthia, more than one
  re-hide.

## For the port

The port has a Script per level, hiding by id, cinematic-object use,
doors, room flags 2/4/32/64/128, cameras, NPC modes 0, 1, 3, 4, 6, 7, 9,
11 and riders, posts, carrying and throwing, hazards, the level timer,
gauges and the meter, ForceCry with tries. Level 4 needs, in order:

1. Spike as the player. The port starts every level with Tommy
   (game.rs `TOMMY`). Use the character from 0x80018598 (6 for levels 4
   and 11): the player is LP 260 (template 9, hierarchy 6), turn rate
   298 (`player::SLOW_TURN`), camera aim 400 above him (0x800c3f58)
   with the level's 512 eye height, no fall damage for template 9
   (0x8005b700). Spike's animation slots 10-13 (EpisodeStart). Cine 28
   leaves him at (-114036, -688, 61842).
2. Kind 7 spawn groups (0x8004fe14): read LP kind 7 into groups with
   thresholds and used flags; at load place class 12 collectibles by LP
   +0x48 (bars to groups 1-7, the seven bones to group 8), with level 4's
   y fixes (entries 1 and 5). LP +0x46 is the number of copies (seven
   bones). The port now places collectibles at their LP positions.
3. Cynthia: undo the door (LP 64 +0x42 = 1 must not send Spike to level
   1); trigger radius 2000 (2D) calling the door function with +0x6c = 0,
   so PostCine(48) only; ACTION use 0x8004f83c as a second way. Cine 48
   is already in `ENDINGS`.
4. Floor flags from COL: `collide::query` should return record +4. Water
   (flag 1) damage 5 standing / 3 moving per 36 VSyncs (0.6 s) in phases
   2-4, not on an object, not while +0x30 bit 32; splash particles,
   footstep and landing sound 57.
5. A `level4` script: phase, respawn, tries; EpisodeStart 0x8007e058;
   room changes 26 and 21 (0x8007f498 case 1); Gameplay script 2 (room
   40): timer 130 s, card 686, HUD sprite 287, phase 4; TimerExpired =
   ForceCry; ForceCry's phase 4 case (timer again, Cynthia re-placed in
   group 0, never on a used spot).
6. Collectible kind 14 (and 4): heal by +0x6d (bones 63), at most 255.
   The port's `touch` only plays the sound and counts bars. Bones float
   at their spots (y -2105 ..); Spike jumps for them.
7. Rats: mode 9 exists (`Mode::Prowl`); add +0x6d = 3 for char 39 (not
   the template's 15), +0x688 0x4c08, and the bite sound 168 for template
   882.
8. Movers (template class 11, object class 10, 0x8004ee00): back and
   forth by step and range per axis, with the player riding (object bit
   0x20000, guess), and the box top tweaks (-666, -450) of 0x80047fa8.
9. Small things: touch sounds 60 / 61 (0x8005ad88); speech ambience
   (0x80080934); the emitter sound 2 (0x8006fe04); the two oil drum
   copies; template 203's y; template 1189 not armed.

## Unknown

- What Spike's animation slots 10-13 and ids 900-903 are.
- What char 3 (id 600) is, and what ids 51, 52, 600, 1492, 1776 do
  (cine 48 props, guess).
- Room flag 1 (rooms 0-3, 6-25, 35, 38) and COL floor flag 32 (room 40).
- Whether the dump can be reached without room 40 (Cynthia before the
  timer).
- Why walking into LP 111 (template 661) gave state 9 and no Damage.
- Object bit 0x20000 and +0xbe bits 64 / 128 / 256 of the movers;
  whether Spike rides them.
- Template 706 (class 10, 0x8004e0f4) and template 1141 (kind 16): what
  they look like and do.
- The sprite 287 position formula (0x800c4a06 is 0 in the EXE).
- How long cine 28 and cine 48 really are (the oracle may cut them short:
  191 and 100 VSyncs).
