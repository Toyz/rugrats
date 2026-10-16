---
number: 70
title: Triage: open questions answered by later work
date: 2026-10-15
area: tooling
files: WORKLOG.md
resolves: 17.2, 17.3, 18.1, 18.2, 18.3, 19.1, 20.1, 20.2, 31.1, 36.1, 36.2, 36.4, 38.3, 48.1, 52.1, 52.2, 54.1, 56.3, 58.2, 59.1, 59.3, 60.2, 61.4, 62.2
started: 2026-10-15T18:15:14-06:00
took: 1m
---

# 70. Triage: open questions answered by later work

Questions still listed as open whose answer is already in the port, with where:

- **17.2, 17.3 (where play saves, the load path):** 61, the save and load pages from the pause and the title.
- **18.1-18.3, 19.1, 20.1, 20.2 (the title's babies, submenus, input and sounds; Start Game, Load Game, Audio):** ui::front, with 60 and 61.
- **31.1 (the save and load menus):** 61.
- **36.1, 36.2, 36.4 (loading screens and notes, "Returning to Main Menus", Select in activity play):** screens::message::loading / returning, and 59 B5/B6.
- **38.3 (the float's cameras):** commit 489ed69, world::game::rooms (cam_hold, the float's posts, the shaft camera).
- **48.1 (activity play back to the Activities menu):** app::play's exit, "Returning to the Activities Menu".
- **52.1 (training speech), 52.2 (message 24 on a door swing):** 53, and doors.rs, which shows message 24.
- **54.1 (hinged and sliding doors):** 56.
- **56.3 (class 8 swinging doors, golf's door 770):** 57, and doors.rs's template 770.
- **58.2 (the beam's reflection):** 65.
- **59.1 (golf player count and picks):** 62.
- **59.3 (B8 save record menu, B9 Audio menu, B11 float cameras, B12 moving toys):** 61, 60, commit 489ed69, and commit 15e4614 (class 2 spinners, class 15 post walkers).
- **60.2, 61.4 (the settings' restore at boot):** 66.
- **62.2 (the Loading Hole frame and player 1's banner):** 67. The count's wobble moves to 67's list.

**Still unknown:**
- nothing
