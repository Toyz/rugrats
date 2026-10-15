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

/// The follow camera's height by level (0x800290a4, 0x800291f8: 512 in
/// levels 4 and 15, -1000 in 7).
fn default_eye_up(level: usize) -> f32 {
    match level {
        4 | 15 => 512.0,
        7 => -1000.0,
        _ => EYE_UP,
    }
}
/// The default character: Tommy (0x800c5680 = 1), whose hierarchy is the
/// database's number 1 (0x800450ac).
const TOMMY: usize = 1;

/// A level's own play (its EpisodeStart callback and the level cases of
/// the event handlers).
pub trait Script {
    /// One frame, after the player and the NPCs moved.
    fn frame(&mut self, g: &mut Game, pads: &Pads);
    /// A cinematic it or a room asked for has played (CineFinish).
    fn cine_finished(&mut self, _g: &mut Game, _n: usize) {}
    /// After a cry (ForceCry's level case); false leaves it to the
    /// default respawn.
    fn cry(&mut self, _g: &mut Game) -> bool {
        false
    }
    /// A room's Gameplay script (room flag 64, 0x80077ee4).
    fn gameplay(&mut self, _g: &mut Game, _n: u8) {}
    /// The level's own models and HUD.
    fn draw(&self, _g: &Game, _moving: &mut Vec<rugrats_render::Vtx>, _overlay: &mut Vec<rugrats_render::Vtx>) {}
    /// The level timer ran out (TimerExpired's level callback).
    fn timer_expired(&mut self, _g: &mut Game) {}
    /// The phase, for tools.
    fn phase(&self) -> u8 {
        0
    }
    /// The level's own state in words, for tools.
    fn describe(&self) -> String {
        String::new()
    }
}

pub struct Game {
    pub level: usize,
    pub set: Set,
    pub col: Col,
    pub anims: Anims,
    pub player: Player,
    pub stage: Stage,
    pub trig: Trig,
    focal: f32,
    /// The camera's eye and its heading (4096 a turn).
    pub eye: Vec3,
    pub yaw: f32,
    /// The rooms and objects at rest, drawn under the player.
    scenery: Vec<rugrats_render::Vtx>,
    /// Where the level ends: a level to go to (the exit code, 0x800c4918)
    /// and a cinematic to play first.
    pub exit: Option<(usize, Option<usize>)>,
    /// A cinematic the level asks for (PostCine), played with the level
    /// waiting.
    pub cine: Option<usize>,
    /// The last checkpoint reached (position, heading).
    pub checkpoint: Option<([i32; 3], i32)>,
    /// Tries left (state +0x23, from 0x800b2920[level] at the start), and
    /// the cry's time left (0x8007f920: up to 2100 dt).
    pub tries: i32,
    crying: i32,
    /// The level's own play, and how a level ended (0x80076d6c's won).
    pub script: Option<Box<dyn Script>>,
    pub won: Option<bool>,
    /// The HUD's sprites, and the health meter's time left and last value.
    pub sprites: Option<crate::hud::Sprites>,
    meter_shown: i32,
    meter_health: i32,
    /// The placed objects' collision boxes in the world.
    pub boxes: Vec<crate::player::Box3>,
    /// The placed characters other than the player.
    pub npcs: Vec<Npc>,
    /// The posts (LP kind 2), what the NPCs asked for this frame, and the
    /// level's rand.
    pub posts: Vec<crate::npc::Post>,
    pub npc_events: Vec<crate::npc::Event>,
    pub rand: crate::rand::Rand,
    /// The room the player is in, and a song the room asked for
    /// (RoomChange, 0x80077a54: room flag 128, song at +0x64 less one).
    room: Option<usize>,
    pub song: Option<usize>,
    /// Sound effects asked for this frame, with their volume (0x8006f644:
    /// the effects setting, less with distance from the camera).
    pub effects: Vec<(u8, i32)>,
    /// Paused (Start, 0x80018c38 step 13): the world holds still and the
    /// pause menu runs (0x8003e05c).
    pub paused: bool,
    pause: Pause,
    pub fonts: crate::text::Fonts,
    /// Quit to the front end (the exit code -1).
    pub quit: bool,
    /// The objects that can be picked up, the one carried, and its grip
    /// (+0x1f8: in the player's frame from hand slot 15).
    pub things: Vec<Thing>,
    /// Bars, cookies and the levels' other collectibles; the bars
    /// available (0x800c4194: found less spent, kept in step here)
    /// here (0x80094a58) and how long the counter shows.
    pub goodies: Vec<Goodie>,
    pub bars: u32,
    bar_shown: i32,
    /// Bars found this frame (0x80094a58), and a save asked for.
    pub found: Vec<usize>,
    pub save: bool,
    pub load: bool,
    carried: Option<usize>,
    grip: Vec3,
    /// Objects hidden by id (0x80078bc4) and the placements always drawn
    /// apart (characters, pickables, collectibles).
    hidden: std::collections::HashSet<u16>,
    apart: Vec<bool>,
    /// Placements whose use was taken away (+0x54 = 0).
    unusable: std::collections::HashSet<usize>,
    /// Thrown objects that struck a character this frame (0x80086ef0):
    /// (character, thing, the thing's +0x6d).
    pub struck: Vec<(usize, usize, i32)>,
    /// A fixed camera (eye, look-at) a level sets (0x800291f8 kind 4).
    pub fixed: Option<(Vec3, Vec3)>,
    /// The follow camera's height (0x800c3f5c) and distance (0x800c4a4c),
    /// and the post a room's fixed camera stands at (kind 3, 0x800259a0).
    eye_up: f32,
    behind: f32,
    post_camera: Option<usize>,
    /// Objects that hurt on touch (not pickable, not characters, +0x6d
    /// over 0: Damage(player, 0, +0x6d), 0x8005e1f4), and the time until
    /// the next hurt.
    hazards: Vec<(crate::player::Box3, i32)>,
    hurt_wait: i32,
    /// The level timer (0x800c4430, dt left), set by a level.
    pub timer: Option<i32>,
    /// The level a finish is recorded for (golf records level 8).
    pub record: usize,
    /// The puzzle board asked for (its object's use, 0x8004d3f0).
    pub board: bool,
    /// Screens a level asks for (the golf box), shown with the level
    /// frozen.
    pub pages: Vec<crate::screens::Page>,
}

