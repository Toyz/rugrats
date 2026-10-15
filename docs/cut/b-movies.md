---
title: The B movies
status: guess
discs: US
covers: US /DATA/OPEN/THQLOGOB.STR, US /DATA/OPEN/NICLOGOB.STR, US /DATA/OPEN/LOGOXAB.STR, US /DATA/OPEN/DEMO01B.STR, US /DATA/OPEN/DEMO02B.STR, US /DATA/OPEN/DEMO03B.STR, US /DATA/OPEN/CREDITSB.STR
worklog: 6
---

# The B movies

Every movie in `/DATA/OPEN` has a partner named with a B (`THQLOGOB.STR`,
`NICLOGOB.STR`, ...), each exactly 2258944 bytes (1103 sectors). They are
in the file table (entries 93, 95, 97, 99, 101, 103, 105), but the movie
table at 0x800bae4c names only the plain files, and no code found so far
loads the B entries by number.

Their sectors differ from the plain movies': submode 0x42 (video, real
time), frames 640 x 192, BS version 2. THQLOGOB's first frame header reads
`a0 05 00 38 01 00 02 00` (0x5a0 words, q_scale 1, version 2).

## Unknown

- What they are: a high-resolution or PAL encode, or placeholders of a
  fixed size, is a guess; none has been decoded and looked at.
- Whether any code reaches file entries 93-105 (odd).
