---
number: 20
title: The title menu answers the pad, and the attract cycle runs
date: 2026-10-14
area: ui, input, play
files: crates/rugrats/src/front.rs, crates/rugrats/src/main.rs, crates/rugrats/src/input.rs, crates/rugrats/examples/stage_shot.rs, docs/engine/front-end.md
started: 2026-10-14T23:25:55-06:00
took: 2m
---

# 20. The title menu answers the pad, and the attract cycle runs

The title menu's loop is ported a frame at a time:

- Down or Select moves to the next item, Up to the one before, each with
  its sound (3, 4), and both reset the 30-second countdown.
- X picks the item: sound 2, or 5 when all four shoulder buttons are held.
  Load Game and Audio then play sound 2 a second time.
- Start opens the Start Game menu, but only while no item is sliding.
- Picking slides the items away, the second and third after 5 and 10
  frames, and then runs the action. The actions are not ported: the port
  logs them and rebuilds the menu, as if they had returned nothing.
- Triangle, or the countdown running out, ends the title.

The program now runs main's chooser cycle: title, then a demo movie
(3 + rand() % 3, skippable), then the opening, then the title again. rand
is the game's own LCG from its seed. The original's babies call rand every
frame and are not ported yet, so the demo picked can differ.

**Still unknown:**
- the Start Game menu, Load Game and Audio screens
- the menu's sounds
- the babies, whose rand calls the demo pick depends on
