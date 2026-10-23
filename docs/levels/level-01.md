---
title: Level 1, Pickles' Home
status: partial
discs: US
covers: US SLUS_006.50:0x80078e7c, US SLUS_006.50:0x80078bc4, US SLUS_006.50:0x80078c68, US SLUS_006.50:0x80078b00, US SLUS_006.50:0x80078e3c, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x8004689c, US SLUS_006.50:0x80046880, US SLUS_006.50:0x80049eec, US SLUS_006.50:0x80049e18, US SLUS_006.50:0x80049e5c, US SLUS_006.50:0x80050fd0, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x8004d49c, US SLUS_006.50:0x8004d3f0, US SLUS_006.50:0x80083a94, US SLUS_006.50:0x80083f24, US SLUS_006.50:0x80083f6c, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80060b44, US SLUS_006.50:0x800601e4, US SLUS_006.50:0x80060910, US SLUS_006.50:0x80062ac4, US SLUS_006.50:0x8005b2d8, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x80058da4, US SLUS_006.50:0x800780ac, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x80072ef0, US SLUS_006.50:0x800732d4, US SLUS_006.50:0x800746bc, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x80017c0c, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x80072bf0, US SLUS_006.50:0x80072b48, US SLUS_006.50:0x8004f4a0, US SLUS_006.50:0x8004845c, US SLUS_006.50:0x8004f6d8, US SLUS_006.50:0x80097788, US /DATA/DB01/DB01LP.BIN, US /DATA/DB01/LEVEL01.BIN, US /DATA/DB00/TEMPLATE.BIN, US /DATA/HINTS
worklog:
---

# Level 1, Pickles' Home

Everything the hub does beyond walking around: Tommy's house, three floors
and a back yard, with a door object for each level. Near a door the hub
draws the level's name and a difficulty word over a sparkle. Doors to
levels 3, 4 and 6 and the mirror to Mirrorland stay shut until +0x30 bit
16. Spike sleeps in the yard, wakes when Tommy comes near, follows him
around his corner of the yard and fetches three toys. Nothing in the hub
hurts, nothing is won here but the first puzzle piece (cine 7), and no
training message ever shows. Read from the pseudo-code and the
disassembly; the labels, the locked door, the mirror, a full fetch, the
dog's barks and the hub's gates and hooks were run in the oracle (see
Checked in the oracle). Builds on [gameplay](../engine/gameplay.md),
[objectives](../engine/objectives.md), [screens](../engine/screens.md),
[front end](../engine/front-end.md) and [level 15](level-15.md) (which
already covers +0x30 bit 16).

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z and 1024 faces +x. "id" is the registry id (LP +0x2a).
"char" is the character (+0x15c). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1). The hub has no goal, clock, counter, HUD or
damage. On the first visit cine 7 "Reptar Puzzle" plays and gives the
first piece (0x80074958, +0x30 bit 4); after that the cine's end hook is
run without the cine (0x800746bc), which hides the other babies and puts
Tommy in the playpen. EpisodeStart (0x80078e7c) then moves Tommy to the
door of the level he came from, hides six "locked" signs, swaps one door
model, and sets Spike up in the yard. Each door object (20 in DB01) is a
use callback (0x8004f78c) that plays a cine and switches level; within
2500 of one, in the same room, its sign is drawn every frame (0x80049eec).
Doors to 3, 4 and 6 show card 712 until +0x30 bit 16 (set when Mirrorland
starts); the mirror (836 / 881) is no door at all until then. Bonus levels
20, 15, 21 and 17 have no doors: they start on their own after 2, 3, 5
and 7 main levels (0x80076d6c). The puzzle board (581) opens the board,
or level 7 with +0x30 bit 512. Seven Reptar bars lie at fixed spots.

## How level 1 is reached

