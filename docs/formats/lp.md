---
title: DBnnLP.BIN and TEMPLATE.BIN, placements and templates
status: partial
discs: US
covers: US /DATA/DB22/DB22LP.BIN, US /DATA/DB00/TEMPLATE.BIN, US SLUS_006.50:0x800454dc, US SLUS_006.50:0x80044b74, US SLUS_006.50:0x8001f684, US SLUS_006.50:0x800202b8, US SLUS_006.50:0x80020ee0, US SLUS_006.50:0x80020d10, US SLUS_006.50:0x800494d8
worklog: 14, 15
---

# DBnnLP.BIN and TEMPLATE.BIN

A database's placed objects (file 402 + db, loader `0x800454dc`) and the
object templates they name (`/DATA/DB00/TEMPLATE.BIN`, file 239, shared
by every database).

## LP layout

Little-endian.

```
u32 count
record[count], 96 bytes
  +0x00 u32 kind          1 an object; 6 and 99 also seen (not yet read)
  +0x04 u32 template      for kind 1: an index into TEMPLATE.BIN
  +0x08 s32 position[3]
  +0x14 u32 unknown_14
  +0x18 s32 rotation[4]   a quaternion (x, y, z, w), 4096 = 1
  +0x28 u16 unknown_28
  +0x2a u16 id            the object registry's (0x800c413c); 0 for
                          objects nothing names
  +0x2c ...               0 in the DB22 records read so far
```

The cinematic stream names its actors and poses by `id`.

## Loading (0x800454dc)

Kind 1 records are objects, except templates 1242, 1041, 1267, 834, 382
and 625, which the loader passes over (and 391-395 in game mode 16, 403
in mode 10, and template 1211 at two particular positions).

The rotation is inverted first (0x8001f684: (-x, -y, -z, w) over |q|^2)
and kept as angles (0x800202b8, object +0x24). A single-model object's
matrix (object +0x04) is made from them by 0x80020ee0, a jointed object's
base by 0x80020d10 (0x8004a5dc). Measured against the original: for a
turn about one axis, the single-model matrix is quat_matrix(q) (the
parts' 0x8001fc48 convention) and the jointed base its transpose. The
object transform (0x8001eb4c) transposes object +0x04, so both draw as
quat_matrix(q) transposed. For a turn about more than one axis the
angles' round trip does not give back the same matrix (about 5% off on
one test); DB22's LP only turns about one axis.

A template that is not a hierarchy is built by the constructor its byte
+0x04 picks from the table at 0x800b7de8 (16 entries). Class 0
(0x800494d8) clears the object's draw (+0x40) when its model came back as
offset 0, the search's miss.

## TEMPLATE layout

1272 templates, 56 bytes each.

```
+0x00 u32 flags          bit 0: built on a hierarchy
+0x04 u8  class          the constructor (0x800b7de8 table)
+0x10 u16 shape          the hierarchy (flags bit 0) or a model id
+0x34 u16, +0x36 u16     unknown (0x3a98 and 0, 1000, 1500, 3500 seen)
```

A model id goes through `model(id)` (0x80044b74), the MOD file's id
search, not the table position.

In DB22, templates 1, 2, 3, 6, 7, 11, 12 and 13 are characters on
hierarchies 3, 2, 13, 4, 5, 1, 15 and 14.

## Measured

Rendering every kind-1 DB22 placement whose template is a model, at its
LP position and rotation (or where the stream's actor puts it), gives the
opening cinematic's props where the original draws them (the door, the
bottle, the block, Chuckie's car and its handle) at stream frame 199.

## Unknown

- Kinds 6 and 99, `unknown_14`, `unknown_28`, the record's tail.
- The template's other fields.
