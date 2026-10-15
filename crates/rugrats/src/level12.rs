//! Level 12, Let There Be Light (docs/engine/level-12.md): Tommy with the
//! flashlight in the dark house. Holding Triangle shines it (1 charge a
//! frame); the light sends a Shadow monster away to another spot. Batteries
//! give 30. The refrigerator's cine 67 drops the light and puts Spike
//! somewhere in the house; finding him plays cine 68: won.

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::npc::{Event, Mode, Target};

/// The flashlight (template 933): 120 charge at the start, 100 after a
/// pick-up; 1 a frame.
const LIGHT: u16 = 933;
const CHARGE: i32 = 120;
const CHARGE_AGAIN: i32 = 100;
const GAUGE: (i16, i16) = (-172, 73);
/// The beam's reach (0x8009134c, read from code) and its half width
/// (guess).
const REACH: f32 = 4096.0;
const WIDTH: f32 = 700.0;
/// Shadow monsters (3333-3340, template 934): guard 10000, reach 1024,
/// bite 20; light-struck, away for 18 frames, then a group 10 spot.
const SHADOWS: [u16; 8] = [3333, 3334, 3335, 3336, 3337, 3338, 3339, 3340];
const SHADOW_REACH: i32 = 1024;
const AWAY: i32 = 18;
const SPOTS: usize = 10;
/// Batteries (kind 12): taken within 1000 by the trigger pass, +30.
const BATTERY: u16 = 12;
const BATTERY_REACH: f32 = 1000.0;
/// The refrigerator doors (cine 67), Spike (cine 68).
const DOORS: [u16; 2] = [601, 602];
const DOOR_REACH: i64 = 1500;
const SPIKE: u16 = 14;
const SPIKE_REACH: i64 = 1200;
const CINE_FRIDGE: usize = 67;
const CINE_WON: usize = 68;
/// Where cine 66 leaves Tommy.
const START: [i32; 3] = [29611, -5365, 15428];

pub struct Level12 {
    charge: i32,
    max: i32,
    /// Monsters gone and frames until they come back.
    away: Vec<(usize, i32)>,
    fridge_done: bool,
    spike_waiting: bool,
    shining: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level12 {
    /// Cine 66's end and EpisodeStart (0x8007b7c8).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Level12, String> {
        g.hide(SPIKE, true);
        g.player.pos = START;
        g.player.heading = 1024;
        g.player.target = 1024;
        g.set_camera_yaw(1024.0);
        if let Some(k) = g.things.iter().position(|t| t.template == LIGHT) {
            g.give(k);
        }
        for id in SHADOWS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Guard;
                n.reach = SHADOW_REACH;
                n.bite_wait = 230;
            }
        }
        Ok(Level12 { charge: CHARGE, max: CHARGE, away: Vec::new(), fridge_done: false, spike_waiting: false, shining: false })
    }

    fn holding(&self, g: &Game) -> bool {
        g.carried().is_some_and(|k| g.things[k].template == LIGHT)
    }

    /// The light (beam type 1): a monster near the line ahead of the hand
    /// is struck (0x800511a8 case 41): sound 110, away.
    fn shine(&mut self, g: &mut Game) {
        let p = g.player.pos;
        let (s, c) = (g.trig.sin(g.player.heading) as f32 / 4096.0, g.trig.cos(g.player.heading) as f32 / 4096.0);
        for k in 0..g.npcs.len() {
            let n = &g.npcs[k];
            if !SHADOWS.contains(&n.id) || n.hidden {
                continue;
            }
            let (dx, dz) = ((n.pos[0] - p[0]) as f32, (n.pos[2] - p[2]) as f32);
            let along = dx * s + dz * c;
            let across = (dx * c - dz * s).abs();
            if along > 0.0 && along < REACH && across < WIDTH {
                g.npcs[k].hidden = true;
                g.npcs[k].mode = Mode::Idle;
                g.effects.push((110, crate::sound::EFFECTS_VOLUME));
                self.away.push((k, AWAY));
            }
        }
    }
}

