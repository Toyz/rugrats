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

use rrt::kit::adpcm::spu;
use rugrats_data::sep::Song;
use rugrats_data::vab::{Tone, Vab};

use crate::disc::GameDisc;
use crate::media::movie::Soundtrack;

pub const RATE: u32 = 44100;
const VOICES: usize = 24;
const MAIN_VOLUME: i32 = 0x3fff;
/// Voices the effects take in turn (0x8006f26c: 17 to 23).
const EFFECT_VOICES: std::ops::Range<usize> = 17..24;

/// A bank: its header and its samples decoded.
pub struct Bank {
    pub vab: Vab,
    samples: Vec<Arc<spu::Sample>>,
}

impl Bank {
    /// The bank whose body is file `vb` (its header is `vb + 1`).
    pub fn load(disc: &GameDisc, vb: usize) -> Result<Bank, String> {
        let vab = Vab::parse(&disc.file(vb + 1)?)?;
        let body = disc.file(vb)?;
        let samples = (0..vab.vag_sizes.len())
            .map(|n| Arc::new(vab.vag(&body, n).map(spu::decode_sample).unwrap_or_default()))
            .collect();
        Ok(Bank { vab, samples })
    }

    /// Program `p`'s tones.
    fn tones(&self, p: u8) -> &[Tone] {
        let mut at = 0;
        for (n, prog) in &self.vab.programs {
            if *n == p as usize {
                return self.vab.tones.get(at..at + prog.tones as usize).unwrap_or(&[]);
            }
            at += prog.tones as usize;
        }
        &[]
    }

    fn program_volume(&self, p: u8) -> (u8, u8) {
        self.vab.programs.iter().find(|(n, _)| *n == p as usize).map_or((127, 64), |(_, g)| (g.volume, g.pan))
    }
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
            music: Arc::new(Bank::load(disc, set.music)?),
            effects: Arc::new(Bank::load(disc, set.effects)?),
            songs: rugrats_data::sep::parse(&disc.file(set.sep)?)?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

/// The SPU's envelope (psx-spx "SPU ADSR Generator").
#[derive(Clone, Copy, Debug)]
struct Envelope {
    adsr1: u16,
    adsr2: u16,
    phase: Phase,
    level: i32,
    wait: u32,
}

impl Envelope {
    fn new(adsr1: u16, adsr2: u16) -> Envelope {
        Envelope { adsr1, adsr2, phase: Phase::Attack, level: 0, wait: 0 }
    }

    /// (exponential, decreasing, shift, step) of the phase.
    fn rate(&self) -> (bool, bool, u32, i32) {
        let (a1, a2) = (self.adsr1 as u32, self.adsr2 as u32);
        match self.phase {
            Phase::Attack => (a1 & 0x8000 != 0, false, a1 >> 10 & 31, 7 - (a1 >> 8 & 3) as i32),
            Phase::Decay => (true, true, a1 >> 4 & 15, -8),
            Phase::Sustain => {
                let down = a2 & 0x4000 != 0;
                let step = (a2 >> 6 & 3) as i32;
                (a2 & 0x8000 != 0, down, a2 >> 8 & 31, if down { -8 + step } else { 7 - step })
            }
            Phase::Release => (a2 & 0x20 != 0, true, a2 & 31, -8),
            Phase::Off => (false, true, 0, 0),
        }
    }

    fn step(&mut self) {
        if self.phase == Phase::Off {
            return;
        }
        if self.wait > 0 {
            self.wait -= 1;
            return;
        }
        let (exp, down, shift, step) = self.rate();
        let mut cycles = 1u32 << shift.saturating_sub(11);
        let mut step = step << 11u32.saturating_sub(shift);
        if exp && !down && self.level > 0x6000 {
            cycles *= 4;
        }
        if exp && down {
            step = step * self.level >> 15;
        }
        self.wait = cycles - 1;
        self.level = (self.level + step).clamp(0, 0x7fff);
        match self.phase {
            Phase::Attack if self.level >= 0x7fff => self.phase = Phase::Decay,
            Phase::Decay if self.level <= ((self.adsr1 as i32 & 15) + 1) * 0x800 => self.phase = Phase::Sustain,
            Phase::Release if self.level == 0 => self.phase = Phase::Off,
            _ => {}
        }
    }
}

#[derive(Clone)]
struct Voice {
    sample: Arc<spu::Sample>,
    /// Position in samples, 12 bits of fraction.
    pos: u64,
    pitch: u32,
    env: Envelope,
    vol: (i32, i32),
    /// The channel and note playing, for note-offs; None for an effect.
    key: Option<(u8, u8)>,
    started: u64,
}

impl Voice {
    fn sample_at(&self, i: i64) -> i16 {
        let pcm = &self.sample.pcm;
        let n = pcm.len() as i64;
        if i < 0 || n == 0 {
            return 0;
        }
        if i < n {
            return pcm[i as usize];
        }
        match self.sample.loop_start {
            Some(l) if (l as i64) < n => pcm[(l as i64 + (i - n) % (n - l as i64)) as usize],
            _ => 0,
        }
    }

