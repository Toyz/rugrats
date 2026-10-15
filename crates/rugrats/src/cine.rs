//! The in-engine cinematics (`play_cine`, SLUS_006.50:0x8007424c): a
//! database's rooms and objects, moved frame by frame by a stream of data
//! sectors with the XA soundtrack interleaved.
//!
//! The original streams at double speed and draws a frame as each data
//! sector arrives: one in five sectors, 30 frames a second. The port reads
//! the stream whole and keeps that schedule.

use std::sync::{Arc, Mutex};

use rrt::glam::{Mat3, Mat4, Vec3};
use rrt::kit::adpcm::xa;
use rugrats_data::cine::{self, Frame};
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::level::Level;
use rugrats_data::model::Models;
use rugrats_data::placement::{Placement, Template};
use rugrats_render::models::{model_triangles, posed_triangles, quat_matrix};
use rugrats_render::{Vram, Vtx};

use crate::disc::GameDisc;
use crate::face::{Blinks, Faces};
use crate::stage::Stage;
use crate::movie::Soundtrack;

/// A database's files, by the game's numbering (file + db).
const MOD: usize = 242;
const LEVEL: usize = 434;
const A_RAW: usize = 499;
const HIE: usize = 370;
const LP: usize = 402;
/// Shared by every database.
const CHARACTR_RAW: usize = 498;
const TEMPLATE: usize = 239;

/// Sectors a second at double speed, and ticks a second.
const SECTORS_PER_SECOND: u64 = 150;
const TICKS_PER_SECOND: u64 = 60;

/// Templates the LP loader (0x800454dc) passes over.
const SKIPPED: [u32; 6] = [1242, 1041, 1267, 834, 382, 625];

/// The cinematics' backdrop, which play_cine's setup (0x8007475c) asks for
/// with `set_backdrop` (0x8001d218).
pub const BACKDROP: [u8; 3] = [217, 175, 141];

/// The GTE's projection distance in cinematics, on the 512 x 240 screen.
const FOCAL: f32 = 178.0;

/// A database as a cinematic sees it: rooms, models, skeletons, and the
/// objects its LP file places.
pub struct Set {
    pub vram: Vram,
    /// The rooms, which do not move.
    pub rooms: Vec<Vtx>,
    pub models: Models,
    pub hierarchies: Vec<Option<Hierarchy>>,
    pub placements: Vec<Placement>,
    pub templates: Vec<Template>,
    faces: Faces,
    /// Each room's bounds (+0x10, +0x20), flags (+0x0c) and song (+0x64).
    pub room_info: Vec<RoomInfo>,
}

#[derive(Clone, Copy, Debug)]
pub struct RoomInfo {
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub flags: u32,
    pub song: u8,
}

