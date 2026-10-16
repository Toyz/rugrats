---
number: 99
title: The segment test (vectorwalk, 0x8002cd7c), checked against the original
date: 2026-10-15
area: engine
files: crates/rugrats/src/world/collide.rs crates/rugrats/examples/vectorwalk_check.rs docs/formats/col.md
started: 2026-10-15T22:44:14-06:00
took: 1m
---

# 99. The segment test (vectorwalk, 0x8002cd7c), checked against the original

The world segment test 0x8002cd7c ("vectorwalk") is ported as
`world::collide::segment`. Many open items wait on it: the camera's
wall pull, NPC line of sight, the golf roll, the pie aim marker's ray
(0x80090554), and the flights' world collision.

`segment(col, from, to, &mut Segment) -> i32` walks the segment a
1024-unit cell at a time. It returns:

- 0: through. Only the last tile passed is written.
- 1: stopped. It writes the point, the normal (a cell side 0x800b7018,
  a diagonal 0x800b7068, or a floor or ceiling plane), the tile, and a
  depth of 0 for a wall.
- 2: `from` is not in the open. The point is `from`, the depth is
  `from`'s y less the floor, and the normal is from - to normalised
  (0x8009f278).

Outputs it does not write keep the caller's values, as in the original.
The whole call tree came across, from one step (0x8002cb70) down through
the diagonal of a halved tile, the crossing point, the tile stacks, the
floor and ceiling planes, the line/plane hit (0x80068510), the first
tile of a stack and the GTE's VectorNormal. VectorNormal keeps only the
low 16 bits of each component, as the IR registers do; its square-root
table at 0x800bd584 was checked entry by entry. docs/formats/col.md has
the details.

**Checked against the original** with `examples/vectorwalk_check.rs`,
which calls 0x8002cd7c in the oracle (`Hle::call_nested`) and the port
on the same COL:

- 600 segments each around the player in levels 3 and 1, in play (the
  game's gp has to be set around the call: at level 1's frame boundary
  the interrupted code has gp = 0);
- 2000 each on databases 0-7 and 9-18 with a bare R3000 and only the COL
  set up.

37,198 segments in all, 0 differences in the return or any output. They
cover every path: 266 diagonal stops, 2,759 floor or ceiling stops, 361
normals wider than 16 bits. Rerun here: 6,000 more, 0 differences.
(Delegated to a single agent; checked and committed here.)

**Still unknown:**
- two segments make the original trap (VectorNormal's add overflows when two of from - to's components are near +-32768); the port returns a zero normal there
- database 8's COL does not parse ("tiles run past the file")
- the callers (camera, NPC line of sight, golf, marker, flights) do not use it yet
