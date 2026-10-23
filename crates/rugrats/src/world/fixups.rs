//! What the object set-up (SLUS_006.50:0x8004691c) changes as it makes a
//! level's objects, beyond the object itself: the shared templates (the
//! table at 0x800c4300, loaded once at boot) and models (+0x88's boxes),
//! some objects' places, templates and ids, copies of some objects (most
//! only blocking, their draw +0x40 cleared), and sound sources
//! (0x8006fe04). Every object of a loaded placement runs it, in the LP's
//! order; the case on the level is its switch at 0x80046e5c.

use rugrats_data::placement::{Placement, Template};

use crate::media::cine::Set;

/// A sound source (0x8006fe04: a 40-byte record at 0x800e3218): the
/// effect, its place, the distance it is heard full and the one it fades
/// out by, and its volume; `random`: its sound drawn at load, the game's
/// rand % 3 (template 1178).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundSource {
    pub sound: u8,
    pub pos: [i32; 3],
    pub near: i32,
    pub far: i32,
    pub volume: u8,
    pub random: bool,
}

/// The template's sound or line word (+0x24) a pickable gets when it has
/// none, and the ids the loader starts level 5's TV robots at
/// (0x800c4170, set by 0x800454dc).
const PICKABLE_LINE: u16 = 19;
const ROBOT_IDS: u16 = 0x32c8;

fn word(t: &Template, at: usize) -> u16 {
    u16::from_le_bytes([t.raw[at], t.raw[at + 1]])
}

fn set_word(t: &mut Template, at: usize, v: i16) {
    t.raw[at..at + 2].copy_from_slice(&v.to_le_bytes());
    t.flags = u32::from_le_bytes(t.raw[..4].try_into().unwrap());
}

fn set_flags(t: &mut Template, f: impl Fn(u32) -> u32) {
    t.flags = f(t.flags);
    t.raw[..4].copy_from_slice(&t.flags.to_le_bytes());
}

/// A yaw (+0x24) as the placements' quaternion holds it: the turns the
/// copies take are 0 and 1024.
fn yaw(a: i16) -> [i32; 4] {
    if a == 1024 { [0, 2896, 0, 2896] } else { [0, 0, 0, 4096] }
}

impl Set {
    /// The room holding `p` (0x8002b660 with no hint: the first whose
    /// bounds do).
    fn room_at(&self, p: [i32; 3]) -> Option<usize> {
        (0..self.room_info.len()).find(|&k| {
            let r = &self.room_info[k];
            (0..3).all(|i| r.min[i] <= p[i] && p[i] <= r.max[i])
        })
    }

    /// Template `t`'s model's box `b`, to change.
    fn model_box(&mut self, t: u32, b: usize) -> Option<&mut ([i16; 3], [i16; 3])> {
        let shape = self.templates.get(t as usize)?.shape as u32;
        self.models.get_mut(shape)?.boxes.get_mut(b)
    }

