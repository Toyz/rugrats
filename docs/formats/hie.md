---
title: DBnnHIE.BIN, a database's hierarchies
status: partial
discs: US
covers: US /DATA/DB22/DB22HIE.BIN, US SLUS_006.50:0x80044e7c, US SLUS_006.50:0x8003205c, US SLUS_006.50:0x8004500c, US SLUS_006.50:0x8004a5dc, US SLUS_006.50:0x8009f598
worklog: 12, 14
---

# DBnnHIE.BIN

The skeletons a database's characters and jointed props are built on.
Loaded by `load_hierarchies(db)` (0x80044e7c, file 370 + db); an object
gets a copy (`0x8004500c`, 52 + 40 x parts bytes) at object +0x154.

## Layout

Little-endian.

```
u32 at[256]              offset of hierarchy i, 0 for none
hierarchy
  s16  parts
  u8   [0x32]            byte lists, 0xff ended (part groups)
  part[parts]            40 bytes each, from +0x34
    u16  unknown_00      0 in DB22
    u16  depth           1 for the root; a part's parent is the nearest
                         earlier part one level up (depth first)
    u8   [16]            0 on disc (run-time)
    s32  offset[3]       the joint, from the parent's
    u32  unknown_20      0 in DB22
    u32  model           a model id (0x80044b74); the loader makes it a pointer
```

DB22's hierarchy 1, Chuckie: hips (root), chest, head, three-part arms
from the chest, three-part legs from the hips, 15 parts.

## Drawing

`draw_character` (0x8003205c) computes one 32-byte matrix a part
(`0x8004a5dc`, into object +0x158) and draws part i's model (hierarchy +
0x58 + 40 i) with the hand-written renderer, scaling all parts by object
+0x36 when it is not 4096 (each part's model about its own joint; the
joints do not move).

`UpdatePartRotations` (0x8004a5dc) works on a matrix stack in the
scratchpad, one entry a depth. Entry 0 is the object's rotation (angles
at object +0x24, 0x80020d10) with no translation. Each part's rotation is
its quaternion (part +0x04, four s32, set by animation or the cinematic
stream) through 0x8001fc48, and its entry is CompMatrix (0x8009f598) of
the entry one level up with (rotation, offset). The root's offset is left
out: the root's joint is the object's position (object +0x18). Each result
is stored transposed (0x80010b54) with the object's position added; the
object transform (0x8001eb4c) transposes it back before multiplying by
the view matrix. So a vertex v of part p lands at

```
world = pos + R(root) ... R(p) v + sum of R(ancestors) offset
```

with R the parts' matrices in 0x8001fc48's convention.

## Unknown

- The header's byte lists, `unknown_00`, `unknown_20`.
