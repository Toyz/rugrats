---
number: 17
title: Memory cards as folders, the save file byte for byte
date: 2026-10-14
area: save, decomp, tooling, test
files: crates/rugrats/src/card.rs, crates/rugrats/src/disc.rs, crates/rugrats/tests/card.rs, crates/rugrats-hle/src/card.rs, crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/examples/card_save.rs, docs/engine/memory-card.md
resolves: 16.2
started: 2026-10-14T21:32:37-07:00
took: 12m
---

# 17. Memory cards as folders, the save file byte for byte

The memory card is a folder per slot, as in the other ports:
`work/memcard/slot1` (always in) and `slot2` (when the folder exists),
each holding the game's one card file, `BASLUS-00650Games   `, byte for
byte. `rugrats::card` builds it: the title frame (0x80071ee0, with the
Shift-JIS conversion and the icon and CLUT from the executable), the save
(1380 words, a sum over all and one per record), and the save write's
last frame, whose tail comes from the work buffer. Before the write the
game had read sector 0 into that buffer (the "MC" frame, checksum 0x0e)
and printed the path over it.

The oracle needed a card first. rrt's stock BIOS reports every event as
fired and no file on any card, so the original showed "There is no Memory
Card". rugrats-hle's `card.rs` keeps a card in `work/oracle/memcard/slot1`
the same way: the two card event classes with real pending state, the
other classes still always fired, `_card_info` and `_card_load`, raw
sector reads from a card image made up from the folder, and the `bu` file
calls. libapi's firstfile also looks the device up in the kernel's device
table (0x150), so the oracle puts one there.

With a card in, the original goes from the logos straight to the title
menu. The `card_save` example calls the original's "new save file" path
(0x800707a8) from the title. Its file and the port's empty save are
byte-identical (test). The Shift-JIS conversion (all 256 bytes), the
title frame, and the sums are each checked against the original's
functions in rrt-emu.

**Still unknown:**
- the game state's 328 bytes
- where play saves
- the load path
