---
number: 85
title: The cry's voice, the lobster's bite rate; walls do not slide
date: 2026-10-15
area: play
files: crates/rugrats/src/world/game/story.rs crates/rugrats/src/world/game/mod.rs crates/rugrats/src/app/mod.rs crates/rugrats/src/app/play.rs crates/rugrats/src/actors/npc.rs crates/rugrats/tests/play.rs
resolves: 52.5
started: 2026-10-15T21:14:13-06:00
took: 6m
---

# 85. The cry's voice, the lobster's bite rate; walls do not slide

Three items from the triage of open questions (the ones a player would notice most) were checked against the original.

- **Walls do not slide in the original either.**
  - Oracle run: in the hub from (5775, 19403), Up+Left held from frame 2700. Tommy reaches (4461, 23200) at the back door by 2900 and stays exactly there to 3300 with the stick still held.
  - The port stops him dead too, so nothing changes. 0x80061a64 and 0x80061b28, named as the slide, are the in-air wall hits: horizontal speed reversed and halved, air control off.
- **The cry's voice:** 0x8007f920 (not in level 1) calls 0x8008df78(2040 + character) for characters 1-5. 0x8008df78 seeks the CD and plays the line at once, cutting off any other. The port's cry was silent.
  - Game::say_now carries such a line, and the app plays it over whatever was said.
  - Test the_cry_has_its_line: Tommy's 2041 in level 2, present in its speech set.
- **The lobster's bite:** the oracle saw 15 every 36 VSyncs while the lobster stays on Tommy (level-16.md): mode 5, the bite's animation, then mode 1. The port's prowlers waited 900 dt after a bite; they now wait 180 (npc::BITE_EVERY).
  - Test the_lobster_bites_every_36_vsyncs: level 3's aisles, a bite each 17-20 frames.
  - The Aisle 7 win still passes with the faster bites.

**Still unknown:**
- The bite animation's own length per prowler (rats, robots): the port uses the lobster's measured 180 for all.
