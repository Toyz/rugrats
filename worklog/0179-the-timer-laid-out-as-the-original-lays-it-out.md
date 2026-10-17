---
number: 179
title: The timer laid out as the original lays it out
date: 2026-10-17
area: ui
files: crates/rugrats/src/world/game/mod.rs
started: 2026-10-17T00:15:49-06:00
took: 3m
---

# 179. The timer laid out as the original lays it out

Worklog 178 left the timer text at a place read off a screenshot. It is now laid out as 0x8007f5a8 does it:

- the seconds ("%ld" of dt / 300) in font 1, spacing 1, centred across x = 0 by 0x8002e144 (flag 1), the port's `Fonts::centre`;
- y = D_800c4428 + (D_800c4a06 * 171 >> 11). Read from the oracle in levels 6 and 18: -110 and 240, so y = -90 (`TIMER_Y`);
- the shadow is the same text 2 right and 2 down.

The clock sprite is made only once, while D_800c464c is zero, at the text's corner then plus (20, 13). After that it stays put while the number's width changes. 0x80078df8 frees it and clears D_800c464c when the timer stops. The port keeps `Game::clock_at` the same way: set by the first text drawn, cleared when `timer` is none.

Level 18 at the same frame: the number's rows match the original's (32 to 50 of 240), and the clock is within a few pixels of it.

**Still unknown:**
- The original's clock was made with the number its first frame showed. The oracle's boot runs about 31 s into the level, so this comparison cannot show whether the port's first number, and with it the clock's x, is the same.
