---
number: 30
title: Hint cards, carrying and throwing
date: 2026-10-15
area: play, ui
files: crates/rugrats/src/game.rs, crates/rugrats/src/player.rs, crates/rugrats/src/front.rs, crates/rugrats/src/text.rs, crates/rugrats/src/main.rs, crates/rugrats-render/src/mesh.rs, crates/rugrats-render/src/renderer.rs, crates/rugrats-render/src/rooms.rs, crates/rugrats-render/src/models.rs, crates/rugrats-data/src/placement.rs, crates/rugrats/examples/card_shot.rs, crates/rugrats/examples/play_script.rs, docs/engine/objectives.md, docs/README.md
started: 2026-10-15T01:56:35-06:00
took: 12m
---

# 30. Hint cards, carrying and throwing

A second delegated map, docs/engine/objectives.md, covers carrying,
flights, use callbacks, collectibles, progress, level 2's script, hint
cards, the HUD and the pause menu. Two pieces of it are now in the port.

**Hint cards** (0x80044434):

- After a cinematic, CineFinish (0x800783d8) shows a HINT file's card.
  Its switch on cine - 1 maps 31 cines to files 672-713, read straight
  from the cases.
- `front::Card` lays the file's CR LF lines out in font 0, grey, centred
  from y -90 by 18 with shadows, over the front end's backdrop, with
  "PRESS X TO CONTINUE" (string 67) at y 80. Cross closes it.
- The card runs the 320-wide display. The oracle's text spans 1.6 times
  the port's first try, so the card's 2D layers are scaled by 512/320
  about the centre.
- Font 0's "r" sits at u 245 and is 12 wide, past the page's last
  texel. The GPU wraps u, so `sprite_triangles` now splits such a
  sprite at the edge.
- Even then the right edge's u is 256, which overflowed the 8-bit u
  packed in `Vtx::uv` into v. `uv` now holds u and v in 16 bits each
  (`Vtx::uv_of`), so a sprite edge can be 256. Every producer changed,
  and the tests pass.
- Card HINT01 now matches the oracle's after cine 7.

**Carrying** (0x8005d11c, 0x800601e4, 0x80060910, 0x80060b44,
0x80087c5c):

- Placed models whose template has +0x28 set (object flag 0x10000) are
  `game::Thing`s. They rest, are carried, or fly, and are drawn and
  collided each frame. The hub has 26.
- Circle near a resting one starts the pick-up (animation 13). The grip
  is the midpoint of hand slots 12 and 15 less slot 15, in the player's
  frame, pushed forward by half the model's depth. The object then rides
  slot 15 (0x8005ff88), and the idle while carrying is 20.
  `posed_parts` exposes the parts' world transforms for this.
- Circle again puts it down: animation 14, a flight of speed 10 along
  the facing.
- Triangle throws: animation 18, (0, -384, 1024) turned by the heading
  at speed 170, so 1856 a component per 0x8008864c's scale. The jump
  lock 0x800c4290 is set to 300.
- Flights fall at 13 with the `>> 6` integration and stop when the
  model's bottom reaches the COL floor.
- A scripted run in the hub picks up object 8, carries it (it follows
  the hand through the animation), throws it, and it lands 2000 units
  ahead.

**Still unknown:**

- The touched-object test proper (0x800631d8). The port takes the
  nearest resting object within 520.
- The animation events that time the grab and the release (+0xe0 bits
  2048, 16). The port acts at the start of the animation.
- A flight's collisions with walls and objects (0x80087594,
  0x80086ef0), and stacking (0x8006c7a0).
