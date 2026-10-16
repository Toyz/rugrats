//! The follow camera: its aim, height and distance.

use super::*;
use crate::world::collide;

/// The follow camera (0x80027304): it aims 768 above the player's origin
/// (0x800c3f58) from 4096 behind (0x800c4a4c) and 1024 up (0x800c3f5c),
/// and eases 1/8 of the way a frame.
pub(super) const AIM_UP: f32 = 768.0;

pub(super) const BEHIND: f32 = 4096.0;

pub(super) const EYE_UP: f32 = 1024.0;

/// A look aside (0x8002729c): L2 and R2 swing the view 341 left or right
/// about the eye, L1 tilts it 341 up; it eases out and back by itself
/// (0x800247fc, the offsets at 0x800de430/434, the targets at 438/43c,
/// 0x800c4a3c bits 16 and 32).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Look {
    pub yaw: i32,
    pub pitch: i32,
    to_yaw: i32,
    to_pitch: i32,
    /// Under way (bit 16) and on the way back (bit 32).
    on: bool,
    back: bool,
}

impl Look {
    /// A look toward (yaw, pitch); none while one is under way.
    pub fn start(&mut self, to_yaw: i32, to_pitch: i32) {
        if !self.on {
            *self = Look { to_yaw, to_pitch, on: true, ..Look::default() };
        }
    }

    /// R1 (0x80026ddc) drops it.
    pub fn cancel(&mut self) {
        *self = Look::default();
    }

    /// One frame (0x800247fc): the step is (rsin(offset * 2048 / target)
    /// / 16 + 128) * dt (136) / 4096, slow at the ends and fast midway;
    /// at the target it turns back, at 0 it ends.
    pub fn step(&mut self, trig: &Trig) {
        if !self.on {
            return;
        }
        let (off, to) = if self.to_yaw != 0 { (&mut self.yaw, self.to_yaw) } else { (&mut self.pitch, self.to_pitch) };
        if to == 0 {
            return;
        }
        let s = trig.sin((*off << 11) / to);
        let v = ((if s < 0 { s + 15 } else { s } >> 4) + 128) * 136;
        let step = if v < 0 { v + 4095 } else { v } >> 12;
        if self.back {
            *off += if to > 0 { -step } else { step };
            if (to > 0 && *off <= 0) || (to < 0 && *off >= 0) {
                *self = Look::default();
            }
        } else {
            *off += if to > 0 { step } else { -step };
            if (to > 0 && *off >= to) || (to < 0 && *off <= to) {
                *off = to;
                self.back = true;
            }
        }
    }
}

/// The follow camera's height by level (0x800290a4, 0x800291f8: 512 in
/// levels 4 and 15, -1000 in 7).
pub(super) fn default_eye_up(level: Level) -> f32 {
    match level {
        Level::CYNTHIA | Level::MIRRORLAND => 512.0,
        Level::REPTAR_2010 => -1000.0,
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

    /// The eye pulled in front of walls (0x80027304): the segment from the
    /// aim point to `goal` (0x8002cd7c), the eye where it stops.
    pub(super) fn clear_eye(&self, goal: Vec3) -> Vec3 {
        let from = self.target();
        let mut hit = collide::Segment::default();
        let ends = [from, goal].map(|p| [p.x as i32, p.y as i32, p.z as i32]);
        if collide::segment(&self.col, ends[0], ends[1], &mut hit) == 1 {
            return Vec3::new(hit.point[0] as f32, hit.point[1] as f32, hit.point[2] as f32);
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
            (Level::OUTER_SPACE, Some((anchor, _))) => anchor,
            _ => p[1],
        };
        Vec3::new(p[0] as f32, y as f32 - up, p[2] as f32)
    }

    pub(super) fn eye_goal(&self) -> Vec3 {
        let a = self.yaw / 4096.0 * std::f32::consts::TAU;
        self.target() - Vec3::new(a.sin(), 0.0, a.cos()) * self.behind - Vec3::new(0.0, self.eye_up, 0.0)
    }
}
