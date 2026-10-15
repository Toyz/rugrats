//! Playing a level (the runner 0x80017c0c, its frame 0x80018c38): the
//! database's rooms and objects on a stage, the collision under them, the
//! player, and the follow camera.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::cine::Frame;
use rugrats_data::col::Col;

use crate::cine::Set;
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::input::{self, Pads};
use crate::player::Player;
use crate::stage::Stage;

const COL: usize = 466;
const ANM: usize = 274;
const FRM: usize = 306;
const ROT: usize = 338;
/// Level starts (0x80017c0c): s32 x, y, z at 0x800b2940 + 16 level, u16
/// angles at 0x800b2b40 + 8 level, the first the heading.
const STARTS: u32 = 0x800b_2940;
const START_ANGLES: u32 = 0x800b_2b40;
/// The camera's field of view in play (0x800290a4: 612).
const FOV: i32 = 612;
/// The follow camera (0x80027304): it aims 768 above the player's origin
/// (0x800c3f58) from 4096 behind (0x800c4a4c) and 1024 up (0x800c3f5c),
/// and eases 1/8 of the way a frame.
const AIM_UP: f32 = 768.0;
const BEHIND: f32 = 4096.0;
const EYE_UP: f32 = 1024.0;
/// The default character: Tommy (0x800c5680 = 1), whose hierarchy is the
/// database's number 1 (0x800450ac).
const TOMMY: usize = 1;

pub struct Game {
    pub level: usize,
    pub set: Set,
    pub col: Col,
    pub anims: Anims,
    pub player: Player,
    pub stage: Stage,
    trig: Trig,
    focal: f32,
    /// The camera's eye and its heading (4096 a turn).
    eye: Vec3,
    yaw: f32,
    /// The rooms and objects at rest, drawn under the player.
    scenery: Vec<rugrats_render::Vtx>,
    /// Where the level ends: a level to go to (the exit code, 0x800c4918)
    /// and a cinematic to play first.
    pub exit: Option<(usize, Option<usize>)>,
    /// The placed objects' collision boxes in the world.
    pub boxes: Vec<crate::player::Box3>,
    /// The placed characters other than the player.
    pub npcs: Vec<Npc>,
    /// The room the player is in, and a song the room asked for
    /// (RoomChange, 0x80077a54: room flag 128, song at +0x64 less one).
    room: Option<usize>,
    pub song: Option<usize>,
    /// Sound effects asked for this frame, with their volume (0x8006f644:
    /// the effects setting, less with distance from the camera).
    pub effects: Vec<(u8, i32)>,
}

/// A placed character, standing (its think and AI are not ported).
pub struct Npc {
    pub hierarchy: rugrats_data::hierarchy::Hierarchy,
    pub animator: crate::animator::Animator,
    rot: Mat3,
    pos: Vec3,
}

/// How near (in x and z, and in y) the player must stand to use a door.
const USE_REACH: i32 = 1500;
const USE_REACH_Y: i32 = 1500;
/// The hub, Pickles' home.
pub const HUB: usize = 1;

