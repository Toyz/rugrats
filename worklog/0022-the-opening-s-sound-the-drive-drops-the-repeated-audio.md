---
number: 22
title: The opening's sound: the drive drops the repeated audio sectors
date: 2026-10-15
area: audio, video, tooling
files: crates/rugrats/src/movie.rs, crates/rugrats/src/cine.rs, crates/rugrats/examples/xa_streams.rs, crates/rugrats/examples/cine_motion.rs, crates/rugrats/examples/cine_ids.rs, crates/rugrats-hle/src/sio.rs, crates/rugrats-hle/src/lib.rs, crates/rugrats-hle/src/gpu.rs, crates/rugrats-hle/examples/gpu_frame.rs, docs/engine/cinematics.md, Cargo.lock
started: 2026-10-14T23:37:28-06:00
took: 24m
---

# 22. The opening's sound: the drive drops the repeated audio sectors

The opening's audio was badly wrong in the port. 5643 of OPEN.BIN's sectors
are XA audio for file 1, channel 1: four in five of the stream, where a
stereo 37.8 kHz channel at double speed needs one in eight. Decoded
end to end they make 300 s of sound for a 47 s scene.

The search went through a few wrong turns:

- Successor tests on the decoded audio and a pitch estimate (median about
  620 Hz at 37.8 kHz) said the audio was one continuous, natural-pitch
  stream.
- The characters' motion per frame looked slow for 30 frames a second.
- The end title card (frames 1184-1412) would last 49 s if the picture
  followed the audio.

The answer was in the bytes: 535 of the first 634 neighbouring audio
sectors are identical. The encoder repeats each sector to fill the
bandwidth, and the CD-ROM controller's decoder drops any sector that
arrives while its output still holds more than a few frames. It has no
flow control, and DuckStation's source models it the same way. The
continuity tests had been comparing sectors with their own copies.

`movie::XaPlayer` models it: a sector arriving at its disc time (1/150 s
each) is decoded only if no more than 10 frames are queued, and a dropped
sector leaves the ADPCM history alone. The opening now plays 882 sectors,
47.1 s, every distinct sector once. The movies go through the same path
(their one in eight never drops).

Along the way the oracle got `RUGRATS_NO_PAD`. With the SIO model and no pad
the game shows "No Controller"; the switch also leaves libpad's handler
out, which gives the old path straight to the opening. The GPU trace now
covers lines and colours and any number of frames. In the oracle's opening
the milk from Tommy's bottle turned out to be about four semi-transparent
gouraud triangles a frame (blend mode 3, grey 166), built by the C model
path's face emitters from a face flag. The port has no such object yet.

retro_rt is now 719d0fa (rrt-dis with the lifting IR and the decompiler).

**Still unknown:** the opening's milk: a model with semi-transparent faces (face flag bit 25, POLY_G3 with blend 3), drawn by the C model path 0x8002390c; which object it is; the characters' eyes and mouths (0x80074ec0, 0x80075304: 8x32 blocks copied into their face textures); the swirl transition in the opening
