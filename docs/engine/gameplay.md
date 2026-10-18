---
title: Gameplay loop
status: partial
discs: US
covers: US SLUS_006.50:0x800185f8, US SLUS_006.50:0x8003cf80, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x80018c38, US SLUS_006.50:0x80018ad0, US SLUS_006.50:0x80018598, US SLUS_006.50:0x80074958, US SLUS_006.50:0x8005a068, US SLUS_006.50:0x8005be24, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x8005d11c, US SLUS_006.50:0x80061330, US SLUS_006.50:0x8005ee78, US SLUS_006.50:0x80066014, US SLUS_006.50:0x8005cb48, US SLUS_006.50:0x8005bc80, US SLUS_006.50:0x800627a0, US SLUS_006.50:0x800596e8, US SLUS_006.50:0x8001b028, US SLUS_006.50:0x8001a888, US SLUS_006.50:0x8002a65c, US SLUS_006.50:0x8002b240, US SLUS_006.50:0x8002af20, US SLUS_006.50:0x8002b660, US SLUS_006.50:0x8004a218, US SLUS_006.50:0x8002953c, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x80025e90, US SLUS_006.50:0x800259a0, US SLUS_006.50:0x80027304, US SLUS_006.50:0x80072bf0, US SLUS_006.50:0x80072d38, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x80077070, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80030b80, US SLUS_006.50:0x80033818, US SLUS_006.50:0x8003e05c, US SLUS_006.50:0x80043d58, US SLUS_006.50:0x8001d3e8, US SLUS_006.50:0x8001ce48, US /DATA/DB01/LEVEL01.COL, US /DATA/DB01/DB01LP.BIN
worklog:
---

# Gameplay loop

How a level runs: from the front end's choice, through the level's setup and
its per-frame loop (player, camera, objects, drawing, HUD, pause), to the
exit code that picks the next level. Read from the pseudo-code and the
disassembly only; nothing here has been checked frame by frame against the
oracle yet.

Conventions: angles are 4096 to a turn. Y points down (a bigger y is lower).
`dt` below is the frame delta in 1/300 s units (0x800c3f1c, see Frame
pacing).

## Main and the level number

`main` (0x800185f8) keeps the next mode in 0x800c3e90 (gp+16):

- -1: the chooser 0x80030158 runs the front end and returns its choice.
- signed `< 254`: the level runner 0x80017c0c(mode) runs a level and returns
  the next mode. A negative mode runs level `-mode` (0x80017c6c).
- otherwise: 0x8007475c(mode >> 8) plays a cinematic on its own.

Main copies the returned mode to the game state's current level
(0x800c5678) before the next round.

The game state block is 328 bytes at 0x800c5678, initialised from
0x800b27d8 (main). Fields used here:

