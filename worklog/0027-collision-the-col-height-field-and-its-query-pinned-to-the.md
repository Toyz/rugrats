---
number: 27
title: Collision: the COL height field and its query, pinned to the original
date: 2026-10-15
area: engine, ui
files: crates/rugrats-data/src/col.rs, crates/rugrats/src/collide.rs, crates/rugrats/tests/collide.rs, crates/rugrats-data/src/lib.rs, crates/rugrats/src/lib.rs, crates/rugrats/src/front.rs, crates/rugrats/src/main.rs, docs/formats/col.md, docs/README.md
started: 2026-10-15T01:26:50-06:00
took: 7m
---

# 27. Collision: the COL height field and its query, pinned to the original

Work toward a playable port (the user's goal). Gameplay needs the floor
under the player. LEVELnn.COL holds it, and was unread until now.

What I found:

- The COL is a height field on 1024-unit tiles: bounds, two normal
  arrays, 512 x columns, and per column a sorted z-cell list into stacks
  of 8-byte tiles.
- A tile has a floor and a ceiling height (max y - byte * 128), each
  optionally sloped through a plane with one of the normals.
- A tile can be halved along a diagonal.
- 0x8002b240 answers, for a point: outside (2), in the open between a
  tile's ceiling and floor (0), or not (1). It also returns both heights
  and the tile.

The port:

- `rugrats_data::col::Col` reads the file.
- `rugrats::collide::query` mirrors 0x8002b240, 0x8002af20 and 0x80068510
  in integer arithmetic.
- `tests/collide.rs` loads each COL into rrt-emu, runs the original's
  setup (0x8002a65c), and compares answers at 4 million lattice points
  across DB00, DB01, DB02, DB05 and DB16. Every point matches: kind, both
  heights and the tile.

Also in this stretch:

- The title's Start Game menu (0x8003cf80) is ported. The buttons map
  with the pseudo-code's pad index one more than the port's constant.
- Main Game returns game mode 1, Training returns 16, and Start chooses
  Main Game at once.
- Main logs the mode until gameplay exists.

**Still unknown:**

- Tile bytes 4, 5, 7 and header +0x24.
- The wall and object collision functions (0x8002b660 to 0x8002cd7c).
