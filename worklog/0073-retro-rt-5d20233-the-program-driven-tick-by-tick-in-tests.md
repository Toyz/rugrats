---
number: 73
title: retro_rt 5d20233: the program driven tick by tick in tests
date: 2026-10-15
area: tooling, test
files: Cargo.lock crates/rugrats/tests/boot.rs crates/rugrats/src/app/mod.rs
started: 2026-10-15T18:24:23-06:00
took: 15m
---

# 73. retro_rt 5d20233: the program driven tick by tick in tests

retro_rt gained Tick::scripted and Tick::exiting (its worklog 15, commit 9c3f9d6, pushed as 5d20233), and the port is re-pinned to it. Before, a Tick could only come from rrt's loop, and rrt_app::headless needs a GPU, so the tests drove pieces of the app instead of the program.

- **tests/boot.rs** runs Rugrats::tick with scripted pads at 60 Hz from the legal screen, as the window does, with no window or GPU:
  - a_new_game_reaches_the_hub: the logos skipped, Start Game, Main Game, the Reptar Puzzle cinematic and its cards, then Tommy in Pickles' Home at tick 2400.
  - activities_open_cookie_race: Activities, Cookie Race, its cinematic skipped, the race in play. This is the path that crashed on the song bank before worklog 72.
  - Each tick checks that the program did not ask to end.
- **Rugrats::level_in_play()** reads the level in play, for the tests.

**Still unknown:**
- nothing