    /// One output frame.
    fn render(&mut self) -> (i32, i32) {
        let i = (self.pos >> 12) as i64;
        let s = spu::interpolate(
            [self.sample_at(i - 3), self.sample_at(i - 2), self.sample_at(i - 1), self.sample_at(i)],
            (self.pos >> 4 & 255) as u8,
        );
        self.env.step();
        let v = s * self.env.level >> 15;
        self.pos += self.pitch.min(0x3fff) as u64;
        let n = self.sample.pcm.len() as u64;
        if self.sample.loop_start.is_none() && self.pos >> 12 >= n + 3 {
            self.env.phase = Phase::Off;
        }
        (v * self.vol.0 >> 15, v * self.vol.1 >> 15)
    }
}

#[derive(Clone, Copy, Debug)]
struct Channel {
    program: u8,
    volume: u8,
    pan: u8,
    expression: u8,
    bend: i32,
    nrpn: u8,
}

impl Default for Channel {
    fn default() -> Channel {
        Channel { program: 0, volume: 127, pan: 64, expression: 127, bend: 0, nrpn: 0 }
    }
}

/// A song playing (libsnd's SEQ interpreter, 0x800a6xxx).
struct Player {
    song: Song,
    pos: usize,
    status: u8,
    channels: [Channel; 16],
    /// Ticks until the next event, in 1/RATE ticks.
    wait: f64,
    tempo: u32,
    loop_at: Option<usize>,
    loop_count: Option<u8>,
    loops: u32,
    done: bool,
}

impl Player {
    fn delta(&mut self) -> u32 {
        let mut v = 0u32;
        while let Some(&b) = self.song.events.get(self.pos) {
            self.pos += 1;
            v = v << 7 | (b & 127) as u32;
            if b & 128 == 0 {
                break;
            }
        }
        v
    }
}

/// The software SPU and libsnd's sequencer.
pub struct Spu {
    voices: Vec<Option<Voice>>,
    music: Option<Arc<Bank>>,
    effects: Option<Arc<Bank>>,
    player: Option<Player>,
    next_effect: usize,
    clock: u64,
    /// The song volume (0-127), as `SsSeqSetVol`/`SsSepSetVol` set it.
    pub song_volume: i32,
}

impl Default for Spu {
    fn default() -> Spu {
        Spu {
            voices: vec![None; VOICES],
            music: None,
            effects: None,
            player: None,
            next_effect: EFFECT_VOICES.start,
            clock: 0,
            song_volume: 127,
        }
    }
}

/// libsnd's volume from note velocity, tone, program and channel volumes
/// and the pans combined (SsUtKeyOn and the SEQ note-on), as SPU voice
/// volumes (0-0x3fff).
fn volumes(parts: &[i32], pans: &[i32]) -> (i32, i32) {
    let v = parts.iter().fold(0x3fff, |v, &p| v * p / 127);
    // Pans 0-127, 64 the centre; each pulls the sound to one side.
    let pan = pans.iter().fold(64i32, |c, &p| (c + p - 64).clamp(0, 127));
    let l = v * (127 - pan).min(64) / 64;
    let r = v * pan.min(64) / 64;
    (l, r)
}

/// The SPU pitch for `note` on a tone sampled at its centre note: 0x1000
/// plays at 44100 Hz.
fn pitch(tone: &Tone, note: u8, bend: i32) -> u32 {
    let semis = note as f64 - tone.center as f64
        + tone.shift as f64 / 128.0
        + bend as f64 / 8192.0 * tone.pitch_bend_max.max(1) as f64;
    (4096.0 * 2f64.powf(semis / 12.0)).round().clamp(0.0, 0x3fff as f64) as u32
}

impl Spu {
    pub fn set_banks(&mut self, music: Arc<Bank>, effects: Arc<Bank>) {
        self.music = Some(music);
        self.effects = Some(effects);
    }

