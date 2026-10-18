---
number: 224
title: The box toss's lunge steers, the box rides the hand without a grip, the knock lobs at the aim the babies leave, and the player's root across turns by frame
date: 2026-10-18
area: play
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/src/world/game/things.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/examples/npc_check.rs, docs/levels/level-20.md
started: 2026-10-18T11:41:59-06:00
took: 1h 19m
---

# 224. The box toss's lunge steers, the box rides the hand without a grip, the knock lobs at the aim the babies leave, and the player's root across turns by frame

The box toss's Angelica and her knock now play as the original's. Level 20 matches on all 300 frames. Level 21 matches up to its loss: the lunge, the box in hand, the knock's lob, the player's push and the babies turning to it.

**The lunge steers (0x80084df0, 0x80082264, 0x8005bc80).**
- The frame she lunges, mode 4 goes on to the follow's speed by the distance (0x80084f28) and the stick. The lunge's +0x688 bit 4 makes the stick skip its turn (0x800820e0). With a speed, its way round (0x800825fc) still turns her 512 to the open side.
- Through the lunge the body turns back toward the stick's last heading (+0x2c, now `Npc::aim`) at its rate, 160 a frame. Level 20's frames 158-160 had the original turn 352 then back by 160 a frame, while the port held still.
- Mode 8 keeps the start frame's speed in +0x176 for the lunge's end (0x80085110).

**No grip in the box toss (0x80079cb0, 0x8007a094).** The round start and the body swap set +0x1f0 and place the box (0x8005ff88 mode 1), but never take a grip (mode 0). +0x1f8 stays 0 and the box rides hand 15 itself. The port took a grip from the pose, so the box sat about 330 off the original's; `Game::hand_over` now puts it in hand without one.

**The knock's aim (0x8005f3a4, 0x80059b34, 0x80057bc4).**
- The knock makes the dummy the player's target (+0x1f4). Each baby updated after Angelica runs the aim test and takes +0x1f4 back when it is in view and nearer, keeping the lob (+0x200) toward itself. In level 21 the box went up toward Phil.
- The dummy's place is the level's: (22086, 775, 12864) in level 20 and (6825, 775, 10814) in level 21 (0x8007a094). The port had level 20's in both.
- New: `Game::aim_dummy`. `consider_aim` compares against the dummy as against a held character.

**The player's root across turns each frame's part by itself** (0x80055904 → 0x80075ee8), as the characters' have since worklog 222. The knock's push back (animation 8) was a unit off.

**The sweep tool** (`npc_check`):
- PLAYERSYNC gives the carried thing its place as the original's frame began. The hand puts it there at the draw, after the characters aim.
- PLAYERBEFORE gives the player its place as that frame began, and the port's step moves it (a knocked player pushed back).
- PLAYERANIM gives the player the original's animation then.
- A cine begun ends the comparison, as an exit does: the app takes it after the frame, and the original's world waits on it.
- The tracks reading is shared (`record`).

**The sweep** (FAST, YCHECK, 300 frames):
- 0 off in levels 1-7, 10-18, 20 and 21 (to its loss's cine 51 at frame 38) and holes 23-31.
- Level 9 matches from its sync at frame 20.

**Still unknown:**
- The view test's push from inside an object's box (the stale stack normal, worklog 221).
- Whether the real console lags where the oracle's instruction budget does.
- The order of the characters' updates against Angelica's in the frame of a knock: the port lets every baby run the aim test after it.
