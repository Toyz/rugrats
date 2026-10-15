//! Level 20, No More Cookies (docs/levels/level-20.md): the babies keep
//! Angelica's cookie box from her. Circle or Square eats a cookie (15 to
//! go: cine 64 wins). Triangle lobs the box to the baby in front; when it
//! comes near, the player becomes that baby. Angelica runs after the box;
//! her lunge knocks it away: a lost round (cine 65).
//!
//! Level 21, Touchdown Tommy (docs/levels/level-21.md), runs the same
//! code with a bottle: 20 drinks, a throw costs one too, only a drink
//! wins; the round start gives the bottle back to Tommy's body.

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::npc::{Mode, Target};

/// The box (template 442) and the cookies in it.
const BOX: u16 = 442;
const COOKIES: i32 = 15;
/// The babies one can throw to (templates 2, 6, 7, 11: ids 1776, 609,
/// 608, 1492), and Angelica.
const BABIES: [u16; 4] = [1776, 609, 608, 1492];
const ANGELICA: u16 = 600;
/// The catch: the box within 1400 (x and z) of the target (0x80057518).
const CATCH: i32 = 1400;
/// Angelica's lunge reach (+0x68c 2000) and her knock (touching).
const LUNGE: i64 = 2000;
const KNOCK: i64 = 1200;
/// The counter's sprite (313) at (-160, 50).
const COUNTER: u32 = 313;
const CINE_WON: usize = 64;
const CINE_LOST: usize = 65;
/// Eating: animation 42, sound 123; a cookie about every 20 frames.
const EAT: i16 = 42;
const EAT_WAIT: i32 = 20;

/// What differs between levels 20 and 21.
#[derive(Clone, Copy)]
struct Rules {
    item: u16,
    count: i32,
    won: usize,
    lost: usize,
    eat: i16,
    /// A throw costs one (the bottle's +0x50 effect, 0x80060e3c).
    throw_costs: bool,
    /// The round start swaps the player back into Tommy (0x8007a0ec).
    back_to_tommy: bool,
}

const RULES_20: Rules = Rules { item: BOX, count: COOKIES, won: CINE_WON, lost: CINE_LOST, eat: EAT, throw_costs: false, back_to_tommy: false };
const RULES_21: Rules = Rules { item: 259, count: 20, won: 50, lost: 51, eat: 40, throw_costs: true, back_to_tommy: true };

pub struct Level20 {
    rules: Rules,
    cookies: i32,
    /// The registry id the player's body carries.
    me: u16,
    /// The baby aimed at, and the one a box in flight goes to.
    target: Option<usize>,
    catcher: Option<usize>,
    eat_wait: i32,
    lunge: i32,
    ended: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level20 {
    /// EpisodeStart (0x80079ff0) and the round start (0x80079cb0).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Level20, String> {
        let mut l = Level20 { rules: RULES_20, cookies: COOKIES, me: 1492, target: None, catcher: None, eat_wait: 0, lunge: 0, ended: false };
        l.round(g);
        g.checkpoint = Some((g.player.pos, g.player.heading));
        Ok(l)
    }

    /// Level 21's EpisodeStart (0x8007a444) and cine 49's end
    /// (0x8007a314): Lil's new home, the grown-ups hidden, the round.
    pub fn start21(g: &mut Game, _disc: &GameDisc) -> Result<Level20, String> {
        for id in [13, 15, 16] {
            g.hide(id, true);
        }
        if let Some(k) = find(g, 609) {
            g.npcs[k].home = [10030, 779, 10051];
        }
        let mut l = Level20 { rules: RULES_21, cookies: 20, me: 1492, target: None, catcher: None, eat_wait: 0, lunge: 0, ended: false };
        l.round(g);
        g.checkpoint = Some((g.player.pos, g.player.heading));
        Ok(l)
    }

