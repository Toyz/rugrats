//! Level 16, training (docs/levels/level-16.md): the house with the
//! training messages. Room 4 is a pie toss at Angelica (Gameplay script
//! 3): five hits in 30 s win, but nothing is awarded and the toss starts
//! again on every entry. No cry, no loss; the door in room 6 goes to the
//! hub with the progress kept.

use rrt::glam::Vec3;

use crate::actors::npc::{Event, Mode, Target};
use crate::actors::player::State;
use crate::disc::GameDisc;
use crate::input::Pads;
use crate::world::game::{Game, Script, Strike, Trigger};

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
/// Templates 111 and 441 think 0x80049eec while their trigger entry has
/// them in (0x8004691c at 0x800469a4: within 2500, then their template's
/// 15000 over it): in the player's room they sparkle, silent.
const SPARKLERS: [u32; 2] = [111, 441];
const CLOCK: i32 = 30 * 300;
/// The end (0x8007e904): Tommy just off the counter's edge.
const AFTER: [i32; 3] = [31000, -1781, -8146];
/// Rooms 4 (the booth) and 5 (the start, holding the counter).
const START_ROOM: usize = 5;
const PIE: u16 = 564;

pub struct Training {
    /// The pie toss under way (+0x30 bit 8).
    toss: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Training {
    /// EpisodeStart (0x80079090): card 711, Angelica idle, song 0.
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Training, String> {
        g.card = Some(START_CARD);
        g.song = Some(0);
        g.tries = 0;
        g.player.slip_sound = 169;
        if let Some(k) = find(g, ANGELICA) {
            g.npcs[k].mode = Mode::Idle;
            // (+0x688 |= 64: she faces the player.)
            g.npcs[k].faces = true;
            // Scheduled by EpisodeStart (0x80079090).
            g.npcs[k].scheduled = true;
        }
        if let Some(k) = find(g, LOBSTER) {
            let n = &mut g.npcs[k];
            n.mode = Mode::Prowl { prev: 0, cur: n.first_post };
            n.damage = BITE;
        }
        Ok(Training { toss: false })
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
            // (+0x688 |= 0x1800.)
            n.passes = true;
        }
        g.player.pos = COUNTER;
        // Standing on the counter (+0x148 with him).
        g.player.floor = COUNTER[1] - g.player.feet();
        g.player.heading = 2048;
        g.player.target = 2048;
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.player.state = State::Stand;
        g.player.frozen = true;
        g.set_room(Some(START_ROOM));
        // The stick turns him camera-relative: the camera looks east.
        let at = Vec3::new(COUNTER[0] as f32, COUNTER[1] as f32, COUNTER[2] as f32);
        g.fixed = Some((at + EYE, at - Vec3::new(0.0, LOOK_UP, 0.0)));
        // The six pies at the pie spot, pie 0 in hand; the speed by
        // Angelica's distance across (0x8001f57c).
        g.start_pie_toss(ANGELICA, true);
        g.training.show(28);
        g.timer = Some(CLOCK);
        self.toss = true;
    }

    /// 0x8007e904: the follow camera back, Tommy off the counter.
    fn end(&mut self, g: &mut Game) {
        self.toss = false;
        g.end_pie_toss();
        g.timer = None;
        g.player.frozen = false;
        g.fixed = None;
        g.player.pos = AFTER;
        g.player.floor = AFTER[1] - g.player.feet();
        g.player.speed = 0;
        g.player.v = [0; 3];
        g.player.state = State::Stand;
        g.set_room(Some(START_ROOM));
        let h = g.player.heading;
        g.set_camera_yaw(h as f32);
    }

    /// A pie that struck Angelica (Damage 50, 0x800780ac from the flight;
    /// 0x8007eafc): sound 44, at 0 her health back and the toss won.
    fn toss_frame(&mut self, g: &mut Game) {
        if !self.toss {
            return;
        }
        // (The pies' strikes are the toss's; the rest the level's own.)
        let (pies, rest): (Vec<Strike>, Vec<Strike>) =
            std::mem::take(&mut g.struck).into_iter().partition(|h| g.things.get(h.thing).is_some_and(|t| t.template == PIE));
        g.struck = rest;
        let hits: Vec<usize> = pies.iter().map(|h| h.npc).collect();
        for k in hits {
            if g.npcs[k].id != ANGELICA {
                continue;
            }
            let at = g.npcs[k].pos;
            g.play_at(44, at);
            let n = &mut g.npcs[k];
            n.hurt(PIE_DAMAGE);
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

impl Script for Training {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        let room = g.current_room();
        let sparkling: Vec<([i32; 3], crate::world::flights::Matrix, i16)> = g
            .set
            .placements
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind == 1 && SPARKLERS.contains(&p.template) && !g.is_hidden(p.id))
            .filter(|&(k, p)| g.trigger_active(Trigger::Placement(k)) == Some(true) && room.is_some() && g.room_of(p.pos) == room)
            .map(|(k, p)| (p.pos, g.placed_matrix(k), g.set.templates.get(p.template as usize).map_or(0, |t| t.raw[0x32] as i16)))
            .collect();
        for (at, frame, damage) in sparkling {
            g.flights.sign_sparks(at, &frame, damage, g.sprites.as_ref());
        }
        for e in std::mem::take(&mut g.npc_events) {
            // Angelica waiting at a post (0x80083e2c, level 16: 0x800b96e4).
            if let Event::Waits(_) = e {
                g.speech.push([643, 646, 582][(g.rand.next() & 255) as usize % 3]);
            }
            if let Event::Bite { by, target: Target::Player, amount } = e
                && amount > 0
            {
                g.post_damage(amount);
                // At the biter (0x80084084: 0x8006f644(52, it)).
                let at = g.npcs[by].pos;
                g.play_at(52, at);
                if g.npcs[by].id == LOBSTER {
                    g.training.show(13);
                }
            }
        }
        // The toss's pies first (their strikes are its own).
        self.toss_frame(g);
        // A thrown can kills the lobster (Damage 50 on 50).
        for Strike { npc: k, damage: amount, .. } in std::mem::take(&mut g.struck) {
            if g.npcs[k].id != LOBSTER {
                continue;
            }
            let at = g.npcs[k].pos;
            g.play_at(44, at);
            let n = &mut g.npcs[k];
            n.hurt(amount);
            if n.health == 0 {
                n.mode = Mode::Idle;
                g.hide(LOBSTER, true);
            }
        }
    }

    fn gameplay(&mut self, g: &mut Game, n: u8) {
        if n == TOSS_SCRIPT {
            self.toss(g);
        }
    }

    /// 0x8007ea10: the clock out; Angelica idle.
    fn timer_expired(&mut self, g: &mut Game) {
        if self.toss {
            if let Some(k) = find(g, ANGELICA) {
                g.npcs[k].mode = Mode::Idle;
                g.npcs[k].faces = true;
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

    fn describe(&self) -> String {
        format!("toss {}", self.toss)
    }
}
