---
number: 52
title: Level 16, training, playable
date: 2026-10-15
area: play
files: crates/rugrats/src/level16.rs crates/rugrats/src/training.rs crates/rugrats/src/player.rs crates/rugrats/src/game.rs crates/rugrats/src/level3.rs
started: 2026-10-15T13:43:31-06:00
took: 52m
---

# 52. Level 16, training, playable

Training is ported: rugrats::level16 plus rugrats::training, the message system that level 1 shares.

- **Messages.** Each of the 36 strings (237-272) shows once per level start. While one shows, the player and the world are frozen and the level timer stops; the camera and drawing keep running. A message closes after 1500 dt or on Cross (sound 126). One asked for while another shows is dropped, so it can show next time. Text is font 1, light blue, with a shadow, centred. Level 1 uses font 0 at y -105, without a freeze.
- **Hooks:**
  - player: 2 (run), 1/16/29 (climb, too tall, not climbable), 32/30 (slip), 31 (long fall);
  - carrying: 5, 6/27, 8;
  - collecting: 3, 7;
  - bites by the lobster: 13;
  - the pie toss start: 28;
  - use on templates 111/441: 34, and on 117/715/720: 33. In training, 111/441 are not doors.
- **Pie toss.** Room 4 runs Gameplay script 3. It shares level 3's pie code (Pie::thrown, roll_pies, draw_pies). Tommy goes to the counter, frozen but able to turn. The fixed camera is 4000 west and 2240 up, and the clock runs 30 s. One Triangle throws one pie. Five hits win; the clock running out ends the toss. At the end Tommy is put at (31000, -1781, -8146). Room 4 keeps flag 64, so the toss starts again on every entry; Game::set_room keeps the counter in room 5.
- **Cry.** There is no cry animation; health goes back to 255 and nothing else happens.
- **Exits.** The door in room 6 goes to the hub. Pause, Quit Level returns to the title (already in place).
- **Lobster.** It prowls posts 10-19, bites for 15, and a thrown can kills it.
- **General changes:**
  - Slipping on floor flag 2: col_flags shows flag 2 only in levels 3 and 16. The slip uses anim 33 and does Damage 5 at its end.
  - Fall damage: 5 on a landing after 193 dt in the air.
  - Door use height: measured to the object's model span, because training's door is hung at y -3092.
  - Effects: now cleared at the start of each frame. Before, the player's effects overwrote the collect sounds.

Checked with play_script:
- message 2 at the start, with a screenshot of the text;
- the toss: 28, then 27, five hits won at frame 785; the timeout after 30 s of unfrozen time; the restart on re-entry;
- the door to hub (1, None);
- the slip: 32, then 30, then health 0 -> 255 with no cry;
- lobster bites with message 13.

**Still unknown:**
- Speech is not ported: the ambience lines 114/24/204 and speech 212 on the slip.
- Message 24 (a door swing) has no hook, because the port has no swinging doors.
- Fall air time is counted on the way down only (a guess). The whole time in the air would hurt a held jump.
- Angelica waits at her WAIT posts (2, 3, 5, 6, 9) for the template's wait time. The oracle shows her running.
- The lobster bites every 900 dt (mode 1, as in level 3). The doc saw about 36-51 VSyncs.
