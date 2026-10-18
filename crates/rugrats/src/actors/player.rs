//! The player (object 0x800e2b48): the stick, the ground move, the floor
//! under it, and its base animation track.
//!
//! The update (0x8005a068) as far as it is ported: the stick (0x8005be24)
//! sets a wanted heading and a speed, the heading turns toward it
//! (0x8005bc80), the ground move adds speed * dt along the facing
//! (0x8005caac, 0x8005cb48), and the floor query (0x800627a0) puts the feet
//! on the floor or sends the player back to where the frame began.

use rrt::glam::{Mat3, Vec3};
use rugrats_data::anim::Anims;
use rugrats_data::col::Col;
use rugrats_data::hierarchy::Hierarchy;
use rugrats_data::model::Models;
use rugrats_render::Vtx;
use rugrats_render::models;

use crate::actors::animator::Animator;
use crate::actors::pose::{self, PartMatrix};
use crate::disc::GameDisc;
use crate::gte::Trig;
use crate::world::boxhit::Exact;
use crate::input::{self, Pads};
use crate::actors::ground;
use crate::world::collide;

/// dt a frame at 30 fps (0x800c3f1c: 5 a vertical blank).
pub const DT: i32 = 10;
/// Speed factors (0x800c4200 running, 0x800c4204 walking).
const RUN: i32 = 17;
const WALK: i32 = 8;
/// The stick's dead zone (0x8005be24) and the widest turn taken at once
/// (0x800c42cc), and the speed above which any turn is (0x800c42c8).
const DEAD: i32 = 71;
const SNAP_TURN: i32 = 1024;
const MOVING: i32 = 100;
/// The turn rate (0x800c41fc): 1025 * dt >> 6 a frame.
pub const TURN: i32 = 1025;
/// Spike's (and char 34's) turn rate (0x800c420c).
pub const SLOW_TURN: i32 = 298;
/// The player's character: Tommy (0x800c5680 = 1).
const TOMMY: u16 = 1;
/// Gravity (0x80061330): vy += 14 * dt, less 5 while jump is held.
const GRAVITY: i32 = 14;
const LIFT: i32 = -5;
/// The timers' step a frame (0x800c3f20: dt * 4096 / 300).
const TIMER_STEP: i32 = 136;
/// The stick's push in the air at full, straight on (`air_push`: 12 an
/// axis, the original's stack).
pub const AIR_PUSH: i32 = 12;

/// Logical animations (0x800572a8's argument) the port uses; the
/// animator turns them into the character table's states (0x80056fe4).
pub mod anim {
    /// Standing. The stand state asks for +0xe7 (17 from init: "as you
    /// were"); what shows in the oracle once stopped is table state 0,
    /// logical 0.
    pub const IDLE: i16 = 0;
    pub const CARRY_IDLE: i16 = 20;
    pub const PICK_UP: i16 = 13;
    pub const PUT_DOWN: i16 = 14;
    pub const THROW: i16 = 18;
    pub const STAND: i16 = 1;
    pub const RUN: i16 = 2;
    pub const JUMP: i16 = 3;
    pub const LAND: i16 = 4;
    pub const FALL: i16 = 15;
    pub const JUMP_MOVING: i16 = 28;
    /// Climbing (state 6's animation).
    pub const CLIMB: i16 = 5;
    /// The cry (0x8007f920: 0x800572a8(player, 16, 2560)).
    pub const CRY: i16 = 16;
    /// The slip (states 17, 18).
    pub const SLIP: i16 = 33;
    /// The bump (state 9, 0x8005d030): run into a thing it cannot climb.
    pub const BUMP: i16 = 6;
    /// The turn in place, to the right (state 64) and to the left (state
    /// 63), which the stand asks while it turns (0x8005d11c).
    pub const TURN_RIGHT: i16 = 52;
    pub const TURN_LEFT: i16 = 53;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stand,
    Walk,
    Run,
    Jump,
    Fall,
    Land,
}

/// Where a body stands and which way it faces: a start, a checkpoint, a
/// respawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pose {
    pub pos: [i32; 3],
    pub heading: i32,
}

impl Pose {
    pub fn new(pos: [i32; 3], heading: i32) -> Pose {
        Pose { pos, heading }
    }
}

pub struct Player {
    pub pos: [i32; 3],
    /// Heading (+0x24) and the one wanted (+0x2c), 4096 a turn.
    pub heading: i32,
    pub target: i32,
    pub speed: i32,
    /// Air velocity (+0x170): x, y, z.
    pub v: [i32; 3],
    pub state: State,
    pub floor: i32,
    /// Feet to origin (hierarchy +0x4c, the root's y offset).
    feet: i32,
    pub hierarchy: Hierarchy,
    /// Health (+0x1a4, 255 at init, 0x80059a88).
    pub health: i32,
    /// Frozen (+0x188 bit 1): turning, no moving.
    pub frozen: bool,
    /// Golf's aim (state 14): 0x8005be24's own case steps the heading, so
    /// the stick turns the body no other way.
    pub script_aims: bool,
    /// Held in place, the turn not kept to the camera's (levels 20 and
    /// 21: 0x8005be24 takes +0x188 bit 1 as 0 there).
    pub free_while_frozen: bool,
    /// The turn rate (0x800c41fc; 298 for chars 6 and 34).
    pub turn_rate: i32,
    /// The stick's size this frame (+0x1b0, 0..4096), and the speed it set
    /// (+0x184 as the state's handler finds it).
    pub stick_size: i32,
    stick_speed: i32,
    /// The highest step walked up (0x800c430c by +0x15e: 256, or 750).
    pub step_limit: i32,
    /// The slant under which a thing is met head-on, climbed or bumped
    /// rather than slid along (0x8005e1f4: 384, 220 as Spike, +0x15c 6).
    pub head_on: i32,
    /// The stick's push in the air this frame (the body's frame: across,
    /// along).
    pub push: [i32; 2],
    pub animator: Animator,
    /// Sound effects asked for this frame (0x8006f644's numbers).
    pub effects: Vec<u8>,
    /// An action's animation playing (pick up 13, put down 14, throw 18):
    /// the player stands until it ends.
    pub action: Option<i16>,
    /// An animation a level holds the player in, looped (golf's stance).
    pub hold: Option<i16>,
    /// Jump ignored (golf, 0x8005d11c).
    pub no_jump: bool,
    /// The highest ledge a climb takes, and climbing refused here
    /// (0x80058da4: room 4 in levels 1 and 12; level 10's type 14).
    pub climb_max: i32,
    pub climb_off: bool,
    /// The moving jump's lift and factor (0x80060f1c case 25: -1000 and
    /// 40; Spike in level 11 -1700 and 20).
    pub moving_jump: (i32, i32),
    /// The last jump was from standing (0x800c4bc4 = 1, 0x80060f1c
    /// case 0); a moving jump clears it.
    pub standing_jump: bool,
    /// The stick's run and walk factors (0x8005be24: 17 and 8; Spike 20
    /// and 10; the tricycle runs at 20, as seen).
    pub gait: (i32, i32),
    /// Floating (state 21, 0x8005db28): the depth anchor (+0x65c) and the
    /// bob's phase (+0x660).
    pub float: Option<(i32, i32)>,
    /// Carrying: the idle is 20 (0x80057244 at the grab).
    pub carrying: bool,
    /// The carried thing's model boxes in the body's frame, each moved by
    /// its place less the body's turned into that frame (0x80067ba4: the
    /// matrix +4 transposed), as one box.
    pub carry_box: Option<([i32; 3], [i32; 3])>,
    /// The templates that can be picked up (template +0x28, object +0x00
    /// bit 0x10000): run into without a ledge, they stand him, no bump.
    pub pickables: Vec<u16>,
    /// What is carried holds the climbs under 522 (0x80058da4: an object
    /// without bit 0x1000000, class 13's).
    pub carry_climbs: bool,
    /// The jump lock after a throw (0x800c4290: 300, less dt a frame).
    pub jump_lock: i32,
    /// Training messages asked for this frame (0x80097788's n).
    pub hints: Vec<u8>,
    /// Running into a thing bumps (state 9, 0x8005e1f4); off for Reptar
    /// (level 7 stands) and level 11's Spike (runs on).
    pub bumps: bool,
    /// The template of the thing last run into this frame (its +0x6d is
    /// Damage, 0x800780ac), for the game.
    pub bumped: Option<u16>,
    /// The bump's root keys had so far (x, z), by frame of its state.
    bump_root: Vec<[i32; 2]>,
    /// The whole move across of the animation showing its last frame, by
    /// frame, so that one run out gives the rest of it.
    across_rest: Option<Vec<[i32; 2]>>,
    /// Time falling (0x800c428c, summed in 0x80061330).
    air: i32,
    /// Slipping on floor flag 2 (states 3, 17, 18: 0x8005dedc), and the
    /// slip's sound (169 in level 16, else 56).
    slipping: bool,
    pub slip_sound: u8,
    /// A long fall hurts (not in the hub, 0x8005b6e0).
    pub falls_hurt: bool,
    /// The box the feet stand on this frame (player +0xf8), an index into
    /// the boxes `frame` was given.
    pub support: Option<usize>,
    /// The probes' record kept between frames (+0xf4 .. +0x11c), and the
    /// thing stood on as its object (+0xf8, +0xfd), found again in each
    /// frame's solids.
    probes: ground::Probes,
    below: Option<Obj>,
    /// Where the jump began (+0x670 .. +0x678, set at the launch), and
    /// whether the flight has met something (D_800c4bc4 flag 2).
    launch_at: [i32; 3],
    air_hit: bool,
    /// The held jump's lift left (D_800c4288), and Cross held this frame.
    lift_time: i32,
    lifting: bool,
    /// The heading before this frame's turn (the way's, +0x178), and the
    /// frame before's (the launch's).
    way_heading: i32,
    launch_heading: i32,
    /// The landing began this frame (state 10's entry, 0x8005b624).
    pub landed: bool,
    /// A climb under way: how far it rises. The port puts the body at the
    /// top at once; the original's climb moves it by the animation's root
    /// keys (0x80055904 into +0xd8..+0xdc, moved by 0x80066e60), so the
    /// body is drawn where that motion has it.
    climb: Option<Climb>,
    /// A standing jump's crouch: its launch, kept until the jump's
    /// airborne state comes (table state 6, whose entry sets +0xe0 bit 31,
    /// 0x800562bc, which 0x8005ec9c takes as leaving the ground).
    pending_jump: Option<[i32; 3]>,
    /// The root's y the animations have moved the body by on the ground
    /// (into +0xda each step, 0x80056834): drawn under its position.
    /// Standing, walking or running, the floor takes back 23 a frame of
    /// it below the floor (the oracle: 868, 845, 822, 799, 779 after a
    /// landing).
    pub root_y: i32,
    /// The base track's root y so far, which animation it was of, and the
    /// frame it was at (a loop starts again under it).
    root_seen: RootSeen,
    /// Where the body was over the floor as the base track's animation
    /// began: its first frame blends it away.
    root_base: i32,
    /// What the animation before still had when it ran out in the step
    /// this one began: on top of this one's curve until the next begins.
    root_rest: i32,
    /// A character's body (object flag 0x4000): it walks under this speed
    /// and runs from it (0x80059008, 0x800590d4: its table row's fifth),
    /// where the player walks to `WALK`.
    pub walk_below: Option<i32>,
    /// A character whose probe held lets it by (+0x688 bit 0x1000,
    /// 0x80064838: the floor delta 0, the way open): the route runners.
    pub passes: bool,
    /// This frame's step was held up (a wall, an object).
    pub held_up: bool,
    /// Damage has left it no health (0x80077864): the cry is due.
    pub hurt_out: bool,
    /// This frame's place was put back where the frame began by the floor
    /// query (0x800627a0: no floor under it), the root's lift with it.
    pub put_back: bool,
}

impl Player {
    /// Loses `amount` health, to 0 at least.
    pub fn hurt(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        // (Damage that leaves none is ForceCry, 0x8007fd0c from
        // 0x80077864: the health alone is never polled.)
        if self.health <= 0 {
            self.hurt_out = true;
        }
    }

