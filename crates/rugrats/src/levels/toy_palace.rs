//! Level 6, Toy Palace (docs/levels/level-06.md): Chuckie walks the dark
//! toy store, frightened by scary toys, bitten by clowns and guards, hurt
//! by bursts and breaking toys. Room 11's cine 46 hands over to Tommy, who
//! stacks Reptar boxes to climb to the switch; its cine 47 wins.

use rugrats_data::hierarchy::Hierarchy;

use crate::actors::animator::Animator;
use crate::actors::npc::{Event, Mode, Target};
use crate::actors::player::{Pose, State, TURN};
use crate::disc::GameDisc;
use crate::input::Pads;
use crate::world::game::{Game, Script};

/// Ids: Tommy, Chuckie, the guards and the clowns.
const TOMMY: u16 = 1492;
const CHUCKIE: u16 = 1776;
/// Guards (EpisodeStart 0x800794a0): (id, range, bite, time between
/// bites in dt as seen).
const GUARDS: [(u16, i32, i32, i32); 5] =
    [(46, 5000, 15, 180), (506, 15000, 6, 80), (920, 7000, 8, 230), (921, 7000, 8, 230), (922, 7000, 8, 230)];
const GUARD_REACH: i32 = 1024;
/// Where cine 45 leaves Chuckie.
const START: [i32; 3] = [-4154, -1378, 9795];
const START_HEADING: i32 = 3134;
/// Scary toys (template flag 0x200000): template and damage.
const SCARY: [(u32, i32); 6] = [(1227, 5), (1028, 5), (1029, 5), (1030, 5), (1031, 5), (1155, 2)];
/// Timed bursts (class 10, 0x8004e0f4): template, period (frames), reach.
const BURSTS: [(u32, i32, i64); 2] = [(969, 50, 3072), (982, 25, 2048)];
const BURST_HURT: i32 = 18;
/// Breakable toys (template 1102, flag 1024): sound 173, thrown up 2000,
/// 5.
const BREAKABLE: u32 = 1102;
const ROOM_PHASE2: usize = 2;
const CINE_SWAP: usize = 46;
const TOMMY_AFTER: [i32; 3] = [40738, -1267, 8097];
const CHUCKIE_AFTER: [i32; 3] = [39612, -1378, 7197];

struct Scary {
    /// Its placement (a walker's position moves), where it is.
    placement: usize,
    pos: [i32; 3],
    radius: i64,
    damage: i32,
}

/// A timed burst placed: hurts every `period` frames within `reach`.
#[derive(Clone, Copy, Debug)]
struct Burst {
    at: [i32; 3],
    period: i32,
    reach: i64,
}

