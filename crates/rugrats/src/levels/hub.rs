//! Level 1, Pickles' home, the hub (docs/levels/level-01.md): the doors
//! to the levels with their signs, and Spike in the yard, who follows
//! Tommy on a leash, barks and fetches the toys he throws. Nothing hurts
//! here.

use rrt::glam::Vec3;
use rugrats_render::Vtx;

use crate::actors::npc::{Mode, Target};
use crate::actors::player::DT;
use crate::disc::GameDisc;
use crate::input::Pads;
use crate::levels::Level;
use crate::ui::front;
use crate::ui::text::Text;
use crate::world::collide;
use crate::world::game::{Game, Script, ThingState};

/// Hidden at every start: cine 7's end hook (0x800732d4 case 7, also
/// when the cine is skipped) and EpisodeStart (0x80078e7c).
const HIDDEN: [u16; 17] = [127, 1776, 608, 609, 514, 56, 57, 58, 59, 55, 5555, 350, 351, 352, 353, 354, 355];
/// Spike (object 14): his home in the yard, the waking distance, the
/// leash (2 x range), his reach for a toy.
const SPIKE: u16 = 14;
const HOME: [i32; 3] = [47762, 727, 32948];
const WAKE: i64 = 30000;
/// Lying down (0x80083f6c: animation 26).
const ASLEEP: i16 = 26;
/// Getting up (0x80083f24: animation 25, mode 1 then back mode 2) and the
/// flinch when a toy hits him (0x800511a8 for char 6: animation 8).
const WAKE_UP: i16 = 25;
const FLINCH: i16 = 8;
/// The mouth (hierarchy slot 0x24) and what it shows (0x800601e4 for
/// char 6): his own model 79, or toy 678's 1212, 679's 1214, 680's 1213.
const MOUTH_SLOT: usize = 0x24 - 0x14;
const MOUTH: u32 = 79;
const MOUTH_TOYS: [(u16, u32); 3] = [(678, 1212), (679, 1214), (680, 1213)];
const LEASH: i64 = 12000;
const REACH: i64 = 1024;
/// The toys he fetches (templates 678-680) and the lines: the throw
/// (191 / 192), the pick-up (0x800601e4), the put-down (0x80062ac4).
const TOYS: [u16; 3] = [678, 679, 680];
/// The can, which can be thrown for him but which he leaves.
const CAN: u16 = 716;
const PICK_UP: [u16; 4] = [193, 159, 234, 235];
const PUT_DOWN: u16 = 190;
/// The barks (0x800b96ec, 11) in rooms 24 and 39-41.
const BARKS: u32 = 0x800b_96ec;
/// Mode 14 (giving up) in the back doorways.
const DOORWAYS: [usize; 2] = [35, 36];
/// Door signs (0x80049eec): within 2500 (2D) in the player's room; the
/// mirror only with +0x30 bit 16.
const SIGN_REACH: i64 = 2500;
const MIRRORS: [u32; 2] = [836, 881];
const HARD: [Level; 3] = [Level::AISLE_SEVEN, Level::CYNTHIA, Level::TOY_PALACE];
const MEDIUM: [Level; 4] = [Level::GRANDPAS_TEETH, Level::OUTER_SPACE, Level::CIRCUS, Level::LET_THERE_BE_LIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spike {
    Asleep,
    /// Mode 2.
    Following,
    /// Mode 12: after the toy (thing index).
    Fetching(usize),
    /// Mode 13: bringing it back.
    Bringing(usize),
}

pub struct Hub {
    spike: Spike,
    barks: Vec<u16>,
    bark_wait: i32,
    /// The toy in Tommy's hand last frame.
    held: Option<usize>,
    /// His range (template +0x1a), kept while a fetch shrinks it.
    range: i32,
    /// The sign texts: level names by level, and Easy / Medium / Hard.
    names: Vec<String>,
    words: [String; 3],
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

fn dist2(a: [i32; 3], b: [i32; 3]) -> i64 {
    let (dx, dz) = ((a[0] - b[0]) as i64, (a[2] - b[2]) as i64);
    dx * dx + dz * dz
}

impl Hub {
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<Hub, String> {
        for id in HIDDEN {
            g.hide(id, true);
        }
        // Cine 7's end hook puts object 80 (the door to 6) at y 746
        // (0x800732d4 case 7); EpisodeStart makes object 518 template
        // 1130 (0x80078e7c).
        g.edit_placements(80, |p| p.pos[1] = 746);
        g.edit_placements(518, |p| p.template = 1130);
        g.no_damage = true;
        g.player.falls_hurt = false;
        if let Some(k) = find(g, SPIKE) {
            let n = &mut g.npcs[k];
            n.home = HOME;
            n.place(HOME, &g.col);
            n.mode = Mode::Idle;
            n.attacks = false;
            n.hold = Some(ASLEEP);
            // The only one EpisodeStart schedules (0x80078e7c).
            n.scheduled = true;
        }
        let barks = disc
            .exe_bytes(BARKS, 22)
            .unwrap_or(&[])
            .as_chunks::<2>()
            .0
            .iter()
            .map(|w| u16::from_le_bytes([w[0], w[1]]))
            .collect();
        let s = |n: u32| front::string(disc, n);
        let names = (0..32).map(|l| s(if l == 25 { 158 } else { 150 + l })).collect();
        let range = find(g, SPIKE).map_or(6000, |k| g.npcs[k].range);
        Ok(Hub { spike: Spike::Asleep, barks, bark_wait: 0, held: None, range, names, words: [s(221), s(220), s(222)] })
    }

    fn spike(&mut self, g: &mut Game) {
        let Some(k) = find(g, SPIKE) else { return };
        let me = g.player.pos;
        let at = g.npcs[k].pos;
        // Asleep beyond 30000, awake within (0x80083f6c, 0x80083f24).
        if dist2(me, at) > WAKE * WAKE {
            if let Spike::Bringing(t) = self.spike {
                self.put_down(g, k, t, false);
            }
            self.spike = Spike::Asleep;
            g.npcs[k].mode = Mode::Idle;
            g.npcs[k].hold = Some(ASLEEP);
            return;
        }
        if self.spike == Spike::Asleep {
            // He gets up (animation 25), then follows.
            self.spike = Spike::Following;
            g.npcs[k].hold = Some(WAKE_UP);
            g.npcs[k].animator.start(WAKE_UP, &g.anims);
            g.npcs[k].mode = Mode::Idle;
        }
        // A one-shot animation (getting up, a flinch) plays out first.
        if let Some(a @ (WAKE_UP | FLINCH)) = g.npcs[k].hold {
            if g.npcs[k].animator.playing(a) {
                g.npcs[k].mode = Mode::Idle;
                return;
            }
            g.npcs[k].hold = None;
        }
        // A toy thrown (0x80060b44): 191 or 192; a following Spike runs
        // after it.
        let holding = g.carried().filter(|&t| TOYS.contains(&g.things[t].template) || g.things[t].template == CAN);
        if let (Some(t), None) = (self.held, holding)
            && matches!(g.things[t].state, ThingState::Flying(_))
        {
            g.speech.push(191 + (g.rand.next() & 1) as u16);
            if self.spike == Spike::Following {
                self.spike = Spike::Fetching(t);
                g.npcs[k].speed_index = 4;
                // Mode 12 runs on to the toy (the follow stops at
                // range / 2).
                self.range = g.npcs[k].range;
                g.npcs[k].range = REACH as i32;
            }
        }
        self.held = holding;
        let room = g.current_room();
        match self.spike {
            Spike::Asleep => {}
            Spike::Following => {
                // Mode 2 (0x800854b4): home past the leash, Tommy within
                // it.
                let n = &mut g.npcs[k];
                n.mode = if dist2(at, HOME) > LEASH * LEASH {
                    Mode::Follow(Target::Point(HOME))
                } else if dist2(me, HOME) < LEASH * LEASH {
                    Mode::Follow(Target::Player)
                } else {
                    Mode::Idle
                };
                // The barks (0x800c4ce4) in his corner of the yard.
                self.bark_wait -= DT;
                if self.bark_wait <= 0 && matches!(room, Some(24 | 39..=41)) && !self.barks.is_empty() {
                    let r = (g.rand.next() & 255) as usize;
                    g.speech.push(self.barks[r % self.barks.len()]);
                    self.bark_wait = ((g.rand.next() & 255) * 8) as i32;
                }
            }
            Spike::Fetching(t) => {
                // Mode 12 (0x80084cd0): to the toy; in reach, and not in
                // Tommy's hand, he takes it.
                let toy = &g.things[t];
                let p = [toy.pos.x as i32, toy.pos.y as i32, toy.pos.z as i32];
                if g.carried() == Some(t) || toy.hidden {
                    self.give_up(g, k);
                } else if dist2(at, p) < REACH * REACH && toy.state == ThingState::Rest {
                    // The can (716) is never taken (0x80084cd0).
                    if toy.template == CAN {
                        self.give_up(g, k);
                        return;
                    }
                    let model = MOUTH_TOYS.iter().find(|m| m.0 == toy.template).map_or(MOUTH, |m| m.1);
                    mouth(g, k, model);
                    g.things[t].hidden = true;
                    g.speech.push(PICK_UP[(g.rand.next() & 3) as usize]);
                    g.npcs[k].range = self.range;
                    g.npcs[k].mode = Mode::Follow(Target::Player);
                    self.spike = Spike::Bringing(t);
                } else {
                    g.npcs[k].mode = Mode::Follow(Target::Point(p));
                }
            }
            Spike::Bringing(t) => {
                // Mode 13: back to Tommy; within range / 2 he puts it down.
                let n = &g.npcs[k];
                if dist2(at, me) <= (n.range as i64 / 2).pow(2) {
                    self.put_down(g, k, t, true);
                }
            }
        }
        // Mode 14 (0x80084cf4): giving up in the back doorways, or on
        // bumping into a wall (state 9 for char 6 in level 1, 0x8005b614).
        let bumped = g.npcs[k].bumped;
        if matches!(self.spike, Spike::Fetching(_) | Spike::Bringing(_))
            && (room.is_some_and(|r| DOORWAYS.contains(&r)) || bumped)
        {
            if let Spike::Bringing(t) = self.spike {
                self.put_down(g, k, t, false);
            }
            self.give_up(g, k);
        }
    }

    fn give_up(&mut self, g: &mut Game, k: usize) {
        self.spike = Spike::Following;
        g.npcs[k].speed_index = 2;
        g.npcs[k].range = self.range;
    }

    /// 0x80062ac4 for Spike: the toy 1024 ahead of him (where he stands
    /// when giving up), speech 190.
    fn put_down(&mut self, g: &mut Game, k: usize, t: usize, ahead: bool) {
        let n = &g.npcs[k];
        let d = if ahead { 1024 } else { 0 };
        let (s, c) = (g.trig.sin(n.heading), g.trig.cos(n.heading));
        let at = [n.pos[0] + ((s * d) >> 12), n.pos[1], n.pos[2] + ((c * d) >> 12)];
        let q = collide::query(&g.col, [at[0], at[1] - 400, at[2]]);
        let y = if q.tile.is_some() && q.kind == 0 { q.floor } else { at[1] };
        let toy = &mut g.things[t];
        toy.pos = Vec3::new(at[0] as f32, y as f32, at[2] as f32);
        toy.hidden = false;
        toy.state = ThingState::Rest;
        toy.flown = None;
        mouth(g, k, MOUTH);
        g.speech.push(PUT_DOWN);
        self.give_up(g, k);
    }
}

/// The model Spike's mouth part shows.
fn mouth(g: &mut Game, k: usize, model: u32) {
    let n = &mut g.npcs[k];
    if let Some(&part) = n.hierarchy.header.get(0x14 + MOUTH_SLOT)
        && let Some(p) = n.hierarchy.parts.get_mut(part as usize)
    {
        p.model = model;
    }
}

/// The signs showing (0x80049eec): placement, its level, where it is.
/// Within 2500 (2D) in the player's room; the mirror only with +0x30 bit
/// 16.
fn signs(g: &Game) -> Vec<(usize, Level, [i32; 3])> {
    let me = g.player.pos;
    let room = g.current_room();
    let flag16 = g.flags & 16 != 0;
    let mut out = Vec::new();
    for (k, d) in g.set.placements.iter().enumerate() {
        let mirror = MIRRORS.contains(&d.template);
        if d.kind != 1 || (!mirror && d.level <= 0) || (mirror && !flag16) || g.is_hidden(d.id) {
            continue;
        }
        let level = if mirror { Level::MIRRORLAND } else { Level(d.level as u8) };
        let pos = g
            .things
            .iter()
            .find(|t| t.placement == k)
            .map_or(d.pos, |t| [t.pos.x as i32, t.pos.y as i32, t.pos.z as i32]);
        if dist2(pos, me) < SIGN_REACH * SIGN_REACH && room.is_some() && g.room_of(pos) == room {
            out.push((k, level, pos));
        }
    }
    out
}

/// An object's +0x6d (template +0x32).
fn sign_damage(g: &Game, k: usize) -> i16 {
    let t = g.set.placements[k].template as usize;
    g.set.templates.get(t).map_or(0, |t| t.raw[0x32] as i16)
}

impl Script for Hub {
    fn frame(&mut self, g: &mut Game, _pads: &Pads) {
        g.npc_events.clear();
        // A toy that hits Spike (0x800511a8, char 6): a flinch, then on.
        if let Some(k) = find(g, SPIKE)
            && g.struck.iter().any(|s| s.npc == k)
            && self.spike != Spike::Asleep
        {
            g.npcs[k].hold = Some(FLINCH);
            g.npcs[k].animator.start(FLINCH, &g.anims);
        }
        g.struck.clear();
        self.spike(g);
        // Each sign that shows sparkles (0x80049eec's 0x80050fd0, its
        // sound off).
        for (k, _, at) in signs(g) {
            let damage = sign_damage(g, k);
            g.flights.sign_sparks(at, damage, g.sprites.as_ref());
        }
    }

    /// The door signs (0x80049eec): Easy / Medium / Hard at y 62 and the
    /// level's name at y 80, font 0 centred across; the name cyan once
    /// the level is won.
    fn draw(&self, g: &Game, _moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        let mut drawn = Vec::new();
        for (_, level, _) in signs(g) {
            if drawn.contains(&level) {
                continue;
            }
            drawn.push(level);
            let (word, rgb) = if HARD.contains(&level) {
                (&self.words[2], [255, 0, 0])
            } else if MEDIUM.contains(&level) {
                (&self.words[1], [75, 75, 255])
            } else {
                (&self.words[0], [0, 255, 0])
            };
            let name_rgb = if g.won_levels & level.record().bit() != 0 { [0, 255, 255] } else { [255, 255, 255] };
            let name = self.names.get(level.index()).cloned().unwrap_or_default();
            for (text, y, rgb) in [(word.clone(), 62, rgb), (name, 80, name_rgb)] {
                let mut t = Text { font: 0, spacing: 1, rgb, x: 0, y, string: text, ..Default::default() };
                g.fonts.centre(&mut t, true, false);
                let mut shadow = t.clone();
                shadow.rgb = [1, 1, 1];
                shadow.x += 2;
                shadow.y += 2;
                shadow.triangles(&g.fonts, overlay);
                t.triangles(&g.fonts, overlay);
            }
        }
    }

    /// No cry in the hub: nothing hurts.
    fn cry(&mut self, _g: &mut Game) -> bool {
        true
    }

    fn describe(&self) -> String {
        format!("spike {:?}", self.spike)
    }
}
