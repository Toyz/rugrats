---
number: 114
title: Effect flights and animated sprites: flames, explosions, debris, splats
date: 2026-10-15
area: render
files: crates/rugrats/src/world/flights.rs crates/rugrats/src/ui/hud.rs crates/rugrats/src/levels/reptar_2010.rs crates/rugrats/src/levels/pie_toss.rs crates/rugrats/src/world/mod.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/tests/flights.rs docs/engine/objectives.md docs/levels/level-07.md
took: 40m
---

# 114. Effect flights and animated sprites: flames, explosions, debris, splats

Delegated to a single agent and checked here. The effects the original
throws as flights now exist in the port: Reptar's flames, the explosion
and debris when level 7's buildings break, and the pie splats. Details
are in docs/engine/objectives.md ("Effect flights", "Animated sprites").

**Animated sprites** (`hud::Sequences`). There is one file for all
databases: DB00/TEXACTRL.DAT (file 551), loaded once by 0x8008946c. It
holds 36 sequences, ids 0x1000 | n. Each has a frame time in ms, a loop
flag, its last frame and a list of TEXPOS entries. Ported:

- a pool of 32 running records;
- start (0x8008a22c), shared start (0x8008a2a4), activate (0x8008a154)
  and release (0x8008a1b8);
- the step (0x8008a358 / 0x80089bb8). A record keeps its own copy of
  the TEXPOS entry and changes it frame by frame; one that does not loop
  stops on its last frame.

`hud::face` draws a quad with any colour, semi-transparency (0x02000000)
and the page blend (flags & 0x600000). `Sprites::quad` goes through it.

**Effect flights** (`world::flights`). A pool of 64.

- Records come from a launch descriptor (0x8008864c) with the per-kind
  setup (0x80086114).
- The step (0x8008851c / 0x80087c5c) covers:
  - delayed starts, growth, gravity, spin and lifetime;
  - the hue / saturation / value fade (0x8001f238 / 0x8001f360);
  - world collision through the segment test (0x80087594's world part,
    with bounce and floor stop).
- Kinds 3 and 5-11 draw as camera-facing quads (0x80088388 /
  0x8008afc4 / 0x8008a954), up to 64 a frame.
- Also ported: the random cone (0x80088bac / 0x80085f44) and asin, acos
  and atan2 (0x80021460, 0x80021500, 0x800a0328), with the shared seeds.

The spawners are the flame (0x8005cd74, from the head bone as 0x8005ce74
computes it), the explosion (0x80053d28), the burst (0x80053e10), debris
(0x8005202c) and the splat (0x8005282c). Building breaks get debris
where the template's touch byte is 14, and an explosion. Template 795
explodes when taken. The splat bits moved here from 94's stand-in; kind
5's colour is 255.

**Checked against the oracle:**

- the real descriptors and records, dumped by hooks on 0x8008864c and
  after 0x80086114;
- the flame's and the explosion's position, life, size and sequence
  frame by frame, from per-VSync peeks (pinned in tests/flights.rs, 11
  tests);
- the cone and the trigonometry against the original functions;
- the flame's start and direction from the oracle's bone 16 matrix;
- the two lamps broken at the start exploding at the oracle's
  positions.

Two fixes on the way:

- Reptar's start heading after cine 79 is 0, as the oracle shows; it was
  2048.
- level-07.md had the flame's 1000 as its speed: it is the damage. The
  speed is 70, 750 the radius, 1500 the life.

After the agent, here: Reptar's half speed (113) makes the fire last as
long as the oracle's, and the fire test waits for it. The empty
`Splats` placeholder left in the pie toss is gone from its API.

**Still unknown:**
- object collision for flights (flag 0x20000), the room lookup and clamp at launch, hit callbacks and their sounds
- kinds 1, 2 and 4 (models, not quads); the screen-edge cull and four-quad split
- the shards (0x8004cdfc) when templates 401 and 795 are taken
- descriptor fields the original leaves as stack garbage (a delay, the debris and splat spin axis): 0 here, so those sprites do not roll
- the callers not yet wired: collectible sparks 0x800514b0 (every level), the use-callback burst 0x80054894, level 6's sparks 0x80054ae0, 0x800529cc, the hit table 0x800b7eb0 into 0x80052b60
