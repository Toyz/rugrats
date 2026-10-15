---
title: Sound on the disc that no table entry reaches
status: partial
discs: US
covers: US /DATA/SOUND/SEPS, US /DATA/SOUND/VAB, US /DATA/SOUND/XA/DB14.ISF, US /DATA/SOUND/XA/DB17.ISF, US /DATA/DB14/SPEECHTB.DAT, US /DATA/DB17/SPEECHTB.DAT
worklog: 1
---

# Sound on the disc that no table entry reaches

Files present in the ISO directory that no entry of the
[file table](../formats/file-table.md) points at.

Sequences (`/DATA/SOUND/SEPS`): `EASTER.SEP`, `GLASSES.SEP` (the table uses
`CGLASSES.SEP`, same size), `GLDRUSH.SEP`, `MINING.SEP` (33026 bytes, the
second largest), `OUTRSPAC.SEP`, `RACE.SEP`, `RETURN.SEP`, `SPACE.SEP`,
`TRIUMPH.SEP`, `AISLE7P1.SEP`, `AISLE7P2.SEP`, `AISLE7P3.SEP`, and two older
versions, `ACTEGG.OLD` and `INTRO.OLD`.

Banks (`/DATA/SOUND/VAB`): `EASTER.VB/VH`, `GLDRUSH.VB/VH`, `RACE.VB/VH`, and
`OLD_MENU.VH` with no body.

Music and speech: `XA/DB14.ISF` (6094848 bytes) and `XA/DB17.ISF` (6438912),
and `DB14/SPEECHTB.DAT`, `DB17/SPEECHTB.DAT`. The table's slots for
databases 14 and 17 point at DB01's music and speech instead (631, 634, 599,
602).

## Unknown

- Whether DB14's and DB17's own music is the right music for those levels
  and the table entry a late redirect, or the files are superseded.
