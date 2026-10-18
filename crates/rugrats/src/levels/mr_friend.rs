//! Level 9, The Mysterious Mr. Friend (docs/levels/level-09.md): Mr.
//! Friend chases Tommy and bites for 15. Thrown balls and cans aim at him;
//! Triangle beside him kicks (50). Each death counts down from 4: the first
//! plays cine 18 and brings him back with two more; at 1 one try is left;
//! at 0 cine 19 wins.

use crate::actors::npc::{Bite, Event, Mode, Target};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::world::game::{Effect, Game, Script, Strike, Touched};

/// The Mr. Friends (char 16): 46 first, 91 and 800 after cine 18.
const FRIENDS: [u16; 3] = [46, 91, 800];
/// Hidden by cine 17's start (0x80072ef0 case 17).
const PROPS: [u16; 8] = [800, 91, 94, 93, 802, 97, 804, 95];
const START: [i32; 3] = [30113, 7947, -11549];
const FRIEND_START: [i32; 3] = [29306, 7694, -11911];
/// His bite (+0x6d of char 16) and Tommy's kick (+0x6d 50).
const BITE: i32 = 15;
const KICK: i32 = 50;
/// Cines: the first death (18), the last (19).
const CINE_MORE: usize = 18;
const CINE_WON: usize = 19;

/// The level's progress (0x800c4cd0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    /// The first Mr. Friend, until his death and cine 18.
    First = 1,
    /// After cine 18: all three after Tommy.
    AllThree = 2,
    /// One death to go: one try left.
    OneLeft = 3,
}

pub struct MrFriend {
    phase: Phase,
    /// 0x800c4cd8: deaths to go.
    counter: i32,
    /// Level 14 (docs/levels/level-14.md, a cut level the retail game
    /// never reaches): one Mr. Friend, no aim, no sounds, every hit
    /// staggers him, his death ends nothing.
    l14: bool,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl MrFriend {
    /// EpisodeStart (0x8007b078).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<MrFriend, String> {
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
            // Scheduled by EpisodeStart (0x8007b078).
            g.npcs[k].scheduled = true;
            g.npcs[k].place(FRIEND_START, &g.col);
        }
        g.aim = true;
        g.checkpoint = Some(g.player.pose());
        Ok(MrFriend { phase: Phase::First, counter: 4, l14: false })
    }

    /// Level 14's EpisodeStart (0x8007b17c): Mr. Friend follows, the
    /// counter 1, no aim.
    pub fn start14(g: &mut Game, _disc: &GameDisc) -> Result<MrFriend, String> {
        if let Some(k) = find(g, 46) {
            let n = &mut g.npcs[k];
            n.damage = BITE;
            n.attacks = true;
            n.bite_wait = 180;
            n.mode = Mode::Follow(Target::Player);
            // (0x8007b17c.)
            n.scheduled = true;
        }
        g.checkpoint = Some(g.player.pose());
        Ok(MrFriend { phase: Phase::First, counter: 1, l14: true })
    }

    /// A hit (his callbacks 0x800833a4 / 0x800834b0): his line, the
    /// damage, the stagger (logical 8 at 2048) and mode 1, his callbacks
    /// off until it ends (`Npc::staggered`); mode 1's wait comes at the
    /// stagger's event 0x2000 (`npc::step`).
    fn hit(&mut self, g: &mut Game, k: usize, amount: i32) {
        let l14 = self.l14;
        if (g.npcs[k].staggered && !l14) || g.npcs[k].hidden {
            return;
        }
        if !l14 {
            g.effects.push(Effect::new(19));
            // His line (0x800833a4 / 0x800834b0 in level 9: 0x800b96c8).
            g.speech.push([153, 187, 146][(g.rand.next() & 255) as usize % 3]);
        }
        let anims = &g.anims;
        let n = &mut g.npcs[k];
        n.hurt(amount);
        // Mode 1 ends in state 0: a hit wakes a waiting 46 too.
        n.rooted = false;
        if n.health == 0 {
            self.die(g, k);
        } else {
            n.mode = Mode::Follow(Target::Player);
            n.bite = Bite::Recover;
            n.staggered = true;
            n.animator.start_at(8, 2048, anims);
        }
    }

    /// Mode 10, 0x80083be0: sound 100, gone, the count down.
    fn die(&mut self, g: &mut Game, k: usize) {
        let (id, at) = (g.npcs[k].id, g.npcs[k].pos);
        g.npcs[k].mode = Mode::Idle;
        g.hide(id, true);
        self.counter = (self.counter - 1).max(0);
        if self.l14 {
            return;
        }
        // At him (0x80084084: 0x8006f644(100, it) in level 9).
        g.play_at(100, at);
        match self.counter {
            3 => g.cine = Some(CINE_MORE),
            1 => {
                // 0x8007cc00: phase 3, one try.
                self.phase = Phase::OneLeft;
                g.checkpoint = Some(g.player.pose());
                g.tries = 1;
            }
            0 => g.cine = Some(CINE_WON),
            _ => {}
        }
    }
}

impl Script for MrFriend {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e
                && amount > 0
            {
                g.post_damage(amount);
            }
        }
        // Things thrown at him: their touch (0x800834b0) staggers him, and
        // the Damage posted with the thing's +0x6d (0x800780ac) takes his
        // health down at the next frame's start (0x80077864).
        for Strike { npc: k, damage: amount, .. } in std::mem::take(&mut g.struck) {
            if FRIENDS.contains(&g.npcs[k].id) && amount > 0 {
                self.hit(g, k, 0);
                g.post_npc_damage(k, amount);
            }
        }
        // Mode 1 with no health left (0x800841cc): mode 10, his end.
        for k in 0..g.npcs.len() {
            let n = &g.npcs[k];
            if FRIENDS.contains(&n.id) && !n.hidden && n.health == 0 && matches!(n.mode, Mode::Follow(_)) {
                self.die(g, k);
            }
        }
        // The kick (state 12, 0x800833a4): Triangle with him touching.
        if pads.pressed(0, input::TRIANGLE) && g.carried().is_none() && g.player.action.is_none() {
            // (The touched object, 0x800631d8: him.)
            let near = match g.touched_any() {
                Some(Touched::Npc(k)) if FRIENDS.contains(&g.npcs[k].id) => Some(k),
                _ => None,
            };
            if let Some(k) = near {
                let a = if self.l14 || g.rand.next() & 1 == 0 { 50 } else { 37 };
                g.player.act(a, &g.anims);
                if !self.l14 {
                    // At him (0x800833a4: 0x8006f644(98, it)).
                    let at = g.npcs[k].pos;
                    g.play_at(98, at);
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
                    // Scheduled by 0x8007cac4.
                    n.scheduled = true;
                    n.damage = BITE;
                    n.attacks = true;
                    if i > 0 {
                        n.speed_index = speed;
                    } else {
                        // 46 died in state 19 and 0x80083728 does not reset
                        // it: home, he waits there until Tommy comes within
                        // a bite (oracle, 4222 to 6400).
                        n.rooted = true;
                    }
                }
            }
            self.phase = Phase::AllThree;
            g.checkpoint = Some(g.player.pose());
            g.song = Some(0);
        }
    }

    /// The deaths to go (0x800c4cd8).
    fn cue(&self) -> i32 {
        self.counter
    }

    fn phase(&self) -> u8 {
        self.phase as u8
    }

    fn describe(&self) -> String {
        format!("phase {:?} counter {}", self.phase, self.counter)
    }
}
