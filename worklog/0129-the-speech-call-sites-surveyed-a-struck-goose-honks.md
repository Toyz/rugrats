---
number: 129
title: The speech call sites, surveyed; a struck goose honks
date: 2026-10-16
area: audio
files: crates/rugrats/src/levels/grandpas_teeth.rs
resolves: 39.3, 53.1
started: 2026-10-16T01:18:04-06:00
took: 5m
---

# 129. The speech call sites, surveyed; a struck goose honks

This is the survey that open item 53.1 asked for. Every call of
0x8008da4c (speech) and 0x8008df78 (speech at once) in the executable was
listed by its function and checked against the port:

- by its literal line number, or
- by the address of the table it picks from.

**Already wired:**

- hit lines (0x800511a8): 1065, 1076, 1079, 185, 556, 557;
- the cookie line 216, the slip 212, the building 1183, the stand
  handler's 1159 and 1167;
- the throws 191/192 and the put-down 190;
- GoalPost 723 (0x80077070);
- the level tables 0x800b9220, 0x800b922c and 0x800b9244;
- golf's 72, 938, 0x800b92d0 and 0x800b92d8;
- the ambience (table 0x800b9618, every level's entry: 0x80080330,
  0x8008046c, 0x80080648, 0x80080934, 0x80080a20, 0x80080ba8,
  0x80080cd8, 0x80080e08, 0x80080fe0, 0x800810c8, 0x800811c0,
  0x80081230, 0x800812c8, 0x800813a0, and the golf holes'
  0x80081474-0x80081758);
- Mr. Friend's 0x800b96c8, level 10's tag 0x800b9728, level 13's resume
  0x800b96e0;
- the hub's 0x800b96ec, the cry's 2040 + character, 2046.

The 0x8008052c and 0x800805e0 that looked unwired are helpers of those.

**Newly wired:** a goose struck by a thrown thing. 0x800511a8 gives
Spike (char 6) and the geese (char 12) animation 8, mode 1. At that
animation's event (+0xe0 bit 512, 0x80084084), a goose plays sound 32 at
itself and says a line of 0x800b93f4 (405, 49, 437, 311, 323, 469; rand
% 6, 0x800805e0). Level 2's script ignored `struck` before. It now holds
the flinch until the animation ends and plays the honk and line at the
strike.

**Resolved from earlier entries:**

- 39.3: the piano keys' notes (worklog 128).
- 53.1: this survey.

**Still unknown:**

- The prowler's 1134 in level 26 (55.2) is still not wired.
- The honk is played at the strike, not at the animation's event 512.
