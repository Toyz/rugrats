---
number: 8
title: LEVELnn.BIN read the way the room loader reads it
date: 2026-10-14
area: format, decomp, tooling
files: crates/rugrats-data/src/level.rs, crates/rugrats-re/src/main.rs
started: 2026-10-14T19:54:32-07:00
took: 2m
---

# 8. LEVELnn.BIN read the way the room loader reads it

The room loader (`load_rooms`, 0x8002a208) walks a LEVEL file as five
counted sections: 56-byte records (none in any file), 48-byte quads of
four (s16 x, y, z, u16 flags) corners and 16 more bytes, 24-byte records
(only LEVEL04's 63), 112-byte room records, then per room a byte list,
per room a section of geometry blobs, and a final run of 12-byte records.
A table at 0x800b6e98 names each database's LEVEL, COL and A.RAW files
(434 + db, 466 + db, 499 + db, for all 32).

`rugrats_data::level::parse` follows the walk exactly and checks each
room's section against the size the file gives for it; `rugrats-re level
FILE` prints it. The first run stopped on four files: the loader moves past
the byte lists with `4 - used % 4`, a whole word when they end aligned, not
a round-up. With that, 31 of 32 files are consumed to the last byte and
every final section is 12 bytes a record. LEVEL08 is the exception: its
first quad reads as 32-bit values and its third count is garbage. The
format page is `docs/formats/level.md`.

**Still unknown:**
- why LEVEL08.BIN does not fit the loader's walk
- the room record's fields and the geometry blobs' format
