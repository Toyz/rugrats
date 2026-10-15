---
title: Pickups, level goals, hint cards, HUD and pause
status: partial
discs: US
covers: US SLUS_006.50:0x800601e4, US SLUS_006.50:0x80060910, US SLUS_006.50:0x80060b44, US SLUS_006.50:0x8005ff88, US SLUS_006.50:0x80062ac4, US SLUS_006.50:0x80061f88, US SLUS_006.50:0x8008864c, US SLUS_006.50:0x80087c5c, US SLUS_006.50:0x8004df94, US SLUS_006.50:0x800b7e90, US SLUS_006.50:0x800514b0, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x8007914c, US SLUS_006.50:0x8007c6e0, US SLUS_006.50:0x8007eb68, US SLUS_006.50:0x80077864, US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80091a44, US SLUS_006.50:0x80094bfc, US SLUS_006.50:0x800930c8, US SLUS_006.50:0x80044434, US SLUS_006.50:0x80097788, US SLUS_006.50:0x8008ba70, US SLUS_006.50:0x8008c7f0, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8003e05c, US SLUS_006.50:0x800398bc, US SLUS_006.50:0x80035fec, US SLUS_006.50:0x800bb4e8, US /DATA/HINTS
worklog:
---

# Pickups, level goals, hint cards, HUD and pause

What the player does in a level beyond moving: carrying and throwing,
collecting, the goal that ends a level, the progress the game keeps
(puzzle pieces and Reptar bars), the hint cards, the HUD and the pause
menu. Read from the pseudo-code and the disassembly only; not yet checked
frame by frame against the oracle. Builds on [gameplay](gameplay.md)
(player, objects, events, exit codes) and [text](text.md) (text items).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn.

## The string table (0x800bb4e8)

Menus, HUD and training messages index one pointer table at 0x800bb4e8.
The ones used here:

| idx | string | idx | string |
|---|---|---|---|
| 1 | Game Paused | 88-90 | Help Tommy win / twelve puzzle pieces / by completing levels! |
| 2 | Continue? | 91-93 | Congratulations! / You earned a / puzzle piece! |
| 3 | Save Game? | 94, 95 | Select Piece, Place Piece |
| 4 | Quit Game? | 96 | Terrific! |
| 5 | Load Game? | 97-99 | You have already / earned the puzzle / piece in this level. |
| 6 | Quit Level? | 100, 101 | Great Job!, You have earned |
| 11, 12 | Yes?, No? | 102-105 | 1 Reptar bar! .. 4 Reptar bars! |
| 24 | Loading | 228, 230, 231 | You did it!, You've found all the, Reptar Puzzle Pieces! |
| 67 | PRESS     TO CONTINUE | 237 | No Hint Available. |
| 71-75 | Returning to the / Activities Menu / Returning to / Main Menus / Pickles' House | 238-272 | training messages 1-35 |

## Carrying

### Object and player fields

| where | field | meaning |
|---|---|---|
| object +0x00 | bit 0x10000 | can be picked up (template +0x28) |
| object +0x00 | bit 0x1000000 | carried but not throwable: ACTION uses the touched object instead (0x8005d11c) |
| object +0x00 | bit 0x10000000 | in flight (set at launch, cleared on landing) |
| object +0x6b | u8 | 255 when its flight ended (written by the flight, 0x80087c5c) |
| object +0x6d | u8 | damage / value (thrown hits, cookies, gauge fills) |
| object +0x70 | ptr | object it rests on; +0x74 list of objects resting on it |
| object +0x90 | ptr | its flight record (0x8008864c) while flying |
| player +0xe0 | bit 2048 | grab point of the animation (guess: an animation event) |
| player +0xe0 | bit 16 | release point of the throw animation (guess: same) |
| player +0xe0 | bit 0x8000000 | animation finished (0x80056484) |
| player +0x18e | s16 | carried y minus player y at the grab |
| player +0x1ec | ptr | object being picked up |
| player +0x1f0 | ptr | object carried |
| player +0x1f4 | ptr | throw target (levels 9, 20, 21) |
| player +0x1f8 | s16[3] | grip offset, in the player's frame |
| 0x800c4290 | s16 | jump lock after a throw: 300 (1 s) |

