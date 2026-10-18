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
use rugrats_data::hierarchy::{self, Hierarchy};
use rugrats_data::level::{self, Level};
use rugrats_data::model::Models;
use rugrats_data::placement::{self, Placement, Template};
use rugrats_render::models::{model_triangles, posed_triangles, quat_matrix};
use rugrats_render::{Vram, Vtx, rooms};

use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::media::ending::{self, Ending};
use crate::media::face::{Blinks, Faces};
use crate::media::movie::Soundtrack;
use crate::stage::Stage;
use crate::ui::hud;
use crate::ui::text::{FONT_AT, FONT_FILE, FONT_SIZE, Fonts};

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

/// Templates the LP loader (0x800454dc) passes over (0x800455d0).
const SKIPPED: [u32; 6] = [1242, 1041, 1267, 834, 382, 625];
/// The kind a skipped object's placement is given (none the loaders read).
pub const NOT_LOADED: u32 = u32::MAX;

/// Whether the LP loader (0x800454dc at 0x800455d0) passes over a kind 1
/// placement of database `db`: the templates in `SKIPPED`, level 16's
/// 391-395, level 10's 403, and template 1211 at (38650, 884, -13520)
/// or (7900, 884, -2993).
fn loader_skips(db: usize, p: &placement::Placement) -> bool {
    SKIPPED.contains(&p.template)
        || (db == 16 && (391..=395).contains(&p.template))
        || (db == 10 && p.template == 403)
        || (p.template == 1211 && matches!(p.pos, [38650, 884, -13520] | [7900, 884, -2993]))
}

/// The cinematics' backdrop, which play_cine's setup (0x8007475c) asks for
/// with `set_backdrop` (0x8001d218).
pub const BACKDROP: [u8; 3] = [217, 175, 141];

/// The GTE's projection distance in cinematics, on the 512 x 240 screen,
/// where a cinematic's lens is out of range: the opening's (field of view
/// 955).
const FOCAL: f32 = 178.0;

/// Cinematic `n`'s projection distance (0x8007424c -> 0x8001dfb8): its
/// field of view, the first half-word of its row of the table at
/// 0x800b8bd4 (955 the opening's, 512 most others), as 160 over the
/// tangent of half of it (0xa0000 / tan in 4.12), SetGeomScreen's H.
pub fn focal(disc: &GameDisc, trig: &Trig, n: usize) -> f32 {
    let Ok(b) = disc.exe_bytes(0x800b_8bd4 + 16 * n as u32, 2) else { return FOCAL };
    let fov = i16::from_le_bytes([b[0], b[1]]) as i32;
    if !(2..2048).contains(&fov) {
        return FOCAL;
    }
    let half = fov / 2;
    let tan = ((trig.sin(half) << 12) / trig.cos(half).max(1)) as i16 as i32;
    if tan <= 0 { FOCAL } else { (0xa0000 / tan) as i16 as f32 }
}

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
    /// Each room's portals (+0x30 quads at +0x34): the room each leads to
    /// (corner 1's flags, quad +0x0e), those with corner 0's flag 2 left
    /// out, as the room lookup 0x8002b660 walks them.
    pub neighbours: Vec<Vec<usize>>,
}

#[derive(Clone, Copy, Debug)]
pub struct RoomInfo {
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub flags: u32,
    pub song: u8,
    /// The cinematic entering plays once (+0x5b, flag 2).
    pub cine: u8,
    /// The Gameplay script entering runs once (+0x5f, flag 64).
    pub script: u8,
    /// The camera entering switches to (+0x5d, flag 4) and its post
    /// (+0x5c).
    pub camera: u8,
    pub camera_post: u8,
}