impl Set {
    /// Database `db`'s files, as the loaders read them (rooms 0x8002a208,
    /// models 0x80044d00, hierarchies 0x80044e7c, placements 0x800454dc).
    pub fn load(disc: &GameDisc, db: usize) -> Result<Set, String> {
        let level: Level = rugrats_data::level::parse(&disc.file(LEVEL + db)?).map_err(|e| e.to_string())?;
        let mut vram = Vram::new();
        // load_rooms uploads A.RAW to (0, 256), 576 x 256; main loads the
        // characters' image at boot (0x800186d8) to (576, 256), 384 x 256.
        for (file, x) in [(A_RAW + db, 0), (CHARACTR_RAW, 576)] {
            let raw = disc.file(file)?;
            let words: Vec<u16> = raw.chunks_exact(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
            vram.load(x, 256, if x == 0 { 576 } else { 384 }, 256, &words);
        }
        Ok(Set {
            vram,
            rooms: rugrats_render::rooms::level_triangles(&level),
            models: Models::parse(&disc.file(MOD + db)?).map_err(|e| e.to_string())?,
            hierarchies: rugrats_data::hierarchy::parse_file(&disc.file(HIE + db)?).map_err(|e| e.to_string())?,
            placements: rugrats_data::placement::parse_lp(&disc.file(LP + db)?),
            templates: rugrats_data::placement::parse_templates(&disc.file(TEMPLATE)?),
            faces: Faces::load(disc, Faces::mask(disc, db)?)?,
            room_info: level
                .rooms
                .iter()
                .map(|r| {
                    let w = |at: usize| i32::from_le_bytes(r.record[at..at + 4].try_into().unwrap());
                    // The loader clears +0x0c; LP kind 6 sets it.
                    RoomInfo { min: [w(0x10), w(0x14), w(0x18)], max: [w(0x20), w(0x24), w(0x28)], flags: 0, song: 0 }
                })
                .collect(),
        })
        .map(|mut set: Set| {
            // LP kind 6 (0x800454dc): room properties, the record's data
            // bytes from +4: room, then +0x5b, +0x5d, the camera +0x5c,
            // flags 32 and 16, +0x5f, and the song (+0x64, flag 128).
            for p in set.placements.iter().filter(|p| p.kind == 6) {
                let a = p.template.to_le_bytes();
                let b = (p.pos[0] as u32).to_le_bytes();
                let Some(r) = set.room_info.get_mut(a[0] as usize) else { continue };
                for (k, (byte, flag)) in [(a[1], 2), (a[2], 4), (b[0], 32), (b[1], 16), (b[2], 64), (b[3], 128)].into_iter().enumerate() {
                    if byte != 0 {
                        r.flags |= flag;
                        if k == 5 {
                            r.song = byte;
                        }
                    }
                }
            }
            set
        })
    }

    /// Whether `id` names an object the LP loader made (0x800461f4 finds it).
    fn exists(&self, id: i16) -> bool {
        self.placements.iter().any(|p| p.kind == 1 && p.id as i16 == id && !SKIPPED.contains(&p.template))
    }

    /// Frame `f`'s faces into `vram`, in the order its records come: an
    /// actor's mouth and eyes go back to frame 0, then take the stream's
    /// (0x80073c34); a pose blinks, then takes the stream's (0x80073f08).
    pub fn faces(&self, f: &Frame, blinks: &mut Blinks, vram: &mut Vram) {
        for a in f.actors.iter().filter(|a| self.exists(a.id)) {
            self.faces.mouth(a.id, 0, vram);
            self.faces.eyes(a.id, 0, vram);
            if a.mouth != 0 {
                self.faces.mouth(a.id, a.mouth, vram);
            }
            if a.eyes != 0 {
                self.faces.eyes(a.id, a.eyes - 1, vram);
            }
        }
        for p in f.poses.iter().filter(|p| self.exists(p.id)) {
            let blink = blinks.step(p.id);
            self.faces.eyes(p.id, blink, vram);
            if p.mouth != 0 {
                self.faces.mouth(p.id, p.mouth, vram);
            }
            if p.eyes != 0 {
                self.faces.eyes(p.id, p.eyes - 1, vram);
            }
        }
    }

    /// The objects as frame `f` leaves them: placed models where the
    /// stream's actors move them (0x80073c34) or where LP put them, and the
    /// posed skeletons (0x80073f08).
    pub fn objects(&self, f: &Frame) -> Vec<Vtx> {
        self.objects_except(f, |_| false)
    }

    /// [`Set::objects`] without the placements `skip` takes (the
    /// characters, in play).
    pub fn objects_except(&self, f: &Frame, skip: impl Fn(usize) -> bool) -> Vec<Vtx> {
        let mut out = Vec::new();
        for (_, place) in self.placements.iter().enumerate().filter(|(k, p)| p.kind == 1 && !SKIPPED.contains(&p.template) && !skip(*k)) {
            let Some(t) = self.templates.get(place.template as usize) else { continue };
            // A placed object's rotation is LP's quaternion inverted
            // (0x8001f684) and kept as angles (0x800202b8); the matrix made
            // from them is quat_matrix(q) for the single-axis turns LP
            // holds, which the object transform (0x8001eb4c) transposes.
            let placed = quat_matrix(place.rotation.map(|v| v as i16)).transpose();
            if let Some(h) = t.hierarchy() {
                // A jointed object the stream does not pose stays at rest.
                let Some(Some(h)) = self.hierarchies.get(h) else { continue };
                if f.poses.iter().all(|p| p.id != place.id as i16) {
                    posed_triangles(h, &self.models, &[], placed, vec3(place.pos), &mut out);
                }
                continue;
            }
            // A model the search misses comes back as offset 0, and the
            // constructor (0x800494d8) then clears the object's draw.
            if self.models.find(t.shape as u32) == 0 {
                continue;
            }
            let Some(m) = self.models.get(t.shape as u32) else { continue };
            // An actor's quaternion goes to the object as 0x8001f9fc's
            // matrix, the transpose of quat_matrix, so it draws as
            // quat_matrix.
            let (pos, rot) = match f.actors.iter().find(|a| a.id == place.id as i16) {
                Some(a) => (a.pos, quat_matrix(a.q)),
                None => (place.pos, placed),
            };
            model_triangles(m, rot, vec3(pos), &mut out);
        }
        for pose in &f.poses {
            let Some(place) = self.placements.iter().find(|p| p.kind == 1 && p.id as i16 == pose.id) else { continue };
            let hierarchy = self.templates.get(place.template as usize).and_then(|t| t.hierarchy());
            let Some(Some(h)) = hierarchy.and_then(|i| self.hierarchies.get(i)) else { continue };
            let mut local = vec![None; h.parts.len()];
            for &(channel, q) in &pose.keys {
                // The hierarchy maps channels to parts (+0x14, 0xff none).
                let part = h.header.get(0x14 + channel as usize).copied().unwrap_or(0xff);
                if let Some(slot) = local.get_mut(part as usize) {
                    *slot = Some(quat_matrix(q));
                }
            }
            posed_triangles(h, &self.models, &local, Mat3::IDENTITY, vec3(pose.pos), &mut out);
        }
        out
    }
}

fn vec3(p: [i32; 3]) -> Vec3 {
    Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)
}

