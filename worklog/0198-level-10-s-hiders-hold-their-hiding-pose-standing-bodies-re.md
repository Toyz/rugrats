---
number: 198
title: Level 10's hiders hold their hiding pose, standing bodies re-ask their after state, the box toss idle and level 12's lone Tommy match the original; the oracle's tools fixed
date: 2026-10-17
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/levels/chuckies_glasses.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats-hle/src/play.rs, crates/rugrats/examples/npc_check.rs, crates/rugrats/tests/hle.rs
started: 2026-10-17T10:58:31-06:00
took: 2h 14m
---

# 198. Level 10's hiders hold their hiding pose, standing bodies re-ask their after state, the box toss idle and level 12's lone Tommy match the original; the oracle's tools fixed

The NPC sweep (`npc_check`, every level with characters, past the opening cinematics, the port's characters given the original's state once) now finds no character off the original in levels 2, 3, 4, 5, 6, 7, 10, 11, 12, 15, 16, 18 and 21, but for the frames the original lags. Most of what it had found were faults in the oracle's tools; four were the port's.

**The port.**
- Level 10's hiders (0x800838b4(baby, 46, 2048)): mode 1 with back mode 1 and the hiding animation (logical 46, table state 56) at half speed, not standing. `Mode::Act`: mode 1 as that call sets it, the animation played where it stands with the body updated and no step; at the base's end (+0xe0 0x8000000) mode 0 for a frame, then the back mode. The tag (0x800831a4) is the use of a hider in it. Their stick turns them to +0x67c, which nothing set for them: stale words, a heading they have long come to; the port keeps theirs.
- A standing character (0x8005d11c, not turning in place) asks the state after its base (+0xe6, 0x80057274) again from its first frame each frame its base is on anything but the idle (0), a fidget (23, 24) or 60 (`Animator::stand`). Elsewhere that is the idle asked once; in the box toss levels the after state is 35 (table 47) and its base starts over every frame while the upper tracks run on, as the original's babies do.
- 0x80082478, a character's first mode: in levels 20 and 21 the after states are 35 and 17 and the tracks are set up on table state 47 without an ask (`Animator::set_up`, 0x80055f7c); elsewhere 0 and 17 on state 0.
- Level 12: the loader (0x800454dc) puts the player on the record of Tommy's own hierarchy (1), before his body becomes 44. The port looked for 44, found none, and left that record (id 1492) as a second Tommy standing where the player starts.

**The tools.**
- `Boot::character` was set at the first level load only. With the cinematics, play mode's activation sets the character again from the hub's level (0x80018784, 0x80018598(1) = Tommy) after it, so levels 6 and 10 ran with Tommy as the player. It is now set at every level load of the boot.
- The player's own body (0x800e2b48) has the id 1492 as Tommy has; pairing by id took it for Tommy. It is left out (`npc_check`, `sync_npc`).
- The state given a character names the logical animation its base was asked as (`animator::logical_of`): never 17 (which is any state), a second half (2, 4, 6, 18, 35, 46) as its first, a fidget as standing, and its after state (+0xe6) playing as standing.
- A frame the original takes three VSyncs over (its time, 0x800c3e98, stands one frame and goes 15 the next; dt 15 where the port has 10) is not counted, and each pair is given the original's state again after it. `CALLS` shows 0x80057214 and 0x800572a8 with their arguments; `MOVES` the body's flags (+0x188) and held stick (+0x1b0).

**Still unknown:**
- Level 10's hiding spots: the port's rand at EpisodeStart is not the original's (every draw since the front end would have to match), so the hiders draw other spots.
- Turning in place while standing (0x80059204, animations 52 and 53): no port character does.
- Level 20's Angelica: her lunge depends on where the player is, which `npc_check` does not give the port; no oracle script plays the box toss.
- The original's lag frames: the port always steps dt 10.
