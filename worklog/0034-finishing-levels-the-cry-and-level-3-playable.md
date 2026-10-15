---
number: 34
title: Finishing levels, the cry, and level 3 playable
date: 2026-10-15
area: play
files: crates/rugrats/src/level3.rs, crates/rugrats/src/game.rs, crates/rugrats/src/npc.rs, crates/rugrats/src/player.rs, crates/rugrats/src/progress.rs, crates/rugrats/src/main.rs, crates/rugrats/src/level2.rs, crates/rugrats/src/cine.rs, crates/rugrats-data/src/hierarchy.rs, crates/rugrats/tests/progress.rs, crates/rugrats/examples/play_script.rs, crates/rugrats/examples/anim_states.rs, docs/engine/level-03.md
started: 2026-10-15T03:02:06-06:00
took: 45m
---

# 34. Finishing levels, the cry, and level 3 playable

A fourth delegated map, docs/engine/level-03.md, traced Incident on
Aisle 7 to its end and checked it in the oracle with a scratch probe.
The port now plays it through. Around it, the rules every level shares.

**Finishing a level** (`Progress::finish`, 0x80076d6c):

- Bonus bars from bit 31 down: 13 and 14 give 2, 17, 18, 20 and 21 give 3.
- The level's puzzle piece once (0x80094bfc). Levels 1-6, 8-12 and 15
  own one (0x800919b4); the piece is a random unearned one (0x80091d50).
- 15 unspent bars buy a piece (0x80094924), also after each bar found.
- The 2nd, 3rd, 5th and 7th main level open No More Cookies,
  Mirrorland, Touchdown Tommy and Gold Rush. Mirrorland goes back to the
  hub. All twelve placed lead to Reptar 2010.
- Activity play (+0x21) keeps nothing and returns to the title.
- The puzzle board (0x8009480c) is not ported: earned pieces are placed.
- The ending cinematics (0x8007737c, table 0x8001532c) send the player
  to the hub with the level won or lost; cine 85 ends the game.
- Level 1's Reptar Puzzle plays once (+0x30 bit 4) and gives the first
  piece. Level 2's intro plays until its checkpoint (bit 2).

**The cry** (ForceCry 0x8007fd0c, 0x8007f920):

- Health at 0 plays state 16 for up to 2100 dt while the world waits.
- Then a try goes (0x800b2920[level]): back to the checkpoint, or the
  hub with none left. A level's script can take over.
- The oracle shows Tommy in level 2 standing arms out. DB02's ANM index
  has no entries 138-150 (idle, fall, cry), so the port's pose matches.

**Framework** for the levels to come:

- A `Script` trait (frame, cine_finished, cry, gameplay, draw).
- Hiding objects by id rebuilds the scenery and its boxes (0x80078bc4).
- Objects with only a cinematic play it on use (0x8004f83c). LP +0x44
  16 plays 14; template 567 plays 8 (0x8004691c).
- Room flag 64 runs the level's Gameplay script (0x80077ee4).
- Thrown objects strike characters with their +0x6d (0x80086ef0).
- A fixed camera, and the player thrown back (state 16).
- NPC modes 9 (prowl) and 3 (block and swat), goal posts (flag 32), and
  loop posts (flag 16, LP +0x42 reload).
- A frozen player aims with the stick.

**Level 3** (`rugrats::level3`):

- The start at (6671, 779, -14463); Larry, Steve, 13 and 333 hidden.
- Cine 8 or 9, then the pie toss. Tommy is frozen on the counter with a
  fixed camera. Pies roll at 225 or 450; four hits on the shared 200
  health win, Mellon Rush. Post 39 first loses: cine 15, a try, back to
  the start.
- Phase 2 (0x8007ece4): lobsters prowl and bite for 15; a can kills one.
- Rooms 22 and 36 run script 1: phase 3, the bullies grown. Cine 86.
- The boss blocks and swats; a can stuns it 600 dt.
- The switch plays cine 14: won.

Checked in scripted runs:

- Room 19 asks for cine 9, then the toss; the walkers wait at posts 1
  and 7 as the oracle shows.
- Four aimed pies ask for cine 13, then phase 2.
- Left alone, the goal post at frame about 2300. The oracle's 4684 are
  vsyncs, so the same.
- The switch asks for cine 14.
- The swat lands Tommy about 4500 back.
- Room 22 sets phase 3; a lobster bites every 900 dt.

**Still unknown:**

- How state 16 uses 0x800c426c: half of 2500 as air speed is fitted to
  the oracle's landing.
- The pie and can hit radii; the pie's descriptor fields.
- The aim when the stick is let go while frozen (the port stops it).
- Fixed cameras (room flag 4) outside the pie toss; hazard damage on
  touch (0x8005e1f4); the aim marker; the splats.
- The fourth bonus bar's tests (0x80076ba8, 0x80076bf0, 0x80076c24).
