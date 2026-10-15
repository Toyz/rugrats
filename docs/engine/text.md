---
title: The text system
status: partial
discs: US
covers: US SLUS_006.50:0x8002ed80, US SLUS_006.50:0x8002e144, US SLUS_006.50:0x8002e67c, US SLUS_006.50:0x8002e2b4, US SLUS_006.50:0x800b78c8, US /DATA/FONTS/PHOUSE.FNT
worklog: 18
---

# The text system

## The fonts

Boot (0x8002ed80) uploads PHOUSE.FNT (file 106) raw to VRAM (960, 256),
64 x 213 words: three 4-bit fonts sharing one CLUT at (960, 468), inside
the image (0x800c3f7c-80 hold it for fonts 0-2; 0x800c3f82 a second at
(960, 511)). Their line heights are 16, 24 and 12. The glyph table at
0x800b78c8 has 91 glyphs a font (' ' to 'z'), four bytes each: u, v,
width, and how far below the line the glyph's top sits.

## A text item (44 bytes)

```
+0x00 s16 font           +0x14 char *string
+0x02 s16 spacing        +0x18 u16 flags: 2 slide x, 4 slide y,
+0x04 u32 colour                   8 bounce, 16 wobble, 32 wait;
+0x08 s16 x, +0x0a y               exactly 1: drawn by 0x8002ea0c
+0x0c s16 target x, y    +0x1a s16 which half of a bounce or wobble
+0x10 s16 rest x, y      +0x20 s16 speed x, +0x22 y (pixels a frame)
                         +0x24 s16 bounce height or wobble count, x;
                         +0x26 y   +0x28 s16 frames to wait
```

Positions are from the screen's centre.

`text_centre(item, flags)` (0x8002e144): with flag 1 takes half the widest
line off x, with flag 2 half the lines' height off y. A line's width is the
sum of its glyphs' widths; the gaps, (glyphs - 1) x spacing over the whole
string, are added once; a new line counts as glyph 10 of the next line.

`text_draw(item)` (0x8002e67c): a draw-mode packet (page 0x21f: (960,
256), 4-bit, dithered), then one sprite (0x64, modulated by the colour) a
glyph, line height tall, at y + the glyph's drop. A space moves on half a
line height; a new line goes back to x and down a line.

`text_slide(item)` (0x8002e2b4), a frame: a waiting item counts down and
then stops waiting. Otherwise y moves toward its target a pixel at a time,
`speed` times. The test is whether flag 4 was set as the frame began, and
the speed is read again each pass. On arrival: done (flag 4 off) with no
bounce or wobble. A bounce goes up a third of its height and back to rest,
the height a third smaller each time, until both meet. A wobble pulls back
3 (0x800c3f6c) at speed 2 and returns, `count` times. The x slide is the
same with flag 2, read every pass.

The port (`rugrats::text`) is checked against all three functions in
rrt-emu: centring, the sprite packets, and 80 frames of each kind of slide.
