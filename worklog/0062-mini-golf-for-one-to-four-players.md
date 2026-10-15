---
number: 62
title: Mini-golf for one to four players
date: 2026-10-15
area: play
files: crates/rugrats/src/golf_setup.rs crates/rugrats/src/golf.rs crates/rugrats/src/front.rs crates/rugrats/src/screens.rs crates/rugrats/src/main.rs
started: 2026-10-15T16:47:42-06:00
took: 31m
---

# 62. Mini-golf for one to four players

Multiplayer mini-golf is ported from the golf.md activity section, which an agent traced and checked in the oracle.

- **rugrats::golf_setup (0x80096134):**
  - The count, 1-4: Left/Right with sounds 3 and 4, Cross with 2, Triangle back to Activities with 1.
  - Then the picks: Cross takes the face (col + 3 row). Triangle undoes every pick, back to the count, with no sound. 0x800960a0 moves the cursor past taken faces.
  - Drawn: "Mini-Golf" (87) red at y -100; "How Many Players?" (86) yellow; "1".."4" at x -160 + 100 n (the count lit 0xf0f0f0, the others 0x404040).
  - On the picks: "One player" / "%s players"; "Player %d, pick your character!" in the player's colour (0x800bb494).
  - The heads are DB00 models 2, 17, 32, 64, 49, 1173 at (-120 + 130 c, 25 + 60 r), depth 5500. The arrow is TEXPOS 373 from the menu's TEXPOS; the star is 427, behind the head, with the number. CANCEL/ACCEPT are drawn with their cubes.
- **The round** (main's GolfRound, 0x80096a30):
  - Slots, strokes per player and hole, and the hole and player in play.
  - The first player gets the level's start (cine 31 on hole one).
  - A holed ball in activity play posts the same level (SwitchEpisode). Main records the strokes, gives the next player the turn on the same Game, and shows the "Player %d / Hole %d" banner until Cross.
  - After the last player: the score card with PRESS X, then the next hole. After hole nine the card, then the Activities menu.
  - A pause quit clears the round.
- **golf.rs:**
  - Golf::activity_turn: the pick's body (all six preloaded), the tee, strokes 0, stage Free, the ball back at the tee, the camera behind.
  - "Player %d" (-230, 60) and "Hole %d" (-230, 80), font 0, in the player's colour.
  - In activity play only holes 0 and 8 hide the other babies.
  - Script::activity_turn is the hook.
- **Page::Scores:** rows of (face slot, strokes), each with the player's head at (-200, -20 + 30 p), depth 7000.
- **Checked:** the real binary, driven headless: Activities, Mini-Golf, two players, Tommy and Angelica picked, the round began (cine 31, Tommy at the tee, "Player 1 / Hole 1" in red). The score card was rendered with screen_shot. A full turn change was not driven (it needs a holed ball).

**Still unknown:**
- The heads' size (scale 0.54 to match the oracle's 51 px) and their orientation (a quarter turn about x, by eye) are approximations. The original's GTE H and matrix for them are not traced.
- The count's wobble, the "Loading Hole %d..." frame, and player 1's banner from hole two on are not ported.
- The second-turn path is untested end to end.
