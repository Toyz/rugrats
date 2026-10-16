---
number: 74
title: Gold Rush and Egg Hunt won by walking onto the pickups
date: 2026-10-15
area: test
files: crates/rugrats/tests/play.rs
started: 2026-10-15T20:12:09-06:00
took: 1m
---

# 74. Gold Rush and Egg Hunt won by walking onto the pickups

Two play tests win levels through the game's own rules, not their ending cinematics. tests/play.rs collect() puts the player beside each pickup and walks him onto it. The level's touch, counter and clock then run as in play.

- **level_17_gold_rush_is_won:** the 62 nickels (template 446) taken; the last ends the clock and cine 26 (won) plays.
- **level_18_egg_hunt_is_won:** the 35 eggs (template 496) taken; cine 24 (won).

**Still unknown:**
- nothing