impl crate::game::Script for Level12 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e {
                if amount > 0 {
                    g.player.health = (g.player.health - amount).max(0);
                }
            }
        }
        // Shining (0x8005a6f4): Triangle held, 1 a frame, sound 161 on the
        // press.
        let holding = self.holding(g);
        self.shining = holding && pads.held(0, input::TRIANGLE) && self.charge > 0;
        if holding && pads.pressed(0, input::TRIANGLE) {
            g.effects.push((161, crate::sound::EFFECTS_VOLUME));
        }
        if self.shining {
            self.charge -= 1;
            self.shine(g);
        }
        // Monsters back (0x80083be0): a group 10 spot, not marked used.
        let mut back = Vec::new();
        self.away.retain_mut(|(k, left)| {
            *left -= 1;
            if *left <= 0 {
                back.push(*k);
                false
            } else {
                true
            }
        });
        for k in back {
            g.effects.push((143, crate::sound::EFFECTS_VOLUME));
            if let Some(i) = crate::game::spawn(&mut g.spots, SPOTS, &mut g.rand) {
                g.spots[i].used = false;
                let at = g.spots[i].pos;
                let n = &mut g.npcs[k];
                n.place(at, &g.col);
                n.home = at;
            }
            g.npcs[k].hidden = false;
            g.npcs[k].mode = Mode::Guard;
        }
        // Batteries within 1000 (0x8004cf58): +30, clamped to the gauge.
        let me = rrt::glam::Vec3::new(g.player.pos[0] as f32, g.player.pos[1] as f32, g.player.pos[2] as f32);
        for b in g.goodies.iter_mut().filter(|b| b.kind == BATTERY && !b.taken) {
            if (b.pos - me).length() < BATTERY_REACH {
                b.taken = true;
                if holding {
                    self.charge = (self.charge + 30).min(self.max);
                }
                g.effects.push((109, crate::sound::EFFECTS_VOLUME));
            }
        }
        // Picked up again (0x800602d8): the gauge's max is 100.
        if holding && self.max == CHARGE && self.fridge_done {
            self.max = CHARGE_AGAIN;
        }
        // Triangle by the refrigerator (cine 67) or Spike (cine 68).
        if pads.pressed(0, input::TRIANGLE) {
            let p = g.player.pos;
            let near = |at: [i32; 3], r: i64| {
                let (dx, dz) = ((at[0] - p[0]) as i64, (at[2] - p[2]) as i64);
                dx * dx + dz * dz < r * r
            };
            if !self.fridge_done && g.set.placements.iter().any(|q| q.kind == 1 && DOORS.contains(&q.id) && near(q.pos, DOOR_REACH)) {
                self.fridge_done = true;
                g.drop_carried();
                g.cine = Some(CINE_FRIDGE);
            } else if self.spike_waiting && find(g, SPIKE).is_some_and(|k| near(g.npcs[k].pos, SPIKE_REACH)) {
                self.spike_waiting = false;
                g.drop_carried();
                g.cine = Some(CINE_WON);
            }
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_FRIDGE {
            // 0x8007bb84 after card 705: Spike idle at a group 0 spot.
            if let Some(k) = find(g, SPIKE) {
                if let Some(i) = crate::game::spawn(&mut g.spots, 0, &mut g.rand) {
                    let at = g.spots[i].pos;
                    g.npcs[k].place(at, &g.col);
                }
                g.hide(SPIKE, false);
                g.npcs[k].mode = Mode::Idle;
                self.spike_waiting = true;
            }
        }
    }

    fn draw(&self, g: &Game, _moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        // The gauge (0x8008c7f0: 360 over 361) while the light is held.
        if self.holding(g) {
            if let Some(s) = &g.sprites {
                let top = 2.0 * 54.0 * (self.max - self.charge.min(self.max)).max(0) as f32 / self.max as f32;
                s.draw(361, GAUGE, 36.0, 54.0, 1024.0, top, g.focal(), overlay);
                s.draw(360, GAUGE, 36.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
            }
        }
    }

    fn describe(&self) -> String {
        format!("charge {} away {} fridge {} spike {} shining {}", self.charge, self.away.len(), self.fridge_done, self.spike_waiting, self.shining)
    }
}
