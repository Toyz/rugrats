---
number: 189
title: What this session has answered
date: 2026-10-17
area: test
resolves: 30.2, 161.3, 163.2, 30.3, 162.2, 182.1, 169.4, 173.1, 170.2, 172.3, 173.2, 180.1, 180.2, 184.1, 180.4, 181.1, 184.2, 184.3, 187.2, 185.1
started: 2026-10-17T04:50:28-06:00
took: 1m
---

# 189. What this session has answered

A triage of the questions this session's work has answered. Each answer is in the entry named; the oracle scripts (`tests/hle.rs`) hold it.

- 30.2, 161.3, 163.2: when the grab and the release come. Since 180 the animator is the original's, so the frame events (+0xe0 bits 2048 and 16) come on the original's frames. The put-down lets go only at its event (186).
- 30.3, 162.2, 182.1: a flight's collisions with things and walls by 0x80087594, including thrown things against things at rest (183). The rolling path (0x80086ef0) stays open (185).
- 169.4, 173.1: the original's y carries the animations' root. Since 184 the port's body height with its root matches the original's on every frame of nine scripts, with the landing's squash and the run's bob.
- 170.2, 172.3, 173.2, 180.1: the parts' matrices (+0x158) and +0x68 are made in the original's integers (181), and characters are met by the pose as last drawn.
- 180.2, 184.1: the bump is table state 28 after a run (187).
- 180.4: a thrown flight starts at the object's position plus its bound's centre (182).
- 181.1: the 1 in height was the root (184).
- 184.2: the oracle's mid-fidget boot is taken over by syncing the animators (187).
- 184.3, 187.2: the NPCs step before they ask (188).
- 185.1: the shelf's put-down in level 6 lets go of nothing: its event never comes (186).

**Still unknown:** nothing
