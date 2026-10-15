---
number: 35
title: Mini golf, room cameras, hazards and the level timer
date: 2026-10-15
area: play
files: crates/rugrats/src/golf.rs, crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats/src/main.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/level2.rs, crates/rugrats/examples/play_script.rs, docs/engine/golf.md
started: 2026-10-15T03:46:43-06:00
took: 40m
---

# 35. Mini golf, room cameras, hazards and the level timer

A fifth delegated map, docs/engine/golf.md, traced the nine holes and
checked them in the oracle. The port now plays the course. Around it
went three things every level uses.

**Golf** (`rugrats::golf`):

- The hole's place in the play order gives its level, par and baby
  (0x80057dcc, 0x800b7ffc, 0x800b7ff0). The baby's body is swapped in;
  the other babies placed in the hole are hidden.
- Walk-up within 0xfffff (d^2), the walk-in, then the stance 425 to the
  side (0x8005bac4).
- The meter: 1 idle, 4 on Triangle, 3 a frame to 220 and back. Triangle
  again swings; the hit comes 13 frames into the stroke.
- Aim: Left and Right 25 a frame, 8 with Square, only while the meter is
  idle. No jump.
- The roll: k = (power << 16) / 6000; pos += v dt >> 6. Friction 15 a
  frame, the slope push (0x80086a20). Walls and object boxes reflect it
  at 3000 / 4096. The ball sits at floor - 104.
- The cup: within 384, a speed over 1441 lips out at half speed, else
  it drops. A ball that stops within 384 drops too.
- Holing out: over par shows "You were over par. / Try Again", waits for
  Cross, then a cry and the hole again. At or under par the next hole.
  A hole in one is a bar. The ninth plays cine 35; Game records level 8.
- HUD: "Par %d", "Swing ", the stroke number in yellow at y -100. The
  meter is sprites 293/294; its bar grows from the right end.
- The golf camera: (440, -2368, -4096) in the player's frame looking at
  (440, -768, 0). While the ball rolls it watches the ball.
- Start cines Golf Cam n by level; level 25 the intro from outside the
  course. Activity golf goes hole to hole, then back to the title.

Checked in scripted runs:

- Hole 1 straight at power 181 drops for a hole in one.
- Hole 9 at power 202 runs downhill, off three walls, and stops short.
- Hole 2 shows Chuckie in the stance with the meter.

**Room cameras** (0x80077c88 into 0x800291f8):

- Room flag 4's byte sets the follow height: 6 is 200, 7 and 11 are
  500, 8 is 1600, 9 is 2000. Kind 3 is a fixed camera at the room's post
  (+0x5c).
- Levels 4 and 15 follow at 512, 7 at -1000 and 6400 behind.
- Level 3's boss rooms now look from post 122, the view the oracle shows.

**Hazards** (0x8005e1f4): walking or running into an object that is not
pickable and not a character costs its +0x6d (template +0x32). Level 3's
ice bags hurt for 10. Standing next to one does not; level 2's puck sat
at Tommy's feet.

**The level timer** (0x800c4430, 0x8007f5a8): seconds at the top with
the clock sprite 375; at 0 sound 117 and the level's TimerExpired. Level
2's clock now runs on it. The bar counter shows the bars available with
sprite 291.

**Still unknown:**

- The world segment test (0x8002cd7c) the roll uses: the port tests the
  height field an axis at a time. The exact VectorNormal and the
  descriptor's stack leftovers.
- The golf camera's fixed view near the cup and the Circle height cycle;
  the aim line sprites (364); the score card of activity golf.
- The sizes of sprites 291 and 375; hazards' hurt rate.
