//! Level 20, No More Cookies (docs/levels/level-20.md): the babies keep
//! Angelica's cookie box from her. Circle or Square eats a cookie (15 to
//! go: cine 64 wins). Triangle lobs the box to the baby in front; when it
//! comes near, the player becomes that baby. Angelica runs after the box;
//! her lunge knocks it away: a lost round (cine 65).
//!
//! Level 21, Touchdown Tommy (docs/levels/level-21.md), runs the same
//! code with a bottle: 20 drinks, a throw costs one too, only a drink
//! wins; the round start gives the bottle back to Tommy's body.

use rugrats_render::Vtx;

use crate::actors::npc::{Bite, Event, Mode, Target};
use crate::actors::player::anim::THROW;
use crate::actors::player::{Pose, State};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::levels::Level;
use crate::ui::hud::Look;
use crate::world::game::{self, Exit, Game, Script, ThingState};

/// The box (template 442) and the cookies in it.
const BOX: u16 = 442;
const COOKIES: i32 = 15;
/// The babies one can throw to (templates 2, 6, 7, 11: ids 1776, 609,
/// 608, 1492), and Angelica.
pub(crate) const BABIES: [u16; 4] = [1776, 609, 608, 1492];
const ANGELICA: u16 = 600;
/// The catch: the box within 1400 (x and z) of the target (0x80057518).
const CATCH: i32 = 1400;
/// The counter's sprite (313) at (-160, 50).
const COUNTER: u32 = 313;
/// The target's arrow (0x80031a28, 0x800c4004..0x800c400c): sprite 364
/// (TEXPOS + 0x1110), half-size 500 x 900, 400 over the target's bounds
/// (+0x7a / +0x82: 1773 for every character here, as seen).
const ARROW: u32 = 364;
const ARROW_SIZE: (f32, f32) = (500.0, 900.0);
const ARROW_RISE: i32 = 1773 + 400;
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

const RULES_20: Rules = Rules {
    item: BOX,
    count: COOKIES,
    won: CINE_WON,
    lost: CINE_LOST,
    eat: EAT,
    throw_costs: false,
    back_to_tommy: false,
};
const RULES_21: Rules =
    Rules { item: 259, count: 20, won: 50, lost: 51, eat: 40, throw_costs: true, back_to_tommy: true };

pub struct BoxToss {
    rules: Rules,
    cookies: i32,
    /// The registry id the player's body carries.
    me: u16,
    /// The baby aimed at, and the one a box in flight goes to.
    target: Option<usize>,
    catcher: Option<usize>,
    eat_wait: i32,
    ended: bool,
    /// Knocked (+0x30 bit 64): the box thrown up, then lost the frame
    /// after it is let go (0x8004df94).
    knocked: bool,
    thrown_up: bool,
    /// The round's end lines: lost (level 20 0x800b9220, 5; level 21
    /// 0x800b922c, 12) and won (level 21 0x800b9244, 6).
    lost_lines: Vec<u16>,
    won_lines: Vec<u16>,
}

