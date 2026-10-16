---
number: 142
title: Level 4 on the oracle, as Spike
date: 2026-10-16
area: test
files: crates/rugrats/tests/flights.rs
started: 2026-10-16T02:25:06-06:00
took: 2m
---

# 142. Level 4 on the oracle, as Spike

Level 4 never finished loading under the probe (worklogs 119, 123, 126).
The probe started every level as Tommy. With the character hook from
worklog 141 (`--onhook 0x800454dc:0x800c5680=6`) it loads, with Spike at
(-114202, y, 64250), the port's start.

**Spike at rest.** The oracle's y wanders between -526 and -688 for the
first 200 VSyncs, then rests at -688. That is Spike's root offset over
the floor at 0, as in the port. His idle (anm 858) has no root keys, so
the early wandering is the start, not the idle.

**The water** (worklog 117, until now only from the code). Spike was put
at (-133210, -688, 63500), the water spot the port's test finds, and Up
held for 60 VSyncs, with hooks on 0x80050770 and 0x8006f644:

- 0x80050770 (the splash) every game frame while walking in the water,
  2947-2961 and before: the port's per-frame splash is right;
- 0x8006f644(57) once over the walk: Spike's water step, every fifth step
  (0x800c42e4), as the port has it.

The two oracle runs are /mnt/data/claude/scratch/screens/run44.txt and
run45.txt.

**Still unknown:**

- The water damage of phases 2-4 (5 standing, 3 moving every 36 VSyncs)
  was not seen. The phase is 1 at the start.
