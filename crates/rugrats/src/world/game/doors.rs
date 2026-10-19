//! Doors (docs/engine/doors.md): hinged (class 4), sliding (class 3) and
//! swinging (class 8) doors, their thinks, the use and the blocking test.

use rugrats_data::model::Models;

use super::*;
use crate::actors::player::{DT, HEIGHT, RADIUS};

/// A door (docs/engine/doors.md): class 4 hinged (0x8004860c, think
/// 0x8004f868, use 0x8004f4a0) turns about its origin; class 3 sliding
/// (0x8004845c, think 0x8004e458, use 0x8004f6d8) slides along its own
/// axes. Shut at the start; each use moves it to its other end.
pub struct Door {
    pub placement: usize,
    pub template: u16,
    pub shape: u32,
    pub(super) hinged: bool,
    /// The placed position, rotation and heading (+0xc0).
    pub(super) start: [i32; 3],
    pub(super) placed: Mat3,
    pub(super) heading: i32,
    /// The swing angle (+0xa8) or the slide offset (+0xa8..+0xac).
    pub(super) offset: [i32; 3],
    /// Speed (+0xb0..), shut and open ends (+0xae, +0xb6), the target
    /// (+0xba), either way (+0xbc), the sound kind (+0xd0).
    pub(super) speed: [i32; 3],
    pub(super) shut: i32,
    pub(super) open_end: i32,
    pub(super) target: i32,
    pub(super) either: bool,
    pub(super) sound: u8,
    /// +0x3c bit 2, and the think scheduled (+0xcc).
    pub open: bool,
    pub moving: bool,
    /// Class 8 (0x80048714): swinging by itself while the player is
    /// within the trigger radius.
    pub(super) swing: Option<Swing>,
}

/// A class 8 door's own state: the easing (+0xb8), the waits at the ends
/// (+0xbe: 8 open, 16 shut), the wait left (+0xd2), the trigger radius,
/// the damage when blocked (+0x6d) and the time to the next.
pub(super) struct Swing {
    pub(super) ease: i32,
    pub(super) waits: u8,
    pub(super) wait: i32,
    pub(super) radius: i32,
    pub(super) damage: i32,
    pub(super) hurt: i32,
}

impl Door {
    /// Takes on another door's state as the original keeps it (for
    /// tools): the offset (+0xa8..+0xac), +0x3c bit 2, whether its think
    /// runs (+0xcc) and the wait left (+0xd2).
    pub fn sync(&mut self, offset: [i32; 3], open: bool, moving: bool, wait: i32) {
        self.offset = offset;
        self.open = open;
        self.moving = moving;
        if let Some(s) = &mut self.swing {
            s.wait = wait;
        }
    }

    /// Where it was placed (+0xc0..+0xc8).
    pub fn start(&self) -> [i32; 3] {
        self.start
    }

    /// Its motion in words, for tools.
    pub fn state(&self) -> String {
        format!(
            "offset {:?} open {} moving {} wait {:?} speed {:?} ends {} {}",
            self.offset,
            self.open,
            self.moving,
            self.swing.as_ref().map(|s| (s.wait, s.waits)),
            self.speed,
            self.shut,
            self.open_end
        )
    }

    /// Its placed heading and the world corners of its first box (for
    /// tools).
    pub fn describe(&self, models: &Models, trig: &Trig) -> String {
        let (rot, pos) = self.pose(self.offset, trig);
        let corners = models.get(self.shape).and_then(|m| m.boxes.first().copied()).map(|(lo, hi)| {
            [(lo[0], lo[2]), (hi[0], lo[2]), (hi[0], hi[2]), (lo[0], hi[2])].map(|(x, z)| {
                let w = pos + rot * Vec3::new(x as f32, 0.0, z as f32);
                (w.x as i32, w.z as i32)
            })
        });
        format!("{} heading {} at {:?}: {:?}", self.template, self.heading, self.start, corners)
    }

    /// The door's pose for an offset or angle.
    pub(super) fn pose(&self, offset: [i32; 3], trig: &Trig) -> (Mat3, Vec3) {
        let start = Vec3::new(self.start[0] as f32, self.start[1] as f32, self.start[2] as f32);
        if self.hinged {
            let e = trig.euler([offset[0] as i16, 0, 0]);
            let cols: Vec<f32> = e.concat().into_iter().map(|v| v as f32 / 4096.0).collect();
            let turn = Mat3::from_cols_slice(&cols);
            (self.placed * turn, start)
        } else {
            let o = Vec3::new(offset[0] as f32, offset[1] as f32, offset[2] as f32);
            (self.placed, start - self.placed * o)
        }
    }
}

