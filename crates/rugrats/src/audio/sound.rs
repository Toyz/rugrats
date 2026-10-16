//! The game's music and sound effects: libsnd (SLUS_006.50:0x800a4710 on)
//! playing SEP songs and key-ons through VAB banks on the SPU's 24 voices,
//! here a software SPU.
//!
//! A song set (0x8006e96c, e.g. the menu's at 0x800b8030) names a music
//! bank, a sound-effect bank and a SEP file: s16 music VB (its VH the next
//! file), s16 effects VB, s16 SEP, u8 songs. `MidiPlaySong` (0x8006eac4)
//! plays one of the SEP's songs; an effect (0x8006f26c) keys on program,
//! tone and note from the table at 0x800b84fc on voices 17-23.

use std::sync::{Arc, Mutex};

use rrt::kit::libsnd::{self, sep::Song};
pub use rrt::kit::libsnd::{Bank, RATE};

use crate::disc::GameDisc;
use crate::media::movie::Soundtrack;

/// Voices the effects take in turn (0x8006f26c: 17 to 23).
const EFFECT_VOICES: std::ops::Range<usize> = 17..24;
/// The sequencer's voices (SsSetReservedVoice(14), 0x800ad588 in the
/// sound init 0x8006e7e8).
const SONG_VOICES: usize = 14;

/// The bank whose body is file `vb` (its header is `vb + 1`).
pub fn load_bank(disc: &GameDisc, vb: usize) -> Result<Bank, String> {
    Bank::new(&disc.file(vb + 1)?, &disc.file(vb)?)
}

/// Level `level`'s song set (0x800b82e8, a pointer a level), and the song
/// 0x8006fea8 plays at once for levels 22 and up (set +0x07).
pub fn level_set(disc: &GameDisc, level: usize) -> Option<(SongSet, Option<usize>)> {
    let p = disc.exe_u32(0x800b_82e8 + 4 * level as u32).ok().filter(|&p| p != 0)?;
    let b = disc.exe_bytes(p, 8).ok()?;
    let h = |k: usize| i16::from_le_bytes([b[k], b[k + 1]]) as usize;
    let set = SongSet { music: h(0), effects: h(2), sep: h(4) };
    Some((set, (level >= 22).then_some(b[7] as usize)))
}

/// The menu's song set (0x800b8030).
pub const MENU_SET: SongSet = SongSet { music: 109, effects: 151, sep: 230 };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SongSet {
    pub music: usize,
    pub effects: usize,
    pub sep: usize,
}

/// A song set loaded.
pub struct Loaded {
    pub set: SongSet,
    pub music: Arc<Bank>,
    pub effects: Arc<Bank>,
    pub songs: Vec<Song>,
}

impl Loaded {
    pub fn load(disc: &GameDisc, set: SongSet) -> Result<Loaded, String> {
        Ok(Loaded {
            set,
            music: Arc::new(load_bank(disc, set.music)?),
            effects: Arc::new(load_bank(disc, set.effects)?),
            songs: libsnd::sep::parse(&disc.file(set.sep)?)?,
        })
    }
}

/// The game's sound on libsnd (rrt's software SPU): one song playing and
/// one fading out under it after a room's switch (0x8006efcc keeps both,
/// +0xa and +0xb), effects on voices 17-23.
pub struct Spu {
    spu: libsnd::Spu,
    /// The slot playing now; the other fades out.
    current: usize,
    /// Each slot's song number in its set.
    ids: [Option<usize>; 2],
}

impl Default for Spu {
    fn default() -> Spu {
        let mut spu = libsnd::Spu::new(2, EFFECT_VOICES);
        spu.set_reserved_voices(SONG_VOICES);
        Spu { spu, current: 0, ids: [None; 2] }
    }
}

impl Spu {
    pub fn set_banks(&mut self, music: Arc<Bank>, effects: Arc<Bank>) {
        self.spu.set_banks(music, effects);
    }

    /// Plays `song`, number `id` of its set (MidiPlaySong), at `volume`
    /// (0-127, 0x8006e74c): `loops` 0 forever. Whatever played stops.
    pub fn play(&mut self, song: &Song, id: usize, loops: u8, volume: i32) {
        self.stop_song();
        self.spu.play(self.current, song, loops, volume);
        self.ids[self.current] = Some(id);
    }

