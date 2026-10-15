---
title: Mini golf, levels 23-31
status: partial
discs: US
covers: US SLUS_006.50:0x8007a584, US SLUS_006.50:0x80057e50, US SLUS_006.50:0x80057d50, US SLUS_006.50:0x80057dcc, US SLUS_006.50:0x80058538, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x8005faf4, US SLUS_006.50:0x8005fb38, US SLUS_006.50:0x8005fcb8, US SLUS_006.50:0x8005f998, US SLUS_006.50:0x8005f810, US SLUS_006.50:0x8005f68c, US SLUS_006.50:0x8005ba44, US SLUS_006.50:0x8005bac4, US SLUS_006.50:0x8005bc80, US SLUS_006.50:0x8005be24, US SLUS_006.50:0x80058278, US SLUS_006.50:0x8008864c, US SLUS_006.50:0x80087c5c, US SLUS_006.50:0x80086ef0, US SLUS_006.50:0x80086b58, US SLUS_006.50:0x80086668, US SLUS_006.50:0x80086cd0, US SLUS_006.50:0x80086a20, US SLUS_006.50:0x8007666c, US SLUS_006.50:0x800763d0, US SLUS_006.50:0x800580a8, US SLUS_006.50:0x80057f78, US SLUS_006.50:0x80057f08, US SLUS_006.50:0x8007a9d8, US SLUS_006.50:0x8007aff0, US SLUS_006.50:0x80043968, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x80025c64, US SLUS_006.50:0x8002953c, US SLUS_006.50:0x800972c4, US SLUS_006.50:0x8008cef0, US SLUS_006.50:0x8008cd00, US SLUS_006.50:0x800818e0, US SLUS_006.50:0x80081758, US SLUS_006.50:0x80096134, US SLUS_006.50:0x80096a30, US SLUS_006.50:0x800494d8, US SLUS_006.50:0x800b7ffc, US SLUS_006.50:0x800b7ff0, US /DATA/DB23/DB23LP.BIN, US /DATA/DB25/DB25LP.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Mini golf, levels 23-31

The nine mini-golf holes ("Hole One" .. "Hole Nine"): starting a hole,
the swing, the ball's roll, holing out, the camera, the HUD, and the
activity-menu version for one to four players. Read from the pseudo-code
and the disassembly; the main paths were run in the oracle (see Checked
in the oracle). Builds on [gameplay](gameplay.md) (player, states,
camera, events), [objectives](objectives.md) (flights, HUD pools, cards,
progress) and [level 3](level-03.md) (cine hooks, ForceCry).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down.
Heading 0 faces +z; a heading h faces (rsin h, 0, rcos h) (0x8009efd8 is
rsin and 0x8009f0a8 rcos: checked by the stance placement below). "frame"
is a game frame (two vertical blanks in the oracle). "hole index" is the
play order 0-8 (0x80057d50), not the level number.

## The game in one paragraph

