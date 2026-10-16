---
number: 128
title: The piano, the rugs and what floors touch
date: 2026-10-16
area: audio
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/boxes.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/app/mod.rs, crates/rugrats/src/audio/sound.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T01:12:27-06:00
took: 6m
---

# 128. The piano, the rugs and what floors touch

The walk/run handler 0x8005e1f4 does more each frame for the player's own
walk (object flag 0x4000 off):

1. **The object in focus** (+0x1e8, worklog 118) has its touch called
   with the player's position. The flag-8 floor objects with touches:
   - level 11's 505: touch 12;
   - level 6's 926: touch 13;
   - level 6's piano keys 999-1006, 1046 and 1047: touch 15.
2. **Tile flag 1, not on an object:** the splash (0x80050770, worklog
   117). In level 4 it also does Damage(player, 0, 3) every 36 VSyncs,
   unless progress +0x30 bit 32 is set or 0x800c4cd0 is 1. That is not
   ported: level 4 does not load under the probe to check it.
3. **Tile flag 4, not on an object:** 0x80054ee0 at a foot, slot 7 or 3.
   The foot is picked by the low bit of the C library's rand
   (0x800b1158, x * 0x41c64e6d + 0x3039 at 0x800eaf80). The puff is one
   sprite 266 (TEXPOS + 3192), kind 5, flags 0x305, size 50, 2000 ms,
   speed 50, within 1000 of up, face 0xd2800000.
4. **On a step** (event 256), with a rug in focus, a sound at the rug:
   1160 → 73, 1161 → 75, 1162 → 76, 1163 → 74. These are level 16's rugs.

**The piano** (0x800522ec). The note goes through 0x8006f3a4, which keys
on program 4, tone 3, note (key - 999) + 78 at the key, unless it is the
note last played (0x800c4358). Then the sprites, from the 0x800c41a0
seed:

- keys 1046 and 1047: one of sprites 417-420, size 300, dropped straight
  down from one of objects 960-966, picked by the seed's first draw;
- the others: two of 415 or 416, size 100, within 2048 of up from the
  player.

Both are flags 0x10505, 500 ms, bounce 1000, spin 3000, speed rand % 250
capped at 100.

**Touch 12** (0x80050d58): five of one of sprites 352-354 from the
player, flags 0x1030705, size 100, ending on the floor.

**Touch 13** (0x80051db4): three of one of sprites 376-379 from the
object, flags 0x30705, size 100, 1000 ms, bounce 2048, spin 3391, face
0xc0800000, the object's damage.

**Direct keys.** A note is not an effect number, so `Effect` gains `key`
(program, tone, note) and `Game::key_at`, which is heard like `play_at`.
The app keys those on directly. Box toss's crunch (0x8006f2fc) was played
as effect 43 + n (worklog 124). It is really program 0, tone 14 (13 in
level 20), note 43 + (rand & 15), and now uses `key_at`.

Test: the_piano_plays walks Chuckie over key 999. Key [4, 3, 78] sounds
and sprites 415/416 fly.

**Still unknown:**

- Level 4's water damage, as above.
- The C rand's seed (0x800eaf80); the port starts it at 0.
- The floor seeds 0x800c4190, 0x800c4198 and 0x800c41a0 are taken as
  48732, like their neighbours, without a peek.
- The flag-4 puff's radius (the descriptor sets 50, then 0): the port
  gives it 50.
