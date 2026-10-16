---
number: 90
title: Golf: the stance, the club, the aim line and Circle's camera
date: 2026-10-15
area: play
files: crates/rugrats/src/levels/golf.rs crates/rugrats/src/ui/hud.rs crates/rugrats/src/levels/box_toss.rs docs/levels/golf.md
started: 2026-10-15T21:53:33-06:00
took: 10m
---

# 90. Golf: the stance, the club, the aim line and Circle's camera

The user saw the golfers in T-pose with no club.

**The T-pose.** States 13-15 ask 0x800572a8 for animations 29, 30 and 31.
Like every argument of 0x800572a8, they are logical: 0x80056fe4 maps them
to the table's states 42, 43 and 44. The port played table states 29-31.
For Lil and Phil those name ANM 802-806, which no golf database (23-31)
has, so the body stayed in its bind pose. The golf constants are now 42,
43 and 44. They play ANM 762, 766 and 770, and the stance and swing
show.

**The club.** Each frame in the golf holes (0x8005a678) the hand part
(hierarchy +0x23) gets a model:

- in states 13-15, the club (0x80057404): Tommy 1069, Chuckie 1216,
  Phil 1215, Lil 1218, Angelica 1217, Susie 1219;
- otherwise the bare hand (0x80057468).

The table is keyed by 0x800c5680, which 0x8005928c sets to the hole's
baby. The port left `g.character` at 1 for every hole. It now follows
the baby, which also picks the right holing-out lines.

**The aim line** (0x8005f68c). It is five pool sprites (0x8008e36c,
drawn by 0x8008e508). The quad is D_800b6da8, (+-4096, +-4096, 0),
turned by Ry Rx Rz and scaled by diag(200, 250, 4096) / 4096
(0x80024424): half-size 200 x 250, flat on the ground, the texture's top
ahead. Flags 0xd2803000 make it a semi-transparent FT4. The five sit
250, 750 .. 2250 from the ball along the heading.

`Sprites::quad` now draws any world sprite on two axes, with a grey and
semi-transparency. The billboards and the box-toss arrow use it too.

**Circle's camera height** (0x800972c4, table 0x800c4730 = 0, -2200,
-6000, 1400). Each press in state 14 counts 1-3 and takes that height.
The fourth press returns to the base that state 14's entry saved
(0x80097288). State 15 resets the count and the height to 0. The height
is added to the golf eye's y. Shots show it higher, overhead, low, then
back.

**Still unknown:**
- whether the port's other animation constants (player.rs `anim`, e.g. RUN 2, JUMP 3, PICK_UP 13, CRY 16) are logical numbers played as table states; 0x80056fe4 maps logical 2 to 3, 3 to 5, 13 to 15, and 16 through code. Checking needs an oracle peek of the player's track (+0xa8) in a known state
- the golf camera's fixed view near the cup and activity golf's score card (35.2's rest)
