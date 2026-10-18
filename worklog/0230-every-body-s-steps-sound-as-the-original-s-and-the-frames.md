---
number: 230
title: Every body's steps sound as the original's and the frames' events are the set-up's; the sweep compares the sounds
date: 2026-10-18
area: play
files: crates/rugrats/src/actors/player.rs, crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/world/game/doors.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats-hle/src/bus.rs
started: 2026-10-18T13:47:16-06:00
took: 1h 45m
---

# 230. Every body's steps sound as the original's and the frames' events are the set-up's; the sweep compares the sounds

Every body's steps now sound as the original's. The frames' events are rewritten as the original's set-up leaves them. The sweep tool compares the sounds each side keys on, and they match in every level it can run but golf's door phase and level 21's last notes.

**The sound comparison (`npc_check` SOUNDS=1).**
- The original's key-ons are hooked at 0x800a99a8: program, tone, and the note at sp + 0x10. Every effect (0x8006f26c, 0x8006f644), the crunch and the piano pass through it.
- The port's effects are mapped through the original's table (0x800b84fc, 3 bytes a sound).
- A sound a frame later in the original still counts, since the trigger pass runs after the oracle's frame ends.
- SOUNDRA=1 prints the callers.
- The tool now syncs the game's rand (0x800c3f94) and the doors' cycles (+0xa8..+0xac, +0x3c bit 2, +0xcc, +0xd2: `Door::sync`) at its frame 0.

**The footstep (0x8007013c).** Out of water a step keys program 0, tone 0, note 22 + (rand & 7), drawing the game's rand only when heard (0x8006f4c0: within 0xfffe across and 0x7fff up of the camera, whatever the volume). The port's comment had it as a table that is empty in play, so nothing played.

The step's sound by path (0x8005e1f4):
- **Running** (0x8005e830), by the player's character (0x800c5680):
  - Spike in water wades (57 on every fifth step in level 4), plays 176 in level 16, else 27.
  - Character 34 plays 108.
  - Others play 27 in water, but the footstep in the hub.
  - Out of water it is the footstep.
- **Walking** (0x8005e920), by the body's own character and template: character 34 plays 108, Reptar (template 8) puffs then steps, the rest step.

`player::step_sound` holds the rule.

**The characters step too.** Their bodies run the same handlers. `Npc::stepped` records a step in `drive_body`. `npc::step` sounds it before the next character's update, drawing a heard footstep's note from the rand there, as the original's body update does. `Event::Step`, `Event::Wade` and `Event::Footstep` go to `Game::npc_steps`. `Npc::template` is new. `World` gains the player's character and the listener (the camera).

**The frames' events (0x800596e8, 0x800573b0).** The player's set-up rewrites events in the shared frame table every level:
- animation 9: frame 2 none, frame 3 a step (256);
- animation 5: frame 2 none;
- animations 232-235: frame 2 none;
- animation 234: frame 8 a grab (2048).

The landing's entry (state 10, 0x8005b2d8) sets 190's frame 3 to a step. The port read the file's events, so level 11's runner stepped twice a stride. `animator::set_up_events` and `landing_events` do it now. With them the frame tables equal the oracle's in levels 1, 6, 11, 16 and 24 (`npc_check` FRAMES=1).

**The sweep with SOUNDS** (FAST, 300 frames). Every NPC track still matches.
- Level 11: all 40 sounds the same.
- Level 13: all 50 the same.
- Levels 1-7, 10, 12, 14-18, 23, 25, 27, 28, 30 and 31: no sound either side.

**Found, not yet done.** The generic object set-up (0x8004691c) changes the shared templates and models, as `npc_check` TEMPLATES=1 shows against the oracle's table (0x80118a0c). The golf door (770) runs at +0x1a 50 in level 24 where the file says 32, so its cycle is 72 frames, not 94. The next worklog takes the whole function on.

**Tools.** The oracle's bus stops on a write to a word (`RUGRATS_STOPW=addr[:was]`). That is how the golf door's 50 was traced to 0x80047418.

**Still unknown:**
- Level 21's footsteps after frame 7: the notes drift from the original's as the rand does.
- The golf holes' door sounds (24, 26, 29) until 0x8004691c's changes are in.
- Whether characters' landings set animation 190's frame (the port does it for any body's).