fn table(disc: &GameDisc, at: u32, n: usize) -> Vec<u16> {
    disc.exe_bytes(at, 2 * n)
        .unwrap_or(&[])
        .as_chunks::<2>()
        .0
        .iter()
        .map(|w| u16::from_le_bytes([w[0], w[1]]))
        .collect()
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

/// The lob at the target (0x80057bc4, kept in player +0x200..+0x20c each
/// frame a target is held): 45 degrees up, (0, -2048, 2048) turned toward
/// the target and made unit, at speed sqrt(s 0x180000 / (2 sin 512 cos
/// 512 >> 12)) with s = 20 (d - 1024) >> 10 for the flat distance d, all
/// in the original's unsigned 32 bits. The flight takes it as a throw's
/// speed: dir * (speed * 65536 / 6000) (0x8008864c).
fn lob(trig: &crate::gte::Trig, from: [i32; 3], to: [i32; 3], up: bool) -> [i32; 3] {
    // (The flat distance by the game's measure, 0x8001f57c.)
    let d = crate::actors::npc::approx_dist(from, to);
    let s = (d - 1024).wrapping_mul(20) as u32 >> 10;
    let p = trig.sin(512).wrapping_mul(trig.cos(512) << 1);
    let a = (if p < 0 { p + 4095 } else { p }) >> 12;
    let speed = ((s << 12).wrapping_add(s << 13) << 7).checked_div(a as u32).map_or(0, |v| v.isqrt()) as i16 as i32;
    // (0, -2048, 2048), or with +0x30 bit 64 (0, -4096, 100), turned toward
    // the target (0x800760f4, 0x80075ee8) and made a unit (0x8007652c).
    let yaw = trig.ratan2(to[0] - from[0], to[2] - from[2]);
    let local = if up { [0, -4096, 100] } else { [0, -2048, 2048] };
    let dir = crate::world::collide::unit(trig.turn_by([yaw as i16, 0, 0], local));
    let k = (speed << 16) / 6000;
    dir.map(|c| (c * k) >> 12)
}

/// The dummy the knock throws at (0x800e5778, its place 0x800e5790..98:
/// set by the round start).
const DUMMY: [i32; 3] = [22086, 775, 12864];
/// The knock's throw (0x8005f3a4: animation 8 at 4096).
const KNOCKED: i16 = 8;

impl BoxToss {
    /// EpisodeStart (0x80079ff0) and the round start (0x80079cb0).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<BoxToss, String> {
        let (lost_lines, won_lines) = (table(disc, 0x800b_9220, 5), Vec::new());
        let mut l = BoxToss {
            rules: RULES_20,
            cookies: COOKIES,
            me: 1492,
            target: None,
            catcher: None,
            eat_wait: 0,
            ended: false,
            knocked: false,
            thrown_up: false,
            lost_lines,
            won_lines,
        };
        l.round(g);
        g.checkpoint = Some(g.player.pose());
        Ok(l)
    }

    /// Level 21's EpisodeStart (0x8007a444) and cine 49's end
    /// (0x8007a314): Lil's new home, the grown-ups hidden, the round.
    pub fn start21(g: &mut Game, disc: &GameDisc) -> Result<BoxToss, String> {
        for id in [13, 15, 16] {
            g.hide(id, true);
        }
        if let Some(k) = find(g, 609) {
            g.npcs[k].home = [10030, 779, 10051];
        }
        let (lost_lines, won_lines) = (table(disc, 0x800b_922c, 12), table(disc, 0x800b_9244, 6));
        let mut l = BoxToss {
            rules: RULES_21,
            cookies: 20,
            me: 1492,
            target: None,
            catcher: None,
            eat_wait: 0,
            ended: false,
            knocked: false,
            thrown_up: false,
            lost_lines,
            won_lines,
        };
        l.round(g);
        g.checkpoint = Some(g.player.pose());
        Ok(l)
    }

    fn round(&mut self, g: &mut Game) {
        (self.knocked, self.thrown_up) = (false, false);
        // Level 21 hands the bottle back to Tommy's body.
        if self.rules.back_to_tommy
            && self.me != 1492
            && let Some(k) = find(g, 1492)
        {
            self.swap(g, k);
        }
        self.cookies = self.rules.count;
        self.catcher = None;
        self.ended = false;
        let item = self.rules.item;
        if let Some(k) = g.things.iter().position(|t| t.template == item) {
            g.things[k].keep = true;
            g.things[k].world_only = true;
            g.give(k);
        }
        for id in BABIES {
            if let Some(k) = find(g, id) {
                let home = g.npcs[k].home;
                g.npcs[k].place(home, &g.col);
                g.npcs[k].mode = Mode::Idle;
                // (+0x688 |= 66: home past half the range, facing the
                // player.)
                g.npcs[k].faces = true;
                g.npcs[k].idle_home = true;
                // Scheduled by EpisodeStart (0x80079cb0, 0x8007a094).
                g.npcs[k].scheduled = true;
            }
        }
        if let Some(k) = find(g, ANGELICA) {
            let home = g.npcs[k].home;
            g.npcs[k].place(home, &g.col);
            g.npcs[k].speed_index = 2;
            g.npcs[k].mode = Mode::Follow(Target::Player);
            g.npcs[k].attacks = false;
            // (+0x688 |= 128: she may lunge; +0x1f4 the player, whom her
            // casts pass.)
            g.npcs[k].lunges = true;
            g.npcs[k].ignores_player = true;
        }
        g.player.gait = (17, 17);
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
        g.forget_pose();
        g.aim_target = None;
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
        g.drop_carried();
        // 0x80079f5c / 0x8007a388: a line, then the cine.
        if !self.lost_lines.is_empty() {
            let k = (g.rand.next() & 255) as usize % self.lost_lines.len();
            g.speech.push(self.lost_lines[k]);
        }
        g.cine = Some(self.rules.lost);
    }
}