- Start Game, Main Game: a new game (main 0x80018754, +0x04 = -1), then
  0x80017c0c(1) ([gameplay](../engine/gameplay.md#from-the-front-end-to-a-level)).
- Load Game: the card menu returns 1 ([front end](../engine/front-end.md)).
- Exit code 1: every ending cine, every door of another level back home,
  a cry with no tries left, the training door (level 16), the credits
  after level 7.
- The loading page: "Search for Reptar / Beginning" while +0x30 bit 4 is
  clear, else "Pickles' Home / Returning to"
  ([screens](../engine/screens.md#the-loading-screen-0x8003b168-0x8003eefc)).
- Leaving: Pause, Quit Level in level 1 gives -1, the title (0x80039cc8).

## State

| address | type | meaning |
|---|---|---|
| progress +0x04 (0x800c567c) | s32 | previous level: picks the door spot (0x800b8718) |
| progress +0x30 (0x800c56a8) | u32 | read: bit 4 cine 7 seen, bit 16 doors 3/4/6 and mirror open, bit 512 board goes to level 7. The hub sets none: its EpisodeStart post skips the \|= 256 (0x8007877c) |
| progress +0x34 (0x800c56ac) | u32 | bit 14 closes a door to level 14 (0x8004689c; no DB01 door has 14) |
| progress +0x38 (0x800c56b0) | u32 | completed levels: a completed door's name is cyan (0x80049eec) |
| 0x800c4ce4 | s32 | Spike's bark countdown (dt), 0 at load |
| 0x800c4ba0 | s32 | 100 at EpisodeStart; the shared sound count (the emitters' bubbles 0x80053344, touch 1, the collectibles' kind 16, the sliding doors' think: gameplay.md, "Emitters and hit callbacks") |
| 0x800c3f20 | s32 | the scheduler's step, 136 (0x88, seen) a frame: a think scheduled with period 136 (0x80049e18) runs every frame |
| 0x800c3f1c | s32 | dt (10 a game frame; seen by the bark countdown) |

Level 1 has no setup or per-frame level callback: 0x800b9598[1] and
0x800b9618[1] are 0 (so no ambience), no TimerExpired, room change,
GoalPost, Gameplay or ForceCry case is reached.

## Data in DB01

LP (DB01LP.BIN): 264 kind 1, 43 kind 6 (rooms), 7 kind 7 (spawn spots),
5 kind 2 (posts), 4 kind 3 (sounds at (36543, 819, 19641), (31290, 819,
13296), (25931, 61, 7135), (585, 819, -5316); not traced), one kind 99.

### Rooms (LEVEL01.BIN)

44 rooms, boxes read from RAM (room +0x10..+0x28: x0, y0, z0, -, x1, y1,
z1). Tommy's origin stands at y 779 on the ground floor, -5365 upstairs,
7502 in the basement, about 655..1005 in the yard.

| floor | rooms | y | what |
|---|---|---|---|
| upstairs | 0-15, 33, 43 | -10240 .. -5120 (15: -11264 .. -6144) | 0-6 the upstairs rooms with the doors to 4, 13, 5, 12, 11, 6 and the mirror; 7, 9 hallways (camera 8); 10-14, 43 small doorway rooms |
| ground | 16, 19-31, 34-36, 42 | -4096 .. 1024 | 23 the living room with the playpen (start), 19 and 21 the kitchen side (doors to 3 and 2), 25 the board, 26 (door to 18), 16 (door to golf) |
| basement | 17, 18, 32 | 1024 .. 8192 | 18 the doors to 9 |
| yard | 24, 37-41 | -11264 .. 1536 | 24 Spike's corner (x 41984..54272), 37-41 west to east |
| back doors | 35, 36 | -3072 .. 1024, z 22528..24576 | the doorways between the ground floor and the yard (35 a sliding door, template 337; 36 two hinged doors, 218 and 1194) |

Room songs (LP kind 6 +0x64, flag 128; 0x80077a54 plays song +0x64 - 1):
song 0 upstairs (0-6, 9), song 2 ground floor (16, 19-21, 23, 25, 26),
song 1 in yard rooms 38-39, song 3 in the basement (18, 32). Rooms 11, 12
and 43 have +0x64 197, 198 and 199: speech said on every entry (5 and up
is a speech line, 0x80077a54; flag 128 is never cleared). Camera kinds
1, 6, 7, 8, 9 on many rooms; room 27 a post camera (+0x5c 3).

### Characters (LP kind 1, class 1)

| LP | template | char | id | LP position | in the hub |
|---|---|---|---|---|---|
| 3 | 11 | 1 | 1492 | (5775, 1001, 19403) | Tommy |
| 140 | 9 | 6 | 14 | (21782, -542, -14483) | Spike, moved to the yard by EpisodeStart |
| 138 | 1 | 3 | 600 | (21733, -706, -13634) | Angelica: no think (+0x44 0, seen) |
| 141, 119, 118 | 5, 10, 3 | 7, 8, 13 | 13, 15, 16 | about (19660..20866, -1490..-1271, -14781..-13700) | grown-ups, no think (guess: Didi, Stu, Grandpa; cine actors) |
| 139, 156, 155 | 2, 7, 6 | | 1776, 608, 609 | Chuckie; Phil and Lil at (5775, 1001, 15307 / 17355) | hidden by cine 7's end hook |

600, 13, 15 and 16 stand outside every room box; the generic init gives
them room 0 (0x80046a88). Guess: never seen in play (a cine stage). There
is no Dil and no Grandpa in play.

### Door objects (use 0x8004f78c)

A placement with LP +0x42 gets the door use when 0x8004689c(level) says so
(0x80046cc4); in level 1 every object with +0x6c then gets trigger 2500,
enter 0x80049e18, leave 0x80049e5c and think 0x80049eec (0x800481c8).
0x8004689c: in activity play (+0x21) always; else only level 14 is gated
(table 0x8001361c), shut once +0x34 bit 14 is set (0x80046880).

| LP | template | id | position | room | level | cine | sign | notes |
|---|---|---|---|---|---|---|---|---|
| 93, 269 | 572, 1225 | 0, 2003 | (36919, 1022, 8502), (36922, 879, 8506) | 21 | 2 | 22 Home to Episode 1 | Medium | two models on one spot (the bathroom, guess); cine 22's start hides 2003 (0x80072ef0) |
| 71 | 111 | 1313 | (-3394, 932, 8498) | none (0) | 2 | 22 | | outside every room: its sign can never show (guess: unused) |
| 74 | 441 | 111 | (22623, 831, 10055) | 19 | 3 | 11 Home to Episode 2 | Hard | locked until bit 16; LP speech 113 is lost (below) |
| 208 | 105 | 52 | (4301, -5210, -8025) | 0 | 4 | 10 Home to Episode 3 | Hard | locked until bit 16; trigger template 1000, but 2500 in the hub |
| 206 | 828 | 115 | (34600, -5255, -6068) | 2 | 5 | 70 Home to Episode 4 | Medium | class 13; +0x6d 120 |
| 207 | 473 | 519 | (1, -5395, 14033) | 5 | 6 | 80 House to ToyPalace | Hard | locked until bit 16; id 519 set by 0x8004790c |
| 84, 85, 86 | 433, 432, 440 | 48, 47, 49 | (23250, 7624, 1957), (22904, 5822, 1724), (20986, 7419, 1985) | 18 | 9 | none | Easy | three objects |
| 82 | 498 | 0 | (9330, -89, 21960) | 23 | 10 | 71 Home to Homelvl 2 | Easy | |
| 205 | 500 | 119 | (15615, -5566, 14132) | 4 | 11 | 91 Home to Circus Angelica | Medium | |
| 177, 265 | 1130, 944 | 517, 0 | (40117, -6508, 17554), (40169, -5675, 17497) | 3 | 12 | 69 Home to Homelvl 4 | Medium | |
| 248 | 1149 | 518 | (-920, 0, 118) | 3 (set) | 12 | none | | EpisodeStart makes it template 1130 with model 1130 and room 3; it stands outside room 3 (guess: never seen) |
| 204 | 442 | 33 | (22190, -5387, -7410) | 1 | 13 | none | Easy | the cookie box |
| 83 | 497 | 0 | (525, 659, -2713) | 26 | 18 | none | Easy | |
| 154 | 399 | 0 | (27939, -568, -9604) | 16 | 25 | 36 House to Golf | Easy | name "Ice Cream Mountain" (0x800bb760) |
| 282, 283 | 836, 881 | 0, 135 | (12054, -6799, 14487) | 5 | 15 | none | Easy | the mirror: a door only with bit 16 (0x800478bc) |

Rooms are from the room boxes (LP position); the RAM agreed for the
registered ones. Objects 2003, 33, 119, 115, 519 and 52 have object bit
0x10000 (pickable, template +0x28; seen in RAM).

No door leads to 7, 8, 14, 16, 17, 20 or 21. Level 8 is played as golf
(25..). 0x8004f83c (a cine-only object, LP +0x44 alone) is not used in
DB01.

### Other objects with behaviour

| LP | template | class | position | what |
|---|---|---|---|---|
| 122 | 581 | 0, use 1 | (3103, 711, 514), room 25 | the puzzle board (0x8004d3f0); LP speech 4 |
| 142 | 431 | 4 | (4843, 459, 18532), room 23, id 453 | the playpen gate: hinged door, Triangle swings it (use 0x8004f4a0, seen at 1978), think 0x8004f868 |
| 50 | 337 | 3 | (7155, -1024, 23483), room 35 | the back door: sliding door (0x8004845c: think 0x8004e458, use 0x8004f6d8; used at 2278) |
| 268, 267 | 218, 1194 | 4 | (15364, -1028, 23509), (19435, -1028, 23648), room 36 | hinged doors to the yard |
| 75, 76 | 226, 227 | 4 | (30720, -611, 14314), (30721, -82, 12293), room 19 | the refrigerator doors (as in [level 12](level-12.md)) |
| 135, 137, 136 | 678, 679, 680 | 0, pickable | (-275, 1271, 31772) room 37, (3417, 1321, 32320) and (4053, 1354, 36462) room 38 | Spike's toys (models 1013-1015) |
| 44, 149, 158 | 94, 708, 52 | 2 | (25674, -3527, 16021) room 19; (15303, 5239, -7222) and (21517, 6841, -14111) room 18 | spinners (0x80048364: think 0x8004e2ac); +0x6d 0 |
| 132, 133 | 614, 616 | 10 | (25986, -1690, 7256) room 19; (24845, -490, 30217) room 40 | emitters (0x80048400: think 0x8004e0f4): spinning models 931 every 10 VSyncs, bubbles every 20 (gameplay.md, "Emitters"); damage is dropped in the hub anyway |
| 1, 81, 172, 174, 181, 197 | 112, 130, 277, 180, 192 | 5 | | +0x00 = 0x400000 (0x80049754; not traced) |
| 134, 169, 280, 281 | 613, 1164, 1032 | 0 | (24845, -46, 30215); (3190, 1132, 40862); (7172, 0, 25776); (17421, 0, 25703) | talkers: speech 194, 91, 195, 236 within the trigger in the player's room (0x8004d0f0) |
| 168, 249-254 | 835 | 12 | spots below | seven Reptar bars, level 1's (+0x48[1]) |
| 235-238, 243, 247 | 117, 715, 720 | 0, use 3 | upstairs at y -7188 (rooms 7, 9, 1) | ids 350-354, 5555: hidden at every start |
| 157, 245, 59-62, 89 | 837, 1137, 391-395 | 0 | | ids 127, 514, 55-59: hidden by cine 7's end hook |

Spawn spots (kind 7, LP 312-318), one per group, threshold 100, one bar
each (seen in RAM): (52653, 965, 24417) room 24 (the yard), (6643, -6755,
3143) room 33, (40574, 602, 2439) ground floor (outside the room boxes),
(29877, 602, 20153) room 19, (16393, -5556, -8720) room 1, (14802, 7738,
-14340) room 18, (25593, -5518, 17398) room 3. Bars are numbered and
recorded under level 1 like any level's (0x80094a58).

## Start

In order (0x80017c0c): no HUD pools in level 1 (0x80017de8 skips
0x8008a714 .. 0x8008e824); the placements; the player on LP 3; no
"already earned" note (0x80018050 skips 0x80094bc0 / 0x80094ad0); the
start cine 0x80074958(1): cine 7 if +0x30 bit 4 is clear (then bit 4 and
0x80094bfc(1)), else 0x800746bc(7), which runs the cine's hooks without
playing it; EpisodeStart 0x80078e7c (installed by 0x8007871c case 1).

Cine 7's end hook (0x800732d4 case 7, also from 0x800746bc):

1. Hide (0x80078bc4) 127, 1776, 608, 609, 514.
2. Object 80 (template 896): y = 746.
3. Hide 56, 57, 58, 59, 55 (templates 391-395: the pieces in the cine,
   guess).
4. Tommy to (5775, 779, 19403) (0x800825b0): in the playpen.

0x80078bc4 ("hidechar") takes the object out of every room's lists
(0x80078b00), calls its leave if it was in range, and sets room -1;
0x80078c68 ("unhidechar") puts it back and calls its enter.

0x80078e7c(player):

1. Spike = object 14, s0 = object 518. If 518 exists: template 1130,
   model of template 1130, room 3.
2. Hide 5555, 350, 351, 352, 353, 354, 355 (the 117 / 715 / 720 objects;
   no 355 in DB01). So their use (0x8004d49c, message 33) can never run.
3. Unless the previous level is -1 or 16: Tommy to 0x800b8718[previous]
   (16 bytes x, y, z), the camera reset (0x80026ddc(camera, player, 0)),
   0x800825b0.
4. Spike: floor +0x148 = 1408, +0x144 = 0, position (47762, 727, 32948)
   (0xba92, 727, 0x80b4), room update (0x8004a318), home (+0x648..) =
   that position; 0x80083a94(Spike, player): mode 2 and back mode 2,
   target the player, think 0x80084084, enter 0x80083f24, leave
   0x80083f6c, speed from the table, onto the trigger list; scheduled
   (0x80049e18); +0x688 |= 2 (leash to home); then his leave (0x80083f6c):
   asleep (below).
5. Checkpoint: Tommy's position and angles, phase 1 (0x80078d60).
6. 0x80078e3c(38, 1), (39, 1), (23, 2), (20, 2): room songs, but only for
   a room whose +0x64 is 0; DB01 already gives all four a song, so these
   change nothing.
7. 0x800c4ba0 = 100.

The door spots (0x800b8718, by previous level): 2 (37002, 779, 9866), 3
(21914, 779, 10665), 4 (4787, -5365, -6977), 5 (34864, -5365, -5324), 6
(1142, -5365, 12967), 7 (5532, 779, -475), 8 and 23-31 (26808, 779,
-8518), 9 (22380, 7947, -2876), 10 (9768, 779, 19329), 11 (16597, -5365,
14246), 12 (38751, -5365, 17399), 13 (22563, -5365, -6729), 15 (9458,
-5365, 13673), 18 (2121, 779, -2837), others (5775, 779, 19403).

Seen (main's level patched, +0x30 = 4): 0x800746bc(7) and its hides at
1819, then 0x80078e7c and its seven hides; Tommy at (5775, 779, 19403)
room 23 in the playpen (screenshot); no card, no speech; objects 1776,
608, 609, 127, 514, 55-59, 350-354 and 5555 in room -1; 600, 13, 15, 16
in room 0 with no think.

## Door signs (think 0x80049eec)

Runs while scheduled: the trigger pass (0x80072d38) calls enter
0x80049e18 when Tommy comes within 2500 (2D) of the object. 0x80049e18
schedules the think with period 136 (0x80072b48), which with the
scheduler's step of 136 (0x800c3f20) is every frame, and sets +0x00 bit
1; leaving (0x80049e5c) unschedules it.

Each run, only when the object's room is the player's (+0x3b against
0x800e2b83) and +0x6c is not 0, in level 1:

1. Name: string 0x800bb4e8[+0x6c + 150] (151 + level - 1: "Grandpa's
   Teeth", "Incident on Aisle 7", ...), or for 25 0x800bb760 "Ice Cream
   Mountain".
2. Difficulty by level (table 0x8001377c, level - 2):
   - Hard, red 0x0000ff: 3, 4, 6 (0x800bb860);
   - Medium, blue 0x00ff4b4b (R 75, G 75, B 255): 2, 5, 11, 12
     (0x800bb858);
   - Easy, green 0x0000ff00: 7-10 and every level above 12 (13, 15, 18,
     25) (0x800bb85c).
3. Templates 836 and 881 without +0x30 bit 16: return, nothing drawn and
   no sparkle.
4. Four text items on the stack, drawn at once (0x8002e67c), font 0,
   spacing 1, centred across (0x8002e144 flag 1), positions from the
   screen's centre: the difficulty word at y 62, the name at y 80, each
   after its shadow (colour 0x010101 at +2, +2). The name is white
   0xffffff, or cyan 0x00ffff00 when +0x38 has the level's bit (for 25,
   bit 8).
5. The sparkle 0x80050fd0 with 0x800c416c cleared (no sound 48): three
   particle flights (0x8008864c, flags 0x30305, 200 x 200 sprite) at the
   object.

In level 16 the same think only sparkles (no +0x6c on 111 / 441 there);
in other levels it returns.

Seen (text hook): "Hard" (-22, 62) red and "Incident on Aisle 7" (-96,
80) white at the spot for 3; "Medium" / "Grandpa's Teeth" twice (572 and
1225) at the spot for 2; "Hard" / "7 Voyages of Cynthia"; "Medium" /
"Visitors from Outer Space"; "Hard" / "Toy Palace"; "Medium" / "Circus
Angelicus"; "Medium" / "Let There Be Light"; "Easy" / "The Cookie Race";
"Easy" / "Egg Hunt"; "Easy" / "Ice Cream Mountain"; "Easy" / "Mirrorland"
twice with bit 16 and nothing without it. Drawn on every game frame (2
VSyncs). With +0x38 = 8 the name for 3 was 0x00ffff00. At the spots for
9, 10 and 15 nothing showed: those doors are more than 2500 away (2D)
from the spot. Screenshot: green "Easy" over white "Ice Cream Mountain"
at the bottom centre.

An object's LP speech line is lost when it becomes a hub door: the
generic init sets enter 0x8004d0f0 for a speech line (0x80046d20) and the
level 1 tail overwrites +0x48 with 0x80049e18 (0x800481c8). DB01 LP 74
(the door to 3) has speech 113; seen: its +0x48 is 0x80049e18.

## Using a door (0x8004f78c)

Unchanged from [gameplay](../engine/gameplay.md#doors-0x8004f78c): with
+0x30 bit 16 clear, a level of 3, 4 or 6 shows card 712 (HINT41) and
nothing else; else music off (0x8006ee14), PostCine(+0x6a) if any,
SwitchEpisode(+0x6c). In level 1 a posted cine is queued even when the
exit code is already set (0x8007737c at 0x800773a0; other levels drop
it).

Seen: bit 16 clear, Triangle at the door to 3: 0x8004f78c and
0x80044434(712) at 1992, no switch. Bit 16 set: 0x80078350(cine 11) and
0x800781f8(player, 3) at 2123.

The hub's own use callbacks never run:

- 0x8004d49c (use 3) on 111 and 441 shows message 34 only in level 16:
  in the hub those templates are doors (their use is 0x8004f78c) and the
  message is not allowed in level 1 (0x80097788).
- On 117, 715 and 720 it shows message 33, but EpisodeStart hides all
  six. Message 35 has no caller. So no training message shows in the hub
  ([level 16](level-16.md#the-calls)).

## How the hub reacts to progress

| what | test | effect |
|---|---|---|
| cine 7 | +0x30 bit 4 (0x80074958) | played once, first piece; later only its end hook |
| doors to 3, 4, 6 | +0x30 bit 16 (0x8004f78c) | card 712 until set |
| mirror 836 / 881 | +0x30 bit 16 (0x800478bc) | use 0 and no sign until set; then a door to 15 with an "Easy / Mirrorland" sign |
| the board | +0x30 bit 512 (0x8004d3f0) | the board, or level 7 |
| sign colour | +0x38 bit level (0x80049eec) | name cyan once completed |
| a door to 14 | +0x34 bit 14 (0x8004689c) | none in DB01 |
| Tommy's spot | +0x04 (0x80078e7c) | beside the last level's door |

Bit 16 is set at the start of cine 73 (Mirrorland's intro, 0x80072ef0
case 73) and by 0x80076d6c(15); cleared only by a new game (0x80018754).
Mirrorland itself starts after the third main level
([objectives](../engine/objectives.md#finishing-a-level-0x80076d6clevel-won)).

Nothing else depends on progress: no object is hidden or shown by
pieces, bars or completed levels, and every door but 3, 4, 6 and the
mirror works from the first visit (2, 5, 9, 10, 11, 12, 13, 18 and golf).
Completed levels stay open (replay). The fixed hides (cine 7's end hook,
EpisodeStart) happen on every visit.

## Spike (object 14, char 6)

Template 9 (shape 6): range +0x68c 6000, reach +0x690 1024, trigger
30000, speed index 1 (row 0 of 0x800b9698: 0, 4, 8, 12, 17, 25), +0x688
= 2 (leash). He is an NPC with think 0x80084084 and +0x00 bit 0x4000.

### Asleep and awake

- Leave (0x80083f6c, also called by EpisodeStart): carrying something, he
  drops it (state 8, mode and back mode 1); else +0x00 bit 1 off,
  animation 26 (blend 4096), mode 1, back mode 1, target the player.
  Mode 1 plays the animation and returns to the back mode, so he lies
  there. Seen: mode 1 / 1, state 19 at (47226, 1396, 32945) room 24.
- Enter (0x80083f24, Tommy within 30000, 2D): +0x00 bit 1, animation 25,
  mode 1, back mode 2: he gets up and then follows. Seen: 0x80083f24 at
  2003 (Tommy teleported to (44000, 700, 32948)), mode 2 at 2030.

### Mode 2: following on a leash (0x800854b4)

1. If he is more than 2 x range (12000) from home and +0x688 bit 2: goal
   = home.
2. If the target (Tommy) is within 12000 of home: goal = Tommy, distance
   to him.
3. Within range / 2 (3000) and heights overlapping (0x80083ff0): stop
   (speed 0); with +0x688 bit 8 attack (never: bit 8 is clear). Else
   speed from the table by +0x66e.
4. Level 1 only, id 14: 0x800c4ce4 -= dt; at 0 or below, when the
   player's room is 24 or 39-41: a bark, speech 0x800b96ec[(rand & 255) %
   11] (1109, 1110, 1112, 1113, 1114, 1116, 1117, 1119, 1120, 1121, 1122),
   then 0x800c4ce4 = (rand & 255) x 8 (0 .. 2040 dt).

So he stays in the east yard: he follows Tommy only while Tommy is within
12000 of (47762, 32948), and walks home when he himself is more than
12000 out. With Tommy farther than that and Spike near home, the goal is
not updated (guess: he walks to where Tommy was last).

Seen: he stopped 2992 and 2119 from Tommy; barks 1112 at 2026, 1110 at
2388, 1112 at 2472, 1114 at 2784, 1113 at 2624, 1109 at 2652, 1122 at
2970; the countdown fell 10 a game frame.

### The fetch game

1. Throw (0x80060b44 at 0x80060bb0, level 1): the thrown object is
   template 678, 679, 680 or 716. Speech 191 or 192 ((rand & 1) + 191),
   always. If Spike is in mode 2 and his goal (+0x67c..+0x684) equals
   Tommy's position exactly (he is following Tommy and his think ran since
   Tommy last moved): mode 12, speed index 4, target = the toy.
2. Mode 12 (0x80084cd0): goal = the toy; speed twice the table beyond 2 x
   range, the table beyond range / 2. Within reach (1024), and the toy is
   not 716 and not in Tommy's hand (0x800e2d38): state 7, +0x1ec = toy.
3. Pick up (0x800601e4 for char 6): the toy leaves its room; part slot
   0x24 gets model 1212 (678), 1214 (679) or 1213 (680) (716: none, a
   printf); speech 193, 159, 234 or 235 (rand & 3); with +0x00 bit 0x4000:
   target = Tommy, mode 13. No pick-up animation for char 6.
4. Mode 13 (as mode 4, 0x80084df0): after Tommy; within range / 2 with
   heights overlapping, or touching: state 8.
5. Put down (0x80060910 -> 0x80062ac4 for char 6, not thrown): slot 0x24
   back to model 79; the toy 1024 ahead of him (0 in mode 14); speech 190;
   a short drop flight. When the animation ends: state 0, speed index 2,
   mode 0 (so back to 2), target Tommy.
6. Giving up, mode 14 (0x80084c8c): in modes 4, 12 or 13 in rooms 35 or
   36 (the back doorways; 0x80084cf4), or on bumping into a wall (state 9
   entry for char 6 in level 1, 0x8005b614): carrying, put down where he
   stands; else speed index 2, mode 0, target Tommy.

A toy that hits him (0x800511a8, char 6): animation 8, mode 1 (a flinch),
then back to following.

Nothing is rewarded: no progress field, counter or sound beyond the
speech changes. Seen: +0x30, +0x34, +0x38, +0x44, +0x46 the same before
and after a fetch.

Seen (toy 679 picked up in the yard, Tommy teleported to (44000, 700,
36000) facing +z, Triangle at 2200): speech 191 at 2228, mode 12 speed 17
toward the toy; state 7 at 2320 (859 from it); speech 159 at 2354, mode
13 carrying 679; state 8 at 2400, 2998 from Tommy; 0x80062ac4 and speech
190 at 2440; mode 0 at 2460, mode 2 with speed index 2 at 2470. A throw
toward the west fence instead: the toy landed on something at y -445,
Spike bumped a wall at 2244 (state 9), mode 14, mode 0, mode 2.

The toys lie in the west yard (rooms 37, 38) and Spike's corner is the
east yard (room 24), so a fetch needs a toy carried across the yard.

## Hub rules

- No damage: Damage events are dropped in level 1 (0x800780ac at
  0x800780d8, also 11, 17, 23-31), so no hazard, burst or object hurts,
  and nothing can make Tommy cry.
- No fall damage and no message 31: the landing after a long fall skips
  the level 16 case and the Damage in level 1 (0x8005b6e0).
- No climbing in room 4 (levels 1 and 12): 0x80058da4 returns 0 for an
  object in room 4 (0x80058e04).
- No HUD (0x80017de8), no freeze (0x80018d70: the player always
  updates), no ambience (0x800b9618[1] = 0).
- EpisodeStart does not set +0x30 bit 256 (0x8007877c).
- Pause, Quit Level gives -1 (the title), as in level 16 (0x80039cc8).

## What does not happen in level 1

- No goal, timer, counter, health meter, cry, ending cine or bar award
  screen. Bars found here count toward pieces like any level's.
- No training message (33 and 35 are allowed in level 1 but never
  reached).
- No hint card at the start; card 712 only from a locked door.
- No door to 7, 8, 14, 16, 17, 20 or 21.

## Hint cards

| card | file | shown | text |
|---|---|---|---|
| HINT41 | 712 | a door to 3, 4 or 6 before +0x30 bit 16 (0x8004f78c) | To go on this adventure you'll need more puzzle pieces. |

## Cines

| cine | name | posted by | start hook (0x80072ef0) | end hook (0x800732d4) |
|---|---|---|---|---|
| 7 | Reptar Puzzle | the start, once | none | the hides above, Tommy to the playpen |
| 22 | Home to Episode 1 | doors to 2 | hide 2003 | none |
| 11 | Home to Episode 2 | door to 3 | none | none |
| 10 | Home to Episode 3 | door to 4 | object 116 (template 30, yard) not drawn | none |
| 70 | Home to Episode 4 | door to 5 | none | none |
| 80 | House to ToyPalace | door to 6 | none | none |
| 71 | Home to Homelvl 2 | door to 10 | none | none |
| 91 | Home to Circus Angelica | door to 11 | none | none |
| 69 | Home to Homelvl 4 | doors to 12 | none | none |
| 36 | House to Golf | door to 25 | none | none |

Cine table flags (0x800b8bd0 + 4): 0x200 for 7, 0x2000200 for the door
cines. Doors to 9, 13, 15 and 18 have no cine.

## Checked in the oracle

The level 16 probe copied to the scratch folder as l01probe, with a text
hook (0x8002e67c: string, font, colour, position; `--texts`) and a Spike
line (object 14: position, room, mode, back mode, state, speed, index,
goal, distance, target and carried templates, home, range, reach, +0x688,
0x800c4ce4). Presses start 300, cross 500, 650, 800, 1650, 1800; main's
level argument patched (0x80018824 = `li a0, 1`, restored at 2400),
0x800c5678 = 1 and +0x30 = 4 (or 0x14) at the runner, cards skipped
(0x8004472c = 0x1000004a). Places by `--tp`, walking by `--goto`.

- Start: 0x800746bc(7) and its hides, 0x80078e7c and its hides, Tommy in
  the playpen, no card; the registry read from RAM (rooms, thinks, uses,
  flags); 44 room boxes and flags.
- Signs: thirteen door spots teleported to; the strings, colours, fonts
  and positions above; the completed colour; the mirror with and without
  bit 16; two signs on one spot.
- Doors: the locked door with card 712; the open door with cine 11 and
  level 3.
- Spike: asleep at the start, woken at 30000, following, stopping at 3000,
  barking in room 24, one full fetch with toy 679 (throw, run, pick up,
  bring back, drop), a fetch given up on a wall bump; no progress change.
- The playpen gate (431) and the back door (337) opened by Triangle.
- Not checked: the barks in rooms 39-41; mode 14 in rooms 35 / 36; a toy
  hitting Spike; toys 678, 680 and 716; Spike walking home past 12000;
  the hub with no damage and no fall damage (from the code; a teleported
  fall kept the floor of the teleport); the room 4 climb rule; speech in
  rooms 11, 12, 43; picking up a door toy; the spinners, bursts and class
  5 objects; the door cines' hooks; the doors to 9, 10, 18 in use.

## For the port

The port has doors (LP +0x42 / +0x44), the 3 / 4 / 6 lock with card 712,
the mirror with bit 16, the board, the door spots, the start cines by
+0x30, the finish screens, carrying and throwing, NPC modes 1, 4, 6, 7, 8,
9, 11, the follow camera, room flags (songs and speech lines), talkers,
spawn groups and collectibles. In the hub it still needs, in order of
how much they matter to playing:

1. The fixed hides at every hub start: cine 7's end hook (1776, 608, 609,
   127, 514, 55-59 hidden; object 80 y 746; Tommy to (5775, 779, 19403)
   before the door spot) whether or not the cine plays, and
   EpisodeStart's 350-355 and 5555. The port has no level 1 script, so
   Phil and Lil (in the living room) and the pieces are likely still
   placed; check.
2. No message 33 in the hub: the port's use check shows it for 117 / 715
   / 720 by position, but those objects are hidden in the original, so
   the hub shows no training message at all.
3. Door signs: trigger 2500 (2D) and the same room; the level's name and
   Easy / Medium / Hard as four font-0 text items centred across at y 62
   and 80 with shadows at +2, +2; colours as above; cyan for a completed
   level; "Ice Cream Mountain" for 25; none for the mirror without bit
   16; drawn every frame with three sparks and no sound.
4. Spike: a `level1` script that moves him to (47762, 727, 32948) with
   that home, asleep until Tommy is within 30000 (animations 26 / 25);
   mode 2 (follow within 12000 of home, stop at 3000, go home past
   12000); barks every (rand & 255) x 8 dt in rooms 24, 39-41; the fetch
   (modes 12, 13, 14) for templates 678-680 thrown while he follows, with
   the mouth models 1212-1214 on slot 0x24 and speech 191/192, 193 / 159 /
   234 / 235, 190; giving up in rooms 35 / 36 or on a wall bump.
5. Hub rules: no damage of any kind in level 1 (the port's long-fall
   Damage 5 and its `hazards` from template +0x32 apply in every level;
   DB01 has +0x32 on 565 (20) and on 828, the door to 5 (120)); no
   climbing in room 4.
6. Hinged and sliding doors (classes 4 and 3: 0x8004f4a0, 0x8004f868,
   0x8004f6d8, 0x8004e458): the playpen gate, the back doors, the
   refrigerator. No port code names them; check how the port treats the
   playpen gate that Tommy starts behind.
7. Small things: the door to 3 says no speech 113 (the port's talkers
   include LP 74); object 518 takes template 1130's model and room 3;
   doors that are pickable toys (2003, 33, 119, 115, 519, 52) and what
   carrying one does to its door; the spinners (class 2).

## Unknown

- Whether carrying a pickable door toy (the door to 5, 6, 11, 13, 4, 2)
  away moves the door with it (its trigger follows the object; not
  tried).
- What the class 5 objects (flags 0x400000) do. (The kind 3 sounds are
  the looping sound sources, 0x8006fe04 and 0x8006f88c: gameplay.md.)
- Who 13, 15 and 16 are and whether 600, 13, 15, 16 can ever be seen.
- Object 518 (template 1149 made 1130, at (-920, 0, 118), room 3) and
  LP 71 (template 111, outside the house): why they exist (guess: left
  over).
- Where Spike goes when Tommy is far but Spike is near home (goal not
  updated).
- Card arg for 712 (from 0x8004f78c: a1 is the player pointer).
