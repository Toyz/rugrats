//! Level 5, Visitors from Outer Space (docs/levels/level-05.md): Angelica
//! on the alien ship with the TV remote. Triangle zaps (3 of 120 charge);
//! a zap stuns a TV robot 20 s and opens a sliding door; batteries give 30.
//! The elevator (id 1969) takes her down while she rides it; room 42 is
//! phase 2. Room 30's cine 38 sets her floating through the ducts with one
//! try (phase 3); room 38's cine 40 wins.

use rrt::glam::Vec3;
use rugrats_render::Vtx;

use crate::actors::npc::{Event, Mode, Target};
use crate::actors::player::{DT, Pose};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::levels::Level;
use crate::world::collide;
use crate::world::fx::{BEAM, Mesh};
use crate::world::game::{Collected, Exit, Game, Script};

/// Hidden by cine 37's end (0x800732d4 case 37) and 0x8007b280.
const CINE_ACTORS: [u16; 11] = [1776, 608, 609, 1492, 130, 132, 131, 134, 514, 524, 8801];
/// Where cine 37 leaves her (oracle).
const START: [i32; 3] = [38395, -5590, 28281];
const START_HEADING: i32 = 1895;
/// The remote (template 828): 120 charge, 3 a zap, the gauge (models
/// 360 / 361 at (((512 * 5) >> 7) - 192, 88 - (240 >> 4))).
const REMOTE: u16 = 828;
const CHARGE: i32 = 120;
const ZAP_COST: i32 = 3;
const GAUGE: (i16, i16) = (-172, 73);
/// The TV robots (template 826, char 35): guard 8000 from home, bite
/// within 1536; a zap stuns them 6000 dt.
const ROBOT: usize = 35;
const ROBOT_REACH: i32 = 1536;
const STUN: i32 = 6000;
/// Sliding doors (template 843): open within 5000, sound 71.
const DOOR: u32 = 843;
const DOOR_REACH: i64 = 5000;
/// The elevator (template 855, id 1969): 60 a frame in level 5
/// (0x80046ed8).
const ELEVATOR_TEMPLATE: u16 = 855;
/// Phase 2's respawn (0x8007b3d0) and the float's start (0x8007b4a0).
const DECK_ROOM: usize = 42;
const DECK: [i32; 3] = [73246, 6697, 9170];
const DUCT_ROOM: usize = 28;
const GARAGE_ROOM: usize = 38;
const FLOAT_AT: [i32; 3] = [92984, -27, 57036];
const CINE_FLOAT: usize = 38;
/// The beam (0x80091834, 0x80090fac, table 0x800bb28c entry 0): speed
/// +0x44 = 10 + 2048, so (speed x dt) >> 12 of the direction a frame
/// (20580 at dt 10); life 1000 less 33 a frame (0x800b44d4[dt]); tested
/// along the way in steps (0x80090860 tests the segment).
const BEAM_SPEED: i32 = (10 + 2048) * DT;
const BEAM_LIFE: i32 = 1000;
const BEAM_DECAY: i32 = 33;
const BEAM_STEP: i32 = 256;
const BEAM_HIT: i64 = 700;
/// The beam mesh's width (0x80090d34: scale 5000).
const BEAM_WIDTH: f32 = 5000.0;

struct Beam {
    /// Where it was and is (+0x14.., +0x04..), its direction (4096 a
    /// unit), its life, and whether it has stopped against something.
    from: [i32; 3],
    pos: [i32; 3],
    dir: [i32; 2],
    life: i32,
    done: bool,
    /// The first wall hit reflects it (+0x2e); later ones stop it.
    bounced: bool,
}

/// A sliding door: its placement, where it stands, whether it is open.
#[derive(Clone, Copy, Debug)]
struct SlidingDoor {
    placement: usize,
    at: [i32; 3],
    open: bool,
}

/// The level's progress (0x800c4cd0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    /// The upper deck, from cine 37's end.
    UpperDeck = 1,
    /// The lower deck, from room 42.
    LowerDeck = 2,
    /// Floating through the ducts after cine 38, one try.
    Float = 3,
}

pub struct OuterSpace {
    phase: Phase,
    respawn: Pose,
    charge: i32,
    doors: Vec<SlidingDoor>,
    beams: Vec<Beam>,
    mesh: Option<Mesh>,
}

