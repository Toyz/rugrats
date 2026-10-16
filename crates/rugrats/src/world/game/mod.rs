//! Playing a level (the runner 0x80017c0c, its frame 0x80018c38): the
//! database's rooms and objects on a stage, the collision under them, the
//! player, and the follow camera.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::cine::Frame;
use rugrats_data::col::Col;
use rugrats_data::placement::{Placement, Spot, Template};
use rugrats_render::{Vtx, models};

use crate::actors::animator::{self, Animator};
use crate::actors::npc::{self, Event, Mode, Post, REACH, World};
use crate::actors::player::anim::{CRY, IDLE};
use crate::actors::player::{Box3, DT, HEIGHT, Player, RADIUS, SLOW_TURN, State, TURN};
use crate::audio::ambience::{self, Tables};
use crate::audio::speech::Speech;
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::input::{self, Pads};
use crate::media::cine::Set;
use crate::rand::Rand;
use crate::save::card::STATE_SIZE;
use crate::save::progress::Progress;
use crate::stage::Stage;
use crate::ui::hud::{self, METER_DEPTH, METER_TIME, Sprites};
use crate::ui::screens::Page;
use crate::ui::text::{Fonts, Text};
use crate::ui::tips::Training;

use boxes::*;
use camera::*;
use doors::*;
use pause::*;
use rooms::*;
use story::*;
use things::*;

pub use crate::actors::npc::Npc;
pub use crate::actors::player::Pose;
pub use crate::levels::Level;

mod boxes;
mod camera;
mod doors;
mod goodies;
mod movers;
mod pause;
mod rooms;
mod story;
mod things;
mod toys;

pub use doors::Door;
pub use goodies::{Collected, Goodie, goodie, spawn};
pub use movers::Mover;
pub use pause::PauseEnd;
pub use story::{hit_line, start_cine};
pub use things::{Breakable, Strike, THROW_ANIM_SPEED, Thing, ThingState};
pub use toys::{Spinner, Walker};

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

/// The default character: Tommy (0x800c5680 = 1), whose hierarchy is the
/// database's number 1 (0x800450ac).
const TOMMY: usize = 1;

/// Where a level ends (the exit code, 0x800c4918): a level to go to,
/// and a cinematic to play first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub level: Level,
    pub cine: Option<usize>,
}

impl Exit {
    /// Straight to `level`.
    pub fn to(level: Level) -> Exit {
        Exit { level, cine: None }
    }
}

/// A sound effect asked for this frame: its number, and for a sound in
/// the world its left and right volumes (0x8006f644); None plays at the
/// effects setting (0x8006f26c).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub sound: u8,
    pub at: Option<(i32, i32)>,
    /// A program, tone and note keyed on directly (0x800a99a8), not
    /// through the effects table: the crunch (0x8006f2fc), the piano
    /// (0x8006f3a4).
    pub key: Option<[u8; 3]>,
}

impl Effect {
    pub fn new(sound: u8) -> Effect {
        Effect { sound, at: None, key: None }
    }
}

/// The positional sounds' falloff (0x800b8024): full within 128, none
/// past 16384, 127 at most; measured as dx^2 / 16384 + dy^2 / 4096 +
/// dz^2 / 16384 (0x800c435c = 14).
const NEAR: i64 = 128;
const FAR: i64 = 16384;
const LOUDEST: i64 = 127;

/// A sound at a distance (0x8006fd68): the falloff, no louder than the
/// effects setting.
fn falloff(d: i64, setting: i32) -> i32 {
    let v = if d < NEAR {
        LOUDEST
    } else if d > FAR {
        0
    } else {
        LOUDEST - (d - NEAR) / ((FAR - NEAR) / LOUDEST)
    };
    (v as i32).min(setting).max(0)
}

/// Rounds a product's 4096ths toward zero, as `(v + 4095) >> 12` for a
/// negative v.
fn q12(v: i32) -> i32 {
    if v < 0 { (v + 4095) >> 12 } else { v >> 12 }
}

/// The stereo split (0x8006fc40) of volume `v` for a sound `angle` off the
/// camera's heading: up to 89/256 less from behind, and the far side up
/// to 3/4 less.
fn pan(trig: &Trig, angle: i32, v: i32) -> (i32, i32) {
    let back = (-trig.cos(angle)).max(0);
    let v = v - q12((v * 89 >> 8) * back);
    let side = q12((v * 192 >> 8) * trig.sin(angle));
    let (l, r) = if side < 0 { (v, v + side) } else { (v - side, v) };
    (l.clamp(0, 127), r.clamp(0, 127))
}

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
    /// Whether ForceCry plays the cry first (some levels' cases skip it).
    fn cry_animation(&self) -> bool {
        true
    }
    /// A room's Gameplay script (room flag 64, 0x80077ee4).
    fn gameplay(&mut self, _g: &mut Game, _n: u8) {}
    /// The player entered room `room` (the level's case of 0x8007f498).
    fn room(&mut self, _g: &mut Game, _room: usize) {}
    /// The level's own models and HUD.
    fn draw(&self, _g: &Game, _moving: &mut Vec<Vtx>, _overlay: &mut Vec<Vtx>) {}
    /// The level timer ran out (TimerExpired's level callback).
    fn timer_expired(&mut self, _g: &mut Game) {}
    /// The phase, for tools.
    fn phase(&self) -> u8 {
        0
    }
    /// Jumps to phase `phase` as the level would set it up (for tools).
    fn skip_to(&mut self, _g: &mut Game, _phase: u8) {}
    /// The level's own state in words, for tools.
    fn describe(&self) -> String {
        String::new()
    }
    /// Activity golf: player `player` (0-3) takes the turn as `character`
    /// (0x80096a30 step 6).
    fn activity_turn(&mut self, _g: &mut Game, _player: usize, _character: usize) {}
    /// What the level's ambience reads (crate::audio::ambience): level 2 and 3
    /// +0x30 bit 8, level 9 the deaths to go, level 11 the ring step,
    /// level 12 the refrigerator opened, golf 15 in the stroke.
    fn cue(&self) -> i32 {
        0
    }
}

