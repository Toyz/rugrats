---
title: The memory card and the save file
status: partial
discs: US
covers: US SLUS_006.50:0x80072730, US SLUS_006.50:0x8007146c, US SLUS_006.50:0x80071ee0, US SLUS_006.50:0x80070e84, US SLUS_006.50:0x80070fa0, US SLUS_006.50:0x80072498, US SLUS_006.50:0x80072320, US SLUS_006.50:0x800721a4, US SLUS_006.50:0x8007062c, US SLUS_006.50:0x80070234, US SLUS_006.50:0x800707a8, US SLUS_006.50:0x8009ac98
worklog: 17
---

# The memory card and the save file

## The card layer (0x800702d8 - 0x80072a68)

`card_init` (0x80072730) opens eight BIOS events: the BIOS's card class
(0xf4000001) and the hardware's (0xf0000011), each for done (4), error
(0x8000), time-out (0x100) and new card (0x2000). A port is numbered 0
or 1; the BIOS channel is `(port & 1) << 4 | port >> 1`.

`card_status(port)` (0x8007146c) calls `_card_info` and waits for a
hardware event: done gives 2 (ready), error 8, time-out 1 (no card), new
card 18, nothing 32. A new card, or a port whose bit in 0x800c4c0c is set,
is reloaded (`_card_load`, 0x80071630) first. `card_formatted`
(0x800716a0) reads sector 0 raw and wants "MC".

The file calls go through `bu%d%d:` paths with the BIOS's open, read,
write, lseek, close, erase and format. libapi's `firstfile` (0x8009ac98)
first looks the device ("bu") up in the kernel's device table (0x150,
0x154) and patches its search function. Reads and writes go in whole
frames: the last partial frame passes through the work buffer at
0x800e5228 (0x80071b38, 0x80071c8c, 0x800721a4, 0x80072320).

## The file

One file, `BASLUS-00650Games   ` (`bu%d%d:%2s%10s%-8s` with the region
"BA" at 0x800c4394, "SLUS-00650" and "Games"), made one block long
(0x80072498, `open` with 0x10200).

```
+0x000  title frame (0x80071ee0)
        "SC", 0x13 (three icon frames), blocks (1)
        +0x04  the title, 64 bytes: "RugRats Games" in full-width
               Shift-JIS (0x80070e84 per character: digits from 0x824f,
               capitals from 0x8260, small letters from 0x8281, the rest
               from a 33-entry table at 0x800b8b64; 0x80070fa0 fills with
               0x8140 first)
        +0x60  the icon CLUT (0x800b8aac)
        +0x80, +0x100, +0x180  three icon frames (0x800b8958,
               0x800b8a18, 0x800b8ad8)
+0x200  the save, 0x1590 bytes
+0x1790 the rest of the save's last frame, from the work buffer: the
        save path read sector 0 there ("MC", its checksum 0x0e last) and
        printed the file's path over it, so "0Games   ", 0, zeros, 0x0e
```

## The save (0x800e3c98, 1380 words)

```
word 0     the sum of every word, this one as 0
word 1     the records in use, a bit each
word 2     a save has been loaded (0x800e3ca0)
word 3     the record last used
record k (1-15) at 344 k
  +0x10    the sum of the 86 words from here, this one as 0 (record 15's
           runs to the save's end)
  +0x14    the name, 12 bytes ("GAME %d")
  +0x20    the game state (0x800c5678, 328 bytes)
  +0x44    three words from a table at 0x800b8718 by the state's +0x00
```

The write (0x8007062c) seals the records' sums, then the whole, checks
them (0x80070234), and writes. A fresh file (0x800707a8) clears the
records' flags and writes the empty save, all zero.

## Saving and loading from the pause (0x800398bc)