    /// A room's song (0x8006efcc with time 30): the song playing fades
    /// out while `song` fades in, both over `ticks` libsnd ticks: 60 a
    /// second, SsSetTickMode(SS_NOTICK) with SsSeqCalledTbyT called from
    /// the VSync callback (0x800192b4). Nothing when `id` is the song playing or the one
    /// fading out; a fade under way drops its old song.
    pub fn switch_song(&mut self, song: &Song, id: usize, volume: i32, ticks: u32) {
        if (0..2).any(|s| self.ids[s] == Some(id) && self.spu.playing(s)) {
            return;
        }
        if !self.spu.playing(self.current) {
            self.play(song, id, 0, volume);
            return;
        }
        let frames = ticks * RATE / 60;
        let (old, new) = (self.current, 1 - self.current);
        self.spu.stop(new);
        self.spu.fade(old, 0, frames, true);
        self.spu.play(new, song, 0, 0);
        self.spu.fade(new, volume, frames, false);
        self.ids[new] = Some(id);
        self.current = new;
    }

    /// The playing song's volume now (SsSeqSetVol, the Audio menu).
    pub fn set_song_volume(&mut self, volume: i32) {
        self.spu.set_volume(self.current, volume);
    }

    pub fn stop_song(&mut self) {
        self.spu.stop_all();
        self.ids = [None; 2];
    }

    /// Sound effect `n` (0x8006f26c): the table's program, tone and note,
    /// at volumes (left, right), 0-127 (SsUtKeyOn's voll and volr).
    pub fn effect(&mut self, table: &[[u8; 3]], n: usize, at: (i32, i32)) {
        if let Some(&[prog, tone, note]) = table.get(n) {
            self.spu.key_on(prog, tone, note, at);
        }
    }

    /// Mixes into `out` (stereo frames).
    pub fn render(&mut self, out: &mut [i16]) {
        self.spu.render(out);
    }
}

/// The effects table (0x800b84fc): program, tone, note for each of 180.
pub fn effect_table(disc: &GameDisc) -> Result<Vec<[u8; 3]>, String> {
    Ok(disc.exe_bytes(0x800b_84fc, 3 * 180)?.as_chunks::<3>().0.iter().map(|e| [e[0], e[1], e[2]]).collect())
}

/// The settings' music and effects volumes (0x800c5684, 0x800c5688) as
/// the game starts them.
pub const MUSIC_VOLUME: i32 = 90;
pub const EFFECTS_VOLUME: i32 = 100;

/// What the audio device plays: the streamed XA soundtrack with the SPU's
/// voices over it.
pub struct Mix {
    pub track: Arc<Mutex<Soundtrack>>,
    pub spu: Arc<Mutex<Spu>>,
}

impl rrt::audio::Source for Mix {
    fn rate(&self) -> u32 {
        RATE
    }

    fn render(&mut self, out: &mut [i16]) {
        out.fill(0);
        if let Ok(mut t) = self.track.lock() {
            t.render(out);
        }
        if let Ok(mut s) = self.spu.lock() {
            s.render(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    /// A room's switch (0x8006efcc): the old song fades out under the new
    /// one fading in, and stops at 0; asking again for either does nothing.
    #[test]
    fn a_room_switch_crossfades() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc");
        let Some(disc) = rrt::disc::Image::find(&dir).ok().and_then(|c| GameDisc::open(&c).ok()) else { return };
        let set = Loaded::load(&disc, MENU_SET).unwrap();
        assert!(set.songs.len() >= 2);
        let mut spu = Spu::default();
        spu.set_banks(set.music.clone(), set.effects.clone());
        spu.play(&set.songs[0], 0, 0, 90);
        let mut out = vec![0i16; 2 * RATE as usize / 10];
        spu.render(&mut out);
        spu.switch_song(&set.songs[1], 1, 90, 30);
        let ids = |s: &Spu| {
            let (a, b) = (s.current, 1 - s.current);
            [a, b].map(|k| s.ids[k].filter(|_| s.spu.playing(k)))
        };
        assert_eq!(ids(&spu), [Some(1), Some(0)]);
        // Halfway (15 of 30 ticks): both near 45.
        let mut half = vec![0i16; 2 * RATE as usize / 4];
        spu.render(&mut half);
        let vols = [spu.current, 1 - spu.current].map(|k| spu.spu.volume(k).unwrap_or(-1.0));
        assert!((vols[0] - 45.0).abs() < 2.0 && (vols[1] - 45.0).abs() < 2.0, "{vols:?}");
        // Asking for the fading song keeps things as they are.
        spu.switch_song(&set.songs[0], 0, 90, 30);
        assert_eq!(ids(&spu), [Some(1), Some(0)]);
        spu.render(&mut half);
        assert_eq!(ids(&spu), [Some(1), None]);
        assert_eq!(spu.spu.volume(spu.current), Some(90.0));
    }
}
