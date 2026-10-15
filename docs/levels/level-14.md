---
title: Level 14, The Return of Mr. Friend
status: partial
discs: US
covers: US SLUS_006.50:0x8007b17c, US SLUS_006.50:0x8007871c, US SLUS_006.50:0x80074958, US SLUS_006.50:0x80083be0, US SLUS_006.50:0x800833a4, US SLUS_006.50:0x800834b0, US SLUS_006.50:0x80084084, US SLUS_006.50:0x80081bac, US SLUS_006.50:0x80083764, US SLUS_006.50:0x800291f8, US SLUS_006.50:0x80078d60, US SLUS_006.50:0x8007fd0c, US SLUS_006.50:0x80076d6c, US SLUS_006.50:0x8007737c, US SLUS_006.50:0x800783d8, US SLUS_006.50:0x8007f5a8, US SLUS_006.50:0x8007f498, US SLUS_006.50:0x8004691c, US SLUS_006.50:0x80061c98, US SLUS_006.50:0x80061f88, US SLUS_006.50:0x800628a4, US SLUS_006.50:0x8005f584, US SLUS_006.50:0x800562bc, US SLUS_006.50:0x8004fe14, US SLUS_006.50:0x8004d49c, US SLUS_006.50:0x80097788, US SLUS_006.50:0x800919b4, US SLUS_006.50:0x80094b10, US SLUS_006.50:0x8004f78c, US SLUS_006.50:0x8003b7bc, US SLUS_006.50:0x8003bea0, US SLUS_006.50:0x8003cf80, US SLUS_006.50:0x800781f8, US SLUS_006.50:0x80018598, US SLUS_006.50:0x80017c0c, US /DATA/DB14/DB14LP.BIN, US /DATA/DB14/LEVEL14.BIN, US /DATA/DB00/TEMPLATE.BIN
worklog:
---

# Level 14, The Return of Mr. Friend

Everything level 14 does, from its start to its only exits: Tommy in his
bedroom, one Mr. Friend chasing him, a can and a toy to throw, the kick.
Read from the pseudo-code and the disassembly; the start, the chase and
the bites, the kick, a thrown can hitting him, his death, what follows it
and the one-try loss were run in the oracle (see Checked in the oracle).
Builds on [level 9](level-09.md) (the same Mr. Friend code),
[gameplay](../engine/gameplay.md), [objectives](../engine/objectives.md) and
[screens](../engine/screens.md).

The short answer to "how does it end": it does not. No cine, no card and
no call of 0x80076d6c(14, ...) exist. Killing Mr. Friend only hides him.
The level is also unreachable in the retail game (below). The win path
the bonus-bar table plans for it (2 bars) is dead code.

Units: dt is 1/300 s (10 a frame at 30 fps), angles 4096 a turn, y down,
heading 0 faces +z. "id" is the registry id (LP +0x2a). "char" is the
character (+0x15c = template +0x10). A VSync is 1/60 s; the oracle's frame
counter counts VSyncs.

## The goal in one paragraph

The player is Tommy (char 1, the default case of 0x80018598). There is no
start cine (0x80074958 case 15, level + 1, is none) and so no hint card.
Play starts at load: Tommy at (9424, -5365, 13784) in room 0, heading 0,
health 255, one try (0x800b2920[14] = 1). Mr. Friend (id 46, char 16, 100
health) chases him and bites for 15. Circle picks up a can (template 716,
+0x6d 30) or a toy (390, 25); Triangle throws it in a plain arc (no aim in
level 14). Triangle with Mr. Friend touched is the kick: 50. At 0 health he
dies and is hidden, and nothing else happens. The first cry ends the level
(SwitchEpisode(1), the hub). Nothing records the level as played or won.

## How level 14 is reached

Nothing in the retail game switches to level 14:

- No door. A placement with LP +0x42 (a level) gets the door use
  0x8004f78c, which posts SwitchEpisode(+0x6c). No kind 1 record in DB01
  has 14 there (all 324 scanned). The only record anywhere with 14 is DB19
  LP 278 (template 0, (33968, 632, 15897)); DB19 has no file-table entry
  ([db19](../cut/db19.md)).
