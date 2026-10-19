---
title: Hinged and sliding doors
status: partial
discs: US
covers: US SLUS_006.50:0x800496a8, US SLUS_006.50:0x8004860c, US SLUS_006.50:0x8004965c, US SLUS_006.50:0x8004845c, US SLUS_006.50:0x80049708, US SLUS_006.50:0x80048714, US SLUS_006.50:0x8004f4a0, US SLUS_006.50:0x8004f868, US SLUS_006.50:0x8004f6d8, US SLUS_006.50:0x8004e458, US SLUS_006.50:0x8006baa8, US SLUS_006.50:0x8006b6d0, US SLUS_006.50:0x8006add8, US SLUS_006.50:0x80046570, US SLUS_006.50:0x80069c40, US SLUS_006.50:0x8006a714, US SLUS_006.50:0x8006abbc, US SLUS_006.50:0x80031b84, US SLUS_006.50:0x80020c30, US SLUS_006.50:0x8005f584, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8001ce48, US /DATA/DB00/TEMPLATE.BIN, US /DATA/DB01/DB01LP.BIN, US /DATA/DB01/DB01MOD.BIN
worklog:
---

# Hinged and sliding doors

Doors that Triangle opens in place, with no level change: the hub's
playpen gate, back door, yard doors and refrigerator, and the same props
in the house levels. Class 4 doors swing about a hinge; class 3 doors
slide. Each use moves a door to its other end and stops (the playpen
gate never quite stops, below); nothing closes by itself. A door that would swing or slide into the player waits. The
player collides with a door's model box at its current place. Read from
the pseudo-code and the disassembly; the hub's playpen gate, back door
and one yard door were run in the oracle (see Checked in the oracle).
Level doors (LP +0x42, 0x8004f78c) are in
[gameplay](gameplay.md#doors-0x8004f78c) and [level 1](../levels/level-01.md).

Units: dt is 1/300 s, 5 per VSync the frame took (10 a game frame of 2
VSyncs); 0x800c3f20 is dt x 4096 / 300 = 136 at dt 10 (both set by
0x8001ce48; seen). Angles are 4096 a turn, y down, heading 0 faces +z
and 1024 faces +x. "Frame" in the oracle is a VSync; thinks run once a
game frame (2 VSyncs). "Think" means one call.

## The classes

The class table 0x800b7de8 is picked by template byte +0x04.

| class | constructor | object | setup | think | use |
|---|---|---|---|---|---|
| 3, sliding | 0x8004965c | 216 bytes, +0x3c bit 2 cleared | 0x8004845c | 0x8004e458 | 0x8004f6d8 |
| 4, hinged | 0x800496a8 | 216 bytes, +0x00 \|= 32, +0x3c bit 2 cleared | 0x8004860c | 0x8004f868 | 0x8004f4a0 |
| 8, swinging | 0x80049708 | class 4, then 0x80048714 | 0x8004860c + 0x80048714 | 0x8004f868 | none |

Both setups first run the generic init 0x8004691c, which sets +0x3c from
LP +0x28 (0 in every door placement) and clears only the first 168 bytes
(0x80021b24(obj, 0, 42)). So +0xa8..+0xd7 hold leftovers unless the setup
writes them (seen: class 4's +0xbe was 0x531a, class 3's +0xb8 6 and
+0xd4 0x40). Both set +0xa6 = 2 and leave +0x44 (think) unscheduled.

+0x3c bit 2 is the door's state: clear = shut, set = open. Every door
placement starts shut (LP +0x28 is 0 in all of them).

## Setup

Template fields are read from TEMPLATE.BIN (56 bytes each, at 0x800c4300).

### Class 4 (0x8004860c)

| object | from | meaning |
|---|---|---|
| +0x44 | 0x8004f868 | think |
| +0x54 | 0x8004f4a0 | use |
| +0x00 | \|= 32 | the draw (0x80031b84) builds the matrix from +0x24 each frame instead of using +0x04 |
| +0xa8, +0xaa, +0xac | 0 | +0xa8 is the swing angle, added to the heading; +0xaa, +0xac unused |
| +0xb0 | template +0x18 (s16) | speed: angle units a second (step below) |
| +0xb2, +0xb4 | template +0x1a, +0x1c | unused by the hinged code |
| +0xae | template +0x1e | the shut angle (0 in every class 4 template) |
| +0xb6 | template +0x20 | the open angle (signed) |
| +0xd4 | 0x8002b8fc(template +0x22) | a sound handle started on open and stopped on close; template +0x22 is 0 in every door, so +0xd4 is 0 (seen) |
| +0xcc | 0 | the scheduler handle; non-zero while moving |
| +0xc0, +0xc4, +0xc8 | +0x24, +0x26, +0x28 (s16) | the placed angles |
| +0xd2 | 0 | the wait (class 8 only) |
| +0xd0 | template +0x24 (u8) | sound kind: 1 plays 14 / 15, else none |
| +0xb8, +0xba | 0 | easing (class 8 only), the target angle |
| +0xbc | template +0x26 | non-zero: opens either way (away from the player) |

+0xbe is not written (left over). It matters only with +0x00 bit 0x20000,
which class 4 never has.

### Class 3 (0x8004845c)

| object | from | meaning |
|---|---|---|
| +0x44 | 0x8004e458 | think |
| +0x54 | 0x8004f6d8 | use |
| +0xa8, +0xaa, +0xac | 0 | the slide offset (x, y, z) in the door's own axes |
| +0xb0, +0xb2, +0xb4 | template +0x18, +0x1a, +0x1c | step a think, halved (the think adds 2 x step) |
| +0xae | template +0x1e | the shut end: offsets summed (+0xa8 + +0xaa + +0xac) |
| +0xb6 | template +0x20 | the open end, same sum |
| +0xc0, +0xc4, +0xc8 | +0x18, +0x1c, +0x20 | the placed position (s32) |
| +0xcc | 0 | the scheduler handle |
| +0xd0 | template +0x24 (u8) | sound kind: 2 plays 16 / 17, else none |
| +0xbe | \|= 16 with template flag 0x10000000, \|= 8 with 0x20000000 | waits (auto doors only) |
| +0xd2 | 0 | the wait |
| +0x94 | template +0x2e, or 0 | the speech line, overriding LP +0x4c (66 for 843, else 0) |

Template flag 0x40000000 makes an automatic door: +0x48 (enter) =
0x8004f6d8, +0x4c (leave) = 0x80049e5c (unschedule), +0x54 = 0, +0x00 |=
0x20000. Only 770 (DB24) uses it; 843 has it but is set up again below.

Template 843 with id not 8801 (all of DB05's): enter 0x8004d078, think
0x8004e7fc, use 0, leave 0x8004d0b0. Those doors rise once when the
player comes within 5000 and are in [level 5](../levels/level-05.md).

## Using a door

Triangle in the stand state starts state 12. When its animation event
comes (+0xe0 & 0x810, 0x8005f584 at 0x8005f5d8), the use +0x54 of the
object the player touches (+0xf4, set by the collision code) is called as
use(door, player, 1). So the door must be the object Tommy is pressed
against, not just near. Seen: Triangle at 2400, use at 2418; Triangle
with the open back door slid away out of reach did nothing.

### Class 4 use (0x8004f4a0)

1. +0xcc non-zero (moving or blocked): return 1, nothing. Seen: a second
   Triangle on the moving gate (2118) changed nothing.
2. +0x64 = VSync.
3. Facing: local_18 = (rsin(h_d) x m02 + rcos(h_d) x m22) >> 12, where h_d
   is the door's heading now (+0x24, the placed heading plus +0xa8) and
   (m02, m22) are the player's matrix column 2 (+0x08, +0x14), which is
   (rsin h_p, rcos h_p) for the player's heading h_p (seen: h 2661 gave
   -3309, -2415). So local_18 is 4096 x cos(h_p - h_d): positive when
   Tommy faces the way the door faces, negative when he faces against it
   (0x80020c30 builds the door's y matrix, 0x8009f6f8 applies it). It
   is the player's facing, not his position.
4. Shut (+0x3c bit 2 clear):
   - +0xbc non-zero (either way): target +0xba = +0xb6 if local_18 > 0,
     else -+0xb6.
   - +0xbc zero (one way): local_18 >= 0 does nothing (return 1);
     else +0xba = +0xb6.
   - Sound 14 when +0xd0 is 1 (170 in level 16); start +0xd4 if non-zero
     (0x8002b98c).
5. Open (bit 2 set): target +0xba = +0xae (shut) when
   - +0xb6 > 0: +0xa8 and local_18 have opposite signs (neither 0);
   - +0xb6 < 0: they have the same sign (neither 0);
   - else nothing (return 1).
   Sound 15 when +0xd0 is 1 (170 in level 16).
6. Schedule the think every frame: +0xcc = 0x80072b48(door, 136). Return
   -1.

So a one-way door opens only for a player facing against it. An
either-way door swings away from the player's facing. Closing needs the
player to face the panel from the side it swung to. Seen (yard door 218,
heading 0, +0xb6 -896): from inside facing +z, target -896, sound 14;
then from the panel's far side at heading 1150, target 0, sound 15.

### Class 3 use (0x8004f6d8)

1. +0xcc non-zero: return 1, nothing (from the code; not run).
2. +0x64 = VSync.
3. +0xd0 is 2: sound 17 if open (bit 2 set), 16 if shut. Other kinds: no
   sound.
4. +0xcc = 0x80072b48(door, 136). Return 0.

No facing test. Seen: back door (337, +0xd0 2): 16 on opening (2418), 17
on closing (2528).

## The hinged think (0x8004f868)

Each think:

1. Blocked: 0x8006baa8(door) >= 0 (below). In level 16 message 24
   (0x80097788). If +0x6d (template +0x32) is not 0 and 51 VSyncs or more
   have passed since +0x64: +0x64 = VSync, Damage(player, door, +0x6d)
   (0x800780ac). Return 0: no move, no wait count; the think stays.
   No class 4 template has +0x32 (only class 8's 159, 160, 595, 602:
   15).
2. Wait: if +0xd2 is not 0, +0xd2 = max(+0xd2 - 136, 0), return 0.
3. Easing (class 8, +0xb8 not 0): +0xb0 = 256 + trunc(+0xb8 x
   rsin(((+0xa8 - +0xae) << 11) / (+0xb6 - +0xae)) / 4096).
4. Step = trunc(+0xb0 x 0x800c3f20 / 4096) (round toward 0). +0xa8 +=
   step if +0xa8 < +0xba, else -= step.
5. Stop test:
   - +0xba is 0: the sign of +0xa8 changed (0 counts as positive);
   - +0xba > 0: +0xa8 > +0xba;
   - +0xba < 0: +0xa8 < +0xba.
   On a stop:
   - +0xa8 = +0xba;
   - if bit 2 was set (a close has ended) and the template is not 1216
     or 672: sound 54 at the door (0x8006f644(54, door +0x18)); stop
     +0xd4 if non-zero (0x8002b9e0);
   - bit 2 toggles;
   - +0x00 bit 0x20000 clear (every class 4): +0xcc = 0, return -1 (off
     the scheduler).
   - bit 0x20000 set (class 8): +0xd2 = 4096 if (+0xbe & 8 and now
     open) or (+0xbe & 16 and now shut); +0xba = +0xae if +0xba was
     +0xb6, else +0xb6; keep going.
6. +0x24 = +0xc0 + +0xa8. Return 0.

The step needs exactly passing the target, not reaching it. A door whose
open angle is a multiple of its step lands on it and never stops: it
steps back and forth by one step and stays on the scheduler, so every
later use is refused (step 1). Seen: the playpen gate (431, step 1, open
1250) reached 1250 and then went 1249, 1250, 1249 ... with +0xcc set
(2136-2142 and on). Closing to 0 always ends (the sign test).

Measured steps at dt 10: 431 one unit a think (1250 thinks, about 42 s,
to open; seen 1 a think from 1994); 218 17 a think, 53 thinks each way
(seen 1982-2086 opening, 2188-2292 closing, sound 54 at 2292).

## The sliding think (0x8004e458)

Each think:

1. +0x64 = VSync. 0x800c4ba0 -= dt (a shared countdown: 0x8004d570,
   0x80050140, 0x80053344 and 0x80053604 play a sound and reset it when
   it runs out; the door only counts it down. Seen: 100 to -20 in 12 thinks).
2. Blocked: 0x8006baa8(door) >= 0 twice (the second call gives the
   same). In level 16 message 24. Return 0, no move.
3. Wait: if +0xd2 is not 0, +0xd2 -= 1 (one a think, not dt), return 0.
4. Open (bit 2 set), going shut: +0xa8, +0xaa, +0xac += 2 x +0xb0, +0xb2,
   +0xb4; end = +0xae. Template 770: sound 136 when +0xaa is exactly 900.
   Shut (bit 2 clear), going open: offsets -= 2 x steps; end = +0xb6.
   Template 770: sound 135 when +0xaa is exactly 1600.
   Template 843 with a negative step uses -end.
5. Position = (+0xc0, +0xc4, +0xc8) - M x (+0xa8, +0xaa, +0xac), M the
   object matrix +0x04 (the placed rotation, 0x8009fbe8). The draw uses
   +0x04 and +0x18 (no +0x00 bit 32).
6. Stop when the sum +0xa8 + +0xaa + +0xac >= end (going shut) or <= end
   (going open). No snap: the last step can overshoot. On a stop:
   - bit 2 toggles;
   - no bit 0x20000: +0xcc = 0, return -1;
   - bit 0x20000 (770): +0xd2 = 15 if (+0xbe & 8, after going shut) or
     (+0xbe & 16, after going open); keep going (so 770 cycles).
7. Return 0.

The step does not use dt: 2 x +0xb0 a think. No sound at a stop.

Seen (back door 337, step 40, open end -2048, placed heading 2048):
+0xa8 0, -80, -160 ... -2080 in 26 thinks (2418-2468); x 7155 to 5075;
then shut: -2000 ... 0 in 26 thinks (2528-2728, with a block between).

Template 770 in the oracle (level 24): sounds 136 and 135 both play on
its cycle (the arithmetic above had +0xaa moving by 64; it is not).

An automatic door's enter (0x8004f6d8) schedules it (+0xcc) and leaves
+0x00 bit 0 clear, so the trigger pass enters it again each time round
and never calls its leave: once the player has come in range it cycles
for good. A class 8 door is scheduled by its trigger (0x80049e18 in,
0x80049e5c out; +0x9c), so it starts when the pass reaches it, not when
the player crosses its radius (level 3's 160: eight frames later than a
distance test). The port's `Game::door_entered` does both.

## Blocking (0x8006baa8)

0x8006baa8(door) returns -1 when free, >= 0 when an edge of the door's
box passes through a solid thing.

1. Template flag 8 on the door: -1 (never blocked).
2. A cylinder around the door's position from the model's first box
   (0x80067b14; (guess) radius and height from the box's largest x, z
   and y extents), and the objects whose cylinders meet it in the door's room
   and its neighbours (0x8006add8 over the room lists +0x48, +0x4c and
   each portal's room; 0x8006abbc).
3. Only the player (0x800e2b48) and objects with +0x00 & 0x10000
   (pickable) or 0x4000 (AI) are tested.
4. Per model box (doors have one): the 8 corners in the door's frame
   (angles +0x24 by 0x80020d10, position +0x18; 0x8006b6d0), and the 12
   edges each tested as a segment against the object (0x8006b65c ->
   0x8006a714). The first edge that hits gives the result.
5. The result is that of the last object tested (a hit is not kept when
   a later eligible object misses; (guess) a quirk, harmless with one
   object near).

Blocking does not push the player; the door just waits a think and
tries again, for ever. Seen: the gate swinging into Tommy stood at
+0xa8 1081 from 4156 to 7100 (the end of the run), the player object
being the hit (0x8006c0a4); the back door closing on Tommy in the
doorway stopped at -1200 (2550) and went on when he left (2700).

## Collision with the player

The player's move tests objects' model boxes (0x80065210 ->
0x8006a714 -> 0x80069c40). 0x80069c40 builds the object's matrix from
its angles +0x24 at the time of the test (0x80020ee0) and uses its
position +0x18, then tests the movement segment against each model box
grown by the body radius. So:

- a hinged door's box turns with it (+0x24 is written each think);
- a sliding door's box moves with it (+0x18 is written each think);
- shut, moving or open, the door is solid where it is drawn, and the
  gap it leaves is open.

0x80046570 (at setup) sets +0x78..+0x84 to a cube of +- the model's
largest corner distance and +0x7e to the horizontal radius. The cube does
not depend on the angle, so it never needs turning; it is the broad test
(0x80069c10 via 0x80069fe0, and 0x8006abbc), centred on +0x18.

Seen: with the back door shut Tommy stopped at z 23134, 349 short of the
panel's centre plane (the box is +-59 thick); with it open he walked
through into room 38. With the playpen gate shut, walking across its line
from outside stopped at (3743, 17404) (state 9, the bump); with it open
the same walk crossed into the playpen.

## The hinge

The hinge is the object's origin, the LP position. The axis is the one
the first angle +0x24 turns about: for an upright placement (a turn about
y only, every door outside DB15) the vertical. A local point (x, y, z)
is at world (x cos h + z sin h, y, -x sin h + z cos h) + position, h the
heading (seen: the gate's free end lies along (cos h, -sin h) in (x, z),
found by placing Tommy on each candidate line).

The models put the hinge at x = 0 and the panel along +x or -x, thin in
z:

| template | model | box (min, max) |
|---|---|---|
| 431 | 502 | (0, -567, -6), (2151, 567, 6) |
| 218 | 379 | (0, -2049, 0), (2031, 2049, 141) |
| 1194 | 1602 | (-2031, -2049, -141), (0, 2049, 0) |
| 226 | 351 | (-1083, -1618, 4), (3, 1628, 114) |
| 227 | 352 | (-1, -2146, 4), (937, 1106, 114) |
| 337 (sliding) | 486 | (-1025, -2041, -59), (1025, 2041, 59), centred |

A positive +0xa8 turns the door like a heading increase (from +z toward
+x).

## Class 8, swinging doors (0x80048714)

Class 4's setup, then: enter +0x48 = 0x80049e18 (scheduled when the
player comes within +0xa0, 2D), leave 0x80049e5c (unscheduled), +0xb0 =
0, +0xb8 = template +0x18 (the easing), +0x00 |= 0x20000, +0xbe = 16 /
8 from template flags 0x10000000 / 0x20000000, +0xd2 = 0, +0xba = +0xb6.
They swing between +0xae and +0xb6 with eased speed (256 at the ends, 256
+ +0xb8 in the middle), wait 4096 (31 thinks) at an end when +0xbe says
so, and never stop while the player is in range. No use. Templates 159,
160, 595, 602 (+0x32 15) hurt a player they are blocked by. Placed in
DB02 (672, 1216: no sound 54), DB03 (159, 160, 595, 602) and DB06 (990,
991); not traced further here (see the level pages).

## Templates

Class 4 (step = trunc(speed x 136 / 4096) a think at dt 10; "thinks" to
open from shut):

| template | model | speed | step | shut, open | either way | +0xd0 | thinks | notes |
|---|---|---|---|---|---|---|---|---|
| 213 | 345 | 512 | 17 | 0, 1024 | no | 1 | 61 | |
| 214 | 346 | 512 | 17 | 0, -1024 | no | 1 | 61 | |
| 217 | 377 | -32 | -1 | 0, -1024 | no | 1 | never | moves away from its target (only DB19, cut) |
| 218 | 379 | 512 | 17 | 0, -896 | yes | 1 | 53 | seen |
| 226 | 351 | 1024 | 34 | 0, 1024 | no | 1 | 31 | trigger +0x34 1000 (unused by class 4) |
| 227 | 352 | 1024 | 34 | 0, -1024 | no | 0 | 31 | silent |
| 229 | 354 | 32 | 1 | 0, 1024 | no | 1 | never stops | lands on 1024 (only DB19, cut) |
| 431 | 502 | 32 | 1 | 0, 1250 | no | 0 | never stops | seen; silent, so only 54 could sound and it never closes |
| 1194 | 1602 | 512 | 17 | 0, 896 | yes | 1 | 53 | |

Closing takes the same number of thinks (to the sign change).

Class 3 (the offset changes by 2 x step a think; ends as offset sums):

| template | model | step (x, y, z) | shut, open | +0xd0 | thinks | flags | notes |
|---|---|---|---|---|---|---|---|
| 207 | 365 | (0, 0, 16) | 0, -384 | 1 | 12 | 0x42 | climbable (flag 64) |
| 224 | 350 | (0, 0, 64) | 0, 2048 | 0 | 1 | 0x2 | the open end is behind the start: it moves -128 and stops; +0x00 0x2000 (template +0x2a) |
| 233 | 359 | (0, 0, 16) | 0, -704 | 1 | 22 | 0x42 | |
| 234 | 360 | (0, 0, 16) | 0, -1024 | 1 | 32 | 0x42 | |
| 337 | 486 | (40, 0, 0) | 0, -2048 | 2 | 26 (to -2080) | 0x800002 | seen |
| 428 | 667 | (0, 0, 40) | 0, 1024 | 0 | 1 | 0x2 | not placed |
| 770 | 1083 | (0, 32, 0) | 2048, 0 | 0 | cycles | 0x70000002 | automatic, trigger 30000, waits 15 at both ends |
| 843 | 1189 | (0, -256, 0) | 0, 5120 | 3 | | 0x50000002 | own think, [level 5](../levels/level-05.md) |
| 866, 867 | 1221, 1222 | (0, 0, 16) | 0, -384 | 0 | 12 | 0x22 | |

## Placements

Every class 3 and 4 placement (LP kind 1); heading from a turn about y
only, else the quaternion (x, y, z, w). No placement has LP +0x14,
+0x28, +0x42, +0x44 or +0x4c set.

| DB | LP | template | class | id | position | heading |
|---|---|---|---|---|---|---|
| 01 | 50 | 337 | 3 | | (7155, -1024, 23483) | 2048 |
| 01 | 75 | 226 | 4 | | (30720, -611, 14314) | -1024 |
| 01 | 76 | 227 | 4 | | (30721, -82, 12293) | -1024 |
| 01 | 142 | 431 | 4 | 453 | (4843, 459, 18532) | 1309 (1308 in RAM) |
| 01 | 267 | 1194 | 4 | | (19435, -1028, 23648) | 0 |
| 01 | 268 | 218 | 4 | | (15364, -1028, 23509) | 0 |
| 05 | 135-150 | 843 | 3 | 523 (136), 8800 (143) | y -7637 and 4658 | 0 / -1024 |
| 05 | 190, 191 | 866, 867 | 3 | | (34862, -6141, 2780) | 1024 |
| 10 | 88, 89 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 10 | 151, 152 | 226, 227 | 4 | | (30714, -611, 14314), (30709, -82, 12293) | -1024 |
| 12 | 76 | 337 | 3 | | (7155, -1024, 23483) | 2048 |
| 12 | 116, 117 | 226, 227 | 4 | 602, 601 | (30720, -611, 14314), (30721, -82, 12293) | -1024 |
| 12 | 120, 121 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 12 | 174 | 431 | 4 | | (4843, 459, 18532) | 1309 |
| 12 | 230, 231 | 226, 227 | 4 | 510, 511 | (30720, 7574, 14314), (30721, 8103, 12293) | -1024 |
| 13 | 124, 125 | 226, 227 | 4 | | (30611, -611, 14314), (30611, -82, 12293) | -1024 |
| 13 | 127, 128 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 13 | 137, 141 | 218 | 4 | | (19206, -1025, 23505), (15531, -1025, 23514) | 1024 |
| 13 | 143 | 431 | 4 | | (4843, 459, 18532) | 1309 |
| 15 | 99, 193 | 207 | 3 | | (18553, -1572, 7018), (23113, -1572, 7018) | q (0, 0, 4096, 0) |
| 15 | 131, 132 | 226, 227 | 4 | | (13553, -832, 14314), (13553, -1361, 12293) | q (-2896, 0, 2896, 0) |
| 15 | 134, 135 | 213, 214 | 4 | | (13062, -1858, 9776), (13057, -1858, 9764) | q (-2896, 0, 2896, 0) |
| 15 | 190 | 431 | 4 | | (39534, -1902, 18532) | q (3454, 0, -2200, 0) |
| 15 | 194 | 233 | 3 | | (21211, -1572, 19782) | q (4096, 0, 0, 0) |
| 16 | 2, 3 | 226, 227 | 4 | | (42018, -2660, -27652), (44034, -2134, -27664) | 2048 |
| 16 | 28, 29 | 218 | 4 | | (30115, -3079, -28681), (30250, -3079, -32764) | 1024, -1024 |
| 18 | 74, 77 | 227, 226 | 4 | | (30709, -82, 12293), (30709, -611, 14314) | -1024 |
| 18 | 123 | 431 | 4 | | (4843, 459, 18532) | 1309 |
| 18 | 156, 157 | 214, 213 | 4 | | (31245, 415, 8914), (31247, 415, 10639) | -1024 |
| 19 | 118, 119 | 217, 218 | 4 | | (15328, -1501, 23537), (19420, -1501, 23537) | 2048 |
| 19 | 142 | 337 | 3 | | (7155, -1024, 23483) | 2048 |
| 19 | 154, 173 | 207 | 3 | | (21278, 624, 7009), (25825, 624, 7018) | 0 |
| 19 | 167, 168 | 234, 233 | 3 | | (23188, 633, 19832), (23188, 406, 19826) | 2048 |
| 19 | 226 | 431 | 4 | | (7709, 461, 18534) | -739 |
| 19 | 260, 261, 262 | 226, 227, 229 | 4 | | (30611, -611, 14314), (30611, -82, 12293), (30606, -1702, 12289) | -1024 |
| 19 | 263 | 224 | 3 | | (31307, 607, 11573) | -1024 |
| 19 | 265, 266 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 19 | 270, 271 | 431 | 4 | | (8603, 461, 20432), (3794, 461, 20419) | -1024 |
| 20 | 49, 50 | 226, 227 | 4 | | (30720, -611, 14314), (30721, -82, 12293) | -1024 |
| 20 | 52, 53 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 20 | 100 | 431 | 4 | | (4843, 459, 18532) | 1309 |
| 21 | 21, 77 | 207 | 3 | | (25825, 130, 7018), (21265, 130, 7018) | 0 |
| 21 | 34, 35 | 226, 227 | 4 | | (30720, -611, 14314), (30721, -82, 12293) | -1024 |
| 21 | 38, 39 | 213, 214 | 4 | | (31247, 415, 10639), (31245, 415, 8914) | -1024 |
| 21 | 78 | 233 | 3 | | (23166, 130, 19782) | 2048 |
| 22 | 31 | 431 | 4 | | (4851, 461, 18534) | 0 |
| 24 | 31 | 770 | 3 | | (37379, -1672, 17307) | 0 |
| 31 | 72 | 224 | 3 | 6660 | (57344, -1024, -11264) | 0 |

DB19 is a cut level and DB22 the opening cine's stage. DB16's double door
is two separate 218 objects (LP 28, 29), so each half is its own use and
opens on its own (as seen in [level 16](../levels/level-16.md)). In
level 12 the refrigerator's use is replaced (601, 602: 0x8004f83c, cine
67), so it never swings there ([level 12](../levels/level-12.md)).

## Checked in the oracle

The level 1 probe copied to /mnt/data/claude/scratch/doors/probe as
doorprobe, with `--door F0:F1:T` (each object of template T, or the
object at an address, every frame: position, angles, +0x3c, +0xa8..+0xd4,
matrix, player), `--hookv0 ADDR` (v0, s1, s2 and their templates at an
address) and a `doormodels DB SHAPES` tool (model boxes). Same start as
level 1 (+0x30 = 4, cards skipped); places by `--tp`, walks by `--goto`.

- Playpen gate (431): Triangle at 1975 facing 2661, use at 1994 (+0xba
  1250, no sound), +0xa8 one a think; a second Triangle at 2118 refused;
  swinging into Tommy by the hinge blocked it at 1081 from 4156 on (the
  player the hit); poked to 1200, it reached 1250 and then stepped
  1249 / 1250 for ever with +0xcc set; the hinge axis and swing sense by
  0x8006baa8 hits on two candidate lines; the shut gate stopped a walk
  across its line, the open one let it through.
- Back door (337): shut, it stopped Tommy at z 23134; Triangle, use at
  2418, sound 16, -80 a think to -2080 at 2468 (26 thinks), x 7155 to
  5075, think off the scheduler, bit 2 set, nothing more until the next
  use; walked through into room 38; Triangle from the old spot did
  nothing; Triangle at the slid panel (5075) closed it (sound 17) and with
  Tommy in the doorway it stopped at -1200 until he left, then reached 0
  exactly in 26 moving thinks, bit 2 clear, no end sound.
- Yard door (218): from inside facing +z: sound 14, target -896, 17 a
  think, snapped to -896 after 53 thinks; from the panel's far side
  (heading 1150): sound 15, target 0, 53 thinks, sound 54 at the door
  (0x8006f644(0x36, door +0x18)).
- 0x800c3f20 = 136 and 0x800c4ba0 counting down by 10 a sliding think.
- Not checked: the refrigerator, 213 / 214, 1194, any class 3 template
  but 337; class 8; 770's cycle; level 16's message 24 and sound 170;
  blocking by a pickable or an NPC; DB15's turned placements; the second
  use of a sliding door while it moves.

## For the port

The port draws these doors at their LP placement and collides with them
as fixed boxes (game.rs object_boxes); Triangle only looks for level and
cine doors (nearest within 1500). In order:

1. Door objects: classes 3 and 4 by template byte +0x04, with the state
   above (+0x3c bit 2, the offset or angle, the target, the moving flag,
   the wait). Shut at the start.
2. The use: Triangle on the object Tommy is touching (the collision's
   touched object), at the use animation's event, not the nearest object.
   Refused while moving. Class 4's facing rule (cos(h_p - h_d), one way /
   either way, closing from the side it swung to); class 3 has none.
