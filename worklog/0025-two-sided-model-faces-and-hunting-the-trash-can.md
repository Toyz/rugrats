---
number: 25
title: Two-sided model faces, and hunting the trash can
date: 2026-10-15
area: video, tooling
files: crates/rugrats-render/src/models.rs, crates/rugrats/examples/opening_shot.rs, crates/rugrats/examples/cine_faces.rs, crates/rugrats-hle/examples/cine_frames.rs, docs/formats/mod.md
started: 2026-10-15T00:49:23-06:00
took: 13m
---

# 25. Two-sided model faces, and hunting the trash can

The user reported that Chuckie vanishes for a frame when he crashes into
the trash can. That search led to two fixes and one dead end.

Tools:

- The port's opening shot example clashed with the oracle's
  `cine_shot`: both build to the same target directory, so whichever
  built last ran. The port's is now `opening_shot`. It also takes a
  range (`F DIR TO STEP`) for contact sheets.
- `cine_frames` (oracle) writes the picture for every cinematic frame
  counter in a range in one run.

The search: I compared the port to the oracle frame by frame over
records 170-240 and 280-700, plus every 20 records across the whole
opening.

- The steady difference is about 12 (mean absolute per channel), from
  the shading the port does not draw.
- Every spike traced back to oracle drops. After a drop, the oracle's
  picture for a counter shows a record two or three behind. At 338 the
  oracle matches the port's 335.
- Chuckie's run, his fall, the can and the robot (records 500-545) match
  the oracle. Nothing vanishes in the records themselves, so the report
  has to come from live playback or from a moment I have not located.

Two things in the hand-written model renderer's face test (0x80010174)
were missing in the port:

- Flag bit 0 skips NCLIP, so the face is drawn from both sides. The port
  culled back faces for every model, which loses the milk spray
  (model 953, flags 1) and hair strands seen from behind. These faces
  now go in both windings.
- Bits 8-9 pick the ordering-table depth: average (and dropped at 16000
  or more), farthest, or nearest vertex, shifted right 2. This is
  documented for the draw-order work and not used yet.

Also, a template swap in the actor reader (1149 to 1130's model) does
not apply to the opening: DB22 places neither template.

**Still unknown:**

- Where the trash-can vanish happens. It needs a time from the user.
