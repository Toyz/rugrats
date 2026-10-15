//! Level 10, Chuckie's Glasses (docs/levels/level-10.md): hide and seek.
//! Phil, Lil and Tommy hide at random spots; Chuckie tags one (Triangle or
//! Circle) and it runs home along its posts. Chuckie in the playpen first
//! catches it (cine 75-77); all three caught plays cine 78: won. A baby
//! reaching post 120 first wins the race (cine 41-43): it hides again and
//! a try is gone.

use crate::actors::npc::post::GOAL;
use crate::actors::npc::{Event, Mode};
use crate::actors::player::{DT, Pose};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::levels::Level;
use crate::world::game::{self, Exit, Game, Script};

/// The hiders: Phil, Lil, Tommy; their catch and win cines.
const BABIES: [(u16, usize, usize); 3] = [(608, 76, 42), (609, 75, 41), (1492, 77, 43)];
/// Hidden by cine 44's end.
const HIDDEN: [u16; 5] = [13, 14, 15, 16, 600];
/// Home: post 120 gets flag 32 (EpisodeStart 0x8007ee54).
const HOME_POST: usize = 120;
/// The playpen (template 1032 at (5926, -1387, 21212)): the area the
/// oracle's catches fit (guess at 0x8006abbc's test).
const PEN: [i32; 2] = [5926, 21212];
const PEN_HALF: [i32; 2] = [1390, 2070];
/// Running speed (table row 1, index 4) and the tag's reach.
const RUN: i32 = 11;
const TAG_REACH: i64 = 900;
const CINE_WON: usize = 78;

struct Baby {
    id: u16,
    spot: Option<usize>,
    caught: bool,
}

pub struct ChuckiesGlasses {
    /// Races a baby has won home first (0x800c4cd0, the phase, less 1).
    races_lost: u8,
    babies: Vec<Baby>,
    /// The running babies' line countdown (0x800c4ce4).
    talk_wait: i32,
}

/// What a running baby says (0x80084084 mode 6, level 10): Tommy
/// 0x800b9704, Phil 0x800b9710, Lil 0x800b971c.
fn running_lines(id: u16) -> &'static [u16] {
    match id {
        1492 => &[178, 180, 181, 166, 170, 171],
        608 => &[189, 190, 192, 166, 170, 171],
        _ => &[197, 198, 166, 170, 171],
    }
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl ChuckiesGlasses {
    /// EpisodeStart (0x8007ee54) after cine 44.
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<ChuckiesGlasses, String> {
        for id in HIDDEN {
            g.hide(id, true);
        }
        if let Some(p) = g.posts.get_mut(HOME_POST - 1) {
            p.flags |= GOAL;
        }
        let mut l = ChuckiesGlasses {
            races_lost: 0,
            babies: BABIES.iter().map(|b| Baby { id: b.0, spot: None, caught: false }).collect(),
            talk_wait: 0,
        };
        for i in 0..l.babies.len() {
            l.hide(g, i);
        }
        g.checkpoint = Some(g.player.pose());
        Ok(l)
    }

    /// 0x8007f004: to a random free spot (0x8004fe14), hiding.
    fn hide(&mut self, g: &mut Game, i: usize) {
        let Some(k) = find(g, self.babies[i].id) else { return };
        let spot = game::spawn(&mut g.spots, 0, &mut g.rand);
        self.babies[i].spot = spot;
        let n = &mut g.npcs[k];
        n.hidden = false;
        n.mode = Mode::Idle;
        n.run_speed = Some(RUN);
        if let Some(s) = spot {
            let (at, link) = (g.spots[s].pos, g.spots[s].link);
            n.first_post = link;
            n.place(at, &g.col);
        }
    }

    fn running(&self, g: &Game) -> bool {
        self.babies.iter().any(|b| find(g, b.id).is_some_and(|k| matches!(g.npcs[k].mode, Mode::Posts { .. })))
    }

    /// A lost race without the cry (the first two): a try, back to the
    /// start; the last try is the hub.
    fn lose_try(&mut self, g: &mut Game) {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some(Exit::to(Level::HUB));
            return;
        }
        if let Some(Pose { pos: at, heading: h }) = g.checkpoint {
            g.player.pos = at;
            g.player.heading = h;
            g.player.target = h;
            g.set_camera_yaw(h as f32);
        }
    }
}

