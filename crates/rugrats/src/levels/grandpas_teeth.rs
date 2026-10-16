//! Level 2, Grandpa's Teeth (docs/levels/level-02.md): a goose has the
//! teeth. Phase 1 the yard to the gap in the fence (room 69, cine 3);
//! phase 2 Tommy rides Spike after the goose to the shuffleboard court
//! (room 44, cine 4); phase 3 Tommy kicks pucks at the goose while it
//! chases Chuckie, 130 s on the clock. The goose at 0 plays cine 5: won.

use rugrats_data::hierarchy::Hierarchy;
use rugrats_render::{Vtx, models};

use crate::actors::animator::Animator;
use crate::actors::npc::{Event, Mode, Npc, REACH, Target};
use crate::actors::player::anim::IDLE;
use crate::actors::player::{DT, Pose, SLOW_TURN, TURN};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::levels::Level;
use crate::ui::hud;
use crate::world::game::{Effect, Exit, Game, Script};

/// Registry ids (LP +0x2a).
const CHUCKIE: u16 = 1776;
const GOOSE: u16 = 11;
const SPIKE: u16 = 14;
const GEESE: [u16; 5] = [900, 901, 902, 903, 904];
/// Shown by cine 3, hidden at the start (0x8007914c).
const BOARD: u16 = 335;
const HIDDEN: u16 = 1313;
/// Characters (hierarchies).
const TOMMY: usize = 1;
const SPIKE_CHAR: u16 = 6;

/// Cinematics: the gap (3), the court (4), the win (5), the loss (16).
const CINE_GAP: usize = 3;
const CINE_COURT: usize = 4;
const CINE_WIN: usize = 5;
const CINE_LOSE: usize = 16;

/// The respawn at the start (0x8007914c) and tries (0x800b2920[2]).
const RESPAWN: [i32; 3] = [1720, -245, 5608];
const TRIES: i32 = 3;
/// Phase 3: Tommy's spot and heading (0x8007c6e0), Spike's, the clock
/// (0x80078dd8: 39000 dt, 130 s).
const KICK_SPOT: [i32; 3] = [37355, -236, 23544];
const SPIKE_SPOT: [i32; 3] = [36000, -245, 25000];
const CLOCK: i32 = 39000;
/// The goose's start in phase 2 (0x800825b0) and its first post.
const GOOSE_START: [i32; 3] = [52232, -527, -27468];
const GOOSE_POST: u8 = 88;
const CHUCKIE_POST: u8 = 114;
/// The goose's route ends at post 90 in room 44, the court.
const GOOSE_END: u8 = 90;
const COURT_ROOM: usize = 44;
/// A puck (0x8004d1a8): speed 200 (template +0x1a), 50 damage, model 756.
const PUCK_SPEED: i32 = 200 * 65536 / 6000;
const PUCK_DAMAGE: i32 = 50;
const PUCK_MODEL: u32 = 756;
const PUCK_REACH: i32 = 420;
const PUCK_RANGE: i32 = 16000;

/// A puck kicked.
struct Puck {
    pos: [i32; 3],
    v: [i32; 2],
    travelled: i32,
}

/// The level's progress (0x800c4cd0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    /// The yard to the gap in the fence.
    Yard = 1,
    /// Tommy rides Spike after the goose to the court.
    Chase = 2,
    /// Tommy kicks pucks at the goose on the court, 130 s.
    Court = 3,
}