- Not an activity. Activities (0x8003b7bc) return 0x80013030[item] = 13,
  17, -2, 18.
- Not a bonus. 0x80076d6c starts 20, 15, 21 or 17 after 2, 3, 5 or 7 main
  levels; never 14.
- No other SwitchEpisode argument is 14 (callers of 0x800781f8: 7, 1,
  door +0x6c, golf).
- The debug menu (0x8003bea0, "Debug Menu") picks any level 1..31 but 19
  and 22 and writes it to 0x800c5678 (0x8003c5e8). The Start Game menu
  (0x8003cf80) calls it only when 0x800c4088 is not 0. The only writes to
  0x800c4088 store zero (0x8003d28c, 0x8003dc1c); it is 0 in the image.

So level 14 is reachable only through the debug menu, which the retail
game never opens (guess: a cut level). Other tables still have a slot for
it: the loading screen's "The Return of / Mr. Friend" (strings 193, 141;
[screens](../engine/screens.md)), the health meter list (0x8008c094), the
EpisodeStart callback, ForceCry's case, footsteps, the bonus bars. The
hub's return spot 0x800b8718[14] is the default (5775, 779, 19403),
shared with levels 0, 1, 16, 17 and 19-22. DB14's own music and speech
files are not in the file table; its slots point at DB01's
([unused sound](../cut/unused-sound.md)).

## Phases and state

| address | type | meaning |
|---|---|---|
| 0x800c4cd8 | s32 | Mr. Friends left: 1 at EpisodeStart (0x8007b17c), 0 at his death (0x80083be0) |
| 0x800c4cd0 | u8 | phase: 1 from EpisodeStart's 0x80078d60; never changed, never read in level 14 |
| 0x800e5830, 0x800e583c | | respawn set by EpisodeStart to the start spot; ForceCry's level 14 case never uses it |
| progress +0x23 (0x800c569b) | u8 | tries: 1 (0x800b2920[14], set by the runner 0x80017c0c at 0x80018138) |

Level 14 has no level timer (0x8007f5a8 table 0x800158cc index 12 is the
default 0x8007f6c8), no room change case (0x8007f498 table 0x80015894
index 11 is the default 0x8007f598), no per-frame ambience
(0x800b9618[14] = 0), no case in the generic object init (0x8004691c table
0x8001369c index 13 is the default 0x800481c8), no CheckPoint room, no room
cine and no Gameplay script (the two kind 6 records hold only songs).

## Data in DB14

### The characters (LP kind 1)

| LP | template | char | id | position | health | range | speed index | +0x6d | what |
|---|---|---|---|---|---|---|---|---|---|
| 32 | 11 | 1 | 1492 at run time | (9424, -5170, 13784) | | | 4 | 50 | Tommy, the player; seen at (9424, -5365, 13784), room 0, heading 0 |
| 30 | 15 | 16 | 46 | (2435, -5649, 4155) | 100 | 2000 | 1 | 15 | Mr. Friend; seen at (2435, -5620, 4155), room 2 |

NPC init (0x80081bac at 0x80081ff0, char 16, no level test): +0x58 =
0x800834b0 (hit), +0x54 = 0x800833a4 (use), +0x6d = 15, +0x688 |= 0x108.
Seen: +0x688 0x10108 at the start, then 0x108; range 2000, reach 1024.
Speed table 0x800b9698 row 0: index 1 = 4.

### Things to pick up

All pickable (object +0x00 bit 0x10000, template +0x28 = 1), class 0.

| LP | template | position | +0x6d (damage) | +0x24 (sound) | flags | room |
|---|---|---|---|---|---|---|
| 40 | 716 | (11738, -5244, 14597) | 30 | 21 | 0x0a | 0 |
| 39 | 390 | (1670, -5428, 12058) | 25 | 20 | 0x02 | 0 |
| 34 | 391 | (6485, -5394, 9317) | 0 | 0 | 0x42 | 2 |
| 35 | 393 | (5015, -5395, 8602) | 0 | 0 | 0x42 | 2 |
| 36 | 394 | (5730, -5405, 9813) | 0 | 0 | 0x42 | 2 |
| 37 | 404 | (8115, -5324, 10963) | 0 | 0 | 0x02, trigger 4096 | 0 |
| 38 | 403 | (12460, -5441, 16618) | 0 | 0 | 0x02, trigger 4096 | 0 |
| 41 | 840 | (-493, -5902, 10791) | 0 | 0 | 0x02 | 0 |

