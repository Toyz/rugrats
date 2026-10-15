//! Moving platforms (class 11).

use super::*;

/// A moving platform (0x80048804, think 0x8004ee00): per axis it steps by
/// the template's +0x18.. until it has gone +0x1e.. and, with template
/// flag 0x10000000, back again, over and over; else it stops there.
pub struct Mover {
    pub id: u16,
    pub shape: u32,
    pub start: [i32; 3],
    pub pos: [i32; 3],
    pub rot: Mat3,
    pub(super) step: [i32; 3],
    pub(super) range: [i32; 3],
    pub(super) gone: [i32; 3],
    pub(super) out: [bool; 3],
    pub(super) back_and_forth: bool,
    /// This frame's move.
    pub delta: [i32; 3],
    /// Armed once the player comes within the template's trigger radius
    /// (+0x34, the trigger list 0x80072d38); 0 always.
    pub(super) radius: i32,
    pub(super) armed: bool,
    /// Template flag 0x40000000 (+0xbe bit 64): moves only while ridden.
    pub(super) ridden_only: bool,
}

impl Mover {
    pub(super) fn world(&self) -> Vec3 {
        Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32)
    }

    /// Back where it started, disarmed (0x80049e5c, 0x800487b8).
    pub fn reset(&mut self) {
        self.pos = self.start;
        self.gone = [0; 3];
        self.out = [true; 3];
        self.armed = false;
    }

    /// Its step on axis `axis` (a level's own, as level 5's 60).
    pub fn set_step(&mut self, axis: usize, step: i32) {
        self.step[axis] = step;
    }

    pub(super) fn step(&mut self, player: [i32; 3], ridden: bool) {
        self.delta = [0; 3];
        if self.ridden_only && !ridden {
            return;
        }
        if !self.armed {
            let (dx, dz) = ((player[0] - self.pos[0]) as i64, (player[2] - self.pos[2]) as i64);
            self.armed = self.radius == 0 || dx * dx + dz * dz < (self.radius as i64).pow(2);
            if !self.armed {
                return;
            }
        }
        for i in 0..3 {
            if self.range[i] == 0 {
                continue;
            }
            let done = if self.out[i] { self.gone[i].abs() >= self.range[i].abs() } else { self.gone[i] == 0 };
            if done {
                if self.back_and_forth {
                    self.out[i] = !self.out[i];
                }
                continue;
            }
            let d = if self.out[i] { self.step[i] } else { -self.step[i] };
            self.gone[i] += d;
            self.pos[i] += d;
            self.delta[i] = d;
        }
    }
}

impl Game {
    /// The mover whose top the player stands on.
    pub(super) fn mover_under_player(&self) -> Option<usize> {
        let p = self.player.pos;
        let feet = p[1] - self.player.feet();
        self.movers.iter().position(|m| {
            self.set.models.get(m.shape).is_some_and(|model| {
                world_boxes(model, m.rot, m.world())
                    .iter()
                    .any(|b| b.covers(p, crate::actors::player::RADIUS) && (b.min_y - feet).abs() <= 48)
            })
        })
    }
}
