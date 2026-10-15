---
number: 61
title: Save and load pages, pause and title
date: 2026-10-15
area: save
files: crates/rugrats/src/card_menu.rs crates/rugrats/src/game.rs crates/rugrats/src/front.rs crates/rugrats/src/main.rs
started: 2026-10-15T16:13:48-06:00
took: 34m
---

# 61. Save and load pages, pause and title

The card pages are ported from memory-card.md "Saving and loading from the pause", which an agent traced and checked in the oracle.

- **rugrats::card_menu::CardMenu runs these pages:**
  - no card (4);
  - create the save file? (13);
  - the record list (22 save, 10 load);
  - overwrite? (23);
  - are you sure? (15);
  - saved (16) and not saved (17);
  - read failed (20) and create failed (21);
  - the load's no-file page (25).

  The format pages (5, 11, 12) and no room (14) cannot come up with the port's folder cards.
- **The record list:**
  - 5 rows by 3 columns, numbered down the columns, with no wrap.
  - Numbers sit at x -190/-45/110 and names at -160/-15/140, y -60 + 25 row, font 0. An empty record shows "---".
  - On the load list, the cursor's record gets a line at y 64 (strings 206-209: level name, pieces, bars mod 15).
- **Colours and sounds** (0x800391d4): items light blue, the heading pink, the cursor blinking ((255, 218, 0), then alternating every 13 frames). Cross plays 126; Triangle, Left and Up play 124; Right and Down play 125; Start closes.
- **Footer:** PRESS X while the footer flag is set, else CANCEL / ACCEPT.
- **Save** (0x80070b14): the whole state, with the hub's spot of the level (0x800b8718) at +0x24 and word 2 cleared, through put_record and Cards::store. An empty card gets the file first.
- **Load** (0x80070970): the record read again and returned. Main puts the state in place, sets the previous level to the saved one, and enters Pickles' Home at that level's door spot.
- **The pause:**
  - Game::state_block comes from main each frame, and Game::loaded goes back to main.
  - The pause takes the menu's colours, and its sounds now play the right way round (Left/Up 124).
  - The counters show on the pause page only.
- **The title's Load Game** uses the same menu in title mode (Cross and Triangle returns end it). Its old list is removed.
- **Checked:**
  - In a scratch working directory with an empty card folder: pause, save (create file, list, "Game saved."), then load (list with GAME 1 and the detail line, are you sure, loaded).
  - The real binary loaded from the title into Pickles' Home (screenshots).
  - play tests pass.

**Still unknown:**
- The burst behind the pages and the button cubes by CANCEL/ACCEPT are not drawn on these pages.
- The footer-flag quirks (PRESS X kept after some returns) are not reproduced exactly.
- The pause's Quit confirm is still two items, where 0x80036368 has four (Yes?, Save Game?, No?, Load Game?).
- The settings' restore at boot from the last record (0x80070c24) is not ported.
