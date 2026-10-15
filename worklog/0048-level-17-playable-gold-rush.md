---
number: 48
title: Level 17 playable: Gold Rush
date: 2026-10-15
area: play
files: crates/rugrats/src/level17.rs, crates/rugrats/src/game.rs, crates/rugrats/src/main.rs, docs/levels/level-17.md
started: 2026-10-15T11:09:54-06:00
took: 42m
---

# 48. Level 17 playable: Gold Rush

A delegated map, docs/levels/level-17.md, traced Gold Rush and checked
it in the oracle. The port now plays it.

**Framework:**

- `Script::cry_animation`: some ForceCry cases skip the cry (level 17
  as Phil; also 10, 11, 20, 21).
- `Game::fourth_bar`: a bonus level's fourth bar. Main adds it to the
  finish's bonus bars (0x80076ba8 for 17, 0x80076bf0 for 18, 0x80076c24
  for 13).
- Phil (char 5) is level 17's player; Game knows activity play.

**Level 17** (`rugrats::level17`):

- The round (0x80079870):
  - 62 nickels (template 446) on group 1's spots, the gold coin (1218)
    on one of group 0's four;
  - 170 s, song 0;
  - 609, 600, 117, 35 and 34 hidden.
- Each nickel counts down the "62" by icon 286. The 62nd sets the clock
  to one tick: it ends next frame.
- The gold coin shows icon 240 and earns the fourth bar.
- The clock out with fewer than 62 is a ForceCry. As Phil there is no
  cry: a try, cine 27, card 709, then Lil plays the same round.
- Lil's clock out is a cry and the hub. All 62 play cine 26 (back in
  Phil's body), won: 3 bars, 4 with the gold coin.

Checked in scripted runs: the start with the counter at 62 and 169 s;
the clock running out at frame 5100, cine 27, Lil's round in phase 2.

**Still unknown:**

- Activity play's way back to the Activities menu (the port goes to the
  title).
- Camera kind 8 in rooms 1-8 with +0x30 bit 8.
- The coins' +0xc0 handles; the tear particles.
