---
number: 98
title: Mr. Friend waits at home in phase 2
date: 2026-10-15
area: play
files: crates/rugrats/src/actors/npc.rs crates/rugrats/src/levels/mr_friend.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/src/levels/grandpas_teeth.rs crates/rugrats/tests/play.rs
resolves: 40.2
started: 2026-10-15T22:41:58-06:00
took: 2m
---

# 98. Mr. Friend waits at home in phase 2

40.2 left the oracle's sight of 46 in level 9's phase 2 unported
(docs/levels/level-09.md, "Cine 18 and phase 2"). He comes back home
still in state 19, the state he died in. 0x80083728 resets his mode,
counters, health and place, but not his state. State 19 does not walk,
so he stood in mode 4 from 4222 to 6400 while 91 and 800 chased and
bit. Brought near Tommy he bit (mode 5), went through mode 1 to state 0,
and chased from then on.

This is how the game plays, not an oracle artefact, so the port now does
the same. `Npc::rooted` marks a following NPC whose state does not walk:

- it holds its place until the target is within a bite (half its range);
- the bite, or a hit (mode 1 ends in state 0), clears it.

`cine_finished(18)` sets it for 46 alone. The test
`level_9_mr_friend_waits_after_his_return` checks three things: he waits
120 frames with Tommy 6000 away, bites when Tommy stands beside him, and
chases afterwards.

**Still unknown:**
- whether a pie thrown from afar wakes him (a hit does in the port; the oracle run did not try it)
