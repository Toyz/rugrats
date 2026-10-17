---
number: 178
title: HUD icons at the original's sizes; the overlay drawn in its order
date: 2026-10-17
area: ui
files: crates/rugrats/src/ui/hud.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats-render/src/renderer.rs
started: 2026-10-17T00:01:31-06:00
took: 14m
---

# 178. HUD icons at the original's sizes; the overlay drawn in its order

The HUD's icons were drawn at guessed sizes. They now use the arguments the original passes to its sprite call, 0x8008f8f4 (model, half-size x and y, depth, position, flag):

- the bar icon (291) is 52 by 64, the clock (375) 64 by 64, and the pause's pieces icon 64 by 56, all at depth 1024 (0x8003e05c for the pieces);
- level 18's egg counter is 53 by 64 at (-192, -80), with its number at (-208, -90); gold rush's counter is 64 by 64 at (-192, -80), number at (-205, -93); box toss's is 128 by 128 at (-160, 50); mirrorland's number is at (-205, 58).

The level clock (0x8007f5a8) is drawn semi-transparent: a nonzero fifth argument picks packet flags 0xd2800000 over 0xd0800000 in the sprite renderer (0x8008f5d8), the semi-transparency bit. It sits 20 right of and 13 below the timer text's corner. `Sprites::draw_as` takes the flag.

The overlay's semi-transparent shapes used to be pulled out and drawn after the whole overlay, so the clock covered its own number and dimmed it. The overlay is now drawn in its order as runs of one kind (opaque, or semi-transparent by blend mode), as the ordering table draws it and as the under layer already was. A screen fade is appended last, so it still covers everything. `Stage::overlay_semi` and `Renderer::set_overlay_semi` are gone.

Level 18 at the same frame, original against port: the egg counter, the clock and its number line up. The original's clock read 132 and the port's 169 there: the oracle's boot runs about 31 seconds past the episode's start, so the difference is the boot's, not the clock's.

**Still unknown:**
- The timer text's exact place comes from the font-width loop in 0x8002e144 and the y from D_800c4428 + (D_800c4a06 * 171 >> 11); the port uses (-20, -92), read off a screenshot for a three-digit number, not computed per width.
