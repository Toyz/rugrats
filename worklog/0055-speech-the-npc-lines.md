---
number: 55
title: Speech: the NPC lines
date: 2026-10-15
area: audio
files: crates/rugrats/src/npc.rs crates/rugrats/src/level7.rs crates/rugrats/src/level9.rs crates/rugrats/src/level10.rs crates/rugrats/src/level12.rs crates/rugrats/src/level13.rs crates/rugrats/src/level16.rs docs/engine/speech.md
started: 2026-10-15T15:06:18-06:00
took: 3m
---

# 55. Speech: the NPC lines

The NPC speech sites from the call-site survey are now wired.

- **Wait lines.** npc::Event::Waits and Resumes report when mode 7 starts at a WAIT post (0x80083e2c) and when it ends because the player comes near (0x80083efc).
  - Level 13: Angelica says 0x800b96d0[rand & 7] when she waits, and 611/612 (0x800b96e0) when she goes on.
  - Level 16: 643/646/582 (0x800b96e4).
- **Mr. Friend** (level 9, char 16): his hit/kick line 0x800b96c8 (153, 187, 146).
- **Level 10:**
  - The tag says Chuckie's 158/160/165 (0x800b9728). The port played this as sound effect 158, which was wrong.
  - Running babies share the countdown 0x800c4ce4. Each one running counts it down; at 0 the speaker's table is used: Tommy 0x800b9704, Phil 0x800b9710, Lil 0x800b971c. Then (rand & 255) x 16.
- **Level 12:** a shadow coming back says 143 or 21. This was also played as effect 143 before.
- **Level 7:** template 931 breaking says 1183.

Tables read with exe_words: 0x800b96c8 (153 187 146), 0x800b96d0 (608 609 610 643 645 646 664 723), 0x800b96e0 (611 612), 0x800b96e4 (643 646 582), 0x800b9704, 0x800b9710, 0x800b971c, 0x800b9728 (158 160 165).

**Still unknown:**
- Level 13's resume line plays in the original only when the player is in the direction of the next post (angle under 768). The port says it on any resume caused by the player being near.
- Level 10's running lines are picked by id (1492/608/609), on the guess that s3 is the NPC itself.
- Level 26's prowler (mode 9: sounds 144/145 or speech 1134) and 0x800c4614's pending line are not wired.
