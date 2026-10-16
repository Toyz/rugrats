//! Moving platforms (class 11).

use super::*;
use crate::actors::player::RADIUS;

/// A moving platform (0x80048804, think 0x8004ee00). Per axis it steps
/// by the template's +0x18.. (+0xb8..) and counts what it has gone
/// (+0xa8..) out to the range +0x1e.. (+0xb0..) and back to 0, the way
/// set in +0xbe bits 1, 2, 4 (out when set).
pub struct Mover {
    pub id: u16,
    pub template: u16,
    pub shape: u32,
    pub start: [i32; 3],
    pub pos: [i32; 3],
    pub rot: Mat3,
    pub(super) step: [i32; 3],
    pub(super) range: [i32; 3],
    pub(super) gone: [i32; 3],
    pub(super) out: [bool; 3],
    /// Object flag 0x20000 (template flag 0x10000000): it turns at the
    /// ends and goes on; else its think stops when every axis is done.
    pub(super) back_and_forth: bool,
    /// This frame's move.
    pub delta: [i32; 3],
    /// The enter callback (0x8004f228) arms it when the player comes
    /// within the template's trigger radius (+0x34, the trigger list
    /// 0x80072d38); 0 always. Armed is "its think is scheduled".
    pub(super) radius: i32,
    pub(super) armed: bool,
    /// The player within the trigger radius last frame (enter is the edge).
    pub(super) inside: bool,
    /// +0xbe bit 64 (template flag 0x40000000): no enter callback; armed
    /// by landing on it (the floor query 0x800627a0), and turning at an end
    /// only while ridden or with the player over it.
    pub(super) ridden_only: bool,
    /// Ridden last frame: landing on it is the edge (the player's +0xf8
    /// becoming it); standing on it does not arm it again. Level 5's
    /// elevator, ridden to the bottom, stays there (oracle: at y 7722 from
    /// frame 4100 to 5600 with Angelica on it): its think ends there,
    /// 0x8004ed00 answering 0 while ridden.
    pub(super) rode: bool,
    /// +0xbe bit 256 (template 855, 0x80048804): unridden, it turns toward
    /// the end nearer the player's height (0x8004ed00); its base +0xc0.
    pub(super) calls: bool,
}