/// A collectible (0x80048a44, touched by 0x800514b0).
pub struct Goodie {
    /// A bar's number in its level.
    pub number: usize,
    pub shape: u32,
    pub kind: u16,
    pub pos: Vec3,
    pub rot: Mat3,
    pub taken: bool,
}

/// Collectible kinds and their sounds (0x800514b0).
pub mod goodie {
    pub const BAR: u16 = 3;
    pub const COOKIE: u16 = 4;
    pub fn sound(kind: u16) -> u8 {
        match kind {
            1 => 30,
            2 | 15 | 17 => 31,
            3 => 28,
            4 | 14 => 29,
            6..=11 => 113,
            _ => 30,
        }
    }
}

/// How long the bar counter shows after a bar (dt units; the health
/// meter's 1500).
const COUNTER_TIME: i32 = 1500;

/// A model that can be picked up (object flag 0x10000).
pub struct Thing {
    /// Its placement, template and registry id; +0x6d (template +0x32).
    pub placement: usize,
    pub template: u16,
    pub id: u16,
    pub damage: i32,
    /// Hidden (off in the world, 0x80078bc4 or a level's pool).
    pub hidden: bool,
    pub shape: u32,
    pub pos: Vec3,
    pub rot: Mat3,
    pub state: ThingState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThingState {
    Rest,
    Carried,
    /// In flight (0x80087c5c): velocity.
    Flying([i32; 3]),
}

/// Hand slots of the grip (0x8005ff88): the object rides slot 15.
const HAND: usize = 15;
const OTHER_HAND: usize = 12;
/// How near an object must be to be touched (the player's radius and some).
const REACH: f32 = 520.0;
/// A throw (0x80060b44): direction (0, -384, 1024) turned by the heading,
/// speed 170; the velocity is dir * (speed * 65536 / 6000) >> 12
/// (0x8008864c); a put-down is speed 10 along (0, 0, 4096).
const THROW_SPEED: f32 = 170.0 * 65536.0 / 6000.0;
const DROP_SPEED: f32 = 10.0 * 65536.0 / 6000.0;
/// The HUD's bar and clock icons (sprites; their sizes are guesses).
const BAR_ICON: u32 = 291;
const CLOCK_ICON: u32 = 375;

/// Text in font 1 with its shadow 2 down and right (0x80090004).
pub fn shadowed(fonts: &crate::text::Fonts, x: i16, y: i16, string: String, out: &mut Vec<rugrats_render::Vtx>) {
    let t = crate::text::Text { font: 1, spacing: 1, rgb: [255, 255, 255], x, y, string, ..Default::default() };
    let mut shadow = t.clone();
    shadow.rgb = [1, 1, 1];
    shadow.x += 2;
    shadow.y += 2;
    shadow.triangles(fonts, out);
    t.triangles(fonts, out);
}

/// How near a flight passes a character to strike it (the radii, guess).
const STRIKE: i32 = 450;
/// Thrown objects fall at 13 (flight flags 0x700).
const THING_GRAVITY: i32 = 13;

pub use crate::npc::Npc;

/// The puzzle board's object (template 581, model 905; use callback 1,
/// 0x8004d3f0).
const BOARD_OBJECT: u32 = 581;
/// How near (in x and z, and in y) the player must stand to use a door.
const USE_REACH: i32 = 1500;
const USE_REACH_Y: i32 = 1500;
/// The hub, Pickles' home.
pub const HUB: usize = 1;

impl Game {
    pub fn open(disc: &GameDisc, level: usize) -> Result<Game, String> {
        Game::open_from(disc, level, None, &crate::progress::Progress::default())
    }

