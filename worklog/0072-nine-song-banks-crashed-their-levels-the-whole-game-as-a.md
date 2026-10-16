---
number: 72
title: Nine song banks crashed their levels; the whole game as a test
date: 2026-10-15
area: bug
files: crates/rugrats-data/src/vab.rs crates/rugrats/src/app/play.rs
started: 2026-10-15T18:20:03-06:00
took: 4m
---

# 72. Nine song banks crashed their levels; the whole game as a test

A test that plays the campaign through the app's own flow found a crash that every earlier test missed.

- **The bug:** nine VH banks list programs up to 14 with one tone each, past the header's programs_used:
  - CIRCUS, LIGHT, RACE, MIRRLAND, GLDRUSH, EASTER, TCHDOWN, COOKIES, ACTEGG (scanned over every VH on the disc).
  - Only the first programs_used have tone blocks behind them, so Vab::parse sliced past the VH's end and panicked.
  - The app loads a level's song set when it opens the level, so entering Circus Angelicus, Let There Be Light, Cookie Race, Mirrorland, Gold Rush, Egg Hunt, No More Cookies or Touchdown Tommy killed the program. The headless play tests never load sound, so none of them hit it.
- **The fix:** Vab keeps only the first programs_used programs. rugrats-data's every_bank_parses checks every bank on the disc against its header's tone count.
- **The whole game as a test** (app::play::tests::the_whole_game_from_new_to_the_credits), on the real disc through Rugrats::enter / exit / after:
  - A new game, Reptar Puzzle's first piece, then each main door in turn (2-6, golf, 9-12).
  - Each level is won through its ending cinematic (Game::cine_finished); golf goes hole by hole to the ninth.
  - The bonus levels open as the count says (No More Cookies, Mirrorland, Touchdown Tommy, Gold Rush) and are won the same way.
  - The board is filled as the pages come, and the twelfth piece leads into Reptar 2010. Reptar Dance (cine 85) ends the game, and the credits open.
  - It passes after the fix; before it, it panicked opening the first of those banks.
- **What the test does not do:** play the levels' own goals (those are in tests/play.rs and the levels' play_script checks). Each level is opened, run through its win, and left as the game does.

**Still unknown:**
- What libsnd does with those extra programs (probably nothing: no tone blocks).
