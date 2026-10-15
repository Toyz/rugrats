---
title: Animations (DBnnANM.BIN, DBnnFRM.BIN, DBnnROT.BIN)
status: partial
discs: US
covers: US /DATA/DB00/DB00ANM.BIN, US /DATA/DB00/DB00FRM.BIN, US /DATA/DB00/DB00ROT.BIN, US SLUS_006.50:0x80054fd0, US SLUS_006.50:0x80055170, US SLUS_006.50:0x800552bc, US SLUS_006.50:0x8001fc48, US SLUS_006.50:0x8004a5dc
worklog: 13
---

# Animations

Three files per database, loaded together by `load_anims(db)`
(0x80054fd0: files 274, 306, 338 + db).

## Layout

Little-endian.

```
ANM  (u16 first_frame, u16 frames)[1280]   first 0xffff: no animation
FRM  frame, 8 bytes:
       u16 unknown_0
       u16 duration          the loader multiplies it by 5
       u16 first_key, u16 keys
ROT  key, 8 bytes (two words):
       w0  bits 0-13 q0, 14-27 q1 (signed); bit 29 set: a lift key
       w1  bits 0-13 q2, 14-26 q3 (signed), 27-31 channel
```

A rotation key is the quaternion (x, y, z, w) = (q0, q1, q2, q3) for its
channel, at any scale; a lift key adds w0's bits 14-27 to the object's y
(object +0x1c). A hierarchy maps channels to parts: the byte at hierarchy
+ 0x14 + channel is the part, 0xff none.

## Playing

`anim_set_frame` (0x80055170) writes a frame's quaternions into the parts
(part + 4..+0x13 of the object's hierarchy copy); `anim_blend`
(0x800552bc) interpolates between two frames by `(duration - left) << 12
/ duration` (0x800202ec). `UpdatePartRotations` (0x8004a5dc) then makes
each part's matrix: the quaternion as a matrix (0x8001fc48, normalised by
its squared length), composed with its parent's on a per-depth stack in
the scratchpad (0x8009f598), translated by the part's offset, and the
object's position added.

The quaternion's matrix, row-major, is

```
1 - 2(yy + zz)   2(xy - wz)       2(xz + wy)
2(xy + wz)       1 - 2(xx + zz)   2(yz - wx)
2(xz - wy)       2(yz + wx)       1 - 2(xx + yy)
```

over `x^2 + y^2 + z^2 + w^2`; `quaternion_matrices_match_the_original`
holds the port's to 0x8001fc48 called in rrt-emu.

DB00 has 140 animations (0-3, 132-155, 239, 243-246, 328-...); DB22's FRM
and ROT are not in the file table ([cut](../cut/db22-animation.md)), and
its opening cinematic poses its characters from the stream.

## Unknown

- `unknown_0`, the duration's unit and how fast it runs down.
- Which animations belong to which hierarchy, and which ids mean what.
