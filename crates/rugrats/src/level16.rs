//! Level 16, training (docs/levels/level-16.md): the house with the
//! training messages. Room 4 is a pie toss at Angelica (Gameplay script
//! 3): five hits in 30 s win, but nothing is awarded and the toss starts
//! again on every entry. No cry, no loss; the door in room 6 goes to the
//! hub with the progress kept.

use rrt::glam::Vec3;

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::level3::{Pie, draw_pies, roll_pies};
use crate::npc::{Event, Mode, Target};

const ANGELICA: u16 = 600;
/// The lobster (template 495, id 0): posts 10-19, bites 15.
const LOBSTER: u16 = 0;
const BITE: i32 = 15;
/// EpisodeStart's card (HINT40) and song.
const START_CARD: usize = 711;
/// The toss (0x8007e680): Tommy on the counter facing 2048, the camera
/// 4000 west and 2240 up looking at him 1570 up, Angelica's health, the
/// clock.
const TOSS_SCRIPT: u8 = 3;
const COUNTER: [i32; 3] = [31189, -1750, -8141];
const EYE: Vec3 = Vec3::new(-4000.0, -2240.0, 0.0);
const LOOK_UP: f32 = 1570.0;
const TOSS_HEALTH: i32 = 250;
const PIE_DAMAGE: i32 = 50;
const CLOCK: i32 = 30 * 300;
/// The end (0x8007e904): Tommy just off the counter's edge.
const AFTER: [i32; 3] = [31000, -1781, -8146];
/// Rooms 4 (the booth) and 5 (the start, holding the counter).
const START_ROOM: usize = 5;
const PIE: u16 = 564;

struct Toss {
    pies: Vec<Pie>,
}