### GRAB and ACTION in the stand state (0x8005d11c)

GRAB is Circle (0x800c4988), ACTION Triangle (0x800c49d4), both on press
(`& 3 == 1`), never with a throw pending (+0x188 bit 4) or AI driving
(+0x00 bit 0x4000).

GRAB:

1. Carrying, not in levels 5, 20, 21 and not a pie (template 564): if
   0x80063888 says there is room, state 8 (put down).
2. Else 0x800631d8 finds the touched object (+0xf4), skipping the one
   stood on (+0xf8).
3. Touched object with bit 0x10000, outside golf (23-31): with a use
   callback (+0x54) call it as (obj, player, 0); a result of -1 means
   "pick me up". Without one, and with empty hands, pick it up. Pick up =
   +0x1ec = obj, state 7.
4. Touched object without bit 0x10000: call its +0x54, except templates
   15, 732, 733.

ACTION:

1. Carrying an object without bit 0x1000000: request a throw (+0x188 |=
   4, animation 18, blend 0x2000). Levels 20-21 need a target in +0x1f4
   and play speech 1159 or 1167.
2. Else level 3 with state +0x30 bit 8: +0x1f0 = 0x800c4cd4 (the
   level's own throwable).
3. Else the touched object's +0x54, if any: state 12 (use).

0x80063888(player, obj) tests the put-down spot: the object's first model
box (model +0x0c, 16-byte boxes `s16 min[4], max[4]`, count at model
+0x0a), centred and turned by the object's angles, grown by its larger
half-extent, against every object in the player's nearby list (+0x100).
1 = free. It also returns 1 when 0x800c4304 is set or +0xff is 0.

### Pick up (state 7, 0x800601e4)

Waits for +0xe0 bit 2048, then:

1. +0x1f0 = +0x1ec. A flight still running is cancelled (0x80088600,
   +0x6b = 255). It leaves the object it rested on (+0x70's +0x74 list)
   and any objects resting on it are dropped (0x80062928 each: a short
   fall, see Flight). It leaves its room's object list (room +0x48).
2. Grip (0x8005ff88 mode 0): hand slot 15 and hand slot 12 of the
   carrier's hierarchy (slot to part through the byte map at hierarchy
   +0x14; part positions from 0x80044fc8 = object +0x158 + 32 * part +
   0x14). The grip is the midpoint of the two less slot 15, turned into
   the player's frame by the transposed rotation (0x80010b54,
   0x8009fc38), and pushed forward (+0x1fc) by half the object's model
   depth (0x8005fd58: the largest `max z - min z` of its boxes). +0x18e =
   object y - player y. The object takes the player's room.
3. Specials: template 933 creates a gauge (0x8008c7f0, models 360 / 361,
   kept in 0x800c4df8) and swaps hand models (1690); levels 20-21 with
   template 886 swap the hand part model; character 6 swaps part slot
   0x24 to model 1212-1214 for templates 678-680 and barks (speech 193,
   159, 234 or 235). Level 16 shows training message 5.
4. Animation 20 (0x80057244), +0x1ec = 0.

Bit 0x8000000 (animation done) returns to state 0.

### What the carried object follows

Every frame the carrier's draw (0x8003205c, after its parts) runs
0x8005ff88(carrier, 1) and then the carried object's draw (+0x40):

- object room = carrier room; object angles +0x24..+0x2a = carrier's;
- object matrix = carrier matrix; object position = hand slot 15's
  position + carrier rotation * grip (+0x1f8, 0x8009fbe8).

So the object rides hand slot 15 at a fixed offset (guess: slot 15 the
right hand, 12 the left). Level 3 with state +0x30 bit 8 also runs
0x8005fe44 (an aim line ahead of the player).

### Put down (state 8, 0x80060910)

Waits for +0xe0 bit 2048, then 0x80062ac4(player, 0) and animation 17.
Bit 0x8000000 returns to state 0. Template 1016 in level 16: training
message 8. Template 933 frees its gauge and restores hand model 1310.

0x80062ac4(player, thrown):

1. 0x80060810: a wall test from the player to the object (0x8006c438);
   if blocked, the object is moved back by the hit's offset.