The pause (0x8003e05c) runs the menu 0x800398bc(0, 0, 1, 1) (see
[objectives](objectives.md#the-menu-0x800398bc-mode-0)). Save Game? sets
state 3 and Load Game? state 9, from the pause page or from the Quit
confirm (0x80036368, its items 2 and 4). The title's Load Game runs the
same function as 0x800398bc(8, 1, 0, 0), from state 8. Every card call
passes port 0, the card in slot 1.

The function keeps the state in s2, the cursor in s1 (0: none; item 0 is
the heading), a grid of s3 rows by s4 columns with the cursor's column
in s5 and row in s6, the chosen record in 0x800c40a8, and on its stack a
saving flag, a loading flag, the footer flag, the item count and the
cursor colour.

### A frame

1. The speech stream (0x8008d654).
2. Moves, while the cursor is not 0. The pause page has its own rules.
   Elsewhere the items are numbered down the columns: Left and Right
   step s3 items (a column), Up and Down one item, inside the grid, with
   no wrap. Left and Up play sound 124, Right and Down 125
   (0x8006f26c), only when the cursor moves.
3. Cross plays 126 in every state, then acts by state (table
   0x80013268).
4. Start (pad +0x10), on any frame but the first, ends the menu with 0
   from any state.
5. Triangle plays 124 in every state, then acts by state (table
   0x800132f8).
6. The state's work (table 0x80013370): a layout, or a card call. A
   Cross that changes the state runs the new state's work in the same
   frame.
7. The draw, unless a card message drew this frame.
8. The controller check (0x80039720).

On the way out of the pause (mode 0) the three button cubes it made with
0x8003f444(0) (0x800c4b14, 0x800c4b18, 0x800c4b40) are freed.

### The draw

Each screen is a list of 44-byte [text](text.md) items, laid out twice:
the shadows at x + 2, y + 2 in 0x010101, drawn first, then the items.
Font 1 for a heading, 0 for the rest. x is centred (0x8002e144 flag 1)
unless given. Each frame 0x800391d4 colours the items: all light blue
0xe4b96a (106, 185, 228), item 0 pink 0x5d17c6 (198, 23, 93), and the
cursor's item blinking: (255, 218, 0) at first, then (236, 205, 19) and
(255, 240, 119) in turn, every 26 vertical blanks (VSync(-1) against
0x800c4b58). The colour carries over from screen to screen.

Then: a backdrop fade started every 30 frames (0x800c4080), frame start
(0x8001d440(1)), backdrop B (0x80034fdc), the burst (0x8003f41c) except
on the record lists, the shadows and items (0x80039180, 0x8002ecd0(1)),
the pause's two counters (0x80090174) on the pause page only, the footer,
and the flip (0x8001d3e8(1)). The footer is PRESS X TO CONTINUE
(0x80035798) while the footer flag is set, else CANCEL and ACCEPT
(0x80035550, 0x80035674), as in
[screens](screens.md#press-x-to-continue-cancel-and-accept). The flag is
set on entering states 5, 11, 12, 13, 15, 16, 17, 20 and 23 and the
load's "No Game File" page. It is cleared at 0x80039e30 (Cross on 4,
11, 14, 16, 20 or 21 and No on 13, back to the pause; Cross on 12, back
to 3; No on 15, back to 10) and at 0x80039e4c (Cross on the load's 5
and 25). Nothing else clears it. So the save list reached from 13 shows PRESS X while the one
reached from 26 shows CANCEL and ACCEPT (both seen), and (from the code,
not seen) the pause page keeps PRESS X after No on 5, Cross on 17, or a
Triangle out of a page that had set it.

### Card messages (0x80039398)

A state that calls the card first lays out two lines, font 0, centred,
light blue, at y -40 and -20, the second "Please wait." (string 57). It
draws one whole frame with them (fade step, backdrop B, burst, the
lines; no heading, cursor or footer), then makes the call, which blocks.
The oracle's card answers at once, so each shows for one frame there.

| layout | first line |
|---|---|
| 0x80038880 | "Checking Memory Card." (56) |
| 0x800389a0 | "Formatting Memory Card." (58) |
| 0x80038ac0 | "Checking for space." (59) |
| 0x80038be0 | "Creating save game file." (60) |
| 0x80038d00 | "Getting save game file." (61) |
| 0x80038e20 | "Checking for save game file." (62) |
| 0x80038f40 | "Reading save game file." (63) |
| 0x80039060 | "Saving game." (64) |

### The pages

Strings are numbers in the table at 0x800bb4e8. The heading is "Save
Game" (14) or "Load Game" (13), font 1, centred. Every other item is
font 0 and centred, at the y given, except Yes? (11) at (-125, 0) and
No? (12) at (60, 0). A Yes / No page is one row of two (s3 1, s4 2) and
its cursor starts on No?.

| state | layout | items (string, y) |
|---|---|---|
| 4 | 0x8003656c | heading -60; "Memory Card is not inserted\n" (16) -30; "into Memory Card slot 1." (17) -10 |
| 5, save | 0x800366d8 | "Save Game" -70; Yes?, No?; "Memory Card not formatted." (18) -40; "Do you wish to format card?" (19) -20 |
| 5, load | 0x800368d4(.., 0) | "Load Game" -60; "Memory Card not formatted." (18) -30 |
| 25, load, no file | 0x800368d4(.., 1) | "Load Game" -60; "No Game File on the Memory Card." (15) -30 |
| 11 | 0x80036d74 | "Save Game" -70; "Memory Card unable to be" (27) -40; "formatted." (28) -20 |
| 12 | 0x80036c14 | "Save Game" -70; "Memory Card formatted" (25) -40; "correctly." (26) -20 |
| 13 | 0x80036a18 | "Save Game" -70; Yes?, No?; "Would you like to create a" (20) -40; "Rugrats save game file?" (21) -20 |
| 14 | 0x80036ed4 | "Save Game" -70; "Not enough room on Memory" (29) -50; "Card to create Rugrats save" (30) -30; "game file." (31) -10 |
| 15 | 0x80038488 | "Load Game" -70; Yes?, No?; "Are you sure you want to" (225) -40; "load this saved game" (226) -20 |
| 16 | 0x80037f40 | "Save Game" -60; "Game saved." (48) -30 |
| 17 | 0x80038060 | "Save Game" -70; "Error." (49) -40; "Game not saved." (50) -20 |
| 20 | 0x800381c0 | heading -70; "Error." (49) -40; "Memory Card read failed." (51) -20 |
| 21 | 0x80038328 | "Save Game" -70; "Error." (49) -40; "Creating save game file failed." (52) -20 |
| 23 | 0x80038684 | "Save Game" -70; Yes?, No?; "Would you like to overwrite this" (55) -40; "Rugrats save game file?" (54) -20 |
| 10, 22 | 0x80037074 | the record list |

### The record list (0x80037074, states 10 and 22)

31 items. The heading ("Save Game" in 22, "Load Game" in 10) is centred
at y -90. Items 1-15 are the numbers "1" to "15" (strings 32-46), items
16-30 the names, all font 0. Record k (1-15) sits in column (k - 1) / 5
and row (k - 1) % 5: its number at x -190, -45 or 110, its name at x
-160, -15 or 140, both at y -60 + 25 row. The name is the record's +0x14
in the save in RAM (0x800e3c98 + 344 k + 0x14, "GAME k") when word 1 bit
k is set, else "---" (47). The grid is 5 rows by 3 columns and the
cursor starts on record 1 each time the list opens. The cursor's name
item takes the cursor colour with its number.

On the load list, when the cursor's record is in use, a 32nd item: "%s -
%d Piece - %d Bar" (206; 207 with Bars, 208 with Pieces, 209 with both)
filled with the level's name (string 0x800b7d60[state +0x00]), the pieces
(bits set in the record's state +0x44, 0x80091980) and the bars (bits
set in the 32 words at state +0x48, 0x80091a80) mod 15. Font 0, centred,
y 64, pink (0x800c40b0); its shadow (0x800c40bc, 0x010101) is at y 66
and the same x. Seen: "Pickles' Home - 0 Pieces - 0 Bars" and
"Grandpa's Teeth - 0 Pieces - 0 Bars".

### Save (state 3)

- 3: the saving flag. "Checking Memory Card.", then 0x80070d80(0): 0 (no
  card: 0x8007146c without bit 2) to 4; 1 (sector 0 read but no "MC":
  0x800716a0 gives 4) to 5; -1 (ready and formatted) to 25.
- 25: "Checking for save game file.", then 0x8007047c(0, 1): it lists the
  card's "bu00:BASLUS-00650*" files (0x80072038) and looks for "Games"
  (0x800c4368). Found: 26. Not: 13.
- 26: "Reading save game file.", then 0x80070598(0): the 0x1590 bytes into
  0x800e3c98, sums checked (0x80070234). Good: 22. Bad: 20.
- 22: Cross on record k sets 0x800c40a8 = k; a record in use (word 1 bit
  k) goes to 23, an empty one straight to 24. Triangle: the pause.
- 23: Yes to 24. No back to 22, the cursor on record 1.
- 24: "Saving game.", then 0x80070b14(0x800e2b48, 0, k): 1 to 16, 0 to 17.
- 13: Yes: "Checking for space." and 0x8007094c(0): no room, 14.
  "Creating save game file." and 0x800707a8(0): failed, 21. "Getting
  save game file." and 0x80070dd8(0), the count of the game's files: 0
  to 21, 1 to 22 (more stays on 13; it cannot happen with one file
  name). No: the pause.
- 5: Yes: "Formatting Memory Card." and 0x80070ca8(0) (0x80071508): 1 to
  12, 0 to 11, -1 (card gone) leaves the page with no cursor, which only
  Start can leave. No: the pause.
- 12: Cross checks the card again (3).
- Cross on 4, 11, 14, 16, 17, 20 or 21, Triangle on 22, and No on 5 or
  13 go back to the pause page with the cursor on Save Game?. Triangle on
  4 goes back with the cursor on Load Game?.

### What the save writes (0x80070b14)

1. The card must be ready (0x8007146c bit 2) and formatted (0x800716a0
   bit 2).
2. 0x80070358: the save's word 2 = 0; record k's +0x20..+0x167 = the
   328-byte game state (0x800c5678); record +0x44..+0x4c, which is the
   state's +0x24..+0x2c (the respawn position), = 0x800b8718[state
   +0x00], the hub's spot for the current level; word 3 = k.
3. 0x8007094c(0): at least one free block: 15 less every file's blocks
   (0x800719a0), ours too, less one (0x80071a80). (guess: a full card
   fails here even though the file exists.)
4. The name "GAME %d" (0x800c4370) printed into a 12-byte stack buffer,
   then 0x80070804(k, name, 0x800c5678, ..): 12 name bytes to +0x14 (the
   bytes after the NUL are stack leftovers, seen "GAME 1\0\0x\xe4\x1f"),
   +0x1f = 0, the state and the spot again (0x80070448), word 1 |= 1 <<
   k.
5. 0x8007062c(0): the sums, then the whole 0x1590-byte save over the
   file (0x80072320).

It returns 1 on success. On a failure the record's name in RAM begins
"---" (0x800c4378). The save is the whole state block, not a part of it.

The file (0x800707a8): 0x80072498(0, "Games", 0x1590) opens
`BASLUS-00650Games` one block long with the title frame above ("RugRats
Games", three icon frames), 0x8007047c finds it, 0x80070558 clears the
records' names, words 1 and 3 and sets 0x800c4364, and 0x8007062c writes
the empty save.

### Load (state 9)

- 9: the loading flag, then 8.
- 8: "Checking Memory Card." and 0x80070d80(0), as in 3: 0 to 4, 1 to 5
  (the load's page, no choice), -1 to 25 with the cursor on record 1.
- 25: "Checking for save game file." Found: 26. Not: the page stays,
  showing "No Game File on the Memory Card.".
- 26: "Reading save game file." Good: 10. Bad: 20.
- 10: Cross on record k with a name (record +0x14 non-zero, 0x800e3cac +
  344 k; the list itself goes by word 1) sets 0x800c40a8 = k and goes to
  15. On an empty record only the sound plays. Triangle: the pause, the
  cursor on Load Game?.
- 15: Yes ends the menu with 1, after 0x80039888(k). No: back to 10, the
  cursor on record 1.
- Cross on 4 or 20: the pause, the cursor on Load Game?. Cross on 5 or
  25: the pause with the cursor on Quit Game? (3; guess: meant 4).

From the title (mode 8) the same pages run, but the Cross and Triangle
returns end the menu with 0 instead, and Yes on 15 returns 1 to the
front end.

### What the load does

0x80039888(k) clears 0x800e3ca0, then 0x80070970(0x800e2b48, 0, k)
checks the card, finds the file, reads all 0x1590 bytes into 0x800e3c98
again (no sum check here), copies record k's 328 bytes to the game state
(0x800c5678) and sets 0x800e3ca0 = 1. The menu returns 1 whether or not
this worked. (From the code: a failed read leaves the state as it was,
and the game still goes to Pickles' Home.)

### Back to the game

After a save the player is on the pause page again, the cursor on Save
Game?. Continue?, Triangle or Start end the menu with 0 and the level
goes on (see [gameplay](gameplay.md#the-frame-0x80018c38)).

After a load the menu returns 1. The pause (0x8003e05c) removes its
counters and cubes, calls 0x80090234(1) and returns 1, and the frame loop
takes it as exit code 1 ([gameplay](gameplay.md#exit-codes)). The
teardown (0x80018230) records +0x04, now the saved level, sets +0x00 =
1, and shows level 1's loading screen (0x8003b168, 0x8003eefc). Level 1,
Pickles' Home, starts. With 0x800e3ca0 set, the runner (0x80017f78)
calls 0x80070cc8(player, 0x800e3c98, 0x800c40a8 by 0x8003e050): the
player goes to the record's +0x44..+0x4c, the saved level's spot, and
gets its room (0x8002b660); then 0x800e3ca0 is cleared. A load never
goes back into the saved level itself.

### Seen in the oracle

rugrats-hle with an empty card folder (RUGRATS_CARD), level 2 entered
with the level patch (state +0x00 read 1):

- Pause, Down, Cross: "Checking Memory Card.", "Checking for save game
  file.", then 13 on No?. Left (124), Cross (126): "Checking for
  space.", "Creating save game file.", "Getting save game file.", and
  the save list, every name "---", PRESS X footer.
- Cross on record 1: "Saving game.", then "Game saved.". The file: word
  1 = 2, word 3 = 1, record 1 "GAME 1", +0x44 (5775, 779, 19403), which
  is 0x800b8718[1]. Cross: the pause, on Save Game?.
- Right (125), Cross: "Checking Memory Card.", "Checking for save game
  file.", "Reading save game file.", the load list with "GAME 1",
  "Pickles' Home - 0 Pieces - 0 Bars" and CANCEL / ACCEPT. Cross: 15 on
  No?. Left, Cross: 0x80070970 loaded record 1, the loading screen,
  level 1, and 0x80070cc8 put the player at (5775, 779, 19403), room
  23.
- With state +0x00 poked to 2 before the save: "Grandpa's Teeth - 0
  Pieces - 0 Bars", and after the load the player stood at (37002, 779,
  9866), room 21, in Pickles' Home.
- Saving again into record 1 (after reading the file, 26 to 22) asked
  "Would you like to overwrite this / Rugrats save game file?" on No?;
  Yes wrote level 2's spot into record 1.
- With 0x80070d80 made to return 0: "Save Game" and "Load Game" over
  "Memory Card is not inserted / into Memory Card slot 1.", CANCEL /
  ACCEPT. Cross went back to Save Game?, Triangle to Load Game?, and
  Start closed the pause.

Not reached there: 5, 11, 12, 14, 17, 20, 21, and the title's mode.
States 6, 7, 18, 19 and 27-31 are never set from either caller (27 only
as the first argument; 28-31 follow from it) (guess: left over).

### Against the port

- The pause's Save Game and Load Game (crates/rugrats/src/world/game/pause.rs and app/play.rs,
  `game.save`, `game.load`) write the last-used record (word 3) and load
  it at once, with none of these pages.
- `Save::put_record` (card.rs) copies the state as it is: it does not put
  0x800b8718[level] into +0x44..+0x4c or clear word 2.
- The title's list (front.rs `load_frame`, `draw_load`) uses font 1, x
  -191 + 155 col and names 35 to the right, y -58 + 25 row, a heading at
  y -95 in (255, 80, 200), white items and a (255, 255, 5) cursor. It
  wraps the cursor, plays the front end's sounds (1-4), loads on Cross
  without the "Are you sure" page, and has no detail line, burst or
  messages.

## In the port and the oracle

The port keeps the cards as folders, `work/memcard/slot1` and `slot2`,
each holding this file byte for byte; slot 1 always has a card. With a
card in, the original goes from the logos straight to the title menu.

The oracle keeps its card in `work/oracle/memcard/slot1` the same way and
answers the card's BIOS calls from it (rugrats-hle `card.rs`), making up a
card image for raw sector reads. Its `card_save` example has the original
make the file; the port's empty save matches it byte for byte.

## Unknown

- The game state's 328 bytes.
- The table at 0x800b8718.
- Where in play the game saves, and the load path's menus.