impl Set {
    /// Database `db`'s files, as the loaders read them (rooms 0x8002a208,
    /// models 0x80044d00, hierarchies 0x80044e7c, placements 0x800454dc).
    pub fn load(disc: &GameDisc, db: usize) -> Result<Set, String> {
        let level: Level = level::parse(&disc.file(LEVEL + db)?).map_err(|e| e.to_string())?;
        let mut vram = Vram::new();
        // load_rooms uploads A.RAW to (0, 256), 576 x 256; main loads the
        // characters' image at boot (0x800186d8) to (576, 256), 384 x 256.
        for (file, x) in [(A_RAW + db, 0), (CHARACTR_RAW, 576)] {
            let raw = disc.file(file)?;
            let words: Vec<u16> = raw.as_chunks::<2>().0.iter().map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
            vram.load(x, 256, if x == 0 { 576 } else { 384 }, 256, &words);
        }
        // The fonts, which boot puts at (960, 256) for good (0x8002ed80).
        let (fx, fy) = FONT_AT;
        let (fw, fh) = FONT_SIZE;
        let font: Vec<u16> =
            disc.file(FONT_FILE)?.as_chunks::<2>().0.iter().map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
        vram.load(fx, fy, fw, fh, &font);
        Ok(Set {
            vram,
            rooms: rooms::level_triangles(&level),
            models: Models::parse(&disc.file(MOD + db)?).map_err(|e| e.to_string())?,
            hierarchies: hierarchy::parse_file(&disc.file(HIE + db)?).map_err(|e| e.to_string())?,
            // (The templates the loader passes over, 0x800455d0, are never
            // made: their placements kept, of no kind any loader reads.)
            placements: placement::parse_lp(&disc.file(LP + db)?)
                .into_iter()
                .map(|mut p| {
                    if p.kind == 1 && loader_skips(db, &p) {
                        p.kind = NOT_LOADED;
                    }
                    p
                })
                .collect(),
            templates: placement::parse_templates(&disc.file(TEMPLATE)?),
            faces: Faces::load(disc, Faces::mask(disc, db)?)?,
            neighbours: {
                let mut quads = level.quads.iter();
                level
                    .rooms
                    .iter()
                    .map(|r| {
                        (&mut quads)
                            .take(r.quads() as usize)
                            .filter(|q| q.corners[0].flags & 2 == 0)
                            .map(|q| q.corners[1].flags as usize)
                            .collect()
                    })
                    .collect()
            },
            room_info: level
                .rooms
                .iter()
                .map(|r| {
                    let w = |at: usize| i32::from_le_bytes(r.record[at..at + 4].try_into().unwrap());
                    // The loader clears +0x0c; LP kind 6 sets it.
                    RoomInfo {
                        min: [w(0x10), w(0x14), w(0x18)],
                        max: [w(0x20), w(0x24), w(0x28)],
                        flags: 0,
                        song: 0,
                        cine: 0,
                        script: 0,
                        camera: 0,
                        camera_post: 0,
                    }
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
                r.camera_post = a[3];
                for (k, (byte, flag)) in
                    [(a[1], 2), (a[2], 4), (b[0], 32), (b[1], 16), (b[2], 64), (b[3], 128)].into_iter().enumerate()
                {
                    if byte != 0 {
                        r.flags |= flag;
                        if k == 5 {
                            r.song = byte;
                        }
                        if k == 0 {
                            r.cine = byte;
                        }
                        if k == 4 {
                            r.script = byte;
                        }
                        if k == 1 {
                            r.camera = byte;
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
        for (_, place) in self
            .placements
            .iter()
            .enumerate()
            .filter(|(k, p)| p.kind == 1 && !SKIPPED.contains(&p.template) && !skip(*k))
        {
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

/// What a cine's shadows are drawn with: the GTE's tables, the floors
/// (COL) and the shadow's sprite (TEXPOS 261).
pub struct Shadows {
    trig: Trig,
    col: rugrats_data::col::Col,
    sprite: rugrats_data::level::Texture,
}

impl Shadows {
    pub fn load(disc: &GameDisc, db: usize) -> Result<Shadows, String> {
        let sprite = hud::Sprites::load(disc, db)?.texture(crate::world::game::shadow::SPRITE).copied().ok_or("no shadow sprite")?;
        Ok(Shadows { trig: Trig::load(disc)?, col: rugrats_data::col::Col::parse(&disc.file(466 + db)?)?, sprite })
    }
}

impl Set {
    /// The posed characters' shadows in frame `f` (0x8003414c, as play's):
    /// each posed from the stream's keys as the original's parts are, on
    /// the floor under where the stream puts it (+0x13c there: no object
    /// top is met in the cines run).
    pub fn shadows(&self, f: &Frame, s: &Shadows, out: &mut Vec<Vtx>) {
        for (_, q) in self.shadow_quads(f, s) {
            q.triangles(&s.sprite, out);
        }
    }

    /// [`Set::shadows`]' quads, by the posed object's id.
    pub fn shadow_quads(&self, f: &Frame, s: &Shadows) -> Vec<(i16, crate::world::game::Quad)> {
        use crate::world::game::shadow;
        let mut out = Vec::new();
        for pose in &f.poses {
            let Some(place) = self.placements.iter().find(|p| p.kind == 1 && p.id as i16 == pose.id) else { continue };
            let hierarchy = self.templates.get(place.template as usize).and_then(|t| t.hierarchy());
            let Some(Some(h)) = hierarchy.and_then(|i| self.hierarchies.get(i)) else { continue };
            let mut turns = vec![None; h.parts.len()];
            for &(channel, q) in &pose.keys {
                let part = h.header.get(0x14 + channel as usize).copied().unwrap_or(0xff);
                if let Some(slot) = turns.get_mut(part as usize) {
                    *slot = Some(q.map(|v| v as i32));
                }
            }
            let parts = crate::actors::pose::posed(&s.trig, h, &turns, [0; 3], pose.pos).0;
            let floor = shadow::tile_floor(&s.col, pose.pos, place.pos[1]);
            if let Some(q) = shadow::quad(&s.trig, h, &parts, floor) {
                out.push((pose.id, q));
            }
        }
        out
    }
}

fn vec3(p: [i32; 3]) -> Vec3 {
    Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)
}

/// The view and projection of frame `f`'s camera. The stream's quaternion
/// goes through 0x80020580, the transpose of the parts' matrix.
pub fn camera(f: &Frame, focal: f32) -> Mat4 {
    crate::stage::view(vec3(f.camera_pos), quat_matrix(f.camera_q).transpose(), focal)
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

/// Cinematic `n`'s music byte (table 0x800b8bd0 + 16 n, +7; 0x80074b80):
/// 32 pauses the song and fades it back in after, 8 lets it play on,
/// neither stops it (0x8007424c); 16 skips the screen's fade (0x80018ad0).
pub fn music_flags(disc: &GameDisc, n: usize) -> u8 {
    disc.exe_bytes(0x800b_8bd7 + 16 * n as u32, 1).ok().and_then(|b| b.first().copied()).unwrap_or(0)
}

/// A cinematic playing.
pub struct Cine {
    pub set: Set,
    stream: Stream,
    started: u64,
    shown: Option<usize>,
    blinks: Blinks,
    /// The opening's timed call (cine 0 only), and what it draws with.
    ending: Option<(Ending, Trig, Fonts)>,
    /// What the frame shown leaves on screen: the rooms, the objects, the
    /// camera, the backdrop.
    pub stage: Stage,
    /// Its lens's projection distance (`focal`).
    focal: f32,
    /// What its characters' shadows are drawn with (none in the opening's,
    /// whose init clears 0x800c3fc4).
    shadows: Option<Shadows>,
    /// Whether it fades out at its end, and how far it has.
    fades: bool,
    fade: i32,
}

impl Cine {
    /// Cinematic `n` on database `db`, its soundtrack handed to `audio`.
    pub fn open(
        disc: &GameDisc,
        n: usize,
        db: usize,
        now: u64,
        audio: &Arc<Mutex<Soundtrack>>,
    ) -> Result<Cine, String> {
        let set = Set::load(disc, db)?;
        let mut stream = Stream::read(disc, n)?;
        if let Ok(mut a) = audio.lock() {
            *a = Soundtrack::new(std::mem::take(&mut stream.sound));
        }
        let mut stage = Stage::new(set.vram.clone(), set.rooms.clone());
        stage.clear = BACKDROP;
        let ending =
            if n == ending::CINE { Some((Ending::new(), Trig::load(disc)?, Fonts::load(disc)?)) } else { None };
        let fades = music_flags(disc, n) & 1 == 0;
        let focal = focal(disc, &Trig::load(disc)?, n);
        let shadows = if n == ending::CINE { None } else { Shadows::load(disc, db).ok() };
        Ok(Cine {
            focal,
            shadows,
            set,
            stream,
            started: now,
            shown: None,
            blinks: Blinks::new(n as u32),
            ending,
            stage,
            fades,
            fade: 0,
        })
    }

    /// The stream's last frame: where it leaves the level's objects.
    pub fn last(&self) -> Option<&Frame> {
        self.stream.frames.last().map(|(_, f)| f)
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
            if let Some((e, ..)) = &mut self.ending {
                e.step(f.number);
            }
        }
        self.stage.vram_rev += 1;
        let f = &self.stream.frames[k].1;
        let mut moving = self.set.objects(f);
        if let Some(s) = &self.shadows {
            self.set.shadows(f, s, &mut moving);
        }
        self.stage.set_moving(moving);
        self.stage.camera = camera(f, self.focal);
        self.stage.overlay.clear();
        if let Some((e, trig, fonts)) = &self.ending
            && Ending::started(Some(f.number))
        {
            e.draw(&self.set.models, trig, fonts, &mut self.stage.overlay);
        }
        // The last 13 frames fade out, 25 a frame shown, unless the music
        // byte has 1 (0x8007424c).
        let last = self.stream.frames.last().map_or(0, |l| l.1.number);
        if self.fades && last.saturating_sub(f.number) < 13 {
            self.fade = (self.fade + 25).min(255);
            hud::fade(self.fade as u8, &mut self.stage.overlay);
        }
        self.shown = Some(k);
        true
    }

    /// Whether the last frame has been shown: play_cine stops when the
    /// stream's counter reaches the header's frame count.
    pub fn over(&self) -> bool {
        self.shown.is_some_and(|k| k + 1 >= self.stream.frames.len())
    }
}
