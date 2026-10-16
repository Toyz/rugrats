//! Level 3, Incident on Aisle 7 (docs/levels/level-03.md). Phase 1 the
//! store: the balloon (cine 8) or the cart ride (room 19, cine 9) brings
//! Tommy to the bakery, where he throws pies at Larry and Steve (the pie
//! toss, 0x8007bc60) as they walk rings of posts toward him. Four hits play
//! cine 13 and phase 2, the lobster aisles; if one reaches post 39 first,
//! cine 15 and back to the start. Rooms 22 and 36 start phase 3: the boss
//! lobster guards the switch, whose use plays cine 14: won.

use rrt::glam::Vec3;
use rugrats_render::Vtx;

use crate::actors::npc::{Event, Mode, Target};
use crate::actors::player::Pose;
use crate::actors::player::anim::THROW;
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::levels::Level;
use crate::levels::pie_toss::{Pie, draw_pies, roll_pies};
use crate::ui::hud;
use crate::world::collide;
use crate::world::game::{self, Exit, Game, Script, Strike};

/// Registry ids (LP +0x2a).
const LARRY: u16 = 75;
const STEVE: u16 = 76;
const BOSS: u16 = 601;
const LOBSTERS: [u16; 9] = [83, 84, 85, 86, 2007, 2008, 2009, 2010, 2011];
/// The boss's "bullies", grown by Gameplay script 1 (0x8007dfe0).
const BULLIES: [u16; 3] = [2008, 2009, 2010];
/// Shown after cine 86, hidden at the start.
const LADDER: u16 = 333;
const BALLOON: u16 = 81;
const CART: u16 = 79;

/// Cinematics: the balloon (8), the cart ride (9), Mellon Rush (13), the
/// loss of the toss (15), the boss (86).
const CINE_BALLOON: usize = 8;
const CINE_CART: usize = 9;
const CINE_WON_TOSS: usize = 13;
const CINE_LOST_TOSS: usize = 15;
const CINE_BOSS: usize = 86;
const CART_ROOM: usize = 19;
const BOSS_ROOM: usize = 36;

/// EpisodeStart (0x800793c8): the start, a respawn.
const START: [i32; 3] = [6671, 779, -14463];
/// The pie toss (0x8007bc60): Tommy on the counter facing -z, the fixed
/// camera's eye, Larry and Steve's shared health.
const COUNTER: [i32; 3] = [45615, -2300, 19446];
/// The aim marker: 24000 ahead and 1024 down (0x800c42fc, 0x800c42f8),
/// sprite 426 (TEXPOS + 0x13f8) in red (0x800c402c..34), opaque.
const MARKER_AHEAD: i32 = 24000;
const MARKER_DOWN: i32 = 1024;
const MARKER: u32 = 426;
const MARKER_RGB: [u8; 3] = [255, 0, 0];
const TOSS_EYE: Vec3 = Vec3::new(45493.0, -4200.0, 22720.0);
const TOSS_HEALTH: i32 = 200;
/// A pie hit takes 50 (0x8007e308).
const PIE_DAMAGE: i32 = 50;
/// The lobster aisles' respawn (0x8007ece4).
const AISLES: [i32; 3] = [45476, 779, 14446];
/// The boss's swat (0x80084084 mode 3): 0x800c4268 = -1750 up and
/// 0x800c426c = 2500 along. How state 16 uses the second is not read;
/// half of it as the air velocity lands where the oracle's throw did
/// (4425 back).
const SWAT_UP: i32 = -1750;
const SWAT_SPEED: i32 = 2500 / 2;
/// A lobster's bite (+0x6d) and the stun a throw gives the boss (600 dt).
const BITE: i32 = 15;
const STUN: i32 = 600;

/// The pie toss under way.
struct Toss {
    holding: bool,
    pies: Vec<Pie>,
}

/// The level's progress (0x800c4cd0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    /// The store, to the bakery and the pie toss.
    Store = 1,
    /// The lobster aisles, after the toss is won.
    Aisles = 2,
    /// The boss lobster's area, from room 22 or 36.
    Boss = 3,
}