3. The thinks at one call a game frame: hinged step trunc(speed x 136 /
   4096) with the pass-the-target stop and the snap (and the playpen
   gate's never-ending 1249 / 1250 with uses refused); sliding 2 x step a
   think, no snap, the position from the placed matrix.
4. Collision: rebuild a door's box each frame from its angle or
   position (the port's per-frame `boxes` list in game.rs already adds
   movers), so the gap opens as it moves.
5. Blocking: before moving, the door's 12 box edges against Tommy (and
   pickables and NPCs); blocked = no move this think, no push. Level 16:
   message 24.
6. Sounds: class 4 kind 1: 14 opening, 15 closing (170 in level 16), 54
   at the door when a close ends (not 1216, 672); class 3 kind 2: 16
   opening, 17 closing.
7. Drawing: a hinged door at heading +0xc0 + +0xa8 about its origin; a
   sliding door at its moved position.
8. Later: class 8 (eased swinging, damage 15 for 159, 160, 595, 602), the
   automatic 770 in golf (DB24), template 843 is level 5's.

## Unknown

- The doors in DB15 (Mirrorland): their placements turn about tilted
  axes; whether +0x24 is still the vertical there, and what the facing
  rule (a y-only matrix) does with them.
- Whether a game frame ever takes 3 VSyncs in these rooms (dt 15, 0x800c3f20
  204: the gate's step would be 1 still, 218's 25).
- Template 217's negative speed: by the code it turns away from its
  target until +0xa8 wraps (32768 thinks); only in DB19 (cut).
- What sounds 14 / 15 / 16 / 17 / 54 / 135 / 136 / 170 are (numbers only).
- 0x8006add8's neighbour scan and 0x8006abbc's cylinder sizes for the
  blocking test, beyond what the runs showed.
- Who else counts down 0x800c4ba0 in the house levels.