716 and 390 are level 9's "can" and third toy (same templates and
values). 391, 393 and 394 are also climbable (flag 64).

### Collectibles and spawn groups (LP kind 7)

Four kind 7 records (x +0x04, y +0x08, z +0x0c, group +0x10, threshold
+0x11):

| LP | group | threshold | position |
|---|---|---|---|
| 44 | 0 | 100 | (-1551, -6652, 4092) |
| 45 | 1 | 33 | (6603, -6654, 2740) |
| 46 | 1 | 66 | (-1061, -6487, 13673) |
| 47 | 1 | 100 | (-1559, -5714, 17613) |

A Reptar bar (LP 42, template 835, class 12, kind 3) goes to group 0;
health (LP 43, template 863, kind 4, heal 63, LP +0x46 = 3 copies) to group
1, one per spot. Seen: 0x8004fe14(.., 0, 1) and (.., 1, 3) from 0x80046008
at load; the bar at (-1551, -6652, 4092), the three health at the three
group 1 spots.

### Other objects

| LP | template | class | what |
|---|---|---|---|
| 29 | 117 | 0 | use 3 (0x8004d49c): training message 33 ("You can only open these..."); 0x80097788 shows it only in levels 1 and 16, so nothing happens (guess: the bedroom door) |
| 31 | 277 | 5 | (4613, -6299, 16381), room 0; object +0x00 0x400000 after init (meaning not traced) |
| 7-22 | 168 | 0 | 16 copies at y -5169 and -5761 around x -1527..6662, z 2579..4656 (climbable, 0x42) |
| 1, 3, 4 | 279 | 0 | scenery at y -8305 |
| 2, 25, 26 | 283 | 0 | scenery at y -6271 |
| 5, 6 | 278 | 0 | scenery, flags 0x800002 |
| 27, 28 | 282 | 0 | scenery at y -6747 |
| 33 | 373 | 0 | scenery (climbable) |

The one kind 99 record (words 0, 4) has no loader case.

### Rooms (LEVEL14.BIN, LP kind 6)

Three rooms; boxes read from RAM (room +0x10..+0x28).

| room | x | y | z | kind 6 | role |
|---|---|---|---|---|---|
| 0 | -2048..13312 | -10240..-5120 | 10240..18432 | song 1 | the start; the can, the toy, the door |
| 1 | 9216..11264 | -9216..-5120 | 9216..10240 | none | holds the bar and health before placement |
| 2 | -2048..7168 | -10240..-5120 | 2048..10240 | song 1 | Mr. Friend's start |

Rooms 0 and 2 have +0x0c = 0x80. No camera flag, no posts: the follow
camera all the time (seen camera +0x4a = 1). Seen floor heights: -5365
at the start spot, -5610 at (3500, 6000) and (4880, 12500). Screenshots:
Tommy's bedroom (bed, nightstand, blue dresser, toys).

## Start (EpisodeStart, 0x8007b17c)

Installed for level 14 by 0x8007871c (case 14). No start cine, so the
runner posts EpisodeStart straight after loading (seen at 1818, from
0x800772ac).

0x8007b17c(player):

1. Mr. Friend = 0x800461f4(46).
2. 0x800291f8(camera, player, 1, 1500, 3000, 0, 0, 712): the follow
   camera, field of view 712 (as level 9).
3. 0x80083764(46, player): mode 4 (follow) and back mode 4, target the
   player, speed from the table; scheduled (0x80049e18).
4. Counter 0x800c4cd8 = 1.
5. Respawn = the player's position and angles, phase 1 (0x80078d60).

Not done here, unlike level 9 (0x8007b078): no aim target (player
+0x1f4), no animation overrides, nothing hidden.

## Mr. Friend

### The chase and the bite