pub struct AisleSeven {
    phase: Phase,
    respawn: Pose,
    toss: Option<Toss>,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl AisleSeven {
    /// EpisodeStart (0x800793c8), after cine 2 (whose end hides 75, 76
    /// and 13).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<AisleSeven, String> {
        for id in [LARRY, STEVE, 13, LADDER] {
            g.hide(id, true);
        }
        g.player.pos = START;
        let heading = g.player.heading;
        g.yaw = heading as f32;
        g.checkpoint = Some(Pose::new(START, heading));
        // Lobsters prowl their posts from the start (0x8008395c, mode 9),
        // the boss blocks (0x800839f8, mode 3).
        for id in LOBSTERS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Prowl { prev: 0, cur: n.first_post };
                n.damage = BITE;
            }
        }
        if let Some(k) = find(g, BOSS) {
            let n = &mut g.npcs[k];
            n.mode = Mode::Block { heading: n.heading };
        }
        Ok(AisleSeven { phase: Phase::Store, respawn: Pose::new(START, heading), toss: None })
    }

    /// The aim marker (0x8005fe44): (0, 1024, 24000) turned by Tommy's
    /// heading from his position, pulled in to where the ray (0x80090554)
    /// first meets the world (the segment test, 0x8002cd7c) or Larry or
    /// Steve (templates 590 / 591; 0x8006a714 against their bounds, here
    /// the bounds cube).
    fn marker(&self, g: &Game) -> [i32; 3] {
        let p = g.player.pos;
        let (s, c) = (g.trig.sin(g.player.heading), g.trig.cos(g.player.heading));
        let far = [p[0] + ((s * MARKER_AHEAD) >> 12), p[1] + MARKER_DOWN, p[2] + ((c * MARKER_AHEAD) >> 12)];
        let mut hit = collide::Segment::default();
        let mut to = if collide::segment(&g.col, p, far, &mut hit) == 1 { hit.point } else { far };
        let len = |q: [i32; 3]| (0..3).map(|k| ((q[k] - p[k]) as f64).powi(2)).sum::<f64>().sqrt();
        for id in [LARRY, STEVE] {
            let Some(k) = find(g, id) else { continue };
            let n = &g.npcs[k];
            // The nearest point of the ray to the body, inside its bounds.
            let d = [to[0] - p[0], to[1] - p[1], to[2] - p[2]].map(|v| v as f64);
            let r = [n.pos[0] - p[0], n.pos[1] - p[1], n.pos[2] - p[2]].map(|v| v as f64);
            let dd = d.iter().map(|v| v * v).sum::<f64>().max(1.0);
            let f = (r.iter().zip(&d).map(|(a, b)| a * b).sum::<f64>() / dd).clamp(0.0, 1.0);
            let q = [0, 1, 2].map(|k| p[k] + (d[k] * f) as i32);
            let half = n.half_size().max(1);
            if (0..3).all(|k| (q[k] - n.pos[k]).abs() <= half) && len(q) < len(to) {
                to = q;
            }
        }
        to
    }

    /// The pie toss (0x8007bc60).
    fn pie_toss(&mut self, g: &mut Game) {
        g.player.pos = COUNTER;
        g.player.heading = 2048;
        g.player.target = 2048;
        g.player.speed = 0;
        g.player.frozen = true;
        g.yaw = 2048.0;
        let look = Vec3::new(COUNTER[0] as f32, -500.0, 8000.0);
        g.fixed = Some((TOSS_EYE, look));
        // The pie on the counter is the pool's: the toss draws its own.
        for t in g.things.iter_mut().filter(|t| t.template == 564) {
            t.hidden = true;
        }
        for id in [LARRY, STEVE] {
            g.hide(id, false);
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.health = TOSS_HEALTH;
                n.max_health = TOSS_HEALTH;
                let home = n.home;
                n.place(home, &g.col);
                n.speed_index = 5;
                n.mode = Mode::Posts { prev: 0, cur: n.first_post };
            }
        }
        self.toss = Some(Toss { holding: true, pies: Vec::new() });
        g.song = Some(1);
    }

    /// The end of the toss (0x8007e3c0): won, Mellon Rush (cine 13); else
    /// Larry and Steve win (cine 15) and Tommy cries back to the start.
    fn end_toss(&mut self, g: &mut Game, won: bool) {
        self.toss = None;
        g.player.frozen = false;
        g.fixed = None;
        for id in [LARRY, STEVE] {
            if let Some(k) = find(g, id) {
                g.npcs[k].mode = Mode::Idle;
            }
        }
        if let Some(r) = g.set.room_info.get_mut(CART_ROOM) {
            if won {
                r.flags &= !2;
            } else {
                r.flags |= 2;
            }
        }
        if won {
            g.disable_use(BALLOON);
            g.cine = Some(CINE_WON_TOSS);
        } else {
            g.cine = Some(CINE_LOST_TOSS);
            // ForceCry at once (phase 1: no cry): a try gone.
            g.tries -= 1;
            if g.tries <= 0 {
                g.exit = Some(Exit::to(Level::HUB));
            }
        }
    }

    /// 0x8007ece4: the lobsters back home, Tommy at the aisles, phase 2.
    fn aisles(&mut self, g: &mut Game) {
        if self.phase == Phase::Boss {
            g.hide(LADDER, true);
            if let Some(r) = g.set.room_info.get_mut(BOSS_ROOM) {
                r.flags |= 2;
            }
        }
        for id in LOBSTERS {
            if let Some(k) = find(g, id) {
                g.hide(id, false);
                let n = &mut g.npcs[k];
                let home = n.home;
                n.place(home, &g.col);
                n.health = n.max_health;
                n.mode = Mode::Prowl { prev: 0, cur: n.first_post };
            }
        }
        self.put(g, AISLES, 2048);
        self.respawn = Pose::new(AISLES, 2048);
        g.checkpoint = Some(self.respawn);
        self.phase = Phase::Aisles;
    }

    fn put(&self, g: &mut Game, at: [i32; 3], heading: i32) {
        g.player.pos = at;
        g.player.heading = heading;
        g.player.target = heading;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.set_camera_yaw(heading as f32);
    }

    /// The toss's frame: Triangle throws the pie held, or takes the next.
    fn toss_frame(&mut self, g: &mut Game, pads: &Pads) {
        let Some(toss) = &mut self.toss else { return };
        if pads.pressed(0, input::TRIANGLE) && g.player.action.is_none() {
            if toss.holding {
                let aim = find(g, LARRY).map(|k| g.npcs[k].pos);
                toss.pies.push(Pie::thrown(g, COUNTER, g.player.heading, aim));
                toss.holding = false;
                g.player.act_at(THROW, crate::world::game::THROW_ANIM_SPEED, &g.anims);
            } else {
                toss.holding = true;
            }
        }
        let targets: Vec<usize> = [LARRY, STEVE].iter().filter_map(|&id| find(g, id)).collect();
        let hits = roll_pies(g, &mut toss.pies, &targets);
        // A hit (0x800511a8, 0x8007e308): both lose 50, the one hit
        // flinches; at 0 the toss is won.
        for hit in hits {
            // The one hit says so (0x800511a8: Larry 1065, Steve 1076 /
            // 1079).
            let r = g.rand.next() & 1;
            g.speech.extend(game::hit_line(g.npcs[hit].character, Level::AISLE_SEVEN, r));
            let mut left = TOSS_HEALTH;
            for id in [LARRY, STEVE] {
                if let Some(k) = find(g, id) {
                    let n = &mut g.npcs[k];
                    n.hurt(PIE_DAMAGE);
                    left = n.health;
                    let then = Box::new(n.mode.clone());
                    if !matches!(n.mode, Mode::Hit { .. }) {
                        n.mode = Mode::Hit { left: 600, then };
                    }
                }
            }
            // At the one hit (0x800511a8: 0x8006f644(44, it)).
            let at = g.npcs[hit].pos;
            g.play_at(44, at);
            if left == 0 {
                self.end_toss(g, true);
                return;
            }
        }
    }
}