    /// Level `level`, come to from level `from`: back in the hub the
    /// player stands at 0x800b8718[from] (16 bytes: x, y, z), as level 1's
    /// EpisodeStart callback (0x80078e7c) puts it, unless from training.
    pub fn open_from(disc: &GameDisc, level: usize, from: Option<usize>, progress: &crate::progress::Progress) -> Result<Game, String> {
        let set = Set::load(disc, level)?;
        let col = Col::parse(&disc.file(COL + level)?)?;
        let anims = Anims::parse(&disc.file(ANM + level)?, &disc.file(FRM + level)?, &disc.file(ROT + level)?)
            .map_err(|e| e.to_string())?;
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
        let start = match from.filter(|&f| level == HUB && f != HUB && f != 16) {
            Some(f) => {
                let b = disc.exe_bytes(0x800b_8718 + 16 * f as u32, 12)?;
                let spot = [0, 4, 8].map(|k| i32::from_le_bytes(b[k..k + 4].try_into().unwrap()));
                if spot == [0; 3] { start } else { spot }
            }
            None => start,
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
        // Pickable models move: drawn and collided each frame.
        let pickable = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1
                && set
                    .templates
                    .get(p.template as usize)
                    .is_some_and(|t| t.pickable() && t.hierarchy().is_none() && set.models.find(t.shape as u32) != 0)
        };
        let things: Vec<Thing> = (0..set.placements.len())
            .filter(|&k| pickable(k))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                Thing {
                    placement: k,
                    template: p.template as u16,
                    id: p.id,
                    damage: t.raw[0x32] as i32,
                    hidden: false,
                    shape: t.shape as u32,
                    pos: Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32),
                    rot: rugrats_render::models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    state: ThingState::Rest,
                }
            })
            .collect();
        // Collectibles: template class 12 (0x80049af4, which makes them
        // with 0x80048a44: object class 11), the kind template +0x26.
        let collectible = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1 && !character(k) && set.templates.get(p.template as usize).is_some_and(|t| t.raw[4] == 12 && set.models.find(t.shape as u32) != 0)
        };
        // Bars are numbered in the order they are made (0x800c4ba8); a bar
        // found before stays found (+0x48).
        let mut bar = 0usize;
        let goodies: Vec<Goodie> = (0..set.placements.len())
            .filter(|&k| collectible(k))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                let kind = u16::from_le_bytes([t.raw[0x26], t.raw[0x27]]);
                let number = if kind == goodie::BAR {
                    bar += 1;
                    bar - 1
                } else {
                    0
                };
                Goodie {
                    number,
                    taken: kind == goodie::BAR && progress.bar_found(level, number),
                    shape: t.shape as u32,
                    kind,
                    pos: Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32),
                    rot: rugrats_render::models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                }
            })
            .collect();
        let pickable = |k: usize| pickable(k) && !collectible(k);
        let apart: Vec<bool> = (0..set.placements.len()).map(|k| character(k) || pickable(k) || collectible(k)).collect();
        let scenery = set.objects_except(&Frame::default(), |k| apart[k]);
        let mut npcs = Vec::new();
        for (k, p) in set.placements.iter().enumerate() {
            if !character(k) || Some(k) == player_place {
                continue;
            }
            let Some(h) = set.templates.get(p.template as usize).and_then(|t| t.hierarchy()) else { continue };
            let Some(Some(hierarchy)) = set.hierarchies.get(h).cloned() else { continue };
            let mut animator = crate::animator::Animator::load(disc, h as u16)?;
            animator.set(crate::player::anim::IDLE, &anims);
            // Template +0x18 health, +0x1a range, +0x1c wait, +0x1e speed
            // index, +0x32 damage (0x80081bac).
            let t = &set.templates[p.template as usize];
            let h16 = |at: usize| u16::from_le_bytes([t.raw[at], t.raw[at + 1]]) as i32;
            let [_, qy, _, qw] = p.rotation.map(|v| v as f64);
            let heading = ((2.0 * qy.atan2(qw) / std::f64::consts::TAU * 4096.0).round() as i32).rem_euclid(4096);
            let mut n = Npc {
                id: p.id,
                character: h,
                hierarchy,
                animator,
                pos: p.pos,
                heading,
                speed: 0,
                speed_index: (h16(0x1e) as usize).min(5),
                range: h16(0x1a),
                wait: h16(0x1c),
                damage: t.raw[0x32] as i32,
                health: h16(0x18),
                max_health: h16(0x18),
                mode: crate::npc::Mode::Idle,
                home: p.pos,
                hidden: false,
                attacks: false,
                cooldown: 0,
                first_post: p.post,
                loops: (p.level as i32, p.level as i32),
                size: 0,
            };
            // On the floor under it (0x80081bac asks 0x8002b240), the root
            // the hierarchy's offset above the feet.
            n.size = n.hierarchy.half_size();
            n.place(p.pos, &col);
            npcs.push(n);
        }
        let focal = trig.geom_screen(FOV).unwrap_or(256) as f32;
        let yaw = heading as f32;
        let boxes = object_boxes(&set, |k| apart[k]);
        let posts = crate::npc::posts(&set.placements);
        let mut g = Game {
            level,
            set,
            col,
            anims,
            player,
            stage,
            trig,
            focal,
            eye: Vec3::ZERO,
            yaw,
            scenery,
            exit: None,
            cine: None,
            checkpoint: None,
            tries: disc.exe_bytes(TRIES + level as u32, 1).map_or(3, |b| b[0] as i8 as i32),
            crying: 0,
            script: None,
            won: None,
            sprites: crate::hud::Sprites::load(disc, level).ok(),
            meter_shown: 0,
            meter_health: 255,
            boxes,
            posts,
            npc_events: Vec::new(),
            rand: crate::rand::Rand::default(),
            npcs,
            room: None,
            song: None,
            effects: Vec::new(),
            paused: false,
            pause: Pause::load(disc)?,
            fonts: crate::text::Fonts::load(disc)?,
            quit: false,
            things,
            goodies,
            bars: progress.bars_available(),
            bar_shown: 0,
            found: Vec::new(),
            save: false,
            load: false,
            carried: None,
            grip: Vec3::ZERO,
            hidden: Default::default(),
            apart,
            unusable: Default::default(),
            struck: Vec::new(),
            fixed: None,
            eye_up: default_eye_up(level),
            behind: if level == 7 { 6400.0 } else { BEHIND },
            post_camera: None,
            hazards: Vec::new(),
            hurt_wait: 0,
            timer: None,
            record: level,
            board: false,
            pages: Vec::new(),
        };
        g.hazards = hazards(&g.set, &g.apart);
        g.script = match level {
            2 => Some(Box::new(crate::level2::Level2::start(&mut g, disc)?)),
            3 => Some(Box::new(crate::level3::Level3::start(&mut g, disc)?)),
            23..=31 => Some(Box::new(crate::golf::Golf::start(&mut g, disc)?)),
            _ => None,
        };
        g.eye = g.eye_goal();
        g.draw();
        Ok(g)
    }

    /// The player's room (0x8002b660, here by the rooms' bounds, the one
    /// it was in first); on a change, the room's song (0x80077a54).
    fn room_change(&mut self) {
        let p = self.player.pos;
        let inside =
            |k: usize| self.set.room_info.get(k).is_some_and(|r| (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i]));
        let room = match self.room.filter(|&k| inside(k)) {
            Some(k) => Some(k),
            None => (0..self.set.room_info.len()).find(|&k| inside(k)),
        };
        if room == self.room {
            return;
        }
        self.room = room;
        let Some(k) = room else { return };
        let r = self.set.room_info[k];
        // Flag 2: the room's cinematic, once (PostCine, then cleared).
        if r.flags & 2 != 0 && r.cine != 0 {
            self.set.room_info[k].flags &= !2;
            self.cine = Some(r.cine as usize);
        }
        // Flag 32: a checkpoint (0x800772e0): where a cry sends the
        // player back to.
        if r.flags & 32 != 0 {
            self.checkpoint = Some((self.player.pos, self.player.heading));
        }
        if r.flags & 128 != 0 && (1..5).contains(&r.song) {
            self.song = Some(r.song as usize - 1);
        }
        // Flag 4: the room's camera (CameraSwitch, 0x80077c88 into
        // 0x800291f8): kind 3 a fixed camera at post +0x5c; the follow
        // kinds set its height (6: 200, 7: 500, 8: 1600, 9: 2000, 11:
        // 500; 1500 means the level's own).
        if r.flags & 4 != 0 && (!matches!(self.level, 20 | 21) || r.camera == 3) {
            match r.camera {
                3 => self.post_camera = Some(r.camera_post as usize),
                1 | 6..=13 => {
                    self.post_camera = None;
                    self.eye_up = match r.camera {
                        6 => 200.0,
                        7 | 11 => 500.0,
                        8 => 1600.0,
                        9 => 2000.0,
                        _ => default_eye_up(self.level),
                    };
                }
                _ => {}
            }
        }
        // Flag 64: the level's Gameplay script, once (0x80077ee4).
        if r.flags & 64 != 0 {
            self.set.room_info[k].flags &= !64;
            self.with_script(|s, g| s.gameplay(g, r.script));
        }
    }

    /// Runs `f` on the level's script, if any.
    pub fn with_script<R: Default>(&mut self, f: impl FnOnce(&mut dyn Script, &mut Game) -> R) -> R {
        let Some(mut s) = self.script.take() else { return R::default() };
        let r = f(s.as_mut(), self);
        self.script = Some(s);
        r
    }

    /// Hides or shows the objects with registry id `id` (0x80078bc4):
    /// characters, pickables and the scenery.
    pub fn hide(&mut self, id: u16, hidden: bool) {
        for n in self.npcs.iter_mut().filter(|n| n.id == id) {
            n.hidden = hidden;
        }
        for t in self.things.iter_mut().filter(|t| t.id == id) {
            t.hidden = hidden;
        }
        let changed = if hidden { self.hidden.insert(id) } else { self.hidden.remove(&id) };
        if changed {
            let skip = |k: usize| self.apart[k] || self.hidden.contains(&self.set.placements[k].id);
            self.scenery = self.set.objects_except(&Frame::default(), skip);
            self.boxes = object_boxes(&self.set, skip);
        }
    }

    /// Takes the use away from the objects with id `id` (+0x54 = 0).
    pub fn disable_use(&mut self, id: u16) {
        for (k, p) in self.set.placements.iter().enumerate() {
            if p.kind == 1 && p.id == id {
                self.unusable.insert(k);
            }
        }
    }

    /// GRAB and ACTION in the stand state (0x8005d11c): Circle picks the
    /// touched object up (state 7) or puts the carried one down (state 8);
    /// Triangle throws it (0x80060b44).
    fn carry(&mut self, pads: &Pads) {
        let p = &self.player;
        let standing =
            matches!(p.state, crate::player::State::Stand | crate::player::State::Walk | crate::player::State::Run)
                && p.action.is_none();
        if !standing {
            return;
        }
        let me = Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32);
        let rot = p.rotation(&self.trig);
        if let Some(k) = self.carried {
            let throw = pads.pressed(0, input::TRIANGLE);
            if throw || pads.pressed(0, input::CIRCLE) {
                let (dir, speed, anim) = if throw {
                    (rot * Vec3::new(0.0, -384.0, 1024.0), THROW_SPEED, crate::player::anim::THROW)
                } else {
                    (rot * Vec3::new(0.0, 0.0, 4096.0), DROP_SPEED, crate::player::anim::PUT_DOWN)
                };
                let v = dir.normalize() * speed;
                self.things[k].state = ThingState::Flying([v.x as i32, v.y as i32, v.z as i32]);
                self.carried = None;
                self.player.carrying = false;
                self.player.act(anim, &self.anims);
                if throw {
                    self.player.jump_lock = 300;
                }
            }
            return;
        }
        if !pads.pressed(0, input::CIRCLE) {
            return;
        }
        let near = self
            .things
            .iter()
            .enumerate()
            .filter(|(_, t)| t.state == ThingState::Rest && !t.hidden)
            .map(|(k, t)| (k, Vec3::new(t.pos.x - me.x, 0.0, t.pos.z - me.z).length(), (t.pos.y - me.y).abs()))
            .filter(|&(_, d, dy)| d < REACH && dy < 1200.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((k, _, _)) = near else { return };
        // The grip (0x8005ff88 mode 0): the hands' midpoint less hand 15,
        // in the player's frame, pushed forward by half the object's depth.
        let parts = self.player.parts(&self.anims, &self.trig);
        let at = |slot| self.player.slot_part(slot).and_then(|i| parts.get(i)).map(|p| p.1);
        let depth = self
            .set
            .models
            .get(self.things[k].shape)
            .map_or(0.0, |m| m.boxes.iter().map(|b| (b.1[2] - b.0[2]) as f32).fold(0.0, f32::max));
        self.grip = match (at(HAND), at(OTHER_HAND)) {
            (Some(a), Some(b)) => rot.transpose() * ((a + b) / 2.0 - a) + Vec3::new(0.0, 0.0, depth / 2.0),
            _ => Vec3::new(0.0, -300.0, depth / 2.0 + 200.0),
        };
        self.things[k].state = ThingState::Carried;
        self.carried = Some(k);
        self.player.carrying = true;
        self.player.act(crate::player::anim::PICK_UP, &self.anims);
    }

    /// Touching collectibles (0x800514b0): taken, with their kind's sound.
    fn touch(&mut self) {
        self.found.clear();
        self.bar_shown = (self.bar_shown - crate::player::DT).max(0);
        let me = Vec3::new(self.player.pos[0] as f32, self.player.pos[1] as f32, self.player.pos[2] as f32);
        for g in self.goodies.iter_mut().filter(|g| !g.taken) {
            let r = self.set.models.get(g.shape).map_or(256.0, |m| m.radius as f32) + crate::player::RADIUS as f32;
            let d = g.pos - me;
            if Vec3::new(d.x, 0.0, d.z).length() < r && d.y.abs() < 1200.0 {
                g.taken = true;
                self.effects.push((goodie::sound(g.kind), crate::sound::EFFECTS_VOLUME));
                if g.kind == goodie::BAR {
                    self.bars += 1;
                    self.bar_shown = COUNTER_TIME;
                    self.found.push(g.number);
                }
            }
        }
    }

    /// Flights (0x80087c5c): gravity 13, pos += (v dt + a dt^2 / 2) >> 6,
    /// until the object's bottom reaches the floor.
    fn fly(&mut self) {
        let dt = crate::player::DT;
        self.struck.clear();
        for (k, t) in self.things.iter_mut().enumerate() {
            let ThingState::Flying(mut v) = t.state else { continue };
            // A flight that reaches a character (0x80086ef0): its touch
            // (0x800511a8) with the thing's +0x6d; the thing drops.
            if v[0] != 0 || v[2] != 0 {
                let hit = self.npcs.iter().position(|n| {
                    let (dx, dz) = (n.pos[0] as f32 - t.pos.x, n.pos[2] as f32 - t.pos.z);
                    !n.hidden
                        && !matches!(n.mode, crate::npc::Mode::Ride { .. })
                        && dx * dx + dz * dz < (STRIKE * STRIKE) as f32
                        && (t.pos.y - (n.pos[1] - n.feet() / 2) as f32).abs() < 900.0
                });
                if let Some(n) = hit {
                    self.struck.push((n, k, t.damage));
                    v[0] = 0;
                    v[2] = 0;
                }
            }
            let bottom =
                self.set.models.get(t.shape).and_then(|m| m.boxes.iter().map(|b| b.1[1]).max()).unwrap_or(0) as f32;
            for k in [0, 2] {
                t.pos[k] += (v[k] * dt >> 6) as f32;
            }
            t.pos.y += ((v[1] * dt + THING_GRAVITY * dt * dt / 2) >> 6) as f32;
            v[1] += THING_GRAVITY * dt;
            let q = crate::collide::query(&self.col, [t.pos.x as i32, (t.pos.y - 1.0) as i32, t.pos.z as i32]);
            let floor = if q.tile.is_some() { q.floor as f32 } else { f32::MAX };
            if q.kind == 2 || t.pos.y + bottom >= floor {
                if floor < f32::MAX {
                    t.pos.y = floor - bottom;
                }
                t.state = ThingState::Rest;
            } else {
                t.state = ThingState::Flying(v);
            }
        }
    }

    /// ForceCry (0x8007fd0c): the player cries (0x8007f920), then a try
    /// is gone.
    pub fn cry(&mut self) {
        if self.crying > 0 || self.exit.is_some() || self.cine.is_some() {
            return;
        }
        self.crying = CRY_TIME;
        self.player.act(crate::player::anim::CRY, &self.anims);
    }

    /// After the cry: health back to 255; the level's own respawn, or by
    /// default the last checkpoint; with no tries left, the hub (not from
    /// levels 1, 20 and 21).
    fn cried(&mut self) {
        self.player.health = 255;
        self.player.action = None;
        if self.with_script(|s, g| s.cry(g)) {
            return;
        }
        if self.tries > 0 {
            self.tries -= 1;
            if self.tries == 0 {
                if !matches!(self.level, 1 | 20 | 21) {
                    self.exit = Some((HUB, None));
                }
                return;
            }
        }
        if let Some((at, heading)) = self.checkpoint {
            self.player.pos = at;
            self.player.heading = heading;
            self.player.target = heading;
            self.player.speed = 0;
            self.yaw = heading as f32;
            self.eye = self.eye_goal();
        }
    }

    /// A cinematic asked for has played (CineFinish, 0x800783d8): the
    /// endings, and the level's own reaction.
    pub fn cine_finished(&mut self, n: usize) {
        self.room = None;
        self.eye = self.eye_goal();
        rrt::tracing::info!("level {}: cine {n} finished", self.level);
        // The ending cinematics (0x8007737c): back to the hub with the
        // level recorded won or lost; Reptar Dance ends the game.
        match ENDINGS.iter().find(|e| e.0 == n) {
            Some(&(_, won)) => {
                self.won = Some(won);
                self.exit = Some((HUB, None));
            }
            None if n == REPTAR_DANCE => self.quit = true,
            None => {}
        }
        self.with_script(|s, g| s.cine_finished(g, n));
    }

    /// The projection distance in play.
    pub fn focal(&self) -> f32 {
        self.focal
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
        self.target() - Vec3::new(a.sin(), 0.0, a.cos()) * self.behind - Vec3::new(0.0, self.eye_up, 0.0)
    }

    /// One frame of the level (0x80018c38): the player, then the camera,
    /// then the draw.
    pub fn frame(&mut self, pads: &Pads) {
        if !self.paused && pads.pressed(0, input::START) {
            self.paused = true;
            self.pause.open();
            self.effects.clear();
            self.draw();
            return;
        }
        if self.paused {
            self.effects.clear();
            match self.pause.frame(pads) {
                Some(PauseEnd::Continue) => self.paused = false,
                Some(PauseEnd::QuitGame) => self.quit = true,
                Some(PauseEnd::Save) => self.save = true,
                Some(PauseEnd::Load) => self.load = true,
                // Quit Level: to Pickles' Home, or the front end from the
                // hub and training (0x800398bc).
                Some(PauseEnd::QuitLevel) => {
                    if matches!(self.level, HUB | 16) {
                        self.quit = true;
                    } else {
                        self.exit = Some((HUB, None));
                    }
                }
                None => {}
            }
            self.effects.extend(self.pause.sounds.drain(..).map(|s| (s, crate::sound::EFFECTS_VOLUME)));
            self.draw();
            return;
        }
        if self.crying > 0 {
            // The cry runs on its own (0x8007f920's loop): the world waits.
            self.crying -= crate::player::DT;
            self.player.animator.step(&self.anims);
            if self.crying <= 0 || !self.player.animator.playing(crate::player::anim::CRY) {
                self.crying = 0;
                self.cried();
            }
            self.draw();
            return;
        }
        let mut boxes = self.boxes.clone();
        for t in self.things.iter().filter(|t| t.state == ThingState::Rest) {
            if let Some(m) = self.set.models.get(t.shape) {
                boxes.extend(world_boxes(m, t.rot, t.pos));
            }
        }
        // The other characters stand in the way: a box of their radius
        // (object +0x86 is 0x2000 / 8 for a baby, guessed at 200) from the
        // feet up.
        for n in self.npcs.iter().filter(|n| !n.hidden && !matches!(n.mode, crate::npc::Mode::Ride { .. })) {
            let feet = (n.pos[1] - n.feet()) as f32;
            boxes.push(crate::player::Box3 {
                lo: [-200, 0, -200],
                hi: [200, 0, 200],
                rot: Mat3::IDENTITY,
                pos: Vec3::new(n.pos[0] as f32, n.pos[1] as f32, n.pos[2] as f32),
                min_y: (feet - 700.0) as i32,
                max_y: feet as i32,
            });
        }
        self.player.frame(pads, self.yaw.round() as i32, &self.col, &boxes, &self.anims, &self.trig);
        self.carry(pads);
        self.fly();
        self.touch();
        let d = (self.eye - Vec3::new(self.player.pos[0] as f32, self.player.pos[1] as f32, self.player.pos[2] as f32))
            .length();
        let volume = (crate::sound::EFFECTS_VOLUME as f32 * (1.0 - d / 16384.0).max(0.0)) as i32;
        self.effects = self.player.effects.iter().map(|&e| (e, volume)).collect();
        // Using an object (the stand handler's ACTION, Triangle, on the
        // touched object): a door (use 0x8004f78c, LP +0x42) goes to its
        // level; one with only a cinematic (0x8004f83c, LP +0x44, made by
        // 0x8004691c: 16 is played as 14, template 567 is 8) plays it.
        if pads.pressed(0, input::TRIANGLE) && !self.player.carrying && self.carried.is_none() {
            let p = self.player.pos;
            let cine_of = |d: &rugrats_data::placement::Placement| match (d.cine, d.template) {
                (16, _) => 14,
                (0, 567) => 8,
                (c, _) => c as usize,
            };
            let near = self
                .set
                .placements
                .iter()
                .enumerate()
                .filter(|(k, d)| {
                    d.kind == 1
                        && !self.apart[*k]
                        && (d.level > 0 || cine_of(d) > 0 || d.template == BOARD_OBJECT)
                        && !self.unusable.contains(k)
                        && !self.hidden.contains(&d.id)
                })
                .map(|(_, d)| d)
                .min_by_key(|d| (d.pos[0] - p[0]).pow(2) as i64 + (d.pos[2] - p[2]).pow(2) as i64);
            if let Some(d) = near.filter(|d| {
                (d.pos[0] - p[0]).abs() < USE_REACH
                    && (d.pos[2] - p[2]).abs() < USE_REACH
                    && (d.pos[1] - p[1]).abs() < USE_REACH_Y
            }) {
                if d.template == BOARD_OBJECT {
                    self.board = true;
                } else if d.level > 0 {
                    rrt::tracing::info!("level {}: a door to level {} (cine {})", self.level, d.level, d.cine);
                    self.exit = Some((d.level as usize, (d.cine > 0).then_some(d.cine as usize)));
                } else {
                    rrt::tracing::info!("level {}: object {} plays cine {}", self.level, d.id, cine_of(d));
                    self.cine = Some(cine_of(d));
                }
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
        let world = crate::npc::World {
            player: self.player.pos,
            player_heading: self.player.heading,
            col: &self.col,
            posts: &self.posts,
            trig: &self.trig,
            anims: &self.anims,
        };
        self.npc_events = crate::npc::step(&mut self.npcs, &world, &mut self.rand);
        self.with_script(|s, g| s.frame(g, pads));
        // Hazards (0x8005e1f4, the walk and run states): walking into one
        // costs its +0x6d; the next only after 300 dt (guess).
        self.hurt_wait = (self.hurt_wait - crate::player::DT).max(0);
        let walking = matches!(self.player.state, crate::player::State::Walk | crate::player::State::Run);
        if self.hurt_wait == 0 && self.crying == 0 && walking && !self.player.frozen {
            let p = self.player.pos;
            let (feet, head) = (p[1] - self.player.feet(), p[1] - self.player.feet() - crate::player::HEIGHT);
            let hurt = self
                .hazards
                .iter()
                .filter(|(b, _)| b.covers(p, crate::player::RADIUS + 40) && b.min_y <= feet + 40 && b.max_y >= head)
                .map(|&(_, d)| d)
                .max();
            if let Some(d) = hurt {
                self.player.health = (self.player.health - d).max(0);
                self.hurt_wait = 300;
            }
        }
        // The level timer (0x8007f5a8): at 0, sound 117 and TimerExpired.
        if let Some(t) = self.timer {
            let t = t - crate::player::DT;
            if t <= 0 {
                self.timer = None;
                self.effects.push((117, crate::sound::EFFECTS_VOLUME));
                self.with_script(|s, g| s.timer_expired(g));
            } else {
                self.timer = Some(t);
            }
        }
        // Damage (0x80077864): health at 0 is a cry.
        if self.player.health <= 0 {
            self.cry();
        }
        self.meter_shown = (self.meter_shown - crate::player::DT).max(0);
        if self.player.health != self.meter_health {
            self.meter_health = self.player.health;
            self.meter_shown = crate::hud::METER_TIME;
        }
        self.room_change();
        let goal = self.clear_eye(self.eye_goal());
        self.eye += (goal - self.eye) / 8.0;
        self.draw();
    }

    fn draw(&mut self) {
        let mut moving = self.scenery.clone();
        self.player.triangles(&self.set.models, &self.anims, &self.trig, &mut moving);
        // The carried object rides hand 15 (0x8005ff88 mode 1).
        if let Some(k) = self.carried {
            let parts = self.player.parts(&self.anims, &self.trig);
            let rot = self.player.rotation(&self.trig);
            if let Some(hand) = self.player.slot_part(HAND).and_then(|i| parts.get(i)) {
                self.things[k].pos = hand.1 + rot * self.grip;
                self.things[k].rot = rot;
            }
        }
        for t in self.things.iter().filter(|t| !t.hidden) {
            if let Some(m) = self.set.models.get(t.shape) {
                rugrats_render::models::model_triangles(m, t.rot, t.pos, &mut moving);
            }
        }
        for g in self.goodies.iter().filter(|g| !g.taken) {
            if let Some(m) = self.set.models.get(g.shape) {
                rugrats_render::models::model_triangles(m, g.rot, g.pos, &mut moving);
            }
        }
        for n in &self.npcs {
            crate::npc::triangles(n, &self.set.models, &self.anims, &self.trig, &mut moving);
        }
        let mut overlay = Vec::new();
        if let Some(s) = &self.script {
            s.draw(self, &mut moving, &mut overlay);
        }
        // The health meter (levels 2-6, 9, 12, 14, 16: 0x8008c094's switch).
        if self.meter_shown > 0 && matches!(self.level, 2..=6 | 9 | 12 | 14 | 16) {
            if let Some(s) = &self.sprites {
                crate::hud::meter(s, self.player.health, 255, self.focal, &mut overlay);
            }
        }
        self.stage.set_moving(moving);
        self.stage.overlay = overlay;
        if self.paused {
            self.pause.triangles(&self.fonts, &mut self.stage.overlay);
        } else {
            // The level timer: "%ld" seconds at the top by the clock
            // (model 375).
            if let Some(t) = self.timer {
                if let Some(sp) = &self.sprites {
                    sp.draw(CLOCK_ICON, (-40, -96), 24.0, 24.0, crate::hud::METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
                }
                shadowed(&self.fonts, -12, -100, format!("{}", t / 300), &mut self.stage.overlay);
            }
            if self.bar_shown > 0 {
                // The bar counter (0x80090004): model 291 at (-192, -76)
                // and the bars available at (-150, -80); bottom left in
                // levels 17-18, bottom right in golf.
                let (icon, text) = match self.level {
                    17 | 18 => ((-192, 76), (-150, 80)),
                    23..=31 => ((192, 76), (145, 70)),
                    _ => ((-192, -76), (-150, -80)),
                };
                if let Some(sp) = &self.sprites {
                    sp.draw(BAR_ICON, icon, 26.0, 32.0, crate::hud::METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
                }
                shadowed(&self.fonts, text.0, text.1, format!("{}", self.bars), &mut self.stage.overlay);
            }
        }
        let post = self.post_camera.and_then(|k| self.posts.get(k.wrapping_sub(1))).map(|p| {
            (Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32), self.target())
        });
        let (eye, target) = self.fixed.or(post).unwrap_or((self.eye, self.target()));
        let f = (target - eye).normalize_or(Vec3::Z);
        let x = Vec3::Y.cross(f).normalize_or(Vec3::X);
        let y = f.cross(x);
        // World to camera: rows x (right), y (down), z (ahead).
        let rows = Mat3::from_cols(x, y, f).transpose();
        self.stage.camera = crate::stage::view(eye, rows, self.focal);
    }
}

/// The world boxes of the placed models (the player's placement left out):
/// each model box with the placement's rotation and position, and the
/// heights its corners span.
/// The boxes of the objects that hurt on touch, with their +0x6d.
fn hazards(set: &Set, apart: &[bool]) -> Vec<(crate::player::Box3, i32)> {
    let mut out = Vec::new();
    for (k, place) in set.placements.iter().enumerate() {
        let Some(t) = set.templates.get(place.template as usize) else { continue };
        let damage = t.raw[0x32] as i32;
        if place.kind != 1 || apart[k] || damage == 0 || t.hierarchy().is_some() {
            continue;
        }
        let Some(m) = set.models.get(t.shape as u32).filter(|_| set.models.find(t.shape as u32) != 0) else { continue };
        let rot = rugrats_render::models::quat_matrix(place.rotation.map(|v| v as i16)).transpose();
        let pos = Vec3::new(place.pos[0] as f32, place.pos[1] as f32, place.pos[2] as f32);
        out.extend(world_boxes(m, rot, pos).into_iter().map(|b| (b, damage)));
    }
    out
}

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
        out.extend(world_boxes(m, rot, pos));
    }
    out
}