    /// Plays `song` (MidiPlaySong) at `volume` (0-127, 0x8006e74c): `loops`
    /// 0 forever.
    pub fn play(&mut self, song: &Song, loops: u8, volume: i32) {
        self.stop_song();
        self.song_volume = volume;
        let tempo = song.tempo.max(1);
        self.player = Some(Player {
            song: song.clone(),
            pos: 0,
            status: 0,
            channels: [Channel::default(); 16],
            wait: 0.0,
            tempo,
            loop_at: None,
            loop_count: (loops > 0).then_some(loops),
            loops: 0,
            done: false,
        });
        if let Some(p) = &mut self.player {
            p.wait = p.delta() as f64;
        }
    }

    pub fn stop_song(&mut self) {
        self.player = None;
        for v in self.voices.iter_mut() {
            if v.as_ref().is_some_and(|v| v.key.is_some()) {
                *v = None;
            }
        }
    }

    /// Sound effect `n` (0x8006f26c): the table's program, tone and note.
    pub fn effect(&mut self, table: &[[u8; 3]], n: usize, volume: i32) {
        let Some(&[prog, tone, note]) = table.get(n) else { return };
        let Some(bank) = self.effects.clone() else { return };
        let Some(t) = bank.tones(prog).get(tone as usize).cloned() else { return };
        let (pv, pp) = bank.program_volume(prog);
        let vol =
            volumes(&[t.volume as i32, pv as i32, volume, bank.vab.master_volume as i32], &[t.pan as i32, pp as i32]);
        let k = self.next_effect;
        self.next_effect = if k + 1 >= EFFECT_VOICES.end { EFFECT_VOICES.start } else { k + 1 };
        self.voices[k] = self.voice(&bank, &t, note, 0, vol, None);
    }

    fn voice(
        &self,
        bank: &Bank,
        t: &Tone,
        note: u8,
        bend: i32,
        vol: (i32, i32),
        key: Option<(u8, u8)>,
    ) -> Option<Voice> {
        let sample = bank.samples.get(t.vag as usize)?.clone();
        Some(Voice {
            sample,
            pos: 0,
            pitch: pitch(t, note, bend),
            env: Envelope::new(t.adsr1, t.adsr2),
            vol,
            key,
            started: self.clock,
        })
    }

    fn note_on(&mut self, c: Channel, ch: u8, note: u8, vel: u8) {
        let Some(bank) = self.music.clone() else { return };
        let (pv, pp) = bank.program_volume(c.program);
        let tones: Vec<Tone> =
            bank.tones(c.program).iter().filter(|t| (t.min_note..=t.max_note).contains(&note)).cloned().collect();
        for t in tones {
            let vol = volumes(
                &[
                    vel as i32,
                    t.volume as i32,
                    pv as i32,
                    c.volume as i32,
                    c.expression as i32,
                    self.song_volume,
                    bank.vab.master_volume as i32,
                ],
                &[t.pan as i32, pp as i32, c.pan as i32],
            );
            let v = self.voice(&bank, &t, note, c.bend, vol, Some((ch, note)));
            // A free voice below the effects', else the oldest.
            let k = (0..EFFECT_VOICES.start)
                .find(|&k| self.voices[k].as_ref().is_none_or(|v| v.env.phase == Phase::Off))
                .unwrap_or_else(|| {
                    (0..EFFECT_VOICES.start)
                        .min_by_key(|&k| self.voices[k].as_ref().map_or(0, |v| v.started))
                        .unwrap_or(0)
                });
            self.voices[k] = v;
        }
    }

    fn note_off(&mut self, ch: u8, note: u8) {
        for v in self.voices.iter_mut().flatten() {
            if v.key == Some((ch, note)) && v.env.phase != Phase::Off {
                v.env.phase = Phase::Release;
                v.env.wait = 0;
                v.key = Some((ch, 255));
            }
        }
    }

