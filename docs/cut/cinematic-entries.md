---
title: Cinematic entries no table row names
status: partial
discs: US
covers: US /DATA/CINEMAT
worklog: 1
---

# Cinematic entries no table row names

The per-episode `CINEMAT/*/*.BIN` files are 0 bytes (except
`HOMELVL3/R2LS.BIN`, 13252); the table carries them with size 0 and the
pictures come from elsewhere. These have no table row at all:
`BONUS9/INTRO.BIN`, `BONUS9/LOSE.BIN`, `BONUS9/WIN.BIN`, `EPISODE2/BOSS.BIN`,
`EPISODE6/DANCE.BIN`, `GOLFCAM/6.BIN` to `9.BIN`, `HOMELVL3/HOME.BIN`,
`HOMELVL3/SEESW.BIN`.

The executable still names "Golf Cam 6" to "Golf Cam 9" and "Reptar Dance"
among its cinematic names.

## Unknown

- What the 0-byte entries are for, and where cinematics take their data.
