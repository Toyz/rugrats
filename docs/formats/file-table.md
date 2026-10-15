---
title: The file table in the executable
status: partial
discs: US
covers: US SLUS_006.50:0x800b2c40, US SLUS_006.50:0x80019794, US SLUS_006.50:0x80019a04
worklog: 1
---

# The file table in the executable

The game never opens a file by name. Every load names a file number, an index
into a table of 714 `(lba, size)` pairs compiled into `SLUS_006.50`. The ISO
directory is only there for the disc mastering; the port reads through this
table the same way (`rugrats_data::files`).

## Layout

Little-endian.

```
entry[714] at 0x800b2c40 .. 0x800b4290
  u32  lba     first sector on the disc; 0 = unused entry
  u32  size    bytes; for an interleaved stream, sectors x 2336
```

## Fields

`lba` - matches the ISO directory's extent for every used entry (checked with
`rugrats-re files`). 29 entries are 0.

`size` - equals the ISO size for Form 1 files. For the 58 interleaved files it
is the sector count times 2336 (2352 less the 12-byte sync and 4-byte header),
e.g. `OPEN.BIN` 7071 sectors: ISO 14481408, table 16517856.

## The code

`read_file(index, dest, len)` at 0x80019794 indexes the table (`index << 3`),
returns 0 straight away for an unused entry, then seeks with libcd and reads
with retries ("Too many retries for seeking... I'm giving up!").
`file_size(index)` at 0x80019a04 returns `size`; for an unused entry it
returns an uninitialised stack word instead.

## Groups

By index, as `rugrats-re files` prints them:

| index | what |
| --- | --- |
| 0 | `CINEMAT/OPEN.BIN` |
| 1-91 | `CINEMAT/*/*.BIN` cinematic entries (all 0 bytes but `HOMELVL3/R2LS.BIN`), unused gaps |
| 92-105 | `OPEN/*.STR` movies |
| 106-108 | fonts, `MENU.TEX` |
| 109-210 | VAB banks, VB then VH; 151-210 the `DBnnSFX` banks |
| 211-233 | `.SEP` sequences |
| 234-238 | `.BS` stills |
| 239-241 | `DB00/TEMPLATE.BIN`, `BURST2.CMD`, `LOGO.CMD` |
| 242-273 | `DBnnMOD.BIN` by database 00-31 |
| 274-305 | `DBnnANM.BIN` |
| 306-337 | `DBnnFRM.BIN` |
| 338-369 | `DBnnROT.BIN` |
| 370-401 | `DBnnHIE.BIN` |
| 402-433 | `DBnnLP.BIN` |
| 434-465 | `LEVELnn.BIN` |
| 466-497 | `LEVELnn.COL` |
| 498 | `DB00/CHARACTR.RAW` |
| 499-530 | `DBnnA.RAW` |
| 531-550 | menu `TIMS/*.TIM` (eyes and mouths) |
| 551 | `DB00/TEXACTRL.DAT` |
| 552 | `MENU/TEXPOS.DAT` |
| 553-584 | `DBnn/TEXPOS.DAT` |
| 585-616 | `SPEECHTB.DAT` per database (several point at DB01's or DB08's) |
| 617-648 | `DBnn.ISF` music per database (several point at DB01's or DB08's) |
| 649-671 | `CINEMAT/CINEPAK/CINEPAK1-N.BIN` |
| 672-713 | `HINTS/HINTnn.TXT` |

The per-database groups have a slot for DB19 that is 0 everywhere: see
[the DB19 database](../cut/db19.md).

## Unknown

- Whether any load reaches a file by LBA without this table.