/// The cinematics that end a level (0x8007737c, table 0x8001532c), and
/// whether it was won: 5 (level 2), 14 (3), 19 (9), 20 and 21 (13), 24
/// (18), 26 (17), 35 (8), 40 (5), 47 (6), 48 (4), 50 (21), 62 (11), 64
/// (20), 68 (12), 72 (15), 78 (10).
const ENDINGS: [(usize, bool); 17] = [
    (5, true),
    (14, true),
    (19, true),
    (20, false),
    (21, true),
    (24, true),
    (26, true),
    (35, true),
    (40, true),
    (47, true),
    (48, true),
    (50, true),
    (62, true),
    (64, true),
    (68, true),
    (72, true),
    (78, true),
];
/// Tries a level gives (s8 by level).
const TRIES: u32 = 0x800b_2920;
/// The cry's longest run (0x8007f920: 2100 dt).
const CRY_TIME: i32 = 2100;
/// Cine 85: the end of the game (exit code -1).
const REPTAR_DANCE: usize = 85;

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
        // Golf: "Golf Cam n" by level; level 25 the intro (main plays
        // "Golf Cam 3", 31, when coming from a hole).
        23 => 29,
        24 => 30,
        25 => 34,
        26 => 32,
        27 => 33,
        28 => 81,
        29 => 82,
        30 => 83,
        31 => 84,
        _ => return None,
    })
}