    /// Gains `amount` health, to 255 at most.
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(255);
    }

    /// Where it stands and faces now.
    pub fn pose(&self) -> Pose {
        Pose::new(self.pos, self.heading)
    }

    /// Puts the player at `start` with heading `heading` on the floor
    /// (0x800596e8), with `hierarchy`.
    /// A body standing at `start`, its floor at its feet's height.
    fn bare(hierarchy: Hierarchy, animator: Animator, start: [i32; 3], heading: i32) -> Player {
        let feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        Player {
            pos: start,
            heading,
            target: heading,
            speed: 0,
            v: [0; 3],
            state: State::Stand,
            floor: start[1],
            feet,
            hierarchy,
            health: 255,
            frozen: false,
            script_aims: false,
            free_while_frozen: false,
            turn_rate: TURN,
            stick_size: 0,
            stick_speed: 0,
            head_on: 384,
            step_limit: 256,
            push: [0; 2],
            animator,
            effects: Vec::new(),
            action: None,
            hold: None,
            no_jump: false,
            climb_max: CLIMB,
            climb_off: false,
            moving_jump: (-1000, 40),
            standing_jump: false,
            gait: (RUN, WALK),
            float: None,
            carrying: false,
            carry_box: None,
            pickables: Vec::new(),
            carry_climbs: false,
            jump_lock: 0,
            hints: Vec::new(),
            bumps: true,
            bumped: None,
            bump_root: Vec::new(),
            across_rest: None,
            air: 0,
            slipping: false,
            slip_sound: 56,
            falls_hurt: true,
            support: None,
            probes: ground::Probes::default(),
            below: None,
            launch_at: [0; 3],
            air_hit: false,
            lift_time: 0,
            lifting: false,
            way_heading: 0,
            launch_heading: 0,
            landed: false,
            climb: None,
            pending_jump: None,
            root_y: 0,
            root_seen: RootSeen::default(),
            root_base: 0,
            root_rest: 0,
            walk_below: None,
            passes: false,
            held_up: false,
            put_back: false,
            hurt_out: false,
        }
    }

    pub fn new(
        disc: &GameDisc,
        hierarchy: Hierarchy,
        start: [i32; 3],
        heading: i32,
        col: &Col,
        anims: &Anims,
    ) -> Result<Player, String> {
        let animator = Animator::load(disc, TOMMY)?;
        let feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        let mut p = Player::bare(hierarchy, animator, start, heading);
        let q = collide::query(col, start);
        if q.tile.is_some() {
            p.floor = q.floor;
            p.pos[1] = q.floor + feet;
        }
        p.animator.set(anim::IDLE, anims);
        Ok(p)
    }

    /// Standing again, out of any air (a body put somewhere).
    pub fn reset_air(&mut self) {
        self.state = State::Stand;
        self.v = [0; 3];
        self.pending_jump = None;
        self.climb = None;
        self.action = None;
    }

    /// Landing (state 10).
    pub fn landing(&self) -> bool {
        self.state == State::Land
    }

    /// On the floor (stand, walk, run, landing).
    pub fn on_ground(&self) -> bool {
        matches!(self.state, State::Stand | State::Walk | State::Run | State::Land)
    }

    /// A character's body (the shared one 0x8005a068 runs for it): its
    /// hierarchy and animator, where it stands, walking under `walk_below`.
    pub fn character_body(hierarchy: Hierarchy, animator: Animator, pos: [i32; 3], heading: i32, walk_below: i32) -> Player {
        let feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        let mut p = Player::bare(hierarchy, animator, pos, heading);
        p.feet = feet;
        p.floor = pos[1] - feet;
        p.walk_below = Some(walk_below);
        p.falls_hurt = false;
        p.bumps = false;
        p
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left), for tools.
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        self.animator.track()
    }

    pub fn anm(&self, a: i16) -> Option<usize> {
        self.animator.anm(a)
    }

    /// One frame of play. `camera_heading` is the camera's (4096 a turn).
    #[allow(clippy::too_many_arguments)]
    pub fn frame(&mut self, pads: &Pads, camera_heading: i32, col: &Col, boxes: &[Box3], solids: &[Exact], anims: &Anims, trig: &Trig) {
        if self.float.is_some() {
            self.float_frame(pads, camera_heading, col, anims, trig);
            return;
        }
        self.effects.clear();
        self.hints.clear();
        // The animation steps first and its root with it (0x8005a068:
        // 0x80056834, then the state's handler, which asks for what plays
        // from the next step).
        self.animator.step(anims, trig);
        self.root_step(anims, col, trig);
        let held = |b: usize| pads.held(0, b);
        let pressed = |b: usize| pads.pressed(0, b);
        self.lifting = held(input::CROSS);
        let start = self.pos;
        // The stick: digital directions are full (4096) on their axis;
        // up is +z.
        let sx: i32 = if held(input::LEFT) {
            -4096
        } else if held(input::RIGHT) {
            4096
        } else {
            0
        };
        let sz: i32 = if held(input::UP) {
            4096
        } else if held(input::DOWN) {
            -4096
        } else {
            0
        };
        // The analog stick's axis where larger (0x8005be24).
        let [ax, az] = pads.port[0].stick;
        let sx = if ax.abs() > sx.abs() { ax } else { sx };
        let sz = if az.abs() > sz.abs() { az } else { sz };
        let mut magnitude = (((sx * sx + sz * sz) as f64).sqrt() as i32).min(4096);
        let on_ground = matches!(self.state, State::Stand | State::Walk | State::Run | State::Land);
        self.jump_lock = (self.jump_lock - DT).max(0);
        // An action holds the player until its animation ends.
        if let Some(a) = self.action {
            if self.animator.playing(a) {
                magnitude = 0;
            } else {
                self.action = None;
                // The bump's state (9) reads no stick; it gives way to
                // standing after this frame's stick (the original turns a
                // frame after the bump ends, `hub_run_as_the_original`).
                if a == anim::BUMP {
                    magnitude = 0;
                }
                self.finish_climb(col, boxes, false);
                // The slip's fall ends (state 18): Damage 5, message 30.
                if self.slipping {
                    self.slipping = false;
                    self.hurt(SLIP_DAMAGE);
                    self.hints.push(30);
                }
            }
        }
        self.stick_size = magnitude;
        if magnitude >= DEAD {
            let dir = (((sx as f64).atan2(sz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
            let want = (dir + camera_heading).rem_euclid(4096);
            let factor = if held(input::SQUARE) { self.gait.1 } else { self.gait.0 };
            let turn = wrap(want - self.heading);
            if matches!(self.state, State::Jump | State::Fall | State::Land) {
                // In the air and through the landing the stick does not
                // steer: the heading holds (the original's 1246 from the
                // takeoff to standing again,
                // `hub_start_run_and_jump_as_the_original`); the air's push
                // takes the stick instead.
                if self.state == State::Land {
                    self.speed = 0;
                }
            } else if self.frozen {
                // Held in place (+0x188 bit 1): only the turn runs, so the
                // stick aims.
                if !self.script_aims {
                    self.target = want;
                }
                self.speed = 0;
            } else if turn.abs() <= SNAP_TURN || self.speed >= MOVING {
                // The target at once; the heading turns to it at the turn
                // rate below, the move along it (the oracle, the hub's
                // start: 2048 to 1888, 1728, 1568 a frame for 1536, the
                // steps along each).
                self.target = want;
                self.speed = (magnitude * factor) >> 12;
                // Running at full speed (0x800590d4): message 2.
                if self.speed > (self.gait.0 * 3933) >> 12 {
                    self.hints.push(2);
                }
            } else {
                self.target = want;
                self.speed = 0;
            }
        } else {
            self.speed = 0;
            // The stick let go (its magnitude under 70): the target is the
            // heading, the turn stops where it is (0x8005c8d8; the oracle,
            // a tap of Up and Left from the hub's start: 2048, 1888, held).
            self.target = self.heading;
        }
        // Held in place (+0x188 bit 1, but in levels 20 and 21): the
        // target kept within 512 of the camera's heading, both as 0..4096
        // (0x8005c3dc); the pie toss's Tommy, put facing 2048 by a camera
        // looking along 1024, turns to 1536.
        if self.frozen && !self.script_aims && !self.free_while_frozen {
            let norm = |a: i32| a.rem_euclid(4096);
            let (want, cam) = (norm(self.target), norm(camera_heading));
            self.target = want.clamp(cam - 512, cam + 512);
        }
        // The state's handler steps along the heading as it is; the turn
        // toward the wanted heading (0x8005bc80) comes after the step is
        // added (0x8005cb48), so this frame's turn moves the next frame's
        // step (`hub_start_run_and_jump_as_the_original`).
        // The way (+0x178): the body's matrix (0x80020d10 of the heading,
        // MatrixNormal'd) applied to (0, 0, 4096), a little off unit
        // (0x8005c8d8).
        self.stick_speed = self.speed;
        let way = trig.turn(self.heading, [0, 0, 4096]);
        self.launch_heading = self.way_heading;
        self.way_heading = self.heading;
        let step = (self.rate() * DT) >> 6;
        let d = fold(self.target - self.heading);
        self.heading = (self.heading + d.clamp(-step, step)).rem_euclid(4096);
        // The stick's push in the air (0x80061330), had there: the axes
        // as the pad gives them (4096 a full one, 8 times that raw).
        self.push = if self.frozen { [0; 2] } else { air_push([sx, sz], camera_heading, self.heading, trig) };
        self.drive(start, way, on_ground, pressed(input::CROSS), held(input::CROSS), col, boxes, solids, anims, trig);
    }

    /// The body's frame from its way on (the state's handler, 0x8005a068
    /// after the stick): the ground move along `way` at the speed, the
    /// jump on `jump` (Cross pressed; `lift` held), the air, the floor,
    /// the state and its animation. The player's stick (`frame`) or a
    /// character's (0x80082264) has set the speed and the heading wanted.
    #[allow(clippy::too_many_arguments)]
    pub fn drive(
        &mut self,
        start: [i32; 3],
        way: [i32; 3],
        on_ground: bool,
        jump: bool,
        lift: bool,
        col: &Col,
        boxes: &[Box3],
        solids: &[Exact],
        anims: &Anims,
        trig: &Trig,
    ) {
        let acting = self.action.is_some();
        self.put_back = false;
        // A wall met head-on (or no way along it) by the ground move.
        let mut wall_met = false;
        // The probe's record that held the step head-on (`ground_move`).
        let mut ground_held: Option<ground::Record> = None;
        // The stand handler's probe held (see `ground_move`).
        let mut stand_held = false;
        // What the probe ahead found (walking or running), if it ran.
        let mut ahead: Option<ground::Ahead> = None;
        let first_step = self.state == State::Stand && self.speed > 0 && !acting && self.pending_jump.is_none();
        // In the air the original's own air move tests and has the step
        // (`air_move`), landing included.
        let mut exact_air = false;
        let mut air_landed = false;
        // The floor the probes measure from (+0x148): a thing stood on's
        // top, which the height field's floor below replaces.
        let probe_floor = self.floor;
        let mut probe_step: Option<(i32, i32)> = None;
        if on_ground {
            // The ground move along the facing (0x8005caac); none while
            // landing.
            if !self.frozen && self.state != State::Land {
                // The thing stood on, found again among this frame's
                // solids; on a slippery floor walls turn the way off them
                // (+0x188 0x100000, 0x8005bd9c).
                self.probes.below = self
                    .below
                    .and_then(|o| boxes.iter().find(|b| b.obj == Some(o)))
                    .and_then(|b| b.solid)
                    .map(|(k, j)| (k as usize, j));
                let here = collide::query(col, [self.pos[0], self.pos[1] - self.feet - 1, self.pos[2]]);
                let reflect = collide::floor_flags(col, &here) & SLIPPERY != 0;
                let ((dx, dz), held, probe_held) = self.ground_move(col, trig, way, self.speed * DT, solids, reflect);
                wall_met = held.is_some();
                // Standing, the stick pushed: a probe that held (1) puts
                // the speed by at the stand handler's end (0x8005d11c),
                // even when the way along the face then starts the run.
                stand_held = first_step && probe_held;
                ground_held = held;
                if first_step {
                    // Standing, the stick pushed (0x8005d11c): the probes
                    // run along the step, a slide or a climb may start and
                    // the state changes, but the step is the walk's or the
                    // run's, from the next frame (the original stands its
                    // first frame, `hub_run_as_the_original`).
                    probe_step = Some((dx, dz));
                } else {
                    // Walking or running, the way open: the probe ahead
                    // (0x800645d8, 0x80064838 in mode 2) of the step from
                    // here: a drop past the step limit is the fall (state
                    // 4).
                    if held.is_none() && matches!(self.state, State::Walk | State::Run) && (dx, dz) != (0, 0) {
                        ahead = Some(self.ahead_step(col, trig, way, self.speed * DT, solids));
                    }
                    self.pos[0] += dx;
                    self.pos[2] += dz;
                }
            }
            if jump && !acting && self.jump_lock == 0 && !self.no_jump && !self.frozen {
                // The launch (0x80060f1c): up 1000, and along the facing
                // 40 a unit of speed when running, else 12.
                self.standing_jump = self.speed <= WALK;
                let (up, factor) = if self.standing_jump { (-1000, 12) } else { self.moving_jump };
                let along = factor * self.speed;
                // (Along the heading at the takeoff, `root_step`.)
                self.pending_jump = Some([0, up, along]);
                self.v = [0; 3];
                self.state = State::Jump;
                // The jump's state turns no more: the heading holds from
                // here (seen standing, mid-turn: 1088 through the crouch).
                self.target = self.heading;
            }
        } else if self.pending_jump.is_some() {
            // Crouching for the jump: still on the floor.
        } else {
            // The air (0x80061330): gravity 14, less 5 while jump is held
            // on the way up; across, the stick's push against the drag
            // (`air_step`).
            air_landed = self.air_move(lift, col, trig, solids, anims, false);
            exact_air = true;
        }
        // Held in place (+0x188 bit 1) and standing, no step and no floor
        // follows: the pie toss's Tommy stays as put, 31 into the counter.
        let held_still = (self.frozen && on_ground).then_some((self.pos[1], self.floor));
        // The floor (0x800627a0): the feet on it, or back where the frame
        // began when the point is not in the open.
        // Asked from a step's height above the feet, so a slope or a
        // step up within STEP_UP is floor, not wall.
        let lift = if on_ground { STEP_UP } else { 1 };
        let mut probe = [self.pos[0], self.pos[1] - self.feet - lift, self.pos[2]];
        let mut q = collide::query(col, probe);
        // In the air the move goes across, then down (0x8005f1b4 through
        // 0x8005ee78): across into a place with no floor record stops the
        // across part only, and the fall goes on.
        if !exact_air && !on_ground && (q.kind != 0 && q.tile.is_none() || q.kind == 2) {
            self.pos[0] = start[0];
            self.pos[2] = start[2];
            probe = [self.pos[0], self.pos[1] - self.feet - lift, self.pos[2]];
            q = collide::query(col, probe);
        }
        // Climbing (state 6, 0x8005f310): walking into a ledge or a
        // climbable object up to 1033 above the feet (521 carrying,
        // 0x80056e44, 0x80058da4) brings the player onto it.
        // (Standing, the stick pushed: 0x8005d11c's probe meets it, and
        // its climb test, from a standstill too.)
        let walking = matches!(self.state, State::Stand | State::Walk | State::Run) && self.speed > 0 && !acting && !self.climb_off;
        let reach = if self.carrying && self.carry_climbs { CLIMB_CARRYING.min(self.climb_max) } else { self.climb_max };
        let mut climb_to: Option<i32> = None;
        // A ledge met within the body's reach, climbable or not
        // (0x80058da4's answer).
        let mut ledge = false;
        // Where a climb met its face: the box it is onto, or the probes'
        // record (+0x104 point, +0x114 normal), for 0x80062cf4's move to it.
        let mut climb_box: Option<usize> = None;
        let mut climb_met: Option<([i32; 3], [i32; 3])> = None;
        // Falling through the floor this step (0x8005f1b4 lands when y +
        // vy reaches it): onto the floor under where the step began.
        let landing = (!on_ground && self.v[1] > 0 && q.kind != 0)
            .then(|| collide::query(col, [self.pos[0], start[1] - self.feet - 1, self.pos[2]]))
            .filter(|f| f.kind == 0 && f.tile.is_some() && self.pos[1] - self.feet >= f.floor);
        if exact_air {
            // (Had by the air move.)
        } else if let Some(f) = landing {
            self.pos[1] = f.floor + self.feet;
            self.floor = f.floor;
        } else if q.kind != 0 && q.tile.is_none() || q.kind == 2 {
            self.pos = start;
            self.put_back = true;
        } else if q.kind != 0 && walking && {
            let feet_y = start[1] - self.feet;
            let up = collide::query(col, [self.pos[0], feet_y - reach, self.pos[2]]);
            let ok = up.kind == 0 && up.floor < feet_y - STEP_UP && up.floor >= feet_y - reach;
            if ok {
                climb_to = Some(up.floor);
            } else {
                // Too tall (0x80058da4): a step of 1025..3071, message 16.
                let tall = collide::query(col, [self.pos[0], feet_y - TOO_TALL, self.pos[2]]);
                if tall.kind == 0 && tall.floor < feet_y - CLIMB && tall.floor >= feet_y - TOO_TALL {
                    self.hints.push(16);
                }
            }
            ok
        } {
            // Onto the ledge.
        } else if q.kind != 0
            && self.passes
            && {
                // A runner let by (+0x688 bit 0x1000) stands on the floor
                // where it now is, however high (0x80067448 from over it).
                let up = collide::query(col, [self.pos[0], start[1] - self.feet - CLIMB, self.pos[2]]);
                if up.kind == 0 {
                    self.floor = up.floor;
                }
                up.kind == 0
            }
        {
        } else if q.kind != 0 {
            // Into a wall or a step too high: back across, keep the height.
            // (0x800627a0 puts the whole place back, so a root's lift of
            // this step goes too.)
            let back = collide::query(col, [start[0], probe[1], start[2]]);
            self.pos = [start[0], self.pos[1], start[2]];
            self.put_back = true;
            if back.kind == 0 {
                self.floor = back.floor;
            }
        } else {
            self.floor = q.floor;
        }
        if let Some((y, floor)) = held_still {
            self.pos[1] = y;
            self.floor = floor;
        }
        // Held head-on by the height field (0x8005e1f4 -> 0x80058da4): a
        // ledge up to the body's reach is climbed (0x80056e44 from the
        // record's floor delta, +0x144); a step of 1025 to 3071 is too
        // tall (message 16); no floor (a narrow way) is neither.
        // An object met there (+0xf4) is climbed only if its type says so
        // (flag 64; message 29 if not), and is the bump's (its damage).
        let met_thing = ground_held.and_then(|r| r.hit).and_then(|(k, _)| solids.get(k));
        if let Some(r) = ground_held
            && walking
            && r.delta != ground::VOID
        {
            let rise = -r.delta;
            let top = probe_floor + r.delta;
            let climbable = met_thing.is_none_or(|e| e.climb);
            ledge |= rise > 0 && rise <= reach;
            if rise > 0 && rise <= reach && climbable {
                climb_met = Some((self.probes.met.point, self.probes.met.normal));
                climb_to = Some(climb_to.map_or(top, |c: i32| c.min(top)));
            } else if rise > 0 && rise <= reach {
                self.hints.push(29);
            } else if rise > CLIMB && rise <= TOO_TALL && climbable {
                self.hints.push(16);
            }
        }
        // Objects' boxes: one whose top is within a step of the feet and
        // not above the body's origin (or below them, falling) is floor
        // (the floor test runs down from the origin, 0x80065578); one the
        // step's probes meet (0x80063d3c: from the body's middle, ahead
        // and to each side, at the box's own middle height kept within
        // the body's, 0x8006a714) sends it back across. A character's body
        // is met where it stands.
        let start_feet = start[1] - self.feet;
        let feet_y = self.pos[1] - self.feet;
        let (dx, dz) = probe_step
            .map_or(((self.pos[0] - start[0]) as f32, (self.pos[2] - start[2]) as f32), |(x, z)| (x as f32, z as f32));
        let len = dx.hypot(dz);
        // The probes are the ground move's (0x80064a34); in the air the body
        // is met where it is.
        let ground = on_ground && matches!(self.state, State::Stand | State::Walk | State::Run);
        let probes = if len < 1.0 || !ground { Vec::new() } else { self.probes([dx / len, dz / len], len) };
        let mut blocked = false;
        let mut ran_into: Option<u16> = met_thing.map(|e| e.template);
        // The face the probes met a thing by (+0x114), for the slide.
        let mut face: Option<[i32; 3]> = None;
        self.support = None;
        let head = feet_y - HEIGHT;
        // Where the probes from the step's start meet box `b` (not a
        // character's, not floor to the body): the face's normal.
        let (body_lo, body_hi) = self.body_box();
        let probe_hit = |b: &Box3, probes: &[(i32, i32)], feet_y: i32, to_y: i32| {
            let floor_like = b.min_y >= start_feet.min(feet_y) - STEP_UP && b.min_y >= start[1].min(to_y);
            if b.character || floor_like || b.max_y <= head {
                return None;
            }
            let h = probe_height(b, start, body_lo, body_hi);
            probes.iter().find_map(|&(dx, dz)| b.segment_hit([start[0], h, start[2]], [start[0] + dx, h, start[2] + dz]).map(|hit| hit.2))
        };
        for (k, b) in boxes.iter().enumerate() {
            // (Things in the air: the air move met them.)
            if exact_air && b.solid.is_some() {
                continue;
            }
            let near = b.covers(self.pos, RADIUS);
            if !near && probes.is_empty() {
                continue;
            }
            let top = b.min_y;
            let floor_like = top >= start_feet.min(feet_y) - STEP_UP && (b.character || !ground || top >= start[1].min(self.pos[1]));
            let met = !floor_like
                && b.max_y > head
                && if b.solid.is_some() && ground {
                    // On the ground the probes met it, if at all
                    // (`ground_move`).
                    false
                } else if b.character || !ground {
                    near && (b.depth(self.pos, RADIUS) > b.depth(start, RADIUS) || b.crosses(start, self.pos))
                } else {
                    let n = probe_hit(b, &probes, feet_y, self.pos[1]);
                    face = face.or(n);
                    n.is_some()
                };
            if near && floor_like {
                // Standing room on it: a floor if higher than the one found.
                if top < self.floor {
                    self.floor = top;
                    self.support = Some(k);
                }
            } else if met {
                ledge |= walking && top >= start_feet - reach;
                if b.climb && walking && top >= start_feet - reach {
                    if climb_to.is_none_or(|c| top < c) {
                        climb_box = Some(k);
                    }
                    climb_to = Some(climb_to.map_or(top, |c: i32| c.min(top)));
                } else {
                    // Not climbable (29) or too tall (16).
                    if walking && !b.character {
                        if top >= start_feet - reach {
                            self.hints.push(29);
                        } else if b.climb && top < start_feet - CLIMB && top >= start_feet - TOO_TALL {
                            self.hints.push(16);
                        }
                    }
                    blocked = true;
                    if ground && !b.character {
                        ran_into = Some(b.template);
                    }
                }
            }
        }
        // The thing stood on, for the next frame's probes (+0xf8).
        self.below = self.support.and_then(|k| boxes.get(k)).and_then(|b| b.obj);
        // Met at a slant (0x8005d11c / 0x8005e1f4, +0x130: the angle
        // between the step turned back and the face's normal): from a
        // head-on 384 (220 as Spike) to a right angle the whole step goes
        // along the face instead (0x80065120 through 0x800763d0's tangent)
        // if the probes find that way open, or meet a face there at 512 or
        // more (held, no bump); else the climb or the bump.
        let slant = |n: [i32; 3], d: [f32; 2]| {
            let nl = (n[0] as f32).hypot(n[2] as f32).max(1.0);
            let cos = -(d[0] * n[0] as f32 + d[1] * n[2] as f32) / nl;
            (cos.clamp(-1.0, 1.0).acos() * 4096.0 / std::f32::consts::TAU) as i32
        };
        if blocked
            && walking
            && ground
            && let Some(n) = face
            && boxes.iter().all(|b| !b.character || !b.covers(self.pos, RADIUS))
        {
            let d = [dx / len, dz / len];
            let a = slant(n, d);
            if a >= self.head_on && a < 1024 {
                let nl = (n[0] as f32).hypot(n[2] as f32).max(1.0);
                let (nx, nz) = (n[0] as f32 / nl, n[2] as f32 / nl);
                let dot = d[0] * nx + d[1] * nz;
                let (tx, tz) = (d[0] - dot * nx, d[1] - dot * nz);
                let tl = tx.hypot(tz).max(1e-3);
                let t = [tx / tl, tz / tl];
                self.pos = start;
                let (sx, sz) = self.ground_step(col, (t[0] * len) as i32, (t[1] * len) as i32);
                let along = [start[0] + sx, start[1], start[2] + sz];
                let sl = (sx as f32).hypot(sz as f32);
                let again = if sl < 1.0 {
                    None
                } else {
                    let probes = self.probes([sx as f32 / sl, sz as f32 / sl], sl);
                    boxes.iter().find_map(|b| probe_hit(b, &probes, feet_y, start[1]))
                };
                match again {
                    None if sl >= 1.0 => {
                        if !first_step {
                            self.pos = along;
                        }
                        blocked = false;
                        ran_into = None;
                        climb_to = None;
                    }
                    Some(n2) if (512..1025).contains(&slant(n2, t)) => {
                        ran_into = None;
                        climb_to = None;
                    }
                    _ => {}
                }
            }
        }
        if blocked {
            self.pos[0] = start[0];
            self.pos[2] = start[2];
        }
        // Whether the step was held up at all.
        let stopped = blocked || wall_met;
        self.held_up = stopped;
        // Running (0x800590d4: the speed over the run's factor times
        // 3933 >> 12) into a thing not climbed: the bump, its damage to the
        // game (0x8005e1f4). (Slower, held up: the stand.)
        // (A wall's has no thing, +0xf4 0: no damage.)
        if (ran_into.is_some() || wall_met && !blocked)
            && climb_to.is_none()
            && self.bumps
            && self.state == State::Run
            && self.speed > (self.gait.0 * 3933) >> 12
            // (With no ledge in reach, a thing that can be picked up, +0x00
            // bit 0x10000, stands him instead.)
            && (ledge || !ran_into.is_some_and(|t| self.pickables.contains(&t)))
        {
            self.bumped = ran_into;
            // The speed (+0x184) stays this frame: the bump's state zeroes
            // it as it runs, the next (the follow camera still sees the
            // player moving, 0x80027304).
            let speed = self.speed;
            self.act(anim::BUMP, anims);
            self.speed = speed;
        } else if ran_into.is_some_and(|t| self.pickables.contains(&t))
            && !ledge
            && climb_to.is_none()
            && self.state == State::Run
            && self.speed > (self.gait.0 * 3933) >> 12
        {
            // (0x8005e1f4 at 0x8005e68c: state 0.)
            self.state = State::Stand;
            self.speed = 0;
        }
        if let Some(top) = climb_to {
            // The climb starts at the face (0x80062cf4): the body put where
            // the way meets it, out along its normal by the hierarchy box's
            // half depth (not the carried thing's), and turned square to it.
            let mut from = start;
            let met = climb_met.or_else(|| {
                let b = boxes.get(climb_box?)?;
                let a = self.heading as f32 * std::f32::consts::TAU / 4096.0;
                let mid = start[1] - self.feet - HEIGHT / 2;
                let to = [start[0] + (a.sin() * 3000.0) as i32, mid, start[2] + (a.cos() * 3000.0) as i32];
                b.segment_hit([start[0], mid, start[2]], to).map(|(_, at, n)| (at, n))
            });
            if let Some((at, n)) = met
                && n[1] == 0
                && n != [0; 3]
            {
                let h = |o: usize| self.hierarchy.header.get(o..o + 2).map_or(0, |v| i16::from_le_bytes([v[0], v[1]]) as i32);
                let depth = h(0x08).abs().max(h(0x10).abs());
                let u = collide::normalize(collide::unit(n));
                let to = [at[0] + ((u[0] * depth) >> 12), start[1], at[2] + ((u[2] * depth) >> 12)];
                from = to;
                self.pos[0] = from[0];
                self.pos[2] = from[2];
                self.target = trig.ratan2(-n[0], -n[2]) & 4095;
            }
            // The rise picks the climb (+0x144, 0x80056e44).
            self.animator.climb = self.floor - top;
            self.climb = Some(Climb { top: top + self.feet, whole: [0; 3], from, moved: [0; 3] });
            self.pos[1] = top + self.feet;
            self.floor = top;
            self.state = State::Stand;
            self.act(anim::CLIMB, anims);
            let rot = self.rotation(trig);
            if let (Some((_, total)), Some(c)) = (self.animator.root_progress(anims), &mut self.climb) {
                let w = rot * Vec3::new(total[0] as f32, total[1] as f32, total[2] as f32);
                c.whole = [w.x as i32, w.y as i32, w.z as i32];
            }
            self.hints.push(1);
        }
        let feet_y = self.pos[1] - self.feet;
        // (A climb is state 6, 0x8005f310, which runs no ground step: it
        // never falls, and its height is the climb's.)
        if self.climb.is_some() {
        } else if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            let falls = match ahead {
                Some(a) => a == ground::Ahead::Fall,
                None => self.floor - feet_y > STEP_DOWN,
            };
            if falls {
                self.state = State::Fall;
                // Off along the way at the running jump's 40 a unit of
                // speed, in the body's own frame (+0x170; the original's
                // (-13, 0, 680) for a character at 17).
                self.v = launch_v(way, 0, self.moving_jump.1 * self.speed, self.heading, trig);
                self.air_hit = false;
                self.lift_time = 0;
                self.air = 0;
            } else {
                // Just landed, the body stays where the air left it this
                // frame (the original's y): the landing's animation takes it
                // from there, its first frame blending from that height
                // (+0xde, 0x80055904).
                let was = self.pos[1];
                self.pos[1] = self.floor + self.feet;
                if air_landed {
                    self.root_y += was - self.pos[1];
                }
            }
        } else if self.pending_jump.is_some() {
            // The crouch (the jump's state before +0xe0 bit 31): its state
            // (0x8005ec9c) runs no step of the ground (0x80064a34, which
            // only the stand, walk, run and slide states call), so the height is
            // neither followed nor dropped, an edge passed or not; the jump
            // goes off from it.
        } else if !exact_air && feet_y >= self.floor && self.v[1] >= 0 {
            self.pos[1] = self.floor + self.feet;
            self.v = [0; 3];
            self.state = State::Land;
            self.landed = true;
            // A long fall (state 10, 0x8005b2d8): Damage 5, message 31.
            if self.air >= LONG_FALL && self.falls_hurt {
                self.hurt(FALL_DAMAGE);
                self.hints.push(31);
            }
        } else if self.v[1] > 0 {
            self.state = State::Fall;
        }
        // Running onto a slippery floor (flag 2, 0x8005c96c): message 32,
        // the slip (anim 33), then its fall.
        let on_floor = matches!(self.state, State::Walk | State::Run);
        if on_floor && self.speed > WALK && !self.slipping && !acting && self.anm(anim::SLIP).is_some() {
            let here = collide::query(col, [self.pos[0], self.pos[1] - self.feet - 1, self.pos[2]]);
            if collide::floor_flags(col, &here) & SLIPPERY != 0 {
                self.hints.push(32);
                self.slipping = true;
                self.speed = 0;
                self.state = State::Stand;
                self.effects.push(self.slip_sound);
                self.act(anim::SLIP, anims);
            }
        }
        if matches!(self.state, State::Stand | State::Walk | State::Run | State::Land) {
            self.state = match self.speed {
                // The landing (state 10, animation 4) plays out first, the
                // stick or not: its handler (0x8005cba8) does not move the
                // body (the oracle: x and z held through state 8).
                _ if self.state == State::Land
                    && (self.animator.asked() != anim::LAND || self.animator.playing(anim::LAND)) =>
                {
                    State::Land
                }
                // Standing, the way held up: still standing (0x8005d11c's
                // probe; the oracle: one bump, then Up held stands).
                _ if stopped && self.state == State::Stand => State::Stand,
                0 => State::Stand,
                s if self.walk_below.map_or(s <= WALK, |w| s < w) => State::Walk,
                _ => State::Run,
            };
            // Held standing, its speed is put by (+0x184 into +0x176) and
            // is 0 (0x8005d11c), the run along the face started or not.
            if stopped && self.state == State::Stand || stand_held {
                self.speed = 0;
            }
        }
        let a = match self.state {
            _ if self.hold.is_some() => self.hold.unwrap(),
            _ if self.action.is_some() && on_ground => self.action.unwrap(),
            State::Stand if self.carrying => anim::CARRY_IDLE,
            // Turning where it stands (0x8005d11c with 0x80059204): the
            // turn in place to the left (53, state 63) or the right (52,
            // state 64); still, the idle.
            State::Stand => match self.turning_in_place() {
                d if d < 0 => anim::TURN_LEFT,
                d if d > 0 => anim::TURN_RIGHT,
                _ => anim::IDLE,
            },
            State::Walk => anim::STAND,
            State::Run => anim::RUN,
            State::Jump => {
                if self.speed > 0 {
                    anim::JUMP_MOVING
                } else {
                    anim::JUMP
                }
            }
            // A jump's way down is still the jump (state 5) in the
            // original: its rise runs on into its fall (5, 6). Falling
            // off an edge is state 4, animation 15.
            State::Fall if matches!(self.animator.asked(), anim::JUMP | anim::JUMP_MOVING) => self.animator.asked(),
            State::Fall => anim::FALL,
            State::Land => anim::LAND,
        };
        // Standing with a thing, the stand's entry asks for the idle on the
        // base track as well as the carry's arms over it.
        if a == anim::CARRY_IDLE {
            self.animator.set(anim::IDLE, anims);
        }
        self.animator.set(a, anims);
        if air_landed {
            self.landed = true;
            // A long fall (state 10, 0x8005b2d8): Damage 5, message 31.
            if self.air >= LONG_FALL && self.falls_hurt {
                self.hurt(FALL_DAMAGE);
                self.hints.push(31);
            }
        }
        if let Some((_, m)) = self.climb_motion(anims, trig)
            && let Some(c) = &mut self.climb
        {
            c.moved = m;
        }
        // Its animation over, the climb ends this frame (the next state
        // has begun under it).
        if self.climb.is_some() && !self.animator.playing(anim::CLIMB) {
            self.finish_climb(col, boxes, true);
            self.action = None;
        }
    }

    /// Floating (state 21, 0x8005db28): the stick moves at half speed, no
    /// gravity; Cross held rises 40 a frame, Circle sinks 40, else a bob of
    /// 256 about the anchor ((218 dt) >> 6 a frame).
    fn float_frame(&mut self, pads: &Pads, camera_heading: i32, col: &Col, anims: &Anims, trig: &Trig) {
        let Some((mut anchor, mut bob)) = self.float else { return };
        self.effects.clear();
        let start = self.pos;
        let held = |b: usize| pads.held(0, b);
        let sx = if held(input::LEFT) {
            -4096
        } else if held(input::RIGHT) {
            4096
        } else {
            0
        };
        let sz = if held(input::UP) {
            4096
        } else if held(input::DOWN) {
            -4096
        } else {
            0
        };
        if sx != 0 || sz != 0 {
            let dir = (((sx as f64).atan2(sz as f64) / std::f64::consts::TAU) * 4096.0).round() as i32;
            self.target = (dir + camera_heading).rem_euclid(4096);
            self.speed = RUN / 2;
        } else {
            self.speed = 0;
        }
        let step = (self.rate() * DT) >> 6;
        let d = fold(self.target - self.heading);
        self.heading = (self.heading + d.clamp(-step, step)).rem_euclid(4096);
        let (s, c) = (trig.sin(self.heading), trig.cos(self.heading));
        self.pos[0] += (s * self.speed * DT) >> 12;
        self.pos[2] += (c * self.speed * DT) >> 12;
        bob = (bob + ((218 * DT) >> 6)).rem_euclid(4096);
        if held(input::CROSS) {
            self.pos[1] -= (262 * DT) >> 6;
            anchor = self.pos[1];
        } else if held(input::CIRCLE) {
            self.pos[1] += (131 * DT) >> 5;
            anchor = self.pos[1];
        } else if self.speed == 0 {
            self.pos[1] = anchor + ((trig.sin(bob) * 256) >> 12);
        }
        // Walls and floors still stop the body.
        let q = collide::query(col, [self.pos[0], self.pos[1] - self.feet, self.pos[2]]);
        if q.kind != 0 {
            self.pos = start;
        }
        self.float = Some((anchor, bob));
        self.state = State::Stand;
        let a = if self.speed > 0 { anim::RUN } else { anim::STAND };
        self.animator.set(a, anims);
        self.animator.step(anims, trig);
    }

    /// Runs as another body (0x80059328 swaps +0x154 and +0x15c): its
    /// hierarchy, its animations, its turn rate, the feet kept on the floor.
    /// The body's half width across a move and its reach along one: the
    /// larger x extent of the hierarchy's box (header +4 and +0xc,
    /// 0x80067ba4), both ways. Seen in the oracle: Tommy (250) stops 268
    /// short of a wall at 170 a frame; Spike (x 315, z -220..1000) 479
    /// short at 200, so his reach is not the box's 1000 along z.
    /// The step's probes against objects (0x80064a34 -> 0x80063d3c), as
    /// (x, z) from where it began: its body box's depth plus the step
    /// (speed +0x184 x D_800c3f1c, 10) straight on along `dir`, and the same
    /// at its half-width to either side (the oracle: running at the shut
    /// back door Tommy stops at 23020, walking at 23120, both on their
    /// step's grid, the probe's reach short of the panel).
    pub fn probes(&self, dir: [f32; 2], step: f32) -> Vec<(i32, i32)> {
        let (lo, hi) = self.body_box();
        let side = lo[0].abs().max(hi[0].abs()) as f32;
        let ahead = lo[2].abs().max(hi[2].abs()) as f32 + step;
        let [ux, uz] = dir;
        [0.0, -side, side].iter().map(|&w| ((ux * ahead + uz * w) as i32, (uz * ahead - ux * w) as i32)).collect()
    }

    /// The height the probes meet box `b` at: its own middle (unturned,
    /// 0x80063d3c) kept within the body's from `at`.
    pub fn probe_height(&self, b: &Box3, at: [i32; 3]) -> i32 {
        let (lo, hi) = self.body_box();
        probe_height(b, at, lo, hi)
    }

    /// The body's box (0x80067ba4): the hierarchy header's (+4..+0x10: lo
    /// x, y, z, hi x, y, z), grown to take in the carried thing's boxes
    /// (`carry_box`).
    pub fn body_box(&self) -> ([i32; 3], [i32; 3]) {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32);
        let (mut lo, mut hi) = ([h(0x04), h(0x06), h(0x08)], [h(0x0c), h(0x0e), h(0x10)]);
        if let Some((clo, chi)) = self.carry_box {
            for k in 0..3 {
                lo[k] = lo[k].min(clo[k]);
                hi[k] = hi[k].max(chi[k]);
            }
        }
        (lo, hi)
    }

    /// The heading the state's handler saw this frame, before the body's
    /// turn (0x8005bc80 comes after it).
    pub fn handler_heading(&self) -> i32 {
        self.way_heading
    }

    fn extents(&self) -> (i32, i32) {
        let h = |at: usize| self.hierarchy.header.get(at..at + 2).map_or(0, |b| i16::from_le_bytes([b[0], b[1]]) as i32).abs();
        let x = h(0x04).max(h(0x0c));
        (x, x)
    }

    /// The ground step `len` long along `way` (a unit, the heading's) as
    /// the height field lets it (0x80064a34, `ground::probe`): had when the
    /// probes are clear; held, slid along the wall's tangent when it is met
    /// at a slant from `head_on` to a right angle and the way along it is
    /// clear or meets a wall there at 512 or more (0x80065120); else held,
    /// with the record that held it (for the climb or the bump).
    /// The probe ahead (0x800645d8) of a step `len` along `way` from here.
    fn ahead_step(&mut self, col: &Col, trig: &Trig, way: [i32; 3], len: i32, solids: &[Exact]) -> ground::Ahead {
        let (lo, hi) = self.body_box();
        let near: Vec<ground::Near> = solids.iter().map(|e| ground::Near { solid: &e.solid, skip: false }).collect();
        let g = ground::Ground {
            col,
            trig,
            near: &near,
            body: ground::Body { lo, hi },
            limit: self.step_limit,
            angles: [self.heading as i16, 0, 0],
            reflect: false,
            low: i32::MIN,
        };
        let mut st = self.probes;
        let r = g.ahead(self.pos, self.floor, way, len, false, &mut st);
        self.probes = st;
        r
    }

    fn ground_move(&mut self, col: &Col, trig: &Trig, way: [i32; 3], len: i32, solids: &[Exact], reflect: bool) -> ((i32, i32), Option<ground::Record>, bool) {
        let (lo, hi) = self.body_box();
        let near: Vec<ground::Near> = solids.iter().map(|e| ground::Near { solid: &e.solid, skip: false }).collect();
        let g = ground::Ground {
            col,
            trig,
            near: &near,
            body: ground::Body { lo, hi },
            limit: self.step_limit,
            angles: [self.heading as i16, 0, 0],
            reflect,
            low: i32::MIN,
        };
        let mut st = self.probes;
        let (pos, floor) = (self.pos, self.floor);
        let character = self.walk_below.is_some();
        let passes = self.passes;
        let mut probe = |w: [i32; 3]| {
            if !character {
                return g.probe(pos, floor, w, len, &mut st);
            }
            match g.probe_middle(pos, floor, w, len, &mut st) {
                ground::Probed::Held { record, .. } if passes => ground::Probed::Clear(ground::Record { delta: 0, ..record }),
                r => r,
            }
        };
        let out = match probe(way) {
            ground::Probed::Clear(_) => (along(way, len, trig), None, false),
            p @ ground::Probed::Held { record, slide } => {
                let a = p.slant().abs();
                match slide {
                    // (A character, flag 0x4000, never slides: 0x80065120
                    // answers no at once.)
                    Some((t, _)) if !character && a >= self.head_on && a < 1024 => {
                        // The way turned along the tangent, the old way's
                        // length kept (its floor square root, 0x8001f4ec),
                        // made a unit again (0x80076a74, 0x8009f278).
                        let old = ((way[0] * way[0] + way[2] * way[2]) as u32).isqrt() as i16 as i32;
                        let w = trig.normal(ground::set_length(t, old, trig));
                        let again = probe(w);
                        if !again.held() || (512..1025).contains(&again.slant().abs()) {
                            (along(w, len, trig), None, true)
                        } else {
                            ((0, 0), Some(record), true)
                        }
                    }
                    // Nor does it climb what holds it at a slant (0x80062cf4:
                    // under `head_on` only): held, no floor to climb to.
                    _ if character && a >= self.head_on => ((0, 0), Some(ground::Record { delta: ground::VOID, ..record }), true),
                    _ => ((0, 0), Some(record), true),
                }
            }
        };
        self.probes = st;
        out
    }

    /// The ground step (dx, dz) as the wall test lets it (0x80066014): three
    /// points ahead of the body, its reach past the step straight on and its
    /// half width to each side, must be in the open at a step's height; a
    /// wall there turns the whole step along whichever axis is open (the
    /// slide, 0x80061a64 / 0x80061b28: seen in the oracle, a run at 45
    /// degrees into a wall goes on along it at full speed). A ledge low
    /// enough to climb is left to the climb.
    fn ground_step(&self, col: &Col, dx: i32, dz: i32) -> (i32, i32) {
        self.ground_step_wall(col, dx, dz).0
    }

    /// [`Self::ground_step`], and whether the wall was met head-on or had
    /// no way along it (the run's bump, 0x8005e1f4: under `head_on`
    /// between the step turned back and the wall's normal, or the slide
    /// meeting a wall again under 512, 0x80065120).
    fn ground_step_wall(&self, col: &Col, dx: i32, dz: i32) -> ((i32, i32), bool) {
        if dx == 0 && dz == 0 {
            return ((0, 0), false);
        }
        let feet = self.pos[1] - self.feet;
        let (side, ahead) = self.extents();
        let reach = if self.carrying && self.carry_climbs { CLIMB_CARRYING.min(self.climb_max) } else { self.climb_max };
        let open = |dx: i32, dz: i32| {
            let len = ((dx as f32).hypot(dz as f32)).max(1.0);
            let (ux, uz) = (dx as f32 / len, dz as f32 / len);
            let far = len + ahead as f32;
            [0.0, side as f32, -side as f32].iter().all(|&w| {
                let x = self.pos[0] + (ux * far + uz * w) as i32;
                let z = self.pos[2] + (uz * far - ux * w) as i32;
                let q = collide::query(col, [x, feet - STEP_UP, z]);
                if q.kind == 0 {
                    return true;
                }
                // A ledge the body can climb: the climb takes it.
                let up = collide::query(col, [x, feet - reach, z]);
                up.kind == 0 && up.floor >= feet - reach
            })
        };
        if open(dx, dz) {
            return ((dx, dz), false);
        }
        // The whole speed along the wall (seen: 170 a frame at 17).
        let len = self.speed * DT;
        let along_x = (dx.signum() * len, 0);
        let along_z = (0, dz.signum() * len);
        // The walls the step meets by itself across each axis, and the
        // slant at which (the cells' sides face along the axes).
        let l = (dx as f32).hypot(dz as f32);
        let slant = |d: i32| ((d.abs() as f32 / l).clamp(-1.0, 1.0).acos() * 4096.0 / std::f32::consts::TAU) as i32;
        let head_on = (dx != 0 && !open(along_x.0, 0) && slant(dx) < self.head_on)
            || (dz != 0 && !open(0, along_z.1) && slant(dz) < self.head_on);
        if head_on {
            return ((0, 0), true);
        }
        let ways = if dx.abs() >= dz.abs() { [along_x, along_z] } else { [along_z, along_x] };
        match ways.into_iter().find(|&(x, z)| (x != 0 || z != 0) && open(x, z)) {
            Some(way) => (way, false),
            None => ((0, 0), true),
        }
    }

    /// What being character `ch` sets on the body: its size class
    /// (0x80062eac: 1 for Spike, 6, and 21, else 0) and so the step it
    /// walks up (0x800c430c: 256, 750), and Spike's own copy of his
    /// hierarchy 1400 deep (0x800450ac sets the copy's +0x10).
    pub fn set_character(&mut self, ch: u16) {
        self.step_limit = if matches!(ch, 6 | 21) { 750 } else { 256 };
        if ch == 6 && let Some(b) = self.hierarchy.header.get_mut(0x10..0x12) {
            b.copy_from_slice(&1400i16.to_le_bytes());
        }
    }

    pub fn set_body(&mut self, hierarchy: Hierarchy, animator: Animator, turn_rate: i32, anims: &Anims) {
        let floor = self.pos[1] - self.feet;
        self.feet = hierarchy.parts.first().map_or(0, |p| p.offset[1]);
        self.pos[1] = floor + self.feet;
        self.hierarchy = hierarchy;
        self.animator = animator;
        self.turn_rate = turn_rate;
        self.animator.set(anim::IDLE, anims);
    }

    /// The feet's height above the root (the hierarchy's root offset).
    pub fn feet(&self) -> i32 {
        self.feet
    }

    /// Thrown through the air (state 16, 0x8005ec9c): along `heading` at
    /// `speed`, up at `up`.
    /// The climb ends where its root motion got to (+0xd8, +0xdc moved by
    /// 0x80066e60), on the ledge it rose to, if there is room there.
    fn finish_climb(&mut self, col: &Col, boxes: &[Box3], whole: bool) {
        let Some(mut c) = self.climb.take() else { return };
        // Run to its end, the last step's motion is had too; the height it
        // ends at against the top is the floor's to take back (the oracle:
        // Chuckie at -193 over a top of -179, then -192).
        if whole {
            c.moved = c.whole;
            self.root_y = c.from[1] + c.whole[1] - c.top;
        }
        let top = c.top - self.feet;
        self.pos[1] = c.top;
        self.floor = top;
        let at = [c.from[0] + c.moved[0], self.pos[1], c.from[2] + c.moved[2]];
        let q = collide::query(col, [at[0], top - 1, at[2]]);
        let on_col = q.kind == 0 && (q.floor - top).abs() <= STEP_UP;
        let on_box = boxes.iter().any(|b| b.covers(at, 0) && (b.min_y - top).abs() <= STEP_UP);
        // Room for the body there: no box over the top within its height.
        let room = !boxes.iter().any(|b| b.covers(at, RADIUS) && b.min_y < top - STEP_UP && b.max_y > top - HEIGHT);
        if (on_col || on_box) && room {
            self.pos[0] = at[0];
            self.pos[2] = at[2];
        }
    }

    /// The base track's root y this step (0x80055904 into +0xda), and the
    /// jump's take-off when its airborne state comes.
    fn root_step(&mut self, anims: &Anims, col: &Col, trig: &Trig) {
        // A state whose word has 32 moves the body by its root keys across
        // (+0xd8, +0xdc, turned by the body's angles, 0x80055904): the bump
        // (Spike's throws him back), Chuckie's pick-up and put-down. (The
        // climb carries itself.)
        // (One run out this step gives what it still had first.)
        // (Each frame's part turned by itself, as 0x80055904 turns each
        // frame the time crosses.)
        let ground = self.climb.is_none() && !matches!(self.state, State::Jump | State::Fall);
        let mut parts: Vec<[i32; 2]> = Vec::new();
        if self.animator.began() {
            if let Some(rest) = self.across_rest.take() {
                for (k, r) in rest.iter().enumerate() {
                    let had = self.bump_root.get(k).copied().unwrap_or([0; 2]);
                    parts.push([r[0] - had[0], r[1] - had[1]]);
                }
            }
            self.bump_root.clear();
        }
        self.across_rest = None;
        if self.animator.moves_across()
            && let Some(frames) = self.animator.root_parts_scaled(anims, None)
        {
            self.bump_root.resize(frames.len(), [0; 2]);
            for (k, (done, _)) in frames.iter().enumerate() {
                let had = self.bump_root[k];
                parts.push([done[0] - had[0], done[2] - had[1]]);
                self.bump_root[k] = [done[0], done[2]];
            }
            let (_, first, count, cur, _) = self.animator.track();
            if cur + 1 == first + count {
                self.across_rest = Some(frames.iter().map(|(_, t)| [t[0], t[2]]).collect());
            }
        }
        let w = parts.iter().filter(|d| **d != [0; 2]).fold([0; 3], |w, d| {
            let t = trig.turn_by([self.heading as i16, 0, 0], [d[0], 0, d[1]]);
            [w[0] + t[0], 0, w[2] + t[2]]
        });
        if ground && w != [0; 3] {
            let (dx, dz) = self.ground_step(col, w[0], w[2]);
            self.pos[0] += dx;
            self.pos[2] += dz;
        }
        let (state, first, count, cur, _) = self.animator.track();
        let (done, total) = self.animator.root_progress(anims).map_or((0, 0), |r| (r.0[1], r.1[1]));
        let was = self.root_seen;
        // An animation that ran to its last frame gives the rest of it.
        let rest = if was.cur + 1 == was.first + was.count && !was.air { was.total - was.done } else { 0 };
        let dy = match (was.state, was.first) == (state, first) {
            // Round again: the rest of the loop, then this pass's part.
            true if cur < was.cur => rest + done,
            true => done - was.done,
            false => rest + done,
        };
        // Root y seen in the air or in a climb does not carry on (the climb
        // puts the body on the top itself).
        let air =
            (matches!(self.state, State::Jump | State::Fall) && self.pending_jump.is_none()) || self.climb.is_some();
        self.root_seen = RootSeen { air, state, first, count, cur, done, total };
        if self.climb.is_some() {
            return;
        }
        if let Some(v) = self.pending_jump {
            // The crouch is table state 5; the state after it (6), or a
            // moving jump's own (41), comes with the launch (0x80060f1c),
            // and the first step in the air is the frame after (the oracle:
            // 868, 868, 814 as the crouch's last part is had, then 664; the
            // run's 778, then 632). Its keys are not had.
            // Off the step the crouch gives way to the state after it (6);
            // a moving jump (41) has no crouch: launched as its state is
            // entered (0x8005b2d8 -> 0x80060f1c), it is in the air the step
            // its animation begins.
            let off = (state != 5 && state != 0 && was.state == 5) || (state == 41 && was.state != 41);
            // (A moving jump's own state adds none of its keys: the
            // original leaves from the run's height that frame, 778 then
            // 632 from the hub's start.)
            if state != was.state {
                if state == 5 {
                    self.root_y += done;
                }
            } else if !off {
                self.root_y += dy;
            }
            if off {
                self.pending_jump = None;
                self.pos[1] += self.root_y;
                self.root_y = 0;
                // Off this frame: the air's first step with it (the
                // animation steps before the state's handler, 0x8005a068).
                // The launch along the heading as it leaves the floor, in
                // the body's frame (the original's +0x170 (-1, -1000, 679)
                // for a run at 17 facing 1246).
                // (Along the way, and in the body, as they were at the start
                // of the frame before: the original sets the launch at the
                // animation's event a frame before its first air step.)
                // (Before this frame's handler: the heading its handler
                // would keep as the way's, +0x178's, the frame before's.)
                let h = self.way_heading;
                self.v = launch_v(trig.turn(h, [0, 0, 4096]), v[1], v[2], h, trig);
                self.launch_at = self.pos;
                self.air_hit = false;
                // The lift's clock and the time in the air from the launch
                // (0x80060f1c).
                self.lift_time = 1024;
                self.air = 0;
            }
            return;
        }
        if matches!(self.state, State::Jump | State::Fall) {
            return;
        }
        // On the ground the body follows the animation's root curve from
        // the floor; a new animation (or a loop's next round) starts from
        // where the body was, which its first frame blends away (+0xde at
        // the frame's start, 0x80055904). The oracle after a climb: -55
        // over the floor, then -41, -45, -40, -23, -18, -10, +1, the run's
        // own curve from its second frame; after a landing, 89 under it
        // and 23 less a frame through the idle's first frame.
        // Its first frame takes the body from where it was to the key
        // (+0xde, 0x80055904), by tz(D frac) of its time gone. The start is
        // taken at the first step that spends time in that frame: begun in
        // the step the animation before ran out in, what that one still had
        // (its rest) comes on top of the height the start is taken from; a
        // start that had no time that step is taken the next, the rest in
        // it. The rest stays until the next start (a loop's next round, or
        // a state asked for again, starts again).
        let switched = (was.state, was.first) != (state, first) || cur < was.cur || self.animator.began();
        let (.., left) = self.animator.track();
        let frame = anims.frames.get(first);
        let dur = frame.map_or(1, |f| f.duration.max(1) as i32);
        let frac = ((dur - left).clamp(0, dur) << 12) / dur;
        if switched {
            if cur == first && frac == 0 {
                self.root_base = self.root_y + rest;
                self.root_rest = 0;
            } else {
                self.root_base = self.root_y;
                self.root_rest = rest;
            }
        }
        self.root_y = if cur > first {
            self.root_rest + done
        } else {
            // (A frame without a root key has its key 0, 0x800557ec.)
            let key = frame
                .and_then(|f| {
                    anims.keys_of(f).iter().find_map(|k| match *k {
                        rugrats_data::anim::Key::Lift { dy, .. } => Some(dy as i32),
                        _ => None,
                    })
                })
                .unwrap_or(0);
            let d = (key - self.root_base).wrapping_mul(frac);
            self.root_base + self.root_rest + if d < 0 { (d + 4095) >> 12 } else { d >> 12 }
        };
    }

    pub fn launch(&mut self, heading: i32, up: i32, speed: i32, trig: &Trig) {
        self.pending_jump = None;
        self.heading = heading;
        self.target = heading;
        self.speed = 0;
        self.action = None;
        self.v = launch_v(trig.turn(heading, [0, 0, 4096]), up, speed, heading, trig);
        self.state = State::Jump;
    }

    /// Starts action animation `a`.
    pub fn act(&mut self, a: i16, anims: &Anims) {
        self.act_at(a, self.animator.written_speed.max(2048), anims);
    }

    /// Starts action animation `a` at animation speed `speed` (0x800572a8's
    /// third argument).
    pub fn act_at(&mut self, a: i16, speed: i32, anims: &Anims) {
        self.action = Some(a);
        self.animator.running = self.speed > (self.gait.0 * 3933) >> 12;
        self.speed = 0;
        self.animator.start_at(a, speed, anims);
    }

    /// One frame in the air (0x80061330), in the original's integers: in
    /// the body's own frame (`v`, +0x170), the stick's `push` against a
    /// drag of 32/4096 of the speed across (its length 0x80076784, its way
    /// 0x8009f278), gravity 14 and `lift` down; the step v dt + a dt^2 / 2
    /// (over 64), turned from the body's frame into the world's
    /// (0x80075ee8). Proved against the original's +0x170 frame by frame
    /// (`hub_start_run_and_jump_as_the_original`).
    fn air_velocity(&mut self, lifting: bool, trig: &Trig) -> [i32; 3] {
        let toward_zero = |v: i32| if v < 0 { (v + 4095) >> 12 } else { v >> 12 };
        let mut moved = [0; 3];
        let mut dt = DT;
        loop {
            // Once the flight has met something (D_800c4bc4 flag 2) the
            // stick no longer pushes and a held jump no longer lifts.
            let push = if self.air_hit { [0, 0] } else { self.push };
            // The held jump's lift (D_800c4288): 1024 at the launch, a
            // timer step (136) less each frame Cross is held on the way
            // up; the frame it runs out has its lift for the rest of the
            // clock's time (back in dt, * 300 / 4096) and the remainder
            // without (0x80061330 has itself again). Any other frame ends
            // it.
            let mut step = dt;
            let mut lift = 0;
            if self.v[1] < 0 && !self.air_hit && self.lift_time > 0 && lifting {
                lift = LIFT;
                self.lift_time -= TIMER_STEP;
                if self.lift_time < 0 {
                    step = toward_zero((TIMER_STEP + self.lift_time) * 300);
                }
            } else {
                self.lift_time = 0;
            }
            // The time in the air for the long fall (D_800c428c), once the
            // lift is spent.
            if self.lift_time <= 0 {
                self.air += step;
            }
            let [vx, vy, vz] = self.v;
            let len = if (vx.unsigned_abs() | vz.unsigned_abs()) >> 16 != 0 {
                0
            } else {
                ((vx * vx + vz * vz) as u32).isqrt() as i32
            };
            let l = toward_zero(-(len << 5));
            let u = collide::normalize([vx, 0, vz]);
            let a = [push[0] + toward_zero(u[0] * l), lift + GRAVITY, push[1] + toward_zero(u[2] * l)];
            let half = |v: i32| (v + (v < 0) as i32) >> 1;
            let d = [0, 1, 2].map(|k| (half(a[k] * step * step) + self.v[k] * step) >> 6);
            self.v = [
                (vx + a[0] * step) as i16 as i32,
                (vy + a[1] * step).min(4096),
                (vz + a[2] * step) as i16 as i32,
            ];
            // Back into the world (0x80075ee8), added to the move (+0xd8).
            let w = trig.turn(self.heading, d);
            for k in 0..3 {
                moved[k] += w[k];
            }
            if self.lift_time >= 0 {
                return moved;
            }
            dt = toward_zero(-self.lift_time * 300);
        }
    }

    /// Part `k`'s joint in the world as posed now (the parts' matrices'
    /// translations, +0x158, UpdatePartRotations).
    pub fn part_at(&self, k: usize, _anims: &Anims, trig: &Trig) -> [i32; 3] {
        self.posed_at(trig).0.get(k).map_or(self.pos, |p| p.joint)
    }

    /// The parts posed at the position itself (no root curve).
    fn posed_at(&self, trig: &Trig) -> (Vec<PartMatrix>, i32) {
        pose::posed(trig, &self.hierarchy, &self.animator.turns(&self.hierarchy), [self.heading as i16, 0, 0], self.pos)
    }

    /// The move test (0x80066014) for the move `d` (+0xd8..+0xdc) against
    /// the height field and the objects `near`: its answer and the object
    /// met (+0xf4), by index into `near`.
    pub fn move_test(&mut self, d: [i32; 3], col: &Col, trig: &Trig, near: &[ground::Near], anims: &Anims) -> (ground::Air, Option<usize>) {
        let plane = collide::query(col, self.pos).floor;
        let (lo, hi) = self.body_box();
        let low = self.lowest_joint(anims, trig);
        let mut st = self.probes;
        st.met.hit = None;
        let g = ground::Ground { col, trig, near, body: ground::Body { lo, hi }, limit: self.step_limit, angles: [self.heading as i16, 0, 0], reflect: false, low };
        let r = g.air(self.pos, d, plane, &mut self.floor, &mut st);
        self.probes = st;
        (r, st.met.hit.map(|(k, _)| k))
    }

    /// The lowest of the body's posed joints (+0x68, set as
    /// UpdatePartRotations, 0x8004a5dc, poses the parts): the air probes
    /// reach down to it at least.
    fn lowest_joint(&self, _anims: &Anims, trig: &Trig) -> i32 {
        self.posed_at(trig).1
    }

    /// The air's move (0x8005f1b4): the velocity's step (0x80061330); held
    /// at the height field's floor under the body (+0x13c); across first,
    /// tested without its fall (0x8005ee78 through 0x80066014) and had as
    /// the test leaves it, then the fall, tested and had. A wall bounces
    /// the body or stops its way across, and the move is had once more
    /// this frame (`again` set for it). Whether it landed.
    #[allow(clippy::too_many_arguments)]
    fn air_move(&mut self, lift: bool, col: &Col, trig: &Trig, solids: &[Exact], anims: &Anims, again: bool) -> bool {
        let saved = self.pos;
        let mut d = self.air_velocity(lift, trig);
        let plane = collide::query(col, self.pos).floor;
        let mut landed = false;
        if self.pos[1] + d[1] >= plane {
            d[1] = plane - self.pos[1];
            if d[1] == 0 {
                self.state = State::Land;
                self.v[1] = 0;
                landed = true;
            }
        }
        let near: Vec<ground::Near> = solids.iter().map(|e| ground::Near { solid: &e.solid, skip: false }).collect();
        let (lo, hi) = self.body_box();
        let low = self.lowest_joint(anims, trig);
        let mut st = self.probes;
        st.below = None;
        if d[0] != 0 || d[2] != 0 {
            let g = ground::Ground { col, trig, near: &near, body: ground::Body { lo, hi }, limit: self.step_limit, angles: [self.heading as i16, 0, 0], reflect: false, low };
            let r = g.air(self.pos, [d[0], 0, d[2]], plane, &mut self.floor, &mut st);
            self.probes = st;
            let (moved, land) = self.air_react(r, [d[0], 0, d[2]], saved, lift, col, trig, solids, anims, again);
            st = self.probes;
            landed |= land;
            self.pos[0] += moved[0];
            self.pos[2] += moved[2];
        }
        if d[1] != 0 {
            let low = self.lowest_joint(anims, trig);
            let g = ground::Ground { col, trig, near: &near, body: ground::Body { lo, hi }, limit: self.step_limit, angles: [self.heading as i16, 0, 0], reflect: false, low };
            let r = g.air(self.pos, [0, d[1], 0], plane, &mut self.floor, &mut st);
            self.probes = st;
            let (moved, land) = self.air_react(r, [0, d[1], 0], saved, lift, col, trig, solids, anims, again);
            landed |= land;
            self.pos[1] += moved[1];
        } else {
            self.probes = st;
        }
        landed
    }

    /// 0x8005ee78: what the air test's answer does to the move `d` (and
    /// the velocity). Clear: had. A wall met going up within 512 of where
    /// the jump began (+0x670): the way across stopped. Met head-on (or at
    /// a right angle and more): the body bounces (0x80061a64: no rising,
    /// no way across, the move had again); at a slant it loses its way
    /// across (0x80061b28) and has the move again. Landed: none of it, the
    /// state 10. A slope overhead met going up: the velocity turned along
    /// it (0x80061be8), the way across stopped. The move left, and whether
    /// it landed.
    #[allow(clippy::too_many_arguments)]
    fn air_react(&mut self, r: ground::Air, d: [i32; 3], saved: [i32; 3], lift: bool, col: &Col, trig: &Trig, solids: &[Exact], anims: &Anims, again: bool) -> ([i32; 3], bool) {
        let jumping = matches!(self.state, State::Jump | State::Fall);
        match r {
            ground::Air::Clear => (d, false),
            ground::Air::Wall { turned } => {
                let slant = match turned {
                    Some((t, a)) => {
                        self.probes.turned = (t, a);
                        a
                    }
                    None => self.probes.turned.1,
                };
                let (dx, dz) = (saved[0] - self.launch_at[0], saved[2] - self.launch_at[2]);
                if jumping && self.v[1] < 0 && ((dx * dx + dz * dz) as u32) <= 0x3ffff {
                    return ([0, d[1], 0], false);
                }
                let a = slant.abs();
                if a < self.head_on || a >= 1024 {
                    if self.v[1] < 0 {
                        self.v[1] = 0;
                    }
                }
                self.air_hit = true;
                if jumping && !again {
                    self.v[0] = 0;
                    self.v[2] = 0;
                    let landed = self.air_move(lift, col, trig, solids, anims, true);
                    return ([0, 0, 0], landed);
                }
                if !(a < self.head_on || a >= 1024) {
                    self.v[0] = 0;
                    self.v[2] = 0;
                }
                (d, false)
            }
            ground::Air::Land => {
                self.state = State::Land;
                self.v = [0; 3];
                ([0; 3], true)
            }
            ground::Air::Slide(t) => {
                if d[1] < 0 && self.v[1] <= 0 {
                    self.air_hit = true;
                    let len = ((self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]) as u32).isqrt() as i16 as i32;
                    let w = ground::set_length(t, len, trig);
                    self.v = to_body(w, self.heading, trig).map(|c| c as i16 as i32);
                    if jumping && !again {
                        let landed = self.air_move(lift, col, trig, solids, anims, true);
                        return ([0, 0, 0], landed);
                    }
                }
                ([0, d[1], 0], false)
            }
        }
    }

    /// The heading as a rotation (0x80020d10's matrix).
    pub fn rotation(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        Mat3::from_cols_array(
            &e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap(),
        )
    }

    /// A climb's root motion so far, turned and scaled into the world
    /// (0x80055904: a body other than 0x800c5680's, 1 Tommy, moves by its
    /// root offset, hierarchy +0x4c, over 0x800c41dc's -561). The y is the
    /// animation's own: the oracle's Tommy rose 904 for a 900 band's
    /// climb, then settled on the top.
    fn climb_motion(&self, anims: &Anims, trig: &Trig) -> Option<(Climb, [i32; 3])> {
        let c = self.climb?;
        (self.action == Some(anim::CLIMB) && self.animator.playing(anim::CLIMB)).then_some(())?;
        let (done, _) = self.animator.root_progress(anims)?;
        // The player's body is 0x800c5680's own character, never scaled
        // (0x80055904 scales only other bodies, by +0x4c over 0x800c41dc).
        let m = self.rotation(trig) * Vec3::new(done[0] as f32, done[1] as f32, done[2] as f32);
        Some((c, [m.x as i32, m.y as i32, m.z as i32]))
    }

    /// Where the body is drawn: during a climb, where its root motion has
    /// carried it from where the climb began; else its position.
    fn drawn_at(&self, anims: &Anims, trig: &Trig) -> Vec3 {
        let pos = Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32);
        match self.climb_motion(anims, trig) {
            Some((c, m)) => Vec3::new((c.from[0] + m[0]) as f32, (c.from[1] + m[1]) as f32, (c.from[2] + m[2]) as f32),
            None => pos + Vec3::new(0.0, self.root_y as f32, 0.0),
        }
    }

    /// Each part's matrix as the original's draw leaves it (+0x158,
    /// 0x8004a5dc), and the joints' greatest y (+0x68).
    pub fn posed(&self, anims: &Anims, trig: &Trig) -> (Vec<PartMatrix>, i32) {
        let at = self.drawn_at(anims, trig);
        let pos = [at.x as i32, at.y as i32, at.z as i32];
        pose::posed(trig, &self.hierarchy, &self.animator.turns(&self.hierarchy), [self.heading as i16, 0, 0], pos)
    }

    pub fn parts(&self, anims: &Anims, trig: &Trig) -> Vec<(Mat3, Vec3)> {
        self.posed(anims, trig).0.iter().map(PartMatrix::float).collect()
    }

    /// The part a hand slot names (hierarchy +0x14 + slot).
    pub fn slot_part(&self, slot: usize) -> Option<usize> {
        self.hierarchy.header.get(0x14 + slot).copied().filter(|&p| p != 0xff).map(|p| p as usize)
    }

    /// The player's triangles: each part's model by its matrix as posed
    /// (0x8004a5dc).
    pub fn triangles(&self, models: &Models, anims: &Anims, trig: &Trig, out: &mut Vec<Vtx>) {
        for (part, (rot, joint)) in self.hierarchy.parts.iter().zip(self.parts(anims, trig)) {
            if let Some(m) = models.get(part.model) {
                models::model_triangles(m, rot, joint, out);
            }
        }
    }
}

