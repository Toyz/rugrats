//! Level 15, Mirrorland (docs/levels/level-15.md): six ? boxes in the
//! upside-down house. Triangle beside one starts its colour: 25 or 30
//! balloons over that floor and 90 s on the clock. Taking them all fills
//! the colour's icon; checkpoints come after the first and third colour;
//! all six play cine 72: won. The clock out is a cry that undoes the
//! colours since the last checkpoint. Tries never run out (0).

use rugrats_render::Vtx;

use crate::actors::player::Pose;
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::world::game::{self, Collected, Game, Script};

/// The boxes (template 1102, ids 207-212) and the room each stands in;
/// the room's (group, slot) (0x8007d02c).
const BOXES: [(u16, usize); 6] = [(207, 25), (208, 19), (209, 21), (210, 5), (211, 2), (212, 45)];
const BOX_REACH: i64 = 1300;
/// Rooms to (group, slot): slot order 3, 1, 2, 0, 4, 5 from LP order.
fn colour_of(room: usize) -> Option<(u16, usize)> {
    Some(match room {
        25 => (0, 3),
        19 | 20 => (1, 1),
        21 => (2, 2),
        5 => (3, 0),
        2 => (4, 4),
        45 => (5, 5),
        _ => return None,
    })
}
/// The balloons' kinds 6-11.
const BALLOON_KINDS: std::ops::RangeInclusive<u16> = 6..=11;
const CLOCK: i32 = 90 * 300;
/// HUD sprites: empty icon 311, full by group (306, 309, 310, 305, 307,
/// 308), the counter's by group (301, 304, 299, 300, 303, 302).
const EMPTY: u32 = 311;
const FULL: [u32; 6] = [306, 309, 310, 305, 307, 308];
const COUNTER: [u32; 6] = [301, 304, 299, 300, 303, 302];
const CINE_WON: usize = 72;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Idle,
    Running,
    Done,
}

/// The level's progress (0x800c4cd0): the checkpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Phase {
    /// No colour done yet.
    Start = 1,
    /// The first colour done: the first checkpoint.
    OneColour = 2,
    /// The third colour done: the second checkpoint.
    ThreeColours = 3,
}

pub struct Mirrorland {
    phase: Phase,
    slots: [Slot; 6],
    /// The slot running, its group, balloons to take and taken.
    running: Option<(usize, u16)>,
    total: u32,
    taken: u32,
    /// Rooms whose box was opened since the last checkpoint.
    since: Vec<usize>,
    respawn: Pose,
    slot_group: [u16; 6],
}

impl Mirrorland {
    /// EpisodeStart (0x8007ceb8).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Mirrorland, String> {
        for id in [16, 136, 137, 138, 139] {
            g.hide(id, true);
        }
        g.tries = 0;
        // The balloons wait in the loader's spawn lists (0x800e2710) until
        // a box places its colour's (0x8007ccb4): none in the world yet.
        for b in g.goodies.iter_mut().filter(|b| BALLOON_KINDS.contains(&b.kind)) {
            b.taken = true;
        }
        let respawn = g.player.pose();
        g.checkpoint = Some(respawn);
        let mut slot_group = [0; 6];
        for room in [25, 19, 21, 5, 2, 45] {
            if let Some((group, slot)) = colour_of(room) {
                slot_group[slot] = group;
            }
        }
        Ok(Mirrorland {
            phase: Phase::Start,
            slots: [Slot::Idle; 6],
            running: None,
            total: 0,
            taken: 0,
            since: Vec::new(),
            respawn,
            slot_group,
        })
    }

    /// The balloons of `group` in or out of the world (0x8004fe14,
    /// 0x8004fd20).
    fn balloons(&self, g: &mut Game, group: u16, out: bool) -> u32 {
        let mut n = 0;
        let ks: Vec<usize> = (0..g.goodies.len())
            .filter(|&k| BALLOON_KINDS.contains(&g.goodies[k].kind) && g.goodies[k].group == group)
            .collect();
        for k in ks {
            if out {
                g.goodies[k].taken = true;
            } else if let Some(i) = game::spawn(&mut g.spots, group as usize, &mut g.rand) {
                let at = g.spots[i].pos;
                g.goodies[k].pos = rrt::glam::Vec3::new(at[0] as f32, at[1] as f32, at[2] as f32);
                g.goodies[k].taken = false;
                n += 1;
            }
        }
        // Spots are freed for the next time.
        for s in g.spots.iter_mut().filter(|s| s.group as u16 == group) {
            s.used = false;
        }
        n
    }

    /// 0x8007d0f4 and 0x8007ccb4: the colour of the player's room.
    fn open(&mut self, g: &mut Game, id: u16, room: usize) {
        let Some((group, slot)) = colour_of(room) else { return };
        // The break's sound at the object (0x8004d434: 114).
        let place = g.set.placements.iter().position(|p| p.id == id);
        let at = place.map(|k| g.set.placements[k].pos);
        g.hide(id, true);
        match at {
            Some(at) => {
                // Its pieces from its own place (0x8004cdfc, 100).
                if let Some(k) = place {
                    g.shatter(k, 100);
                }
                g.play_at(114, at);
                // Its burst of five (0x80054894), the sprite by its room.
                let room = g.room_of(at).unwrap_or(room);
                g.flights.break_burst(room, at, 0, g.sprites.as_ref());
            }
            None => g.play(114),
        }
        if self.since.is_empty() {
            self.respawn = g.player.pose();
            g.checkpoint = Some(self.respawn);
        }
        self.since.push(room);
        self.total = self.balloons(g, group, false);
        self.taken = 0;
        self.running = Some((slot, group));
        self.slots[slot] = Slot::Running;
        g.timer = Some(CLOCK);
    }

    /// 0x8007d250: a colour done.
    fn done(&mut self, g: &mut Game) {
        let Some((slot, _)) = self.running.take() else { return };
        g.timer = None;
        self.slots[slot] = Slot::Done;
        let count = self.slots.iter().filter(|s| **s == Slot::Done).count();
        match count {
            1 | 3 => {
                self.phase = if count == 1 { Phase::OneColour } else { Phase::ThreeColours };
                self.respawn = g.player.pose();
                g.checkpoint = Some(self.respawn);
                self.since.clear();
            }
            6 => g.cine = Some(CINE_WON),
            _ => {}
        }
    }
}

