---
number: 14
title: The opening cinematic's frame matches the oracle's: camera, props, characters
date: 2026-10-14
area: format, render, decomp, engine
files: crates/rugrats-data/src/cine.rs, crates/rugrats-data/src/placement.rs, crates/rugrats-data/src/model.rs, crates/rugrats-render/src/models.rs, crates/rugrats-render/tests/against_original.rs, crates/rugrats-viewer/src/main.rs, crates/rugrats-hle/examples/cine_shot.rs, crates/rugrats-hle/examples/objects.rs, crates/rugrats-hle/src/gpu.rs, crates/rugrats/src/disc.rs
resolves: 7.1, 10.2, 12.2
started: 2026-10-14T20:25:52-07:00
took: 25m
---

# 14. The opening cinematic's frame matches the oracle's: camera, props, characters

The cinematic stream's frame records are read: a camera (position, and a
quaternion packed as four 13-bit fields over two words whose top 6 bits
count the actors and poses), then 24-byte actors (single-model objects:
quaternion, position, id) and poses (hierarchy objects: position, id, and
one 8-byte quaternion key a channel). The ids are the object registry's,
which the database's LP file assigns; LP records name TEMPLATE.BIN
entries, and a template is a hierarchy or a model id.

`rugrats-viewer 22 --cine F` draws stream frame F: the rooms, every LP
object built on a model (where the stream's actor puts it, else where LP
does), every pose, from the stream's camera. The oracle's `cine_shot`
example runs the original to a counter value, writes the picture, and
lists the camera and each jointed object's position (and with
CINE_PARTS, its parts' quaternions) to set against the stream.

Five corrections got the picture to line up with the original's:

- The camera's quaternion function (0x80020580) is the transpose of the
  parts' (0x8001fc48), pinned by a test. The view matrix (0x8001e0f4)
  scales the camera matrix's x row by 512 / 320 and its y row by 240 /
  240 for the screen; H is 178.
- A model reference is an id, found by binary search on the MOD table's
  high 11 bits (0x80044b74), not a table position. They agree in DB22 only
  up to 78, so props drew as the wrong models.
- The hierarchy root's offset is left out (0x8004a5dc zeroes it): the
  root's joint is the object's position. With it the characters stood
  350-470 units too high.
- The background is not the clear colour but a fill of (217, 175, 141)
  that play_cine's setup asks for (0x8001d218 on view 3). The house has
  no ceilings; the flat peach above the walls is that fill.
- The oracle's picture at counter 200 shows stream frame 199. Static
  props line up in both, and only the moving ones (the twins, the car)
  are a frame apart. Scoring frames 196 to 204 against the shot picks 199.

The camera position matches the oracle's exactly, every pose's keys match
the oracle's part quaternions, and the composed transform (CompMatrix at
0x8009f598 is m0 * m1; the stored result is transposed by 0x80010b54 and
transposed back by 0x8001eb4c) is the port's. Transposing the parts'
rotations instead turns Chuckie around, so that is not it.

**Still unknown:**
- the actors' and poses' animation and state fields
- the LP kinds 6 and 99
- the wall shading (depth cue, lighting) the port does not draw yet
