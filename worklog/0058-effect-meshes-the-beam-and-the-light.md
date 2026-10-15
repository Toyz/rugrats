---
number: 58
title: Effect meshes: the beam and the light
date: 2026-10-15
area: render
files: crates/rugrats/src/fx.rs crates/rugrats/src/level5.rs crates/rugrats/src/level12.rs
started: 2026-10-15T15:36:37-06:00
took: 7m
---

# 58. Effect meshes: the beam and the light

0x800243c4 draws small meshes kept in the executable. Its descriptor is: u16 faces, u16 vertices, then pointers to flags, corner indices, vertices (s16 x, y, z, pad), colour indices, palette and textures (0 here). 0x80023c2c walks the faces:
- the flag word's top two bits give the vertex count (2 a triangle, 3 a quad);
- 0x20000000 makes the face semi-transparent;
- the corners are bytes, and the colours are per-corner palette indices.

rugrats::fx::Mesh loads two of them and draws them stretched along a segment: z along it, scaled by length / 4096 (0x8002474c), and x, y scaled by the width / 4096, centred on the midpoint. Untextured faces use Vtx::FLAT; semi-transparent faces are additive (abr 1).
- **Beam** (0x80090d34, tables 0x800bb0ec, 0x800bafcc, 0x800baefc, 0x800bb05c, 0x800baee4; 36 faces, 26 vertices; width 5000): three spindles along z = +-2048. Radius 32 is an opaque grey/white core; radii 128 and 256 are an additive red glow.
- **Light** (0x80090e70, tables 0x800bb25c, 0x800bb1fc, 0x800bb194, 0x800bb22c, 0x800bb17c; 12 faces, 13 vertices; width 15000): a cone from z +2048 to rings of radius 128/256/512 at -2048, additive grey fading to black.
- **Level 5 beam speed and life,** from the table 0x800bb28c and 0x800b44d4: speed +0x44 = 10 + 2048, so it moves (speed x dt) >> 12 of its unit direction a frame, 20580. Life is 1000, less 33 a frame at dt 10, so about 30 frames. The port had guessed 600 a frame and 40 frames, and drew a battery model. It is now tested in 256-unit steps along each frame's segment and drawn over that segment: a red laser with a white core (screenshot).
- **Level 12 light:** each frame while shining, the cone is drawn from hand bone 15 + (0, 0, 350) to 4096 ahead (screenshot).

**Still unknown:**
- The orientation 0x8007652c / 0x8008e6c8 computes was not traced. The beam is symmetric along z; the light's point is put at the hand (guess).
- The beam's first-wall reflection (0x8007666c >= 1025, sound 70) is not ported; the beam stops at a wall.
- The light's length with forward speed, (100 + speed) / 100 x 4096, is not ported.