/// A model's boxes at `rot`, `pos`, with the heights their corners span.
fn world_boxes(m: &rugrats_data::model::Model, rot: Mat3, pos: Vec3) -> Vec<crate::player::Box3> {
    let mut out = Vec::new();
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
    out
}

/// How the pause menu ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseEnd {
    Continue,
    /// Save Game (state 3, 0x80070d80): the menu stays.
    Save,
    /// Load Game (state 9): a confirmed load returns 1, to the hub.
    Load,
    QuitGame,
    QuitLevel,
}

/// The pause menu (0x8003e05c, 0x800398bc mode 0, laid out by
/// 0x80035fec): "Game Paused" over five items, the cursor 1-5; a quit
/// asks again (0x80036368) with "No?" first.
struct Pause {
    strings: [String; 6],
    yes: String,
    no: String,
    cursor: usize,
    /// Asking about item 3 or 5: Yes (true) or No.
    confirm: Option<(usize, bool)>,
    sounds: Vec<u8>,
}

/// Where the items sit (0x80035fec), from the screen's centre.
const PAUSE_AT: [(i16, i16); 5] = [(-185, -30), (-145, -10), (-60, -30), (30, -10), (80, -30)];
/// The menu's sounds: move 124 and 125, pick 126.
const MOVE: u8 = 124;
const MOVE_BACK: u8 = 125;
const PICK: u8 = 126;

