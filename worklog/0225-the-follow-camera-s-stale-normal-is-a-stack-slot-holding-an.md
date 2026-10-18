---
number: 225
title: The follow camera's stale normal is a stack slot holding an address in the original's memory
date: 2026-10-18
area: engine
files: crates/rugrats/src/world/game/camera.rs, docs/engine/gameplay.md
started: 2026-10-18T13:01:35-06:00
took: 3m
---

# 225. The follow camera's stale normal is a stack slot holding an address in the original's memory

The follow camera's stale normal is traced to what it holds. It cannot be carried over to the port: part of it is an address in the original's memory.

**The slot (0x8006a9cc's local +0x38).** The camera's line test (0x80090554 → 0x80090340 → 0x8006a9cc) hands each object's exact test (0x80069c40) one normal buffer in 0x8006a9cc's frame, at 0x801ff948 in the oracle. A hit turns the buffer in place by the object's matrix (0x80010c14 at 0x80069f8c). A start inside a box writes nothing before that turn.

Traced on level 3's shelf, every 0x80069c40 call with its buffer before and after, for 14 frames of the turn-in-place run:
- The camera's line tests each frame (0x8002655c, 0x8002661c, 0x80026798, then the view test's 0x80026a30 and the side tests) all find the slot as [x, 0, P]. P is written just before each line test and is an address in the original's memory: 0x800E17C4, 0x800E162C, 0x800E17A4, 0x800E160C, by call.
- At the view test (0x80026914, from 0x800280a0) the slot was [1642, 0, 0x800E17C4]. The shelf's turn of it (the GTE taking the low halves) gave the [1062700, 0, 889182] that the push sets to 18.

So the push from inside a box depends on where the original's heap put a record. The port keeps no such addresses and pushes nothing. The camera's doc (gameplay.md, mode 3) and `Game::blocked` say so.

**Still unknown:**
- Which record P's address is (a room's or an object list node, saved by a callee of 0x80090554 at that depth), and where 1642 comes from.
- Whether the real console lags where the oracle's instruction budget does.
- The order of the characters' updates against Angelica's in the frame of a knock.