impl Mover {
    pub(super) fn world(&self) -> Vec3 {
        Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32)
    }

    /// Back where it started, disarmed (0x80049e5c, 0x800487b8).
    pub fn reset(&mut self) {
        self.pos = self.start;
        self.gone = [0; 3];
        self.out = [false; 3];
        self.armed = false;
        self.inside = false;
        self.rode = false;
    }

    /// Its step on axis `axis` (a level's own, as level 5's 60).
    pub fn set_step(&mut self, axis: usize, step: i32) {
        self.step[axis] = step;
    }

    /// Whether axis `i` has nothing left to go its way: out, |gone| has
    /// reached |range|; back, gone is 0. An axis with no range is done.
    fn done(&self, i: usize) -> bool {
        self.range[i] == 0 || if self.out[i] { self.gone[i].abs() >= self.range[i].abs() } else { self.gone[i] == 0 }
    }

    /// Arming (0x8004f228): the think scheduled, each done axis turned.
    fn arm(&mut self) {
        self.armed = true;
        for i in 0..3 {
            if self.range[i] != 0 && self.done(i) {
                self.out[i] = !self.out[i];
            }
        }
    }

    /// 0x8004ed00 for a calling mover, unridden and the player not over
    /// it: level 5's elevator just answers 1; else it turns toward the end
    /// (the base +0xc0 or the base + the y range) nearer the player's
    /// height, answering 1 when it turned.
    fn call(&mut self, player_y: i32, elevator: bool) -> bool {
        if elevator {
            return true;
        }
        let base = self.start[1];
        let (to_base, to_end) = ((player_y - base).abs(), (player_y - (base + self.range[1])).abs());
        if self.out[1] && to_base < to_end {
            self.out[1] = false;
            true
        } else if !self.out[1] && to_end < to_base {
            self.out[1] = true;
            true
        } else {
            false
        }
    }

    /// One frame: arming as the original's callbacks do, then the think
    /// (0x8004ee00) while armed. `over` is 0x8004ec34 (the player inside
    /// its bounds in x and z), `blocked` 0x8006baa8 (the player in its
    /// boxes), `elevator` level 5's template 855.
    pub(super) fn step(&mut self, player: [i32; 3], ridden: bool, over: bool, blocked: bool, elevator: bool) {
        self.delta = [0; 3];
        let (dx, dz) = ((player[0] - self.pos[0]) as i64, (player[2] - self.pos[2]) as i64);
        let near = self.radius == 0 || dx * dx + dz * dz < (self.radius as i64).pow(2);
        let entered = near && !self.inside;
        self.inside = near;
        let landed = ridden && !self.rode;
        self.rode = ridden;
        if !self.armed {
            if (self.ridden_only && landed) || (!self.ridden_only && entered) {
                self.arm();
            } else {
                return;
            }
        }
        // Unridden: held while something is in its way, and asked whether
        // to turn toward the player.
        let mut called = false;
        if !ridden {
            if blocked {
                return;
            }
            called = self.calls && !over && self.call(player[1], elevator);
        }
        let done = [0, 1, 2].map(|i| self.done(i));
        for i in 0..3 {
            if done[i] {
                continue;
            }
            // Level 5's elevator, unridden, does not go back up past its
            // middle (0x8004efdc).
            if i == 1 && elevator && called && self.gone[1] < self.range[1] / 2 {
                continue;
            }
            let d = if self.out[i] { self.step[i] } else { -self.step[i] };
            self.gone[i] += d;
            self.pos[i] += d;
            self.delta[i] = d;
        }
        if self.back_and_forth {
            if !self.ridden_only || ridden || over {
                for (out, (&range, &done)) in self.out.iter_mut().zip(self.range.iter().zip(&done)) {
                    if range != 0 && done {
                        *out = !*out;
                    }
                }
            }
        } else if done.iter().all(|&d| d) && !(self.calls && !ridden && !over && self.call(player[1], elevator)) {
            // Every axis done and nothing to call it: the think stops.
            self.armed = false;
        }
    }
}

impl Game {
    /// For mover `m`: the player over it in x and z within its model's
    /// bounds, turned into its frame (0x8004ec34), and the player's body in
    /// one of its boxes from the side (0x8006baa8's test of the player).
    pub(super) fn mover_tests(&self, m: &Mover) -> (bool, bool) {
        let Some(model) = self.set.models.get(m.shape) else { return (false, false) };
        let p = self.player.pos;
        let d = Vec3::new((p[0] - m.pos[0]) as f32, (p[1] - m.pos[1]) as f32, (p[2] - m.pos[2]) as f32);
        let local = m.rot.transpose() * d;
        let over = model.boxes.first().is_some_and(|(lo, hi)| {
            lo[0] as f32 <= local.x && local.x <= hi[0] as f32 && lo[2] as f32 <= local.z && local.z <= hi[2] as f32
        });
        let feet = p[1] - self.player.feet();
        let blocked = world_boxes(model, m.rot, m.world())
            .iter()
            .any(|b| b.covers(p, RADIUS) && b.min_y < feet - 48 && b.max_y > feet - HEIGHT);
        (over, blocked)
    }

    /// The mover whose top the player stands on.
    pub(super) fn mover_under_player(&self) -> Option<usize> {
        let p = self.player.pos;
        let feet = p[1] - self.player.feet();
        self.movers.iter().position(|m| {
            self.set.models.get(m.shape).is_some_and(|model| {
                world_boxes(model, m.rot, m.world()).iter().any(|b| b.covers(p, RADIUS) && (b.min_y - feet).abs() <= 48)
            })
        })
    }
}