pub struct ToyPalace {
    pub phase: u8,
    scary: Vec<Scary>,
    scared: bool,
    bursts: Vec<Burst>,
    clock: i32,
    breakables: Vec<usize>,
    tommy: (Hierarchy, Animator),
    chuckie: (Hierarchy, Animator),
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl ToyPalace {
    /// EpisodeStart (0x800794a0) after cine 45 (whose end hides Tommy).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<ToyPalace, String> {
        g.hide(TOMMY, true);
        g.player.pos = START;
        g.player.heading = START_HEADING;
        g.player.target = START_HEADING;
        g.set_camera_yaw(START_HEADING as f32);
        g.checkpoint = Some(Pose::new(START, START_HEADING));
        for (id, range, bite, wait) in GUARDS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Guard;
                n.range = range;
                n.damage = bite;
                n.bite_wait = wait;
                n.reach = GUARD_REACH;
            }
        }
        let template = |t: u32| g.set.templates.get(t as usize);
        let mut scary = Vec::new();
        let mut bursts = Vec::new();
        let mut breakables = Vec::new();
        for (k, p) in g.set.placements.iter().enumerate().filter(|(_, p)| p.kind == 1) {
            if let Some(&(_, damage)) = SCARY.iter().find(|s| s.0 == p.template) {
                let radius =
                    template(p.template).map_or(3000, |t| u16::from_le_bytes([t.raw[0x34], t.raw[0x35]]) as i64);
                scary.push(Scary { placement: k, pos: p.pos, radius, damage });
            }
            if let Some(&(_, period, reach)) = BURSTS.iter().find(|b| b.0 == p.template) {
                bursts.push(Burst { at: p.pos, period, reach });
            }
            if p.template == BREAKABLE {
                breakables.push(k);
            }
        }
        let body = |c: usize| -> Result<(Hierarchy, Animator), String> {
            let h = g.set.hierarchies.get(c).cloned().flatten().ok_or(format!("no hierarchy {c}"))?;
            Ok((h, Animator::load(disc, c as u16)?))
        };
        let (tommy, chuckie) = (body(1)?, body(2)?);
        Ok(ToyPalace { phase: 1, scary, scared: false, bursts, clock: 0, breakables, tommy, chuckie })
    }

    /// Cine 46's end (0x8007b670, 0x800585b0): the player becomes Tommy at
    /// Tommy's place; Chuckie stays as a character watching him.
    fn swap(&mut self, g: &mut Game) {
        let Some(k) = find(g, TOMMY) else { return };
        // Where cine 46 leaves them (oracle): Tommy in room 9, Chuckie by
        // him.
        let (here, heading) = (CHUCKIE_AFTER, g.player.heading);
        let there = TOMMY_AFTER;
        let (h, a) = self.tommy.clone();
        g.player.set_body(h, a, TURN, &g.anims);
        g.player.pos = there;
        g.character = 1;
        let n = &mut g.npcs[k];
        n.id = CHUCKIE;
        n.character = 2;
        n.hierarchy = self.chuckie.0.clone();
        n.animator = self.chuckie.1.clone();
        n.pos = here;
        n.heading = heading;
        n.hidden = false;
        n.mode = Mode::Idle;
        g.tries = 3;
        self.phase = 3;
        g.checkpoint = Some(Pose::new(there, g.player.heading));
        g.song = Some(1);
        g.set_camera_yaw(g.player.heading as f32);
    }
}

impl Script for ToyPalace {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e
                && amount > 0
            {
                g.player.hurt(amount);
            }
        }
        let p = g.player.pos;
        let d2 = |a: [i32; 3]| {
            let d = [(a[0] - p[0]) as i64, (a[1] - p[1]) as i64, (a[2] - p[2]) as i64];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
        };
        // Scary toys (0x80059570): once per approach; leaving clears it
        // (0x800596b0).
        // The walker 1155 carries its scare along (0x8004dba0).
        for s in &mut self.scary {
            if let Some(w) = g.walkers.iter().find(|w| w.placement == s.placement) {
                s.pos = w.pos;
            }
        }
        match self.scary.iter().find(|s| d2(s.pos) < s.radius * s.radius) {
            Some(s) if !self.scared => {
                self.scared = true;
                g.player.hurt(s.damage);
                g.play(72);
            }
            Some(_) => {}
            None => self.scared = false,
        }
        // Timed bursts: every period, 18 within reach (3D).
        self.clock += 1;
        for &Burst { at, period, reach } in &self.bursts {
            if self.clock % period == 0 && d2(at) <= reach * reach {
                g.player.hurt(BURST_HURT);
                g.play(79);
            }
        }
        // Breaking toys (0x8005ac98): standing on one breaks it.
        let feet = p[1] - g.player.feet();
        let on_ground = matches!(g.player.state, State::Stand | State::Walk | State::Run);
        if on_ground {
            let under = self.breakables.iter().position(|&k| {
                let q = g.set.placements[k];
                let (dx, dz) = ((q.pos[0] - p[0]).abs(), (q.pos[2] - p[2]).abs());
                dx < 900 && dz < 900 && feet < q.pos[1] - 700
            });
            if let Some(i) = under {
                let k = self.breakables.remove(i);
                g.hide_placement(k, true);
                g.play(173);
                let h = g.player.heading;
                g.player.launch(h, -2000, 0, &g.trig);
                g.player.hurt(5);
            }
        }
    }

    fn room(&mut self, g: &mut Game, room: usize) {
        if room == ROOM_PHASE2 {
            if self.phase < 3 {
                self.phase = 2;
            }
            g.checkpoint = Some(g.player.pose());
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_SWAP && self.phase < 3 {
            self.swap(g);
        }
    }

    fn phase(&self) -> u8 {
        self.phase
    }

    fn describe(&self) -> String {
        format!("phase {} scared {}", self.phase, self.scared)
    }
}