    /// Applies the object set-up's changes for `level` (0x800c5678).
    pub(crate) fn fix_up(&mut self, level: u8) {
        let mut robot_id = ROBOT_IDS;
        let mut copies: Vec<Placement> = Vec::new();
        let mut sizes = vec![0; self.placements.len()];
        let mut copy_sizes = Vec::new();
        for k in 0..self.placements.len() {
            let p = self.placements[k];
            let copied = copies.len();
            // An LP kind 3 record is a sound source (the loader, 0x800454dc
            // at 0x80045b24): its sound the template word's high half,
            // near and far the rotation's first two words, its volume the
            // third's low half.
            if p.kind == 3 {
                self.sound_sources.push(SoundSource {
                    sound: (p.template >> 16) as u8,
                    pos: p.pos,
                    near: p.rotation[0],
                    far: p.rotation[1],
                    volume: p.rotation[2] as u8,
                    random: false,
                });
                continue;
            }
            if p.kind != 1 {
                continue;
            }
            let Some(t) = self.templates.get(p.template as usize).copied() else { continue };
            let template = p.template;
            // A collectible's set-up (0x80048a44) makes its model's first
            // box the cube of 375 (kind 12's, model 0, which it takes) but
            // for kinds 0 and 5, then sizes it again: the model shared.
            let goodie_kind = word(&t, 0x26) as u16;
            let mut shape = t.shape as u32;
            if t.raw[4] == 12 && t.hierarchy().is_none() && !matches!(goodie_kind, 0 | 5) {
                if goodie_kind == 12 {
                    shape = 0;
                }
                if let Some(b) = self.models.get_mut(shape).and_then(|m| m.boxes.get_mut(0)) {
                    *b = ([-375; 3], [375; 3]);
                }
            }
            // Its size, before the changes below to its model's boxes
            // (0x8004691c: 0x80046570 first).
            sizes[k] = self.models.get(shape).map_or(0, |m| crate::world::boxhit::bound_size(&m.boxes));
            // The object's flags: the LP's, then the template's pickable
            // (+0x28), for one with a model (template flag 1 clear).
            let mut flags = p.flags;
            if t.flags & 1 == 0 && word(&t, 0x28) != 0 {
                flags |= 0x10000;
            }
            // A pickable whose template has no +0x24 gives it 19; 1164
            // gets 20.
            if word(&t, 0x24) == 0 && flags & 0x10000 != 0 {
                set_word(&mut self.templates[template as usize], 0x24, PICKABLE_LINE as i16);
            }
            if template == 1164 {
                set_word(&mut self.templates[template as usize], 0x24, 20);
            }
            // 402 and 948: a use of their own (0x8004d53c), not pickable.
            if matches!(template, 402 | 948) {
                flags &= !0x10000;
            }
            // 527 and 125: posed parts, and no object test (template 8).
            let no_test = |set: &mut Set, flags: &mut u32| {
                *flags |= 0x2000;
                set_flags(&mut set.templates[template as usize], |f| f | 8);
            };
            if matches!(template, 527 | 125) {
                no_test(self, &mut flags);
            }
            let room = self.room_at(p.pos);
            let mut pos = p.pos;
            let mut id = p.id;
            let mut new_template = template;
            let not_climbed = |set: &mut Set| set_flags(&mut set.templates[template as usize], |f| f & !64);
            // A copy at `at`, turned `turn`, drawn or not.
            let mut copy = |at: [i32; 3], turn: i16, drawn: bool| {
                copies.push(Placement { pos: at, rotation: yaw(turn), drawn, ..p });
            };
            // The room 1 movers of levels 1, 10 and 13 (0x800479bc,
            // 0x80047a08).
            let room_1 = |pos: &mut [i32; 3]| match template {
                112 if room == Some(1) => pos[0] = 0x4268,
                132 if room == Some(1) => pos[0] = 0x4e20,
                _ => {}
            };
            match level {
                1 => {
                    match template {
                        473 => id = 519,
                        381 => {
                            if let Some(b) = self.model_box(template, 4) {
                                b.0[1] = -200;
                            }
                            not_climbed(self);
                        }
                        1244 => no_test(self, &mut flags),
                        248 => {
                            not_climbed(self);
                            if let Some(b) = self.model_box(template, 0) {
                                b.0[1] = -3000;
                            }
                        }
                        _ => {}
                    }
                    room_1(&mut pos);
                }
                3 => match template {
                    883 if p.pos[0] == 0x1bd56 => {
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[2] = -2000;
                        }
                    }
                    336 if p.pos == [0, -28, 0] => {
                        copy([2368, -28, -14250], 1024, false);
                        copy([0x5386, -500, 0x1abb], 1024, false);
                        copy([0x5528, -200, 0x1abb], 1024, false);
                    }
                    98 | 674 => pos[2] = -16000,
                    // (Only its +0x96 becomes 623, its model and calls
                    // already 100's: 100's record with 623's flags, not
                    // climbed.)
                    100 if p.pos == [2549, 516, -13770] || p.pos == [2547, 516, -14810] => new_template = 623,
                    624 => copy([0x16f97, p.pos[1], p.pos[2]], 0, true),
                    _ => {}
                },
                4 => match template {
                    203 if p.pos[0] == -226738 && p.pos[2] == 0x1334a => pos[1] = -400,
                    543 if p.pos == [-244131, -1047, 0x17106] => {
                        let turn = self.placements[k].rotation;
                        copies.push(Placement { pos: [-281957, -2313, 0xfce3], rotation: turn, ..p });
                        copies.push(Placement { pos: [-280950, -2318, 0xfa2c], rotation: turn, ..p });
                    }
                    1189 => no_test(self, &mut flags),
                    425 | 1185 => {
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[1] = -666;
                        }
                    }
                    653 | 1182 | 1183 | 1184 => {
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[1] = -450;
                        }
                    }
                    _ => {}
                },
                5 => match template {
                    826 => {
                        id = robot_id;
                        robot_id += 1;
                    }
                    1231 => id = 524,
                    855 => set_word(&mut self.templates[template as usize], 0x1a, 60),
                    _ => {}
                },
                6 => match template {
                    1028 => {
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[1] = -1000;
                        }
                    }
                    900 => pos[1] = -1750,
                    _ => {}
                },
                7 => match template {
                    1138 => copy([52000, -279, -15500], 0, true),
                    1128 | 795 => flags |= 0x10000,
                    687 => no_test(self, &mut flags),
                    _ => {}
                },
                10 => {
                    match template {
                        126 => copy([0x3c19, 512, 800], 0, false),
                        527 if p.pos[0] == 0x3b50 && p.pos[2] == -135 => pos[0] = 0x3f38,
                        381 => {
                            if let Some(b) = self.model_box(template, 4) {
                                b.0[1] = -200;
                            }
                            not_climbed(self);
                        }
                        1244 | 61 => no_test(self, &mut flags),
                        248 => {
                            not_climbed(self);
                            if let Some(b) = self.model_box(template, 0) {
                                b.0[1] = -3000;
                            }
                        }
                        _ => {}
                    }
                    room_1(&mut pos);
                }
                11 => {
                    if template == 204 {
                        copy([0x38f9, 950, 0xbe4b], 0, true);
                        copy([0x35f7, 852, 0xcaf8], 0, true);
                    }
                }
                12 => match template {
                    1070 => {
                        if let Some(b) = self.model_box(template, 1) {
                            *b = ([-900, -339, -1200], [1500, 1463, 1200]);
                        }
                    }
                    381 => {
                        if let Some(b) = self.model_box(template, 4) {
                            b.0[1] = -200;
                        }
                        not_climbed(self);
                    }
                    1271 => {
                        copy([0x558c, -2380, p.pos[2]], 0, true);
                        copy([0x3b60, -5550, 161], 0, true);
                    }
                    960 => {
                        not_climbed(self);
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[1] = -3000;
                        }
                    }
                    954 => set_word(&mut self.templates[template as usize], 0x24, 20),
                    678 => set_word(&mut self.templates[template as usize], 0x24, 167),
                    _ => {}
                },
                13 => room_1(&mut pos),
                15 => match template {
                    223 => {
                        for x in [0x4a9c, 0x5078, 0x5654, 0x5c30, 0x620c, 0x65f4] {
                            copy([x, 2443, 0x1a90], 0, false);
                        }
                    }
                    248 => {
                        not_climbed(self);
                        copy([p.pos[0], 2443, p.pos[2]], 0, false);
                        copy([0x1a90, 2443, p.pos[2]], 0, false);
                    }
                    54 => {
                        copy([p.pos[0], p.pos[1], 3000], 0, false);
                        copy([0x55f0, p.pos[1], 3000], 0, false);
                    }
                    _ => {}
                },
                16 => match template {
                    381 => {
                        if let Some(b) = self.model_box(template, 4) {
                            b.0[1] = -200;
                        }
                        not_climbed(self);
                    }
                    1164 => set_word(&mut self.templates[template as usize], 0x24, 175),
                    _ => {}
                },
                18 => {
                    if template == 248 {
                        not_climbed(self);
                        copy([p.pos[0], -800, p.pos[2]], 0, false);
                    }
                }
                20 => {
                    if template == 27 && p.pos == [0x73f9, 152, 3088] {
                        pos[2] = 0x1400;
                    }
                }
                23 => {
                    if template == 1264 {
                        let shape = self.templates[template as usize].shape as u32;
                        if let Some(m) = self.models.get_mut(shape) {
                            m.boxes.truncate(1);
                            if let Some(b) = m.boxes.first_mut() {
                                *b = ([-4000, -2000, -1000], [4000, 500, 1000]);
                            }
                        }
                    }
                }
                24 => {
                    if template == 770 {
                        set_word(&mut self.templates[template as usize], 0x1a, 50);
                    }
                }
                27 => {
                    if matches!(template, 232 | 712)
                        && let Some(b) = self.model_box(template, 0)
                    {
                        b.0[2] = -3100;
                    }
                }
                28 => match template {
                    1191 => pos[1] = 0,
                    1168 => {
                        if let Some(b) = self.model_box(template, 0) {
                            b.0[0] = -2000;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            // The sound sources it starts (0x8006fe04: sound, place, the
            // two distances, the volume).
            let source = |sound: u8, near: i32, far: i32, volume: u8| SoundSource { sound, pos, near, far, volume, random: false };
            let started = match (level, template) {
                (3, 91) => Some(source(4, 1024, 4000, 127)),
                (3, 614) => Some(source(5, 993, 4000, 64)),
                (16, 227) => Some(source(0, 993, 2990, 64)),
                (23, 657) => Some(source(0, 1024, 10000, 127)),
                (28, 1191) => Some(source(0, 1024, 2000, 127)),
                (28, 920) => Some(source(1, 1024, 2000, 127)),
                (28, 535) => Some(source(2, 1024, 2000, 127)),
                (31, 789) => Some(source(0, 1024, 4000, 127)),
                (_, 1178) => Some(SoundSource { random: true, ..source(0, 1024, 4000, 127) }),
                _ => None,
            };
            self.sound_sources.extend(started);
            if new_template != template {
                let mut renamed = self.templates[template as usize];
                let flags_of = self.templates.get(new_template as usize).map_or(renamed.flags, |t| t.flags);
                set_flags(&mut renamed, |_| flags_of);
                new_template = match self.templates.iter().position(|t| *t == renamed) {
                    Some(at) => at as u32,
                    None => {
                        self.templates.push(renamed);
                        (self.templates.len() - 1) as u32
                    }
                };
            }
            let q = &mut self.placements[k];
            q.pos = pos;
            q.id = id;
            q.template = new_template;
            q.flags = flags;
            // (A copy is the object copied whole, its size and all.)
            copy_sizes.extend(std::iter::repeat_n(sizes[k], copies.len() - copied));
        }
        self.placements.extend(copies);
        sizes.extend(copy_sizes);
        self.sizes = sizes;
    }
}