impl Game {
    /// Whether a door at offset `offset` would cut through the player, an
    /// NPC or a pickable (0x8006baa8, the box against their bodies).
    pub(super) fn door_blocked(&self, k: usize, offset: [i32; 3]) -> bool {
        let d = &self.doors[k];
        if self.set.templates.get(d.template as usize).is_some_and(|t| t.flags & 8 != 0) {
            return false;
        }
        let Some(model) = self.set.models.get(d.shape) else { return false };
        let (rot, pos) = d.pose(offset, &self.trig);
        let boxes = world_boxes(model, rot, pos);
        let p = self.player.pos;
        let (feet, head) = (p[1] - self.player.feet(), p[1] - self.player.feet() - HEIGHT);
        let body = |at: [i32; 3], r: i32, lo: i32, hi: i32| {
            boxes.iter().any(|b| b.covers(at, r) && b.min_y <= lo && b.max_y >= hi)
        };
        body(p, RADIUS, feet, head)
            || self.npcs.iter().filter(|n| !n.hidden).any(|n| {
                let f = n.pos[1] - n.feet();
                body(n.pos, 200, f, f - 600)
            })
            // Pickables lying in the way (+0x00 0x10000).
            || self.things.iter().filter(|t| t.state == ThingState::Rest && !t.hidden).any(|t| {
                let at = [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32];
                body(at, 100, at[1], at[1] - 200)
            })
    }

    /// The doors' thinks, one a game frame (0x8004f868, 0x8004e458).
    pub(super) fn door_thinks(&mut self) {
        let me = self.player.pos;
        for k in 0..self.doors.len() {
            // Class 8 and the automatic doors run while scheduled (their
            // trigger entries' enters and leaves, `door_entered`); one off
            // the list, while the player is within its radius.
            if let Some(s) = &mut self.doors[k].swing {
                s.hurt = (s.hurt - DT).max(0);
                let d = &self.doors[k];
                if self.trigger_active(Trigger::Placement(d.placement)).is_none() {
                    let (dx, dz) = ((d.start[0] - me[0]) as i64, (d.start[2] - me[2]) as i64);
                    let near = dx * dx + dz * dz < (self.doors[k].swing.as_ref().map_or(0, |s| s.radius) as i64).pow(2);
                    self.doors[k].moving = near;
                }
            }
            if !self.doors[k].moving {
                continue;
            }
            // The wait at an end (+0xd2, 136 a think).
            let hinged = self.doors[k].hinged;
            // A sliding door's think (0x8004e458) first takes dt off the
            // shared sound count (0x800c4ba0).
            if !hinged {
                self.shelf_sound -= DT;
            }
            if let Some(s) = &mut self.doors[k].swing
                && s.wait > 0
            {
                // Hinged waits count 136 a think, sliding ones 1.
                s.wait = (s.wait - if hinged { 136 } else { 1 }).max(0);
                continue;
            }
            // The easing: 256 + ease x sin(how far along, half a turn).
            if let Some(s) = &self.doors[k].swing {
                let d = &self.doors[k];
                let span = d.open_end - d.shut;
                if s.ease != 0 && span != 0 {
                    let along = ((d.offset[0] - d.shut) << 11) / span;
                    self.doors[k].speed[0] = 256 + s.ease * self.trig.sin(along) / 4096;
                }
            }
            let d = &self.doors[k];
            let mut next = d.offset;
            let stop;
            if d.hinged {
                // Step = trunc(speed x 136 / 4096); stop on passing the
                // target (to 0: a sign change), then snap.
                let step = d.speed[0] * 136 / 4096;
                let a = d.offset[0];
                let b = if a < d.target { a + step } else { a - step };
                stop = match d.target {
                    0 => (a >= 0) != (b >= 0),
                    t if t > 0 => b > t,
                    t => b < t,
                };
                next[0] = if stop { d.target } else { b };
            } else {
                // 2 x step a think toward the end, no snap.
                let sign = if d.open { 1 } else { -1 };
                for i in 0..3 {
                    next[i] += sign * 2 * d.speed[i];
                }
                let sum: i32 = next.iter().sum();
                stop = if d.open { sum >= d.shut } else { sum <= d.open_end };
            }
            if self.door_blocked(k, next) {
                // Level 16: message 24. A hurting door (class 8, +0x6d)
                // hurts every 51 VSyncs (25 frames).
                self.training.show(24);
                if let Some(s) = &mut self.doors[k].swing
                    && s.damage > 0
                    && s.hurt == 0
                    && !self.no_damage
                {
                    s.hurt = 250;
                    self.player.hurt(s.damage);
                }
                continue;
            }
            let d = &mut self.doors[k];
            d.offset = next;
            // Template 770: sound 136 at +0xaa 900 going shut, 135 at 1600
            // going open.
            if d.template == 770 {
                if d.open && next[1] == 900 {
                    self.effects.push(Effect::new(136));
                } else if !d.open && next[1] == 1600 {
                    self.effects.push(Effect::new(135));
                }
            }
            if stop {
                // A hinged close ends with sound 54 (not 1216, 672), at
                // the door (0x8004f868: 0x8006f644(54, door)).
                if d.hinged && d.open && !matches!(d.template, 1216 | 672) {
                    let at = self.set.placements[d.placement].pos;
                    self.play_at(54, at);
                }
                let d = &mut self.doors[k];
                d.open = !d.open;
                if let Some(s) = &mut d.swing {
                    // Back the other way; wait at an end its flags name
                    // (class 8: 4096 at 136 a think; sliding: 15 thinks,
                    // 8 after going shut, 16 after going open).
                    if d.hinged {
                        d.target = if d.target == d.open_end { d.shut } else { d.open_end };
                        if (s.waits & 8 != 0 && d.open) || (s.waits & 16 != 0 && !d.open) {
                            s.wait = 4096;
                        }
                    } else if (s.waits & 8 != 0 && !d.open) || (s.waits & 16 != 0 && d.open) {
                        s.wait = 15;
                    }
                } else {
                    d.moving = false;
                }
            }
        }
    }

