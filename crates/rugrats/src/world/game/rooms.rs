//! Rooms: which one the player is in, what entering one does, and the
//! placements that speak when the player comes near.

use super::*;

/// The placements with a speech line (LP +0x4c; template 613 says 194,
/// 0x80046d04), on the trigger list at their template's radius (+0x34).
/// The hub's doors lose theirs to the sign's enter (0x800481c8).
pub(super) fn talkers(set: &Set, level: Level) -> Vec<Talker> {
    set.placements
        .iter()
        .enumerate()
        .filter(|(_, p)| p.kind == 1 && !(level == Level::HUB && p.level > 0))
        .filter_map(|(k, p)| {
            let line = if p.template == 613 { 194 } else { p.speech };
            (line != 0).then_some(Talker { placement: k, line })
        })
        .collect()
}

/// A placement that speaks when the trigger pass enters it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Talker {
    pub(super) placement: usize,
    line: u16,
}

impl Game {
    /// Sets the player's room without entering it (a level moving the
    /// player).
    pub fn set_room(&mut self, room: Option<usize>) {
        self.room = room;
    }

    /// The room holding `p` (0x8002b660, by the rooms' bounds).
    /// The room lookup (0x8002b660): `hint` if its bounds hold `p`, else
    /// the first of its portals' rooms that does, else the first room.
    pub fn room_from(&self, p: [i32; 3], hint: Option<usize>) -> Option<usize> {
        let inside =
            |k: usize| self.set.room_info.get(k).is_some_and(|r| (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i]));
        let Some(h) = hint.filter(|&h| h < self.set.room_info.len()) else { return self.room_of(p) };
        if inside(h) {
            return Some(h);
        }
        self.set.neighbours[h].iter().copied().find(|&k| inside(k)).or_else(|| self.room_of(p))
    }

    /// Each character's room, kept up (see `npc_rooms`).
    pub(super) fn track_npc_rooms(&mut self) {
        self.npc_rooms.resize(self.npcs.len(), None);
        for k in 0..self.npcs.len() {
            let found = self.room_from(self.npcs[k].pos, self.npc_rooms[k]);
            if found.is_some() {
                self.npc_rooms[k] = found;
            }
        }
    }

    pub fn room_of(&self, p: [i32; 3]) -> Option<usize> {
        (0..self.set.room_info.len()).find(|&k| {
            let r = &self.set.room_info[k];
            (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i])
        })
    }

    /// A talker's enter (0x8004d0f0): with the player in its room it is
    /// in and says its line; else it stays out, to be entered again.
    pub(super) fn talk(&mut self, t: usize) -> bool {
        let Talker { placement, line, .. } = self.talkers[t];
        let d = &self.set.placements[placement];
        if self.hidden.contains(&d.id) || self.room.is_none() || self.room_of(d.pos) != self.room {
            return false;
        }
        self.speech.push(line);
        true
    }

    /// The player's room (0x8002b660, here by the rooms' bounds, the one
    /// it was in first); on a change, the room's song (0x80077a54).
    pub(super) fn room_change(&mut self) {
        let room = self.room_from(self.player.pos, self.room);
        if room == self.room {
            return;
        }
        self.room = room;
        let Some(k) = room else { return };
        self.with_script(|s, g| s.room(g, k));
        let r = self.set.room_info[k];
        // Flag 2: the room's cinematic, once (PostCine, then cleared).
        if r.flags & 2 != 0 && r.cine != 0 {
            self.set.room_info[k].flags &= !2;
            self.cine = Some(r.cine as usize);
        }
        // Flag 32: a checkpoint (0x800772e0): where a cry sends the
        // player back to.
        if r.flags & 32 != 0 {
            self.checkpoint = Some(self.player.pose());
        }
        // Flag 128: +0x64 below 5 a song, switched to over 30 ticks
        // (0x8006efcc), else a speech line.
        if r.flags & 128 != 0 && (1..5).contains(&r.song) {
            self.room_song = Some(r.song as usize - 1);
        } else if r.flags & 128 != 0 && r.song >= 5 {
            self.speech.push(r.song as u16);
        }
        // Flag 4: the room's camera (CameraSwitch, 0x80077c88 into
        // 0x800291f8): kind 3 a fixed camera at post +0x5c; the follow
        // kinds set its height (6: 200, 7: 500, 8: 1600, 9: 2000, 11:
        // 500; 1500 means the level's own).
        let floating = self.level == Level::OUTER_SPACE && self.player.float.is_some();
        if r.flags & 4 != 0 && floating && r.camera == 7 {
            // Level 5 floating: kind 7 becomes 13, the camera holds where
            // it is (0x800c4a50 = 1; rooms 73-75).
            self.cam_hold = true;
        } else if r.flags & 4 != 0 && self.level == Level::OUTER_SPACE && r.camera == 3 {
            // Level 5's kind 3 (0x800293c0): floating, only while held, a
            // fixed post by room; walking, the shaft camera (rooms 15, 40).
            if floating {
                if self.cam_hold {
                    let post = match k {
                        60 => 5,
                        62 => 3,
                        68 => 8,
                        70 => 2,
                        71 => 4,
                        72 => 6,
                        _ => 1,
                    };
                    self.post_camera = Some(post);
                }
            } else {
                self.post_camera = None;
                self.shaft = true;
            }
        } else if r.flags & 4 != 0
            && (!matches!(self.level, Level::NO_MORE_COOKIES | Level::TOUCHDOWN_TOMMY) || r.camera == 3)
        {
            // The follow setup lets go of a level 5 hold (0x80025e90).
            if r.camera != 3 {
                self.cam_hold = false;
                self.shaft = false;
            }
            match r.camera {
                3 => self.post_camera = Some(r.camera_post as usize),
                1 | 6..=13 => {
                    self.post_camera = None;
                    // Level 7 keeps -1000 for every switch (0x80029288).
                    self.eye_up = if self.level == Level::REPTAR_2010 {
                        default_eye_up(Level::REPTAR_2010)
                    } else {
                        match r.camera {
                            6 => 200,
                            7 | 11 => 500,
                            8 => 1600,
                            9 => 2000,
                            _ => default_eye_up(self.level),
                        }
                    };
                }
                _ => {}
            }
        }
        // Flag 64: the level's Gameplay script, once (0x80077ee4); level
        // 16's room keeps it (0x800789e4 never sets +0x11).
        if r.flags & 64 != 0 {
            if self.level != Level::TRAINING {
                self.set.room_info[k].flags &= !64;
            }
            self.with_script(|s, g| s.gameplay(g, r.script));
        }
    }
}
