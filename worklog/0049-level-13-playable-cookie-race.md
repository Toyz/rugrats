---
number: 49
title: Level 13 playable: Cookie Race
date: 2026-10-15
area: play
files: crates/rugrats/src/level13.rs, crates/rugrats/src/npc.rs, crates/rugrats/src/game.rs, crates/rugrats/examples/posts.rs, docs/levels/level-13.md
started: 2026-10-15T11:52:10-06:00
took: 31m
---

# 49. Level 13 playable: Cookie Race

A delegated map, docs/levels/level-13.md, traced Cookie Race and checked
it in the oracle. The port now plays it.

**NPCs on slopes.** Angelica stopped at the first rise of her route.
NPC steps asked the height field one unit above the feet: the slope bug
the player had (worklog 43). They now ask from a step (384) above.
Levels 2, 3 and 10 runs give what they gave before.

**Level 13** (`rugrats::level13`):

- Angelica runs her posts from 50 (EpisodeStart 0x80079600) with a post
  reach of 1536 (+0x690, now per NPC) at speed 17. She waits at the
  flag 2 posts.
- Her LP spot is off the map: cine 6 moves her. The port starts her at
  post 50 on the upper floor (-5120), with Tommy's bedroom; where the
  cine leaves her is a guess.
- Post 3 (flag 32) is GoalPost: speech 723, cine 20, lost (the hub,
  only +0x34). Tommy entering room 20 plays cine 21 (room flag 2), won:
  2 bars, 4 with all five cookies (0x80076c24), none if won before.
- ForceCry has no cry here; the one try goes straight to the hub.

Checked: left alone, she ran 50 .. 43, down the stairs, 34 .. 1 and to
post 3, asking for cine 20 at frame 2379. The oracle's GoalPost came
4683 VSyncs after the start, 2341 frames.

**Still unknown:**

- The "player ahead" resume rule and the waits' speech.
- Bumping Angelica (0x800511a8); the activity's ambience.