impl Script for BoxToss {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        let events = std::mem::take(&mut g.npc_events);
        if self.ended {
            return;
        }
        self.eat_wait = (self.eat_wait - 1).max(0);
        let item = self.rules.item;
        let holding = g.carried().is_some_and(|k| g.things[k].template == item);
        // Eating (0x80057950): Circle or Square, standing.
        let standing = matches!(g.player.state, State::Stand);
        if holding
            && standing
            && self.eat_wait == 0
            && (pads.pressed(0, input::CIRCLE) || pads.pressed(0, input::SQUARE))
        {
            self.cookies -= 1;
            self.eat_wait = EAT_WAIT;
            g.player.act(self.rules.eat, &g.anims);
            // A crunch at the player (0x8006f2fc: program 0, tone 14, 13
            // in level 20, note 43 + rand & 15), else 123 unplaced.
            if self.rules.throw_costs {
                let tone = if g.level == Level::NO_MORE_COOKIES { 13 } else { 14 };
                let (note, at) = (43 + (g.rand.next() & 15) as u8, g.player.pos);
                g.key_at([0, tone, note], at);
            } else {
                g.play(123);
            }
            if self.cookies <= 0 {
                self.ended = true;
                // 0x8007a504 (level 21): a line, then cine 50.
                if !self.won_lines.is_empty() {
                    let k = (g.rand.next() & 255) as usize % self.won_lines.len();
                    g.speech.push(self.won_lines[k]);
                }
                g.cine = Some(self.rules.won);
                return;
            }
        }
        self.target = g.aim_target;
        // The throw (0x80060b44's levels 20-21 branch): a lob to the
        // target.
        // A press while a throw is under way is ignored (+0x188 bit 4).
        if holding
            && g.pending_throw.is_none()
            && pads.pressed(0, input::TRIANGLE)
            && let Some(k) = self.target
        {
            let v = lob(&g.trig, g.player.pos, g.npcs[k].pos, false);
            g.player.act_at(THROW, crate::world::game::THROW_ANIM_SPEED, &g.anims);
            // 0x8005d11c: speech 1159 or 1167.
            g.speech.push(if g.rand.next() & 1 != 0 { 1159 } else { 1167 });
            g.throw_carried(v);
            if self.rules.throw_costs {
                self.cookies = (self.cookies - 1).max(0);
            }
            self.catcher = Some(k);
            g.npcs[k].mode = Mode::Idle;
        }
        // The catch (0x80057518): the box in flight (+0x90) within 1400
        // of the catcher by 0x8001f57c's distance (dx + dz less half the
        // smaller).
        if let Some(k) = self.catcher
            && let Some(b) = g.things.iter().find(|t| t.template == item)
        {
            let q = g.npcs[k].pos;
            let (dx, dz) = ((q[0] - b.pos.x as i32).abs(), (q[2] - b.pos.z as i32).abs());
            let near = (dx + dz - dx.min(dz) / 2) < CATCH + 1;
            if matches!(b.state, ThingState::Flying(_)) && near {
                self.catcher = None;
                self.swap(g, k);
            } else if b.state == ThingState::Rest {
                // A missed throw: the box on the floor.
                self.catcher = None;
            }
        }
        // Angelica: after the box; her lunge knocks it from the holder,
        // and a box on the floor is lost when she lunges.
        if let Some(a) = find(g, ANGELICA) {
            // She follows the box itself (mode 4 with the box as the
            // target), in hand, flying or on the floor.
            // (+0x678 the box: where it is as she steps.)
            let item = self.rules.item;
            if let Some(b) = g.things.iter().position(|t| t.template == item)
                && matches!(g.npcs[a].mode, Mode::Follow(_))
            {
                g.npcs[a].mode = Mode::Follow(Target::Thing(b));
            }
            // Her lunge (state 28): its entry's sound at her (0x8005b2d8);
            // meeting the player who holds the box, the knock (0x8005f3a4);
            // the box resting on the floor while she lunges, lost
            // (0x80057950).
            for e in &events {
                match *e {
                    Event::Lunge(k) if k == a => {
                        let at = g.npcs[a].pos;
                        g.play_at(121, at);
                    }
                    // The knock (0x8005f3a4): +0x30 |= 64; the player
                    // throws the box nearly straight up at the dummy.
                    Event::Knock(k) if k == a && holding && !self.knocked => {
                        self.knocked = true;
                        g.player.act_at(KNOCKED, 4096, &g.anims);
                        let v = lob(&g.trig, g.player.pos, DUMMY, true);
                        g.throw_carried(v);
                    }
                    _ => {}
                }
            }
            // The knocked box let go: its think ends the flight and posts
            // ForceCry the frame after (0x8004df94): the round lost.
            if self.knocked && !holding {
                if self.thrown_up {
                    self.lose(g);
                }
                self.thrown_up = true;
            }
            let resting = g.things.iter().any(|t| t.template == item && t.state == ThingState::Rest);
            if g.npcs[a].bite == Bite::Lunge && !holding && resting && g.carried().is_none() {
                self.lose(g);
            }
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == self.rules.lost {
            if g.tries <= 0 {
                g.won = Some(false);
                g.exit = Some(Exit::to(Level::HUB));
            } else {
                // Card 709, then the round again (0x80079f1c).
                g.card = Some(709);
                if let Some(Pose { pos: at, heading: h }) = g.checkpoint {
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

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        if let Some(s) = &g.sprites {
            if self.rules.throw_costs {
                // Level 21's bottle gauge (0x8008c7f0: 366 over 365, max 20).
                let top = 2.0 * 54.0 * (self.rules.count - self.cookies.max(0)) as f32 / self.rules.count as f32;
                s.draw(365, (-172, 73), 36.0, 54.0, 1024.0, top, g.focal(), overlay);
                s.draw(366, (-172, 73), 36.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
            } else {
                // The counter (0x80079cb0): half 128 by 128 at depth 1024,
                // its count on it.
                s.draw(COUNTER, (-160, 50), 128.0, 128.0, 1024.0, 0.0, g.focal(), overlay);
            }
            // The arrow over the target (0x80031a28): sprite 364 turned
            // half round (2048) to point down, over the target's bounds,
            // bright for 5 frames of every 10.
            if let Some(k) = self.target {
                let n = &g.npcs[k];
                let at = rrt::glam::Vec3::new(n.pos[0] as f32, (n.pos[1] - ARROW_RISE) as f32, n.pos[2] as f32);
                let grey = if g.frames % 10 < 5 { 208 } else { 128 };
                let (right, down) = (-g.view.1.row(0) * ARROW_SIZE.0, -g.view.1.row(1) * ARROW_SIZE.1);
                s.quad(ARROW, at, (right, down), Look { grey, semi: false }, moving);
            }
        }
        if !self.rules.throw_costs {
            game::shadowed(&g.fonts, -160, 50, format!("{}", self.cookies.max(0)), overlay);
        }
    }

    fn describe(&self) -> String {
        format!(
            "cookies {} me {} target {:?} catcher {:?} ended {}",
            self.cookies, self.me, self.target, self.catcher, self.ended
        )
    }
}
