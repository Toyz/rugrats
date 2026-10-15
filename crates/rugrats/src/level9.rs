//! Level 9, The Mysterious Mr. Friend (docs/engine/level-09.md): Mr.
//! Friend chases Tommy and bites for 15. Thrown balls and cans aim at him;
//! Triangle beside him kicks (50). Each death counts down from 4: the first
//! plays cine 18 and brings him back with two more; at 1 one try is left;
//! at 0 cine 19 wins.

use crate::disc::GameDisc;
use crate::game::Game;
use crate::input::{self, Pads};
use crate::npc::{Event, Mode, Target};

/// The Mr. Friends (char 16): 46 first, 91 and 800 after cine 18.
const FRIENDS: [u16; 3] = [46, 91, 800];
/// Hidden by cine 17's start (0x80072ef0 case 17).
const PROPS: [u16; 8] = [800, 91, 94, 93, 802, 97, 804, 95];
const START: [i32; 3] = [30113, 7947, -11549];
const FRIEND_START: [i32; 3] = [29306, 7694, -11911];
/// His bite (+0x6d of char 16) and Tommy's kick (+0x6d 50).
const BITE: i32 = 15;
const KICK: i32 = 50;
/// Touching him: he stops within range / 2 (1000) to bite.
const KICK_REACH: i64 = 1100;
/// Cines: the first death (18), the last (19).
const CINE_MORE: usize = 18;
const CINE_WON: usize = 19;

pub struct Level9 {
    pub phase: u8,
    /// 0x800c4cd8: deaths to go.
    counter: i32,
    /// Level 14 (docs/engine/level-14.md, a cut level the retail game
    /// never reaches): one Mr. Friend, no aim, no sounds, every hit
    /// staggers him, his death ends nothing.
    l14: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl Level9 {
    /// EpisodeStart (0x8007b078).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Level9, String> {
        for id in PROPS {
            g.hide(id, true);
        }
        for id in FRIENDS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.damage = BITE;
                n.attacks = true;
                n.bite_wait = 180;
            }
        }
        // Where cine 17 leaves them (oracle).
        g.player.pos = START;
        if let Some(k) = find(g, 46) {
            g.npcs[k].mode = Mode::Follow(Target::Player);
            g.npcs[k].place(FRIEND_START, &g.col);
        }
        g.aim = true;
        g.checkpoint = Some((g.player.pos, g.player.heading));
        Ok(Level9 { phase: 1, counter: 4, l14: false })
    }

    /// Level 14's EpisodeStart (0x8007b17c): Mr. Friend follows, the
    /// counter 1, no aim.
    pub fn start14(g: &mut Game, _disc: &GameDisc) -> Result<Level9, String> {
        if let Some(k) = find(g, 46) {
            let n = &mut g.npcs[k];
            n.damage = BITE;
            n.attacks = true;
            n.bite_wait = 180;
            n.mode = Mode::Follow(Target::Player);
        }
        g.checkpoint = Some((g.player.pos, g.player.heading));
        Ok(Level9 { phase: 1, counter: 1, l14: true })
    }

    /// Mode 1 (0x800841cc): staggered, its callbacks off, 300 + (rand & 3)
    /// x 300 dt after the hit animation.
    fn hit(&mut self, g: &mut Game, k: usize, amount: i32) {
        let wait = 300 + (g.rand.next() & 3) as i32 * 300;
        let l14 = self.l14;
        let n = &mut g.npcs[k];
        if (matches!(n.mode, Mode::Hit { .. }) && !l14) || n.hidden {
            return;
        }
        n.health = (n.health - amount).max(0);
        if !l14 {
            g.effects.push((19, crate::sound::EFFECTS_VOLUME));
        }
        if n.health == 0 {
            self.die(g, k);
        } else {
            n.mode = Mode::Hit { left: 600 + wait, then: Box::new(Mode::Follow(Target::Player)) };
        }
    }

    /// Mode 10, 0x80083be0: sound 100, gone, the count down.
    fn die(&mut self, g: &mut Game, k: usize) {
        let id = g.npcs[k].id;
        g.npcs[k].mode = Mode::Idle;
        g.hide(id, true);
        self.counter = (self.counter - 1).max(0);
        if self.l14 {
            return;
        }
        g.effects.push((100, crate::sound::EFFECTS_VOLUME));
        match self.counter {
            3 => g.cine = Some(CINE_MORE),
            1 => {
                // 0x8007cc00: phase 3, one try.
                self.phase = 3;
                g.checkpoint = Some((g.player.pos, g.player.heading));
                g.tries = 1;
            }
            0 => g.cine = Some(CINE_WON),
            _ => {}
        }
    }
}

impl crate::game::Script for Level9 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e {
                if amount > 0 {
                    g.player.health = (g.player.health - amount).max(0);
                }
            }
        }
        // Things thrown at him (0x800834b0 and Damage with the thing's
        // +0x6d).
        for (k, _, amount) in std::mem::take(&mut g.struck) {
            if FRIENDS.contains(&g.npcs[k].id) && amount > 0 {
                self.hit(g, k, amount);
            }
        }
        // The kick (state 12, 0x800833a4): Triangle with him touching.
        if pads.pressed(0, input::TRIANGLE) && g.carried().is_none() && g.player.action.is_none() {
            let p = g.player.pos;
            let near = g.npcs.iter().position(|n| {
                let (dx, dz) = ((n.pos[0] - p[0]) as i64, (n.pos[2] - p[2]) as i64);
                FRIENDS.contains(&n.id) && !n.hidden && dx * dx + dz * dz < KICK_REACH * KICK_REACH
            });
            if let Some(k) = near {
                let a = if self.l14 || g.rand.next() & 1 == 0 { 50 } else { 37 };
                g.player.act(a, &g.anims);
                if !self.l14 {
                    g.effects.push((98, crate::sound::EFFECTS_VOLUME));
                }
                self.hit(g, k, KICK);
            }
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_MORE {
            // 0x8007cac4 after card 710: all three home, full health, after
            // Tommy; phase 2; song 0.
            for (i, id) in FRIENDS.into_iter().enumerate() {
                let speed = (g.rand.next() & 2 | 1) as usize;
                if let Some(k) = find(g, id) {
                    g.hide(id, false);
                    let n = &mut g.npcs[k];
                    n.health = n.max_health;
                    let home = n.home;
                    n.place(home, &g.col);
                    n.mode = Mode::Follow(Target::Player);
                    n.damage = BITE;
                    n.attacks = true;
                    if i > 0 {
                        n.speed_index = speed;
                    }
                }
            }
            self.phase = 2;
            g.checkpoint = Some((g.player.pos, g.player.heading));
            g.song = Some(0);
        }
    }

    fn phase(&self) -> u8 {
        self.phase
    }

    fn describe(&self) -> String {
        format!("phase {} counter {}", self.phase, self.counter)
    }
}