2. Character 6, not thrown: part slot 0x24 back to model 79, the object
   pushed 1024 ahead (or 0 when +0x699 is 14), speech 190.
3. Room from the object's position (0x8002b660; -1 keeps the player's),
   moved between room lists (0x8004a440).
4. A drop flight: direction (0, 0, 4096) turned by the player's angles
   and normalised, speed 10 (0x80061f88, 0x8008864c). If no flight slot
   is free it is set straight on the floor (0x8005fdb4, +0x148).
5. Flight flag, think = 0x8004df94 on the scheduler, +0x1f0 = 0.

### Throw (0x80060b44, every frame from the player update)

Runs while carrying with +0x188 bit 4, at the release point (+0xe0 bit
16, or state +0x30 bit 64):

1. Level 1, templates 678-680 or 716 (toys for the dog): the dog
   (character object 14, 0x800461f4) barks (speech 191/192); if it waits
   (+0x699 = 2) at the player's position it fetches (+0x699 = 12, +0x66e
   = 4, +0x678 = the object).
2. Room from position, room lists (0x8004a440).
3. Direction: levels 20-21 the stored aim (+0x200..+0x20c) and
   0x800574cc; else (0, -384, 1024) (0x800b7fe0) turned by the player's
   angles, normalised, speed 170 (0x800b7fec); 0x80060a64 pulls the
   object back to the midpoint if a wall is between.
4. Flight (0x80061f88, 0x8008864c) into +0x90; flight flag; think =
   0x8004df94; then the object's +0x48 and +0x50.
5. A counted pickup: when 0x800c4298 exists, 0x800c4bd6 -= 1 (not below
   0) and the gauge shows it (0x8008ca80).
6. +0x1f0 = 0, animation 17, jump lock 0x800c4290 = 300, +0x188 bit 4
   off.

Level 16 with a finished animation and no throw shows training message 6
(27 for the pie, template 564).

Aimed throws replace the direction (0x80061c98): level 9 aims at +0x1f4
with vy -300 when the turn is under 513; level 3 (+0x30 bit 8) and level
16 (0x800c56a8 bit 8) roll: flags 0x0a070400, direction (0, 35, 1024),
speed 225 near object 75 / 600 (within 0x32c8) else 450.

### Flight (0x80061f88, 0x8008864c, 0x80087c5c)

0x80061f88(obj, desc, thrower, dir) fills a launch descriptor:

| desc | value |
|---|---|
| +0x00 | flags: 0x09070700, or 0x190f0700 with object bit 0x800000; level 9 adds 0x40000; pie adds 0x40000000 |
| +0x04..+0x0c | start: the object's box centre + position |
| +0x10 | radius: the larger of the model box's \|min y\|, \|max y\| |
| +0x20 | &obj +0x18 (the flight writes the position back) |
| +0x24 | &obj +0x6b (set to 255 when done) |
| +0x2c..+0x38 | direction (4096 unit) and speed |
| +0x44 | bounce (guess), obj +0xa2 clamped to 4095 |
| +0x4c | obj +0x5c callback |
| +0x64 | sound: template +0x24 (or +0x2e with object bit 0x40000) |
| +0x6a | damage, obj +0x6d |

0x8008864c takes a record from the flight pool (0x800c4cf0, count
0x800c4d00), clamps the start into its room's box, and sets the velocity
v = dir * (speed * 65536 / 6000) >> 12 (170 gives 1856 a component at
full; a magnitude of 1856). Each frame (0x8008851c(dt), list 0x800c4ce8,
0x80087c5c):

- gravity by flags bits 8-11: 0x100 -6, 0x200 -3, 0x400 1, 0x500 3, 0x600
  6, 0x700 13, 0x800 27, else 0. Thrown objects (0x700) fall at 13, the
  player at 14.
- integrate (0x8004fbb4): pos += (v dt + a dt^2 / 2) >> 6; v += a dt.
- collide when flags bits 16-19 are set: against objects too (0x80086ef0)
  with flag 0x10000000, else the world (0x80087594). A hit on a character
  posts Damage(player, character, +0x8e) (0x80086ef0, 0x80087594).
- done when: flag 0x1000000 and the floor was hit (result bit 2), flag
  0x2000000 and a wall (bit 1), flag 0x20000000 and bit 8, or the
  lifetime +0x0c runs out (0x800b44d4[dt] a frame).
