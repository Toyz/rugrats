---
number: 66
title: Audio settings kept: restored at boot, kept by a new game
date: 2026-10-15
area: save
files: crates/rugrats/src/save/progress.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/app/flow.rs crates/rugrats/tests/card.rs
started: 2026-10-15T18:03:26-06:00
took: 2m
---

# 66. Audio settings kept: restored at boot, kept by a new game

The Audio menu's settings now behave as the original's (docs/engine/front-end.md).

- **At boot**, 0x80070c24 (called from show_still while the boot stills show) tries both cards. 0x800702d8 copies the last record saved (word 3, its bit in word 1) at +0x2c/+0x30/+0x34 into the state's +0x0c/+0x10/+0x14. Progress::restore_audio does the same over the Cards, and the app calls it before the legal screen.
- **A new game** no longer resets them. 0x80091a44, the new game's reset, clears only +0x40..+0x46 and the two bar arrays; the volumes stay. Before, the port replaced the whole state with Progress::default(), so Start Game or Activities after the Audio menu put the volumes back to 90/100. Progress::new_game keeps the settings.
- **Test** (tests/card.rs, settings_come_back_at_boot): a record with music 40, effects 70 and mono, stored on a scratch card, comes back at boot and survives new_game. No card gives nothing.

**Still unknown:**
- Whether 0x80070c24's card checks (0x8007047c, 0x80070598) differ from the port's "file present and whole" for a card with a damaged save.