impl Game {
    pub fn open(disc: &GameDisc, level: usize) -> Result<Game, String> {
        let set = Set::load(disc, level)?;
        let col = Col::parse(&disc.file(COL + level)?)?;
        let anims = Anims::parse(&disc.file(ANM + level)?, &disc.file(FRM + level)?, &disc.file(ROT + level)?).map_err(|e| e.to_string())?;
        let trig = Trig::load(disc)?;
        let start = disc.exe_bytes(STARTS + 16 * level as u32, 12)?;
        let start = [0, 4, 8].map(|k| i32::from_le_bytes(start[k..k + 4].try_into().unwrap()));
        let angles = disc.exe_bytes(START_ANGLES + 8 * level as u32, 2)?;
        let heading = u16::from_le_bytes([angles[0], angles[1]]) as i32 & 4095;
        // The placements loader (0x800454dc) moves the player onto the LP
        // record whose template is the character's hierarchy, turned by
        // its quaternion (0x8001f684, 0x80059a88).
        let mine = |p: &&rugrats_data::placement::Placement| {
            p.kind == 1 && set.templates.get(p.template as usize).is_some_and(|t| t.hierarchy() == Some(TOMMY))
        };
        let (start, heading) = match set.placements.iter().find(mine) {
            Some(p) => {
                let [_, y, _, w] = p.rotation.map(|v| v as f64);
                let yaw = (2.0 * y.atan2(w) / std::f64::consts::TAU * 4096.0).round() as i32;
                (p.pos, yaw.rem_euclid(4096))
            }
            None => (start, heading),
        };
        rrt::tracing::info!("level {level}: the player starts at {start:?}, heading {heading}");
        let hierarchy = set.hierarchies.get(TOMMY).cloned().flatten().ok_or("no hierarchy for Tommy")?;
        let player = Player::new(disc, hierarchy, start, heading, &col, &anims)?;
        let mut stage = Stage::new(set.vram.clone(), set.rooms.clone());
        stage.clear = [0, 0, 0];
        // The placed characters move on their own; the rest stands.
        let player_place = set.placements.iter().position(|p| mine(&p));
        let character = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1 && set.templates.get(p.template as usize).is_some_and(|t| t.hierarchy().is_some())
        };
        let scenery = set.objects_except(&Frame::default(), character);
        let mut npcs = Vec::new();
        for (k, p) in set.placements.iter().enumerate() {
            if !character(k) || Some(k) == player_place {
                continue;
            }
            let Some(h) = set.templates.get(p.template as usize).and_then(|t| t.hierarchy()) else { continue };
            let Some(Some(hierarchy)) = set.hierarchies.get(h).cloned() else { continue };
            let mut animator = crate::animator::Animator::load(disc, h as u16)?;
            animator.set(crate::player::anim::STAND, &anims);
            let rot = rugrats_render::models::quat_matrix(p.rotation.map(|v| v as i16)).transpose();
            rrt::tracing::debug!("npc {} hierarchy {h}: track {:?}, {} parts", p.id, animator.track(), hierarchy.parts.len());
            // On the floor under it (0x80081bac asks 0x8002b240), the root
            // the hierarchy's offset above the feet.
            let q = crate::collide::query(&col, p.pos);
            let feet = hierarchy.parts.first().map_or(0, |r| r.offset[1]);
            let y = if q.tile.is_some() { q.floor + feet } else { p.pos[1] };
            npcs.push(Npc { hierarchy, animator, rot, pos: Vec3::new(p.pos[0] as f32, y as f32, p.pos[2] as f32) });
        }
        let focal = trig.geom_screen(FOV).unwrap_or(256) as f32;
        let yaw = heading as f32;
        let boxes = object_boxes(&set, |k| character(k));
        let mut g = Game { level, set, col, anims, player, stage, trig, focal, eye: Vec3::ZERO, yaw, scenery, exit: None, boxes, npcs, room: None, song: None, effects: Vec::new() };
        g.eye = g.eye_goal();
        g.draw();
        Ok(g)
    }

    /// The player's room (0x8002b660, here by the rooms' bounds, the one
    /// it was in first); on a change, the room's song (0x80077a54).
    fn room_change(&mut self) {
        let p = self.player.pos;
        let inside = |k: usize| self.set.room_info.get(k).is_some_and(|r| (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i]));
        let room = match self.room.filter(|&k| inside(k)) {
            Some(k) => Some(k),
            None => (0..self.set.room_info.len()).find(|&k| inside(k)),
        };
        if room == self.room {
            return;
        }
        self.room = room;
        if let Some(r) = room.and_then(|k| self.set.room_info.get(k)) {
            if r.flags & 128 != 0 && (1..5).contains(&r.song) {
                self.song = Some(r.song as usize - 1);
            }
        }
    }

    /// Puts the camera behind heading `yaw` at once (for tools).
    pub fn set_camera_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.eye = self.eye_goal();
        self.draw();
    }

    /// The eye pulled in front of walls: walked from the aim point toward
    /// `goal` and stopped short of the first point not in the open (as
    /// 0x8002cd7c's "vectorwalk" keeps the camera inside).
    fn clear_eye(&self, goal: Vec3) -> Vec3 {
        let from = self.target();
        let steps = 32;
        let mut last = from;
        for k in 1..=steps {
            let p = from + (goal - from) * (k as f32 / steps as f32);
            let q = crate::collide::query(&self.col, [p.x as i32, p.y as i32, p.z as i32]);
            if q.kind != 0 {
                return from + (last - from) * 0.9;
            }
            last = p;
        }
        goal
    }

    fn target(&self) -> Vec3 {
        let p = self.player.pos;
        Vec3::new(p[0] as f32, p[1] as f32 - AIM_UP, p[2] as f32)
    }

    fn eye_goal(&self) -> Vec3 {
        let a = self.yaw / 4096.0 * std::f32::consts::TAU;
        self.target() - Vec3::new(a.sin(), 0.0, a.cos()) * BEHIND - Vec3::new(0.0, EYE_UP, 0.0)
    }

    /// One frame of the level (0x80018c38): the player, then the camera,
    /// then the draw.
    pub fn frame(&mut self, pads: &Pads) {
        self.player.frame(pads, self.yaw.round() as i32, &self.col, &self.boxes, &self.anims, &self.trig);
        let d = (self.eye - Vec3::new(self.player.pos[0] as f32, self.player.pos[1] as f32, self.player.pos[2] as f32)).length();
        let volume = (crate::sound::EFFECTS_VOLUME as f32 * (1.0 - d / 16384.0).max(0.0)) as i32;
        self.effects = self.player.effects.iter().map(|&e| (e, volume)).collect();
        // Using a door (the stand handler's ACTION, Triangle, on an object
        // whose use is 0x8004f78c): its level next.
        if pads.pressed(0, input::TRIANGLE) {
            let p = self.player.pos;
            let near = self.set.placements.iter().filter(|d| d.kind == 1 && d.level > 0).min_by_key(|d| (d.pos[0] - p[0]).pow(2) as i64 + (d.pos[2] - p[2]).pow(2) as i64);
            if let Some(d) = near.filter(|d| (d.pos[0] - p[0]).abs() < USE_REACH && (d.pos[2] - p[2]).abs() < USE_REACH && (d.pos[1] - p[1]).abs() < USE_REACH_Y) {
                rrt::tracing::info!("level {}: a door to level {} (cine {})", self.level, d.level, d.cine);
                self.exit = Some((d.level as usize, (d.cine > 0).then_some(d.cine as usize)));
            }
        }
        // Select: the restart menu (0x80043d58); for now, back to the hub.
        if pads.pressed(0, input::SELECT) {
            self.exit = Some((HUB, None));
        }
        // The camera buttons turn it by an eighth (L2, R2: 341 a press).
        if pads.pressed(0, input::L2) {
            self.yaw -= 341.0;
        }
        if pads.pressed(0, input::R2) {
            self.yaw += 341.0;
        }
        // Follow: swing behind the player as it runs, ease 1/8 a frame.
        if self.player.speed > 0 {
            let d = ((self.player.heading as f32 - self.yaw + 2048.0).rem_euclid(4096.0)) - 2048.0;
            self.yaw += d / 16.0;
        }
        self.yaw = self.yaw.rem_euclid(4096.0);
        for n in &mut self.npcs {
            n.animator.step(&self.anims);
        }
        self.room_change();
        let goal = self.clear_eye(self.eye_goal());
        self.eye += (goal - self.eye) / 8.0;
        self.draw();
    }

    fn draw(&mut self) {
        let mut moving = self.scenery.clone();
        self.player.triangles(&self.set.models, &self.anims, &self.trig, &mut moving);
        for n in &self.npcs {
            let locals = n.animator.locals(&n.hierarchy, &self.anims);
            rugrats_render::models::posed_triangles(&n.hierarchy, &self.set.models, &locals, n.rot, n.pos, &mut moving);
        }
        self.stage.set_moving(moving);
        let f = (self.target() - self.eye).normalize_or(Vec3::Z);
        let x = Vec3::Y.cross(f).normalize_or(Vec3::X);
        let y = f.cross(x);
        // World to camera: rows x (right), y (down), z (ahead).
        let rows = Mat3::from_cols(x, y, f).transpose();
        self.stage.camera = crate::stage::view(self.eye, rows, self.focal);
    }
}

