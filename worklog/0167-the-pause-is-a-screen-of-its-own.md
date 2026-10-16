---
number: 167
title: The pause is a screen of its own
date: 2026-10-16
area: ui
files: crates/rugrats/src/ui/screens.rs, crates/rugrats/src/world/game/pause.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/app/mod.rs, crates/rugrats/src/ui/front.rs, crates/rugrats/examples/play_script.rs
started: 2026-10-16T15:53:09-06:00
took: 9m
---

# 167. The pause is a screen of its own

The in-game pause is now a screen of its own, as the original's is. The
oracle (hub, Start in play, then Save Game?): the pause (0x8003e05c) puts
the front end's machinery up over the level, not its text over the world:
the tiled backdrop cycling its patterns (0x80034fdc), the burst sign
(0x8003f41c: the camera step 0x8003f350 and the sign 0x8003f184), the
CANCEL and ACCEPT cubes with their labels by the second camera
(0x80040010), the counters under them (models 292 and 291, 0x80090174),
and on a card page's footer the PRESS X cube (0x80035798). The sign is in
place five frames after Start (no fly-in seen at the shots' 10 frames).
The port had drawn only the menu's text over the running level.

`screens::Backing` is that screen (the front end's VRAM, backdrop, burst
and button cube, the camera where the front end's stops); `Pause` holds
one, with its three cubes (new at each opening, as 0x800398bc makes them)
and the level's two counter icons copied into its VRAM from the level's.
`Game::view` is the stage to show: the pause's while paused; the app and
`play_script` show it. The port's pictures now match the oracle's page for
page (pause, Save Game).

**Still unknown:**
- Whether the sign flies in or out at the pause's start and end (0x80040268
  steps the camera; the oracle's first shot, 5 frames on, has it in
  place).
