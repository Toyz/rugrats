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
    /// Where the level ends: a level to go to (the exit code, 0x800c4918).
    pub exit: Option<usize>,
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
        // Everything placed but the player itself.
        let player_place = set.placements.iter().position(|p| mine(&p));
        let scenery = set.objects_except(&Frame::default(), player_place);
        let focal = trig.geom_screen(FOV).unwrap_or(256) as f32;
        let yaw = heading as f32;
        let mut g = Game { level, set, col, anims, player, stage, trig, focal, eye: Vec3::ZERO, yaw, scenery, exit: None };
        g.eye = g.eye_goal();
        g.draw();
        Ok(g)
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
        self.player.frame(pads, self.yaw.round() as i32, &self.col, &self.anims, &self.trig);
        // Using a door (the stand handler's ACTION, Triangle, on an object
        // whose use is 0x8004f78c): its level next.
        if pads.pressed(0, input::TRIANGLE) {
            let p = self.player.pos;
            let near = self.set.placements.iter().filter(|d| d.kind == 1 && d.level > 0).min_by_key(|d| (d.pos[0] - p[0]).pow(2) as i64 + (d.pos[2] - p[2]).pow(2) as i64);
            if let Some(d) = near.filter(|d| (d.pos[0] - p[0]).abs() < USE_REACH && (d.pos[2] - p[2]).abs() < USE_REACH && (d.pos[1] - p[1]).abs() < USE_REACH_Y) {
                rrt::tracing::info!("level {}: a door to level {} (cine {})", self.level, d.level, d.cine);
                self.exit = Some(d.level as usize);
            }
        }
        // Select: the restart menu (0x80043d58); for now, back to the hub.
        if pads.pressed(0, input::SELECT) {
            self.exit = Some(if self.level == HUB { HUB } else { HUB });
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
        let goal = self.eye_goal();
        self.eye += (goal - self.eye) / 8.0;
        self.draw();
    }

    fn draw(&mut self) {
        let mut moving = self.scenery.clone();
        self.player.triangles(&self.set.models, &self.anims, &self.trig, &mut moving);
        self.stage.set_moving(moving);
        let f = (self.target() - self.eye).normalize_or(Vec3::Z);
        let x = Vec3::Y.cross(f).normalize_or(Vec3::X);
        let y = f.cross(x);
        // World to camera: rows x (right), y (down), z (ahead).
        let rows = Mat3::from_cols(x, y, f).transpose();
        self.stage.camera = crate::stage::view(self.eye, rows, self.focal);
    }
}