    fn round(&mut self, g: &mut Game) {
        // Level 21 hands the bottle back to Tommy's body.
        if self.rules.back_to_tommy && self.me != 1492 {
            if let Some(k) = find(g, 1492) {
                self.swap(g, k);
            }
        }
        self.cookies = self.rules.count;
        self.catcher = None;
        self.ended = false;
        let item = self.rules.item;
        if let Some(k) = g.things.iter().position(|t| t.template == item) {
            g.things[k].keep = true;
            g.give(k);
        }
        for id in BABIES {
            if let Some(k) = find(g, id) {
                let home = g.npcs[k].home;
                g.npcs[k].place(home, &g.col);
                g.npcs[k].mode = Mode::Idle;
            }
        }
        if let Some(k) = find(g, ANGELICA) {
            let home = g.npcs[k].home;
            g.npcs[k].place(home, &g.col);
            g.npcs[k].speed_index = 2;
            g.npcs[k].mode = Mode::Follow(Target::Player);
            g.npcs[k].attacks = false;
        }
        g.player.gait = (17, 17);
    }

    /// 0x80059b34 in levels 20-21: the nearest baby within 512 of the
    /// heading.
    fn aim(&self, g: &Game) -> Option<usize> {
        let p = g.player.pos;
        let h = g.player.heading;
        g.npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| BABIES.contains(&n.id) && n.id != self.me && !n.hidden)
            .filter(|(_, n)| {
                let (dx, dz) = ((n.pos[0] - p[0]) as f64, (n.pos[2] - p[2]) as f64);
                let yaw = ((dx.atan2(dz) / std::f64::consts::TAU) * 4096.0).round() as i32;
                ((yaw - h + 2048).rem_euclid(4096) - 2048).abs() < 513
            })
            .min_by_key(|(_, n)| (n.pos[0] - p[0]) as i64 * (n.pos[0] - p[0]) as i64 + (n.pos[2] - p[2]) as i64 * (n.pos[2] - p[2]) as i64)
            .map(|(k, _)| k)
    }

    /// 0x80057570: the player and the catching baby trade bodies, places
    /// and ids; the box goes to the new holder.
    fn swap(&mut self, g: &mut Game, k: usize) {
        let here = g.player.pos;
        let heading = g.player.heading;
        let n = &mut g.npcs[k];
        std::mem::swap(&mut g.player.hierarchy, &mut n.hierarchy);
        std::mem::swap(&mut g.player.animator, &mut n.animator);
        let there = n.pos;
        n.pos = here;
        n.heading = heading;
        n.mode = Mode::Idle;
        let (old_char, old_id) = (g.character, self.me);
        g.character = n.character;
        n.character = old_char;
        self.me = n.id;
        n.id = old_id;
        g.player.pos = there;
        g.player.v = [0; 3];
        let item = self.rules.item;
        if let Some(b) = g.things.iter().position(|t| t.template == item) {
            g.give(b);
        }
    }

    /// A lost round (ForceCry case 19, 0x80079f5c): no cry, a try, cine
    /// 65.
    fn lose(&mut self, g: &mut Game) {
        if self.ended {
            return;
        }
        self.ended = true;
        g.tries -= 1;
        g.effects.push((121, crate::sound::EFFECTS_VOLUME));
        g.drop_carried();
        g.cine = Some(self.rules.lost);
    }
}