- done: *(+0x34) = 255 (the object's +0x6b).

The thrown object's think 0x8004df94: while +0x6b is not 255 it only
keeps its room up to date (0x8004a34c). At 255: clears the flight flag;
if the flight hit an object (+0x98), 0x8006c7a0 decides which rests on
which (+0x70 / +0x74); +0x90 = 0, +0x6b = 0, +0x44 = 0, returns -1 (off
the scheduler). In training (state +0x30 bit 64) it posts ForceCry and
clears bit 64.

### Use callbacks (+0x54, table 0x800b7e90 by template +0x07)

| idx | function | does |
|---|---|---|
| 0 | none | |
| 1 | 0x8004d3f0(obj, player) | the Reptar puzzle board: with state +0x30 bit 512 (set by level 7's start, 0x8007b748) SwitchEpisode(player, 7); else the puzzle screen 0x8009480c(0) |
| 2 | 0x8004d434(obj) | breaks it: model shards (0x8004cdfc, 100), sound 114, a burst of five sparks (0x80054894, sprite by room), hides it (0x80078bc4(obj +0x38)), then 0x8007d0f4(player): records the room in 0x800c4cb8[0x800c441d++], clears +0x54 of characters 207-212 but one, and on the first sets the checkpoint (guess: the hide-and-seek spots); returns 1 |
| 3 | 0x8004d49c(obj) | training message 34 for templates 111 and 441 ("Some objects will launch a new episode."), 33 for 117, 715, 720 ("You can only open these..."), else 0; returns 1 |
| 4 | 0x8004d514(obj, player) | Damage(player, 0, -10): +10 health, a cookie; returns 1 |

Entries 5 on belong to the next table (0x800b7ea4). Doors and cinematic
objects get 0x8004f78c / 0x8004f83c instead (see gameplay).

### Collectibles (touch, 0x800514b0)

Template +0x0a = 10 gives +0x58 = 0x800514b0 (table 0x800b7e50). On touch
it bursts three sparks, leaves the room list and, for object class
+0xa6 = 11, acts by kind +0xa9:

| kind | sound | effect |
|---|---|---|
| 1, 2 | 30, 31 | player +0x1a9++; text shows 0x800e2744 (or 0x800e2740) - count; at 0, 0x800c4430 = 1 |
| 3 | 28 | Reptar bar: 0x80094a58(level, bar +0xa8); HUD bar counter; level 16 message 3 |
| 4, 14 | 29 | cookie: health += +0x6d (max 255), health meter shown |
| 6-11 | 113 | player +0x1a9++; remaining 0x800c4b94 - count; at 0, 0x8007d250 |
| 12 | 109 (level 12) / 69 (level 5) | carried object's +0x6d += +0x6d, into its gauge |
| 15, 17 | 31 | level 11 / 17 specials |

## Progress (the game state, 0x800c5678)

Adds to the fields in [gameplay](gameplay.md). 328 bytes; cleared for a
new game by 0x80091a44 (via 0x80094cfc).

| offset | type | meaning |
|---|---|---|
| +0x20 | u8 | health at the last checkpoint (0x800772e0) |
| +0x22 | u8 | main levels completed (0x80076d6c) |
| +0x23 | u8 | from 0x800b2920; ForceCry counts it down (guess: tries) |
| +0x24..+0x2c | s32[3] | checkpoint / respawn position |
| +0x30 | u32 | flags: 1 use the respawn, 2 checkpoint reached (0x800772e0), 8 a level's minigame phase, 16 Mirrorland done (doors open), 64 training, 512 final level reached, 1024 a bar to award at teardown |
| +0x34 | u32 | bit per level: played (0x80076d6c) |
| +0x38 | u32 | bit per level: completed |
| +0x3c | s32 | counter, ++ in 0x80094cc8 (unknown) |
| +0x40 | u32 | bit per level: its puzzle piece was given (0x80094bfc) |
| +0x44 | u16 | pieces earned, a bit each (12) |
| +0x46 | u16 | pieces placed on the board (0x80092754) |
| +0x48 | u32[32] | Reptar bars found, a bit per bar, per level (0x800c56c0) |
| +0xc8 | u32[32] | bars spent on pieces (0x800c5740) |

Helpers: 0x80094b8c pieces earned (bit count of +0x44, at most 12);
0x80091d28 pieces placed; 0x80094a0c bars available = found - spent
(0x80091a80 counts all 32 words).

### Puzzle pieces

Twelve pieces. Each of levels 1-6, 8-12 and 15 owns one (0x800919b4).

- 0x80094bfc(level): unless 12 are earned, if the level owns a piece and
  +0x40 bit `level` is clear: set it and award a random unearned piece
  (0x80091d50: `rand & 255) % 12` until free; with +0x30 bit 512 any),
  then the puzzle screen 0x8009480c(level). Level 1's piece comes after
  cine 7 "Reptar Puzzle" (0x80074958).
- 15 unspent bars buy a piece: 0x80094924 (after each bar, 0x80094a58)
  marks 15 bars spent (0x80091c14) and awards one.
- The puzzle screen (0x8009480c -> 0x800943bc -> 0x800930c8) shows "Help
  Tommy win / twelve puzzle pieces / by completing levels!" when only one
  piece is earned and the level is 1, else "Congratulations! / You
  earned a / puzzle piece!", then Select Piece / Place Piece (placing
  sets +0x46). Text items at 0x800e7300 (44 bytes each, shadows at
  0x800e7390). From the board (level 0) no message is set up
  (0x80040194(-1), 0x80091e4c).
- With all 12 placed (0x80094978), the screen 0x80094178 "You did it!
  You've found all the Reptar Puzzle Pieces!" (sound 2046) and the next
  level is 7, Reptar 2010.

### Reptar bars

Bars are numbered per level (object +0xa8). 0x80094a58(level, n) sets
+0x48[level] bit n. Bonus levels award bars at the end by setting bits
31 downward (0x80094b10) and show "Great Job! You have earned n Reptar
bars!" (0x80094ab0 -> 0x80093b6c): Cookie Race (13) 2, or 4 by
0x80076c24; level 14 2; Gold Rush (17) 3, or 4 by 0x80076ba8; Egg Hunt
(18) 3, or 4 by 0x80076bf0; 20 and 21 3. A level ending with +0x30 bit
1024 gets 1 (teardown, 0x800182b0).

### Finishing a level (0x80076d6c(level, won))

Called by the ending cinematics (0x8007737c, see gameplay) and the
teardown. Nothing is recorded in activity play (+0x21). Otherwise:

1. +0x34 |= bit `level`.
2. Won: bonus bars as above; then, with fewer than 12 pieces, the level
   piece (0x80094bfc) and the all-placed check (0x80094978).
3. All placed and not +0x30 bit 512: +0x38 bit, SwitchEpisode(7).
4. Else, first completion: +0x38 |= bit; for levels 2-6 and 8-12, +0x22
   += 1, and at 2 / 3 / 5 / 7 the bonus level 20 (No More Cookies) / 15
   (Mirrorland) / 21 (Touchdown Tommy) / 17 (Gold Rush) starts next
   (SwitchEpisode, +0x30 bits 1-2 cleared).
5. Level 15 (Mirrorland): +0x30 |= 16 and SwitchEpisode(1).

### Back to the hub

Exit code 1 runs level 1. Its EpisodeStart callback (0x80078e7c) puts
the player at 0x800b8718[previous level] (16 bytes: x, y, z, 0) unless
the previous level is -1 or 16, and resets the camera (0x80026ddc).
Level 2's spot is (37002, 779, 9866). Pause "Quit Level" also sets
0x800c40ac, and the runner then places the player with 0x80070448
(0x80017fc0).

## Level 2, Grandpa's Teeth

A three-phase level; the phase is 0x800c4cd0, set with the checkpoint
by 0x80078d60(pos, angles, phase) (position 0x800e5830, angles
0x800c4cc8). Character 11 carries model 190 on its part slot 0x1f (guess:
the goose with Grandpa's teeth); 1776 is Chuckie; 14 the dog Spike
(guess, HINT04).

1. Start (EpisodeStart callback 0x8007914c): seven fixed-camera flags
   set; without a checkpoint (+0x30 bit 2) Chuckie follows Tommy
   (0x80083764), characters 900-904 get a trigger (0x80049e18, radius
   2000; guess: the geese), 335 is hidden; with one, 335 is shown. 1313
   hidden. Checkpoint (1720, -245, 5608), phase 1. Room sounds for rooms
   0, 68, 45 (0x80078e3c). Card HINT03 after cine 1.
2. Cine 3 "Goose Transition" ends: card HINT04 ("Ride on Spike to chase
   the goose") and 0x8007c08c: phase 2, ride animations for character
   6. The goose cannot be hurt in phase 2 (0x80077864).
3. Cine 4 "Shuffle Board Transition" ends: card HINT05 ("kick pucks at
   the goose. Be sure to miss Chuckie!") and 0x8007c6e0: +0x30 bit 8,
   Tommy fixed at (0x91eb, -236, 0x5bf8) facing 2048 and frozen (+0x188
   bit 1), a fixed camera, goose health 200 (+0x1a4, max +0x66c),
   Chuckie's health reset, two gauges on the goose and Chuckie (0x8008c5e0,
   models 278 / 279 with 282), timer 130 s (0x80078dd8), phase 3.

Win and lose (0x80077864 with +0x30 bit 8 -> 0x8007eb68): each hit takes
the puck's damage off the target's +0x1a4 (not below 0) and updates its
gauge (0x8008ca4c). The goose at 0 posts Cine 5 "Episode 1 Endiing";
Chuckie at 0 is ForceCry (back to the checkpoint). The timer running out
posts TimerExpired with ForceCry (0x8007f5a8 -> 0x8007ecc4).

After cine 5 (0x8007737c): respawn (0x901c, 1001, 0x2839) into +0x24,
+0x30 |= 1, exit code 1, 0x80076d6c(2, 1): the level's piece and the
level count. Cine 16 "Episode 1 2nd Ending" records a loss
(0x80076d6c(2, 0)); 0x8007ec48 posts it (its caller is not found).

## Level timer (0x800c4430)

0x80078dd8(s) sets it to s * 300 (dt units). The heartbeat (0x8007733c)
runs 0x8007f5a8 while it is set and nothing is frozen: it counts down by
dt and draws the seconds ("%ld", value / 300) at the top with the clock
model 375 (0x800c464c). At 0: sound 117 and TimerExpired with a
per-level callback: 2 0x8007ecc4, 3 0x8007e3c0, 4 0x8007e2e8, 11
0x8007dc44, 15 0x8007d4e4, 16 0x8007ea10, 17 0x80079b38, 18 0x800797ec.

## Hint cards (0x80044434)

The cards between cinematics and play are text files, /DATA/HINTS/
HINTnn.TXT, file numbers 672-713 (HINT01-HINT42). 0x80044434(file, arg):

1. Loads the file (load_file) and splits it at CR LF; each line is a
   text item (font 0, spacing 1, grey 0x7f7f7f) centred (0x8002e144
   flag 1), y from -90 down by 18, with a shadow item 2 pixels right and
   down in 0x010101.
2. "PRESS     TO CONTINUE" (string 67) at y 80, white with a shadow; a
   window from 0x8003f444 (drawn by 0x8003f934).
3. Each frame: the speech stream (0x8008d654), every 30th frame
   0x80034e80, the texts, the window, a flip; until Cross is pressed
   (port 0 word +0x3c `& 3 == 1`, sound 126).
4. Keeps (file, arg) in 0x800c4084 / 0x800c4086. In play, L1 and R1
   pressed together show it again (0x8005b1a0 -> 0x800448c8); the level
   loop clears it on exit (0x8001916c).

In the text, `@`, `#`, `%` and `^` stand for buttons (guess from the
wording: Triangle, Circle, Cross, Square); each file ends with a `~`
line.

Who shows which card:

| card | file | shown |
|---|---|---|
| HINT01 "Search the house for objects that lead to levels..." | 672 | after cine 7 Reptar Puzzle (0x800783d8) |
| HINT02 "...press the Select button to restart levels." | 673 | no caller found |
| HINT03 "Enter the opening in the fence to find Grandpa's teeth..." | 674 | after cine 1 Episode 1 Intro |
| HINT04, HINT05 | 675, 676 | after cines 3 and 4 (level 2) |
| HINT08 | 679 | after cine 2 Episode 2 Intro |
| others up to HINT39 | 677-710 | after other cinematics: 0x800783d8 maps cine to card in its switch (table 0x800154bc) |
| HINT15 | 686 | Gameplay script 2 (0x8007e198) |
| HINT40 "Make your way through the training level..." | 711 | level 16's start (0x80079090) |
| HINT41 "To go on this adventure you'll need more puzzle pieces." | 712 | a locked door (0x8004f78c) |
| HINT42 "You need to complete the puzzle to get to Reptar 2010..." | 713 | the puzzle screen with 12 pieces earned (0x800925ec) |

The "Help Tommy win twelve puzzle pieces" text is not a card: it is the
puzzle screen's (0x800930c8).

### Training messages (0x80097788)

Shorter boxes, strings 237 + n (0x800bb89c[n]), shown only in level 16,
or in level 1 for n = 33 and 35. Each n has a remaining-shows count
(0x800e7f08[n]) and a 1500-tick cooldown (0x800e7e90[n]); one shows for
1500 ticks (0x800c4738) or until Cross (0x8001bda4(port, 15)), sound
126. While one shows the world is frozen (0x80097584 returns 1, see
gameplay).

## HUD

Set up outside level 1 by the runner, drawn in layer 1.

| pool / setup | draw | what |
|---|---|---|
| 0x800c45c0, 48 bytes (0x8008c094 -> 0x8008bcd4) | 0x8008c0fc -> 0x8008bdf4 | health meter, levels 2-6, 9, 12, 14, 16 (switch 0x800128a0) |
| 0x800c4d48, three 76-byte gauges (0x8008c584(3)) | 0x8008cad0 -> 0x8008c394 | gauges: a fill of value / max |
| 0x800c4d60 (0x8008ce78(1)) | 0x8008cfec -> 0x8008cd00 | a sliding power bar (0x8008cef0) |
| 0x800c4d78 (0x8008d348(1)) | 0x8008d4b4 -> 0x8008d2fc | a second bar (0x8008d3b8) |
| 0x800e6c80 .. 32 x 16 bytes cleared (0x8008a714) | | unknown |
| 0x800c461c, at most 8 x 60 bytes (0x8008e824(128)) | 0x8008e7ac (world pass) | unknown, not HUD |

Pieces are HUD models placed by 0x8008f8f4(model, scale, pos, ...) and
drawn with 0x8008fa10 / 0x8008fa4c / 0x8008fa9c.

Health meter (0x8008ba70): models 258 (frame), 260, 259 (two fills), or
289 / 288 for character 6. Max = the player's +0x1a4 at creation (+0x16).
It slides in for 1500 ticks after any change (0x8008c124, from Damage
and cookies) and slides out over the last 300. At 0 health it refills to
max (0x8008bd10).

Gauges: 0x8008c7f0(model a, pos a, scale a, model b, ..., max) (pickup
933: models 360 / 361; 0x8007a094: 366 / 365; level 12 and cine 37:
360 / 361) and 0x8008c5e0, which follows an object's +0x1a4 (level 2's
goose and Chuckie). Value +0x32 / +0x42, max +0x2a / +0x3a.

Bars: 0x8008cef0 (models 358 / 357 in 0x80058a78; golf swing 0x8005fb38,
model 293) and 0x8008d3b8 (models 356 / 355, 0x800588e0, level 11).

Counters drawn as text with an icon (0x80090004 text + model):

- Reptar bars: on touching a bar, model 291 and the bars available
  ("%d", 0x800c4194), kept in 0x800c4650. Text at (-150, -80), icon at
  (-192, -76): top left (y is from the centre, up negative). Levels
  17-18: (-150, 80) / (-192, 76), bottom left; levels 23-31: (145, 70) /
  (192, 76), bottom right.
- Level timer: model 375, see Level timer.
- Kind 1, 2 and 6-11 collectibles: a remaining-count text kept in player
  +0x220 (0x80090214).

A normal level (level 2 before its last phase) shows only the health
meter and, briefly, the bar counter.

## Pause (0x8003e05c)

1. 0x80090234(0), save the GTE state (0x8009fd08); fade in (0x80040268,
   0x8003f41c) for two frames.
2. Two counters: pieces earned (0x80094b8c, "%d") with model 292 at
   (-192, 76), bars available (0x80094a0c) with model 291 at (192, 76).
3. The menu 0x800398bc(0, 0, 1, 1); the counters are removed.
4. Result -1: "Returning to the / Activities Menu" (activity play) or
   "Returning to / Main Menus", with "Loading" (string 24), over the
   front end's backdrop until its window closes. -2: "Returning to /
   Pickles' House". Anything else: 0x80090234(1).
5. Restore the GTE state, sample the pads twice, return the result.

The loop uses it as the exit code (see gameplay).

### The menu (0x800398bc, mode 0)

Laid out by 0x80035fec (no pad: "No Controller" / "Unsupported
Controller." instead):

| cursor | item | x, y |
|---|---|---|
| | Game Paused | centre, -60 |
| 1 | Continue? | -185, -30 |
| 2 | Save Game? | -145, -10 |
| 3 | Quit Game? | -60, -30 |
| 4 | Load Game? | 30, -10 |
| 5 | Quit Level? | 80, -30 |

Left / Right move by 2, Up / Down between odd and even (sounds 124, 125).
Cross picks (sound 126); Triangle backs out (state 0: continue).

- Continue: returns 0.
- Save Game: the card save path (state 3, 0x80070d80).
- Load Game: state 9; a confirmed load returns 1.
- Quit Game / Quit Level: a confirm (0x80036368: the question, Yes? at
  1, Save Game? 2, No? 3 (the default), Load Game? 4). Yes on Quit Game,
  or on Quit Level in level 1, 16 or activity play, returns -1. Yes on
  Quit Level in another level returns 1 (to Pickles' Home) and sets
  0x800c40ac.

The result variable only takes 0, 1 and -1 in this function, so the
pause's -2 branch ("Returning to Pickles' House") is not reached from
here (guess: dead, or left from an older menu).

## For the port

1. Progress block (+0x34..+0x148) and its helpers (bit counts, piece
   award with the game's rand, bars spent), with 0x80076d6c. Testable
   offline against saves.
2. Hint cards: parse /DATA/HINTS, CR LF lines, the 18-pixel layout and
   shadow, Cross to close, L1+R1 replay.
3. Pick up / put down / throw with the grip on hand slot 15, then the
   flight record (gravity 13, the >> 6 integration, floor stop) and the
   landing think.
4. Use callbacks 1-4 and the collectible kinds 3 (bars) and 4 (cookies);
   the health meter and bar counter.
5. Pause menu mode 0 and its three results.
6. Level 2's phases, timer, gauges and the Damage routing; then the
   puzzle screen.

## Unknown

- What sets +0xe0 bits 2048 and 16 (guess: animation key events) and the
  hand slots' real names (12, 15).
- The world and object collision of a flight (0x80087594, 0x80086ef0),
  the bounce (+0x44), and 0x8006c7a0's stacking rule.
- The +0x50 table 0x800b7eb0: its entries (0x80053604, 0x80052ba0,
  0x80052ed8, 0x80053090, 0x80053170) read object +0x98 and spawn
  particles; what +0x98 holds.
- Use callback 2's 0x8007d0f4 / 0x8007d02c (which level, what the six
  characters 207-212 are).
- What posts cine 3 and cine 4 in level 2 (room events, LP +0x44 or
  triggers), and who calls 0x8007ec48 (cine 16).
- Characters 11, 14 and 900-904 in level 2 (goose, Spike, geese are
  guesses).
- The second argument of 0x80044434 (4-9), and how `~` and the button
  glyphs are drawn by the font.
- The puzzle screen's input and drawing (0x80092754 and neighbours), and
  the "You've found all the Reptar bars for this level" screen (strings
  223, 224).
- State +0x23 and +0x3c; flag bits 32, 128, 256 of +0x30.
- The 32 x 16-byte table at 0x800e6c80 and the 8-entry pool 0x800c461c.
- The pause's -2 path; the menu's save / load states (3-31).
- Whether level 1's EpisodeStart placement always overrides the respawn
  point set by cine 5.
