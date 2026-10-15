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