    /// A door's trigger enter or leave, if placement `k` is a door that
    /// runs by them; whether its entry is in after. Class 8 (0x80049e18,
    /// 0x80049e5c): scheduled while in. An automatic door (template flag
    /// 0x40000000, enter 0x8004f6d8): scheduled on the enter, which leaves
    /// +0x00 bit 0 clear, so it is entered again and never left: it goes
    /// on cycling.
    pub(super) fn door_entered(&mut self, k: usize, entered: bool) -> Option<bool> {
        let d = self.doors.iter_mut().find(|d| d.placement == k && d.swing.is_some())?;
        let automatic = !d.hinged;
        if entered {
            d.moving = true;
        } else if !automatic {
            d.moving = false;
        }
        Some(entered && !automatic)
    }

    /// Whether door `k` is touched: the last move's probes met it (+0xf4,
    /// 0x80063d3c), as a run's reach along the heading does from where it
    /// stopped; or the body is at it.
    fn door_touched(&self, k: usize) -> bool {
        let p = self.player.pos;
        let feet = p[1] - self.player.feet();
        let d = &self.doors[k];
        self.set.models.get(d.shape).is_some_and(|m| {
            let (rot, pos) = d.pose(d.offset, &self.trig);
            let a = self.player.heading as f32 * std::f32::consts::TAU / 4096.0;
            let probes = self.player.probes([a.sin(), a.cos()], 170.0);
            world_boxes(m, rot, pos).iter().any(|b| {
                let h = self.player.probe_height(b, p);
                (b.covers(p, RADIUS + 150) && b.min_y <= feet && b.max_y >= feet - HEIGHT)
                    || probes.iter().any(|&(dx, dz)| b.segment_hit([p[0], h, p[2]], [p[0] + dx, h, p[2] + dz]).is_some())
            })
        })
    }

    /// Triangle on the door Tommy is touching (0x8004f4a0, 0x8004f6d8):
    /// refused while it moves; a hinged door by the player's facing.
    pub(super) fn use_door(&mut self) -> bool {
        let touched = (0..self.doors.len()).find(|&k| {
            let d = &self.doors[k];
            d.swing.is_none() && !self.unusable.contains(&d.placement) && self.door_touched(k)
        });
        let Some(k) = touched else { return false };
        let level16 = self.level == Level::TRAINING;
        let hp = self.player.heading;
        let d = &mut self.doors[k];
        if d.moving {
            return true;
        }
        if d.hinged {
            // local_18 = 4096 cos(h_p - h_d), h_d the door's heading now.
            let facing = self.trig.cos(hp - (d.heading + d.offset[0]));
            let a = d.offset[0];
            if !d.open {
                if d.either {
                    d.target = if facing > 0 { d.open_end } else { -d.open_end };
                } else if facing >= 0 {
                    return true;
                } else {
                    d.target = d.open_end;
                }
                if d.sound == 1 {
                    self.effects.push(Effect::new(if level16 { 170 } else { 14 }));
                }
            } else {
                let ok = a != 0
                    && facing != 0
                    && if d.open_end > 0 { (a > 0) != (facing > 0) } else { (a > 0) == (facing > 0) };
                if !ok {
                    return true;
                }
                d.target = d.shut;
                if d.sound == 1 {
                    self.effects.push(Effect::new(if level16 { 170 } else { 15 }));
                }
            }
        } else if d.sound == 2 {
            self.effects.push(Effect::new(if d.open { 17 } else { 16 }));
        }
        d.moving = true;
        true
    }
}
