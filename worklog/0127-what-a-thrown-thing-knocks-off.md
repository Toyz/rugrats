---
number: 127
title: What a thrown thing knocks off
date: 2026-10-16
area: render
files: crates/rugrats/src/world/flights.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/tests/flights.rs
started: 2026-10-16T01:08:33-06:00
took: 4m
---

# 127. What a thrown thing knocks off

A thrown thing's flags carry 0x40000. When it meets an object
(0x80086ef0), the object's touch (+0x58, from template +0x0a through
table 0x800b7e50) is called with the object and the point met. The table:

| touch | function | |
|---|---|---|
| 1 | 0x80050140 | model 637 |
| 2 | 0x800504a4 | 0x80050348(906) |
| 3 | 0x800504c4 | 0x80050348(907) |
| 4 | 0x800504e4 | 919 or 920 by a coin (0x800c4180) |
| 5 | 0x80050570 | 960 |
| 6 | 0x80050590 | 908 |
| 7 | 0x800505b0 | 970 or 637 by a coin (0x800c4184) |
| 8 | 0x800509d8 | feathers (worklog 120) |
| 9 | 0x800511a8 | the characters' touch |
| 10 | 0x800514b0 | collectibles |
| 11 | 0x80051d44 | |
| 12 | 0x80050d58 | |
| 13 | 0x80051db4 | |
| 14 | 0x8005202c | debris |
| 15 | 0x800522ec | |

Touches 1-7 are on level 3's shelves and stands and level 16's 315 and
318.

**Touch 1** (0x80050140) throws one model 637 from the object:

- kind 2, flags 0x30702;
- speed rand % 150, capped at 50 (0x800c4178);
- within 2048 of the point met, taken as a direction, as the code passes
  it;
- after rand % 500 ms (same seed);
- 2000 ms, bounce 1000, spin 3391, the model's radius.

Then sound 50 at the object, but only when 0x800c4ba0, less dt on each
touch, has run out; it is then set back to 100.

**Touches 2-7** (0x80050348, 0x800505b0) throw the model from the point
met:

- flags 0x1030702, ending on the floor;
- speed rand % 250, capped at 100 (0x800c417c; 0x800c4184 for 7);
- within 2048 of up;
- no lifetime, bounce 1000, spin 3000.

The five seeds are 48732, peeked on the original at frame 2 of level 3.
The sound count starts at 0.

The port calls `Flights::touch_parts` when a thrown thing meets an object
box (worklog 125) whose template has touch 1-7. The object test now runs
before the COL wall test. Shelves stand inside COL walls too, and the
wall test used to take the step back before the object was met.

Test: a_thrown_thing_knocks_parts_off throws a thing at level 3's
template 197, its bottom at the box's middle height. A model 637 flight
comes off.

**Still unknown:**

- Most touch 2-7 objects are template flag 8 (0x0a). The port's object
  boxes leave those out, so nothing thrown meets them. Whether the
  original's flight test (0x80086340) skips flag 8 is not known.
- Touches 11-13 and 15 (levels 5, 6, 11) are not ported.
- Pies (level 3's toss, level 16) are not things in the port and do not
  meet objects.
