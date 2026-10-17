---
number: 181
title: The parts' matrices are the original's integers
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/pose.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T01:06:19-06:00
took: 17m
---

# 181. The parts' matrices are the original's integers

The parts' matrices were float: a quaternion to a float matrix, composed down the hierarchy in floats. They are now the original's integers (UpdatePartRotations, 0x8004a5dc), in `actors::pose`:

- the object's angles' matrix at the bottom of a stack by depth (0x80020d10, `Trig::rotation`), its translation 0;
- each part's turn: its kept quaternion through 0x8001fc48, which is the transpose of 0x8001f9fc's layout, then MatrixNormal. A part never sampled keeps the disc's zeros, which give no turn;
- composed onto its parent's with CompMatrix (0x8009f598): a column at a time through MVMVA (sf = 1, so a floor shift), each element kept to 16 bits, the offset's components taken as 16 bits and turned by the parent's, plus the parent's. The root's offset is left out;
- transposed into the +0x158 layout (the world into the part), each joint moved by the object's position. The joints' greatest y is +0x68.

The animator's kept pose was already by channel; `Animator::turns` gives it by part. The float `locals` is gone.

The player draws, holds things and finds its +0x68 and part joints (`part_at`, `lowest_joint`) by this pose. The grip and carry are integer too: ApplyMatrixSV (0x8009fc38, 0x8009fbe8) with the body's matrix (+4), and the carry first moves the thing on the frame after the grab, as mode 1 does.

NPCs draw by it and keep the pose as drawn (`Npc::drawn`). Flag 0x2000 characters are met by their posed parts in the collision tests (0x8006a318), and those tests read the +0x158 the last draw left (the draw poses once a frame, 0x8003205c). A body never drawn is posed afresh, as 0x8006a318 poses into a scratch.

A tenth oracle script, `hub_pick_up_as_the_original`, picks up level 1's toy 896, carries it and throws it on both. Every part's quaternion equals the original's (the hierarchy's +4 of each part) on every frame after the pick-up's first eight, which still blend from the fidget the oracle's boot left it in. The carried toy is on the original's x and z on every frame and within 1 of its y. The other 9 scripts still match.

**Still unknown:**
- The 1 in height: the original's y carries the animation's root (+0xda through each state handler: 0x8005cb48, 0x8005d978; +0xde on a first frame, 0x80055904), while the port keeps a root curve apart (`root_y`) that runs about two frames ahead of the original's.
- The cines still pose their hierarchies with the float `posed_triangles`.
