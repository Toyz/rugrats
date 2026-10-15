---
number: 23
title: Faces, the milk spray and the lens splash in the opening
date: 2026-10-15
area: video, tooling
files: crates/rugrats/src/face.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/stage.rs, crates/rugrats/src/main.rs, crates/rugrats/src/lib.rs, crates/rugrats-data/src/cine.rs, crates/rugrats-data/src/placement.rs, crates/rugrats-render/src/models.rs, crates/rugrats-render/src/renderer.rs, crates/rugrats/examples/cine_faces.rs, crates/rugrats/examples/cine_shot.rs, crates/rugrats/examples/stage_shot.rs, crates/rugrats-hle/examples/cine_blink.rs, crates/rugrats-hle/examples/face_calls.rs, docs/engine/cinematics.md, docs/formats/mod.md
started: 2026-10-15T00:00:58-06:00
took: 25m
---

# 23. Faces, the milk spray and the lens splash in the opening

The opening had no moving faces, and the user saw no milk from Tommy's
bottle. Both come from one mechanism. The stream's per-object "animation"
and "state" fields pick frames that the game copies into VRAM over a
block of texture.

What the original does:

- Face banks are files 531-550, picked by a mask word per database
  (0x800b91a0; the opening's is 0xd515). 0x80074b9c loads them, each
  with a 64-byte header skipped.
- 0x80074ec0(frame, id) is the mouth setter and 0x80075304(id, frame)
  the eye setter. Each maps an id to a VRAM rect and a bank and does a
  LoadImage of that frame.
- An actor record resets both to frame 0 and then applies its fields. A
  pose blinks first (0x80075698), then applies its fields.
- The blink is per object (+0xa5 state, +0xa7 count). It uses a rand of
  its own at 0x800c4c48, which play_cine seeds with the cinematic's
  number. It starts with a 1 in 32 chance, then shows eye frame 1 for
  three records and frame 2 for two.
- Ids 88 and 112 are not characters. They are the milk spray (records
  599-608) and the splash over the lens (from 830). The stream animates
  their textures through the mouth field.

The port:

- `face::Faces` and `face::Blinks` hold the banks and the blinks.
- `Set::faces` applies each record in order, including records the
  display skips, onto the stage's VRAM.
- `Stage::vram_rev` makes the renderer re-upload VRAM, through a new
  `Renderer::load_vram_rect`.

Checks against the oracle:

- Every face rect at records 300, 604 and 840 matches, except blink
  timing.
- `cine_blink` shows why the blinks differ. The oracle drops records
  when a frame runs long: 104 of the first 840, starting at 89. Each drop
  skips a pose pass and its rand calls. Up to the first drop, the rand
  matches step for step.

The splash then drew opaque. The oracle drew it with packet code 0x26.
Finding where the semi bit comes from took several tries:

- First guess: flag bit 25, as in the C face emitters (0x80022964 and
  kin). Hooks on those, and on AddPrim, never fired for cinematic
  models.
- A LP flags word (+0x14, which becomes object +0x0) is 0 for both
  objects, so not that.
- The model's bytes in RAM equal the file's, so nothing patches them.
- The answer is the hand-written model renderer at 0x80010000. It keeps
  `(flags & 0x80) << 18` in gp and ORs it into every packet code. Face
  flag bit 7 is semi-transparency, and an untextured semi face takes its
  blend from bits 5-6.

`model_triangles` now sets `Vtx::SEMI` from bit 7. The stage splits out
the semi triangles for `set_semi`. The splash at record 839 now blends
over the room like the oracle's.

Also found on the way: 0x800221cc picks a face's ordering-table depth
(first, min, max or average vertex, minus 64), which is where the
draw-order bug will be fixed.

**Still unknown:**

- Whether a real PlayStation drops cinematic records as the oracle does,
  which decides when the characters blink.
- Model face flags bits 8-9 (culling, 0x800101dc) and the other bits.