/// The world boxes of the placed models (the player's placement left out):
/// each model box with the placement's rotation and position, and the
/// heights its corners span.
fn object_boxes(set: &Set, skip: impl Fn(usize) -> bool) -> Vec<crate::player::Box3> {
    let mut out = Vec::new();
    for (k, place) in set.placements.iter().enumerate() {
        if place.kind != 1 || skip(k) {
            continue;
        }
        let Some(t) = set.templates.get(place.template as usize) else { continue };
        if t.hierarchy().is_some() || set.models.find(t.shape as u32) == 0 {
            continue;
        }
        let Some(m) = set.models.get(t.shape as u32) else { continue };
        let rot = rugrats_render::models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        for &(lo, hi) in &m.boxes {
            let (mut a, mut b) = (f32::MAX, f32::MIN);
            for c in 0..8 {
                let v = Vec3::new(
                    if c & 1 == 0 { lo[0] } else { hi[0] } as f32,
                    if c & 2 == 0 { lo[1] } else { hi[1] } as f32,
                    if c & 4 == 0 { lo[2] } else { hi[2] } as f32,
                );
                let y = (rot * v + pos).y;
                a = a.min(y);
                b = b.max(y);
            }
            out.push(crate::player::Box3 {
                lo: lo.map(|v| v as i32),
                hi: hi.map(|v| v as i32),
                rot,
                pos,
                min_y: a as i32,
                max_y: b as i32,
            });
        }
    }
    out
}

/// A level's start cinematic (0x80074958).
pub fn start_cine(level: usize) -> Option<usize> {
    Some(match level {
        1 => 7,
        2 => 1,
        3 => 2,
        4 => 28,
        5 => 37,
        6 => 45,
        7 => 79,
        9 => 17,
        10 => 44,
        11 => 52,
        12 => 66,
        13 => 6,
        15 => 73,
        17 => 12,
        18 => 25,
        20 => 63,
        21 => 49,
        _ => return None,
    })
}
