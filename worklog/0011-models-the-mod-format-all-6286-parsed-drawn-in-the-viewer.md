---
number: 11
title: Models: the MOD format, all 6286 parsed, drawn in the viewer
date: 2026-10-14
area: format, render, decomp
files: crates/rugrats-data/src/model.rs, crates/rugrats-render/src/models.rs, crates/rugrats-viewer/src/main.rs
started: 2026-10-14T20:14:27-07:00
took: 5m
---

# 11. Models: the MOD format, all 6286 parsed, drawn in the viewer

A database's models are its MOD file: `0xc0de0001`, a count, a table of
offsets (the high 11 bits of each are the model's own index), and models
with a 32-byte header pointing at vertices (8 bytes: s16 x, y, z, w), a
colour palette, and face sections. The face sections are the GPU's own
primitive codes, which the hand-written model renderer (0x80010000)
switches on: F3, F4, G3, G4 records are 12 bytes (a flags word, four vertex
indices, then an RGB and the code for flat faces or four palette indices
for gouraud), FT3, FT4, GT3, GT4 are 24 (the same, then u0 v0 CLUT, u1 v1
page, u2 v2 u3 v3).

Getting there took three passes of a Python sweep over all 31 MOD files.
Textured records at 16 bytes derailed at the first FT3; at 24 for triangles
and 28 for quads, 816 of 6286 models still failed, on FT4; the fourth texel
pair shares the last word with the third, so every textured kind is 24 and
all 6286 parse to their last byte. `every_model_parses` holds the Rust
reader to that.

`rugrats-viewer DB --models` lays a database's models out in rows. DB22's
are the cast in pieces (hands, shirts, legs, Grandpa's head and glasses)
and the house's furniture, coloured and textured: character textures come
from `CHARACTR.RAW`, which main loads to VRAM (576, 256), 384 x 256, beside
the database's own A.RAW. The format page is `docs/formats/mod.md`.

**Still unknown:**
- the vertex's fourth halfword and the face flags word
- how the renderer lights gouraud faces, if it does beyond the palette
