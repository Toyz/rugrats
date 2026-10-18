---
number: 227
title: The characters cast the original's shadows
date: 2026-10-18
area: render
files: crates/rugrats/src/world/game/shadow.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T13:10:26-06:00
took: 16m
---

# 227. The characters cast the original's shadows

The characters now cast the original's shadows (0x8003414c). The port drew none before. On a run along level 1's hall, a turn and a stop, the quad matches the one the original builds, frame by frame.

**What the original draws.** The character draw (0x8003205c) draws a shadow after the parts, unless the character sinks (worklog 226), rides another (+0x00 0x80000) or level 3's cine 15 runs (+0x30 bit 128).
- **The floor (0x8006c118).** A segment runs from the body's place down to its floor (+0x13c) through its room's two object lists (0x8006a9cc). The nearest top it meets wins, else the floor. A character on the second list (+0x00 0x4000) is passed over.
- **The facing.** It is the first part's matrix row 2 across when |m21| < |m11|, else row 1 negated, made a unit (0x8009f278, `Trig::normal`). R's rows are (z, 0, -x), (0, 4096, 0), (x, 0, z).
- **The size.** The parts at slots 0x10, 0xe, 0xb, 6, 2 and 0x11 (0x80012ff0, read in the oracle) are taken flat from the first part and turned by R (0x80010c2c → 0x8009fc64: mvmva, >> 12). Their box in x and z reaches at least 128 behind and 164 ahead.
- **The quad.** The corners go back through R and sit at the first part's x and z over the floor. It is a POLY_FT4 (code 0x2e) of TEXPOS entry 261 (0x800c4484 + 3132): a 64-texel blob at (128, 128) of page 0x1b, CLUT 0x7dec, grey 128, the page | 64, so subtracted.
- **The switch.** 0x800c3fc4 is off only from the opening cine's init (0x8008f078) until play's (0x8008f308).

**The port.** `world/game/shadow.rs` does the above: `Game::shadow_floor`, `Game::shadow` and the `Quad`. `Game::draw` gives the player and every character in a drawn room a shadow, unless sunk or riding. The renderer's semi-transparent world pass already blends by the page's mode 2 and pulls the quad off the floor.

**Oracle.** `level_1_shadow_as_the_original` reads the original's quad at 0x80034518 (facing, extents, place) for the player's hierarchy over 26 frames of a run, a turning run and a stop. It equals the port's of the frame before: the original draws before it updates.

The level 1 start is not used. Booted without cinematics, the oracle never runs cine 7's end hook (0x800732d4), so Phil and Lil stay in Tommy's way. The port hides them as the game does.

**Still unknown:**
- The in-engine cines' shadows. play_cine draws through the same render (0x80030b80), so its characters have them, but the port's cine player (media/cine.rs) draws none.
- Which objects lie on a room's second list with +0x00 0x4000, against the port's "no character".
- Who reads floor flag 8.
