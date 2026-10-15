---
title: Speech
status: partial
discs: US
covers: US SLUS_006.50:0x8008d8e8, US SLUS_006.50:0x8008da4c, US SLUS_006.50:0x8008ddfc, US SLUS_006.50:0x8008df30, US SLUS_006.50:0x8004d0f0, US SLUS_006.50:0x80046d04, US SLUS_006.50:0x800818e0, US SLUS_006.50:0x80080308, US SLUS_006.50:0x80080318, US SLUS_006.50:0x80077a54, US SLUS_006.50:0x800511a8, US SLUS_006.50:0x8007a9d8
worklog: 53
---

# Speech

The babies' and the grown-ups' spoken lines are XA audio. Each level
has its own set. A line is asked for by number with 0x8008da4c. The
drive plays one line at a time.

## A level's set (0x8008d8e8)

The level setup (0x80017c0c) calls 0x8008d8e8(database). It loads:

- the index, file 585 + database, kept at 0x800c45f4;
- the XA file's first sector, from file 617 + database, kept at
  0x800c45fc.

The index:

| offset | what |
|---|---|
| +0x00 | u16 count n (0x800c45ea) |
| +0x04 | n u16 words (0x800c45ec): the line number in the low 12 bits, flags in the top 4 |
| after, aligned to 4 | n records of 4 bytes (0x800c45f0): u16 sector offset, u8 length in units of 8 sectors, u8 channel less one |

A table of n bytes (0x800c45f8) starts as each word's top 4 bits.

The words are sorted by line number. 0x8008ddfc finds a line by binary
search and returns its index, or -1. 0x8008df30 gives its record.

## Saying a line (0x8008da4c)

1. Look the line up; not in the set: nothing.
2. The drive must be idle (0x800c45d8 is 0 or 9); else nothing. The line
   is dropped, not queued.
3. (flags & 3) == 1: nothing (a line already said).
4. Seek to the XA start + offset - 8, filter file 1 and channel
   (record channel + 1) (CdlSetfilter, 13), and read until + length x 8
   sectors (0x800c4924).
5. Lines below 2040 get flag bit 1. So they are said once a level, unless
   flag bit 2 is set (then (flags & 3) is 3).
6. 0x800c45e6 = the line; with 0x800c4618, 0x800c45e4 = 900 (guess: the
   mouth's time).

The XA files interleave 8 channels (DB16's file 633: channels 1-8 in
turn). Lines 2041-2046 (the result screens' voices) are in every set.

## Who asks

- **Talkers (0x8004d0f0).** The placement loader (0x80046d04) gives an
  object its LP +0x4c line (template 613 always says 194). With a line
  and no enter callback, enter is 0x8004d0f0 and leave 0x8004d160. The
  trigger list (0x80072d38) calls enter when the player comes within the
  object's +0xa0 (template +0x34) in 2D, and at the setup for those
  already in range. Enter says the line when the object is in the
  player's room.
- **Rooms (0x80077a54).** Room flag 128 with +0x64 of 5 or more says
  that line on entering (below 5 it is a song). No room in DB2-13 uses
  a line.
- **The ambience (0x800818e0).** Every frame, 0x800b9618[level] runs; in
  activity play the level is picked by character (1: 25, 2: 26, 3: 31,
  4: 23, 5: 29). Each callback counts 0x800c4cc0 down and says a line
  from its tables. The setup (0x800b9598) sets the countdown: 900
  (0x80080308), or 300 with the first-line flag 0x800c4cc4
  (0x80080318: levels 7 and 17). The callbacks are in the level pages.
- **Hits (0x800511a8).** A thrown thing hitting a character: Angelica
  (char 3) 556 or 557, char 10 1065, char 11 1076 or 1079 (rand & 1);
  Chuckie (char 2) 185 in level 2.
- **Golf holing out (0x8007a9d8).** [Golf](../levels/golf.md).
- **Levels 20-21.** The throw (0x8005d11c) 1159 or 1167; round ends
  0x80079f5c (0x800b9220), 0x8007a388 (0x800b922c), 0x8007a504
  (0x800b9244). DB20's set has none of 1159, 1167 or 1148-1180, so those
  calls are silent.
- **Level 13's race.** GoalPost (0x80077070) 723, then cine 20.
- **Level 16.** The slip (state 3, 0x8005b3c4) 212; a cookie
  (0x800514b0) 216.
- **Level 17.** The clock about to run out (0x8007f8a4, timer 10) with
  under 30 nickels: Lil (char 4) 986, Phil (char 5) 859.
- **The puzzle board.** "You did it!" 2046 (0x80094978).
- **Waiting at a post (0x80083e2c, mode 7 entered).** Level 13
  0x800b96d0[rand & 7] (608, 609, 610, 643, 645, 646, 664, 723); level 16
  0x800b96e4[(rand & 255) mod 3] (643, 646, 582). Level 13 also says 611
  or 612 (0x800b96e0) when the wait ends with the player ahead
  (0x80084084 mode 7).
- **Mr. Friend (char 16, level 9).** Hit (0x800834b0) or kicked
  (0x800833a4): 0x800b96c8[mod 3] (153, 187, 146).
- **Level 10's hide and seek.** The tag (0x800831a4) by Chuckie
  (template 2): 0x800b9728[mod 3] (158, 160, 165). Running babies
  (0x80084084 mode 6) count 0x800c4ce4 down and say Tommy (template 11)
  0x800b9704, Phil (7) 0x800b9710, Lil (6) 0x800b971c; then
  (rand & 255) x 16.
- **Level 12's shadows.** Mode 10 ending (0x80084380, +0x688 bit 0x8000):
  143 or 21.
- **Level 7.** Template 931 breaking (0x8004d7c4): 1183.
- **The hub's Spike.** Barks 0x800b96ec (mode 2, rooms 24, 39-41), the
  fetch's 191 / 192 (0x80060b44), 193 / 159 / 234 / 235 (0x800601e4) and
  190 (0x80062ac4) ([level 1](../levels/level-01.md)).
- **Not yet listed:** level 26's prowler (0x80084084 mode 9: sounds 144,
  145 or speech 1134 every 1000 dt); 0x8008d654's pending line
  (0x800c4614).

## Port

- `rugrats::speech::Speech` loads a set and decodes a line with rrt-kit's
  XA player.
- Game::speech holds the lines asked for in a frame. Main plays the first
  one when the soundtrack is free and drops the rest, as the drive does.
- `rugrats::ambience` ports every callback with its tables. Game::talk
  handles the talkers.
- Tools: `speech_index DB [LINE OUT.wav]`, `speech_spots DB`,
  `room_lines DB`, `xa_headers LBA COUNT`.

## Unknown

- Whether a cine posted in the same frame as a line waits for it. The
  port's cine replaces the line at once.
- 0x800c4618 and 0x800c45e4 (900).
- Level 26's prowler and 0x800c4614.
