---
number: 10
title: Rooms drawn natively: blob meshes, the texture table, room centres, rugrats-viewer
date: 2026-10-14
area: format, render, decomp, tooling, build
files: crates/rugrats-data/src/level.rs, crates/rugrats-render/src/rooms.rs, crates/rugrats-render/src/renderer.rs, crates/rugrats-viewer/src/main.rs
resolves: 8.2
started: 2026-10-14T20:06:26-07:00
took: 8m
---

# 10. Rooms drawn natively: blob meshes, the texture table, room centres, rugrats-viewer

`rugrats-viewer 22` flies around database 22, the Pickles' house of the
opening cinematic: staircase, hallway, the blue carpet, doors, the painted
walls with their baked shading, all from the disc's data and drawn with
wgpu. It is the first 3D the port draws.

## The geometry, read from the hand-written renderer

The room renderer (`draw_room`, 0x80033818) tests each blob in a room's list
against its bounding box (`blob_visible`, 0x80032cf4, eight corners through
RTPT, out if all depths are 0 or less) and hands survivors to the
hand-written renderer at 0x80010e14. A blob is a link word, the bounds, a
vertex count, the offset of its faces, then 12-byte vertices (s16 x, y, z,
pad; r, g, b and a one-shot brightening), then face sections `u16 kind, u16
count`: 0x61 quads of four u8 indices and a u32 texture word, 0x60
triangles of three indices, a pad byte (0xcd in all 13772) and the texture
word, 0xff the end. A Python sweep over all 3083 blobs of the 31 readable
LEVEL files found only those three kinds and ended every blob exactly on
its 0xff; `every_level_parses_and_its_faces_name_its_textures` now holds
the Rust parser to the same.

The texture word's low half indexes a 12-byte table the renderer reads
through its context (+0x58) and copies into the packet's uv slots word for
word: u0 v0 CLUT, u1 v1 texture page, u2 v2 u3 v3, TEXPOS.DAT's layout. The
cinematic never loads TEXPOS.DAT, and the LEVEL file's final section is
12-byte records: in every file the highest texture any face names is
exactly that section's count less one (DB22: 205 of 206; DB15: 1030 of
1031). It is the room texture table. The texture image is `DBnnA.RAW`,
uploaded whole by `load_rooms` to VRAM (0, 256), 576 x 256.

All of a database's blobs are centred on the origin; the 112-byte room
record places them: +0x00 the room's centre (s32 x, y, z), +0x10 and +0x20
its bounds, +0x30 its portal count. DB22's room 0 is centred at (19456,
-7168, 1536) with bounds (13312, -10240, 0) to (25600, -4096, 3072), and
its blobs reach exactly +-6144, +-3072, +-2048.

## The renderer, from hwtr

`rugrats-render` is hwtr's wgpu renderer unchanged where it is the
console's: VRAM as an R16Uint texture, texture page and CLUT decoded in the
fragment shader, texel times vertex colour over 128, colour 0 transparent,
the four semi-transparency modes. Only the triangles are Rugrats': the
first shot drew the floor and no walls, since the game's front faces (NCLIP
positive, y down) are clockwise in the renderer's view; faces are now
emitted 0 2 1 and 1 2 3.

Not yet ported: the depth cue beyond 8192, the near-face subdivision (the
port draws with perspective-correct texturing, so subdivision has nothing
to correct), objects, and the portal visibility walk.

**Still unknown:**
- the far colour the depth cue blends to, and where the game sets it
- whether the house has ceilings the camera never sees, or none