impl OuterSpace {
    /// EpisodeStart (0x8007b214) and cine 37's end (0x8007b280).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<OuterSpace, String> {
        for id in CINE_ACTORS {
            g.hide(id, true);
        }
        g.player.pos = START;
        g.player.heading = START_HEADING;
        g.player.target = START_HEADING;
        g.set_camera_yaw(START_HEADING as f32);
        if let Some(k) = g.things.iter().position(|t| t.template == REMOTE) {
            g.give(k);
        }
        for n in g.npcs.iter_mut().filter(|n| n.character == ROBOT) {
            n.mode = Mode::Guard;
            n.reach = ROBOT_REACH;
            n.bite_wait = 230;
        }
        // The init sets template 855's y step to 60 (0x80046ed8: template
        // +0x1a), so every mover of it.
        for m in g.movers.iter_mut().filter(|m| m.template == ELEVATOR_TEMPLATE) {
            m.set_step(1, 60);
        }
        let doors = g
            .set
            .placements
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind == 1 && p.template == DOOR)
            .map(|(placement, p)| SlidingDoor { placement, at: p.pos, open: false })
            .collect();
        let respawn = Pose::new(START, START_HEADING);
        g.checkpoint = Some(respawn);
        let mesh = Mesh::load(disc, &BEAM).ok();
        Ok(OuterSpace { phase: Phase::UpperDeck, respawn, charge: CHARGE, doors, beams: Vec::new(), mesh })
    }

    fn set_phase(&mut self, g: &mut Game, phase: Phase, at: [i32; 3], heading: i32) {
        self.phase = phase;
        self.respawn = Pose::new(at, heading);
        g.checkpoint = Some(self.respawn);
    }

    fn put(g: &mut Game, at: [i32; 3], heading: i32) {
        g.player.pos = at;
        g.player.heading = heading;
        g.player.target = heading;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.set_camera_yaw(heading as f32);
    }

    /// A door opens (sound 71) or shuts.
    fn door(&mut self, g: &mut Game, i: usize, open: bool) {
        let d = &mut self.doors[i];
        if d.open != open {
            d.open = open;
            g.hide_placement(d.placement, open);
            // At the door (0x8004d078 / 0x8004d0b0: 0x8006f644(71, it)).
            let at = g.set.placements[d.placement].pos;
            g.play_at(71, at);
        }
    }

    /// The zap (0x80059d3c): a beam from her hand along her facing.
    fn zap(&mut self, g: &mut Game) {
        g.player.act(39, &g.anims);
        if self.charge <= 0 {
            return;
        }
        self.charge = (self.charge - ZAP_COST).max(0);
        // At her (0x80059d3c: 0x8006f644(68, player)).
        let me = g.player.pos;
        g.play_at(68, me);
        let (s, c) = (g.trig.sin(g.player.heading), g.trig.cos(g.player.heading));
        let p = g.player.pos;
        let at = [p[0], p[1] - 500, p[2]];
        self.beams.push(Beam { from: at, pos: at, dir: [s, c], life: BEAM_LIFE, done: false, bounced: false });
    }

    fn beams(&mut self, g: &mut Game) {
        let mut opened = Vec::new();
        let mut beams = std::mem::take(&mut self.beams);
        beams.retain_mut(|b| {
            b.life -= BEAM_DECAY;
            if b.life <= 0 || b.done {
                return false;
            }
            b.from = b.pos;
            // The segment this frame, in steps: the first wall or thing
            // stops it there.
            let mut gone = 0;
            while gone < BEAM_SPEED {
                let step = BEAM_STEP.min(BEAM_SPEED - gone);
                gone += step;
                let next = [b.pos[0] + ((b.dir[0] * step) >> 12), b.pos[1], b.pos[2] + ((b.dir[1] * step) >> 12)];
                let q = collide::query(&g.col, next);
                if q.kind != 0 {
                    if b.bounced {
                        b.done = true;
                        break;
                    }
                    // The first wall reflects it about the wall's normal
                    // (0x8007666c, sound 70), the normal from the ray
                    // 0x80090554's world part, the segment test 0x8002cd7c.
                    b.bounced = true;
                    let mut hit = collide::Segment::default();
                    let n = if collide::segment(&g.col, b.pos, next, &mut hit) == 1 {
                        hit.normal
                    } else {
                        [-b.dir[0], 0, -b.dir[1]]
                    };
                    let u = collide::normalize([b.dir[0], 0, b.dir[1]]);
                    let c = collide::fix(n[0], u[0]) + collide::fix(n[1], u[1]) + collide::fix(n[2], u[2]);
                    if c < 0 {
                        let r = [0, 1, 2].map(|k| u[k] - (c.wrapping_mul(n[k]) >> 11));
                        b.dir = [r[0], r[2]];
                    } else {
                        b.dir = [-b.dir[0], -b.dir[1]];
                    }
                    let at = b.pos;
                    g.play_at(70, at);
                    continue;
                }
                b.pos = next;
                let near = |p: [i32; 3]| {
                    let (dx, dz) = ((p[0] - b.pos[0]) as i64, (p[2] - b.pos[2]) as i64);
                    dx * dx + dz * dz < BEAM_HIT * BEAM_HIT && (p[1] - b.pos[1]).abs() < 2500
                };
                // A robot (0x800511a8 case 33): stunned 20 s, sound 63.
                if let Some(n) = g.npcs.iter_mut().find(|n| n.character == ROBOT && !n.hidden && near(n.pos)) {
                    n.mode = Mode::Hit { left: STUN, then: Box::new(Mode::Guard) };
                    let at = n.pos;
                    g.play_at(63, at);
                    b.done = true;
                    break;
                }
                // A door (0x80051d44): it opens.
                if let Some(i) = self.doors.iter().position(|d| !d.open && near(d.at)) {
                    opened.push(i);
                    b.done = true;
                    break;
                }
            }
            true
        });
        self.beams = beams;
        for i in opened {
            self.door(g, i, true);
        }
    }
}