The same think as level 9 (0x80084084 mode 4, mode 5): beyond twice the
range (4000) speed 2 x table (8), then the table speed; within range / 2
(1000) with heights overlapping, mode 5: state 19, animation 37,
Damage(player, player, 15) (0x800780ac from 0x80084724), then mode 1. He
bites from up to about 900 away, so he need not touch Tommy.

Seen: from the start he walked toward Tommy at speed 8 from (2435, 4155)
and stopped at (4880, -5620, 8100) at 1938 with speed 0 (guess: blocked
by furniture on the straight line; not traced). With Tommy put at (4880,
12500), straight ahead, he walked on into room 0 and bit. With Tommy at
(3500, 6000) he came to (3851, 6814) and bit every 36 VSyncs: 255 to 0
in 17 bites. Footsteps: sound 132 (0x8006f644 from 0x800563ac) every 40
VSyncs while walking (0x800562bc: char 16 in levels 9 and 14).

### Hit, kick and death: level 14 differences

The callbacks test for level 9; in level 14 they do less:

- Hit, +0x58 = 0x800834b0 (a thrown thing reached him): no speech, and
  +0x54 and +0x58 are **not** cleared (level 9 clears both). Unless in
  mode 1 with animation 29: animation 8 (blend 2048), mode 1. So in level
  14 every hit restarts the stagger, and a kick still works on a
  staggered Mr. Friend after a throw (from the code; not run). The
  Damage of a can or toy does not depend on the callback in either level
  (0x80087594 damages a class 1 object with or without it; see
  [level 9](level-09.md#the-flight-0x80061f88-0x8008864c)).
- Use, +0x54 = 0x800833a4 (the kick): no sound 98, no speech. +0x54 = 0.
  Unless mode 1 with animation 29: health -= the user's +0x6d (Tommy 50),
  not below 0; animation 8, mode 1.
- Mode 1 (0x800841cc): health 0 or less goes to mode 10; else state 19,
  the wait 300 + (rand & 3) x 300 dt on the animation event, then mode 0
  and back to the chase with both callbacks restored (char 16).
- Mode 10 (0x80084314): +0x688 bit 256 is set but the death sound is
  level 9's 100 or level 6's 172 only, so no sound; the death effect
  0x800858d4(npc, 24, 100); then 0x80083be0.
- 0x80083be0 char 16: counter - 1 (1 to 0); the cine posts and 0x8007cc00
  test level 9, so only the hide (0x80078bc4(46): room -1, unscheduled)
  happens.

Seen: Tommy walked into him (Up held) and Triangle at 2214: state 12 at
2220, 0x800833a4 from 0x8005f614 at 2230, health 100 -> 50, mode 1 (state
19), no sound or speech call. In a second run (kick at 2236) he was back
in mode 4 at 2690. With his health
poked to 40, the same kick: mode 10 and 0x800858d4 at 2232, 0x80083be0,
0x80078bc4(46), room -1, counter 0. Then nothing for 2000 VSyncs: no
PostCine, no SwitchEpisode, no 0x80076d6c; Tommy kept playing.

### The kick (state 12)

As level 9: ACTION (Triangle) with the touched object (+0xf4) having a
+0x54 starts state 12 (0x8005d11c); the call comes at the animation event
(0x8005f584 -> 0x8005f614). State 12's animation (0x800628a4) is 50 in
level 14 (37 or 50 at random only in level 9). He must be touched: his
bite range (about 900) is beyond touching, so Tommy walks into him.

## Throwing

0x80061c98 has cases for levels 3, 16 and 9 only, so level 14's throw is
the generic arc of 0x80061f88 along Tommy's facing (no aim; 0x80059b34
still picks a target but nothing reads it here). Flight flags 0x09070700
for the can and the toy (from 0x80061f88; not read in the oracle); 0x40000
is in it, so the hit callback runs without level 9's OR. 0x8008864c sets record flag 0x80000 when the thing's +0x6d is
not 0: the can (30) and the toy (25) do Damage; the other pickups (0) only
call the hit callback (stagger; from the code, not run).

Seen: the can picked up with Circle (state 7, 0x800601e4 at 2058,
carried, +0x6d 30). Thrown at 2228 from (3500, -5610, 6000) toward Mr.
Friend 900 away: it rose to y -6549 and came down about 4000 ahead (y
-5735 at 4100 out, -5331 at 4740), passing over him; no hit. With Mr.
Friend held still at his start (speed index poked to 0) and Tommy about
7800 away on the same x, the can thrown at 2222 landed (sound 21 at 2254)
and bounced into him at 2280: 0x800834b0 from 0x800878b8, sound 21 at the
can (0x800878d4), Damage(player, 46, 30) from 0x8008792c; health 100 ->
70, mode 1 (state 19). Each bounce plays sound 21.

## ForceCry (0x8007fd0c, level 14 case)

Level 14 is case 13 (0x80080128): the cry (0x8007f920) and health 255,
with no respawn pointer (s2 stays 0), so Tommy stays where he cried.
Tries: 1 at the start, so the first cry leaves 0: SwitchEpisode(player,
1) (0x800781f8 called at 0x800802d0, the hub). No 0x80076d6c: nothing is recorded.

Seen: health poked to 10, a bite at 2126: ForceCry from 0x80077a40, the
cry, SwitchEpisode(1) at 2546 (420 VSyncs later), level 1 loaded. Without
the poke: 17 bites, ForceCry at 2600, SwitchEpisode(1) at 3020.

## How the level ends

- Won: never. There is no ending cine (none in the cine table for level
  14; the "Mr. Friend" cines 17-19 are level 9's) and no 0x80076d6c(14,
  ..) call: its callers pass 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 15, 17,
  18, 20 or 21 (0x8007737c, 0x8007fd0c; no pointer to it, rugrats-re
  xrefs). His death only hides him.
- Lost: the first cry, SwitchEpisode(1). Nothing recorded.
- Pause, Quit Level (generic, not run).

What 0x80076d6c(14, 1) would do if something called it (dead code):
state +0x34 bit 14; case 1 of table 0x800152cc (level - 13): 2 bonus bars
(0x80094b10(14, 2): +0x48[14] bits 31 and 30) and "Great Job! You have
earned 2 Reptar bars!" (0x80094ab0); no puzzle piece (0x800919b4(14) = 0,
level 14 owns none); not counted as a main level (0x80076f54: only 2-6
and 8-12). The hub's exit would still need a SwitchEpisode, which 0x80076d6c
does not post for level 14.

