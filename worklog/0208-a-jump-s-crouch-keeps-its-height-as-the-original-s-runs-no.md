---
number: 208
title: A jump's crouch keeps its height, as the original's runs no ground step
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/player.rs, docs/engine/gameplay.md
started: 2026-10-17T19:00:09-06:00
took: 16m
---

# 208. A jump's crouch keeps its height, as the original's runs no ground step

The last guess in the port's code: what the jump's crouch does to the height.

**The crouch runs no ground step.** The ground step is 0x80064a34: the floor probe, the height followed to the floor and, for a drop past the limit, the fall (0x800645d8 to state 4). It is called only by these state handlers:
- the stand (0x8005d11c, kind 1);
- the walk and the run (0x8005e1f4, state 2's 0x8005ec7c calling it; kind 2);
- the slides (0x8005dedc, kind 2).

The jump's handler (0x8005ec9c, states 5, 16 and 26) does nothing until the animation sets +0xe0 bit 31. It then sets +0x188 bit 8 and runs the air (0x8005f1b4). So through the crouch the body's height is not followed and not dropped, whether an edge is passed or not, and the jump goes off from that height.

The port kept the height past a drop of more than 384 (the guess), but put the feet on the floor below a smaller one. It now keeps the height through the whole crouch. On a flat floor nothing changes; the play and oracle jump tests hold.

An oracle run off a ledge was tried (level 16's raised block, level 3's raised floor, the hub's 1024 step at (24100, 0, 1300)). It was dropped because the original's camera after a teleport does not follow as the port's does, so the stick sends the two different ways.

**Still unknown:**
- The crouch carried over an edge, compared frame by frame in the oracle (no ledge found whose camera the test can match).
