//! The follow camera: its aim, height and distance.

use super::*;

/// The follow camera (0x80027304): it aims 768 above the player's origin
/// (0x800c3f58) from 4096 behind (0x800c4a4c) and 1024 up (0x800c3f5c),
/// and eases 1/8 of the way a frame.
pub(super) const AIM_UP: f32 = 768.0;

pub(super) const BEHIND: f32 = 4096.0;

pub(super) const EYE_UP: f32 = 1024.0;

/// The follow camera's height by level (0x800290a4, 0x800291f8: 512 in
/// levels 4 and 15, -1000 in 7).
pub(super) fn default_eye_up(level: usize) -> f32 {
    match level {
        4 | 15 => 512.0,
        7 => -1000.0,
        _ => EYE_UP,
    }
}

impl Game {
    /// The projection distance in play.
    pub fn focal(&self) -> f32 {
        self.focal
    }

    /// Puts the camera behind heading `yaw` at once (for tools).
    pub fn set_camera_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        self.eye = self.eye_goal();
        self.draw();
    }

    /// The eye pulled in front of walls: walked from the aim point toward
    /// `goal` and stopped short of the first point not in the open (as
    /// 0x8002cd7c's "vectorwalk" keeps the camera inside).
    pub(super) fn clear_eye(&self, goal: Vec3) -> Vec3 {
        let from = self.target();
        let steps = 32;
        let mut last = from;
        for k in 1..=steps {
            let p = from + (goal - from) * (k as f32 / steps as f32);
            let q = crate::world::collide::query(&self.col, [p.x as i32, p.y as i32, p.z as i32]);
            if q.kind != 0 {
                return from + (last - from) * 0.9;
            }
            last = p;
        }
        goal
    }

    pub(super) fn target(&self) -> Vec3 {
        let p = self.player.pos;
        // The aim is 400 up for Spike (0x800c3f58).
        let up = if self.character == 6 { 400.0 } else { AIM_UP };
        // Floating in level 5 the goal's y is the anchor (+0x65c), not
        // the bobbing body's (0x80025e90).
        let y = match (self.level, self.player.float) {
            (5, Some((anchor, _))) => anchor,
            _ => p[1],
        };
        Vec3::new(p[0] as f32, y as f32 - up, p[2] as f32)
    }

    pub(super) fn eye_goal(&self) -> Vec3 {
        let a = self.yaw / 4096.0 * std::f32::consts::TAU;
        self.target() - Vec3::new(a.sin(), 0.0, a.cos()) * self.behind - Vec3::new(0.0, self.eye_up, 0.0)
    }
}
