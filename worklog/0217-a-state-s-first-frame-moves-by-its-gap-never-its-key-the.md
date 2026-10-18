---
number: 217
title: A state's first frame moves by its gap, never its key: the sweep's characters match in 14 levels; level 10's babies hide at the original's spots; guards bite by bit 8
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md, docs/levels/level-10.md
started: 2026-10-18T01:54:43-06:00
took: 1h 47m
---

# 217. A state's first frame moves by its gap, never its key: the sweep's characters match in 14 levels; level 10's babies hide at the original's spots; guards bite by bit 8

The characters' heights now match the original's frame for frame in the sweep. Level 10's babies hide where the original's do. The guard and follower leftovers from worklog 216 are settled.

**A state's first frame moves by its gap, never its key (0x80055904).** On a state's first frame the original adds the stored gap (+0xde) times the time gone, whatever the time. It sets the gap only when the frame begins with its whole time; the key counts only on the frames after. The settling (0x800452f8) clears the gap, so a character set up at load does not move across its first frame. A body attached to another (+0x00 bit 0x80000, 0x8008357c) takes the key alone as its gap.
- `RootCurve::set_up` takes the gap.
- A continuing root replaces the first frame's key by the gap in what it has done and in its whole.
- The tools sync the original's +0xde. Before, the root moved by the key there, which made the 1-unit drift in level 11's runner (worklog 202).

**The height sweep** (`npc_check`, YCHECK, 300 frames from frame 0) covers every playable level.
- No character is off in levels 1, 2, 3, 4, 6, 7, 10, 11, 12, 14, 15, 16, 17 and 18. Level 2's follower Chuckie was 74 off and level 10's hiders 3.
- Level 9's Mr. Friend matches from frame 20, after the original's respawn (the boot leaves its player bitten and crying).

The tool now also syncs:
- the player's place (the guards aim at it; the port starts level 6's Chuckie at cine 45's end, the boot elsewhere);
- the trigger pass's cursor (0x800c4c20);
- a bite's wait (+0x69c) and the events it waits on (+0xe0);
- its body after its place is final, so the body does not restart standing.
- whether each character is on the update list (+0x9c, 0x80072b48's node: the hub's EpisodeStart schedules Spike without bit 0);
- a held state as asked (the hub's sleeping Spike plays state 40, which the held 39 completes into).

**Level 10's hiding spots (0x8004fe14).** EpisodeStart (0x8007ee54) places Phil, Lil and Tommy in one call, group 0, a rand each, threshold 0 on the retries. `level_10_babies_hide_as_the_original` reads the original's rand as that call begins. It runs the port's start again from that rand and finds each baby at the original's spot.

**Guards attack by +0x688 bit 8.** No code sets bit 512: of the constants ORed into +0x688, only a mask has it, so the snap and mode 1 path is dead. A guard without bit 8 never bites. The port's guards now carry it as the original's do:
- the robots (0x80081bac case 33);
- the shadows (0x800a);
- level 6's guards and clowns (0x0a, 0x10a);
- the geese (10).

**Followers read their height before the body's step** (0x80084cd0 runs before 0x8005a068), as guards do since worklog 216.

**A set up's lift by channel.** 0x80055170 applies a key only on a channel the hierarchy maps (+0x14, not 0xff). The owed lift now keeps each key's channel and drops the unmapped ones.

**Still unknown:**
- Level 13's Angelica blocked at the start line (speed 17, not moving): the original's lift stays put; its handler drops the lift when the collision test (0x80066014, results 2 and 3 in 0x8005d978) blocks that way, where the port adds it.
- Level 5's robot 17 after the original's lag frame (dt 15): the tool's frame pairing shifts there.
- Levels 20 and 21's Angelica (600) is paired with an original object whose tracks are not a character's and that never moves; where the original keeps the box toss's Angelica is not found.
- The trigger pass's cursor at a level's start in play: the port starts at the front; the oracle's runs start wherever the boot left it.
- What the trigger list's other entries' enters do in the port (doors, speech lines, signs, batteries): each still runs by its own distance every frame.
- The crouch over an edge.