/// The view and projection of frame `f`'s camera. The stream's quaternion
/// goes through 0x80020580, the transpose of the parts' matrix.
pub fn camera(f: &Frame) -> Mat4 {
    crate::stage::view(vec3(f.camera_pos), quat_matrix(f.camera_q).transpose(), FOCAL)
}

/// A cinematic's stream: its frames, each with the tick (from the start)
/// it is due, and its soundtrack.
pub struct Stream {
    pub frames: Vec<(u64, Frame)>,
    pub sound: Vec<(i16, i16)>,
}

/// The cinematic table's stream channels (0x800b4290, 4 bytes a cine; +1
/// the channel, -1 for none). With one, the stream starts 2 sectors a
/// channel in and its data sectors carry a channel byte (the data is at
/// +16, not +12); without, the XA filter is file 1 channel 1.
const XA_FILE: u8 = 1;
const XA_CHANNEL: u8 = 1;
/// The cinematics' stream table (0x800b4290): u8 cine, s8 channel (-1
/// none), u16 file.
const STREAMS: u32 = 0x800b_4290;
/// Cinematics in the table (play_cine refuses 93 and up).
pub const CINES: usize = 93;

impl Stream {
    /// Cinematic `n`'s stream, as 0x80019da4 starts it. Without a channel
    /// it is file `n`, its XA file 1 channel 1 and its data sectors whole.
    /// With one it is the table's file from 2 sectors a channel in, its XA
    /// that channel, and its data sectors those whose first word is the
    /// channel, the frame from +4 (0x80019c88).
    pub fn read(disc: &GameDisc, n: usize) -> Result<Stream, String> {
        if n >= CINES {
            return Err(format!("Invalid cine number {n}"));
        }
        let entry = disc.exe_bytes(STREAMS + 4 * n as u32, 4)?;
        let channel = (entry[1] != 0xff).then_some(entry[1]);
        let (start, xa_channel) = match channel {
            Some(c) => (disc.file_lba(u16::from_le_bytes([entry[2], entry[3]]) as usize)? + 2 * c as u32, c),
            None => (disc.file_lba(n)?, XA_CHANNEL),
        };
        let mut frames = Vec::new();
        // The drive drops audio sectors the decoder is too busy for, so the
        // opening's repeated sectors play once.
        let mut xa = xa::Player::new(xa::DOUBLE_SPEED);
        let mut total = None;
        for lba in start.. {
            let s = disc.sector(lba)?;
            let raw = &s.raw[..];
            let at = u64::from(lba - start);
            let (file, chan, sub) = (raw[16], raw[17], raw[18]);
            // XA audio: Form 2, audio, real time; the filter picks one.
            if sub & 0x64 == 0x64 {
                if (file, chan) == (XA_FILE, xa_channel) {
                    xa.sector(at, xa::Coding(raw[19]), &raw[24..24 + xa::SECTOR_AUDIO]);
                }
                continue;
            }
            if sub & 0x08 == 0 {
                continue;
            }
            let d = match channel {
                Some(c) => {
                    if u32::from_le_bytes(raw[24..28].try_into().unwrap()) != c as u32 {
                        continue;
                    }
                    &raw[28..24 + 2048]
                }
                None => &raw[24..24 + 2048],
            };
            if total.is_none() {
                let h = cine::header(d).ok_or(format!("cine {n}: no stream header"))?;
                total = Some(h.frames as usize);
                continue;
            }
            let Some(f) = cine::frame(d) else { break };
            frames.push(((at + 1) * TICKS_PER_SECOND / SECTORS_PER_SECOND + 1, f));
            if Some(frames.len()) == total || lba - start > 200_000 {
                break;
            }
        }
        Ok(Stream { frames, sound: xa.sound })
    }
}

