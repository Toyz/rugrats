---
number: 45
title: Levels 14 and 15, the hub's gates, docs/levels
date: 2026-10-15
area: play
files: crates/rugrats/src/level15.rs, crates/rugrats/src/level9.rs, crates/rugrats/src/game.rs, crates/rugrats/src/hud.rs, crates/rugrats/src/main.rs, crates/rugrats-data/src/placement.rs, crates/rugrats-re/src/docs.rs, docs/levels/level-14.md, docs/levels/level-15.md
started: 2026-10-15T09:08:02-06:00
took: 1h 28m
---

# 45. Levels 14 and 15, the hub's gates, docs/levels

Two more delegated maps, and the level pages moved to their own folder.

**docs/levels** (asked by the user): levels 2-15 and golf moved out of
docs/engine. Links both ways and the code's doc paths followed.
rugrats-re's index has a "Levels" section.

**Level 14** (docs/levels/level-14.md): The Return of Mr. Friend is cut.
Nothing reaches it in retail:

- no hub door has 14;
- the Activities list is 13, 17, -2, 18;
- the bonus chain is 20, 15, 21, 17;
- the debug menu (0x8003bea0) is never enabled.

Nothing wins it either: no ending cine, no 0x80076d6c(14, ..) caller, no
puzzle piece; its 2-bar case is dead. The port runs it as it is, through
level 9's code with a level 14 flag:

- Mr. Friend follows and bites for 15;
- the kick (50) and throws stagger him every time;
- there are no sounds, and his death ends nothing.

**Level 15, Mirrorland** (`rugrats::level15`):

- Six ? boxes (207-212) in rooms 25, 19, 21, 5, 2, 45. Triangle beside
  one starts its colour:
  - the room picks the group and slot (0x8007d02c);
  - the colour's balloons go to the group's spots;
  - 90 s on the clock.
- Balloons taken count up (sound 113). The last stops the clock and
  fills the icon.
- Checkpoints come after the first and third colour; six play cine 72,
  won.
- The clock out is a cry. It returns Tommy to the respawn and undoes
  every colour opened since the last checkpoint. Tries are 0: never
  out.
- HUD: six icons (311 empty, 305-310 full by group) and the counter's
  balloon with the number left.

**Framework:**

- Placement +0x4a is kept apart from +0x48. Collectibles with many
  copies not placed at load wait for the level (the balloons).
- `Sprites::billboard`: a sprite as a camera-facing quad in the world,
  wound for the culled world pass. The balloons are sprites 299-304,
  240 x 300. Game keeps the last view and a frame count.
- The hub's gates on +0x30 bit 16, set at cine 73's start:
  - doors to 3, 4 and 6 show card 712 until then;
  - the mirror (templates 836, 881) becomes the door to 15.

Checked: box 210 opened beside it starts blue, 25 balloons over the
bedroom floor, 90 on the clock; the balloons show where they float and
hide behind walls.

**Still unknown:**

- The door name signs; the bob's speeds (random in the original).
- Balloons taken by walking only (the port's touch takes them in the
  air too).