/// How far below the feet a floor may drop and still be walked onto, and
/// how far above them a box's top may stand and still be stepped onto
/// (0x80066014 tests 384).
const STEP_DOWN: i32 = 384;
const STEP_UP: i32 = 384;
/// The body's radius and height for the object test.
pub const RADIUS: i32 = 160;
pub const HEIGHT: i32 = 600;
/// A climb under way: where it began, and its root motion so far in the
/// world.
#[derive(Clone, Copy, Debug)]
struct Climb {
    /// The body's height on the top (the ledge plus the feet).
    top: i32,
    /// The climb animation's whole root motion, in the world.
    whole: [i32; 3],
    from: [i32; 3],
    moved: [i32; 3],
}

/// The base track as `Player::root_step` last saw it.
#[derive(Clone, Copy, Debug, Default)]
struct RootSeen {
    /// Seen in the air: none of it carries on to the ground.
    air: bool,
    state: i16,
    first: usize,
    count: usize,
    cur: usize,
    done: i32,
    total: i32,
}

/// The highest ledge a climb takes (0x80056e44: under 1034; under 522
/// while carrying).
pub const CLIMB: i32 = 1033;
const CLIMB_CARRYING: i32 = 521;
/// The highest step message 16 calls too tall (0x80058da4).
const TOO_TALL: i32 = 3071;
/// A landing after this long falling hurts (0x8005b2d8).
const LONG_FALL: i32 = 193;
const FALL_DAMAGE: i32 = 5;
/// Floor flag 2 (slippery) and the slip's fall's damage.
const SLIPPERY: u8 = 2;
const SLIP_DAMAGE: i32 = 5;

