---
title: Boot, main and the start-up calls
status: partial
discs: US
covers: US SLUS_006.50:0x800977e8, US SLUS_006.50:0x800185f8, US SLUS_006.50:0x800193fc, US SLUS_006.50:0x80019630
worklog: 3
---

# Boot, main and the start-up calls

## crt0 (`_start`, 0x800977e8)

PsyQ's: zeroes 0x800c4910 .. 0x800eafb0 (bss), sets sp from the word at
0x800c3f34 less 8, gp = 0x800c3e80, `InitHeap(0x800eafb0 + 4, top -
stack size - bss end)` with the stack size at 0x800c3f38, then `main`.

## main (0x800185f8)

```
game_init()                        0x800193fc
...                                sound, display, the state block
copy 328 bytes 0x800b27d8 -> 0x800c5678   the game state's initial values
0x80019a3c(498 = DB00/CHARACTR.RAW, sp+16), 0x800219d8(result) ...
loop on mode = gp+16 (0x800c3e90):
  -1   -> 0x80030158(-1) picks the next mode (front end)
  25 in activity play -> 0x80096a30()
  < 254 -> 0x80017c0c(mode) runs a database
  else -> 0x8007475c(mode >> 8)
```

It prints "Activation of normal play mode" or "...activity play mode" by the
byte at 0x800c5699 (0 normal, 1 activity).

## game_init (0x800193fc)

1. ORs bit 0, bit 20 and bit 15 into words at 0x8009f2f8, 0x8009f870 and
   0x8009fc70 (inside the library's code range; what they are is open).
2. 0x80021a2c; clears the scratchpad's last 1 KB from 0x1f8003fc down.
3. 0x8001a0e4; `cd_init` (0x80019630): CdRemove, `_96_init`, CdInit,
   CdReadyCallback(0), CdSyncCallback(0), CdControl(Setmode, 0x80: double
   speed).
4. libetc's interface call at 0x8009b168; 0x80072730 (memory card); 0x8001beb8.
5. `VSyncCallback(0x800192b4)`.
6. Root counter 0xf2000001 set to 0xffff, mode 0x1000, started.
7. 0x8001de38, 0x8006e7e8, 0x80021e40, `VSync(0)`, 0x8002ed80.

## The frame

`0x8001d00c` ends a frame: it finishes the drawing (0x8001d2b0,
0x8001d34c), and either swaps at once (0x8001ce0c, when the byte at gp+0xa8
is set) or sets bit 1 of the flag byte gp+0x84 (0x800c3f04) with a 20-frame
time-out at gp+0x85. The vertical-blank side (`0x8001d0d8`, through the
VSync callback) does the swap once `DrawSync(1)` reports the GPU idle and
clears the bit; `0x8001d3c4` polls it.

## Unknown

- What the patched words at 0x8009f2f8, 0x8009f870 and 0x8009fc70 are.
- The modes main dispatches: what 0x80030158, 0x80017c0c and 0x8007475c run.