From Pickles' Home a door (template 399, cine 36 "House to Golf",
[gameplay](gameplay.md#doors-0x8004f78c)) leads to level 25, the first
hole. Card HINT27 explains: walk up to the ball and the club appears;
Left and Right aim; Triangle starts the power meter, which runs up and
down; Triangle again swings. There is no stroke limit. Holing out at or
under par goes to the next hole. Over par shows "You were over par. Try
Again", Tommy (or whoever plays) cries and the hole starts again with its
strokes cleared. Each hole has its own baby (Tommy, Chuckie, Angelica and
two more). Holing out on the ninth hole plays cine 35 "Golf Ending" and
records level 8 as won (its puzzle piece). A hole in one is worth a
Reptar bar. The loading screen calls the course "Ice Cream Mountain"
(string 158, 0x8003b518; read, not seen), and level 8's progress bit is
the one golf sets, so golf is level 8's episode in the progress block
(guess: level 8 itself is not a playable database).

## The nine holes

The hole index maps level to play order (0x80057d50) and back
(0x80057dcc):

| level | index | name shown | par (0x800b7ffc) | char (0x800b7ff0) | start cine (0x80074958) | player start (0x8007a584) | ball (LP, template 724) | cup (LP, template 789) | ambience (0x800b9618) |
|---|---|---|---|---|---|---|---|---|---|
| 25 | 0 | Hole One | 4 | 1 | 34 "Golf Intro" then 31, or 31 | (35382, 11, 12372) | (37391, 150, 12761) | (37277, 223, 33158) | 0x80081474 |
| 28 | 1 | Hole Two | 5 | 2 | 81 "Golf Cam 6" | (38387, -124, 8978) | (38408, 150, 6681) | (56830, -909, 537) | 0x80081524 |
| 24 | 2 | Hole Three | 4 | 3 | 30 "Golf Cam 2" | (35197, 215, 2194) | (37388, 150, 475) | (37268, 237, 33138) | 0x800815e0 |
| 29 | 3 | Hole Four | 5 | 5 | 82 "Golf Cam 7" | (53580, -142, -9917) | (55754, 150, -9662) | (41473, -927, 4659) | 0x8008168c |
| 23 | 4 | Hole Five | 5 | 4 | 29 "Golf Cam 1" | (43692, -138, -15253) | (45567, -331, -14901) | (31200, 271, 11753) | 0x80081758 |
| 30 | 5 | Hole Six | 5 | 1 | 83 "Golf Cam 8" | (37468, 11, -10897) | (39343, 150, -10657) | (54745, 229, 8735) | 0x80081474 |
| 26 | 6 | Hole Seven | 4 | 2 | 32 "Golf Cam 4" | (35502, -98, 12393) | (37401, 150, 12752) | (37332, 235, 35207) | 0x80081524 |
| 31 | 7 | Hole Eight | 5 | 3 | 84 "Golf Cam 9" | (36284, -215, -9095) | (38407, 150, -8718) | (47631, 221, 5589) | 0x800815e0 |
| 27 | 8 | Hole Nine | 3 | 1 | 33 "Golf Cam 5" | (43693, -2037, 11334) | (45579, -1899, 11740) | (45563, 253, 40424) | 0x80081474 |

Total par 40. Ball and cup positions are the LP records (kind 1); the
player start is passed to 0x800825b0, and the floor snap moves y (level
25: y 11 seen, level 28: -98). The "Golf Cam" numbers follow the level
number, not the play order (cine table 0x800b8bd0).

Characters: 1 Tommy (registry id 1492), 2 Chuckie (1776), 3 Angelica
(600), 4 and 5 the twins (609 and 608; 4 is Lil and 5 Phil, guess from
the pick screen's faces), 37 Susie (guess, activity play only). Hand and
club models per character are under The club.

## Golf state

| address | type | meaning |
|---|---|---|
| 0x800c42b0 | ptr | the ball: the object of template 724 (0x800494d8, class 0) |
| 0x800c42b4 | ptr | the cup: template 789 (0x800494d8) |
| 0x800c4cd0 | u8 | hole index, set at the end of EpisodeStart (0x8007a584); shared with other levels' phase byte |
| 0x800c429c | ptr | the power meter (0x8008cef0), 0 when none |
| player +0x1b4 | u16 | power, 1 = idle meter, else 4..220 |
| player +0x1c0 | s32 | meter direction: 0 up, 1 down |
| player +0x1b6 | u8[9] | strokes per hole, by hole index |
| player +0x1bf | u8 | total strokes (written only; nothing reads it in single play) |
| player +0x220 | ptr | the stroke-count text item (0x80090004) |
| ball +0x00 bit 0x8000000 | | lipped out this roll: no more cup tests until the ball stops |
| ball +0x90 | ptr | flight record while rolling |
| ball +0xa2 | u16 | bounce, template +0x36 = 3000 |
| 0x800c46d0 | ptr | the activity-golf block (also holds the camera-height cycle, +0x860..+0x866) |

Templates (TEMPLATE.BIN): 724 the ball, model 1066, class 0, +0x28 = 1
(pickable, but GRAB skips pickups in golf, 0x8005d11c), +0x36 = 3000;
789 the cup, model 1100. The ball's box is (-174..174) on each axis
(object +0x78..+0x84, oracle), so its box centre is its position.

## Starting a hole

### Player init (0x80057e50, from 0x800596e8 in levels 23-31)

1. Power +0x1b4 = 1.
2. Hole index 0: all nine stroke bytes and the total cleared. Other
   holes: only that hole's byte (the total is kept).
3. Normal play: character = 0x800b7ff0[hole] (0x8005928c: new hierarchy,
   state +0x08, player reset). Activity play: 0x80097268 (the current
   player's pick) and 0x80097288(camera +0x9c) (the camera-height base).
4. 0x800c429c = 0.

### Level-start cinematic (0x80074958)

Case level + 1 of table 0x80014e00. Level 25 plays 34 "Golf Intro"
unless the game is in activity play or the previous level (state +0x04)
is 25, 28 or 24; then 31 "Golf Cam 3". After cine 34, 0x8007737c (case
33) posts cine 31 and CineFinish shows card 698 (HINT27, arg 9)
(0x800783d8 case 33). The other levels' cines have no card and no
after-handler.

Cine hooks:

- Cine 31 start (0x80072ef0): draw (+0x40) = 0 for ids 1492, 1776, 600,
  15, 608, 609. End (0x800732d4): their draws back (0x800307dc(2)), then
  15, 600, 1776, 608, 609 hidden (0x80078bc4).
- Cine 33 end: 15, 600, 608, 609, 1776 hidden.
- Cine 35 start: 0x8005928c(1, player): the character goes back to Tommy.

### EpisodeStart (0x8007a584)

Installed for levels 23-31 by 0x8007871c (case 23..31). In order:

1. Text "Par %d" (string 195) with par[hole] at (-192, Y).
2. Text "Swing " (string 196) at (50, Y).
3. Text "%d" (0x800c4424) with strokes[hole] at (150, Y), kept in player
   +0x220.

   All three: 0x80090004, colour 0x13cdec (yellow), Y = ((0x800c4a06 *
   171) >> 11) - 120; 0x800c4a06 is 240 in the oracle, so Y = -100 (top
   of the screen).
4. Player placed at the start (0x800825b0, table above). Per level:
   23: 0x800c4ba0 = 500; 24: 1000; 29: 100 (the object-sound countdown,
   see [level 3](level-03.md#phases-and-state)).
   26: object id 9999 (template 8, hierarchy 9) gets mode 8
   (0x80083918) and is scheduled (0x80049e18); prints "Reptar's
   ALIVE!!!!".
   31: outside activity play id 6660 (template 224) is hidden; id 9998
   (template 516, hierarchy 47) follows the player (0x80083764, mode 4)
   with range +0x68c = 3500 and is scheduled.
5. Level 23: camera position = (47002, -2186, -19891).
6. 0x800c4cd0 = hole index.

In the oracle a CameraSwitch at the same moment set the follow camera
with 0x800c3f5c = 1600 (0x800291f8(camera, player, 1, 1600) from
0x80077c88).

### Tries

The tries byte (state +0x23) is not set for golf: 0x800b2920[23..31] is
-1 and the runner skips negative values (0x80018138). It keeps the hub's
0, and ForceCry only counts down a non-zero byte, so retries are
unlimited (0x8007fd0c; seen 0 at hole 1).

## The swing

### Walking up to the ball (0x8005a56c)

In the player update, levels 23-31, state not 13, 14 or 15:

1. 0x80058538: the ball exists, is not rolling (+0x90 = 0), and
   (ball.x - player.x)^2 + (ball.z - player.z)^2 <= 0xfffff (about 1024).
   (It reads +0x90 before the null test.)
2. 0x80057244(player, 30, 1) (+0xe6 = 30); state = 13.
3. Stroke text: sprintf "%d" (0x800c42ac) with strokes[hole] + 1, into
   +0x220 (0x80090214): it shows the number of the stroke about to be
   played ("Swing 1" before the first).

The state change (0x8005b2d8) runs later in the same update. JUMP is
ignored in golf in the stand and walk states (0x8005d11c, 0x8005e1f4).
While the ball rolls the stick gives no speed (0x8005be24 at 0x8005befc:
level >= 22 and ball +0x90 set). If the ball stops within reach the
player goes straight back to state 13 (seen).

### The stance (0x8005bac4)

0x8005bac4(player, c, r, off): a = heading + off, plus 4096 when
negative, less 4096 when above 4096;

```
x = c.x + trunc(r * -rsin(a) / 4096) + trunc(-rsin(a - 1024) / 32)
z = c.z + trunc(r * -rcos(a) / 4096) + trunc(-rcos(a - 1024) / 32)
```

(trunc rounds toward zero: the code adds 4095 or 31 to negatives before
the shift.) y is not touched.

### State 13, walk-in (0x8005faf4)

Entry (0x8005b2d8 case 13): handler 0x8005faf4, animation 29;
0x8005ba44(player, ball, 1024): heading += the turn toward the ball
(0x80075e4c direction, 0x80075d7c difference), target heading = heading,
then 0x8005bac4(player, ball, 1024, 0): 1024 behind the ball and 128 to
the side. Camera: 0x800291f8(camera, player, 12, 0) (golf camera).

Each frame: 0x8005f998 (feet, below). When the animation ends (+0xe0 bit
0x8000000): state 14, power = 1. In the oracle the player moved round
to the stance during the animation (guess: its root motion); state 13
lasted 56 frames.

0x8005f998: floor under parts 7 and 3 (guess: the feet) less +0x148. If
either is more than 1024 above, 0x8005f810 (guess: pushes the player and
the ball apart along a wall the stance would cross: both move by
0x8001f5c0(dir, 4)); else the player's y and +0x148 move by the higher
foot's difference.

### State 14, aim and power (0x8005fb38)

Entry: handler 0x8005fb38, animation 30, 0x80097288(camera +0x9c).

Each frame, in order:

1. 0x8005f998.
2. No meter yet: 0x8008cef0(294, ..., 293) into 0x800c429c (see HUD).
3. Power != 1: power += 3 (direction 0) or -= 3 (direction 1); then if
   power >= 220, direction = 1. So 4, 7, .., 220, 217, .., 4, 1, and at 1
   the meter idles again. The step is per frame, not per dt.
4. Triangle pressed (0x800c49d4 `& 3 == 1`, not AI): power 1: power = 4,
   direction 0 (meter starts). Else: state 15, 0x80057244(player, 0, 1),
   return. The swing keeps the power of this frame (after step 3).
5. Meter value = power (0x8008d038). Aim line (0x8005f68c).

In the turn step of the update (0x8005bc80), state 14 re-places the
player each frame: 0x8005bac4(player, ball, 425, 1024): 425 to the side
of the ball and 128 behind it, facing along the heading.

### Aiming (0x8005be24)

State 14 with power 1 only (with the meter running the stick code exits
at once, case 8 of 0x8005be24's table at 0x8001409c). Forward input is
ignored.

- Left or Right held (or the analog X axis 0x800c49a8 non-zero): step 25,
  or 8 with WALK (Square) held. X axis (0x800c499c >> 3, or the analog
  one when larger) below 70: -step, else +step (an analog axis between
  0 and 69 also turns left). The step is clamped to magnitude * dt >> 6
  (640 at full stick and dt 10) and added to the target heading +0x2c,
  wrapped to 0..4095.
- The turn (0x8005bc80, 160 a frame at dt 10) makes the heading follow
  at once.
- Circle pressed: camera +0x9c = 0x800972c4() (camera height, see
  Camera).
- Circle and Cross held: camera +0x88 += Up/Down axis >> 6, clamped to
  -6000..0 (guess: overwritten by the camera update each frame, so no
  effect; not checked).

Seen: Right adds 25 a frame (0 to 125 in 5 frames), Left takes 25,
Square+Left 8.

### Aim line (0x8005f68c)

Five flat sprites (0x8008e76c, flags 0xd2803000, sprite 364, size (200,
250), grey (128, 128, 128), angles (-1024, heading, 0), the player's
room) at 250, 750, 1250, 1750 and 2250 from the ball along (0, 0, 4096)
turned by the heading (0x80075ee8). Only drawn in state 14. Seen: arrows
on the green toward the aim.

### State 15, the stroke (0x8005fcb8)

Entry: camera +0x9c = 0x800972ac() (resets the height cycle, returns
0x800c4730[0] = 0), 0x80097288(0), handler 0x8005fcb8, animation 31.

Each frame, when +0xe0 bit 2048 is set (guess: the strike event of
animation 31) and the ball is not rolling:

1. Sound 118 (0x8006f26c).
2. strokes[hole] += 1; total += 1.
3. Hit (0x80058278).
4. The meter is freed (0x8008d06c), 0x800c429c = 0.

Seen: the hit came 13 frames after state 15 began, and the state was 0
again 7 frames later (who sets it was not found; guess: the end of
animation 31).

### The club

Each frame in levels 23-31 (0x8005a678) the hand part (hierarchy slot
+0x23, part +0x58) gets a model: in states 13-15 the club
(0x80057404: char 1 1069, 2 1216, 3 1215, 4 1218, 5 1217, 37 1219),
else the bare hand (0x80057468: 7, 22, 37, 54, 69, 1185).

## The hit (0x80058278)

Builds a launch descriptor on the stack for 0x8008864c (layout in
[objectives](objectives.md#flight-0x80061f88-0x8008864c-0x80087c5c)):

| desc | value |
|---|---|
| +0x00 | 0x19030000: collide with world and objects (0x30000, flag 0x10000000 routes to 0x80086ef0), stop when the roll stops (0x1000000), 0x8000000 |
| +0x04..+0x0c | ball box centre ((+0x78 + +0x80) / 2 etc., rounded toward zero) + ball position |
| +0x10 | radius: the larger of \|model box +0x02\| and \|+0x0a\| (ball +0x88 +0x0c); without a model the larger of \|+0x7a\| and \|+0x82\|. 104 in the oracle |
| +0x1c | 0 (no angle output) |
| +0x20 | &ball +0x18 (position written back) |
| +0x24 | &ball +0x6b (255 when the roll ends); ball +0x6b = 0 first |
| +0x28 | 0 (no lifetime) |
| +0x2c..+0x34 | direction: player angles +0x24 (heading), +0x26 through 0x80076040: (rsin h * rcos p >> 12, -rsin p, rcos h * rcos p >> 12), then VectorNormal (0x8007652c). +0x26 was 0 in the oracle, so the roll is flat |
| +0x38 | speed = power |
| +0x42 | 0 (no spin) |
| +0x44 | bounce: ball +0xa2 clamped to 4095 (3000) |
| +0x46, +0x48 | 0 |
| +0x4c | ball +0x5c (0 for template 724) |
| +0x64 | hit sound 137 |
| +0x66, +0x67, +0x68 | 0 |
| +0x6a | ball +0x6d (0) |

Not written, stack leftovers: +0x14, +0x18, +0x3c..+0x40 (spin axis),
+0x50 (to record +0x84), +0x54 (to 0x80086114), +0x58..+0x60 (record
+0x74, +0x78, +0x70), +0x65. The spin rate +0x42 is 0, so the axis is
never used; the others are unknown.

Then: ball +0x90 = 0x8008864c(desc); ball into its room's list
(0x8004a440); if a record was given: think +0x44 = 0x800580a8 and
0x80072b48(ball, 68) (period 68 of 0x800c3f20's 4.12 s units, which is
136 a frame: it runs every frame).

### The flight record (0x8008864c)

As in [objectives](objectives.md#flight-0x80061f88-0x8008864c-0x80087c5c),
with these golf values (oracle dump at the hit, power 94, heading 957):

| rec | value |
|---|---|
| +0x08 | 0x19030000 |
| +0x0c | 0: no lifetime |
| +0x10..+0x18 | start (37391, 150, 12761), clamped into the room box (room +0x10..+0x28) |
| +0x1c | radius 104 |
| +0x28 | bounce 3000 |
| +0x38 | room of the start (0x8002b660(start, -1)); -1 gives no record and no roll |
| +0x3c..+0x44 | v = dir * k >> 12, k = trunc((speed << 16) / 6000): (1022, 0, 105) for k 1026 and dir (4081, 0, 420) |
| +0x52 | spin rate 0 |
| +0x88 | sound 137 |
| +0x92 | 1 |

k for power 1..220 is 10..2402 (220 gives 2402, the value 0x80057f08
tests for).

## The roll (0x80087c5c with flags 0x19030000)

Each frame 0x8008851c(dt) steps every record (after the scheduler, so
the ball's think sees last frame's result). For the golf flags:

1. Keep the old position, radius and +0x20/+0x24 in +0x54..+0x68.
2. Gravity: flags bits 8-11 are 0, so acceleration (0, 0, 0).
3. Integrate (0x8004fbb4): p' = p + ((v * dt) >> 6) per axis (arithmetic
   shift; the a dt^2 / 2 term is 0); v unchanged.
4. No spin (+0x52 = 0). No drag (flags bits 12-14 are 0).
5. Collide and roll: 0x80086ef0(rec, &v, &p', dt), below. Result bit 2
   (stopped) with flag 0x1000000 ends the flight.
6. Write p' to +0x10.. and the ball position (+0x30); v to +0x3c...
7. Ended: *(+0x34) = 255 (ball +0x6b).

The lifetime (+0x0c = 0) never ends it.

### Collision step (0x80086ef0)

1. Room of the old position (+0x10) with hint +0x38 (0x8002b660); when
   none, the old position is clamped into the old room's box.
2. R = max(radius, 10).
3. Segment: s = old position, e = p' + VectorNormal(v) * R >> 12 (the
   leading edge).
4. 0x80086b58(segment, v): n = the floor plane at s (COL record byte +1,
   planes 0x800c4a7c, 6 bytes s16 x, y, z; no record: (0, -4096, 0)).
   d = 0x80075f58(s, e) (isqrt of the squared distance; 16384 when any
   axis differs by 16384 or more). e = s + P * d >> 12, with P =
   0x800763d0(-v, n): VectorNormal(v^ - n (v^ . n)), the move bent along
   the floor's plane at the start.
5. Old room found: 0x80086668(rec, flags & 0xf0000, segment, room, ...):
   objects along the segment room by room (0x80086340, through portals
   with 0x8002ade0) and the world segment test 0x8002cd7c (the camera's
   "vectorwalk"); a world hit replaces an object hit when it is nearer on
   each axis (per-axis compare, guess). Out: hit point H, normal N.
6. World hit: result |= 9; object hit: result |= 1, record +0x98 = the
   object (unless AI). Either: callback +0x80 (none), sound 137 at the
   ball (0x8006f644). Flag 0x40000 (object touch callbacks) is not set
   for golf.
7. Bounce, when hit: 0x8007666c(v, N, &r): v^ = VectorNormal(v); c =
   (N.x v^.x >> 12) + (N.y v^.y >> 12) + (N.z v^.z >> 12); c < 0: r = v^ -
   (c N >> 11) per axis, else r = v^; returns acos(c) (0x80021500). If the
   angle is 1024 or less (not moving into the surface) result bit 1 is
   cleared. Else: p' = H - VectorNormal(v) * R >> 12; s = |v|
   (0x80076784); v = r * (s * bounce >> 12) >> 12; spin rate *= bounce >>
   12.
8. Roll (0x80086cd0(p', v, R, dt)); 0 sets result bit 2.
9. Write p'.

Seen: a wall at x about 39900 turned (771, 0, 71) into (-551, 0, 49):
reflected, times 3000 / 4096, less one friction step.

### Rolling (0x80086cd0)

```
f = (98 * dt) >> 6                 0x800c4460; 15 at dt 10
v.y = 0
s = |v|                            0x80076784: isqrt(x^2 + y^2 + z^2), 0 if any |c| >= 65536
u = VectorNormal(v)                0x8007652c
floor query at p' (0x8002b240)     no record: v = 0, return 0 (stopped)
n = plane[record +1]
p'.y = floor - R                   the ball sits on the floor; it never flies
a = slope_push(n.y, dt)            0x80086a20
if a == 0 and s <= f: v = 0, return 0
v.x = trunc(u.x * (s - f) / 4096); v.z = trunc(u.z * (s - f) / 4096); v.y = 0
if a == 0: return 1
h = VectorNormal((n.x, 0, n.z))    downhill
v.x += trunc(h.x * a / 4096); v.z += trunc(h.z * a / 4096)
if |v| > f: return 1
v = 0, return 0
```

slope_push (0x80086a20), with t = asin(n.y) from the table at 0x800b4598
(0x80021460):

```
slope = (-t - 1024 >= 0) ? -t - 1024 : t + 1024     0 on a flat floor (n.y = -4096)
slope >= 768 (0x800c4470): 1190 (0x800c4468), not scaled by dt
slope <= 0   (0x800c446c): 0    (0x800c4464)
else: ((((99 * slope) >> 6) + 2) * dt) >> 6
      99 = ((0 - 1190) << 6) / (0 - 768), 2 = 1190 - ((99 * 768) >> 6)
```

On a slope steeper than s - f can hold, s - f goes negative and the ball
turns back downhill. Seen on the flat first hole: speed falls by 15 a
frame (1015, 1000, 984, ...), y stays floor - 104 = 152.

Normalisation everywhere is VectorNormal (0x8009f278 through 0x8007652c,
which first shifts the vector down so its largest axis fits 12 bits).
Its results are not exact (the hit's direction came out 4102 long), so
the port needs the same routine to match the oracle to the unit.

## The ball's think (0x800580a8)

Every frame while rolling:

1. Room update (0x8004a34c).
2. d2 = (cup.x - ball.x)^2 + (cup.z - ball.z)^2 (2D; y is not tested).
3. Bit 0x8000000 clear and d2 <= 0x24000 (384^2): s = |v| of the record
   (0x80088c6c, 0x80076784).
   - s > (0x800c42b8 << 16) / 6000 = (132 << 16) / 6000 = 1441: lip
     out. Sound 119, bit 0x8000000 set, 0x80057f78(&v, s). Returns 0.
   - Else holed: sound 120, bit cleared, record freed (0x80088600), +0x90
     = 0, +0x6b = 0, +0x44 = 0, 0x8007a9d8(player). Returns -1 (off the
     scheduler).
4. Else, rolling ended (+0x90 set and +0x6b = 255): +0x90 = +0x6b = +0x44
   = 0, bit 0x8000000 cleared, camera back to follow (0x800291f8(player
   camera 0x800e2d6c, player, 1, 0)). Returns -1.

A ball that stops inside 384 of the cup is holed (step 3 runs before
step 4). The limit is stored as a power (132): 1441 is k for power 132,
so a ball launched at 132 or less always drops once it is within 384.

### Lip out (0x80057f78)

```
c = VectorNormal(cup - ball)                3D (0x80075e4c with normalise)
d = yaw difference of c and v               0x8007614c: ratan2(x, z) each, wrapped to -2048..2048 (sign order not checked)
|d| < 256 (0x800c42bc): c = -c              straight back
else: ang = (ratan2(v.x, v.z), -asin(v.y), 0)               0x800760f4
      ang.yaw += 2 * turn(s, d)                              0x80057f08
      c = 0x80076040(ang)
v.x = trunc(c.x * s / 8192); v.z = trunc(c.z * s / 8192)    half speed; v.y kept
turn(s, d) = s == 2402 ? 0 : ((((-d << 6) / 2402) * s) >> 6) + d    (d (1 - s / 2402))
```

Seen: power 202 straight at the cup lipped out at 133 short (speed 2175)
and came back at about -948.

## Holing out (0x8007a9d8)

1. Activity play (state +0x21 = 1): SwitchEpisode(player, this level).
   Nothing else (the round driver takes the next turn).
2. strokes[hole] > par[hole] (over par):
   - Speech by character (0x8008da4c): 1: 72 at par + 2, then one of
     0x800b9250[rand & 7]; 2: 336 at par + 1, 337 at par + 2, then
     0x800b9260[(rand & 255) % 7]; 3: 587 / 588, then 0x800b9270[% 10];
     4: 938 at par + 2, then 0x800b9298[% 7]; 5: 809 / 810, then
     0x800b9284[% 9]. Tries byte 1: also 0x800b92d8[rand & 1].
   - ForceCry(player) with a 2-frame delay (0x800782d4).
   - 0x8008dbec; then 0x80043968: a box "You were over par." (string 214,
     y -50) / "Try Again" (232, y -15) with "PRESS X TO CONTINUE", until
     Cross (sound 126).
   - ForceCry (0x8007fd0c, golf case): cry (0x8007f920), health 255, then
     0x8007aff0 unless the tries byte just reached 0 (then
     SwitchEpisode(1)).
   - 0x8007aff0: SwitchEpisode(player, level of hole 0x800c4cd0); for
     that hole up to the current one: total -= strokes, strokes = 0.
3. Last hole (index 8), at or under par: strokes 1: state +0x30 |= 1024
   (a bar at teardown). Speech 0x800b92d0[(rand & 255) % 3]. PostCine(35
   "Golf Ending"). After it (0x8007737c case 34): exit code 1,
   0x80076d6c(8, 1): level 8 played and won, its puzzle piece, the
   main-level count ([objectives](objectives.md#finishing-a-level-0x80076d6clevel-won)).
4. Other holes, at or under par: speech: 1: 70 at 1 stroke; 2: 338 at par
   - 1, else 0x800b92a8[% 5]; 3: 586 at 1, 589 at par - 1, else
   0x800b92b4[rand & 3]; 4: 939 at par - 1, else 0x800b92c4[% 5]; 5: 812
   at 1, 811 at par - 1, else 0x800b92bc[% 3]. Strokes 1: state +0x30 |=
   1024. SwitchEpisode(player, 0x80057dcc(hole + 1)).

The next hole is a normal level load: its player init clears its own
stroke byte, keeps the others, and sets its baby. A hole in one shows
"Great Job! You have earned 1 Reptar bar!" at teardown (0x80094ab0 from
0x800182f0).

There is no stroke limit (bytes wrap at 256) and par is only judged at
holing out.

### Out of bounds

No general out-of-bounds or water rule was found. A ball whose new
position has no COL record stops (0x80086cd0 returns 0); the start and
the old position are clamped into their rooms' boxes (0x8008864c,
0x80086ef0).

One level rule: level 23's per-frame callback (0x80081758) moves a ball
that is not rolling and lies within x +-2000, z +-1000 of the object in
0x800c4168 to (x, 280, 2753). 0x800c4168 is set by the generic init
(0x8004691c) for template 1264, whose model box it rewrites to
(-4000, -2000, -1000)..(4000, 500, ?), or for template 1231 (id 524).
DB23 has template 1264 at (45552, -1050, -3079). Seen: a ball poked to
(45552, -1200, -3079) was moved to (45552, 280, 2753) the next frame.
What the object is (guess: a hazard such as water) is not known.

## Camera

### Golf mode 12 (0x80025c64 kind 12)

Set on entering state 13 (0x800291f8(camera, player, 12, 0) -> case 11
-> 0x80025c64):

- camera +0x4a = 12; 0x800c4a58 = 4096 (a delay); 0x800c4a3c =
  0x80000000; 0x800c4a44 = 0; camera +0x98, +0x9c, +0xa0 = 0.
- Target +0x4c = +0x50 = 0x800de440 (a 36-byte matrix and position).
- Eye offset +0x84..+0x8c = (440, -768 - 0x800c3f5c + camera +0x9c,
  -4096); look offset +0x74..+0x7c = (440, -768, 0) (0x800c3f58 = 768).

Each frame (0x8002953c, mode 12, level >= 22):

- Player not in state 13 or 14: while 0x800c4a58 > 0 it counts down by
  0x800c3f20 (136 a frame: about 30 frames after the stroke). Then, if
  the ball is rolling and d2 to the cup (2D) is below 0x800b6df0[level -
  22] (131072000 for every level 22-31, so 11448), 0x800259a0(camera,
  ball, 0x800c4a50): the room's fixed camera, watching the ball.
- Mode 12 with 0x800c4a3c 0 or negative: eye offset as above (camera
  +0x9c only added while 0x800c4a3c is 0; it was 0 from about 50 frames
  in), target = the player's angles as a matrix (0x80020ee0) and the
  player's position. So the eye sits 4096 behind the player, 440 to the
  side and 2368 up.
- The rest of the camera code follows (0x8001e6fc, [gameplay](gameplay.md#camera-0x8002953c)).

When the ball stops: back to follow (kind 1). In the golf holes the
follow height 0x800c3f5c is 1600 (seen).

Seen: offsets (440, -2368, -4096) and (440, -768, 0); near the cup after
the delay the camera switched to a fixed eye at (31580, -3554, 29640)
with mode 3.

### Camera height (Circle)

0x800972c4: counter 0x800c46d0 +0x862 += 1; at 4 it resets to 0 and
returns the base +0x864; else returns 0x800c4730[counter]: -2200, -6000,
1400. The base is set on entering state 14 (0x80097288(camera +0x9c));
state 15 resets the counter and sets +0x9c = 0. Seen: Circle once gave
+0x9c = -2200 and eye offset y -4568; the swing put it back to -2368.

### "Golf Cam n" cines

Cines 29-33 and 81-84 ("Golf Cam 1" .. "Golf Cam 9", flags 0x58000200)
and 34 "Golf Intro" (0x30000200) are the level-start cinematics above;
35 "Golf Ending" (0x200) ends the course. Their content (guess: a
flyover of the hole) was not read.

## HUD

| item | where | what |
|---|---|---|
| "Par %d" | (-192, -100) | par, from EpisodeStart |
| "Swing " | (50, -100) | |
| "%d" | (150, -100) | strokes; EpisodeStart shows strokes[hole], the walk-up shows strokes[hole] + 1 |
| power meter | (0, 76) | 0x8008cef0, pool 0x800c4d60, drawn by 0x8008cd00 |
| aim line | world | five sprites 364, state 14 |
| Reptar bars | (145, 70) / icon (192, 76) | bottom right in levels 23-31 ([objectives](objectives.md#hud)) |
| "Player %d" / "Hole %d" | (-230, 60) / (-230, 80) | activity play only, in the player's colour |

All texts are 0x80090004 items, y from the screen centre (down
positive).

### The power meter (0x8008cef0)

0x8008cef0(294, (1, 10, 512), (104, 76), 293, (110, 12, 512), (0, 76)):
two HUD quads (0x8008f8f4: half sizes (w, h) at depth 512, at the given
x, y): bar +0xc = model 293, half size 110 x 12 at (0, 76), the frame;
bar +0x8 = model 294, half size 1 x 10 at (104, 76). Bar +0x16 = max
220, +0x1e = value (0x8008d038).

Draw (0x8008cd00): piece 294's corners reset (0x8008fad8), then
vertices 1 and 3 (its +x side) move by

```
a  = value < 2 ? 0 : -(value << 10) / 220        (value < 2 also clears piece +0x16)
dx = ((110 - 5) * a >> 10) * (1024 / 512)
```

so at 220 the piece spans x from about -105 to 103: it grows from the
right end of the frame to the left as the power rises. Seen: a bar at the
bottom centre, the dark part growing leftward from the right end.

## Ambience (0x800818e0)

Each frame, 0x800b9618[level] runs; in activity play the level is picked
by the character (1: 25, 2: 26, 3: 31, 4: 23, 5: 29), so the lines are
the baby's voice. Every golf callback counts 0x800c4cc0 down by dt; at
or below 0 it plays one speech line and reloads 1800 + (rand & 255) * 16
(2700 + .. for 0x8008168c). The line is a fixed one when the player is in
state 15 (598, 270, 347, 820 or 868, 949 or 995), else random from
0x800b9580 (12), 0x800b950c (14), 0x800b9528 (19), 0x800b9568 (11),
0x800b9550 (11). Level 23's callback also does the ball reset above.

## Activity play: Mini-Golf for one to four players

Activities (front end 0x8003b7bc) item Mini-Golf returns -2 and runs
0x80096134:

1. Player count 1-4 (0x800c46cc), Left and Right change it, Cross
   accepts.
2. Each player picks a face on a 3 x 2 grid (slot = x + 3 y; Left/Right,
   Up/Down; Cross). Slot to character: 0x800bb4ac = [1, 2, 3, 5, 4, 37]
   (Tommy, Chuckie, Angelica, the twins, Susie (guess)). Slots go to
   0x800c46d0 +0xfc + player, and bit slot of +0xf8. Triangle (pad word
   +0x34, guess) in the pick step clears every pick and goes back to the
   count; in the count step it leaves with 0.
3. Returns 0x80057dcc(0) = 25.

main (0x800187f0): activity play and mode 25 run 0x80096a30 instead of
the level runner. 0x80096a30:

1. State +0x21 = 1 (activity), hole counter 0x800c46d0 +0x866 = 0.
2. For each hole index 0..8: its own copy of the runner's setup (level
   0x80057dcc(index)); holes 0 and 8 hide 600, 1776, 609, 608, 15. The
   player (411 words, 1644 bytes) and the ball (42 words, 168 bytes) are
   copied to 0x800c46d0 +264 and +1908 (0x80021b48(dst, src, words)). From the second hole on, the score card (0x80095c2c(3,
   count, hole, 1)) shows until Cross.
3. For each player: current player +0x860; "Player %d" and "Hole %d" at
   (-230, 60) / (-230, 80) in the player's colour 0x800bb494 (red, green,
   blue, yellow); character = 0x800bb4ac[slot]; player init
   (0x800596e8). The first player gets the music and the start cine
   (0x80074958 with no player: no CineFinish, no card); the others a
   banner (0x800965e8). Camera reset; the frame loop (0x80018c38).
4. Holing out ends the turn (SwitchEpisode to the same level). Strokes go
   to 0x800c46d0 +0x81c + 9 * player + hole. Player and ball are put back
   from the copies, so each player plays the hole from the tee.
5. Any exit code <= 0 (quit) ends the round. After hole 9 the score card
   (strings 123 "Hole", 124 "Total", 125 "Par"; 0x80095768) shows until
   Cross; returns -1 (the front end).

Activity play has no par judgement (0x8007a9d8 returns first), no
crying, no bars and nothing recorded (0x80076d6c skips activity play).

### Level 22, "Multiplayer Golf"

No code was found that runs level 22 as golf: the activity round uses
levels 23-31. Some golf tests use level >= 22 (0x8002953c, 0x8005be24),
0x800b6df0 has an entry for 22, and the runner sets 0x800c3fb0 for it
([gameplay](gameplay.md#the-level-runner-0x80017c0c)). DB22 holds the
opening cinematic's data ([lp](../formats/lp.md)). Guess: a cut
multiplayer course; the name (string 172) stays.

## Checked in the oracle

A scratch probe on rugrats-hle (one-shot pokes, pokes at a hook, hooks
with arguments, memory dumps at hooks, a per-frame golf trace): presses
start 300, cross 500, 650, 800, 1650, 1800; main's level argument patched
once (0x80018824 = `li a0, N`, restored to `move a0, a1` at 3000);
0x800c5678 = N at the runner's entry; cards skipped (0x8004472c =
0x1000004a).

- Level 25 from the menu: cine 34, then EpisodeStart (Tommy at (35382,
  11, 12372)), then cine 31. HUD "Par 4", "Swing 0" at the top. Tries
  byte 0.
- Tommy poked near the ball: state 13 with heading 957 at (36386, 11,
  12529) (matches 0x8005bac4), state 14 after 56 frames at (37221, 11,
  13170) (425 stance, matches), golf camera mode 12.
- Power: 4 the frame after Triangle, +3 a frame, 94 kept at release;
  hit 13 frames into state 15; sound 118; strokes 1 / total 1.
- Descriptor and record dumped at 0x8008864c: values in The hit and The
  flight record. Roll on a flat green: speed -15 a frame, y = floor -
  104; a wall bounce (reflect, times 3000 / 4096); stop, camera back to
  follow, state 13 again because the ball stopped within reach.
- Aim: Right +25, Left -25, Square+Left -8 a frame; the player orbits the
  ball. Circle: camera +0x9c -2200.
- Putt, ball poked 1500 short, power 55: holed at 334 from the cup
  (speed 383), speech 70, SwitchEpisode(28), "Great Job! 1 Reptar bar".
  Hole two (level 28): Chuckie, cine 81, Par 5, strokes [1, 0, ..]
  total 1.
- Over par (strokes poked to 4, par 4): speech 1213, ForceCry, the "You
  were over par. Try Again" box until Cross, cry, 0x8007aff0,
  SwitchEpisode(25), cine 31 (previous level 25), strokes all 0.
- Power 202 straight at the cup: lip out (sound 119, 0x80057f78 with
  speed 2175), the ball came back; after the delay the fixed camera
  (0x800259a0) watched it.
- Level 27 (hole nine) putt: speech 1202, cine 35 (character set to 1),
  then 0x80076d6c(8, 1), exit 1, the puzzle-piece screen.
- Level 23: Lil (char 4) with cine 29; the ball reset to (45552, 280,
  2753).
- Activities, Mini-Golf, two players: the pick screen, then 0x80096a30,
  hole 1 with "Player 1" / "Hole 1" in red at the bottom left.
- Not checked: speech lines' words, the Circle+Cross camera adjust,
  slopes (slope_push), object hits during a roll, the last-try speech,
  Select's restart, the loading screen's text, the score card, levels
  24, 26, 29-31 beyond their table values.

## For the port

Order (port code is crates/rugrats/src/game.rs and friends):

1. Data: the nine holes' tables (par, character, cine, start), the hole
   index maps, templates 724 and 789 found by the class-0 constructor,
   LP placements. The strings 195, 196, 202-204, 214, 232, 123-125 and
   173-181, 158.
2. EpisodeStart 0x8007a584 and player init 0x80057e50 (strokes, baby),
   the start cines and their hooks (31, 33, 35), card HINT27 after cine
   34.
3. Player: the walk-up test, states 13-15 with the stance placement, the
   meter's per-frame bounce, aiming in state 14, the club models, no
   jump, no stick while the ball rolls.
4. The ball's roll: the descriptor, k = (power << 16) / 6000, the >> 6
   integration, 0x80086b58's bent segment, the world segment test and
   reflection with bounce 3000, the rolling friction 15, slope_push, floor
   snap y = floor - R. Needs rsin/rcos, ratan2, the asin/acos table
   (0x800b4598), isqrt (0x8001f4ec) and VectorNormal to match the
   oracle exactly.
5. The think: cup radius 384, speed limit 1441, lip out, holing out,
   stop, camera back to follow.
6. Holing out: par judgement, speech, the over-par box, ForceCry and the
   retry, the next hole, the last hole's cine 35 and level 8's piece, the
   hole-in-one bar.
7. Golf camera mode 12, the 4096 delay and the fixed camera near the cup,
   the Circle height cycle.
8. HUD: three texts, the meter quads, the aim sprites.
9. Level 23's ball reset; the per-level speech.
10. Activity play: setup, the round driver with snapshots, score card.

## Unknown

- What +0xe0 bit 2048 is in animation 31 (the strike frame), and what
  returns state 15 to 0.
- The stack leftovers in the descriptor (+0x50, +0x54 to 0x80086114,
  +0x58..+0x60) and what the record uses them for.
- The object scan 0x80086340 and the world test 0x8002cd7c in detail
  (which normal a wall gives, which objects stop the ball; whether the
  cup can block it).
- Whether the Circle+Cross+Up/Down camera adjust does anything.
- The level 23 object (template 1264) and the template 1231 object (id
  524); ids 9999 (DB26), 9998 and 6660 (DB31).
- 0x8007614c's sign (which side a lip out turns to).
- The loading screen's text for the holes (read only); the score card's
  layout (0x80095768).
- Whether a Select restart (0x80043d58) leaves the total too high (the
  hole's byte is cleared, the total is not); nothing in single play reads
  the total.
- Speech line contents; whether level 8 ever loads its own database.
