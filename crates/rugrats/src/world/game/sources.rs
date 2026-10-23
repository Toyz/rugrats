//! The looping sound sources (SLUS_006.50:0x8006fe04 starts one, at the
//! object set-up; 0x8006f88c runs them each frame): a source in hearing is
//! keyed on a voice of its own, 15 up (the free voices 0x800c4be8, 7 at a
//! level's start: 15, 16 and 17), turned by its distance and bearing from
//! the eye each frame, and keyed off out of hearing, its voice freed.
//!
//! What a source plays is the level's table 0x800b847c[level] at its sound
//! (s16 program, tone, note); its loudness the falloff of its own near, far
//! and volume (0x8006fc40 over 0x8006fd68 with its record), or with the
//! mono setting the positional sounds' falloff (0x800b8008).

use super::*;

/// The level's source sounds (0x800b847c: a pointer a level).
const TABLES: u32 = 0x800b_847c;
/// The first voice the sources take, and the free ones at a level's start.
const FIRST_VOICE: usize = 15;
const FREE_VOICES: u8 = 7;

/// A source as 0x8006f88c keeps it (0x800e3218, 40 bytes each).
#[derive(Clone, Copy, Debug)]
pub struct Source {
    /// +0x00 its sound: the program, tone and note it keys.
    pub key: [u8; 3],
    /// +0x0c its place; +0x1c, +0x20, +0x24 its falloff.
    pub pos: [i32; 3],
    near: i64,
    far: i64,
    volume: i64,
    /// +0x08: on (1 as started).
    pub on: bool,
    /// +0x04: the voice it holds.
    pub voice: Option<usize>,
}

/// What the sources ask of the SPU this frame (`SsUtKeyOnV`,
/// `SsUtSetVVol`, `SsUtKeyOffV`), for the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceCmd {
    On { voice: usize, key: [u8; 3], at: (i32, i32) },
    Turn { voice: usize, at: (i32, i32) },
    Off { voice: usize },
}

impl Game {
    /// The loader's and the set-up's sources in LP order (a random one's
    /// sound drawn from `rand`), then the level start's: level 4's
    /// (0x8007e058) added, level 10's (0x8007ee54) in place of them; with
    /// their sounds from the level's table.
    pub(super) fn sources_of(disc: &GameDisc, level: Level, set: &Set, rand: &mut Rand) -> Vec<Source> {
        let started = |sound: u8, pos: [i32; 3], near: i32, far: i32, volume: u8| crate::world::fixups::SoundSource { sound, pos, near, far, volume, random: false };
        let mut all: Vec<crate::world::fixups::SoundSource> = set
            .sound_sources
            .iter()
            .map(|s| if s.random { crate::world::fixups::SoundSource { sound: ((rand.next() & 255) % 3) as u8, ..*s } } else { *s })
            .collect();
        match level.0 {
            4 => all.push(started(2, [-83212, -688, 42873], 1024, 10000, 127)),
            10 => {
                all = vec![
                    started(0, [30714, -611, 14314], 993, 2990, 64),
                    started(1, [629, -1, -5204], 993, 2990, 64),
                    started(2, [37941, 76, 19831], 993, 2990, 64),
                ]
            }
            _ => {}
        }
        let table = disc
            .exe_bytes(TABLES + 4 * level.0 as u32, 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .unwrap_or(0);
        if table == 0 {
            return Vec::new();
        }
        all.iter()
            .filter_map(|s| {
                let e = disc.exe_bytes(table + 6 * s.sound as u32, 6).ok()?;
                let h = |i: usize| i16::from_le_bytes([e[i], e[i + 1]]) as u8;
                Some(Source {
                    key: [h(0), h(2), h(4)],
                    pos: s.pos,
                    near: s.near as i64,
                    far: s.far as i64,
                    volume: s.volume as i64,
                    on: true,
                    voice: None,
                })
            })
            .collect()
    }

    /// One frame of the sources (0x8006f88c), from the eye.
    pub(super) fn source_thinks(&mut self) {
        self.voice_cmds.clear();
        let (eye, _) = self.view;
        let audio = Progress { block: self.state_block }.audio();
        let (listener, facing) = self.listener();
        for i in 0..self.sources.len() {
            let s = self.sources[i];
            let d = [s.pos[0] - eye.x as i32, s.pos[1] - eye.y as i32, s.pos[2] - eye.z as i32];
            let at = if d[0].abs() > 0xfffe || d[1].abs() >= 0x7fff || d[2].abs() > 0xfffe {
                (0, 0)
            } else {
                let sq = |v: i32| (v as i64) * (v as i64);
                let dist = (sq(d[0]) >> 14) + (sq(d[1]) >> 12) + (sq(d[2]) >> 14);
                if audio.mono {
                    let v = falloff(dist, audio.effects);
                    (v, v)
                } else {
                    let v = source_falloff(dist, s.near, s.far, s.volume, audio.effects);
                    let angle = self.trig.bearing(listener, s.pos) - facing;
                    pan(&self.trig, angle.rem_euclid(4096), v)
                }
            };
            let heard = at != (0, 0);
            match (heard && s.on, s.voice) {
                (true, Some(voice)) => self.voice_cmds.push(VoiceCmd::Turn { voice, at }),
                (true, None) if self.free_voices != 0 => {
                    let bit = self.free_voices.trailing_zeros() as usize;
                    let voice = FIRST_VOICE + bit;
                    self.free_voices &= !(1 << bit);
                    self.sources[i].voice = Some(voice);
                    self.voice_cmds.push(VoiceCmd::On { voice, key: s.key, at });
                }
                (false, Some(voice)) => {
                    self.free_voices |= 1 << (voice - FIRST_VOICE);
                    self.sources[i].voice = None;
                    self.voice_cmds.push(VoiceCmd::Off { voice });
                }
                _ => {}
            }
        }
    }

    /// The free voices as the sources hold them (for tools).
    pub fn sync_source_voices(&mut self) {
        self.free_voices = FREE_VOICES;
        for v in self.sources.iter().filter_map(|s| s.voice) {
            self.free_voices &= !(1 << (v - FIRST_VOICE));
        }
    }

    /// The free source voices at a level's start (0x800c4be8).
    pub(super) const fn source_voices() -> u8 {
        FREE_VOICES
    }
}

/// A source's loudness at `d` (0x8006fd68 with its record): full within
/// `near`, none past `far`, `volume` at most, falling a step each
/// (far - near) / volume; no louder than the effects setting.
fn source_falloff(d: i64, near: i64, far: i64, volume: i64, setting: i32) -> i32 {
    let v = if d < near {
        volume
    } else if d > far {
        0
    } else {
        let step = if volume == 0 { 0 } else { (far - near) / volume };
        volume - if step == 0 { 0 } else { (d - near) / step }
    };
    (v as i32).min(setting).max(0)
}
