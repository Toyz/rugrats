---
title: The CD and its file system
status: partial
discs: US
covers: US SYSTEM.CNF, US SLUS_006.50, US CDFIX.DAT, US /DATA
worklog: 1
---

# The CD and its file system

One Mode 2 data track, 258684 sectors (57:29), no CD-DA. Volume `SLUS_00650`,
publisher `THQ`, created 1998-11-01. ISO 9660 with CD-XA attributes.

## Boot

`SYSTEM.CNF`: `BOOT = cdrom:\slus_006.50;1`, `TCB = 4`, `EVENT = 10`,
`STACK = 801FFFF0`. The one executable, `SLUS_006.50` (743424 bytes), is a
PS-X EXE loaded at 0x80010000 (741376 bytes of text, entry 0x800977e8, $gp
0x800c3e80 set by code). There are no overlays: no other file on the disc is
code.

## Sector kinds

Counted over the whole track by subheader submode (low bits masked):

| submode | sectors | what |
| --- | ---: | --- |
| 0x08 data | 87950 | ordinary Form 1 files |
| 0x64 Form 2 + audio + real time | 132763 | XA audio in the interleaved files |
| 0x48 data + real time | 29568 | interleaved stream data |
| 0x42 video + real time | 7526 | interleaved stream video |
| 0x20 Form 2 | 154 | |
| 0x00 | 723 | empty sectors (system area, gaps) |

The 58 interleaved files (XA attribute `2555:il`) are the `.STR` movies, the
`CINEMAT` streams and the `.ISF` music; their sizes in the ISO directory count
2048 bytes a sector, the game's own table counts 2336 (see
[the file table](../formats/file-table.md)).

## Directories

| path | files | what |
| --- | ---: | --- |
| `/DATA/DBnn` (00-31) | 359 | one directory per database (level): models, animation, rooms, collision, textures, speech table |
| `/DATA/CINEMAT` | 24 streams + 82 empty | `OPEN.BIN` and `CINEPAK/CINEPAK1-N.BIN` streams; per-episode `*.BIN` entries are all 0 bytes but one |
| `/DATA/OPEN` | 15 | `.STR` movies: logos, three demos, credits; each with a `...B.STR` partner |
| `/DATA/SOUND/XA` | 20 | `DBnn.ISF` interleaved music per database |
| `/DATA/SOUND/SEPS` | 37 | `.SEP` sequence packs (and two `.OLD`) |
| `/DATA/SOUND/VAB` | 108 | `.VH`/`.VB` sound banks |
| `/DATA/MENU` | 27 | `MENU.TEX`, `TEXPOS.DAT`, `TIMS/` (eyes/mouth TIMs, `.BS` stills) |
| `/DATA/FONTS` | 2 | `PHOUSE.FNT`, `SYSTEM.FNT` |
| `/DATA/HINTS` | 42 | `HINTnn.TXT` |
| `/CDFIX.DAT` | 1 | 31457280 bytes at LBA 243174, to the end of the disc; no entry in the game's table |

## Unknown

- What `CDFIX.DAT` holds (padding to the disc's outer edge is the guess).
- The submode 0x20 sectors' owners.