/// The step `len` long along `way` (the facing's sine and cosine):
/// 0x8005caac sets the way (+0x178) to that length with 0x80076944, the
/// way normalized (0x8009f278) and scaled, each axis rounded toward zero.
fn along(way: [i32; 3], len: i32, trig: &Trig) -> (i32, i32) {
    let v = ground::set_length(way, len, trig);
    (v[0], v[2])
}

/// A world vector in the body's own frame, facing `heading`
/// (0x80075f20: 0x80020ee0's matrix of (heading, 0, 0), MatrixNormal'd,
/// ApplyMatrixLV).
fn to_body(w: [i32; 3], heading: i32, trig: &Trig) -> [i32; 3] {
    trig.unturn_by([heading as i16, 0, 0], w)
}

/// The launch along `way` (+0x178) `along` long (its VectorNormal scaled,
/// 0x80076944) in the body's frame, with `up` (0x80060f1c).
fn launch_v(way: [i32; 3], up: i32, along: i32, heading: i32, trig: &Trig) -> [i32; 3] {
    let w = ground::set_length(way, along, trig);
    let b = to_body(w, heading, trig);
    [b[0] as i16 as i32, (up + b[1]) as i16 as i32, b[2] as i16 as i32]
}

/// The stick's push in the air (0x80061330), in the body's frame: each
/// axis (raw, 8 times the 4096 scale; up negative) times 340 >> 15, times
/// 0x4a90be59 >> 51 (12 at full), turned by the camera's heading into the
/// world (0x80020bc0, 0x8009f6f8), then into the body's (`to_body`), its
/// sideways part halved. The original's stack, read at 0x800616ac and
/// 0x800616d4 through a jump with Up and Left held: (-12, 0, 12) before,
/// (-1, 0, 16) after, as this gives.
pub fn air_push(stick: [i32; 2], camera: i32, heading: i32, trig: &Trig) -> [i32; 2] {
    let (xr, zr) = (stick[0] * 8, -stick[1] * 8);
    let floor15 = |p: i32| (if p < 0 { p + 0x7fff } else { p }) >> 15;
    let (lx, lz) = (floor15(xr * 340), floor15(-(zr * 340)));
    let scale = |l: i32| {
        let v = (l as i64) << 16;
        (((v * 0x4a90_be59) >> 32) as i32 >> 19) - (v >> 31) as i32
    };
    let (lx, lz) = (scale(lx), scale(lz));
    let a = camera.rem_euclid(4096);
    let (s, c) = (trig.sin(a), trig.cos(a));
    let w = [(c * lx + s * lz) >> 12, 0, (-s * lx + c * lz) >> 12];
    let b = to_body(w, heading, trig);
    [(b[0] + (b[0] < 0) as i32) >> 1, b[2]]
}

