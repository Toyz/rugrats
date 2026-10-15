//! The full-motion movies (`movie_play`, SLUS_006.50:0x8008b448): the logos
//! at boot, the attract demos and the credits. Each is a `.STR` stream of
//! 320 x 240 frames shown in 24-bit colour, with its XA soundtrack
//! interleaved.
//!
//! The original shows a frame once its sectors have streamed in at double
//! speed (150 sectors a second) and checks the pads for a skip after each.
//! The port reads the stream whole and keeps that schedule.

use std::sync::{Arc, Mutex};

use rrt::image::Picture;
use rrt::kit::adpcm::xa;
use rugrats_data::movie::{self as str_, Frame, Frames};
use rugrats_data::{bs, mdec};

use crate::disc::GameDisc;

/// The movie table (0x800bae4c): frames shown, then the first sector.
pub const TABLE: u32 = 0x800b_ae4c;
pub const COUNT: usize = 7;

/// Sectors a second at double speed, and ticks a second.
pub const SECTORS_PER_SECOND: u64 = 150;
const TICKS_PER_SECOND: u64 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub frames: u32,
    pub lba: u32,
}

/// What the CD-ROM controller's XA-ADPCM decoder makes of a stream as the
/// drive reads it at double speed: a sector arriving while the decoder's
/// output still holds more than a few frames is dropped unread (the
/// hardware has no flow control; DuckStation models it this way). Streams
/// lean on that: the opening's encoder repeats each audio sector several
/// times to fill the bandwidth, and the decoder plays each once.
pub struct XaPlayer {
    decoder: xa::Decoder,
    /// The decoded sound at 44100 Hz, from the stream's first sector.
    pub sound: Vec<(i16, i16)>,
}

impl Default for XaPlayer {
    fn default() -> XaPlayer {
        XaPlayer { decoder: xa::Decoder::new(), sound: Vec::new() }
    }
}

impl XaPlayer {
    /// Frames the output may still hold when a sector is taken.
    const LOW_WATER: usize = 10;

    /// The audio sector read `at` sectors into the stream.
    pub fn sector(&mut self, at: u64, coding: u8, data: &[u8]) {
        let now = (at * 44100 / SECTORS_PER_SECOND) as usize;
        if self.sound.len() > now + Self::LOW_WATER {
            return;
        }
        if self.sound.len() < now {
            self.sound.resize(now, (0, 0));
        }
        self.decoder.decode(xa::Coding(coding), data, &mut self.sound);
    }
}

/// The soundtrack the audio device pulls: 44100 Hz stereo.
#[derive(Default)]
pub struct Soundtrack {
    samples: Vec<(i16, i16)>,
    pos: usize,
}

impl Soundtrack {
    /// 44100 Hz stereo samples, from the start.
    pub fn new(samples: Vec<(i16, i16)>) -> Soundtrack {
        Soundtrack { samples, pos: 0 }
    }
}

impl rrt::audio::Source for Soundtrack {
    fn rate(&self) -> u32 {
        44100
    }

    fn render(&mut self, out: &mut [i16]) {
        for frame in out.chunks_exact_mut(2) {
            let (l, r) = self.samples.get(self.pos).copied().unwrap_or((0, 0));
            self.pos += usize::from(self.pos < self.samples.len());
            frame[0] = l;
            frame[1] = r;
        }
    }
}

pub struct Movie {
    /// Each frame with the tick (from the start) it is due.
    frames: Vec<(u64, Frame)>,
    /// How many the table says to show.
    count: usize,
    skippable: bool,
    shown: Option<usize>,
    pub picture: Picture,
    started: u64,
}

impl Movie {
    /// Reads movie `n` from the disc, and hands its soundtrack to `audio`.
    pub fn open(
        disc: &GameDisc,
        n: usize,
        skippable: bool,
        now: u64,
        audio: &Arc<Mutex<Soundtrack>>,
    ) -> Result<Movie, String> {
        let e = disc.movies.get(n).ok_or(format!("no movie {n}"))?;
        let mut assembler = Frames::default();
        let mut frames = Vec::new();
        let mut xa = XaPlayer::default();
        let mut lba = e.lba;
        while frames.len() < e.frames as usize {
            let s = disc.sector(lba)?;
            let at = u64::from(lba - e.lba);
            lba += 1;
            let raw = &s.raw[..];
            // XA audio: Form 2, audio, real time.
            if raw[18] & 0x64 == 0x64 {
                xa.sector(at, raw[19], &raw[24..24 + xa::SECTOR_AUDIO]);
                continue;
            }
            if let Some(f) = str_::chunk(raw).and_then(|c| assembler.push(c)) {
                let due = (at + 1) * TICKS_PER_SECOND / SECTORS_PER_SECOND + 1;
                frames.push((due, f));
            }
        }
        if let Ok(mut a) = audio.lock() {
            *a = Soundtrack { samples: xa.sound, pos: 0 };
        }
        Ok(Movie {
            frames,
            count: e.frames as usize,
            skippable,
            shown: None,
            picture: Picture::filled(320, 240, [0, 0, 0, 255]),
            started: now,
        })
    }

    /// Advances to tick `now`; false once the movie is over.
    pub fn tick(&mut self, now: u64, skip: bool, tables: &mdec::Tables) -> bool {
        let t = now - self.started;
        let due = self.frames.iter().take_while(|(d, _)| *d <= t).count();
        if let Some(k) = due.checked_sub(1).filter(|&k| Some(k) != self.shown) {
            let f = &self.frames[k].1;
            match frame_picture(f, tables) {
                Ok(p) => self.picture = p,
                Err(e) => rrt::tracing::warn!("movie frame {}: {e}", f.number),
            }
            self.shown = Some(k);
        }
        let done = self.shown.is_some_and(|k| k + 1 >= self.count) && due >= self.count;
        !(done || (self.skippable && skip))
    }
}

/// A frame as the MDEC turns it out in 24-bit mode.
pub fn frame_picture(f: &Frame, tables: &mdec::Tables) -> Result<Picture, bs::BsError> {
    let (w, h) = (f.width as usize, f.height as usize);
    let words = mdec::decode(&bs::rle(&f.bs)?, tables, mdec::Depth::Bits24, false, false);
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    let mut pic = Picture::filled(w as u32, h as u32, [0, 0, 0, 255]);
    let per_col = h / 16;
    for (mb, px) in bytes.chunks(16 * 16 * 3).enumerate() {
        let (mx, my) = (mb / per_col * 16, mb % per_col * 16);
        for (i, rgb) in px.chunks(3).enumerate() {
            let (x, y) = (mx + i % 16, my + i / 16);
            if x < w && y < h && rgb.len() == 3 {
                pic.set(x as u32, y as u32, [rgb[0], rgb[1], rgb[2], 255]);
            }
        }
    }
    Ok(pic)
}
