---
number: 53
title: Speech: XA lines, talkers, ambience
date: 2026-10-15
area: audio
files: crates/rugrats/src/speech.rs crates/rugrats/src/ambience.rs crates/rugrats/src/game.rs crates/rugrats/src/main.rs crates/rugrats/src/screens.rs crates/rugrats-data/src/placement.rs
started: 2026-10-15T14:35:28-06:00
took: 11m
---

# 53. Speech: XA lines, talkers, ambience

Spoken lines are now ported. Before this, the port had none in play.

- **Index.** 0x8008d8e8 loads a database's index (file 585 + db). The layout is: u16 count, pad, then count u16 words. The low 12 bits of each word are the line id; the ids are sorted, and the lookup is a binary search (0x8008ddfc). The top 4 bits are flags. After the words, 4-byte records: u16 sector offset, u8 length in units of 8 sectors, u8 channel - 1. The XA itself is file 617 + db.
- **Playing a line.** 0x8008da4c seeks to base + offset - 8, filters file 1 and channel record+1, and stops at + length x 8. xa_headers shows DB16's file interleaving channels 1-8. Lines below 2040 set flag bit 1 once played, and (flags & 3) == 1 skips them, so they play once unless flag bit 2 is set.
- **One line at a time.** A request is dropped while the drive is busy (0x800c45d8 not 0 or 9). Main keeps this rule: a line asked for while the soundtrack plays is dropped and not marked as played.
- **Talkers.** The loader (0x80046d04) gives an object its LP +0x4c line; template 613 says 194. The enter callback is 0x8004d0f0. The trigger radius is template +0x34. A line is said when the player comes within that radius (2D) while the object is in the player's room, including at setup.
- **Ambience.** rugrats::ambience ports every callback of 0x800b9618 (levels 2-7, 9-12, 15-17, 20-21, golf) with the tables read from the executable. The setup countdown comes from 0x800b9598: 900, or 300 plus the first-line flag for 7 and 17. In activity play the callback is picked by character. Script::cue passes the script state the callbacks need:
  - level 2: phase 3;
  - level 3: the toss;
  - level 9: deaths to go;
  - level 11: ring step 9/5;
  - level 12: the fridge;
  - golf: the stroke.
- **Wired one-offs:**
  - level 16: slip 212 and cookie 216;
  - "You did it!" (2046), through Screen::speech.

Checked:
- The tables match the docs (0x800b9400 1110..., 0x800b94c0 114/24/204, 0x800b92f8 1148...).
- First lines against the oracle notes: level 7's 1181 at frame 30 (doc: 62 VSyncs); level 16's 24 at frame 90 (doc: 180 VSyncs); level 4 every 100 frames (doc: every 200 VSyncs).
- Decoded lines are 1.5-7.8 s with sane peaks; not listened to.
- No panics in any level.

**Still unknown:**
- The rest of the 66 call sites (event lines: barks, golf over-par, GoalPost 723, and others) are not wired. They need a call-site survey.
- Level 5's attacking robot and level 6's guard distance are approximated (any guard-mode NPC in range). The original uses the last NPC to think (0x800c443c / 0x800c4438, +0x674).
- 0x800c4618 / 0x800c45e4 = 900 after a line (guess: lip-sync time) is not ported.
- The volume of speech against the SPU music is not checked.
