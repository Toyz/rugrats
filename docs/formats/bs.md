---
title: BS still pictures
status: partial
discs: US
covers: US /DATA/MENU/TIMS/LEGAL512.BS, US /DATA/MENU/TIMS/THQ512.BS, US /DATA/MENU/TIMS/NICK512.BS, US /DATA/MENU/TIMS/NSPAC512.BS, US /DATA/MENU/TIMS/E3SCR512.BS, US SLUS_006.50:0x80030010, US SLUS_006.50:0x800c1f20
worklog: 5
---

# BS still pictures

Five `.BS` files in `/DATA/MENU/TIMS`, all 512 x 240: MDEC run-length blocks
packed with MPEG-1's AC codes. libpress decodes the codes on the CPU
(DecDCTvlc) and hands the result to the MDEC; `rugrats_data::bs` does both.

## Layout

Little-endian.

```
u16  words      run-length output, in 32-bit words
u16  0x3800     with `words`, the MDEC decode command (15-bit, unsigned)
u16  q_scale
u16  version    2; E3SCR512.BS is 3
bitstream: 16-bit little-endian words, most significant bit first
  per block:  dc:10, then MPEG-1 table B.14 codes with a sign bit,
              "10" end of block, "000001" escape to run:6 level:10
```

Blocks go Cr, Cb, Y0, Y1, Y2, Y3 per 16 x 16 macroblock; macroblocks run down
each 16-pixel column (15 per column), columns left to right.

| file | words | q_scale | version |
| --- | ---: | ---: | ---: |
| `LEGAL512.BS` | 21408 | 3 | 2 |
| `THQ512.BS` | 6240 | 1 | 2 |
| `NICK512.BS` | 15264 | 1 | 2 |
| `NSPAC512.BS` | 14624 | 1 | 2 |
| `E3SCR512.BS` | 21568 | 7 | 3 |

## Decoding

Each block emits `q_scale << 10 | dc`, then `run << 10 | level` per code,
then 0xfe00; the output is padded to `words` with 0xfe00. The bitstream's
last bits hold less than a block and are not decoded. The MDEC tables are
libpress's: the executable keeps them as ready MDEC commands, 0x40000001 and
128 bytes (luma and chroma quant tables, identical) at 0x800c1f20, and
0x60000000 and the 64-entry IDCT scale table at 0x800c1fa4.

## Shown

`show_still(file)` (0x80030010) decodes into VRAM at (0, 256), draws it in
32 x 32 textured quads with colour 0x7f7f7f (0x800c3f9c), so each 5-bit
channel is `c * 127 >> 7`, into both display buffers, and holds until 250
vertical blanks after it began. Only the legal screen is shown this way at
boot (main, 0x800186cc).

## Unknown

- Version 3's differential DC (only E3SCR512.BS) is not decoded.
- Where THQ512, NICK512 and NSPAC512 are shown, if anywhere (the logos at boot are `.STR` movies).
