---
number: 222
title: Levels 20 and 21 as the original: Angelica's lunge as state 28 stopped by what it meets, the knock's throw up, idle characters facing the player and walking home, followed things read where they are
date: 2026-10-18
area: engine
files: crates/rugrats/src/actors/npc.rs, crates/rugrats/src/actors/animator.rs, crates/rugrats/src/actors/player.rs, crates/rugrats/src/levels/box_toss.rs, crates/rugrats/src/world/game/mod.rs, crates/rugrats/examples/npc_check.rs, docs/levels/level-20.md
started: 2026-10-18T07:59:30-06:00
took: 2h 3m
---

# 222. Levels 20 and 21 as the original: Angelica's lunge as state 28 stopped by what it meets, the knock's throw up, idle characters facing the player and walking home, followed things read where they are

Levels 20 and 21's characters play as the original's. Angelica lunges as body state 28 and is stopped by what she meets. The knock throws the box up. The babies face the player and walk home. Followed things are read where they are. Level 20 matches on 297 of 300 frames, and level 21's babies on all 300.

**The lunge (0x80084df0, 0x8005f3a4).** Mode 4 lunges when the way to the target is clear, the target is within the range by the game's distance and +0x688 has bit 128. The speed is kept (+0x176), the mode is 8 and the body state 28 (animation 43 at 4096, sound 121 in levels 20-21), +0x688 |= 4.
- State 28 carries her by the animation's root across.
- The body's move test (0x80066014, now `Player::move_test`) stops her where she meets anything. Meeting the player is the knock.
- On the animation's end she is in mode 0 and gets her speed back. That frame's across is still had, and the body's state 0 entry asks the stand (0x80057274).

The port's lunge had been a 40-frame timer with a knock by distance (`Bite::Lunge`, `Event::Lunge`, `Event::Knock`, `Npc::lunges`).

**The knock (0x8005f3a4 with 0x80057bc4).** The original's knock goes:
- +0x30 |= 64;
- the player throws the box (animation 8) at the dummy (0x800e5778 at (22086, 775, 12864)) with the bit 64 lob, (0, -4096, 100), nearly straight up;
- the box's think ends that flight and posts ForceCry the frame after.

The port dropped the box and lost the round at once. A box resting on the floor while she lunges is lost (0x80057950).

**The lob in the original's integers.** The flat distance is the game's (0x8001f57c), where the port used the true one. The direction is (0, -2048, 2048) turned toward the target by its ratan2 (0x800760f4, 0x80075ee8) and made a unit (0x8007652c). The speed is the integer square root (0x8001f4ec) times 65536 / 6000.

**Mode 8 (0x80084fc4).**
- With +0x688 bit 2 a character walks home when more than half its range from it.
- Otherwise it is still, and with bit 64 its stick turns it to the player, playing the turn in place (state 63) as it turns (0x800820e0's +0x688 0x10000).

The port's `Idle` only stood. The setters (`Npc::faces`, `Npc::idle_home`):
- levels 20 and 21's babies (66);
- the Angelicas of levels 11 and 16;
- level 6's Chuckie after cine 46;
- level 10's caught hiders;
- every character 9 (0x80081bac).

**Followed things where they are.** A follower's target (+0x678) is read as it steps; box toss and the hub's Spike had set a place a frame old (`Target::Thing`, `World::things`). A thing in flight is back in its room's list, so the characters' tests meet it. Angelica's casts pass the player (+0x1f4, 0x80082b60; `Npc::ignores_player`).

**The root.**
- Each frame's part of the across is turned and rounded by itself, as 0x80055904 turns each (one unit at a frame boundary).
- A state set up on its first frame with its whole time begins on the next step, which takes the gap anew.

**The sweep tool** pairs each original object once and leaves a character without one unpaired. With `PLAYERSYNC` it runs the original's frame first and gives the port its player's place and what it carries. It syncs the lunge (mode 8, state 28), +0x688 bits 2, 64 and 128, +0x176, +0x1f4, and a follower's goal when it follows no player; `TOUCH` prints what a body met.

**Still unknown:**
- Level 20's three frames at a lunge's start (158-160): the original's heading turns by 352 then 160 a frame back, the port's not at all.
- Level 21 from its knock: the player's own throw moves it within the port's frame, which the tool's placing of the player cannot follow.