impl crate::game::Script for Level20 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        g.npc_events.clear();
        if self.ended {
            return;
        }
        self.eat_wait = (self.eat_wait - 1).max(0);
        let item = self.rules.item;
        let holding = g.carried().is_some_and(|k| g.things[k].template == item);
        // Eating (0x80057950): Circle or Square, standing.
        let standing = matches!(g.player.state, crate::player::State::Stand);
        if holding && standing && self.eat_wait == 0 && (pads.pressed(0, input::CIRCLE) || pads.pressed(0, input::SQUARE)) {
            self.cookies -= 1;
            self.eat_wait = EAT_WAIT;
            g.player.act(self.rules.eat, &g.anims);
            let sound = if self.rules.throw_costs { 43 + (g.rand.next() & 15) as u8 } else { 123 };
            g.effects.push((sound, crate::sound::EFFECTS_VOLUME));
            if self.cookies <= 0 {
                self.ended = true;
                g.cine = Some(self.rules.won);
                return;
            }
        }
        self.target = self.aim(g);
        // The throw (0x80060b44's levels 20-21 branch): a lob to the
        // target.
        if holding && pads.pressed(0, input::TRIANGLE) {
            if let Some(k) = self.target {
                let (p, q) = (g.player.pos, g.npcs[k].pos);
                let d = (((q[0] - p[0]) as f64).hypot((q[2] - p[2]) as f64)).max(1.0);
                let frames = (d / 400.0).max(10.0);
                let per = 64.0 / 10.0;
                let v = [
                    ((q[0] - p[0]) as f64 / frames * per) as i32,
                    -(65.0 * frames) as i32,
                    ((q[2] - p[2]) as f64 / frames * per) as i32,
                ];
                g.player.act(crate::player::anim::THROW, &g.anims);
                g.effects.push((if g.rand.next() & 1 == 0 { 159 } else { 167 }, crate::sound::EFFECTS_VOLUME));
                g.throw_carried(v);
                if self.rules.throw_costs {
                    self.cookies = (self.cookies - 1).max(0);
                }
                self.catcher = Some(k);
                g.npcs[k].mode = Mode::Idle;
            }
        }
        // The catch (0x80057518): the box within 1400 of the catcher.
        if let Some(k) = self.catcher {
            if let Some(b) = g.things.iter().find(|t| t.template == item) {
                let q = g.npcs[k].pos;
                if (b.pos.x as i32 - q[0]).abs() < CATCH && (b.pos.z as i32 - q[2]).abs() < CATCH {
                    self.catcher = None;
                    self.swap(g, k);
                } else if b.state == crate::game::ThingState::Rest {
                    // A missed throw: the box on the floor.
                    self.catcher = None;
                }
            }
        }
        // Angelica: after the box; her lunge knocks it from the holder,
        // and a box on the floor is lost when she lunges.
        if let Some(a) = find(g, ANGELICA) {
            let p = g.player.pos;
            let q = g.npcs[a].pos;
            let d2 = (q[0] - p[0]) as i64 * (q[0] - p[0]) as i64 + (q[2] - p[2]) as i64 * (q[2] - p[2]) as i64;
            self.lunge = (self.lunge - 1).max(0);
            // The lunge (state 28, sound 121) carries her about 1200 on.
            let floor_box = !holding && self.catcher.is_none();
            if d2 < LUNGE * LUNGE && self.lunge == 0 {
                self.lunge = 40;
                g.effects.push((121, crate::sound::EFFECTS_VOLUME));
                if d2 < KNOCK * KNOCK && (holding || floor_box) {
                    self.lose(g);
                }
            }
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == self.rules.lost {
            if g.tries <= 0 {
                g.won = Some(false);
                g.exit = Some((crate::game::HUB, None));
            } else {
                // Card 709, then the round again (0x80079f1c).
                g.card = Some(709);
                if let Some((at, h)) = g.checkpoint {
                    g.player.pos = at;
                    g.player.heading = h;
                }
                self.round(g);
            }
        }
    }

    fn cry(&mut self, g: &mut Game) -> bool {
        self.lose(g);
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        if let Some(s) = &g.sprites {
            if self.rules.throw_costs {
                // Level 21's bottle gauge (0x8008c7f0: 366 over 365, max 20).
                let top = 2.0 * 54.0 * (self.rules.count - self.cookies.max(0)) as f32 / self.rules.count as f32;
                s.draw(365, (-172, 73), 36.0, 54.0, 1024.0, top, g.focal(), overlay);
                s.draw(366, (-172, 73), 36.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
            } else {
                s.draw(COUNTER, (-160, 50), 48.0, 48.0, 1024.0, 0.0, g.focal(), overlay);
            }
            // The arrow over the target (0x80031a28; a battery-sized
            // marker, guess).
            if let Some(k) = self.target {
                let n = &g.npcs[k];
                let at = rrt::glam::Vec3::new(n.pos[0] as f32, (n.pos[1] - 1200) as f32, n.pos[2] as f32);
                s.billboard(COUNTER, at, 150.0, 150.0, g.view.1.row(0), g.view.1.row(1), moving);
            }
        }
        if !self.rules.throw_costs {
            crate::game::shadowed(&g.fonts, -130, 43, format!("{}", self.cookies.max(0)), overlay);
        }
    }

    fn describe(&self) -> String {
        format!("cookies {} me {} target {:?} catcher {:?} ended {}", self.cookies, self.me, self.target, self.catcher, self.ended)
    }
}