## Hint cards and text

None: no start cine, so no CineFinish card (0x800783d8); no level 14
case posts one. The loading screen reads "The Return of" (193) / "Mr.
Friend" (141) / "Loading" (0x8003b168; [screens](../engine/screens.md)). No "already
earned" note (only levels 13 and 18 have one for bars).

## Checked in the oracle

The level 12 probe copied to the scratch folder as l14probe (one-shot
pokes, 16-bit pokes, hooks with arguments, object dumps, room dumps,
screenshots; one addition: dumping objects as 0x8004691c finishes them).
Presses start 300, cross 500, 650, 800, 1650, 1800; main's level argument
patched (0x80018824 = `li a0, 14`), 0x800c5678 = 14 at the level runner,
cards skipped (0x8004472c = 0x1000004a; level 14 has none), 0x80018598
made to return 1. There is no start cine to keep or skip. Tommy was moved
with poked position, room byte and floor heights (+0x13c, +0x148), as in
level 12.

- Start: no cine (0x80074958(14) plays nothing), EpisodeStart 0x8007b17c
  at 1818, 0x800291f8 and 0x80083764(46); tries 1, phase 1, respawn the
  start spot, camera kind 1; Tommy at (9424, -5365, 13784) room 0.
- Mr. Friend: mode 4, walked toward Tommy at speed 8 and stopped at
  (4880, -5620, 8100); followed into room 0 when Tommy stood in line;
  bites of 15 every 36 VSyncs; footsteps 132.
- The kick: state 12, 0x800833a4 from 0x8005f614, 100 -> 50, no sound or
  speech; back to the chase by 2690.
- The death: mode 10, 0x800858d4, 0x80083be0, hidden, counter 0, no
  sound; nothing after it for 2000 VSyncs.
