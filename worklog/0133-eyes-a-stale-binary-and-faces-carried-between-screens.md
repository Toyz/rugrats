---
number: 133
title: Eyes: a stale binary, and faces carried between screens
date: 2026-10-16
area: render
files: crates/rugrats/src/media/face.rs, crates/rugrats/src/app/flow.rs, crates/rugrats/src/app/mod.rs, crates/rugrats/src/world/game/mod.rs
started: 2026-10-16T01:32:32-06:00
took: 12m
---

# 133. Eyes: a stale binary, and faces carried between screens

The user saw characters without eyes again.

**The cause** was the binary, not the code. `./target/release/rugrats` was
built at 18:07 on 2026-10-15, before worklog 84's fix (about 21:00).
careful.sh sends this session's builds to /mnt/data/claude/targets/rugrats,
so `./target` was never rebuilt. The current code draws eyes:

- in the hub's start cinematic: Stu, Didi, Tommy, Chuckie, Phil, Lil;
- in play: Tommy, walking toward the camera.

`./target/release/rugrats` is now rebuilt from HEAD. Both binaries were
shot headless and checked.

**The faces between screens.** The PlayStation has one VRAM. The
characters' texture is loaded once, at boot (0x800186d8), and every
cinematic writes its mouths and eyes into it (0x80074ec0, 0x80075304). So
play and the next cinematic start from the faces the last cinematic left.
The port gave each screen its own VRAM, built from the texture's file.

`FaceBlocks` (media/face.rs) now copies every mouth and eye block from a
cinematic's VRAM when it ends (the opening, and every scene). The app
puts those blocks into the next cinematic's VRAM and into the game's
VRAM, the game's once for each copy taken (`Game::face_serial`).

Comparing with and without the copy in play: the texture already has
Tommy's open eyes, so the picture is the same at frame 8430. The copy
matters when a cinematic ends on a blink or an open mouth.

**Still unknown:**

- The title's and the hint cards' own stages do not take the blocks.