pub struct Game {
    pub level: Level,
    pub set: Set,
    pub col: Col,
    pub anims: Anims,
    pub player: Player,
    pub stage: Stage,
    pub trig: Trig,
    focal: f32,
    /// The camera's eye as drawn, and the follow camera (0x80027304).
    pub eye: Vec3,
    follow: Follow,
    /// The follow camera ran last frame (camera +0x4a = 1); a fixed
    /// camera's end sets it up again (0x80025e90).
    following: bool,
    /// The look aside (L1, L2, R2).
    look: Look,
    /// The rooms and objects at rest, drawn under the player.
    scenery: Vec<Vtx>,
    /// Where the level ends: a level to go to (the exit code, 0x800c4918)
    /// and a cinematic to play first.
    pub exit: Option<Exit>,
    /// A cinematic the level asks for (PostCine), played with the level
    /// waiting.
    pub cine: Option<usize>,
    /// The last checkpoint reached (position, heading).
    pub checkpoint: Option<Pose>,
    /// Tries left (state +0x23, from 0x800b2920[level] at the start), and
    /// the cry's time left (0x8007f920: up to 2100 dt).
    pub tries: i32,
    crying: i32,
    /// The level's own play, and how a level ended (0x80076d6c's won).
    pub script: Option<Box<dyn Script>>,
    pub won: Option<bool>,
    /// The HUD's sprites, and the health meter's time left and last value.
    pub sprites: Option<Sprites>,
    /// The effect flights and their animated sprites (0x8008864c).
    pub flights: crate::world::flights::Flights,
    meter_shown: i32,
    meter_health: i32,
    /// The placed objects' collision boxes in the world.
    pub boxes: Vec<Box3>,
    /// The placed characters other than the player.
    pub npcs: Vec<Npc>,
    /// Each character's room (+0x3b): found from its place at load
    /// (0x8002b660 with no hint), then kept up as it moves, the last one
    /// found kept when none is. One in no room is in no room's object
    /// list, so it is never drawn.
    pub npc_rooms: Vec<Option<usize>>,
    /// The posts (LP kind 2), what the NPCs asked for this frame, and the
    /// level's rand.
    pub posts: Vec<Post>,
    pub npc_events: Vec<Event>,
    pub rand: Rand,
    /// The room the player is in, and a song the room asked for
    /// (RoomChange, 0x80077a54: room flag 128, song at +0x64 less one).
    room: Option<usize>,
    pub room_song: Option<usize>,
    /// A song a level starts at once (MidiPlaySong).
    pub song: Option<usize>,
    /// Sound effects asked for this frame, with their volume (0x8006f644:
    /// the effects setting, less with distance from the camera).
    pub effects: Vec<Effect>,
    /// Paused (Start, 0x80018c38 step 13): the world holds still and the
    /// pause menu runs (0x8003e05c).
    pub paused: bool,
    pause: Pause,
    pub fonts: Fonts,
    /// Quit to the front end (the exit code -1).
    pub quit: bool,
    /// The game's end: the progress cleared, the credits, the hub.
    pub ending: bool,
    /// The objects that can be picked up, the one carried, and its grip
    /// (+0x1f8: in the player's frame from hand slot 15).
    pub things: Vec<Thing>,
    /// Bars, cookies and the levels' other collectibles; the bars
    /// available (0x800c4194: found less spent, kept in step here)
    /// here (0x80094a58) and how long the counter shows.
    pub goodies: Vec<Goodie>,
    pub bars: u32,
    bar_shown: i32,
    /// Bars found this frame (0x80094a58).
    pub found: Vec<usize>,
    /// The game state as main has it (for a save), and a record the
    /// pause's Load Game read (0x80070970).
    pub state_block: [u8; STATE_SIZE],
    pub loaded: Option<Box<[u8; STATE_SIZE]>>,
    carried: Option<usize>,
    grip: Vec3,
    /// Objects hidden by id (0x80078bc4) and the placements always drawn
    /// apart (characters, pickables, collectibles).
    hidden: std::collections::HashSet<u16>,
    apart: Vec<bool>,
    /// Placements hidden one by one (hide_placement).
    hidden_places: std::collections::HashSet<usize>,
    /// Collectibles taken this frame.
    pub collected: Vec<Collected>,
    /// Placements whose use was taken away (+0x54 = 0).
    unusable: std::collections::HashSet<usize>,
    /// The moving platforms (template class 11).
    pub movers: Vec<Mover>,
    /// A throw waiting for its animation's release, and a pick-up for its
    /// grab.
    pub pending_throw: Option<things::PendingThrow>,
    pub pending_grab: Option<usize>,
    /// Spike's water steps to the next splash sound in level 4
    /// (0x800c42e4).
    pub(super) water_steps: i32,
    /// The objects' boxes this frame, without things and characters: what
    /// a thrown thing lands on and bounces off.
    pub solid: Vec<Box3>,
    /// Which carried-over faces its VRAM has (the app's count).
    pub face_serial: u64,
    /// Touch 1's sound count (0x800c4ba0).
    pub(super) shelf_sound: i32,
    /// The piano's last note (0x800c4358): the same one is not played
    /// again.
    pub(super) last_note: Option<u8>,
    /// The C library's rand (0x800b1158: x * 0x41c64e6d + 0x3039 at
    /// 0x800eaf80), which picks the foot of a flag 4 puff.
    pub(super) crand: u32,
    /// The dust's coin flips standing and moving (0x800c42e0, 0x800c42ec,
    /// seeded 0xa8c0 and 0x2fdd).
    pub(super) dust_seeds: [Rand; 2],
    /// The breakables (template class 14: level 7's buildings).
    pub breakables: Vec<Breakable>,
    /// The hinged and sliding doors (classes 4 and 3).
    pub doors: Vec<Door>,
    /// Spinning toys (class 2) and post walkers (class 15).
    pub spinners: Vec<Spinner>,
    pub walkers: Vec<Walker>,
    /// Thrown objects that struck a character this frame (0x80086ef0).
    pub struck: Vec<Strike>,
    /// A fixed camera (eye, look-at) a level sets (0x800291f8 kind 4).
    pub fixed: Option<(Vec3, Vec3)>,
    /// The follow camera's height (0x800c3f5c) and distance (0x800c4a4c),
    /// and the post a room's fixed camera stands at (kind 3, 0x800259a0).
    eye_up: i32,
    behind: i32,
    post_camera: Option<usize>,
    /// Objects that hurt on touch (not pickable, not characters, +0x6d
    /// over 0: Damage(player, 0, +0x6d), 0x8005e1f4), and the time until
    /// the next hurt.
    hazards: Vec<Hazard>,
    hurt_wait: i32,
    /// The level timer (0x800c4430, dt left), set by a level.
    pub timer: Option<i32>,
    /// The level a finish is recorded for (golf records level 8).
    pub record: Level,
    /// The puzzle board asked for (its object's use, 0x8004d3f0).
    pub board: bool,
    /// The last frame's camera: eye and world-to-camera rows; frames run.
    pub view: (Vec3, Mat3),
    pub frames: u32,
    /// The playable character (state +0x08).
    pub character: usize,
    /// The spawn spots (LP kind 7).
    pub spots: Vec<Spot>,
    /// A hint card the level asks for (0x80044434).
    pub card: Option<usize>,
    /// Throws aim at characters (level 9's case of 0x80061c98).
    pub aim: bool,
    /// A bonus level's fourth bar was earned (0x80076ba8 and friends).
    pub fourth_bar: bool,
    /// Activity play (+0x21).
    pub activity: bool,
    /// The progress flags (+0x30) when the level opened: bit 16
    /// (Mirrorland entered) opens the hub's doors to 3, 4 and 6 and the
    /// mirror.
    pub flags: u32,
    /// Screens a level asks for (the golf box), shown with the level
    /// frozen.
    pub pages: Vec<Page>,
    /// The training messages (levels 1 and 16).
    pub training: Training,
    /// Speech lines asked for this frame (0x8008da4c), and the level's
    /// lines.
    pub speech: Vec<u16>,
    /// A line said at once over whatever plays (0x8008df78: the cry's).
    pub say_now: Option<u16>,
    pub lines: Speech,
    /// Placements that speak when the player comes near (LP +0x4c, the
    /// trigger list 0x80072d38).
    talkers: Vec<Talker>,
    /// The ambience countdown (0x800c4cc0), its first-line flag
    /// (0x800c4cc4) and its line tables.
    pub ambience: i32,
    pub ambience_first: bool,
    ambience_lines: std::rc::Rc<Tables>,
    /// Level 5's camera hold while floating (0x800c4a50) and its shaft
    /// camera (kind 3 walking, rooms 15 and 40).
    cam_hold: bool,
    shaft: bool,
    /// Damage events are dropped (level 1, 0x800780d8).
    pub no_damage: bool,
    /// The levels won (+0x38), for the hub's signs, and the pieces earned
    /// (0x80094b8c) for the pause.
    pub won_levels: u32,
    pub pieces: u32,
    /// Activity golf: the strokes of the hole just holed.
    pub strokes: Option<u32>,
}

/// The HUD's bar and clock icons (sprites; their sizes are guesses).
const BAR_ICON: u32 = 291;

const PIECE_ICON: u32 = 292;

const CLOCK_ICON: u32 = 375;

