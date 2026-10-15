---
number: 31
title: Pause menu, collectibles, progress and saving
date: 2026-10-15
area: play, save, ui
files: crates/rugrats/src/game.rs, crates/rugrats/src/progress.rs, crates/rugrats/src/card.rs, crates/rugrats/src/front.rs, crates/rugrats/src/main.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/disc.rs, crates/rugrats/src/lib.rs, crates/rugrats/src/player.rs, crates/rugrats/tests/card.rs, crates/rugrats/examples/level_shot.rs
started: 2026-10-15T02:08:35-06:00
took: 6m
---

# 31. Pause menu, collectibles, progress and saving

The level is now something to play and come back to.

**Pause** (0x8003e05c, 0x800398bc mode 0, laid out by 0x80035fec):

- "Game Paused" in font 1, over Continue?, Save Game?, Quit Game?,
  Load Game? and Quit Level? in font 0 at the original's positions.
  The strings are 1-6 of the table at 0x800bb4e8.
- Left/Right move by 2 and Up/Down switch rows, with sounds 124 and
  125. Cross picks (126), Triangle or Start continues.
- Quit Game and Quit Level ask again with "Yes?"/"No?" (0x800c48f0,
  0x800c48ec), No first.
- Quit Game goes to the title. Quit Level goes to the hub, or to the
  title from the hub and training.
- Save Game writes the card (below). Load Game in play is not ported.
- The fonts are now in every level's VRAM at (960, 256), where boot
  leaves them for good.

**Collectibles.** Template class 12 (0x80049af4) makes them through
0x80048a44 (object class 11), the kind from template +0x26.

- Touching one within its model radius plus the player's takes it, with
  its kind's sound from 0x800514b0's cases.
- Each level has seven Reptar bars (kind 3), numbered in the order they
  are made (0x800c4ba8), plus a cookie or two (kind 4).
- A bar shows the count at (-150, -80) for 1500 dt.

**Back in the hub** after a level, the player stands at
0x800b8718[level] (Grandpa's Teeth: (37002, 779, 9866)), as level 1's
EpisodeStart callback places it.

**Progress** (`rugrats::progress`, the 328-byte state at 0x800c5678):
the level and the one before, character, volumes, played, completed,
pieces, and bars found and spent per level. A found bar sets its bit
(0x80094a58) and stays taken on later visits.

**Saving** goes through the 1:1 card file in work/memcard/slot1.

- `Save::put_record` writes record k: "GAME k", the state at +0x20,
  the used bit in word 1, last-saved in word 3, then the sums
  (0x8007062c).
- Pause's Save Game writes the record last used, else 1.
- The title's Load Game reads it back and starts in the hub.
- A test round-trips a state through the record and the file's bytes,
  with the sums checking.

**Still unknown:**

- The save and load menus (record choice, overwrite prompts). The port
  uses the last record.
- Whether the original hides found bars or makes them again
  differently.