/// [`Player::probe_height`] for a body box `lo`..`hi`.
fn probe_height(b: &Box3, at: [i32; 3], lo: [i32; 3], hi: [i32; 3]) -> i32 {
    let mid = ((b.lo[1] + b.hi[1]) as f32 / 2.0 + b.pos.y) as i32;
    mid.clamp(at[1] + lo[1], at[1] + hi[1])
}

/// An object's collision box: its model box (`lo`, `hi`) in the object's
/// space, the object's rotation and position, and the world heights it
/// spans (y down: `min_y` its top).
#[derive(Clone, Copy, Debug)]
pub struct Box3 {
    pub lo: [i32; 3],
    pub hi: [i32; 3],
    pub rot: Mat3,
    pub pos: Vec3,
    pub min_y: i32,
    pub max_y: i32,
    /// Its template has flag 64: it can be climbed onto (0x80058da4).
    pub climb: bool,
    /// Another character's body (no climb test).
    pub character: bool,
    /// The placed object's template (0 for none).
    pub template: u16,
    /// Seen from above, when it is turned other than about y.
    pub outline: Outline,
    /// The object it is a box of, in the original's integers (none for a
    /// character's), and its solid in this frame's list with the box's
    /// index ([`crate::world::boxhit::Exact`]).
    pub obj: Option<Obj>,
    pub solid: Option<(u16, u8)>,
}

