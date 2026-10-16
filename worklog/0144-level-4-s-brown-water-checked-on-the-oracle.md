---
number: 144
title: Level 4's brown water checked on the oracle
date: 2026-10-16
area: test
files: crates/rugrats/src/levels/cynthia.rs
started: 2026-10-16T02:31:21-06:00
took: 1m
---

# 144. Level 4's brown water checked on the oracle

Level 4 was booted as Spike (worklog 142) with the phase poked to 2
(0x800c4cd0). Spike was put in the water at (-133210, -688, 63500), stood,
walked from VSync 2960 for 40, then stood again. Damage (0x800780ac) was
hooked:

| VSync | amount | from |
|---|---|---|
| 2893 | 5 | 0x8005d93c (standing) |
| 2929 | 5 | standing |
| 2965 | 5 | standing |
| 2969 | 3 | 0x8005eb08 (walking) |
| 3005 | 5 | standing |
| 3041 | 5 | standing |

So:

- standing costs 5 and walking 3, each every 36 VSyncs;
- each has its own timer: the walk's first hit came 4 VSyncs after a
  standing one;
- the first hit is at once.

The port's cynthia.rs has it so: `hurt[2]`, 180 dt (18 frames, 36
VSyncs), 5 standing, 3 moving, a timer at 0 hurts at once. Nothing to
change. The run is /mnt/data/claude/scratch/screens/run46.txt.

**Still unknown:**

- nothing