pub struct GrandpasTeeth {
    phase: Phase,
    tries: i32,
    respawn: Pose,
    puck: Option<Puck>,
    tommy: (Hierarchy, Animator),
    spike: (Hierarchy, Animator),
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl GrandpasTeeth {
    /// EpisodeStart (0x8007914c).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<GrandpasTeeth, String> {
        let tommy = (g.player.hierarchy.clone(), Animator::load(disc, TOMMY as u16)?);
        let spike_h = g.set.hierarchies.get(SPIKE_CHAR as usize).cloned().flatten().ok_or("no hierarchy for Spike")?;
        let spike = (spike_h, Animator::load(disc, SPIKE_CHAR)?);
        let mut l = GrandpasTeeth {
            phase: Phase::Yard,
            tries: TRIES,
            respawn: Pose::new(RESPAWN, g.player.heading),
            puck: None,
            tommy,
            spike,
        };
        // Chuckie follows Tommy (0x80083764, speed index 4).
        if let Some(c) = find(g, CHUCKIE) {
            g.npcs[c].mode = Mode::Follow(Target::Player);
            g.npcs[c].speed_index = 4;
        }
        // The geese guard their spots (mode 11, bites of 10).
        for id in GEESE {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Guard;
                n.attacks = true;
                n.health = 255;
            }
        }
        {
            let id = HIDDEN;
            if let Some(k) = find(g, id) {
                g.npcs[k].hidden = true;
            }
        }
        if let Some(k) = find(g, BOARD) {
            g.npcs[k].hidden = g.checkpoint.is_none();
        }
        if g.checkpoint.is_some() {
            l.respawn = g.player.pose();
        }
        Ok(l)
    }

    /// CineFinish (0x800783d8) for this level's cinematics.
    pub fn cine_finished(&mut self, g: &mut Game, n: usize) {
        match n {
            CINE_GAP => {
                if let Some(k) = find(g, BOARD) {
                    g.npcs[k].hidden = false;
                }
                self.chase(g, false);
            }
            CINE_COURT => self.court(g),
            CINE_WIN => {
                if let Some(k) = find(g, GOOSE) {
                    g.npcs[k].hidden = true;
                }
                // 0x8007737c case 4: exit 1, the level won.
                g.won = Some(true);
                g.exit = Some(Exit::to(Level::HUB));
            }
            CINE_LOSE => {
                g.won = Some(false);
                g.exit = Some(Exit::to(Level::HUB));
            }
            _ => {}
        }
    }

    /// Phase 2 (0x8007c08c): Tommy rides Spike after the goose, which
    /// carries Chuckie along its route of posts.
    fn chase(&mut self, g: &mut Game, back: bool) {
        g.timer = None;
        self.puck = None;
        g.player.frozen = false;
        let (h, a) = (self.spike.0.clone(), self.spike.1.clone());
        g.player.set_body(h, a, SLOW_TURN, &g.anims);
        // Spike as an NPC stands down; Tommy rides the player (y - 250).
        if let Some(k) = find(g, SPIKE) {
            g.npcs[k].hidden = true;
            g.npcs[k].mode = Mode::Idle;
        }
        if find(g, 0xffff).is_none() {
            let mut rider = Npc {
                id: 0xffff,
                character: TOMMY,
                hierarchy: self.tommy.0.clone(),
                animator: self.tommy.1.clone(),
                pos: g.player.pos,
                heading: g.player.heading,
                speed: 0,
                speed_index: 0,
                range: 0,
                wait: 0,
                damage: 0,
                health: 255,
                max_health: 255,
                mode: Mode::Ride { on: Target::Player, up: 250, turn: 0 },
                home: g.player.pos,
                hidden: false,
                attacks: false,
                cooldown: 0,
                first_post: 0,
                loops: (0, 0),
                size: 0,
                rooted: false,
                bumped: false,
                reach: 600,
                bite_wait: 900,
                run_speed: None,
                post_reach: REACH,
                hold: None,
            };
            rider.animator.set(IDLE, &g.anims);
            g.npcs.push(rider);
        }
        let goose = find(g, GOOSE);
        if let Some(k) = goose {
            let n = &mut g.npcs[k];
            n.range = 4000;
            n.speed_index = 5;
            n.damage = 0;
            n.attacks = false;
            n.health = n.max_health;
            n.hidden = false;
            n.place(GOOSE_START, &g.col);
            n.mode = Mode::Posts { prev: 0, cur: GOOSE_POST };
        }
        if let (Some(c), Some(k)) = (find(g, CHUCKIE), goose) {
            g.npcs[c].mode = Mode::Ride { on: Target::Npc(k), up: 200, turn: 2048 };
            g.npcs[c].health = g.npcs[c].max_health;
        }
        if !back {
            self.respawn = g.player.pose();
        }
        self.phase = Phase::Chase;
    }

    /// Phase 3 (0x8007c6e0): Tommy at the end of the court, kicking pucks;
    /// the goose after Chuckie round the court; 130 s.
    fn court(&mut self, g: &mut Game) {
        let (h, a) = (self.tommy.0.clone(), self.tommy.1.clone());
        g.player.set_body(h, a, TURN, &g.anims);
        if let Some(k) = find(g, 0xffff) {
            g.npcs.remove(k);
        }
        g.player.pos = KICK_SPOT;
        g.player.heading = 2048;
        g.player.target = 2048;
        g.player.frozen = true;
        g.set_camera_yaw(2048.0);
        let (chuckie, goose) = (find(g, CHUCKIE), find(g, GOOSE));
        if let (Some(k), Some(c)) = (goose, chuckie) {
            // The route ends at post 90 on the court; a goose not there yet
            // (a try lost, a restart) starts from it.
            let court = g.set.room_info.get(COURT_ROOM).copied();
            let n = &mut g.npcs[k];
            let on_court = court.is_some_and(|r| {
                (r.min[0]..=r.max[0]).contains(&n.pos[0]) && (r.min[2]..=r.max[2]).contains(&n.pos[2])
            });
            if !on_court {
                let at = g.posts.get(GOOSE_END as usize - 1).map_or(n.pos, |p| p.pos);
                n.place(at, &g.col);
            }
            n.range = 2000;
            n.speed_index = 2;
            n.attacks = true;
            n.health = n.max_health;
            n.mode = Mode::Follow(Target::Npc(c));
        }
        if let Some(c) = chuckie {
            let n = &mut g.npcs[c];
            n.speed_index = 4;
            n.health = 200;
            n.max_health = 200;
            let at = g.posts.get(CHUCKIE_POST as usize - 1).map_or(n.pos, |p| p.pos);
            n.place(at, &g.col);
            n.mode = Mode::Posts { prev: 0, cur: CHUCKIE_POST };
        }
        if let Some(s) = find(g, SPIKE) {
            g.npcs[s].hidden = false;
            g.npcs[s].heading = 2048;
            g.npcs[s].place(SPIKE_SPOT, &g.col);
            g.npcs[s].mode = Mode::Idle;
        }
        g.timer = Some(CLOCK);
        self.phase = Phase::Court;
    }

    /// ForceCry (0x8007fd0c): a try gone; back to the respawn, or out.
    pub fn cry(&mut self, g: &mut Game) {
        if g.exit.is_some() || g.cine.is_some() {
            return;
        }
        self.tries -= 1;
        g.player.health = 255;
        if self.tries <= 0 {
            if self.phase == Phase::Court {
                // 0x8007ec48: the loss, cine 16, then the hub.
                g.timer = None;
                g.cine = Some(CINE_LOSE);
            } else {
                g.exit = Some(Exit::to(Level::HUB));
            }
            return;
        }
        let Pose { pos: at, heading } = self.respawn;
        match self.phase {
            Phase::Yard => {
                g.player.pos = at;
                g.player.heading = heading;
                if let Some(c) = find(g, CHUCKIE) {
                    g.npcs[c].place(at, &g.col);
                }
            }
            Phase::Chase | Phase::Court => {
                if self.phase == Phase::Court {
                    // Back to phase 2, the court's cinematic armed again.
                    if let Some(r) = g.set.room_info.get_mut(44) {
                        r.flags |= 2;
                    }
                }
                self.chase(g, true);
                g.player.pos = at;
                g.player.heading = heading;
            }
        }
        g.set_camera_yaw(g.player.heading as f32);
    }

    /// One frame of the level's own play.
    pub fn frame(&mut self, g: &mut Game, pads: &Pads) {
        // Bites (Damage, 0x80077864): on the player, health; at 0 a cry.
        for e in std::mem::take(&mut g.npc_events) {
            match e {
                Event::Bite { target: Target::Player, amount, .. } if amount > 0 => {
                    g.player.hurt(amount);
                    g.play(30);
                }
                _ => {}
            }
        }
        if g.player.health <= 0 {
            g.cry();
        }
        if self.phase != Phase::Court {
            return;
        }
        // A kick (state 12, the puck's use 0x8004d1a8): along the facing.
        if self.puck.is_none() && pads.pressed(0, input::TRIANGLE) {
            let (s, c) = (g.trig.sin(g.player.heading), g.trig.cos(g.player.heading));
            let start = [
                g.player.pos[0] + ((s * 512) >> 12),
                g.player.pos[1] - g.player.feet(),
                g.player.pos[2] + ((c * 512) >> 12),
            ];
            self.puck = Some(Puck { pos: start, v: [(s * PUCK_SPEED) >> 12, (c * PUCK_SPEED) >> 12], travelled: 0 });
            g.play(36);
            g.player.act(7, &g.anims);
        }
        if let Some(p) = &mut self.puck {
            p.pos[0] += (p.v[0] * DT) >> 6;
            p.pos[2] += (p.v[1] * DT) >> 6;
            p.travelled += ((p.v[0].abs() + p.v[1].abs()) * DT) >> 6;
            // A hit on the goose or Chuckie (0x80086ef0): Damage 50.
            let hit = [GOOSE, CHUCKIE].into_iter().filter_map(|id| find(g, id)).find(|&k| {
                let n = &g.npcs[k];
                let (dx, dz) = ((n.pos[0] - p.pos[0]) as f64, (n.pos[2] - p.pos[2]) as f64);
                ((dx * dx + dz * dz).sqrt() as i32) < PUCK_REACH
            });
            let gone = p.travelled > PUCK_RANGE;
            if let Some(k) = hit {
                let n = &mut g.npcs[k];
                n.hurt(PUCK_DAMAGE);
                let then = match &n.mode {
                    Mode::Hit { then, .. } => then.clone(),
                    m => Box::new(m.clone()),
                };
                n.mode = Mode::Hit { left: 300, then };
                g.effects.push(Effect::new(34));
                // 0x8007eb68: the goose at 0 wins, Chuckie at 0 is a cry.
                if n.health == 0 {
                    if n.id == GOOSE {
                        g.cine = Some(CINE_WIN);
                        g.timer = None;
                    } else {
                        self.puck = None;
                        g.cry();
                        return;
                    }
                }
            }
            if hit.is_some() || gone {
                self.puck = None;
            }
        }
    }

    /// The puck, the clock and the two gauges (as text for now).
    pub fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        if let Some(p) = &self.puck
            && let Some(m) = g.set.models.get(PUCK_MODEL)
        {
            let pos = rrt::glam::Vec3::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32);
            models::model_triangles(m, rrt::glam::Mat3::IDENTITY, pos, moving);
        }
        if self.phase == Phase::Court {
            // The two gauges (0x8008c5e0 in 0x8007c6e0): the goose's (278)
            // bottom left, Chuckie's (279) bottom right.
            if let Some(s) = &g.sprites {
                if let Some(k) = find(g, GOOSE) {
                    hud::gauge(s, 278, (-200, 88), g.npcs[k].health, g.npcs[k].max_health, g.focal(), overlay);
                }
                if let Some(k) = find(g, CHUCKIE) {
                    hud::gauge(s, 279, (200, 88), g.npcs[k].health, g.npcs[k].max_health, g.focal(), overlay);
                }
            }
        }
    }
}

impl Script for GrandpasTeeth {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        GrandpasTeeth::frame(self, g, pads);
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        GrandpasTeeth::cine_finished(self, g, n);
    }

    fn cry(&mut self, g: &mut Game) -> bool {
        GrandpasTeeth::cry(self, g);
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        GrandpasTeeth::draw(self, g, moving, overlay);
    }

    /// The clock (0x8007ecc4): a cry.
    fn timer_expired(&mut self, g: &mut Game) {
        g.cry();
    }

    /// +0x30 bit 8: phase 3.
    fn cue(&self) -> i32 {
        (self.phase == Phase::Court) as i32
    }

    fn phase(&self) -> u8 {
        self.phase as u8
    }
}