/// An object as the original holds it for the collision tests: position
/// (+0x18), angles (+0x24), model and which of its boxes this is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Obj {
    pub pos: [i32; 3],
    pub angles: [i16; 3],
    pub shape: u32,
    pub index: u8,
}

/// A box's outline seen from above when it is turned other than about y
/// (a thing tipped on its side, level 3's templates 526 and 650): its
/// corners' convex hull, counter-clockwise in (x, z); two points for one
/// edge on (level 6's wheels, flat in their own plane). Empty (`n` 0) for
/// one turned only about y, its own x and z rectangle.
#[derive(Clone, Copy, Debug, Default)]
pub struct Outline {
    pts: [[f32; 2]; 8],
    n: u8,
}

impl Outline {
    /// The outline of the box `lo`..`hi` turned by `rot` at `pos`.
    pub fn of(lo: [i32; 3], hi: [i32; 3], rot: Mat3, pos: Vec3) -> Outline {
        if (rot.y_axis.y - 1.0).abs() < 1e-3 {
            return Outline::default();
        }
        let mut pts = [[0.0f32; 2]; 8];
        for (k, p) in pts.iter_mut().enumerate() {
            let c = Vec3::new(
                if k & 1 == 0 { lo[0] } else { hi[0] } as f32,
                if k & 2 == 0 { lo[1] } else { hi[1] } as f32,
                if k & 4 == 0 { lo[2] } else { hi[2] } as f32,
            );
            let w = pos + rot * c;
            *p = [w.x, w.z];
        }
        // The convex hull (monotone chain).
        pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
        let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
        let mut hull = [[0.0f32; 2]; 16];
        let mut n = 0;
        for pass in 0..2 {
            let start = n;
            for k in 0..8 {
                let p = pts[if pass == 0 { k } else { 7 - k }];
                while n >= start + 2 && cross(hull[n - 2], hull[n - 1], p) <= 0.0 {
                    n -= 1;
                }
                hull[n] = p;
                n += 1;
            }
            n -= 1;
        }
        let mut out = Outline { pts: [[0.0; 2]; 8], n: n.min(8) as u8 };
        out.pts[..out.n as usize].copy_from_slice(&hull[..out.n as usize]);
        out
    }

