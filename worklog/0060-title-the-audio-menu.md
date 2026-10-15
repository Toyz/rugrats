---
number: 60
title: Title: the Audio menu
date: 2026-10-15
area: ui
files: crates/rugrats/src/audio_menu.rs crates/rugrats/src/front.rs crates/rugrats/src/main.rs crates/rugrats/src/progress.rs crates/rugrats/src/input.rs
started: 2026-10-15T15:57:51-06:00
took: 16m
---

# 60. Title: the Audio menu

The title's Audio menu is ported, from the front-end.md section an agent traced and checked in the oracle.

- **rugrats::audio_menu::AudioMenu:**
  - Settings: music, effects, mono.
  - The pads are ignored for the first 20 frames.
  - Up and Down/Select wrap the cursor, with sounds 4 and 3.
  - A held Left or Right moves a volume by 1 a frame, within 0-100. The music volume goes to the song as it changes (Spu::song_volume, SsSeqSetVol).
  - Releasing Left or Right on Sfx plays the test sound 5. Left or Right on the stereo item flips it, with no sound.
  - Cross/Start keeps the values (2); Triangle restores the entry values (1).
- **Blocks** ease position and angles as 0x8003f7f0 does:
  - boombox 335;
  - meters 339;
  - needles 340, at 21 v - 1024 within +-1024, turning at 128;
  - arrow blocks 334;
  - speakers 343, on the stereo pair at entry and turned by a flip.
- **Labels** slide in with their shadows. A shadow steps only while its text is moving, so Sfx's shadow ends one pixel short as in the oracle. A flip puts the label at (40 or 60, 75).
- **The Title** runs it as Menu::Audio:
  - The backdrop's fade steps twice a frame.
  - CANCEL shows at once and ACCEPT after the intro.
  - At the end the title's items return with Audio chosen.
- **Settings and sound:**
  - Progress::audio/set_audio keep the settings in the state block (+0x0c/+0x10/+0x14), so saves carry them.
  - Main plays songs at the music setting and effects at the effects setting, which scales the positional volume.
  - Pads::released is new.
- **Checked:** the real binary, driven headless, opened the menu from the title. Holding Left turned the music needle, Down moved the cursor, and Right on the stereo item showed MONO with the speakers turned. Screenshots taken.

**Still unknown:**
- Mono's single volume for positional sounds is not ported; the port's effects have no pan.
- Card-boot restore of the settings (0x80070c24) is not ported; they load with a saved game only.
- The meters and needles use the generic draw, not 0x80042d04's back colour.
