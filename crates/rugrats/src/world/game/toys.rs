//! Spinning toys (class 2) and post walkers (class 15).

use super::*;

/// A spinning toy (class 2, 0x80048364, think 0x8004e2ac): while the
/// player is within its trigger radius its angles turn by template +0x18..
/// +0x1c a VSync (two a game frame).
pub struct Spinner {
    pub placement: usize,
    pub(super) shape: u32,
    pub(super) pos: Vec3,
    pub(super) placed: Mat3,
    pub(super) speed: [i32; 3],
    pub(super) angle: [i32; 3],
    pub(super) radius: i32,
}

impl Spinner {
    /// Its rotation now: the think adds the turn to the object's own
    /// angles (+0x24..+0x28, its placement's), and the matrix is made from
    /// their sum (0x80020ee0); not the placement turned by the spin. Level
    /// 3's tipped toy (template 526, z angle -1024, x turning 35 a VSync,
    /// oracle) so spins about the upright, its height fixed.
    pub(super) fn rot(&self, trig: &Trig) -> Mat3 {
        let angles = [0, 1, 2].map(|i| (placed_angles(self.placed)[i] + self.angle[i]) as i16);
        let e = trig.euler(angles);
        let cols: Vec<f32> = e.concat().into_iter().map(|v| v as f32 / 4096.0).collect();
        Mat3::from_cols_slice(&cols)
    }
}

/// The angles (a, b, c), 4096 a turn, whose 0x80020ee0 matrix `placed` is
/// (as the object transform uses it, transposed): its rows are `placed`'s
/// columns, the third (cos b sin a, -sin b, cos b cos a) and the middle
/// column (sin c cos b, cos c cos b).
fn placed_angles(placed: Mat3) -> [i32; 3] {
    let m = |r: usize, c: usize| placed.col(r)[c];
    let turn = |v: f32| (v * 4096.0 / std::f32::consts::TAU).round() as i32;
    let b = (-m(2, 1)).atan2(m(2, 0).hypot(m(2, 2)));
    let a = m(2, 0).atan2(m(2, 2));
    let c = m(0, 1).atan2(m(1, 1));
    [turn(a), turn(b), turn(c)]
}

/// A post walker (class 15, 0x80048974, think 0x8004dba0): to its post
/// (+0xa8) at LP +0x48 / 300 a think, turning at most 512, the next post
/// the first neighbour not the one come from; within LP +0x4a of a post it
/// moves on (under 11: it stops).
pub struct Walker {
    pub placement: usize,
    pub template: u16,
    pub(super) shape: u32,
    pub pos: [i32; 3],
    pub(super) heading: i32,
    pub(super) speed: i32,
    pub(super) reach: i32,
    pub(super) post: u8,
    pub(super) prev: u8,
    pub(super) radius: i32,
    pub(super) stopped: bool,
}

impl Walker {
    pub(super) fn world(&self) -> Vec3 {
        Vec3::new(self.pos[0] as f32, self.pos[1] as f32, self.pos[2] as f32)
    }

    /// Its placed rotation turned to its heading now.
    pub(super) fn rot(&self, trig: &Trig) -> Mat3 {
        let e = trig.euler([self.heading as i16, 0, 0]);
        let cols: Vec<f32> = e.concat().into_iter().map(|v| v as f32 / 4096.0).collect();
        Mat3::from_cols_slice(&cols)
    }
}

impl Game {
    /// The spinners' and walkers' thinks while the player is within their
    /// trigger radius (enter 0x80049e18, leave 0x80049e5c).
    pub(super) fn toy_thinks(&mut self) {
        let me = self.player.pos;
        let near = |at: [i32; 3], r: i32| {
            let (dx, dz) = ((at[0] - me[0]) as i64, (at[2] - me[2]) as i64);
            dx * dx + dz * dz < (r as i64) * (r as i64)
        };
        for s in &mut self.spinners {
            let at = [s.pos.x as i32, s.pos.y as i32, s.pos.z as i32];
            if near(at, s.radius) {
                // Angles += speed x VSyncs since the last think.
                for i in 0..3 {
                    s.angle[i] = (s.angle[i] + 2 * s.speed[i]).rem_euclid(4096);
                }
            }
        }
        for k in 0..self.walkers.len() {
            let w = &self.walkers[k];
            if w.stopped || !near(w.pos, w.radius) || w.post == 0 {
                continue;
            }
            let Some(post) = self.posts.get(w.post as usize - 1).copied() else { continue };
            let (dx, dz) = ((post.pos[0] - w.pos[0]) as i64, (post.pos[2] - w.pos[2]) as i64);
            let w = &mut self.walkers[k];
            if w.reach < 11 {
                w.stopped = true;
                continue;
            }
            if dx * dx + dz * dz < (w.reach as i64).pow(2) {
                let next = post.next.iter().copied().find(|&n| n != 0 && n != w.prev).unwrap_or(w.prev);
                w.prev = w.post;
                w.post = next;
                continue;
            }
            // Turn at most 512, then step along the heading.
            let want = ((dx as f64).atan2(dz as f64) / std::f64::consts::TAU * 4096.0).round() as i32;
            let d = (want - w.heading + 2048).rem_euclid(4096) - 2048;
            w.heading = (w.heading + d.clamp(-512, 512)).rem_euclid(4096);
            let (s, c) = (self.trig.sin(w.heading), self.trig.cos(w.heading));
            let next = [w.pos[0] + ((s * w.speed) >> 12), w.pos[1], w.pos[2] + ((c * w.speed) >> 12)];
            // Blocked by the player (0x8006baa8): it waits.
            let (bx, bz) = ((next[0] - me[0]) as i64, (next[2] - me[2]) as i64);
            if bx * bx + bz * bz > 400 * 400 {
                w.pos = next;
            }
        }
    }
}