    /// Its edges: each start and outward normal (x, z).
    fn edges(&self) -> impl Iterator<Item = ([f32; 2], [f32; 2])> + '_ {
        let n = self.n as usize;
        (0..n).map(move |k| {
            let (e0, e1) = (self.pts[k], self.pts[(k + 1) % n]);
            (e0, [e1[1] - e0[1], e0[0] - e1[0]])
        })
    }
}

impl Box3 {
    /// Whether a circle of radius `r` at `p` (x, z) overlaps the box seen
    /// from above.
    pub fn covers(&self, p: [i32; 3], r: i32) -> bool {
        self.depth(p, r) > 0.0
    }

    /// Whether the step from `a` to `b` (x, z) passes through the box's
    /// own rectangle seen from above (0x80069c40 tests the movement
    /// segment): a thin wall cannot be stepped across.
    pub fn crosses(&self, a: [i32; 3], b: [i32; 3]) -> bool {
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        let mut clip = |o: f32, d: f32, lo: f32, hi: f32| {
            if d.abs() < 1e-6 {
                return lo <= o && o <= hi;
            }
            let (mut u, mut v) = ((lo - o) / d, (hi - o) / d);
            if u > v {
                std::mem::swap(&mut u, &mut v);
            }
            t0 = t0.max(u);
            t1 = t1.min(v);
            t0 <= t1
        };
        match self.outline.n {
            0 => {
                let local = |p: [i32; 3]| self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
                let (p, q) = (local(a), local(b));
                for i in [0, 2] {
                    if !clip(p[i], q[i] - p[i], self.lo[i] as f32, self.hi[i] as f32) {
                        return false;
                    }
                }
            }
            // Edge on: the step crosses its segment.
            1 | 2 => {
                let (e0, e1) = (self.outline.pts[0], self.outline.pts[(self.outline.n - 1) as usize]);
                let (pa, pb) = ([a[0] as f32, a[2] as f32], [b[0] as f32, b[2] as f32]);
                let cross = |o: [f32; 2], p: [f32; 2], q: [f32; 2]| (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0]);
                let (d1, d2) = (cross(e0, e1, pa), cross(e0, e1, pb));
                let (d3, d4) = (cross(pa, pb, e0), cross(pa, pb, e1));
                // Not when the step starts on it.
                return d1 != 0.0 && d1.signum() != d2.signum() && d3.signum() != d4.signum();
            }
            _ => {
                // Each edge's outward normal: the segment's part inside it.
                let (pa, pb) = ([a[0] as f32, a[2] as f32], [b[0] as f32, b[2] as f32]);
                for (e0, n) in self.outline.edges() {
                    let o = n[0] * (pa[0] - e0[0]) + n[1] * (pa[1] - e0[1]);
                    let d = n[0] * (pb[0] - pa[0]) + n[1] * (pb[1] - pa[1]);
                    if !clip(o, d, f32::NEG_INFINITY, 0.0) {
                        return false;
                    }
                }
            }
        }
        // Not when the step only leaves it (it began inside).
        t0 > 0.0
    }

    /// Where the segment from `a` to `b` first enters the box (in 3D, in
    /// the box's own frame): the fraction along it, the point, and the
    /// outward normal (4096 a unit) of the face it enters by. None when it
    /// misses or starts inside.
    pub fn segment_hit(&self, a: [i32; 3], b: [i32; 3]) -> Option<(f32, [i32; 3], [i32; 3])> {
        let local = |p: [i32; 3]| self.rot.transpose() * (Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) - self.pos);
        let (p, q) = (local(a), local(b));
        let d = q - p;
        let (mut t0, mut t1, mut axis) = (0.0f32, 1.0f32, None);
        for k in 0..3 {
            let (lo, hi) = (self.lo[k] as f32, self.hi[k] as f32);
            if d[k].abs() < 1e-6 {
                if p[k] < lo || p[k] > hi {
                    return None;
                }
                continue;
            }
            let (mut u, mut v, mut sign) = ((lo - p[k]) / d[k], (hi - p[k]) / d[k], -1.0);
            if u > v {
                std::mem::swap(&mut u, &mut v);
                sign = 1.0;
            }
            if u > t0 {
                t0 = u;
                axis = Some((k, sign));
            }
            t1 = t1.min(v);
            if t0 > t1 {
                return None;
            }
        }
        let (k, sign) = axis?;
        let mut n = Vec3::ZERO;
        n[k] = sign;
        let n = self.rot * n;
        let at =
            Vec3::new(a[0] as f32, a[1] as f32, a[2] as f32).lerp(Vec3::new(b[0] as f32, b[1] as f32, b[2] as f32), t0);
        Some((
            t0,
            [at.x as i32, at.y as i32, at.z as i32],
            [(n.x * 4096.0) as i32, (n.y * 4096.0) as i32, (n.z * 4096.0) as i32],
        ))
    }

    /// The outward normal (4096 a unit, in the world) of the side of the
    /// box seen from above that `p`, outside it, is beyond the most.
    pub fn side_normal(&self, p: [i32; 3]) -> [i32; 3] {
        let local = self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
        let sides = [
            (self.lo[0] as f32 - local.x, Vec3::NEG_X),
            (local.x - self.hi[0] as f32, Vec3::X),
            (self.lo[2] as f32 - local.z, Vec3::NEG_Z),
            (local.z - self.hi[2] as f32, Vec3::Z),
        ];
        let (_, n) = sides.into_iter().fold((f32::MIN, Vec3::X), |a, s| if s.0 > a.0 { s } else { a });
        let n = self.rot * n;
        [(n.x * 4096.0) as i32, 0, (n.z * 4096.0) as i32]
    }

    /// How deep a circle of radius `r` at `p` is in the box seen from above:
    /// the distance to the nearest side, negative outside.
    pub fn depth(&self, p: [i32; 3], r: i32) -> f32 {
        let r = r as f32;
        match self.outline.n {
            0 => {
                let local = self.rot.transpose() * (Vec3::new(p[0] as f32, self.pos.y, p[2] as f32) - self.pos);
                let x = (local.x - (self.lo[0] as f32 - r)).min(self.hi[0] as f32 + r - local.x);
                let z = (local.z - (self.lo[2] as f32 - r)).min(self.hi[2] as f32 + r - local.z);
                x.min(z)
            }
            // How far inside each edge (its outward normal a unit).
            // Edge on (a wheel standing in its own plane): a segment.
            1 | 2 => {
                let (a, b) = (self.outline.pts[0], self.outline.pts[(self.outline.n - 1) as usize]);
                let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
                let (px, pz) = (p[0] as f32 - a[0], p[2] as f32 - a[1]);
                let t = ((px * dx + pz * dz) / (dx * dx + dz * dz).max(1e-6)).clamp(0.0, 1.0);
                r - (px - t * dx).hypot(pz - t * dz)
            }
            _ => self
                .outline
                .edges()
                .map(|(e0, n)| {
                    let len = n[0].hypot(n[1]).max(1e-6);
                    r - (n[0] * (p[0] as f32 - e0[0]) + n[1] * (p[2] as f32 - e0[1])) / len
                })
                .fold(f32::INFINITY, f32::min),
        }
    }

}

