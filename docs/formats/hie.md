---
title: DBnnHIE.BIN, a database's hierarchies
status: partial
discs: US
covers: US /DATA/DB22/DB22HIE.BIN, US SLUS_006.50:0x80044e7c, US SLUS_006.50:0x8003205c, US SLUS_006.50:0x8004500c
worklog: 12
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
    u32  model           index into the MOD file; the loader makes it a pointer
```

DB22's hierarchy 1, Chuckie: hips (root), chest, head, three-part arms
from the chest, three-part legs from the hips, 15 parts.

## Drawing

`draw_character` (0x8003205c) computes one 32-byte matrix a part
(`0x8004a5dc`, into object +0x158) and draws part i's model (hierarchy +
0x58 + 40 i) with the hand-written renderer, scaling all parts by object
+0x36 when it is not 4096.

## Unknown

- The header's byte lists, `unknown_00`, `unknown_20`.
