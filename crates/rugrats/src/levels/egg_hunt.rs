//! Level 18, Egg Hunt (docs/levels/level-18.md): Tommy finds the 35 eggs
//! before 170 s run out; the gold egg behind the couch is worth a fourth
//! bar. The 35th egg ends the clock and cine 24 wins; eggs left when it
//! runs out play cine 23 and the hub.

use rugrats_render::Vtx;

use crate::disc::GameDisc;
use crate::input::Pads;
use crate::world::game::{self, Collected, Game, Script};

/// The eggs (template 496, kind 1) and the gold egg (1219, kind 17).
const EGG: u16 = 496;
const GOLD: u16 = 1219;
const GOLD_KIND: u16 = 17;
const CLOCK: i32 = 170 * 300;
/// HUD: the egg icon (143) and the gold egg's (245).
const COUNTER: u32 = 143;
const GOLD_ICON: u32 = 245;
const CINE_WON: usize = 24;
const CINE_LOST: usize = 23;

pub struct EggHunt {
    taken: u32,
    total: u32,
    gold: bool,
}

impl EggHunt {
    /// EpisodeStart (0x80079670).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<EggHunt, String> {
        for id in [600, 99] {
            g.hide(id, true);
        }
        // Activity play leaves the gold egg out (0x80045fa0).
        if g.activity {
            for x in g.goodies.iter_mut().filter(|x| x.template == GOLD) {
                x.taken = true;
            }
        }
        let total = g.goodies.iter().filter(|x| x.template == EGG).count() as u32;
        g.timer = Some(CLOCK);
        g.checkpoint = Some(g.player.pose());
        g.song = Some(0);
        Ok(EggHunt { taken: 0, total, gold: false })
    }
}

impl Script for EggHunt {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        for Collected { kind, .. } in std::mem::take(&mut g.collected) {
            if kind == GOLD_KIND {
                self.gold = true;
                g.fourth_bar = true;
            } else if kind == 1 {
                self.taken += 1;
                // 0x80051b50: the last egg ends the clock next tick.
                if self.taken >= self.total {
                    g.timer = Some(1);
                }
            }
        }
    }

    /// 0x800797ec: all eggs, cine 24; else ForceCry (no cry, the one try)
    /// and cine 23.
    fn timer_expired(&mut self, g: &mut Game) {
        if self.taken >= self.total {
            g.cine = Some(CINE_WON);
        } else {
            g.tries -= 1;
            g.cine = Some(CINE_LOST);
        }
    }

    fn cry_animation(&self) -> bool {
        false
    }

    fn draw(&self, g: &Game, _moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        if let Some(s) = &g.sprites {
            s.draw(COUNTER, (-192, -80), 32.0, 32.0, 1024.0, 0.0, g.focal(), overlay);
            if self.gold {
                s.draw(GOLD_ICON, (-140, -60), 32.0, 32.0, 1024.0, 0.0, g.focal(), overlay);
            }
        }
        game::shadowed(&g.fonts, -170, -93, format!("{}", self.total.saturating_sub(self.taken)), overlay);
    }

    fn describe(&self) -> String {
        format!("eggs {}/{} gold {}", self.taken, self.total, self.gold)
    }
}
