---
title: DBnnMOD.BIN, a database's models
status: partial
discs: US
covers: US /DATA/DB22/DB22MOD.BIN, US SLUS_006.50:0x80044d00, US SLUS_006.50:0x80044c98, US SLUS_006.50:0x80010000, US SLUS_006.50:0x80031b84
worklog: 11
---

# DBnnMOD.BIN

A database's models (DB22: 215): character parts, props, furniture.
Loaded by `load_models(db)` (0x80044d00, file 242 + db), drawn by the
hand-written model renderer (0x80010000) through an object's draw
(0x80031b84 and others).

## Layout

Little-endian.

```
u32  0xc0de0001
u16  count, u16 pad
u32  at[count]          low 21 bits: offset from the table's end;
                        high 11: the model's own index; 0xffffffff: none
model
  +0x00 u32  link
  +0x04 u32  unknown_04          the loader writes the index here
  +0x08 u16  radius              the culling test's (0x80031834)
  +0x0a u16  unknown_0a
  +0x0c u32  bounds_at           16 bytes (s16 x, y, z, pad twice)
  +0x10 u16  vertices
  +0x12 u16  colours
  +0x14 u32  vertices_at         s16 x, y, z, w each
  +0x18 u32  colours_at          r, g, b, pad each (none when colours is 0)
  +0x1c u32  faces_at
faces: sections u16 kind, u16 count, records
  0x20 F3   0x28 F4    u32 flags, u8 v[4], u8 r, g, b, code     12 bytes
  0x30 G3   0x38 G4    u32 flags, u8 v[4], u8 colour index[4]   12 bytes
  0x24 FT3  0x2c FT4   u32 flags, u8 v[4], u8 r, g, b, pad,
  0x34 GT3  0x3c GT4     u8 u0, v0, u16 clut, u8 u1, v1, u16 tpage,
                         u8 u2, v2, u3, v3                       24 bytes
  0xff the end
```

The loader (0x80044c98) turns the four offsets into pointers and writes
the index at +0x04. The face kinds are the GPU's primitive codes, which the
renderer switches on.

## Measured

All 6286 models in the 31 readable MOD files parse to their last byte with
these sizes (DB08MOD.BIN has no 0xc0de0001). Faces by kind: G3 95689, FT3
72490, F3 38346, G4 7123, FT4 6199, GT3 5220, F4 1594, GT4 231.

## Drawing

An object's model is at object +0x88; `draw_object` (0x80031b84) sets the
GTE to its transform (0x8001eb4c), tests the model's radius, and calls the
renderer with the depth buckets (0x800ce238) and the context at 0x800de3b0;
the projected vertices go to the scratchpad when the model has fewer than
126, to a buffer otherwise. Character textures are in `CHARACTR.RAW`,
which main loads to VRAM (576, 256), 384 x 256, beside the database's A.RAW.

## Unknown

- The vertex's fourth halfword, the bounds block, `unknown_0a`, the face
  flags word (0 or 1 seen).
- How GT faces' colours and the lighting combine (the renderer was not read
  past the vertex pass).
