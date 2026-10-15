---
number: 63
title: The port restructured into idiomatic Rust
date: 2026-10-15
area: design
files: crates/rugrats/src/lib.rs crates/rugrats/src/main.rs crates/rugrats/src/app crates/rugrats/src/levels crates/rugrats/src/world crates/rugrats/src/actors crates/rugrats/src/ui crates/rugrats/src/audio crates/rugrats/src/media crates/rugrats/src/save
started: 2026-10-15T17:18:13-06:00
took: 34m
---

# 63. The port restructured into idiomatic Rust

The port's layout followed the original's call structure: level*.rs flat in the crate root, a single Game file of 2500 lines, tuples and bare numbers. It is now laid out by subsystem and typed. Behaviour is unchanged. The play tests (tests/play.rs) were run after every step, and a headless boot to the hub gives the same frame byte for byte.

- **Modules:**
  - actors (player, npc, animator)
  - audio (sound, speech, ambience)
  - levels (one file a level, named for it: hub, grandpas_teeth ... golf; pie_toss shared by 3 and 16)
  - media (movie, still, cine, face)
  - save (card, progress)
  - ui (front, screens, hud, text, tips, card_menu, audio_menu, golf_setup)
  - world (collide, fx, game/)
  - app (the boot)
- **world::game** is split into children: doors, toys, movers, things, goodies, camera, rooms, boxes, pause, story. mod.rs keeps Game, the Script trait, opening a level and its frame and draw.
- **app** is the boot as a by-value state machine. Each Boot state's tick takes the state and returns the next (flow.rs), so the `mem::replace` / `unreachable!` pattern is gone. play.rs is a level in play and its exits; golf_round.rs is GolfRound. main.rs is only the CLI.
- **Types instead of tuples:**
  - Exit { level, cine }, Effect { sound, volume }
  - Pose { pos, heading } (checkpoints, respawns, NPC targets)
  - Talker, Hazard, Strike, Collected
  - speech Line, toy_palace Burst, outer_space SlidingDoor
- **levels::Level** is a newtype over the level number. It has named consts (HUB, GRANDPAS_TEETH ... GOLF_DOOR, FIRST_HOLE) and the rules the engine keys on as methods:
  - is_hole, is_box_toss, is_episode, has_piece, no_damage, has_meter
  - record (25 reads as 8), character, bit, index
  - Progress, Exit, Game, the app and the loading pages take Level. Table lookups by number (start_cine, ambience, the loading text) match on `level.0`.
- **Level scripts:** phase is a per-file `enum Phase` (#[repr(u8)], the original numbers of 0x800c4cd0). Chuckie's Glasses keeps `races_lost`, since its phase only counts lost races.
- **Player::hurt/heal and Npc::hurt** replace the health arithmetic at 23 sites.
- **Imports** replace about 400 inline `crate::a::b::X` and `rugrats_data::`/`rugrats_render::` paths. Clippy's machine fixes are applied: explicit shift precedence, collapsed ifs, as_chunks, is_multiple_of.
- **Doc paths** that named old files (front.rs, game.rs, main.rs) now point to the new ones.

**Still unknown:**
- Game still has many pub fields that scripts reach into. A narrower script interface (a World/Camera/Hud split) is not done.
- Sound effects are still played by number (`g.play(71)`). Their names are not traced, so there are no named consts yet.
- rugrats-data's chunks_exact lints are left; as_chunks there changes the parsers' signatures.
