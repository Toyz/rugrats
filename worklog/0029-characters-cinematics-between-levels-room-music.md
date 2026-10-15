---
number: 29
title: Characters, cinematics between levels, room music
date: 2026-10-15
area: play, audio, video
files: crates/rugrats/src/animator.rs, crates/rugrats/src/player.rs, crates/rugrats/src/game.rs, crates/rugrats/src/cine.rs, crates/rugrats/src/sound.rs, crates/rugrats/src/main.rs, crates/rugrats/src/lib.rs, crates/rugrats/examples/cine_list.rs, crates/rugrats/examples/opening_shot.rs, crates/rugrats/examples/level_shot.rs, crates/rugrats-hle/examples/call_args.rs
started: 2026-10-15T01:44:45-06:00
took: 12m
---

# 29. Characters, cinematics between levels, room music

Three more pieces of a playable game.

**Characters.** The player's four-track animation is now
`rugrats::animator::Animator`, loaded by character.

- 0x8008a42c's table pairs are read off its switch. Two point at
  run-time tables and are skipped.
- A placed character's character number is its template's shape, which
  is also its hierarchy (0x8004822c sets +0x15c from template +0x10).
- Placed characters play their stand state on the floor under them:
  root = floor + root offset, as 0x80081bac's query does. Lil in the hub
  stands instead of sinking into the floor.

**Cinematics.** `Stream::read` now reads every cine, not just the
opening, through the stream table at 0x800b4290 (u8 cine, s8 channel,
u16 file).

- A cine with a channel lives in one of the CINEPAK files. Its stream
  starts 2 × channel sectors in, its XA is filtered to (1, channel), and
  its data sectors are those whose first word is the channel, with the
  frame from +4 (0x80019da4, 0x80019c88).
- `cine_list` reads all of them. 91 stream and 2 have no file. Each
  stream's last frame time equals its sound length: Episode 1 Intro is
  79.5 s for both.
- Entering a level plays its start cine (0x80074958's table; the hub and
  level 2 only the first time). A door with a cine plays it on the
  current database, then the level loads.
- The Reptar Puzzle cine on DB01 matches the oracle's pictures.

**Room music.** The hub's music is not XA. The oracle (`call_args`, a
new tool printing a function's arguments per call) shows the level
setup opening song set 0x800b8198 (HOUSE.SEP), then song 2 from
0x8006efcc as play starts.

- The chain: RoomChange (0x80077a54) plays room +0x64 less one,
  crossfaded over 30, when the room has flag 128. Songs 5 and up are
  speech clips instead (0x8008da4c, the ISF).
- Room flags come from LP kind-6 records. Their data from +4 is: room,
  +0x5b, +0x5d, camera +0x5c, flags 32 and 16, +0x5f, and the song.
  The loader clears +0x0c first.
- The port loads 0x800b82e8[level]'s set on entering a level and plays
  the song of each room the player walks into. Levels 22 and up play
  set +0x07 at once (0x8006fea8).
- The hub start room 23 asks for song 2, as in the oracle.

**Still unknown:**

- The room lookup proper (0x8002b660 walks portals); the port takes
  the first room whose bounds hold the player.
- The crossfade (0x800a5bbc, 0x800a59f4) is not ported; songs switch at
  once.