| offset | type | meaning |
|---|---|---|
| +0x00 | s32 | current level (initially 1) |
| +0x04 | s32 | previous level (set at 0x80018368) |
| +0x08 | s32 | playable character for this level, from 0x80018598 |
| +0x21 | u8 | 0 normal play, 1 activity play (main's printf) |
| +0x23 | u8 | per-level byte from 0x800b2920 (meaning unknown) |
| +0x24..+0x2c | s32[3] | respawn position (set by cinematic 5's handler) |
| +0x30 | u32 | flags: 1/2 use the respawn, 2 seen cine 1, 4 seen cine 7, 16 doors unlocked (guess), 32 scripted player, 64, 1024 |
| +0x38 | u32 | bit per level: completed (0x80018354) |

0x80018598(level) gives the character: 6 for levels 4 and 11, 3 for 5, 9
for 7, 2 for 6 and 10, 5 for 17, 1 otherwise.

### Level numbers

The level number is the database number (DBnn). Names from 0x8003b168
(name index 0x800b7d60[level] into the string table 0x800bb4e8):

| lvl | name | lvl | name | lvl | name |
|---|---|---|---|---|---|
| 0 | Menus | 11 | Circus Angelicus | 22 | Multiplayer Golf |
| 1 | Pickles' Home (the hub) | 12 | Let There Be Light | 23-31 | Hole One .. Hole Nine |
| 2 | Grandpa's Teeth | 13 | Cookie Race | | |
| 3 | Incident on Aisle 7 | 14 | The Return of Mr. Friend | | |
| 4 | 7 Voyages of Cynthia | 15 | Mirrorland | | |
| 5 | Visitors from Outer Space | 16 | Training Level | | |
| 6 | Toy Palace | 17 | Gold Rush | | |
| 7 | Reptar 2010 | 18 | Egg Hunt | | |
| 8 | Ice Cream Mountain | 19 | Be Bad | | |
| 9 | The Mysterious Mr. Friend | 20 | No More Cookies | | |
| 10 | Chuckie's Glasses | 21 | Touchdown Tommy | | |

## From the front end to a level

The Start Game submenu 0x8003cf80 has three items (index in s1):

- 0 Main Game: returns 1, or the result of 0x8003bea0 (a debug menu, "Debug
  Menu") when 0x800c4088 is set. Sets state +0x21 = 0.
- 1 Activities: 0x8003b7bc (-2 goes to 0x80096134); sets +0x21 = 1.
- 2 Training: returns 16 (Training Level), +0x21 = 0.

The front end and the chooser pass that value up. Main, on 1 with
0x800e3ca0 clear, starts a new game: 0x80094cfc, state +0x34, +0x38, +0x22
cleared, +0x04 = -1, +0x30 bit 16 cleared (0x80018754). Then
0x80017c0c(1): Main Game starts in level 1, Pickles' Home, the hub.

### The level-start cinematic (0x80074958)

The setup calls 0x80074958(level, player) (0x800180a4). By level:

| lvl | cine | name (cine table +0x00) |
|---|---|---|
| -1 | 0 | Opening Cine (no player) |
| 1 | 7 | Reptar Puzzle; only while state +0x30 bit 4 is clear, then sets it and 0x80094bfc(1) |
| 2 | 1 | Episode 1 Intro; while +0x30 bit 2 is clear |
| 3 | 2 | Episode 2 Intro |
| 4 | 28 | Episode3 Intro |
| 5 | 37 | Space Case Intro |
| 6 | 45 | Toy Palace Intro |
| 7 | 79 | Reptar Intro |
| 9 | 17 | Mr. Friend Intro |
| 10 | 44 | Chuckies Glasses Intro |
| 11 | 52 | Circus Angelicus R1 Intro |
| 12 | 66 | Let There Be Light Intro |
| 13 | 6 | Race Angelica Intro |
| 15 | 73 | MirrorLand Intro |
| 17 | 12 | Gold Rush Intro |
| 18 | 25 | Bonus 6 Intro |
| 20 | 63 | No More Cookies Start |
| 21 | 49 | Touchdown Tommy Start |
| 23-31 | 29, 30, 31 or 34, 32, 33, 81-84 | Golf Cam n / Golf Intro |
| 0, 8, 14, 16, 19, 22 | none | |

With the "seen" flag set, levels 1 and 2 call 0x800746bc(cine) instead of
playing it (it sets 0x800c4c38 and runs 0x80072ef0 and 0x800732d4; guess:
it poses the scene as the cinematic leaves it). After a played cinematic it
posts CineFinish (0x800783d8) with the player.

The cine names come from the first word of each 16-byte entry in the cine
table at 0x800b8bd0 (see [cinematics](cinematics.md)).

## The level runner (0x80017c0c)

Setup, in order (addresses of the calls):

1. A 0x1014-byte block (0x800c3e84), 0x8008fc0c, 0x80078dcc.
2. The sound-effect set 0x800b82e8[level] to 0x800c4914.
3. 0x800c3fb0 = 1 for levels 8, 19 and 22 (the camera does not follow; see
   Camera), else 0.
4. The start position 0x800b2940 + 16 * level (s32 x, y, z) to
   0x800de600..608, and three u16 angles 0x800b2b40 + 8 * level to
   0x800c4ae8..aec. Level 1: (4913, -1290, 20178), yaw 2320.
5. 0x80030430 (empty), 0x8001d218, 0x80046340(level), 0x8008a3d8(32),
   0x8008a328(level + 1), 0x80094d1c.
6. `load_rooms` 0x8002a208 (LEVEL, A.RAW, COL; [level](../formats/level.md)),
   0x8006e96c(sfx set), 0x80078a50, 0x80072d38(1) (the trigger pass, see
   Triggers), 0x80072c9c, 0x80062ca8, the collision header 0x8002a65c,
   models 0x80044d00, animations 0x80054fd0, hierarchies 0x80044e7c,
   0x80088c44, 0x8008f898(8), 0x8008fef8(10).
7. Outside level 1: the HUD pools 0x8008a714, 0x8008c584(3),
   0x8008ce78(1), 0x8008d348(1), 0x8008e824(128).
8. 0x8004aae8, 0x80091538, 0x8008d8e8, 0x80088d90, 0x80088f04, 0x80046414
   (allocates the fixed-camera table 0x800c4b9c), 0x80046460.
9. The placements 0x800454dc(level, 0) ([lp](../formats/lp.md)), the face
   mask 0x800b91a0[level], the face banks 0x80074b9c, 0x8008b088(64),
   0x8006e720(32).
10. For each player (count 0x800c4294, 1): 0x800596e8 (player init),
    0x8008c094 for some levels, the character id at player +0x38 (1492,
    1776, 600, 14, 608, 609, 508 by state +0x08), and the card or respawn
    placement (0x80070cc8 / 0x80070448, 0x800825b0).
11. 0x8001d218, 0x8008fbdc, the memory budget 0x80019204, music
    0x8006fea8, the level-start cinematic 0x80074958.
12. Each player's camera reset 0x800290a4(0x800de460 + 172 * i, player).
13. 0x800453ec ("Before ResetHierarchy"), 0x800819a4.
14. The loop: 0x80018c38. Its result is the exit code.

Teardown (0x80018230 on): it records the previous level (+0x04), applies
the result (`>= 0`: +0x00 = result, then the loading screen 0x8003b168 /
0x8003eefc), resets the players (0x80059454), frees everything, and returns
the exit code. A -1 exit clears the progress fields. When gp+0 (0x800c3e80)
is set (cine 85, Reptar Dance) it runs 0x8008b1e4 (guess: the credits) and
sets state +0x30 bit 4.

## The frame (0x80018c38)

Before the first frame: 0x800193a8, the pads (0x8001acc0) three times,
EpisodeStart (0x8007871c(player, level)), 0x8009fd78, the pending events
and cinematic (0x80018ad0), the camera's field of view (0x8001dfb8).

Each frame, until the exit code 0x800c4918 is non-zero:

1. Profiling: 0x800c3ebc, root counter 1 (0x8009abc4), 0x800193a8;
   0x800c3e98 += dt.
2. Freeze: 0x800c3eb9 = 0x80097584(dt). Non-zero while a training message
   waits for Cross (0x800c473c, 0x800c4738); otherwise it counts down 36
   timers at 0x800e7e90.
3. Player update 0x8005a068(0x800e2b48), unless frozen (level 1 always
   updates). It returns -1 always (s7 is never changed), so the
   0x800585b0 call after it never runs.
4. Camera: player +0x224 = 0x800de460 and 0x8002953c(0x800de460, player);
   for 0x800c3fb0 levels +0x224 = 0x800de550 and 0x80030438 (empty).
5. World update, unless frozen: the scheduler 0x80072bf0(0x800c3f20), NPC
   lists 0x8008851c(dt) and 0x8008a358(dt), 0x8008c0d4(player), ten
   96-byte records at 0x800e2788 (0x8004aba4: floor query, 300-tick
   lifetime; guess: shadows or dropped things), eight 92-byte effect slots at
   0x800e6ef0 (0x8009173c, handlers 0x800baedc).
6. 0x8001d440(-1): wait for the last flip, start the frame.
7. With the player's room flag 16 (room +0x0c): 0x80088c80 and 0x80088e0c
   (camera, light 0x800c4a00); the cinematic player calls them too.
8. Layer 0 (0x8001d2b0(0), 0x8001d34c(0)); DrawSync first when the last
   frame ran long (0x800c3f40 / 2 < 0x800c3ec4).
9. The world: `render` 0x80030b80(camera), then 0x8006f88c(sfx set),
   0x8006f170, 0x800884ac, 0x8008e7ac, 0x80090174, 0x8006d6e8,
   0x8006d7bc, 0x8008af20, 0x8008af80, the trigger pass 0x80072d38(0),
   0x80091590, 0x80034a98(camera).
10. Soft reset: 0x800c3efc set (Select and Start held, see Input) makes the
    exit code -1.
11. Layer 1: the HUD pools 0x8008c0fc, 0x8008cad0, 0x8008cfec, 0x8008d4b4,
    0x8008af20 / 0x8008af80 again, text 0x8002ecd0(1); layer 2; the
    training message 0x80097564 when frozen.
12. Flip 0x8001d3e8(1) (it samples the pads after the flip), then the
    speech stream 0x8008d654.
13. Pause: Start pressed (`*0x800c49ac & 3 == 1`) or no pad
    (0x8001b6b4(0) < 0): 0x8008dbec, 0x8006ec40, then the pause menu
    0x8003e05c; its result is the exit code; 0 resumes (0x8006ed30,
    0x8008dcb0).
14. Restart: Select pressed (`*0x800c49d8 & 3 == 1`) outside activity play:
    0x80043d58 ("Restart Level?" Yes / No); its result is the exit code.
15. 0x800818e0 (activity play: per-activity values), then 0x80018ad0: the
    event queue (0x80077070) and any cinematic requested in 0x800c3e9c.
16. Frame time stats (0x800c3ec0, 0x800c3ec4, max 0x800c3ec8).

After the loop: each player's camera reset (0x80029078) and field of view
612, 0x800c56a8 bit 64 cleared, return 0x800c4918.

### Frame pacing (0x8001d3e8, 0x8001ce48)

The flip waits until at least 2 vertical blanks have passed since the last
one (0x8001ceb4(last, 2, 8): n clamped to 8), so 30 frames a second at
best. Then:

- 0x800c3f1c = 5 * n (dt, 1/300 s units: 10 at 30 fps),
- 0x800c3f20 = 5 * n * 4096 / 300 (seconds in 4.12: 136 at 30 fps),
- a 32-entry history at 0x800ce0b8.

0x8001d00c ends the drawing and queues the swap for the VSync callback
([boot](boot.md)).

### Mid-level cinematics (0x80018ad0)

0x80077070 drains the event queue. If it left a cinematic number in
0x800c3e9c: unless its flag (0x80074b80) has bit 16 it fades out over 127
dt units by rendering frames with 0x8001886c (the same draw, no player or
world update) while 0x800c3f2c rises; then play_cine 0x8007424c(n) and
CineFinish (0x800783d8(player, n)), repeating while events queue more.

## Input mapping (0x8001b028)

0x8001b028(port, map) points each action at a word of the port's input
state (0x800cda18, 21 words a port: [input](input.md)). The maps are at
0x800b44ac (six: 0-2 digital, type 0x41; 3-5 analog, 0x53). Map 0, the
default, after its three header bytes:

| slot | button | pointer | action |
|---|---|---|---|
| 0 | Start | 0x800c49ac | pause |
| 1 | Select | 0x800c49d8 | restart menu |
| 2 | Cross | 0x800c4994 | jump (JUMP) |
| 3 | Square | 0x800c4984 | walk while held (WALK) |
| 4 | Circle | 0x800c4988 | pick up / put down (GRAB) |
| 5 | Triangle | 0x800c49d4 | use / throw (ACTION) |
| 6 | L1 | 0x800c49d0 | camera 0x8002729c(cam, player, 0, 341) |
| 7 | R1 | 0x800c49b0 | camera 0x80026ddc(cam, player, 0) |
| 8 | L2 | 0x800c49c8 | camera 0x8002729c(cam, player, -341, 0) |
| 9 | R2 | 0x800c49cc | camera 0x8002729c(cam, player, 341, 0) |
| 10, 11 | Up, Down | 0x800c49e4, 0x800c498c | axis 0x800c49b4 |
| 12, 13 | Left, Right | 0x800c497c, 0x800c4998 | axis 0x800c499c |
| 14, 15 | left stick Y, L3 | 0x800c49b8, 0x800c49a4 | axis 0x800c49c4 |
| 16, 17 | left stick X, L3 | 0x800c49a0, 0x800c4990 | axis 0x800c49a8 |
| 18, 19 | right stick Y, R3 | 0x800c4974, 0x800c496c | axis 0x800c4980 |
| 20, 21 | right stick X, R3 | 0x800c49e8, 0x800c49e0 | axis 0x800c4970 |

The JUMP / ACTION / WALK / GRAB names are the controls screen's
(0x800405e4); the actions are from the code below. Maps 1 and 2 swap
Cross, Square and Circle around. The tables are indexed by port * 4, and
neighbouring actions are 4 bytes apart, so only port 0 is consistent
(player +0x1a7 is 0).

A button action is pressed when `*ptr & 3 == 1`, held when `*ptr & 1`.

### The look buttons (0x8002729c)

L1, L2 and R2 look aside; they do not turn the camera. 0x8002729c(cam,
player, yaw, pitch) starts a look when the camera is the follow camera
(cam +0x4a = 1), none is under way (0x800c4a3c bit 16) and the player is
not floating (state 21). It sets bit 16, the offsets 0x800de430 (yaw) and
0x800de434 (pitch) to 0, and the targets 0x800de438 = yaw, 0x800de43c =
pitch: L1 (0, 341), L2 (-341, 0), R2 (341, 0).

Each camera frame (0x8002953c, with bit 16) 0x800247fc steps the yaw
offset toward its target, or the pitch offset when the yaw target is 0:

- progress = offset * 2048 / target; s = rsin(progress), rounded toward
  0 when divided by 16;
- step = (s / 16 + 128) * 0x800c3f20 / 4096 (136: 4 to 12 a frame);
- out (bit 32 clear): offset += step toward the target; reaching it
  clamps and sets bit 32;
- back (bit 32 set): offset -= step toward 0; reaching 0 clamps and
  clears bits 16 and 32.

0x8001e838(cam, 0, yaw, pitch) then turns the camera matrix about the
eye: rotY(yaw) (0x80020c30) times the matrix, or the matrix times
rotX(pitch) (0x80020ca0); MulMatrix0 is 0x8009f488. In the oracle, R2
looks right, L2 left, L1 up, and a look is out and back in about 100
game frames. R1 (0x80026ddc) zeroes the four words and keeps only bits
0x4200 of 0x800c4a3c before setting 0x9c0, which ends a look.

### Axes (0x8001ab10, 0x8001a888)

Twelve 20-byte axes at 0x800cdac0: `s32 value, s32 type, ptr a, ptr b,
s32 last`. Updated after each pad sample (0x8001a888):

- type 4 (two buttons): -127 with a held alone, +128 with b alone, else 0;
- type 2 (stick byte): byte - 127 with a dead zone of 24, scaled to 127;

then value = clamp(v * 4096 / 127, -4096, 4096) * 8, so -32768..32768.
A digital Up gives axis 0x800c49b4 = -32768. An analog pad also sets the
direction buttons' bits when a left-stick byte is below 52 or 203 and up
(0x8001acc0).

Holding Select and Start together for 0x7aeb root-counter ticks (either
port) sets 0x800c3efc: a soft reset (0x8001a9dc).

## The player object

The player is a 1644-byte (0x66c) object at 0x800e2b48 (players are 1644
apart; 0x800c4294 = 1). Its camera is 172 bytes at 0x800de460. Fields,
from the code that uses them:

| offset | type | meaning |
|---|---|---|
| +0x00 | u32 | flags: 0x80, 0x4000 AI-driven (no pad), 0x100000 on a slippery surface (guess), 0x1000000 |
| +0x04 | s16[9] | rotation matrix, from +0x24 (0x80020d10) |
| +0x18 | s32[3] | position x, y, z |
| +0x24 | s16[3] | angles; +0x24 is the heading |
| +0x2c | s16 | target heading |
| +0x38 | u16 | registry id (1492 Tommy, ...) |
| +0x3a | u8 | brightness, eased toward the floor's (+0x8c +5) |
| +0x3b | s8 | room |
| +0x8c | ptr | the collision record under it (COL, below) |
| +0x96 | u16 | template |
| +0xd8 | s16[3] | this frame's displacement, added to the position |
| +0xe0 | u32 | state-local flags, cleared on a state change |
| +0xf4 | ptr | object touched (set by the collision code) |
| +0xf8 | ptr | object stood on (its motion carries the player, 0x800623a0) |
| +0x130 | s32 | unknown (a ledge or step height; tested against 384 and 1024) |
| +0x13c | s32 | floor y under it |
| +0x140 | s32 | ceiling y over it |
| +0x154 | ptr | hierarchy (+0x4c: feet to origin) |
| +0x15c | s16 | character (state +0x08) |
| +0x160 | s32[3] | position at the start of the frame |
| +0x170 | s16[3] | air velocity |
| +0x178 | s32[3] | facing unit vector (4096) for the ground speed |
| +0x184 | s32 | ground speed |
| +0x188 | u32 | flags: 1 frozen, 4 throw requested, 8 airborne, 32 moved, 64 scripted, 2048 alternate camera |
| +0x18c | u8 | state; +0x18d the state last entered |
| +0x1a7 | u8 | port |
| +0x1b0 | u16 | stick magnitude |
| +0x1e8 | ptr | object in focus (0x8006af4c) |
| +0x1ec | ptr | object being picked up |
| +0x1f0 | ptr | object carried |
| +0x224 | ptr | camera |
| +0x658 | fn | state handler |
| +0x664 | s32 | carried-over speed on slippery floors |

### Player init (0x800596e8)

Puts the player at the level's start (0x800de600, angles 0x800c4ae8),
finds its room (0x8002b660), builds its hierarchy from the character
(0x800450ac), clears the state (0, handler 0x8005d11c), and snaps it to the
floor: y = floor (+0x13c) + hierarchy +0x4c. The placements loader then
moves it onto the LP record whose template's shape is the character (or to
the respawn point with state +0x30 bits 1-2) and posts RoomChange (255)
(0x800454dc, 0x800457f4).

### The player update (0x8005a068)

1. 0x800c4290 (a post-throw jump lock) -= dt.
2. With state +0x30 bit 32 and +0x188 bit 64: a scripted pass (animation,
   handler, physics, clamps) and return.
3. Keep the position (+0x160). 0x80075698 (id, position), 0x80062ed4.
4. Pad-driven (+0x00 bit 0x4000 clear): the stick (0x8005be24), then the
   camera buttons R1, L1, L2, R2. AI-driven: 0x80082264.
5. Animation 0x80056834; 0x80067028 outside states 1-2.
6. Level code: golf (23-31), Circus Angelicus (11), Reptar 2010 (7),
   Toy Palace (6), 7 Voyages (4).
7. The throw release 0x80060b44, the state change 0x8005b2d8, the handler
   (+0x658), the state change again.
8. Unless frozen (+0x188 bit 1): 0x80067084 (with +0x00 bit 0x80), then
   0x8005cb48: position += +0xd8.
9. Turn 0x8005bc80, floor test 0x800627a0, ride 0x800623a0, 0x80083634,
   focus 0x8006af4c, 0x800631a4, room 0x8004a318, brightness, and a clamp
   of the position to the room's bounds (room +0x10 .. +0x28).

### The stick (0x8005be24)

- stick = (Left/Right axis 0x800c499c >> 3, 0, -(Up/Down axis 0x800c49b4)
  >> 3); each component replaced by the analog one when larger. Full is
  4096.
- magnitude = |stick| (0x80076784), at most 4096, kept at +0x1b0. Below 71
  nothing moves.
- The direction is turned by the camera's heading (atan2 of camera matrix
  -m[0][2], m[2][2]: 0x800a0328) into the wanted heading.
- Speed factor: 17 (0x800c4200), or 8 (0x800c4204) with WALK held; level 6
  uses 20 / 10 (0x800c4210/14) and level 9 10 / 10 (0x800c421c/20).
- If the turn is at most 1024 (0x800c42cc), or the player is already moving
  at 100 or more (0x800c42c8): heading and target heading are set at once
  and the speed is magnitude * factor >> 12 (17 at full). Else it turns in
  place by magnitude * dt >> 6 toward it, speed 0.
- The speed lands in +0x184, the facing vector in +0x178/+0x180.

The ground displacement is speed * dt along the facing (0x8005caac: 170
units a frame at 30 fps; 0x8005e134 scales by 400/4096 more). 0x8005bc80
turns +0x24 toward +0x2c by 0x8005bbf4: 1025 (0x800c41fc) * dt >> 6, or
298 (0x800c420c) for characters 6 and 34.

### States (0x8005b2d8)

Entering a state sets the handler and the animation (0x800572a8(player,
anim, blend)). Names are guesses from the code that sets them. The ground
step (0x80064a34: the floor followed, a drop past the limit to state 4) is
run by the stand (kind 1), the walk and run (0x8005e1f4) and the slides
(0x8005dedc, kind 2) only.

| state | handler | anim | guess |
|---|---|---|---|
| 0 | 0x8005d11c | 1 (0x80057274) | stand |
| 1 | 0x8005e1f4 | 1 | walk |
| 2 | 0x8005ec7c | 2 | run |
| 3 | 0x8005dedc | 10, sound 190 | |
| 4 | 0x8005f1b4 | 15 | fall |
| 5 | 0x8005ec9c | 28 (moving) or 3 | jump; on a floor flag 4: state 17; before +0xe0 bit 31 (the crouch) no ground step, the height kept |
| 6 | 0x8005f310 | 5 | climb onto the touched object (+0xf8 = +0xf4) |
| 7 | 0x800601e4 | 13 | pick up |
| 8 | 0x80060910 | 14 | put down |
| 9 | 0x8005d030 | 6 | hurt / cry (sounds) |
| 10 | 0x8005cba8 | 4 | land; falls of 193+ post Damage |
| 11 | 0x8005f368 | 24 | |
| 12 | 0x8005f584 | 0x800628a4() | use an object |
| 13-15 | 0x8005faf4, 0x8005fb38, 0x8005fcb8 | 29, 30, 31 | golf aim, swing |
| 16 | 0x8005ec9c | 28 | |
| 17, 18 | 0x8005dedc | 33 | |
| 19, 25 | 0x8005f370 | -, 12 | |
| 20 | 0x8005cc24 | | |
| 21 | 0x8005db28 | 1 | (camera uses +0x65c) |
| 22 | 0x8005f1b4 | 15 | fall |
| 23 | 0x8005f584 | 37 | |
| 24 | 0x8005debc | 8 | |
| 26 | 0x8005ec9c | 28 | |
| 27 | 0x8005f4cc | 39 | |
| 28 | 0x8005f3a4 | 43 | |
| 29, 30 | 0x8005cc2c, 0x8005ce74 | 49, 51 | Reptar 2010 (Triangle, Circle) |

### Animations (0x800572a8, 0x80056fe4, 0x80056834)

The anim numbers above, and every 0x800572a8 argument, are logical. The
first thing 0x800572a8 does is 0x80056fe4, which maps a logical number to
a state of the character's table (object +0xec, from 0x8008a42c). Only
those states index the table.

| logical | state | logical | state | logical | state |
|---|---|---|---|---|---|
| 0, 32, 38, 44, 54+ | 0 | 1 | 1 | 2 | 3 |
| 3 | 5 | 4 | 8 | 7 | 27 |
| 8 | 30 | 9 | 31 | 10 | 33 |
| 11 | 20 | 12 | 22 | 13 | 15 |
| 14 | 16 | 15 | 7 | 16 | 60 |
| 18 | 26 | 19 | 25 | 20 | 19 |
| 21 | 17 | 22, 23 .. 31 | 34, 36 .. 44 | 33 .. 37 | 45 .. 49 |
| 39 .. 43 | 50 .. 54 | 45 .. 49 | 55 .. 59 | 50, 51, 52, 53 | 61, 62, 64, 63 |

Three cases are not a table lookup:

- 5 (climb) picks by the rise at +0x144 (0x80056e44): under 394 is 9,
  then one more per 128, to 14 under 1034; higher is -1.
- 6 is 28 while 0x800590d4 says running, else 32.
- 17 keeps the state playing. +0xe7, the stand's animation, is 17 from
  init.

Seen in the oracle on level 3, the player's track state (+0xa8):

- stopped: 0;
- running: 3 and 4 in turn;
- walking: 1 and 2;
- a jump: 5, then 6, then 8 on landing.

At a state's last frame, 0x8005562c reads its word at 0x800b9c30:

- 4: back the other way (ping-pong, +0xb);
- 2: loop;
- 1: complete. 0x80056484 then picks the next state:
  - 1 and 2 swap, 3 and 4 swap, 17 and 18 swap, 34 and 35 swap;
  - 5 goes to 6, 39 and 58 to 40, 45 to 46;
  - anything else goes to logical +0xe6 (0: standing).
- otherwise (8 or nothing): hold the last frame.

Bit 32 adds the root's motion into +0xd8..+0xdc. At loop ends,
0x800562bc plays the footsteps of states 1 and 2 (sound 164-166 by
character, 132 for Spike in levels 9 and 14), sound 6 for states 6 and
41, and sound 11 for state 8.

### The root's height (0x80055170, 0x80055904)

A frame's lift key moves the body up and down (+0x1c):

- **A set up** (0x80055f7c, each column by 0x80055ed8) puts each track at
  its state's second frame. 0x80055170 then applies the first frame's
  keys: the rotations into the parts, and each lift key's dy, as it is
  (not scaled), added to the height. Every character is set up so at
  load, on the idle (0x80081bac's mode init, 0x80082534 into 0x80082478).
  A settled one is then put back on its floor (0x800452f8). An unsettled
  one (template flag 0x10000000) keeps it: level 5's robots, four columns
  of frame 1305's -311, from the LP's -7824 to -9068.
- **Each step** (0x80056834, 0x80055904 for each frame the time crosses)
  adds the frame's lift key times the change in its time gone. A body
  other than the player's scales the key by its feet (hierarchy +0x4c)
  << 12 over 0x800c41dc's -561.
- **A state's first frame** moves the body by the gap (+0xde), never by
  its key, whatever the time. The gap is set when the frame begins with
  its whole time (0x80055560): the feet, the floor (+0x148) and the key,
  less the height. Across that frame the body comes to its floor, its feet
  and the key, settled or not.
  - An attached body (+0x00 bit 0x80000, set by 0x8008357c: level 2's
    Spike on the player, Chuckie on the goose) takes the key alone as its
    gap.
  - The settling (0x800452f8) clears the gap, so a body set up at load
    moves by nothing across its first frame.
  - An unsettled body's floor is the one under its place (0x8002b240 at
    init, +0x13c).
- **Applying it.** The step's sum (+0xda) goes into the height at the
  body's end (0x8005cb48) unless +0x188 bit 1. Some state handlers take
  it themselves and clear it (0x8005d978).
- **Put back.** After it, the floor query (0x800627a0) asks 0x8002b240 at
  the new place (the origin, +0x1c with the lift in it). With no tile
  there, or the origin below the floor (+0x13c) or above the ceiling
  (+0x140), the whole place goes back to where the frame began (+0x160),
  this step's lift with it. The keys go on regardless, so the lift stays
  where it was put back. Level 13's Angelica, held against the raised
  block at the start line (a tile floored at -5120), runs in place at
  the same height every frame.

A body set up and not yet begun anew goes on by its keys from where it
is. Level 5's robot wakes at -9068 and drifts with its idle's keys. Its
turn in place (state 63) then begins, and the gap of 675 brings it to the
floor's -5120, its feet's -2083 and the key's -1154 (-8357) over the
frame.

The stand handler (0x8005d11c), in order:

- ACTION pressed: carrying, request a throw (+0x188 bit 4, anim 18; the
  release is 0x80060b44); else find the touched object (0x800631d8) and,
  if it has a use callback (+0x54), state 12.
- GRAB pressed: carrying, put it down (0x80063888, state 8); else the
  touched object: a pickup (+0x00 bit 0x10000) whose +0x54 returns -1 or
  is null: state 7; else call its +0x54.
- JUMP pressed, not in golf and 0x800c4290 = 0: state 5.
- Else 0x800590d4 (run) / 0x80059008 (walk) with the move test 0x80064a34:
  state 2 / 1; 0x80058da4 and 0x80062cf4: state 6.

Held by the probes in the walk and run (0x8005e1f4, 0x80064a34 answering
1), a thing met at a slant under 384 (Spike 220):

- A ledge in the body's reach (0x80058da4) and 0x80062cf4 allowing it:
  the climb (state 6). 0x80062cf4 first puts the body at the record's
  point (+0x104) out along its normal (+0x114) by the hierarchy box's
  half depth (not a carried thing's), and the wanted heading (+0x2c)
  square to the face. For a height-field wall the point is the COL
  segment's (0x80064178 -> 0x800671f8 -> 0x8002cd7c).
- A ledge not climbed: running (0x800590d4: the stick held and the
  speed over the run's factor times 3933 >> 12) bumps (state 9), else
  stands (state 0).
- No ledge: running, it bumps when the thing met (+0xf4) is none (a
  wall) or cannot be picked up; one that can (+0x00 bit 0x10000) stands
  him (state 0), as does level 7; a running thing met with +0x6d posts
  that damage. Not running: state 0.

Seen in level 6 as Chuckie, a box in hand run at a box on the floor:
the bump, then (the stick at a slant, speed 16) the stand.

### In the air (0x80061330, 0x8005f1b4)

0x80061330(obj, dt):

- Holding JUMP while 0x800c4288 lasts (+0x172 < 0) adds a lift of -5.
- Air control: the stick (0x800c499c / 0x800c49b4 or the analog axes) >> 7,
  turned by the camera heading, halved unless 0x800c4bc4 bit 1.
- Drag: -|v| * 32 / 4096 along v (by 8 in 0x800c4bd0 state 7).
- Gravity: vy += (14 + lift) * dt; displacement = (v * dt + a * dt^2 / 2)
  >> 6 into +0xd8; v x and z clamped to +-44739, vy to 4096.

The fall handler (0x8005f1b4) lands when y + vy reaches the floor
(+0x13c): vy = floor - y and state 10. It moves across (0x8005ee78) then
down.

### Floor and walls

- 0x800627a0 asks the floor and ceiling at the position (0x8002b240 into
  +0x13c, +0x140, +0x8c). No record, or y outside [ceiling, floor]: back to
  the frame's start position and ask again.
- 0x8005ee78 moves with 0x80066014, which returns 2 on the floor (y >=
  floor: normal (0, -4096, 0) at +0x134), 1 against a wall (the floor
  ahead is above the feet: normal = the reversed move), 3 landing on an
  object; it then tests the bounding shape (0x80067ba4: the body box,
  with a carried object's boxes taken in, see objectives) against objects
  (0x80065c5c, 0x800658cc, 0x800657d0; "quickRoomListCylinderScan"
  0x8006add8). A wall stops the move (0x80061a64 / 0x80061b28 slide);
  a ceiling hit (vy < 0) 0x80061be8.

## Collision: LEVELnn.COL

Loaded with the LEVEL file (0x8002a208, file 466 + db) to 0x800c4a88;
0x8002a65c sets the pointers. Little-endian.

```
+0x00  s32 min[3], pad         0x800c4a78: the grid's bounds
+0x10  s32 max[3], pad         heights count down from max y (+0x14)
+0x20  u32 floor_planes        0x800c4a70 points here
+0x24  u32 unknown_24          0x800c4a84 points here
+0x28  u32 planes_at           from the file's start: 0x800c4a7c
+0x2c  column[512]             0x800c4a94; 8 bytes each
         u32 data             filled at load: file + 0x102c + the sum before
         s16 keys             count of 2-byte keys
         s16 records          count of 8-byte records
+0x102c column data: u8 key[keys][2], then record[records]
at planes_at: plane[floor_planes] then the ceiling planes (0x800c4a90),
         6 bytes each: s16 nx, ny, nz
```

A column is 1024 units of x: (x - min x) >> 10. Its keys are sorted
(z cell, first record): (z - min z) >> 10 is binary-searched (0x8002aec4).
The records from there are a stack, lowest first:

```
+0  u8  floor        floor y = max y - floor * 128
+1  u8  floor_plane  1-based; 0 flat
+2  u8  ceiling      ceiling y = max y - ceiling * 128
+3  u8  ceiling_plane
+4  u8  flags        1, 2 (slippery, guess), 4 (tested by jump and land)
+5  u8  brightness   for objects standing on it
+6  u8  bit 0 last in the stack; bits 5-6 the triangle; bit 7 the cell is split
+7  u8  unknown_7
```

0x8002b240(pos, &ceiling, &floor, &record) walks the stack to the record
whose ceiling is not above the position. A split cell holds two triangles
(diagonal by bit 5: x + z or x against z, within the cell); only records of
the position's triangle count. A sloped floor or ceiling is the plane
through the cell's corner (x & ~1023, height, z & ~1023) with the
record's normal, met by a vertical segment (0x8002af20, 0x80068510).
Outside the bounds it answers floor = ceiling = y and returns 2.

Rooms (LEVEL records) are separate: 0x8002b660(pos, hint) returns the room
whose box holds the point, trying the hint, then the rooms its portals lead
to (portal +0x0e; portals with +0x06 bit 2 skipped), then all; -1 for none.
Characters move between room lists +0x4c and objects between +0x48 when
their room changes (0x8004a218, 0x8004a34c); the player's change posts
RoomChange (0x8004a318).

## Camera (0x8002953c)

0x8002953c(camera, player) runs each frame. The camera is an object (172
bytes): position +0x14..+0x1c, matrix +0x00, mode +0x4a, room +0x80,
offsets +0x74..+0x8c, its target +0x4c (0x800de440, a 36-byte matrix and
position), flags in 0x800c4a3c.

- Mode 3, follow: 0x80027304(camera, player, other). The eye looks at the
  player's position less 768 in y (0x800c3f58; 400 for character 6) from
  4096 behind (0x800c4a4c; 6400 in level 7; 0x800c4a68 4800 or 7200),
  pulled in front of walls by a segment test (0x8002cd7c, "vectorwalk").
  It eases toward the goal (by 1/8 or 1/16 a frame, 0x80029c78).
- Mode 1: snap to the target. Mode 4: no motion. Mode 12: golf.
- 0x800291f8(camera, player, kind, arg) switches: kind 1 or 2 follow
  (0x80025e90 / the room's fixed camera); the room's camera byte (room
  +0x5c, from LP kind 6) picks a fixed camera (LP kind 2, 28 bytes each at
  0x800c4b9c) through 0x800259a0.
- The camera's room is found from its position with the old room as hint
  (0x8002b660); outside every room it is clamped into the player's room.
- 0x800290a4 resets it: room = the player's, field of view 612, 0x800c3f5c
  = 1024 (512 in levels 4 and 15, -1000 in 7).

`render` (0x80030b80) draws from the camera's room through its portals
(0x80034058); a camera outside every room draws them all ("Render --
Camera outside room"). Per room (0x80033818): its 24-byte boxes when room
flag 1 (0x80030ce0), its geometry blobs (0x80010e14), and the draw
callback (+0x40) of every object in lists +0x48 and +0x4c.

## Objects

Objects are 168 bytes (0x800464d8(168)); the generic init 0x8004691c fills
them from the LP record and the template (56 bytes at 0x800c4300 +
56 * template):

| offset | meaning |
|---|---|
| +0x00 | flags: 1 player in range, 0x2000, 0x10000 can be picked up (template +0x28), 0x40000, 0x800000 |
| +0x04 | matrix; +0x18 position; +0x24 angles; +0x3b room |
| +0x40 | draw: 0x800b7d2c[template +0x08] |
| +0x44 | think: run by the scheduler |
| +0x48 | enter (trigger) / hit |
| +0x4c | leave |
| +0x50 | 0x800b7eb0[template +0x09]; called on a thrown hit |
| +0x54 | use: 0x800b7e90[template +0x07], or the door / cine callbacks |
| +0x58 | 0x800b7e50[template +0x0a]; push / step (guess) |
| +0x5c | 0x800b7ea4[template +0x0b] |
| +0x6a | cinematic to play on use (LP +0x44) |
| +0x6c | level to go to on use (LP +0x42) |
| +0x78..+0x84 | s16 bounds; +0x86 radius, +0x7e horizontal radius (0x80046570) |
| +0x88 | model |
| +0x94 | speech line (LP +0x4c) |
| +0x96 | template |
| +0xa0 | trigger radius (template +0x34) |

The class table 0x800b7de8 (16 constructors, 0x800494d8 ..
0x80049d54) is picked by template byte +0x04.

The scheduler (0x80072bf0(0x800c3f20)) walks a list at 0x800c43b8 of
timers (+0x08 period, +0x0c countdown, +0x10 object): when one runs out it
calls the object's +0x44; a negative result removes it.

### Triggers (0x80072d38)

Objects with an LP trigger are kept in the list 0x800c4c24. The full pass
(argument 1) walks it from the front and calls +0x48 on every one whose
squared distance, its three 32-bit products summed and compared signed
with +0xa0, is under it. It runs only before a loader fills the list: in
the level runner before 0x800454dc (0x80017d98), in main's own
cinematics (0x80074858) and in the activity reset (0x80096b54). So it
walks an empty list and leaves the cursor null, and each level's first
five-a-frame pass starts at the list's head. Each
frame (argument 0) five of them, round robin, are tested by 2D distance
(0x8001f57c): coming within +0xa0 calls +0x48 (sets +0x00 bit 1), going
out calls +0x4c. A placement with a speech line (LP +0x4c) gets
0x8004d0f0 / 0x8004d160: in the player's room it plays line +0x94 through
0x8008da4c (XA speech).

The list is built by the loader (0x800454dc): each placed object, in LP
order, whose init gave it an enter callback (+0x48) is pushed on the
front (0x8001a1c0), so the list runs from the last placed to the first;
later additions (EpisodeStart's, 0x80049af4's copies, 0x8007914c) go in
front of those. The cursor (0x800c4c20) goes on from where it stopped,
back to the front at the end. Seen in level 5 (51 entries: sound objects
1139, movers 1156, the sliding doors 843, the robots and others; the
cursor at the 25th): Angelica put beside robot 13002, the 43rd, it is
scheduled on the fourth frame, five a frame from the cursor, and holds
still until then. The port's pass (`Game::trigger_pass`) runs the same
list, and so starts each level at the head.

What the enters and leaves do, and whether the object is in after:
- a character (0x80049e18 / 0x80049e5c): scheduled, unscheduled;
- a speech line (0x8004d0f0): in only with the player in its room, and
  then it speaks; 0x8004d160 lets it out;
- level 5's sliding door (0x8004d078 / 0x8004d0b0): opened (in, also when
  a beam opens it), shut;
- level 12's battery (0x8004cf58): taken and off the list;
- the hub's signs and level 16's sparklers: scheduled, their think
  (0x80049eec) showing them in the player's room;
- level 4's Cynthia (0x8004d138): used (0x8004f78c), the win.

The original's pass runs after the oracle's frame boundary (0x80072bf0,
then the boundary, then 0x80072d38), so what it does in a frame shows in
the oracle a frame later (`level_5_door_opens_as_the_original`).

### Doors (0x8004f78c)

A placement with LP +0x42 (a level) gets use = 0x8004f78c: unless
progress allows (state +0x30 bit 16, or the target is not 3-6) it shows
message 712 (0x80044434); else 0x8006ee14, PostCine(+0x6a) and
PostSwitchEpisode(+0x6c). LP +0x44 alone gives 0x8004f83c: just the
cinematic. DB01 (measured, 324 records): templates 111, 572 and 1225 go
to level 2 with cine 22 (Home to Episode 1), 441 to 3 with 11, 105 to 4
with 10, 828 to 5 with 70, 473 to 6 with 80, 498 to 10 with 71, 500 to 11
with 91, 1130 and 944 to 12 with 69, 399 to 25 with 36; 497 to 18, 432,
433 and 440 to 9, 442 to 13, 881 to 15, 1149 to 12 with no cinematic.

### Pickups and carrying

GRAB on an object with +0x00 bit 0x10000 starts state 7 (0x800601e4,
carried in +0x1f0). ACTION throws it (0x80060b44): its room is taken from
its position, a trajectory computed (0x80061f88, 0x8008864c into +0x90),
think = 0x8004df94, then its +0x48 and +0x50. Level code collects specific
templates on touch (+0xf4): in level 7 templates 401 and 795, in level 6
any template with flag 1024 (sound 173, Damage event); a counter
0x800c4bd6 drives the HUD counter 0x800c4298 (0x8008ca80).

## Events (0x80077070)

Posted to a queue (0x800c4408) and drained each frame by 0x80018ad0. A
20-byte event: +0x00 type, +0x04 object, +0x08 delay in frames, +0x0c
callback (called after), +0x10 u8 argument.

| type | posted by | handled |
|---|---|---|
| 1 | HeartBeat 0x80078054 (each drain) | 0x8007733c; ends the drain |
| 2 | EpisodeStart 0x8007871c | 0x800c4410 = 1, state +0x30 bit 8 off; per-level callback |
| 3 | SwitchEpisode 0x800781f8 | exit code = argument (the level) |
| 4 | Cine 0x80078350 | 0x8007737c(obj, cine, 1): queue the cinematic |
| 5 | CineFinish 0x800783d8 | 0x8007737c(obj, cine, 0) |
| 6 | CameraSwitch 0x80078264 | 0x80077c88 |
| 7 | RoomChange 0x80078174 | 0x80077a54 |
| 8 | Damage 0x800780ac | 0x80077864 |
| 9 | TimerExpired 0x8007897c | callback only |
| 10 | CheckPoint 0x80077f98 | 0x800772e0 |
| 11 | GoalPost 0x80077ff0 | per level (3, 13, 10) |
| 12 | Gameplay 0x800789e4 | 0x80077ee4 |
| 13 | ForceCry 0x800782d4 | 0x8007fd0c |

0x8007737c(obj, cine, 0), after a cinematic, ends levels: for the ending
cinematics it sets the exit code 1 (back to Pickles' Home) and records the
level with 0x80076d6c(level, won). Leaving, with the level: cine 5
(Episode 1 Endiing) 2 (and the respawn point (0x901c, 1001, 0x2839)), 14
3, 35 8, 19 9, 40 5, 47 6, 48 4, 62 11, 68 12, 72 15, 78 10, 20 and 21 13,
24 18, 26 17, 64 20, 50 21. Recording a loss without leaving: 16 (2), 23
(18), 27 (17), 65 (20), 51 (21). Cine 85 (Reptar Dance) sets gp+0 and the exit code
-1: the end. (The switch is on cine - 1, table 0x8001532c.)

## Exit codes

| code | from | next |
|---|---|---|
| 0 | | keep playing |
| n > 0 | SwitchEpisode, ending cinematics, Restart | level n |
| -1 | pause "quit" (0x8003e05c), soft reset, cine 85 | the front end |
| -2 | pause, "Returning to Pickles' House" | run as level 2 by main (see Unknown) |

## Pause (0x8003e05c)

Saves the GTE state (0x8009fd08), shows the memory-card menu
0x800398bc(0, 0, 1, 1) with "Game Paused", "Continue?", "Save Game?" and
two counters (0x80094b8c, 0x80094a0c; models 292 and 291 through
0x8008f8f4). Its result is returned: 0 continue; -1 shows "Returning to the
Activities Menu" (activity play) or "Returning to Main Menus" over the
title backdrop and sign; -2 shows "Returning to Pickles' House". The
in-game "PAUSE GAME" string belongs to the controls screen (0x800405e4,
called from 0x800407d0).

0x80043d58 (Select) asks "Restart Level?" with Yes? / No? on the menu
blocks; 0x8004418c shows "No Controller" / "Unsupported Controller".

## HUD

Drawn in layer 1 after the world (step 11): three pools of linked
elements, created outside the hub: 0x800c4d48 (0x8008c584(3); drawn by
0x8008c538 / 0x8008c394: gauges whose fill is +0x32 / +0x2a, or +0x42 /
+0x3a), 0x800c4d60 (0x8008ce78(1), 0x8008cd00), 0x800c4d78
(0x8008d348(1), 0x8008d278); and 0x8008c0fc (0x8008bdf4 when 0x800c45c0).
The pieces are drawn through 0x8008fa10 / 0x8008fa4c / 0x8008fa9c.

## For the port

A suggested order, each step checkable against the oracle:

1. Main's level dispatch, the runner's setup for level 1 (rooms, COL,
   models, LP, player init, start position) and the frame loop's order
   with dt = 10 at 30 fps.
2. The input map 0 and the axes; Start / Select reading.
3. The COL grid (0x8002b240) and room lookup (0x8002b660), checked by
   snapping the player to the floor at the level 1 start.
4. The stick and the stand / walk / run states (0x8005be24, 0x8005b2d8,
   0x8005d11c, 0x8005e1f4), turning, ground move and the floor revert.
5. The follow camera (0x80027304) with the room clamp; the fixed cameras
   (LP kinds 2 and 6) after.
6. Jump, fall and land (0x80061330, 0x8005f1b4, 0x8005cba8) and the wall
   test (0x80066014).
7. The scheduler, triggers and doors (LP +0x42 / +0x44), the event queue
   (SwitchEpisode, Cine, CineFinish) and the level-start cinematics.
8. Pickups and carrying, the HUD, pause and restart.

## Unknown

- What 0x8007737c's other cinematic cases do (0x80079c10, 0x8007a314, ...),
  and the per-level EpisodeStart callbacks.
- Why the pause's -2 ("Returning to Pickles' House") reaches main as level
  2: main takes the absolute value (0x80017c6c). Not yet seen in the oracle.
- 0x800746bc: what "seen" levels 1 and 2 do instead of playing the cinematic.
- State names (the table is from animations and sounds only), the handlers
  of states 1, 3, 17-21, and the animation numbers' meaning per character.
- The object collision path in 0x80066014 (0x80067ba4, 0x80065c5c,
  0x800658cc). (+0x130 is the slant between the step and the face that
  held it: under 384 head-on (220 for Spike) the stand handler tries the
  climb (0x80062cf4), up to 1024 the slide (0x80065120), which an
  object of flag 0x4000 never takes.)
- COL +0x24, record bytes +4 and +7, the plane equation's exact form
  (0x80068510).
- The follow camera's details (0x80027304 is 1885 instructions): its flag
  bits 0x800c4a3c, easing, what R1 (0x80026ddc) does exactly, camera
  modes 2, 5 and 12.
- The world-update callees 0x8008851c, 0x8008a358, 0x8008c0d4, 0x8009173c
  and the draws 0x8006f88c .. 0x80034a98 (NPCs, effects, shadows: guesses).
- The HUD elements: which pool shows what.
- LP kinds 3 (0x8006fe04, sound?) and 7, record +0x46..+0x4a (template 835
  chains), and template bytes +0x07..+0x0b's tables.
- Level-specific code in the player update (levels 4, 6, 7, 11, 20-31).
- State +0x23 (0x800b2920) and the flags in state +0x30 / 0x800c56a8.