/// Text in font 1 with its shadow 2 down and right (0x80090004).
pub fn shadowed(fonts: &Fonts, x: i16, y: i16, string: String, out: &mut Vec<Vtx>) {
    let t = Text { font: 1, spacing: 1, rgb: [255, 255, 255], x, y, string, ..Default::default() };
    let mut shadow = t.clone();
    shadow.rgb = [1, 1, 1];
    shadow.x += 2;
    shadow.y += 2;
    shadow.triangles(fonts, out);
    t.triangles(fonts, out);
}

/// The hub mirror (templates 836, 881) and the locked door's card.
const MIRROR_A: u32 = 836;

const MIRROR_B: u32 = 881;

const LOCKED_CARD: usize = 712;

/// The puzzle board's object (template 581, model 905; use callback 1,
/// 0x8004d3f0).
const BOARD_OBJECT: u32 = 581;

/// How near (in x and z, and in y) the player must stand to use a door.
const USE_REACH: i32 = 1500;

const USE_REACH_Y: i32 = 1500;

/// The pie (template 564) and training's box (1016) for the messages.
const PIE: u16 = 564;

const HIGH_BOX: u16 = 1016;

/// Using these shows message 34 (training's hub door objects) or 33 (the
/// hub's locked doors) (0x8004d49c).
const EPISODE_OBJECTS: [u32; 2] = [111, 441];

const LOCKED_OBJECTS: [u32; 3] = [117, 715, 720];

impl Game {
    /// Plays sound effect `sound` at full volume (0x8006f26c).
    pub fn play(&mut self, sound: u8) {
        self.effects.push(Effect::new(sound));
    }

    /// Plays sound effect `sound` at `pos` (0x8006f644): heard from the
    /// camera, quieter with distance and, unless mono, panned by its
    /// bearing from the camera's heading.
    pub fn play_at(&mut self, sound: u8, pos: [i32; 3]) {
        if let Some(at) = self.heard_at(pos) {
            self.effects.push(Effect { sound, at: Some(at), key: None });
        }
    }

    /// Keys on program `prog`'s tone `tone` at `note` (0x800a99a8), at
    /// `pos` as `play_at` hears it.
    pub fn key_at(&mut self, key: [u8; 3], pos: [i32; 3]) {
        if let Some(at) = self.heard_at(pos) {
            self.effects.push(Effect { sound: 0, at: Some(at), key: Some(key) });
        }
    }

    /// The volumes a sound at `pos` is heard at (0x8006f4c0), none when
    /// too far off.
    fn heard_at(&self, pos: [i32; 3]) -> Option<(i32, i32)> {
        let (eye, rows) = self.view;
        let d = [pos[0] - eye.x as i32, pos[1] - eye.y as i32, pos[2] - eye.z as i32];
        if d[0].abs() > 0xfffe || d[1].abs() >= 0x7fff || d[2].abs() > 0xfffe {
            return None;
        }
        let sq = |v: i32| (v as i64) * (v as i64);
        let dist = (sq(d[0]) >> 14) + (sq(d[1]) >> 12) + (sq(d[2]) >> 14);
        let audio = Progress { block: self.state_block }.audio();
        let v = falloff(dist, audio.effects);
        let at = if audio.mono {
            (v, v)
        } else {
            // Bearings as headings: 0 faces +z, 1024 +x (0x80021590).
            let bearing = |x: f32, z: f32| (x.atan2(z) * 4096.0 / std::f32::consts::TAU) as i32;
            let ahead = rows.row(2);
            let angle = bearing(d[0] as f32, d[2] as f32) - bearing(ahead.x, ahead.z);
            pan(&self.trig, angle.rem_euclid(4096), v)
        };
        Some(at)
    }

    pub fn open(disc: &GameDisc, level: Level) -> Result<Game, String> {
        Game::open_from(disc, level, None, &Progress::default())
    }