- The can: picked up, thrown in an arc; a miss at 900; a hit from far:
  0x800834b0, sound 21, Damage 30, 100 -> 70, mode 1.
- The loss: ForceCry, tries 1 -> 0, SwitchEpisode(1) 420 VSyncs later, the
  hub loaded; no 0x80076d6c.
- Spawn groups: the bar and three health placed at the kind 7 spots.
- Rooms and objects read from RAM; screenshots of the bedroom.
- Not checked: the toy 390 thrown; a 0-damage pickup hitting him; a hit
  while staggered; why he stops at (4880, 8100); the bar and health taken;
  Pause, Quit Level; the hub placement after a loss (the hub's start cine
  7 was playing in the probe's fresh game).

## For the port

The port has everything level 9 uses (follow and bite, NPC health,
staggers, kicks and deaths in level scripts, thrown things that strike
characters), plus spawn groups, the health meter, tries, ForceCry and the
hub exit. `start_cine` already gives none for 14, the meter list has 14,
and `tries` reads 1 from 0x800b2920. Level 14 needs, in order:

1. A way in. Normal play never reaches level 14 (no door, activity, bonus
   or enabled debug menu). If the port wants it playable, it needs an
   entry the original lacks: a level select or tool switch (port choice).
2. A `level14` script, EpisodeStart 0x8007b17c: 46 to
   `Follow(Target::Player)`, bite 15 and attacks on; counter 1; aim off
   (`g.aim = false`); follow camera with field of view 712; no checkpoint
   is needed (ForceCry never respawns here, and with one try the default
   `cried` goes to the hub). Play starts at load, Tommy at (9424, -5365,
   13784) heading 0, Mr. Friend at (2435, -5620, 4155).
3. Mr. Friend's level 14 callbacks, which differ from `level9.rs`: the kick
   with no sound 98 and no speech, animation 50 only (not 37 or 50); a
   thrown thing hits and restarts the stagger even while he is staggered
   (level 14 never turns the hit callback off; only the kick's use is off
   until mode 1 ends), and a can or toy takes its Damage then too
   (`Level9::hit` ignores hits during `Mode::Hit`, which is wrong here);
   death with no sound 100, the death effect, hidden; counter to 0.
   Sharing the code with level 9 needs those switches.
4. The end. Faithfully: nothing; his death leaves an empty bedroom and the
   level runs until the player cries (1 try: the hub, nothing recorded)
   or quits. If the port wants a win instead (not the original), the dead
   case in 0x80076d6c gives 2 bonus bars (+0x48[14] bits 31 and 30, the
   bars message), no puzzle piece, no main-level count; it must also send
   the player to the hub itself. `ENDINGS` has no entry to add (no cine).
5. Throws without aim: the generic arc; it only meets Mr. Friend's height
   about 3100-4700 ahead, so close throws fly over him (seen). Damage by
   the thing's +0x6d (can 30, toy 25); 0-damage pickups stagger only.
6. Small things: Mr. Friend's footsteps (sound 132, 0x800562bc) if the
   port has no NPC footsteps yet; the can's sound 21 on a hit and on each
   bounce; template 117's use does nothing outside levels 1 and 16
   (0x80097788); songs 1 in rooms 0 and 2.

## Unknown

- Why Mr. Friend stops at (4880, -5620, 8100) on his first walk (an
  obstacle on the straight line, guess) and whether he paths around it.
- What Tommy's floor is at the start (-5365) against -5610 elsewhere in
  the room (a rug or a lower step).
- Whether a 0-damage pickup staggers him in play, and whether a hit while
  staggered restarts the stagger (code says yes; not run).
- What templates 168 (16 copies), 277 (class 5), 278, 279, 282, 283, 373
  and 840 are; why the bar and health start in room 1.
- What the kind 99 record (0, 4) is for.
- Whether level 14 was meant to be a bonus level (the 2-bar case in
  0x80076d6c) reached from the hub (DB19 LP 278 has 14 at +0x42) and how
  it was meant to end.
- Pause, Quit Level from level 14 and where the hub puts Tommy after it
  (0x800b8718[14] = (5775, 779, 19403), the default; not seen).
