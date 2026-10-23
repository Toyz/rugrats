---
number: 248
title: The marks' face word, and the flight quads' screen test
date: 2026-10-23
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, docs/engine/gameplay.md, docs/engine/objectives.md
started: 2026-10-23T07:13:58-06:00
took: 1h 1m
---

# 248. The marks' face word, and the flight quads' screen test

Two of the long-open drawing items are decoded. A mark's face word turns out to set its draw depth, and the flight quads have a screen test. The port also now draws a mark grey 128 in a room of flag 8, and culls flight quads as the original's.

**The face word** (0x800243c4 through 0x80023c2c, 0x800233f4, 0x800221cc; gameplay.md, the flights' marks):
- **Top two bits:** the primitive (3 a quad).
- **0x800000:** textured.
- **0x2000000 and 0x1000000:** the POLY's semi and raw bits (code 0x2c for the FT4).
- **0x600000:** the page's blend.
- **0x200:** a screen test (0x800239e8, 0x80023a8c).
- **0x400:** a face with a corner clipped is skipped (vertex +6 & 48).
- **0x800:** a face wholly off one side is skipped.
- **32, 64, 128 and 256:** the flag word, the corners, the texture and the colour are kept for the next face.
- **0x7000:** the ordering table's depth (0x800221cc): 0 the first corner's, 0x1000 the last's, 0x2000 the corners' mean, 0x3000 the nearest, 0x4000 the farthest.
- **0x8000:** that depth 64 nearer.

**The marks** (0x8008652c):
- **Which word.** It is chosen by the normal met, not by the tile's +4 as gameplay.md had it. A wall (the normal's y 0) gets 0xc080be00: in at its nearest corner pulled 64 forward. A floor or ceiling gets 0xc0802e00: in at the mean. +0x89 adds 0x2000000.
- **The draw** (0x8008e508) is one quad, or, with the record's bit 0x8000000, a 3 x 3 grid of four (the word | 416).
- **The colour.** The record's colour, but 128 when the room of the mark (its +0x0a, the flight's launch room +0x38) has flag 8 (room +0x0c).
- **The port** has a depth buffer and already passes a decal's equal depth and biases it forward. It now keeps each flight's launch room (`Flight::room`, `Flown::room`) and lays a mark grey 128 when that room has flag 8.

**The flight quads' screen test** (0x8008a954), worklog 114's and 239's open item:
- It comes after the depth test (0 to 65535 ahead).
- The corners (-w, -h), (w, -h) and (-w, h) are turned by the roll (rcos, -rsin; rsin, rcos), x scaled as the view (0x8001e0f4), and projected (RTPT).
- The quad is drawn only when one of them is on the screen within 167 / 128 of its extents: 0x800c4a00's 512 and 240, so |sx| < 668 and |sy| < 313.
- The port's `Flights::draw` takes that test from the game's render eye.
- The four-quad split (face 0x8000000: the centre and the edges' midpoints by three more RTPTs) only changes how the texture lies; the port draws one quad.

**Checked:** every test file, and the hle suite.

No room of any level has flag 8 once its LP kind 6 records are read (every level opened in the port, room +0x0c), so in the game a mark always takes its own colour.

**Still unknown:**
- The quads' screen test has not been compared on the oracle's picture.