    /// Level `level`, come to from level `from`: back in the hub the
    /// player stands at 0x800b8718[from] (16 bytes: x, y, z), as level 1's
    /// EpisodeStart callback (0x80078e7c) puts it, unless from training.
    pub fn open_from(disc: &GameDisc, level: Level, from: Option<Level>, progress: &Progress) -> Result<Game, String> {
        let who = level.character();
        let db = level.index();
        // Level 12's Tommy is hierarchy 44 (0x8005981c).
        let body = if level == Level::LET_THERE_BE_LIGHT { 44 } else { who };
        let set = Set::load(disc, db)?;
        let col = Col::parse(&disc.file(COL + db)?)?;
        let anims = Anims::parse(&disc.file(ANM + db)?, &disc.file(FRM + db)?, &disc.file(ROT + db)?)
            .map_err(|e| e.to_string())?;
        let trig = Trig::load(disc)?;
        let start = disc.exe_bytes(STARTS + 16 * db as u32, 12)?;
        let start = [0, 4, 8].map(|k| i32::from_le_bytes(start[k..k + 4].try_into().unwrap()));
        let angles = disc.exe_bytes(START_ANGLES + 8 * db as u32, 2)?;
        let heading = u16::from_le_bytes([angles[0], angles[1]]) as i32 & 4095;
        // The placements loader (0x800454dc) moves the player onto the LP
        // record whose template is the character's hierarchy, turned by
        // its quaternion (0x8001f684, 0x80059a88).
        let mine = |p: &&Placement| {
            p.kind == 1 && set.templates.get(p.template as usize).is_some_and(|t| t.hierarchy() == Some(body))
        };
        let (start, heading) = match set.placements.iter().find(mine) {
            Some(p) => {
                let [_, y, _, w] = p.rotation.map(|v| v as f64);
                let yaw = (2.0 * y.atan2(w) / std::f64::consts::TAU * 4096.0).round() as i32;
                (p.pos, yaw.rem_euclid(4096))
            }
            None => (start, heading),
        };
        let start = match from.filter(|&f| level == Level::HUB && f != Level::HUB && f != Level::TRAINING) {
            Some(f) => {
                let b = disc.exe_bytes(0x800b_8718 + 16 * f.index() as u32, 12)?;
                let spot = [0, 4, 8].map(|k| i32::from_le_bytes(b[k..k + 4].try_into().unwrap()));
                if spot == [0; 3] { start } else { spot }
            }
            None => start,
        };
        rrt::tracing::info!("level {level}: the player starts at {start:?}, heading {heading}");
        let hierarchy = set.hierarchies.get(body).cloned().flatten().ok_or("no hierarchy for the player")?;
        let mut player = Player::new(disc, hierarchy.clone(), start, heading, &col, &anims)?;
        if who != TOMMY {
            // Another body (0x80059328): Spike turns slowly (0x80062eac).
            let a = Animator::load(disc, who as u16)?;
            let turn = if matches!(who, 6 | 21) { SLOW_TURN } else { TURN };
            player.set_body(hierarchy, a, turn, &anims);
            player.pos = start;
        }
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
                    .is_some_and(|t| t.pickable() && t.hierarchy().is_none() && set.models.has(t.shape as u32))
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
                    keep: t.raw[4] == 13,
                    world_only: false,
                    shape: t.shape as u32,
                    pos: Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32),
                    rot: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    state: ThingState::Rest,
                    last_hit: None,
                    bounced: false,
                }
            })
            .collect();
        // Collectibles: template class 12 (0x80049af4, which makes them
        // with 0x80048a44: object class 11), the kind template +0x26.
        let collectible = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1
                && !character(k)
                && set
                    .templates
                    .get(p.template as usize)
                    .is_some_and(|t| t.raw[4] == 12 && set.models.get(t.shape as u32).is_some())
        };
        // Bars are numbered in the order they are made (0x800c4ba8); a bar
        // found before stays found (+0x48).
        let mut bar = 0usize;
        // The spawn spots (kind 7); level 4's loader moves entries 1 and 5
        // (0x80045e08).
        let mut spots: Vec<Spot> = set.placements.iter().filter_map(Spot::of).collect();
        if level == Level::CYNTHIA {
            if let Some(s) = spots.get_mut(1) {
                s.pos[1] = 900;
            }
            if let Some(s) = spots.get_mut(5) {
                s.pos[1] = 825;
            }
        }
        let mut rand = Rand::default();
        // Collectibles: LP +0x46 copies each (0x80049af4), placed at their
        // spawn group's spots when they have one (0x80045f34).
        let mut goodies: Vec<Goodie> = Vec::new();
        for k in (0..set.placements.len()).filter(|&k| collectible(k)) {
            let p = &set.placements[k];
            let t = &set.templates[p.template as usize];
            let kind = u16::from_le_bytes([t.raw[0x26], t.raw[0x27]]);
            for _ in 0..p.copies.max(1) {
                let number = if kind == goodie::BAR {
                    bar += 1;
                    bar - 1
                } else {
                    0
                };
                // No copies: the one object stays where it is placed. Many
                // copies not placed at load (+0x4a 0) wait for the level
                // (level 15's balloons).
                let group = p.group.filter(|_| p.copies > 0);
                let held = !p.placed_at_load && p.copies > 1;
                let at = group.and_then(|g| spawn(&mut spots, g as usize, &mut rand)).map_or(p.pos, |i| spots[i].pos);
                goodies.push(Goodie {
                    number,
                    template: p.template as u16,
                    group: p.spawn_group,
                    taken: held || kind == goodie::BAR && progress.bar_found(level, number),
                    shape: t.shape as u32,
                    kind,
                    value: t.raw[0x32] as i32,
                    pos: Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32),
                    rot: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                });
            }
        }
        let pickable = |k: usize| pickable(k) && !collectible(k);
        // Movers: template class 11 (0x80049ab0, 0x80048804).
        let mover = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1
                && !character(k)
                && set
                    .templates
                    .get(p.template as usize)
                    .is_some_and(|t| t.raw[4] == 11 && set.models.has(t.shape as u32))
        };
        let movers: Vec<Mover> = (0..set.placements.len())
            .filter(|&k| mover(k))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                let h = |at: usize| i16::from_le_bytes([t.raw[at], t.raw[at + 1]]) as i32;
                Mover {
                    id: p.id,
                    template: p.template as u16,
                    shape: t.shape as u32,
                    start: p.pos,
                    pos: p.pos,
                    rot: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    step: [h(0x18), h(0x1a), h(0x1c)],
                    range: [h(0x1e), h(0x20), h(0x22)],
                    gone: [0; 3],
                    out: [false; 3],
                    back_and_forth: t.flags & 0x1000_0000 != 0,
                    delta: [0; 3],
                    radius: u16::from_le_bytes([t.raw[0x34], t.raw[0x35]]) as i32,
                    armed: false,
                    inside: false,
                    ridden_only: t.flags & 0x4000_0000 != 0,
                    calls: p.template == 855,
                }
            })
            .collect();
        // Breakables: template class 14 (0x80049c50, think 0x8004d7c4).
        let breakable = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1 && !character(k) && set.templates.get(p.template as usize).is_some_and(|t| t.raw[4] == 14)
        };
        let breakables: Vec<Breakable> = (0..set.placements.len())
            .filter(|&k| breakable(k))
            .map(|k| {
                let p = &set.placements[k];
                Breakable {
                    template: p.template as u16,
                    shape: set.templates[p.template as usize].shape as u32,
                    pos: Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32),
                    rot: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    broken: false,
                    gone: false,
                }
            })
            .collect();
        // Doors: template class 3 and 4 (not level 5's 843 or golf's
        // automatic 770).
        let door = |k: usize| {
            let p = &set.placements[k];
            p.kind == 1
                && !character(k)
                && p.template != 843
                && set
                    .templates
                    .get(p.template as usize)
                    .is_some_and(|t| matches!(t.raw[4], 3 | 4 | 8) && set.models.has(t.shape as u32))
        };
        let doors: Vec<Door> = (0..set.placements.len())
            .filter(|&k| door(k))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                let h = |at: usize| i16::from_le_bytes([t.raw[at], t.raw[at + 1]]) as i32;
                let [_, qy, _, qw] = p.rotation.map(|v| v as f64);
                let heading = ((2.0 * qy.atan2(qw) / std::f64::consts::TAU * 4096.0).round() as i32).rem_euclid(4096);
                Door {
                    placement: k,
                    template: p.template as u16,
                    shape: t.shape as u32,
                    hinged: t.raw[4] != 3,
                    start: p.pos,
                    placed: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    heading,
                    offset: [0; 3],
                    speed: [h(0x18), h(0x1a), h(0x1c)],
                    shut: h(0x1e),
                    open_end: h(0x20),
                    target: if t.raw[4] == 8 { h(0x20) } else { 0 },
                    either: h(0x26) != 0,
                    sound: t.raw[0x24],
                    open: false,
                    moving: false,
                    // Class 8, and class 3 with flag 0x40000000 (golf's
                    // 770): automatic.
                    swing: (t.raw[4] == 8 || (t.raw[4] == 3 && t.flags & 0x4000_0000 != 0)).then(|| Swing {
                        ease: h(0x18),
                        waits: if t.flags & 0x1000_0000 != 0 { 16 } else { 0 }
                            | if t.flags & 0x2000_0000 != 0 { 8 } else { 0 },
                        wait: 0,
                        radius: u16::from_le_bytes([t.raw[0x34], t.raw[0x35]]) as i32,
                        damage: t.raw[0x32] as i32,
                        hurt: 0,
                    }),
                }
            })
            .collect();
        let class_of = |k: usize, c: u8| {
            let p = &set.placements[k];
            p.kind == 1
                && !character(k)
                && !pickable(k)
                && set
                    .templates
                    .get(p.template as usize)
                    .is_some_and(|t| t.raw[4] == c && set.models.has(t.shape as u32))
        };
        let radius_of = |t: &Template| u16::from_le_bytes([t.raw[0x34], t.raw[0x35]]) as i32;
        let spinners: Vec<Spinner> = (0..set.placements.len())
            .filter(|&k| class_of(k, 2))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                let h = |at: usize| i16::from_le_bytes([t.raw[at], t.raw[at + 1]]) as i32;
                Spinner {
                    placement: k,
                    shape: t.shape as u32,
                    pos: Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32),
                    placed: models::quat_matrix(p.rotation.map(|v| v as i16)).transpose(),
                    speed: [h(0x18), h(0x1a), h(0x1c)],
                    angle: [0; 3],
                    radius: radius_of(t),
                }
            })
            .collect();
        let walkers: Vec<Walker> = (0..set.placements.len())
            .filter(|&k| class_of(k, 15))
            .map(|k| {
                let p = &set.placements[k];
                let t = &set.templates[p.template as usize];
                let [_, qy, _, qw] = p.rotation.map(|v| v as f64);
                let heading = ((2.0 * qy.atan2(qw) / std::f64::consts::TAU * 4096.0).round() as i32).rem_euclid(4096);
                Walker {
                    placement: k,
                    template: p.template as u16,
                    shape: t.shape as u32,
                    pos: p.pos,
                    heading,
                    speed: p.spawn_group as i32 / 300,
                    reach: p.word_4a as i32,
                    post: p.copies as u8,
                    prev: 0,
                    radius: radius_of(t),
                    stopped: false,
                }
            })
            .collect();
        let spun = |k: usize| class_of(k, 2) || class_of(k, 15);
        let apart: Vec<bool> = (0..set.placements.len())
            .map(|k| character(k) || pickable(k) || collectible(k) || mover(k) || breakable(k) || door(k) || spun(k))
            .collect();
        let scenery = set.objects_except(&Frame::default(), |k| apart[k]);
        let mut npcs = Vec::new();
        for (k, p) in set.placements.iter().enumerate() {
            if !character(k) || Some(k) == player_place {
                continue;
            }
            let Some(h) = set.templates.get(p.template as usize).and_then(|t| t.hierarchy()) else { continue };
            // Level 12's Spike is hierarchy 45 (0x80048284).
            let h = if level == Level::LET_THERE_BE_LIGHT && h == 6 { 45 } else { h };
            let Some(Some(hierarchy)) = set.hierarchies.get(h).cloned() else { continue };
            let mut animator = Animator::load(disc, h as u16)?;
            animator.set(IDLE, &anims);
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
                mode: Mode::Idle,
                home: p.pos,
                hidden: false,
                attacks: false,
                cooldown: 0,
                first_post: p.post,
                loops: (p.level as i32, p.level as i32),
                size: 0,
                rooted: false,
                bumped: false,
                reach: 600,
                bite_wait: 900,
                run_speed: None,
                post_reach: REACH,
                hold: None,
            };
            // On the floor under it (0x80081bac asks 0x8002b240), the root
            // the hierarchy's offset above the feet.
            n.size = n.hierarchy.half_size();
            n.place(p.pos, &col);
            npcs.push(n);
        }
        let focal = trig.geom_screen(FOV).unwrap_or(256) as f32;
        let boxes = object_boxes(&set, |k| apart[k]);
        let posts = npc::posts(&set.placements);
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
            follow: Follow::default(),
            following: true,
            look: Look::default(),
            scenery,
            exit: None,
            cine: None,
            checkpoint: None,
            tries: disc.exe_bytes(TRIES + db as u32, 1).map_or(3, |b| b[0] as i8 as i32),
            crying: 0,
            script: None,
            won: None,
            sprites: Sprites::load(disc, db).ok(),
            flights: crate::world::flights::Flights::load(disc)?,
            meter_shown: 0,
            meter_health: 255,
            boxes,
            posts,
            npc_events: Vec::new(),
            rand,
            npc_rooms: Vec::new(),
            npcs,
            room: None,
            song: None,
            room_song: None,
            effects: Vec::new(),
            paused: false,
            pause: Pause::load(disc, level)?,
            fonts: Fonts::load(disc)?,
            quit: false,
            ending: false,
            things,
            goodies,
            bars: progress.bars_available(),
            bar_shown: 0,
            found: Vec::new(),
            state_block: [0; STATE_SIZE],
            loaded: None,
            carried: None,
            grip: Vec3::ZERO,
            hidden: Default::default(),
            apart,
            unusable: Default::default(),
            hidden_places: Default::default(),
            collected: Vec::new(),
            struck: Vec::new(),
            movers,
            pending_throw: None,
            pending_grab: None,
            water_steps: 0,
            solid: Vec::new(),
            face_serial: 0,
            shelf_sound: 0,
            last_note: None,
            crand: 0,
            dust_seeds: [Rand { state: 0xa8c0 }, Rand { state: 0x2fdd }],
            breakables,
            doors,
            spinners,
            walkers,
            fixed: None,
            eye_up: default_eye_up(level),
            behind: if level == Level::REPTAR_2010 { BEHIND_LEVEL_7 } else { BEHIND },
            post_camera: None,
            hazards: Vec::new(),
            hurt_wait: 0,
            timer: None,
            record: level,
            board: false,
            view: (Vec3::ZERO, Mat3::IDENTITY),
            frames: 0,
            pages: Vec::new(),
            character: who,
            spots,
            card: None,
            aim: false,
            fourth_bar: false,
            activity: progress.activity(),
            flags: progress.flags(),
            training: Training::load(disc, level),
            speech: Vec::new(),
            say_now: None,
            lines: Speech::load(disc, db),
            talkers: Vec::new(),
            ambience: ambience::setup(level).0,
            ambience_first: ambience::setup(level).1,
            ambience_lines: std::rc::Rc::new(Tables::load(disc)),
            // Damage is dropped in levels 1, 11, 17 and 23-31 (0x800780d8).
            no_damage: level.no_damage(),
            cam_hold: false,
            shaft: false,
            won_levels: (0..32).map(Level).filter(|&l| progress.won(l)).fold(0, |a, l| a | l.bit()),
            pieces: progress.pieces(),
            strokes: None,
        };
        g.talkers = talkers(&g.set, level);
        g.player.falls_hurt = !g.no_damage;
        // Spinners still hurt on touch where they stand (their +0x6d).
        let hazard_skip: Vec<bool> =
            (0..g.apart.len()).map(|k| g.apart[k] && !g.spinners.iter().any(|s| s.placement == k)).collect();
        g.hazards = hazards(&g.set, &hazard_skip);
        g.script = crate::levels::script(level, &mut g, disc)?;
        // The camera reset behind the player (0x800290a4).
        let heading = g.player.heading;
        g.set_camera_yaw(heading as f32);
        Ok(g)
    }

    /// Hides or shows placement `k` of the scenery (a level's own doors).
    pub fn hide_placement(&mut self, k: usize, hidden: bool) {
        let changed = if hidden { self.hidden_places.insert(k) } else { self.hidden_places.remove(&k) };
        if changed {
            let skip = |j: usize| {
                self.apart[j] || self.hidden.contains(&self.set.placements[j].id) || self.hidden_places.contains(&j)
            };
            self.scenery = self.set.objects_except(&Frame::default(), skip);
            self.boxes = object_boxes(&self.set, skip);
        }
    }

    /// Changes the scenery placements with registry id `id` (a level's
    /// own set-up: a position, a template) and rebuilds what is drawn and
    /// collided with.
    pub fn edit_placements(&mut self, id: u16, f: impl Fn(&mut Placement)) {
        for p in self.set.placements.iter_mut().filter(|p| p.id == id) {
            f(p);
        }
        let skip = |j: usize| {
            self.apart[j] || self.hidden.contains(&self.set.placements[j].id) || self.hidden_places.contains(&j)
        };
        self.scenery = self.set.objects_except(&Frame::default(), skip);
        self.boxes = object_boxes(&self.set, skip);
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
            let skip = |k: usize| {
                self.apart[k] || self.hidden.contains(&self.set.placements[k].id) || self.hidden_places.contains(&k)
            };
            self.scenery = self.set.objects_except(&Frame::default(), skip);
            self.boxes = object_boxes(&self.set, skip);
        }
    }

    /// A world point in the camera's frame (x right, y down, z ahead).
    pub fn to_view(&self, p: Vec3) -> Vec3 {
        self.view.1 * (p - self.view.0)
    }

    /// Whether id `id` is hidden.
    pub fn is_hidden(&self, id: u16) -> bool {
        self.hidden.contains(&id)
    }

    /// The room the player is in.
    pub fn current_room(&self) -> Option<usize> {
        self.room
    }

    /// Takes the use away from the objects with id `id` (+0x54 = 0).
    pub fn disable_use(&mut self, id: u16) {
        for (k, p) in self.set.placements.iter().enumerate() {
            if p.kind == 1 && p.id == id {
                self.unusable.insert(k);
            }
        }
    }

    /// One frame of the level (0x80018c38): the player, then the camera,
    /// then the draw.
    pub fn frame(&mut self, pads: &Pads) {
        self.frames = self.frames.wrapping_add(1);
        self.effects.clear();
        self.speech.clear();
        if !self.paused && pads.pressed(0, input::START) {
            self.paused = true;
            self.pause.open();
            self.effects.clear();
            self.draw();
            return;
        }
        if self.paused {
            self.effects.clear();
            match self.pause.frame(pads, &self.state_block) {
                Some(PauseEnd::Continue) => self.paused = false,
                Some(PauseEnd::QuitGame) => self.quit = true,
                Some(PauseEnd::Loaded(state)) => {
                    self.paused = false;
                    self.loaded = Some(state);
                }
                // Quit Level: to Pickles' Home, or the front end from the
                // hub and training (0x800398bc).
                Some(PauseEnd::QuitLevel) => {
                    if matches!(self.level, Level::HUB | Level::TRAINING) {
                        self.quit = true;
                    } else {
                        self.exit = Some(Exit::to(Level::HUB));
                    }
                }
                None => {}
            }
            self.effects.extend(self.pause.sounds.drain(..).map(Effect::new));
            self.draw();
            return;
        }
        if self.crying > 0 {
            // The cry runs on its own (0x8007f920's loop): the world waits.
            self.crying -= DT;
            self.player.animator.step(&self.anims);
            if self.crying <= 0 || !self.player.animator.playing(CRY) {
                self.crying = 0;
                self.cried();
            }
            self.draw();
            return;
        }
        // A training message (0x80097584): no player or world update, the
        // level timer stopped; the camera and drawing go on.
        // The ambience runs even while a message holds the world.
        let cue = self.script.as_ref().map_or(0, |s| s.cue());
        let lines = self.ambience_lines.clone();
        ambience::frame(self, &lines, cue);
        let mut closed = Vec::new();
        if self.training.frame(pads, &mut closed) {
            self.effects.extend(closed.into_iter().map(Effect::new));
            self.draw();
            return;
        }
        // Movers step (0x8004ee00), carrying a player who stands on one.
        let standing_on = self.mover_under_player();
        let me = self.player.pos;
        let level5 = self.level == Level::OUTER_SPACE;
        let tests: Vec<(bool, bool)> = self.movers.iter().map(|m| self.mover_tests(m)).collect();
        for (k, m) in self.movers.iter_mut().enumerate() {
            let (over, blocked) = tests[k];
            let elevator = level5 && m.template == 855;
            m.step(me, standing_on == Some(k), over, blocked, elevator);
        }
        if let Some(k) = standing_on {
            let d = self.movers[k].delta;
            for i in 0..3 {
                self.player.pos[i] += d[i];
            }
        }
        let mut boxes = self.boxes.clone();
        for b in self.breakables.iter().filter(|b| !b.gone) {
            let template = if b.broken { b.template as usize + 1 } else { b.template as usize };
            let flag8 = self.set.templates.get(template).is_some_and(|t| t.flags & 8 != 0);
            let shape = if b.broken { b.shape + 1 } else { b.shape };
            if !flag8 && let Some(model) = self.set.models.get(shape) {
                boxes.extend(world_boxes(model, b.rot, b.pos).into_iter().map(|mut x| {
                    x.template = template as u16;
                    x
                }));
            }
        }
        for m in &self.movers {
            if let Some(model) = self.set.models.get(m.shape) {
                boxes.extend(world_boxes(model, m.rot, m.world()).into_iter().map(|mut b| {
                    b.template = m.template;
                    b
                }));
            }
        }
        self.toy_thinks();
        for s in &self.spinners {
            if let Some(model) = self.set.models.get(s.shape) {
                boxes.extend(world_boxes(model, s.rot(&self.trig), s.pos));
            }
        }
        for w in &self.walkers {
            if let Some(model) = self.set.models.get(w.shape) {
                boxes.extend(world_boxes(model, w.rot(&self.trig), w.world()));
            }
        }
        // Doors are solid where they are drawn (0x80069c40).
        self.door_thinks();
        for d in &self.doors {
            if let Some(model) = self.set.models.get(d.shape) {
                let (rot, pos) = d.pose(d.offset, &self.trig);
                boxes.extend(world_boxes(model, rot, pos));
            }
        }
        self.solid = boxes.clone();
        for t in self.things.iter().filter(|t| t.state == ThingState::Rest && !t.hidden) {
            if let Some(m) = self.set.models.get(t.shape) {
                let climb = self.set.templates.get(t.template as usize).is_some_and(|tp| tp.flags & 64 != 0);
                boxes.extend(world_boxes(m, t.rot, t.pos).into_iter().map(|mut b| {
                    b.climb = climb;
                    b.template = t.template;
                    b
                }));
            }
        }
        // The other characters stand in the way: a box of their radius
        // (object +0x86 is 0x2000 / 8 for a baby, guessed at 200) from the
        // feet up.
        for n in self.npcs.iter().filter(|n| !n.hidden && !matches!(n.mode, Mode::Ride { .. })) {
            let feet = (n.pos[1] - n.feet()) as f32;
            boxes.push(Box3 {
                lo: [-200, 0, -200],
                hi: [200, 0, 200],
                rot: Mat3::IDENTITY,
                pos: Vec3::new(n.pos[0] as f32, n.pos[1] as f32, n.pos[2] as f32),
                min_y: (feet - 700.0) as i32,
                max_y: feet as i32,
                climb: false,
                character: true,
                template: 0,
            });
        }
        // Climbs refused (0x80058da4): room 4 in levels 1 and 12 (0x80058e04),
        // ledges of type 14 (906-1033) in level 10 (0x80058e14).
        self.player.climb_off = matches!(self.level, Level::HUB | Level::LET_THERE_BE_LIGHT) && self.room == Some(4);
        self.player.climb_max = if self.level == Level::CHUCKIES_GLASSES { 905 } else { crate::actors::player::CLIMB };
        let start = self.player.pos;
        self.player.frame(pads, self.camera_heading(), &self.col, &boxes, &self.anims, &self.trig);
        // Moved across (+0x188 bit 32, from the displacement +0xd8).
        let moved = self.player.pos[0] != start[0] || self.player.pos[2] != start[2];
        for n in std::mem::take(&mut self.player.hints) {
            // The slip's start (state 3, 0x8005b3c4): speech 212 in level 16.
            if n == 32 && self.level == Level::TRAINING {
                self.speech.push(212);
            }
            self.training.show(n as usize);
        }
        self.walk_effects();
        if std::mem::take(&mut self.player.landed) {
            let on = self.player.support.and_then(|k| boxes.get(k)).map_or(0, |b| b.template);
            self.land_sounds(on);
        }
        self.carry(pads);
        self.fly();
        self.touch();
        self.effects.extend(closed.into_iter().map(Effect::new));
        // The player's sounds are in the world (0x800562bc's 0x8006f644).
        let level = self.level.0;
        let entered = self.player.animator.take_entered();
        let sounds = entered.into_iter().filter_map(|s| animator::entry_sound(s, self.character, level));
        let sounds: Vec<u8> = std::mem::take(&mut self.player.effects).into_iter().chain(sounds).collect();
        for sound in sounds {
            let at = self.player.pos;
            self.play_at(sound, at);
        }
        self.track_npc_rooms();
        for k in 0..self.npcs.len() {
            let n = &mut self.npcs[k];
            let (c, at) = (n.character, n.pos);
            let sounds: Vec<u8> =
                n.animator.take_entered().into_iter().filter_map(|s| animator::entry_sound(s, c, level)).collect();
            if !n.hidden {
                for sound in sounds {
                    self.play_at(sound, at);
                }
            }
        }
        // Using an object (the stand handler's ACTION, Triangle, on the
        // touched object): a door (use 0x8004f78c, LP +0x42) goes to its
        // level; one with only a cinematic (0x8004f83c, LP +0x44, made by
        // 0x8004691c: 16 is played as 14, template 567 is 8) plays it.
        let used_door =
            pads.pressed(0, input::TRIANGLE) && !self.player.carrying && self.carried.is_none() && self.use_door();
        if pads.pressed(0, input::TRIANGLE) && !self.player.carrying && self.carried.is_none() && !used_door {
            let p = self.player.pos;
            // Messages 34 and 33 (0x8004d49c); in training these objects
            // do nothing else.
            let hint = self.set.placements.iter().find(|d| {
                d.kind == 1
                    && !self.hidden.contains(&d.id)
                    && (EPISODE_OBJECTS.contains(&d.template) || LOCKED_OBJECTS.contains(&d.template))
                    && (d.pos[0] - p[0]).abs() < USE_REACH
                    && (d.pos[2] - p[2]).abs() < USE_REACH
                    && self.use_dy(d, p[1]) < USE_REACH_Y
            });
            if let Some(d) = hint {
                self.training.show(if EPISODE_OBJECTS.contains(&d.template) { 34 } else { 33 });
            }
            let training_object =
                |d: &Placement| self.level == Level::TRAINING && EPISODE_OBJECTS.contains(&d.template);
            let mirror = |d: &Placement| matches!(d.template, MIRROR_A | MIRROR_B);
            let flag16 = self.flags & 16 != 0;
            let cine_of = |d: &Placement| match (d.cine, d.template) {
                (16, _) => 14,
                (0, 567) => 8,
                (c, _) => c as usize,
            };
            let near = self
                .set
                .placements
                .iter()
                .enumerate()
                // A door object that can be picked up (the hub's doors to
                // 4, 5, 6, 11, 13) works where it lies.
                .map(|(k, d)| {
                    let thing = self.things.iter().find(|t| t.placement == k);
                    let mut d = *d;
                    if let Some(t) = thing {
                        d.pos = [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32];
                    }
                    (k, d, thing.map(|t| t.state == ThingState::Rest && !t.hidden))
                })
                .filter(|(k, d, thing)| {
                    d.kind == 1
                        && (!self.apart[*k] || *thing == Some(true))
                        && !training_object(d)
                        && (d.level > 0 || cine_of(d) > 0 || d.template == BOARD_OBJECT || mirror(d))
                        && (!mirror(d) || flag16)
                        && !self.unusable.contains(k)
                        && !self.hidden.contains(&d.id)
                })
                .map(|(_, d, _)| d)
                .min_by_key(|d| (d.pos[0] - p[0]).pow(2) as i64 + (d.pos[2] - p[2]).pow(2) as i64);
            if let Some(d) = near.filter(|d| {
                (d.pos[0] - p[0]).abs() < USE_REACH
                    && (d.pos[2] - p[2]).abs() < USE_REACH
                    && self.use_dy(d, p[1]) < USE_REACH_Y
            }) {
                if d.template == BOARD_OBJECT && self.flags & 512 != 0 {
                    // 0x8004d3f0: once Reptar 2010 has begun (+0x30 bit
                    // 512), the board goes back to it.
                    self.exit = Some(Exit::to(Level::REPTAR_2010));
                } else if d.template == BOARD_OBJECT {
                    self.board = true;
                } else if mirror(&d) {
                    // The mirror (0x800478bc): Mirrorland, with bit 16.
                    self.exit = Some(Exit { level: Level::MIRRORLAND, cine: (d.cine > 0).then_some(d.cine as usize) });
                } else if d.level > 0 && matches!(d.level, 3 | 4 | 6) && !flag16 {
                    // 0x8004f78c: "you'll need more puzzle pieces" (HINT41).
                    self.card = Some(LOCKED_CARD);
                } else if d.level > 0 {
                    rrt::tracing::info!("level {}: a door to level {} (cine {})", self.level, d.level, d.cine);
                    self.exit =
                        Some(Exit { level: Level(d.level as u8), cine: (d.cine > 0).then_some(d.cine as usize) });
                } else {
                    rrt::tracing::info!("level {}: object {} plays cine {}", self.level, d.id, cine_of(&d));
                    self.cine = Some(cine_of(&d));
                }
            }
        }
        // Select: the restart menu (0x80043d58), outside activity play.
        if pads.pressed(0, input::SELECT) && !self.activity {
            self.pages.push(Page::Restart);
        }
        // The look buttons (0x8002729c): L2/R2 left and right, L1 up;
        // only with the follow camera, not while floating (state 21).
        let following = self.fixed.is_none() && self.post_camera.is_none();
        if following && self.player.float.is_none() {
            for (button, yaw, pitch) in [(input::L1, 0, 341), (input::L2, -341, 0), (input::R2, 341, 0)] {
                if pads.pressed(0, button) {
                    self.look.start(yaw, pitch);
                }
            }
        }
        self.look.step(&self.trig);
        // R1: the camera back behind the player (0x80026ddc), the look
        // dropped; only with the follow camera, not while level 5's float
        // holds it.
        if pads.pressed(0, input::R1) && following && !self.shaft && !self.cam_hold {
            self.camera_behind();
        }
        let world = World {
            player: self.player.pos,
            player_heading: self.player.heading,
            col: &self.col,
            posts: &self.posts,
            trig: &self.trig,
            anims: &self.anims,
            walls: &boxes,
        };
        self.npc_events = npc::step(&mut self.npcs, &world, &mut self.rand);
        self.with_script(|s, g| s.frame(g, pads));
        self.flights.step(&self.col, self.sprites.as_ref(), DT);
        // Hazards (0x8005e1f4, the walk and run states): walking into one
        // costs its +0x6d; the next only after 300 dt (guess).
        self.hurt_wait = (self.hurt_wait - DT).max(0);
        let walking = matches!(self.player.state, State::Walk | State::Run);
        if self.hurt_wait == 0 && self.crying == 0 && walking && !self.player.frozen && !self.no_damage {
            let p = self.player.pos;
            let (feet, head) = (p[1] - self.player.feet(), p[1] - self.player.feet() - HEIGHT);
            let hurt = self
                .hazards
                .iter()
                .filter(|h| {
                    let b = &h.area;
                    b.covers(p, RADIUS + 40) && b.min_y <= feet + 40 && b.max_y >= head
                })
                .map(|h| h.damage)
                .max();
            if let Some(d) = hurt {
                self.player.hurt(d);
                self.hurt_wait = 300;
            }
        }
        // The level timer (0x8007f5a8): at 0, sound 117 and TimerExpired.
        if let Some(t) = self.timer {
            let t = t - DT;
            if t <= 0 {
                self.timer = None;
                // At the player (0x8007f5a8: 0x8006f644(117, player)).
                let me = self.player.pos;
                self.play_at(117, me);
                self.with_script(|s, g| s.timer_expired(g));
            } else {
                self.timer = Some(t);
            }
        }
        // Damage (0x80077864): health at 0 is a cry.
        if self.player.health <= 0 {
            self.cry();
        }
        self.meter_shown = (self.meter_shown - DT).max(0);
        if self.player.health != self.meter_health {
            self.meter_health = self.player.health;
            self.meter_shown = METER_TIME;
        }
        self.room_change();
        self.talk();
        self.camera_frame(moved);
    }

    /// The camera's frame (0x8002953c) and the picture from it, `moved`
    /// when the player moved across this frame. [`Game::frame`] ends with
    /// it; a tool that moves the player itself can run it alone.
    pub fn camera_frame(&mut self, moved: bool) {
        // Level 5: a held camera stays put; the shaft camera stands 5800
        // along z and AIM_UP + height up (0x80025c64 kind 12). A fixed
        // camera's end sets the follow camera up behind the player again
        // (0x800291f8 into 0x80025e90).
        let follows = self.fixed.is_none() && self.post_camera.is_none() && !self.shaft;
        if self.shaft {
            let t = self.target();
            self.eye = t + Vec3::new(0.0, -self.eye_up as f32, 5800.0);
        } else if follows && !self.following {
            let heading = self.player.heading;
            self.set_camera_yaw(heading as f32);
        } else if follows && !self.cam_hold {
            self.follow(moved);
        }
        self.following = follows;
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
                models::model_triangles(m, t.rot, t.pos, &mut moving);
            }
        }
        for m in &self.movers {
            if let Some(model) = self.set.models.get(m.shape) {
                models::model_triangles(model, m.rot, m.world(), &mut moving);
            }
        }
        for d in &self.doors {
            if let Some(model) = self.set.models.get(d.shape) {
                let (rot, pos) = d.pose(d.offset, &self.trig);
                models::model_triangles(model, rot, pos, &mut moving);
            }
        }
        for s in self.spinners.iter().filter(|s| !self.hidden.contains(&self.set.placements[s.placement].id)) {
            if let Some(model) = self.set.models.get(s.shape) {
                models::model_triangles(model, s.rot(&self.trig), s.pos, &mut moving);
            }
        }
        for w in self.walkers.iter().filter(|w| !self.hidden.contains(&self.set.placements[w.placement].id)) {
            if let Some(model) = self.set.models.get(w.shape) {
                models::model_triangles(model, w.rot(&self.trig), w.world(), &mut moving);
            }
        }
        for b in self.breakables.iter().filter(|b| !b.gone) {
            let shape = if b.broken { b.shape + 1 } else { b.shape };
            if let Some(model) = self.set.models.get(shape) {
                models::model_triangles(model, b.rot, b.pos, &mut moving);
            }
        }
        // Level 15's balloons (kinds 6-11) are sprites its script draws.
        let sprites_only = |g: &&Goodie| self.level == Level::MIRRORLAND && (6..=11).contains(&g.kind);
        for g in self.goodies.iter().filter(|g| !g.taken).filter(|g| !sprites_only(g)) {
            if let Some(m) = self.set.models.get(g.shape) {
                models::model_triangles(m, g.rot, g.pos, &mut moving);
            }
        }
        for (k, n) in self.npcs.iter().enumerate() {
            if self.npc_rooms.get(k).is_some_and(|r| r.is_some()) {
                npc::triangles(n, &self.set.models, &self.anims, &self.trig, &mut moving);
            }
        }
        for (shape, angles, at) in self.flights.models() {
            if let Some(m) = self.set.models.get(shape) {
                let e = self.trig.euler(angles);
                let e = e.as_flattened();
                let rot = Mat3::from_cols_array(&std::array::from_fn(|k| e[k] as f32 / 4096.0));
                let at = Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
                models::model_triangles(m, rot, at, &mut moving);
            }
        }
        // A broken object's pieces: the object's turn, then each piece's
        // own since it broke.
        for (shard, angles, at) in self.flights.shards() {
            let e = self.trig.euler(angles);
            let e = e.as_flattened();
            let spin = Mat3::from_cols_array(&std::array::from_fn(|k| e[k] as f32 / 4096.0));
            let at = Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
            models::model_triangles(&shard.model, shard.base * spin, at, &mut moving);
        }
        self.flights.draw(self.view, self.sprites.as_ref(), &mut moving);
        let mut overlay = Vec::new();
        if let Some(s) = &self.script {
            s.draw(self, &mut moving, &mut overlay);
        }
        // The health meter (levels 2-6, 9, 12, 14, 16: 0x8008c094's switch).
        if self.meter_shown > 0
            && self.level.has_meter()
            && let Some(s) = &self.sprites
        {
            hud::meter(s, self.player.health, 255, self.focal, &mut overlay);
        }
        self.stage.set_moving(moving);
        self.stage.overlay = overlay;
        if self.paused {
            self.pause.triangles(&self.fonts, &mut self.stage.overlay);
            // The counters (0x8003e05c): pieces earned with model 292 at
            // (-192, 76), bars available with 291 at (192, 76); on the
            // pause page only (0x80090174).
            let page = self.pause.card.is_none();
            if let (Some(sp), true) = (&self.sprites, page) {
                sp.draw(PIECE_ICON, (-192, 76), 26.0, 32.0, METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
                sp.draw(BAR_ICON, (192, 76), 26.0, 32.0, METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
            }
            if page {
                shadowed(&self.fonts, -160, 70, format!("{}", self.pieces), &mut self.stage.overlay);
                shadowed(&self.fonts, 150, 70, format!("{}", self.bars), &mut self.stage.overlay);
            }
        } else {
            // The level timer: "%ld" seconds at the top by the clock
            // (model 375).
            if let Some(t) = self.timer {
                if let Some(sp) = &self.sprites {
                    sp.draw(CLOCK_ICON, (-40, -96), 24.0, 24.0, METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
                }
                shadowed(&self.fonts, -12, -100, format!("{}", t / 300), &mut self.stage.overlay);
            }
            if self.bar_shown > 0 {
                // The bar counter (0x80090004): model 291 at (-192, -76)
                // and the bars available at (-150, -80); bottom left in
                // levels 17-18, bottom right in golf.
                let (icon, text) = match self.level {
                    Level::GOLD_RUSH | Level::EGG_HUNT => ((-192, 76), (-150, 80)),
                    l if l.is_hole() => ((192, 76), (145, 70)),
                    _ => ((-192, -76), (-150, -80)),
                };
                if let Some(sp) = &self.sprites {
                    sp.draw(BAR_ICON, icon, 26.0, 32.0, METER_DEPTH, 0.0, self.focal, &mut self.stage.overlay);
                }
                shadowed(&self.fonts, text.0, text.1, format!("{}", self.bars), &mut self.stage.overlay);
            }
            // The training message, after the HUD (0x80097564).
            self.training.draw(&self.fonts, &mut self.stage.overlay);
        }
        let post = self
            .post_camera
            .and_then(|k| self.posts.get(k.wrapping_sub(1)))
            .map(|p| (Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32), self.target()));
        // The follow camera looks at its look-at (0x800de454); the shaft
        // and the held camera at the player.
        let look = if self.shaft || self.cam_hold { self.target() } else { self.follow_look() };
        let (eye, target) = self.fixed.or(post).unwrap_or((self.eye, look));
        let f = (target - eye).normalize_or(Vec3::Z);
        let x = Vec3::Y.cross(f).normalize_or(Vec3::X);
        let y = f.cross(x);
        // The look turns the view about the eye (0x8001e838): yaw toward
        // the right, pitch up (y is down).
        let (x, y, f) = if self.fixed.is_none() && post.is_none() {
            let a = |v: i32| v as f32 * std::f32::consts::TAU / 4096.0;
            let (sy, cy) = a(self.look.yaw).sin_cos();
            let (x, f) = (x * cy - f * sy, f * cy + x * sy);
            let (sp, cp) = a(self.look.pitch).sin_cos();
            (x, y * cp + f * sp, f * cp - y * sp)
        } else {
            (x, y, f)
        };
        // World to camera: rows x (right), y (down), z (ahead).
        let rows = Mat3::from_cols(x, y, f).transpose();
        self.view = (eye, rows);
        self.stage.camera = crate::stage::view(eye, rows, self.focal);
    }
}

#[cfg(test)]
mod tests {
    use super::falloff;

    /// 0x8006fd68 with 0x800b8024's 128, 16384, 127: full near, a step of
    /// 1 each 128 further, nothing past 16384, never above the setting.
    #[test]
    fn positional_falloff() {
        assert_eq!(falloff(0, 127), 127);
        assert_eq!(falloff(128, 127), 127);
        assert_eq!(falloff(128 + 128 * 63, 127), 64);
        assert_eq!(falloff(16384, 127), 0);
        assert_eq!(falloff(16385, 127), 0);
        assert_eq!(falloff(0, 100), 100);
    }
}
