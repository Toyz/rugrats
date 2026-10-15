---
number: 1
title: The disc, and the file table the game reads it through
date: 2026-10-14
area: disc, content, format, tooling, build
files: crates/rugrats-data/src/files.rs, crates/rugrats-re/src/main.rs
started: 2026-10-14T18:48:02-07:00
took: 7m
---

# 1. The disc, and the file table the game reads it through

The US disc (`SLUS-00650`, THQ, mastered 1998-11-01) is one Mode 2 track of
258684 sectors with no CD-DA. It boots `SLUS_006.50`, a 743424-byte PS-X EXE
loaded at 0x80010000 with its entry at 0x800977e8; nothing else on the disc is
code, so there are no overlays to chase. The PsyQ library ids left in it
(`sys.c,v 1.140 1998/01/12`, `intr.c,v 1.75`, `bios.c,v 1.86`) date the SDK
to early 1998, a little newer than hwtr's.

The game does not open files by name. `read_file` at 0x80019794 shifts its
first argument left by 3 and indexes a table at 0x800b2c40 of 714
`(lba, size)` pairs, then seeks and reads with libcd. Searching the image for
the LBA of `DB00/BURST2.CMD` (115370) found it; the table's start was found
by walking back while pairs looked like LBA and size, and the code's
`lui 0x800b; addiu 0x2c40` confirms it. It ends at 0x800b4290, where the
words stop being LBAs. `file_size` at 0x80019a04 returns the size, and for an
unused entry (LBA 0) returns an uninitialised stack word.

Every used entry's LBA is the start of an ISO directory entry. Form 1 sizes
equal the ISO's; the 58 interleaved files are sized at 2336 bytes a sector
(OPEN.BIN: 7071 sectors, ISO 14481408, table 16517856). The table groups by
kind and then by database number 00-31 (MOD at 242, ANM 274, FRM 306, ROT
338, HIE 370, LP 402, LEVEL 434, COL 466, A.RAW 499, TEXPOS 553, speech 585,
music 617), which is what makes the gaps readable.

`rugrats-re files` prints the table against the ISO directory and then the
files no entry reaches. Those are the first cut content: all of `DB19`
(rooms, collision, textures, `Q.DAT`, but no model files), `DB22FRM.BIN` and
`DB22ROT.BIN`, DB14's and DB17's music and speech (their slots point at DB01's),
twelve sequences and four banks including `MINING.SEP`, `RACE`, `EASTER` and
`GLDRUSH`, and eleven cinematic entries such as `GOLFCAM/6-9.BIN` and
`EPISODE6/DANCE.BIN`. Each has a page under `docs/cut/`.

The executable names its databases in a list at 0x800bb744 that starts
"Pickles' Home", "Grandpa's Teeth", "Incident on Aisle 7"; read in order it
gives DB19 the name "No More Cookies", but nothing yet shows the list is
indexed by database number.

## The workspace

The tools are hwtr's, carried over: `rugrats-psx` is `hwtr-psx` (the EXE
format, disassembler, recursive-descent analysis, pseudo-code),
`rugrats-re` is `hwtr-re` without the BIG archive and state-machine commands
that only fit Hot Wheels, and `rugrats-data` starts with hwtr's TIM and VAB
readers, both Sony formats. The disc, ISO and XA plumbing is retro_rt's
`rrt::disc`. The first analysis pass finds 2293 functions to 0x800b2794
but no jump tables and 15800 code words no function owns, so it does not
yet fit this compiler's output as well as it fit hwtr's.

**Still unknown:**
- whether the database name list at 0x800bb744 is indexed by database number
- what the 0-byte cinematic entries are for, and where cinematics take their pictures
- what `CDFIX.DAT` holds
- why the analysis finds no jump tables in this executable
