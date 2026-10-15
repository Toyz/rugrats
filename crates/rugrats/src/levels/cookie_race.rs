//! Level 13, Cookie Race (docs/levels/level-13.md): Tommy races Angelica
//! from the bedroom down to the kitchen. She runs her posts from 50,
//! waiting at some; reaching post 3 (flag 32) first plays cine 20, lost.
//! Tommy entering room 20 plays cine 21, won. All five cookies taken earn
//! a fourth bar.

use crate::actors::npc::{Event, Mode};
use crate::disc::GameDisc;
use crate::input::Pads;
use crate::world::game::{Collected, Game, Script, goodie};

const ANGELICA: u16 = 600;
/// Her route (EpisodeStart 0x80079600): from post 50, reach 1536.
const FIRST_POST: u8 = 50;
const POST_REACH: i32 = 1536;
const UPPER_FLOOR: i32 = -5120;
/// The cookies (template 863, kind 4).
const COOKIE: u16 = 863;
const CINE_LOST: usize = 20;

/// 0x800b96d0.
const WAIT_LINES: [u16; 8] = [608, 609, 610, 643, 645, 646, 664, 723];

pub struct CookieRace {
    cookies: u32,
    total: u32,
    raced: bool,
}

impl CookieRace {
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<CookieRace, String> {
        // Her LP spot is off the map (cine 6 moves her): she starts at
        // her first post (guess at where the cine leaves her).
        let first = g.posts.get(FIRST_POST as usize - 1).map(|p| p.pos);
        if let Some(k) = g.npcs.iter().position(|n| n.id == ANGELICA) {
            if let Some(at) = first {
                // Upstairs, with the bedroom: snapped from above the
                // upper floor (-5120).
                let feet = g.npcs[k].feet();
                g.npcs[k].place([at[0], UPPER_FLOOR - 600 + feet, at[2]], &g.col);
            }
            let n = &mut g.npcs[k];
            n.post_reach = POST_REACH;
            n.speed_index = 4;
            n.mode = Mode::Posts { prev: 0, cur: FIRST_POST };
        }
        let total = g.goodies.iter().filter(|x| x.template == COOKIE).count() as u32;
        g.checkpoint = Some(g.player.pose());
        Ok(CookieRace { cookies: 0, total, raced: false })
    }
}

impl Script for CookieRace {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            // Angelica waiting at a post taunts (0x80083e2c: 0x800b96d0,
            // rand & 7), and again going on when Tommy comes near
            // (0x80084084 mode 7: 611 or 612).
            match e {
                Event::Waits(_) => g.speech.push(WAIT_LINES[(g.rand.next() & 7) as usize]),
                Event::Resumes(_) => g.speech.push(if g.rand.next() & 1 == 0 { 611 } else { 612 }),
                _ => {}
            }
            // GoalPost's level 13 case (0x80077070): speech 723, cine 20.
            if let Event::Goal(k) = e {
                if g.npcs[k].id == ANGELICA && !self.raced {
                    self.raced = true;
                    g.speech.push(723);
                    g.cine = Some(CINE_LOST);
                }
            }
        }
        for Collected { kind, .. } in std::mem::take(&mut g.collected) {
            if kind == goodie::COOKIE {
                self.cookies += 1;
                // 0x80076c24: all five for the fourth bar.
                if self.total > 0 && self.cookies >= self.total {
                    g.fourth_bar = true;
                }
            }
        }
    }

    /// ForceCry case 12: no cry, the one try, the hub.
    fn cry_animation(&self) -> bool {
        false
    }

    fn describe(&self) -> String {
        format!("cookies {}/{} raced {}", self.cookies, self.total, self.raced)
    }
}
