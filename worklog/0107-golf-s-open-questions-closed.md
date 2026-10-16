---
number: 107
title: Golf's open questions closed
date: 2026-10-15
area: play
resolves: 35.1, 35.2
started: 2026-10-15T23:04:10-06:00
took: 1m
---

# 107. Golf's open questions closed

Entry 35's two open items are answered by later work:

- 35.1, the world segment test and the exact VectorNormal: 0x8002cd7c
  is ported and matched on 37,198 segments (99). VectorNormal
  (0x8009f278) is exact to the GTE's 16-bit registers. The roll's
  collision step uses both (103).
- 35.2:
  - the cup camera: 106;
  - the Circle height cycle and the aim line sprites 364: 90;
  - the activity golf score card (`Page::Scores`, 0x80095768): 62.

The descriptor's stack leftovers (35.1) are not read by any path the
port takes.

**Still unknown:**
- nothing
