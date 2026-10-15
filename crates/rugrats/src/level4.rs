//! Level 4, 7 Voyages of Cynthia (docs/engine/level-04.md): Spike walks
//! the sewers to the garbage dump. Rooms 26 and 21 move the respawn on
//! (phases 2, 3); from phase 2 the brown water hurts. Room 40 starts the
//! 130 s search (phase 4); Cynthia waits at one of six spots, and coming
//! within 2000 of her plays cine 48: won. The clock running out is a cry,
//! the clock again and Cynthia elsewhere.

use rrt::glam::{Mat3, Vec3};

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::Pads;
use crate::npc::{Event, Mode, Target};

/// Cynthia (id 34), the rats (char 39).
const CYNTHIA: u16 = 34;
const RATS: [u16; 4] = [444, 445, 447, 448];
/// The rooms that move the phase on (0x8007f498 case 1).
const SEWER: usize = 26;
const WATER: usize = 21;
/// Cynthia's spawn group and the reach that wins (0x80047e30: 2000).
const HIDING: usize = 0;
const REACH: i64 = 2000;
const CINE_WON: usize = 48;
/// Script 2 (0x8007e198): the clock, card HINT15, the portrait (sprite
/// 287, half 32 x 64 at depth 1024, at (-60, -70)).
const CLOCK: i32 = 130 * 300;
const CARD: usize = 686;
const PORTRAIT: u32 = 287;
/// The water (COL flag 1): 5 standing, 3 moving, every 36 VSyncs (18
/// frames).
const WATER_FLAG: u8 = 1;
const STAND_HURT: i32 = 5;
const MOVE_HURT: i32 = 3;
const HURT_EVERY: i32 = 180;
/// A rat's bite: +0x6d 3, sound 168.
const BITE: i32 = 3;
const BITE_SOUND: u8 = 168;

pub struct Level4 {
    pub phase: u8,
    respawn: ([i32; 3], i32),
    /// Where Cynthia waits, and whether the win was asked for.
    cynthia: [i32; 3],
    shape: u32,
    found: bool,
    hurt: [i32; 2],
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level4 {
    /// EpisodeStart (0x8007e058).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Level4, String> {
        // Cynthia is drawn by the level at her spot; she is no door
        // (0x80047e30 takes LP +0x42 away).
        let place = g.set.placements.iter().find(|p| p.kind == 1 && p.id == CYNTHIA).copied();
        let shape = place.and_then(|p| g.set.templates.get(p.template as usize)).map_or(0, |t| t.shape as u32);
        g.hide(CYNTHIA, true);
        g.disable_use(CYNTHIA);
        if let Some(s) = g.spots.get_mut(1) {
            s.pos[1] = 100;
        }
        let mut l = Level4 { phase: 1, respawn: (g.player.pos, g.player.heading), cynthia: place.map_or([0; 3], |p| p.pos), shape, found: false, hurt: [0; 2] };
        l.hide_cynthia(g);
        g.checkpoint = Some(l.respawn);
        // Spike's states 10-13 (0x8008a668(6, list, slot, 1)).
        for a in 10..=13 {
            g.player.animator.set_state(a, [902, 903, 901, 900]);
        }
        for id in RATS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Prowl { prev: 0, cur: n.first_post };
                n.damage = BITE;
            }
        }
        Ok(l)
    }

    /// 0x8004fe14(&cynthia, 0, 1): a spot of group 0 not used yet.
    fn hide_cynthia(&mut self, g: &mut Game) {
        if let Some(i) = crate::game::spawn(&mut g.spots, HIDING, &mut g.rand) {
            self.cynthia = g.spots[i].pos;
        }
        rrt::tracing::info!("level 4: Cynthia at {:?}", self.cynthia);
    }

    fn set_phase(&mut self, g: &mut Game, phase: u8) {
        self.phase = phase;
        self.respawn = (g.player.pos, g.player.heading);
        g.checkpoint = Some(self.respawn);
    }
}

impl crate::game::Script for Level4 {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e {
                if amount > 0 {
                    g.player.health = (g.player.health - amount).max(0);
                    g.effects.push((BITE_SOUND, crate::sound::EFFECTS_VOLUME));
                }
            }
        }
        // The brown water (0x8005d8e0, 0x8005eab0): from phase 2, on a
        // water floor and not on an object.
        self.hurt = self.hurt.map(|t| (t - crate::player::DT).max(0));
        if self.phase >= 2 {
            let p = g.player.pos;
            let feet = p[1] - g.player.feet();
            let q = crate::collide::query(&g.col, [p[0], feet - 1, p[2]]);
            let on_floor = q.kind == 0 && (q.floor - feet).abs() < 32;
            if on_floor && crate::collide::floor_flags(&g.col, &q) & WATER_FLAG != 0 {
                let moving = g.player.speed > 0;
                let k = moving as usize;
                if self.hurt[k] == 0 {
                    self.hurt[k] = HURT_EVERY;
                    let d = if moving { MOVE_HURT } else { STAND_HURT };
                    g.player.health = (g.player.health - d).max(0);
                }
            }
        }
        // Cynthia's trigger (0x8004d138): within 2000 (2D).
        let p = g.player.pos;
        let (dx, dz) = ((self.cynthia[0] - p[0]) as i64, (self.cynthia[2] - p[2]) as i64);
        if !self.found && dx * dx + dz * dz < REACH * REACH {
            self.found = true;
            g.timer = None;
            g.cine = Some(CINE_WON);
        }
    }

    fn room(&mut self, g: &mut Game, room: usize) {
        match room {
            SEWER if self.phase < 2 => self.set_phase(g, 2),
            WATER if self.phase < 3 => self.set_phase(g, 3),
            _ => {}
        }
    }

    /// Script 2 (0x8007e198): the clock, the card, phase 4.
    fn gameplay(&mut self, g: &mut Game, n: u8) {
        if n == 2 && g.timer.is_none() {
            g.timer = Some(CLOCK);
            g.card = Some(CARD);
            self.set_phase(g, 4);
        }
    }

    /// TimerExpired (0x8007e2e8): a cry.
    fn timer_expired(&mut self, g: &mut Game) {
        g.cry();
    }

    /// ForceCry's level 4 case: a try; in phase 4 the clock again and
    /// Cynthia elsewhere; the respawn. No tries left: the hub.
    fn cry(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some((crate::game::HUB, None));
            return true;
        }
        if self.phase == 4 {
            g.timer = Some(CLOCK);
            self.hide_cynthia(g);
        }
        let (at, heading) = self.respawn;
        g.player.pos = at;
        g.player.heading = heading;
        g.player.target = heading;
        g.player.speed = 0;
        g.set_camera_yaw(heading as f32);
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        if let Some(m) = g.set.models.get(self.shape).filter(|_| self.shape != 0) {
            let pos = Vec3::new(self.cynthia[0] as f32, self.cynthia[1] as f32, self.cynthia[2] as f32);
            rugrats_render::models::model_triangles(m, Mat3::IDENTITY, pos, moving);
        }
        if self.phase == 4 {
            if let Some(s) = &g.sprites {
                s.draw(PORTRAIT, (-60, -70), 32.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
            }
        }
    }

    fn phase(&self) -> u8 {
        self.phase
    }

    fn describe(&self) -> String {
        format!("phase {} cynthia {:?} found {}", self.phase, self.cynthia, self.found)
    }
}
