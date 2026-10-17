---
number: 171
title: The camera's rows and the objects in the probe
date: 2026-10-16
area: play
files: crates/rugrats/src/world/game/camera.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/gte.rs, crates/rugrats/src/world/boxhit.rs, crates/rugrats/src/actors/ground.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/examples/boxhit_check.rs, crates/rugrats/examples/object_check.rs, crates/rugrats/examples/ground_check.rs, crates/rugrats/examples/below_check.rs, crates/rugrats/examples/solids_dump.rs, crates/rugrats/examples/common/oracle.rs, crates/rugrats/tests/hle.rs
started: 2026-10-16T20:12:48-06:00
took: 1h 8m
---

# 171. The camera's rows and the objects in the probe

The follow camera's rows are now the original's integers, and the ground
probe meets objects as the original does: every part proved against the
original called on the live player.

**The camera.** The stick reads its heading off the camera matrix
(+0x224), and the port built that matrix in floats: a unit off now and
then, so the heading the stick set drifted (level 18's slide was off by
5 by frame 12). The original's (0x8001e838 for the follow camera): the
look-at from the eye as drawn toward the look-at point (0x800211e8: z the
way there by VectorNormal, x (z.z, 0, -z.x), y (-(zx zy), zx^2 + zz^2,
-(zy zz)) in 12-bit steps, then MatrixNormal), then the look aside, the
swing about y if there is one, else the tilt about x, never both
(0x80020c30, 0x80020ca0, MulMatrix0). Its rows are not unit (4093 to
4119 long: libgte's VectorNormal). Logged against the original frame by
frame it is the same matrix to the unit. The row scaling for the screen
(0x8001e0f4: x by 512/320) is the drawn copy's, not the one the stick
reads. The ground step's length now uses the exact VectorNormal too
(`along`, 0x80076944): along a wall the port stepped 169 where the
original steps 170.

**Objects in the probe** (`actors::ground::Ground`, `world::boxhit`):

- **The dispatch** (0x8006a714): the bound's cube about the object (half
  side the largest of +0x78's extents, 0x800698fc), then a model's boxes
  (0x80069c40, first met wins), a character's one hierarchy box
  (0x8006a008), or a hierarchy of posed parts (0x8006a318: each part's
  matrix from +0x158, the nearest hit of all parts kept, by part).
- **The object probe** (0x80063d3c): each object near (the nearby list,
  +0x100) but those of type flag 8, a model's every box but the one stood
  on, a segment from the body at the box's middle height (within the
  body's span) to the probe; met past its start (0x80067284), and no
  height-field wall as near (0x80063c3c: sums of squares, unsigned), it
  is +0xf4 and +0xfc.
- **The normal is a buffer, not a result**: each box's slab test writes
  its face's normal there, hit or not (one or two axes out), in the box's
  own frame; a hit turns it back. A segment starting inside a box turns
  back whatever the buffer held. The port carries it as the original
  does, across boxes, objects and probes.
- **The floor on an object met** (0x80067448): across at the box's middle
  (turned by the object's angles, 0x80075ee8 = 0x80020d10's matrix) toward
  the point, the point nudged 16 on along the body's own z (0x80067374),
  then down through 2 * +0x86 above and below the body's origin; met again
  on the same box that is the floor (none if met at its start), else the
  box's bottom face unturned.
- **The thing stood on** (+0xf8, +0xfd): a floor probe down through it
  takes its top when as high; going down a step, the wall test meets it
  on the drop's way back; the object probe passes its box by.
- **The wall test** (0x80064178): going up, an object met keeps its face.
- **The record left on the player** is the one decided on, copied back;
  where none is (both sides held, the middle clear) the stop is on the
  last probe's, the right's.
- **The run's probe of its own step** (0x800645d8, mode 2): its kind
  sets the thing stood on; a rise or small drop is followed (position and
  floor by the delta); a step up or down with an upright face starts the
  eased step (flag 0x80, +0x150 to +0x14c, 0x80066f28); a drop past the
  limit falls (state 4).
- **On a slippery floor** (the floor's flag 2, player flag 0x100000 set
  each frame by 0x8005bd9c) a wall turns the way back off it, a
  reflection (0x8007627c), not along it.

Proved: `boxhit_check` (slab, object, dispatch: 100,000 random cases
each, 0 different); `object_check` (the object probe, 2,500 spots by
objects in five levels); `ground_check` (the whole probe with the
objects near: 4,400 spots in ten levels, 0 different, 400 meeting a
thing); `below_check` (standing on a thing, both modes: 946 spots in
seven levels, 0 different).

The original's frame time is not always 10: `D_800c3f1c` is 5 a vertical
blank, 15 when a frame takes three (level 2 at its start). The checks
read it; the port steps at a fixed 10.

**Placed objects' angles** (`Trig::placed_angles`): the placement's
quaternion inverted (0x8001f684), its matrix (0x8001f9fc: 2^24 over the
squared length, 12-bit products, MatrixNormal), the angles read off it
(0x8002114c: b = -asin m21, a = ratan2(-m02, m22), c = ratan2(-m10,
m11)). Exact for all 747 placed objects of levels 1, 3 and 18
(`solids_dump`). An object's size (+0x86) and bound are its model's
radius, its boxes the model's, for nine in ten; characters have size
8192.

With the camera exact, level 18's slide matches the original exactly
for its first 17 frames (from frame 5 before).

**Still unknown:**
- The port's world does not yet hand its objects to `Ground`: things,
  doors and characters still meet the player through the float boxes,
  so level 18's slide along door 213 and level 1's jump at toy 403 still
  differ.
- The nearby list's order: the rooms' object lists (0x800c4a8c, +0x48
  and +0x4c), each object tested against three times the body's extents
  (0x8006abbc) and prepended; the port would test in its own order, which
  differs only when two objects are met by one probe.
- The objects whose model is not their template's (type 881 placed as
  836 in level 3), whose radius or boxes differ from the template's
  model's (about one in ten), and the characters' posed parts.
- The floor under after a move with objects (0x800627a0, 0x80065578),
  the eased step's per-frame motion, the climb test on objects
  (0x80058da4) and the type 11 objects' callback when held (+0x58,
  0x8005d5c4).
- The follow camera's eye drifts from the original's by a few units
  after a turn (the spring, 0x80027304), so the matrix follows it.