pub struct Level16 {
    toss: Option<Toss>,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level16 {
    /// EpisodeStart (0x80079090): card 711, Angelica idle, song 0.
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Level16, String> {
        g.card = Some(START_CARD);
        g.song = Some(0);
        g.tries = 0;
        g.player.slip_sound = 169;
        if let Some(k) = find(g, ANGELICA) {
            g.npcs[k].mode = Mode::Idle;
        }
        if let Some(k) = find(g, LOBSTER) {
            let n = &mut g.npcs[k];
            n.mode = Mode::Prowl { prev: 0, cur: n.first_post };
            n.damage = BITE;
        }
        Ok(Level16 { toss: None })
    }

    /// 0x8007e680: Angelica on her posts with 250, Tommy on the counter
    /// with a pie, message 28, 30 s.
    fn toss(&mut self, g: &mut Game) {
        if let Some(k) = find(g, ANGELICA) {
            let n = &mut g.npcs[k];
            n.health = TOSS_HEALTH;
            n.max_health = TOSS_HEALTH;
            let home = n.home;
            n.place(home, &g.col);
            n.mode = Mode::Posts { prev: 0, cur: n.first_post };
        }
        g.player.pos = COUNTER;
        g.player.heading = 2048;
        g.player.target = 2048;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.player.state = crate::player::State::Stand;
        g.player.frozen = true;
        g.set_room(Some(START_ROOM));
        // The stick turns him camera-relative: the camera looks east.
        g.yaw = 1024.0;
        let at = Vec3::new(COUNTER[0] as f32, COUNTER[1] as f32, COUNTER[2] as f32);
        g.fixed = Some((at + EYE, at - Vec3::new(0.0, LOOK_UP, 0.0)));
        // The counter's pie is the pool's: the toss draws its own, and
        // the pies end up thrown (guess: none back on the counter).
        if let Some(k) = g.carried() {
            if g.things[k].template == PIE {
                g.drop_carried();
            }
        }
        for t in g.things.iter_mut().filter(|t| t.template == PIE) {
            t.hidden = true;
        }
        g.training.show(28);
        g.timer = Some(CLOCK);
        self.toss = Some(Toss { pies: Vec::new() });
    }

    /// 0x8007e904: the follow camera back, Tommy off the counter.
    fn end(&mut self, g: &mut Game) {
        self.toss = None;
        g.timer = None;
        g.player.frozen = false;
        g.fixed = None;
        g.player.pos = AFTER;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.player.state = crate::player::State::Stand;
        g.set_room(Some(START_ROOM));
        let h = g.player.heading;
        g.set_camera_yaw(h as f32);
    }

    fn toss_frame(&mut self, g: &mut Game, pads: &Pads) {
        let Some(toss) = &mut self.toss else { return };
        // The next pie is in hand at once (0x8004eb10); message 27.
        let ready = g.player.action.is_none();
        if ready {
            g.training.show(27);
        }
        if pads.pressed(0, input::TRIANGLE) && ready {
            let aim = find(g, ANGELICA).map(|k| g.npcs[k].pos);
            toss.pies.push(Pie::thrown(g, g.player.pos, g.player.heading, aim));
            g.player.act(crate::player::anim::THROW, &g.anims);
        }
        let targets: Vec<usize> = find(g, ANGELICA).into_iter().collect();
        let hits = roll_pies(g, &mut toss.pies, &targets);
        // A hit (0x8007eafc): 50 off; at 0 her health back and won.
        for _ in hits {
            let Some(k) = find(g, ANGELICA) else { break };
            g.effects.push((44, crate::sound::EFFECTS_VOLUME));
            let r = g.rand.next() & 1;
            g.speech.extend(crate::game::hit_line(3, 16, r));
            let n = &mut g.npcs[k];
            n.health = (n.health - PIE_DAMAGE).max(0);
            if n.health == 0 {
                n.health = n.max_health;
                n.mode = Mode::Idle;
                rrt::tracing::info!("level 16: the pie toss won");
                self.end(g);
                return;
            }
        }
    }
}

impl crate::game::Script for Level16 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            // Angelica waiting at a post (0x80083e2c, level 16: 0x800b96e4).
            if let Event::Waits(_) = e {
                g.speech.push([643, 646, 582][(g.rand.next() & 255) as usize % 3]);
            }
            if let Event::Bite { by, target: Target::Player, amount } = e {
                if amount > 0 {
                    g.player.health = (g.player.health - amount).max(0);
                    g.effects.push((52, crate::sound::EFFECTS_VOLUME));
                    if g.npcs[by].id == LOBSTER {
                        g.training.show(13);
                    }
                }
            }
        }
        // A thrown can kills the lobster (Damage 50 on 50).
        for (k, _, amount) in std::mem::take(&mut g.struck) {
            if g.npcs[k].id != LOBSTER {
                continue;
            }
            g.effects.push((44, crate::sound::EFFECTS_VOLUME));
            let n = &mut g.npcs[k];
            n.health = (n.health - amount).max(0);
            if n.health == 0 {
                n.mode = Mode::Idle;
                g.hide(LOBSTER, true);
            }
        }
        self.toss_frame(g, pads);
    }

    fn gameplay(&mut self, g: &mut Game, n: u8) {
        if n == TOSS_SCRIPT {
            self.toss(g);
        }
    }

    /// 0x8007ea10: the clock out; Angelica idle.
    fn timer_expired(&mut self, g: &mut Game) {
        if self.toss.is_some() {
            if let Some(k) = find(g, ANGELICA) {
                g.npcs[k].mode = Mode::Idle;
            }
            self.end(g);
        }
    }

    /// ForceCry's level 16 case: no cry, health 255, nothing else.
    fn cry_animation(&self) -> bool {
        false
    }

    fn cry(&mut self, _g: &mut Game) -> bool {
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<rugrats_render::Vtx>, _overlay: &mut Vec<rugrats_render::Vtx>) {
        if let Some(toss) = &self.toss {
            draw_pies(g, &toss.pies, g.player.action.is_none(), moving);
        }
    }

    fn describe(&self) -> String {
        let pies = self.toss.as_ref().map(|t| t.pies.len());
        format!("toss {:?}", pies)
    }
}