impl Script for AisleSeven {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            match e {
                Event::Bite { by, target: Target::Player, amount } if amount > 0 => {
                    g.player.hurt(amount);
                    // At the biter (0x80084084: 0x8006f644(52, it)).
                    let at = g.npcs[by].pos;
                    g.play_at(52, at);
                }
                // GoalPost (0x80077ff0): one of them reached post 39.
                Event::Goal(k) if self.toss.is_some() && [LARRY, STEVE].contains(&g.npcs[k].id) => {
                    self.end_toss(g, false);
                }
                Event::Swat { by, heading } => {
                    g.player.launch(heading, SWAT_UP, SWAT_SPEED, &g.trig);
                    // At the swatter (0x800857fc: 0x8006f644(148, it)).
                    let at = g.npcs[by].pos;
                    g.play_at(148, at);
                }
                _ => {}
            }
        }
        // Thrown objects (0x800511a8): a lobster loses the thing's +0x6d
        // and at 0 is gone (0x8007ebd4); the boss is stunned 600 dt.
        for Strike { npc: k, damage: amount, .. } in std::mem::take(&mut g.struck) {
            let id = g.npcs[k].id;
            let at = g.npcs[k].pos;
            g.play_at(44, at);
            let n = &mut g.npcs[k];
            let back = match &n.mode {
                Mode::Hit { then, .. } => (**then).clone(),
                m => m.clone(),
            };
            if id == BOSS {
                n.mode = Mode::Hit { left: STUN, then: Box::new(back) };
            } else if LOBSTERS.contains(&id) {
                n.hurt(amount);
                if n.health == 0 {
                    // 0x8007ebd4: sparks with the lobster's +0x6d and
                    // sound 48 (0x80050fd0), then gone.
                    n.mode = Mode::Idle;
                    g.flights.sign_sparks(at, BITE as i16, g.sprites.as_ref());
                    g.play(48);
                    g.hide(id, true);
                } else {
                    n.mode = Mode::Hit { left: 600, then: Box::new(back) };
                }
            }
        }
        self.toss_frame(g, pads);
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        match n {
            CINE_BALLOON | CINE_CART => {
                if n == CINE_CART {
                    g.hide(CART, true);
                }
                self.pie_toss(g);
            }
            CINE_WON_TOSS => {
                for id in [LARRY, STEVE, 74] {
                    g.hide(id, true);
                }
                g.song = Some(0);
                self.aisles(g);
            }
            CINE_LOST_TOSS => {
                for id in [LARRY, STEVE] {
                    g.hide(id, true);
                }
                self.put(g, START, self.respawn.heading);
            }
            CINE_BOSS => {
                g.hide(LADDER, false);
                g.song = Some(1);
                let at = [g.player.pos[0], g.player.pos[1], 8000];
                self.put(g, at, g.player.heading);
            }
            _ => {}
        }
    }

    fn gameplay(&mut self, g: &mut Game, n: u8) {
        // Script 1 (0x8007dfe0): phase 3, the respawn here, the bullies
        // grown to a half size of 1000.
        if n == 1 {
            self.phase = Phase::Boss;
            self.respawn = g.player.pose();
            g.checkpoint = Some(self.respawn);
            for id in BULLIES {
                if let Some(k) = find(g, id) {
                    g.npcs[k].size = g.npcs[k].size.max(1000);
                }
            }
        }
    }

    /// ForceCry's level 3 case (0x8007fd0c): a try gone; phase 3 starts
    /// the aisles again (0x8007ece4); back to the respawn.
    fn cry(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some(Exit::to(Level::HUB));
            return true;
        }
        if self.phase == Phase::Boss {
            self.aisles(g);
        } else if self.phase == Phase::Aisles {
            for id in LOBSTERS {
                if let Some(k) = find(g, id)
                    && g.npcs[k].health > 0
                {
                    let home = g.npcs[k].home;
                    g.npcs[k].place(home, &g.col);
                }
            }
        }
        let Pose { pos: at, heading } = self.respawn;
        self.put(g, at, heading);
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        let Some(toss) = &self.toss else { return };
        draw_pies(g, &toss.pies, toss.holding, moving);
        if toss.holding
            && let Some(s) = &g.sprites
        {
            let at = self.marker(g);
            let clip = g.stage.camera * rrt::glam::Vec4::new(at[0] as f32, at[1] as f32, at[2] as f32, 1.0);
            if clip.w > 16.0 {
                let (x, y) = (256.0 + 256.0 * clip.x / clip.w, 120.0 - 120.0 * clip.y / clip.w);
                s.at_pixels(MARKER, (x, y), MARKER_RGB, overlay);
            }
        }
        // The gauge (0x8008c5e0 on Larry): model 280 over 282.
        if let (Some(s), Some(k)) = (&g.sprites, find(g, LARRY)) {
            hud::gauge(s, 280, (-148, 73), g.npcs[k].health, TOSS_HEALTH, g.focal(), overlay);
        }
    }

    /// +0x30 bit 8: the pie toss.
    fn cue(&self) -> i32 {
        self.toss.is_some() as i32
    }

    fn phase(&self) -> u8 {
        self.phase as u8
    }
}