impl Script for Mirrorland {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        // Balloons taken (0x800514b0 kinds 6-11): sound 113, the count.
        for Collected { kind, .. } in std::mem::take(&mut g.collected) {
            if BALLOON_KINDS.contains(&kind) && self.running.is_some() {
                g.play(113);
                self.taken += 1;
                if self.taken >= self.total {
                    self.done(g);
                }
            }
        }
        // Triangle beside a box (0x8004d434), none running.
        if pads.pressed(0, input::TRIANGLE) && self.running.is_none() {
            let p = g.player.pos;
            let near = BOXES.iter().find_map(|&(id, _)| {
                g.set
                    .placements
                    .iter()
                    .find(|q| q.kind == 1 && q.id == id)
                    .filter(|q| {
                        let (dx, dz) = ((q.pos[0] - p[0]) as i64, (q.pos[2] - p[2]) as i64);
                        dx * dx + dz * dz < BOX_REACH * BOX_REACH && !g.is_hidden(id)
                    })
                    .map(|_| id)
            });
            if let (Some(id), Some(room)) = (near, g.current_room()) {
                self.open(g, id, room);
            }
        }
    }

    /// 0x8007d4e4: the clock out is a cry.
    fn timer_expired(&mut self, g: &mut Game) {
        g.cry();
    }

    /// ForceCry's level 15 case and 0x8007d52c: back to the respawn, the
    /// colours since the last checkpoint undone; tries never run out.
    fn cry(&mut self, g: &mut Game) -> bool {
        let rooms = std::mem::take(&mut self.since);
        for room in &rooms {
            if let Some((group, slot)) = colour_of(*room) {
                self.balloons(g, group, true);
                self.slots[slot] = Slot::Idle;
            }
            if let Some(&(id, _)) = BOXES.iter().find(|b| b.1 == *room || colour_of(b.1) == colour_of(*room)) {
                g.hide(id, false);
            }
        }
        self.running = None;
        g.timer = None;
        let Pose { pos: at, heading: h } = self.respawn;
        g.player.pos = at;
        g.player.heading = h;
        g.player.target = h;
        g.set_camera_yaw(h as f32);
        true
    }

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        let Some(s) = &g.sprites else { return };
        let f = g.focal();
        // The balloons: sprites by group (0x80048df8: 299-304, 240 x 300),
        // bobbing 128 (0x8004d570), facing the camera.
        let t = g.frames as f32;
        let (right, down) = (g.view.1.row(0), g.view.1.row(1));
        for (k, b) in g.goodies.iter().enumerate().filter(|(_, b)| !b.taken && BALLOON_KINDS.contains(&b.kind)) {
            let bob = 128.0 * (t * 0.07 + k as f32 * 1.7).sin();
            let at = b.pos + rrt::glam::Vec3::new(0.0, bob, 0.0);
            s.billboard(COUNTER[b.group as usize % 6], at, 240.0, 300.0, right, down, moving);
        }
        // The six icons (x 50 + 24k, y 73 - 15 (k & 1)).
        for k in 0..6 {
            let at = (50 + 24 * k as i16, 73 - 15 * (k as i16 & 1));
            let sprite = if self.slots[k] == Slot::Done { FULL[self.slot_group[k] as usize] } else { EMPTY };
            s.draw(sprite, at, 36.0, 48.0, 1024.0, 0.0, f, overlay);
        }
        // The counter: the colour's icon at (-192, 73) and the balloons
        // left.
        if let Some((_, group)) = self.running {
            s.draw(COUNTER[group as usize], (-192, 73), 48.0, 64.0, 1024.0, 0.0, f, overlay);
            // (Its count at (-205, 58), 0x8007ccb4.)
            game::shadowed(&g.fonts, -205, 58, format!("{}", self.total.saturating_sub(self.taken)), overlay);
        }
    }

    fn phase(&self) -> u8 {
        self.phase as u8
    }

    fn describe(&self) -> String {
        format!(
            "phase {:?} slots {:?} running {:?} {}/{}",
            self.phase, self.slots, self.running, self.taken, self.total
        )
    }
}