/// A cinematic playing.
pub struct Cine {
    pub set: Set,
    stream: Stream,
    started: u64,
    shown: Option<usize>,
    blinks: Blinks,
    /// What the frame shown leaves on screen: the rooms, the objects, the
    /// camera, the backdrop.
    pub stage: Stage,
}

impl Cine {
    /// Cinematic `n` on database `db`, its soundtrack handed to `audio`.
    pub fn open(disc: &GameDisc, n: usize, db: usize, now: u64, audio: &Arc<Mutex<Soundtrack>>) -> Result<Cine, String> {
        let set = Set::load(disc, db)?;
        let mut stream = Stream::read(disc, n)?;
        if let Ok(mut a) = audio.lock() {
            *a = Soundtrack::new(std::mem::take(&mut stream.sound));
        }
        let mut stage = Stage::new(set.vram.clone(), set.rooms.clone());
        stage.clear = BACKDROP;
        Ok(Cine { set, stream, started: now, shown: None, blinks: Blinks::new(n as u32), stage })
    }

    /// The stream frame shown, from 1.
    pub fn frame(&self) -> Option<u16> {
        self.shown.map(|k| self.stream.frames[k].1.number)
    }

    /// Advances to tick `now`; true when it shows a new frame, the point
    /// where play_cine samples the pads (0x8001d3e8) and tests for a skip.
    pub fn tick(&mut self, now: u64) -> bool {
        let t = now - self.started;
        let due = self.stream.frames.iter().take_while(|(d, _)| *d <= t).count();
        let Some(k) = due.checked_sub(1).filter(|&k| Some(k) != self.shown) else { return false };
        // The original reads every record, so faces and blinks take each
        // frame passed over too.
        for (_, f) in &self.stream.frames[self.shown.map_or(0, |s| s + 1)..=k] {
            self.set.faces(f, &mut self.blinks, &mut self.stage.vram);
        }
        self.stage.vram_rev += 1;
        let f = &self.stream.frames[k].1;
        self.stage.set_moving(self.set.objects(f));
        self.stage.camera = camera(f);
        self.shown = Some(k);
        true
    }

    /// Whether the last frame has been shown: play_cine stops when the
    /// stream's counter reaches the header's frame count.
    pub fn over(&self) -> bool {
        self.shown.is_some_and(|k| k + 1 >= self.stream.frames.len())
    }
}