    /// Runs the song's events due now.
    fn sequence(&mut self) {
        let Some(mut p) = self.player.take() else { return };
        // Ticks a frame: resolution per quarter, tempo microseconds a quarter.
        p.wait -= p.song.resolution as f64 * 1e6 / p.tempo as f64 / RATE as f64;
        while p.wait <= 0.0 && !p.done {
            let Some(&b) = p.song.events.get(p.pos) else {
                p.done = true;
                break;
            };
            let status = if b & 0x80 != 0 {
                p.pos += 1;
                if b < 0xf0 {
                    p.status = b;
                }
                b
            } else {
                p.status
            };
            let arg = |p: &mut Player| {
                let v = p.song.events.get(p.pos).copied().unwrap_or(0);
                p.pos += 1;
                v
            };
            let ch = status & 15;
            match status & 0xf0 {
                0x90 => {
                    let (n, v) = (arg(&mut p), arg(&mut p));
                    if v == 0 {
                        self.note_off(ch, n);
                    } else {
                        self.note_on(p.channels[ch as usize], ch, n, v);
                    }
                }
                0x80 => {
                    let n = arg(&mut p);
                    arg(&mut p);
                    self.note_off(ch, n);
                }
                0xb0 => {
                    let (c, v) = (arg(&mut p), arg(&mut p));
                    let chan = &mut p.channels[ch as usize];
                    match c {
                        7 => chan.volume = v,
                        10 => chan.pan = v,
                        11 => chan.expression = v,
                        99 => chan.nrpn = v,
                        // libsnd's loop marks: NRPN 20 a loop start (data
                        // the count), 30 its end.
                        6 if chan.nrpn == 20 => {
                            p.loop_at = Some(p.pos);
                            p.loop_count = p.loop_count.or((v != 127 && v != 0).then_some(v));
                        }
                        6 if chan.nrpn == 30 => {
                            if let Some(at) = p.loop_at {
                                p.loops += 1;
                                if p.loop_count.is_none_or(|n| p.loops < n as u32) {
                                    p.pos = at;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                0xc0 => p.channels[ch as usize].program = arg(&mut p),
                0xd0 => {
                    arg(&mut p);
                }
                0xa0 => {
                    arg(&mut p);
                    arg(&mut p);
                }
                0xe0 => {
                    let (lo, hi) = (arg(&mut p) as i32, arg(&mut p) as i32);
                    p.channels[ch as usize].bend = (hi << 7 | lo) - 8192;
                }
                0xf0 if status == 0xff => {
                    let kind = arg(&mut p);
                    let len = arg(&mut p) as usize;
                    let at = p.pos;
                    p.pos += len;
                    match kind {
                        0x51 if len == 3 => {
                            let e = &p.song.events;
                            p.tempo = (e[at] as u32) << 16 | (e[at + 1] as u32) << 8 | e[at + 2] as u32;
                        }
                        0x2f => {
                            // The end: from the top again unless a count ran out.
                            p.loops += 1;
                            if p.loop_count.is_none_or(|n| p.loops < n as u32) {
                                p.pos = 0;
                                p.status = 0;
                            } else {
                                p.done = true;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            if !p.done {
                p.wait += p.delta() as f64;
            }
        }
        self.player = Some(p);
    }

    /// Fills `out` with stereo frames.
    pub fn render(&mut self, out: &mut [i16]) {
        for frame in out.chunks_exact_mut(2) {
            self.sequence();
            let (mut l, mut r) = (frame[0] as i32, frame[1] as i32);
            for v in self.voices.iter_mut() {
                let Some(voice) = v else { continue };
                let (a, b) = voice.render();
                l += a;
                r += b;
                if voice.env.phase == Phase::Off {
                    *v = None;
                }
            }
            // The main volume libsnd sets (SsSetMVol 127: 0x3fff).
            frame[0] = (frame[0] as i32 + ((l - frame[0] as i32) * MAIN_VOLUME >> 15)).clamp(-32768, 32767) as i16;
            frame[1] = (frame[1] as i32 + ((r - frame[1] as i32) * MAIN_VOLUME >> 15)).clamp(-32768, 32767) as i16;
            self.clock += 1;
        }
    }
}

/// The effects table (0x800b84fc): program, tone, note for each of 180.
pub fn effect_table(disc: &GameDisc) -> Result<Vec<[u8; 3]>, String> {
    Ok(disc.exe_bytes(0x800b_84fc, 3 * 180)?.chunks_exact(3).map(|e| [e[0], e[1], e[2]]).collect())
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