impl Script for OuterSpace {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e
                && amount > 0
            {
                g.player.hurt(amount);
            }
        }
        // Batteries (kind 12, 0x80051c68): +30 charge, sound 69.
        for Collected { kind, value } in std::mem::take(&mut g.collected) {
            if kind == 12 {
                self.charge = (self.charge + value).min(CHARGE);
                g.play(69);
            }
        }
        // Triangle with the remote (0x8005a6f4): the zap.
        let holding = g.carried().is_some_and(|k| g.things[k].template == REMOTE);
        if holding && pads.pressed(0, input::TRIANGLE) && g.player.action.is_none() {
            self.zap(g);
        }
        self.beams(g);
        // Sliding doors (0x8004d078, 0x8004d0b0): open within 5000.
        let p = g.player.pos;
        for i in 0..self.doors.len() {
            let at = self.doors[i].at;
            let (dx, dz) = (((at[0] - p[0]) as i64).abs(), ((at[2] - p[2]) as i64).abs());
            let d = dx.max(dz) + dx.min(dz) / 2;
            let near = d < DOOR_REACH && (at[1] - p[1]).abs() < 4000;
            self.door(g, i, near);
        }
    }

    fn room(&mut self, g: &mut Game, room: usize) {
        match room {
            DECK_ROOM => self.set_phase(g, Phase::LowerDeck, DECK, 1015),
            DUCT_ROOM if g.player.float.is_some() => g.cry(),
            GARAGE_ROOM => g.player.float = None,
            _ => {}
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_FLOAT {
            // 0x800732d4 case 38, 0x8007b4a0: the float, one try.
            for id in [8800, 131, 142, 134] {
                g.hide(id, true);
            }
            g.hide(8801, false);
            Self::put(g, FLOAT_AT, 441);
            g.player.float = Some((FLOAT_AT[1], 0));
            self.set_phase(g, Phase::Float, FLOAT_AT, 441);
            g.tries = 1;
            g.song = Some(1);
        }
    }

    /// ForceCry's level 5 case (0x8007ff18, 0x8007b624).
    fn cry(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some(Exit::to(Level::HUB));
            return true;
        }
        if self.phase == Phase::UpperDeck {
            for m in g.movers.iter_mut().filter(|m| m.template == ELEVATOR_TEMPLATE) {
                m.reset();
            }
        }
        let Pose { pos: at, heading } = self.respawn;
        Self::put(g, at, heading);
        if self.phase == Phase::Float {
            g.player.float = Some((at[1], 0));
        }
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        // The beams (0x80090d34): the mesh stretched over this frame's
        // segment.
        if let Some(mesh) = &self.mesh {
            for b in &self.beams {
                let at = |p: [i32; 3]| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32);
                mesh.along(at(b.from), at(b.pos), BEAM_WIDTH, moving);
            }
        }
        // The charge gauge (0x8008c7f0: 360 over 361).
        if let Some(s) = &g.sprites {
            let top = 2.0 * 54.0 * (CHARGE - self.charge).max(0) as f32 / CHARGE as f32;
            s.draw(361, GAUGE, 36.0, 54.0, 1024.0, top, g.focal(), overlay);
            s.draw(360, GAUGE, 36.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
        }
    }

    fn phase(&self) -> u8 {
        self.phase as u8
    }

    fn describe(&self) -> String {
        let b: Vec<_> = self.beams.iter().map(|b| (b.pos, b.dir, b.bounced, b.done)).collect();
        format!(
            "phase {:?} charge {} beams {:?} doors open {}",
            self.phase,
            self.charge,
            b,
            self.doors.iter().filter(|d| d.open).count()
        )
    }
}
