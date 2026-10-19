---
number: 232
title: Level 5's robot bite, the cry and the respawn as the original's, frame for frame
date: 2026-10-18
area: play
files: crates/rugrats/src/world/game/story.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/outer_space.rs, crates/rugrats/tests/hle.rs, docs/engine/gameplay.md
started: 2026-10-18T18:11:22-06:00
took: 44m
---

# 232. Level 5's robot bite, the cry and the respawn as the original's, frame for frame

Level 5's robot biting Angelica, her cry and her respawn now play as the original's, frame for frame, over 400 frames.

**The scene.** `level_5_robot_bites_as_the_original` sets up:
- the oracle without lag frames, booted with its cinematics (cine 36's end sets the respawn, 0x8007b280);
- Angelica put 1200 short of robot 13000 on its floor;
- the robot synced, scheduled (+0x9c) and the trigger cursor set.

The robot sinks to her and bites, she cries at 0 and starts again at the respawn. Her health and her height, lift and all, are compared every frame.

Four differences came out:
- **The cry's lift.** The cry's start runs the body's update after asking animation 16 (0x8007f920), which puts that step's lift into +0x1c: she sinks 18. The port stepped the animator alone. `Player::step_animation` (the animator's step and the root's) is new, and `Game::cry` uses it.
- **The respawn a frame late.** The cry's loop returns after its last VSync wait, so the respawn and the rest of that frame come in the frame after its 2100 dt run out. The port respawned in that last frame. `Game::cry_over` holds the respawn to the next frame, which then plays on as any other.
- **The stand asked at the loop's end** (0x800572a8(player, 0, 4096)), so that frame's body update plays its first frame. The port asked it only in the stand's handler, after the step, a frame late.
- **No lift left after the respawn.** The respawn writes +0x18..+0x20 whole (0x80078d60). The port kept the cry's lift in `root_y`. The checkpoint respawn and level 5's own put now clear it.

The player's root also takes a state left only by what the step spent on it (`Animator::left_from`, as the characters' since worklog 223). A state asked anew between steps gives nothing, where the port gave the rest of any state left on its last frame, a cry held there too.

gameplay.md has the cry ("The cry (ForceCry's loop, 0x8007f920)").

**Still unknown:**
- The cry start's first body update (in state 0, before animation 16 is asked): the port runs only the second and matches here.
- The class 10 objects (0x8004e0f4 with the +0x50 table) and the sound sources' playing, from worklog 231.
