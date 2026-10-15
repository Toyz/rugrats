---
title: The PsyQ libraries in the executable
status: partial
discs: US
covers: US SLUS_006.50:0x800977e8..0x800b2794
worklog: 3
---

# The PsyQ libraries in the executable

The game's own code is 0x80017c0c .. 0x800977e8; from crt0 at 0x800977e8 to
the end of the code (0x800b2794) is the PsyQ runtime, 515 functions. Ids left
in the image: `sys.c,v 1.140 1998/01/12` (libgpu), `intr.c,v 1.75
1997/02/07`, `bios.c,v 1.86 1997/03/28`. Names are in `symbols/slus_006.txt`.

| range | library | how it was named |
| --- | --- | --- |
| 0x800977e8 | crt0 | the bss clear and InitHeap |
| 0x8009a9e8 .. 0x8009b858 | BIOS call stubs (`li t2, 0xa0/b0/c0; jr t2; li t1, n`) | the stub shape |
| 0x8009af48 .. 0x8009bbf0 | libetc: VSync, interrupt dispatch, VSync and DMA callbacks | "VSync: timeout", "unexpected interrupt", "DMA bus error" |
| 0x8009bc18 .. 0x8009f... | libgpu | "ResetGraph(%d)...", "DrawSync(%d)...", "GPU timeout" |
| 0x800a0858 .. 0x800a0ba4 | libpress (MDEC) | "MDEC_rest", "MDEC_in_sync", "MDEC_out_sync" |
| 0x800a1918 .. 0x800a3798 | libcd | "CdInit: Init failed", "CD_sync", "CD_cw", "DiskError" |
| 0x800a4710 .. 0x800ae018 | libspu / libsnd | "DMA STATUS ERROR", "This is not SEP Data.", "SPU:T/O" |
| 0x800b1478 .. 0x800b26f8 | libc: printf, sprintf, puts | the format tables |

## libetc's interrupt model

`ResetCallback` (0x8009b2fc) takes a setjmp buffer at 0x800bbad0 and hands
it to the BIOS's HookEntryInt; an interrupt longjmps there and calls the
dispatcher at 0x8009b3d4. The dispatcher reads I_STAT and I_MASK through the
pointers at 0x800bcb24 and 0x800bcb28, and for each pending bit (0-10)
acknowledges it and calls `handlers[bit]` from 0x800bba9c, then
ReturnFromException. Handler 0 is the vertical blank (0x8009b8b0: counter
0x800bcb60, then eight callbacks at 0x800bcb40); handler 3 the DMA one
(0x8009b9c4: DICR flags, callbacks at 0x800bcb74).

## Unknown

- The exact bounds of libgpu, libspu and libsnd, and libpad/libmcrd if
  linked (the pad has not been traced).
