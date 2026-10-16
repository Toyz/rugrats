---
number: 83
title: Rugrats never turns reverb up
date: 2026-10-15
area: audio
files: docs/engine/sound.md
resolves: 82.1
started: 2026-10-15T20:52:27-06:00
took: 2m
---

# 83. Rugrats never turns reverb up

Whether the game's reverb is heard (82's question): no.

- **The SPU init** (0x800add98, from the sound init's 0x800a5e08) writes SPUCNT and leaves the reverb volumes at 0.
- **SpuSetReverb** (0x800af2f8) and the reverb mode/depth setter (0x800af4d8) are reached only from libsnd.
  - SsUtSetReverbType (0x800a9e38) and the depth calls (0x800a9d68, 0x800a9da8) sit behind the SEQ control handlers that SsSepOpen (0x800a5ec8) stores at 0x800e9cd0.. (fn 0x800a78b8 and its neighbours).
  - The dispatcher 0x800a7a58 sends CC 6 (data entry), 7, 10, 11, 64, 91, 98-101 and 121 to them.
- **The game's songs:** across its 33 SEP files the controllers are 0, 6, 7, 10, 11, 98 and 99 only, so no CC 91 (reverb depth). The NRPNs are 20 and 30 (the loop marks) and 40, always with data 0.
- So nothing turns reverb on with a depth. Only one tone of 1779 has the reverb bit anyway. libsnd's missing reverb loses nothing here.

**Still unknown:**
- nothing
****