impl Pause {
    fn load(disc: &GameDisc) -> Result<Pause, String> {
        let s = |n: u32| crate::front::string(disc, n);
        Ok(Pause {
            strings: [s(1), s(2), s(3), s(4), s(5), s(6)],
            yes: disc.exe_string_at_addr(0x800c_48f0)?,
            no: disc.exe_string_at_addr(0x800c_48ec)?,
            cursor: 1,
            confirm: None,
            sounds: Vec::new(),
        })
    }

    fn open(&mut self) {
        self.cursor = 1;
        self.confirm = None;
    }

    fn frame(&mut self, pads: &Pads) -> Option<PauseEnd> {
        let pressed = |b: usize| pads.pressed(0, b);
        if let Some((item, yes)) = &mut self.confirm {
            if pressed(input::LEFT) || pressed(input::RIGHT) {
                *yes = !*yes;
                self.sounds.push(MOVE);
            }
            if pressed(input::TRIANGLE) {
                self.confirm = None;
                return None;
            }
            if pressed(input::CROSS) {
                self.sounds.push(PICK);
                let (item, yes) = (*item, *yes);
                self.confirm = None;
                if yes {
                    return Some(if item == 3 { PauseEnd::QuitGame } else { PauseEnd::QuitLevel });
                }
            }
            return None;
        }
        if pressed(input::LEFT) && self.cursor > 2 {
            self.cursor -= 2;
            self.sounds.push(MOVE_BACK);
        }
        if pressed(input::RIGHT) && self.cursor < 4 {
            self.cursor += 2;
            self.sounds.push(MOVE);
        }
        if pressed(input::UP) || pressed(input::DOWN) {
            self.cursor = if self.cursor % 2 == 1 { (self.cursor + 1).min(4) } else { self.cursor - 1 };
            self.sounds.push(MOVE);
        }
        if pressed(input::TRIANGLE) || pressed(input::START) {
            return Some(PauseEnd::Continue);
        }
        if pressed(input::CROSS) {
            self.sounds.push(PICK);
            match self.cursor {
                1 => return Some(PauseEnd::Continue),
                3 | 5 => self.confirm = Some((self.cursor, false)),
                2 => return Some(PauseEnd::Save),
                _ => return Some(PauseEnd::Load),
            }
        }
        None
    }

    fn triangles(&self, fonts: &crate::text::Fonts, out: &mut Vec<rugrats_render::Vtx>) {
        use crate::text::Text;
        // The heading in font 1, the items in font 0 (0x80035fec).
        let mut item = |s: &str, x: i16, y: i16, lit: bool, centre: bool| {
            let mut t = Text { font: if centre { 1 } else { 0 }, spacing: 1, string: s.to_string(), x, y, rgb: if lit { [255, 255, 5] } else { [255, 255, 255] }, ..Default::default() };
            if centre {
                fonts.centre(&mut t, true, false);
            }
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(fonts, out);
            t.triangles(fonts, out);
        };
        match self.confirm {
            Some((q, yes)) => {
                item(&self.strings[q], 0, -60, false, true);
                item(&self.yes, -60, -20, yes, false);
                item(&self.no, 40, -20, !yes, false);
            }
            None => {
                item(&self.strings[0], 0, -60, false, true);
                for (k, &(x, y)) in PAUSE_AT.iter().enumerate() {
                    item(&self.strings[k + 1], x, y, self.cursor == k + 1, false);
                }
            }
        }
    }
}
