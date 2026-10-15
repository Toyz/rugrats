//! Level 17, Gold Rush (docs/levels/level-17.md): Phil picks up all 62
//! nickels before 170 s run out; the gold coin is worth a fourth bar.
//! All 62 end the clock at once and cine 26 wins. The clock out loses a
//! try: Phil's round ends with cine 27 and Lil plays the second; her
//! clock out is a cry and the hub.

use crate::actors::npc::Mode;
use crate::disc::GameDisc;
use crate::input::Pads;
use crate::world::game::{Collected, Exit, Game};

/// The nickels (template 446, kind 2, group 1) and the gold coin (1218,
/// kind 15, group 0).
const NICKEL: u16 = 446;
const NICKELS: u32 = 62;
const GOLD: u16 = 1218;
const GOLD_KIND: u16 = 15;
const CLOCK: i32 = 170 * 300;
/// Hidden at the end of cines 12 and 27.
const HIDDEN: [u16; 5] = [609, 600, 117, 35, 34];
const LIL: u16 = 609;
/// HUD: the counter's icon (286) and the gold coin's (240).
const COUNTER: u32 = 286;
const GOLD_ICON: u32 = 240;
const CINE_WON: usize = 26;
const CINE_LOST: usize = 27;

pub struct GoldRush {
    pub phase: u8,
    taken: u32,
    gold: bool,
    /// Lil's body plays (after cine 27).
    lil: bool,
    activity: bool,
}

impl GoldRush {
    /// EpisodeStart (0x80079af0) and the round setup (0x80079870).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<GoldRush, String> {
        for id in HIDDEN {
            g.hide(id, true);
        }
        let activity = g.activity;
        let mut l = GoldRush { phase: 1, taken: 0, gold: false, lil: false, activity };
        l.round(g);
        Ok(l)
    }

    /// 0x80079870: the coins back on their spots, the clock, phase.
    fn round(&mut self, g: &mut Game) {
        for s in g.spots.iter_mut() {
            s.used = false;
        }
        let ks: Vec<usize> = (0..g.goodies.len())
            .filter(|&k| g.goodies[k].template == NICKEL || g.goodies[k].template == GOLD)
            .collect();
        for k in ks {
            let group = g.goodies[k].group as usize;
            if g.goodies[k].template == GOLD && (self.gold || self.activity) {
                continue;
            }
            if let Some(i) = crate::world::game::spawn(&mut g.spots, group, &mut g.rand) {
                let at = g.spots[i].pos;
                g.goodies[k].pos = rrt::glam::Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
            }
            g.goodies[k].taken = false;
        }
        self.taken = 0;
        g.timer = Some(CLOCK);
        g.checkpoint = Some(g.player.pose());
        g.song = Some(0);
    }

    /// 0x800585b0: the player and Lil trade bodies and places.
    fn swap_lil(&mut self, g: &mut Game) {
        let Some(k) = g.npcs.iter().position(|n| n.id == LIL) else { return };
        let here = g.player.pos;
        let n = &mut g.npcs[k];
        std::mem::swap(&mut g.player.hierarchy, &mut n.hierarchy);
        std::mem::swap(&mut g.player.animator, &mut n.animator);
        let there = n.pos;
        n.pos = here;
        n.mode = Mode::Idle;
        std::mem::swap(&mut g.character, &mut n.character);
        n.hidden = true;
        g.player.pos = there;
        g.player.v = [0; 3];
        self.lil = !self.lil;
    }
}

impl crate::world::game::Script for GoldRush {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        g.npc_events.clear();
        for Collected { kind, .. } in std::mem::take(&mut g.collected) {
            if kind == GOLD_KIND {
                self.gold = true;
                g.fourth_bar = true;
            } else if kind == 2 {
                // 0x800514b0: sound 31; the 62nd ends the clock next tick.
                self.taken += 1;
                if self.taken >= NICKELS {
                    g.timer = Some(1);
                }
            }
        }
    }

    /// 0x80079b38: all taken wins (cine 26, as Phil); else ForceCry.
    /// Under 30 taken the baby says so first (0x8007f8a4: Lil 986, Phil
    /// 859).
    fn timer_expired(&mut self, g: &mut Game) {
        if self.taken < 30 {
            match g.character {
                4 => g.speech.push(986),
                5 => g.speech.push(859),
                _ => {}
            }
        }
        if self.taken >= NICKELS {
            if self.lil {
                self.swap_lil(g);
            }
            g.cine = Some(CINE_WON);
        } else {
            g.cry();
        }
    }

    /// Phil's case has no cry (0x800800e0); Lil's does.
    fn cry_animation(&self) -> bool {
        self.lil
    }

    /// ForceCry's level 17 case: a try; Phil's loss plays cine 27, Lil's
    /// is the hub.
    fn cry(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if self.lil || g.tries <= 0 {
            g.exit = Some(Exit::to(crate::world::game::HUB));
        } else {
            g.cine = Some(CINE_LOST);
        }
        true
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_LOST {
            // Card 709, then Lil's round (0x80079c10).
            g.card = Some(709);
            for id in HIDDEN.iter().filter(|&&id| id != LIL) {
                g.hide(*id, true);
            }
            self.swap_lil(g);
            self.phase = 2;
            self.round(g);
        }
    }

    fn draw(&self, g: &Game, _moving: &mut Vec<rugrats_render::Vtx>, overlay: &mut Vec<rugrats_render::Vtx>) {
        if let Some(s) = &g.sprites {
            s.draw(COUNTER, (-192, -80), 32.0, 32.0, 1024.0, 0.0, g.focal(), overlay);
            if self.gold {
                s.draw(GOLD_ICON, (-140, -60), 32.0, 32.0, 1024.0, 0.0, g.focal(), overlay);
            }
        }
        crate::world::game::shadowed(&g.fonts, -170, -93, format!("{}", NICKELS.saturating_sub(self.taken)), overlay);
    }

    fn phase(&self) -> u8 {
        self.phase
    }

    fn describe(&self) -> String {
        format!("phase {} taken {} gold {} lil {}", self.phase, self.taken, self.gold, self.lil)
    }
}