/// An angle difference in -2048..2048.
fn wrap(a: i32) -> i32 {
    (a + 2048).rem_euclid(4096) - 2048
}

impl Player {
    /// Whether it turns where it stands (0x80059204 for the player, which
    /// the stand reaches only when the stick's speed is no walk or run,
    /// 0x800590d4 and 0x80059008: both need one): the stick pushed (+0x1b0)
    /// and no speed from it (+0x184), the turn left to make (folded as the
    /// turn folds it); else 0.
    fn turning_in_place(&self) -> i32 {
        if self.stick_size != 0 && self.stick_speed == 0 { fold(self.target - self.heading) } else { 0 }
    }

    /// The turn rate this frame (0x8005bbf4): a slow body (Spike, 34)
    /// turning in place turns half as fast again.
    fn rate(&self) -> i32 {
        if self.turn_rate == SLOW_TURN && self.turning_in_place() != 0 { (self.turn_rate * 3) >> 1 } else { self.turn_rate }
    }
}

/// The turn's difference of two headings in 0..4096 (0x8005bc80): from
/// 2049 less 4096, under -2048 plus 4096, so a half turn either way keeps
/// its sign (a body facing 0 asked for 2048 turns up through 1024).
fn fold(d: i32) -> i32 {
    if d >= 2049 {
        d - 4096
    } else if d < -2048 {
        d + 4096
    } else {
        d
    }
}
