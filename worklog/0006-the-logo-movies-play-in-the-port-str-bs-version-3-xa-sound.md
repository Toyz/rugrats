---
number: 6
title: The logo movies play in the port: STR, BS version 3, XA sound, the input module
date: 2026-10-14
area: video, audio, format, input, test
files: crates/rugrats/src/movie.rs, crates/rugrats/src/input.rs, crates/rugrats/src/main.rs, crates/rugrats-data/src/movie.rs, crates/rugrats-data/src/bs.rs, crates/rugrats-hle/src/gpu.rs, crates/rugrats-hle/src/lib.rs
resolves: 5.2
---

# 6. The logo movies play in the port: STR, BS version 3, XA sound, the input module

After the legal screen, main plays three movies, THQ, Nickelodeon and
n-Space (`movie_play(0..2, skippable)`, 0x8008b448), and the port now does
too: frames decoded natively, the XA soundtrack through retro_rt's audio
output, a button press skipping to the next logo.

## Finding them

The oracle showed colour noise in the shape of the THQ logo: the movies are
decoded at 24 bits and shown in the GPU's 24-bit display mode, which the
oracle's screen grab read as 15-bit. With the mode bit (GP1 0x08 bit 4)
modelled, the oracle shows them right. `movie_play` takes movie `n` from a
table at 0x800bae4c of `{frames, lba}`; seven movies, the last the 1875-frame
credits. None of the seven `...B.STR` files is in it (`docs/cut/b-movies.md`).

## Decoding, held to the original

A frame is the 2016-byte parts of its STR sectors (`rugrats_data::movie`).
THQ_LOGO's frames are BS version 3: the DC as MPEG-1 codes it, a size from
table B.12 (luma) or B.13 (chroma) and a difference from the last block of
the same kind, kept per Cr, Cb and luma, in steps of 4. NICKLOGO's are
version 2, and end with a DC of 0x1ff, which libpress takes as the end of
the frame; the decoder read on into the padding and failed at bit 22026 of
frame 1 until it did the same. The oracle's MDEC dumps cover the three logos:
`thq_logo_frames_decode_to_what_libpress_hands_the_mdec` holds all 101
THQ frames to them by number, and
`the_logo_movies_decode_as_the_original_decodes_them` holds all 345 frames
the original decoded (it dropped 6 of 351) to the port's, word for word.

## Pacing and the skip

The oracle shows the original sampling the pads (0x8001acc0) and asking for
a skip (0x8001baf8 mode 0) about every 4 vertical blanks during a logo: a
frame is shown when its sectors have arrived, about 10 sectors at 150 a
second. The port reads the whole stream up front (a PC holds it easily) and
shows frame k when its last sector would have arrived, decoding it then; the
soundtrack is decoded with rrt's XA decoder and starts at its first sector's
time. The skip is checked every other tick.

The input module is ported as the game keeps it: per port a 32-frame
history of each of the 16 buttons, newest in bit 0, so "pressed" is the
low two bits reading 01 (`a_press_is_seen_for_one_frame`). Its six
controller maps (0x800b44ac) are not ported yet.

The oracle gained the BIOS pad driver (B0:12 InitPAD, B0:13 StartPAD) and
`--press F:BUTTON[:LEN]` scripts, and `--count ADDR` to count calls per
frame, which is how the pacing above was measured.

**Still unknown:**
- the exact frame pacing of `movie_play` (frame drops, the CD's start-up seek)
- what the B movies are
- the controller maps' contents and the action list