impl Script for ChuckiesGlasses {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            // GoalPost (0x8007f150): a baby home first.
            if let Event::Goal(k) = e {
                let id = g.npcs[k].id;
                let Some(i) = self.babies.iter().position(|b| b.id == id && !b.caught) else { continue };
                self.races_lost += 1;
                g.npcs[k].mode = Mode::Idle;
                if self.races_lost < 3 {
                    g.cine = Some(BABIES[i].2);
                } else {
                    g.cry();
                }
            }
        }
        // The tag (0x800831a4): Triangle or Circle beside a hiding baby,
        // none running.
        if (pads.pressed(0, input::TRIANGLE) || pads.pressed(0, input::CIRCLE)) && !self.running(g) {
            let p = g.player.pos;
            let near = self.babies.iter().position(|b| {
                !b.caught
                    && find(g, b.id).is_some_and(|k| {
                        let n = &g.npcs[k];
                        let (dx, dz) = ((n.pos[0] - p[0]) as i64, (n.pos[2] - p[2]) as i64);
                        n.mode == Mode::Idle && dx * dx + dz * dz < TAG_REACH * TAG_REACH
                    })
            });
            if let Some(i) = near
                && let Some(k) = find(g, self.babies[i].id)
            {
                if let Some(s) = self.babies[i].spot.take() {
                    g.spots[s].used = false;
                }
                let first = g.npcs[k].first_post;
                g.npcs[k].mode = Mode::Posts { prev: 0, cur: first };
                // Chuckie says 0x800b9728[(rand & 255) mod 3].
                g.speech.push([158, 160, 165][(g.rand.next() & 255) as usize % 3]);
            }
        }
        // Each running baby counts the shared countdown down and says a
        // line at 0.
        for b in &self.babies {
            if find(g, b.id).is_some_and(|k| matches!(g.npcs[k].mode, Mode::Posts { .. })) {
                self.talk_wait -= DT;
                if self.talk_wait <= 0 {
                    let lines = running_lines(b.id);
                    g.speech.push(lines[(g.rand.next() & 255) as usize % lines.len()]);
                    self.talk_wait = ((g.rand.next() & 255) * 16) as i32;
                }
            }
        }
        // The playpen (0x8007f2e8): a running baby is caught.
        let p = g.player.pos;
        let in_pen = (p[0] - PEN[0]).abs() < PEN_HALF[0] && (p[2] - PEN[1]).abs() < PEN_HALF[1];
        if in_pen {
            let caught = self
                .babies
                .iter()
                .position(|b| !b.caught && find(g, b.id).is_some_and(|k| matches!(g.npcs[k].mode, Mode::Posts { .. })));
            if let Some(i) = caught {
                self.babies[i].caught = true;
                if let Some(k) = find(g, self.babies[i].id) {
                    g.npcs[k].mode = Mode::Idle;
                }
                g.cine = Some(BABIES[i].1);
            }
        }
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        // After a catch: all three caught plays cine 78.
        if BABIES.iter().any(|b| b.1 == n) && self.babies.iter().all(|b| b.caught) {
            g.cine = Some(CINE_WON);
        }
        // After a lost race (0x8007f068): hiding again, a try.
        if let Some(i) = BABIES.iter().position(|b| b.2 == n) {
            self.hide(g, i);
            self.lose_try(g);
        }
    }

    /// ForceCry's level 10 case: the cry only at the third race lost.
    fn cry(&mut self, g: &mut Game) -> bool {
        g.tries -= 1;
        if g.tries <= 0 {
            g.exit = Some(Exit::to(Level::HUB));
        }
        true
    }

    /// The game's phase: 1 plus the races lost.
    fn phase(&self) -> u8 {
        self.races_lost + 1
    }

    fn describe(&self) -> String {
        let s: Vec<_> = self.babies.iter().map(|b| (b.id, b.spot, b.caught)).collect();
        format!("races lost {} babies {s:?}", self.races_lost)
    }
}
