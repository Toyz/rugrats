//! The title's Audio menu (0x80041970, docs/engine/front-end.md): music
//! and effects volumes (0-100) and stereo or mono, over the title's
//! backdrop, with a boombox, two meters whose needles show the volumes,
//! two arrow blocks and two speakers turned for stereo or mono.

use rrt::glam::{Mat3, Vec3};
use rugrats_render::Vtx;

use crate::input::{self, Pads};
use crate::text::{self, Fonts, Text};

/// The settings it edits (state +0x0c, +0x10, +0x14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub music: i32,
    pub effects: i32,
    pub mono: bool,
}

/// The models (database 0): boombox, meter, needle, arrow block, speaker.
const BOOMBOX: u32 = 335;
const METER: u32 = 339;
const NEEDLE: u32 = 340;
const ARROWS: u32 = 334;
const SPEAKER: u32 = 343;
/// The speakers' first angles: stereo (0x800c4124) and mono (0x800c4128).
const STEREO_TURN: [i16; 2] = [1536, 2662];
const MONO_TURN: [i16; 2] = [2048, 2048];
/// Frames before the pads are read (0x80042014).
const INTRO: u32 = 20;
/// Strings: "Audio", "Music", "Sfx", "STEREO", "MONO".
const HEADING: u32 = 83;
const MUSIC: u32 = 84;
const SFX: u32 = 85;
const STEREO: u32 = 109;
const MONO: u32 = 110;
/// The chosen item's colour (0x05ffff) and the others'.
const CHOSEN: [u8; 3] = [255, 255, 5];
const WHITE: [u8; 3] = [255, 255, 255];
const SHADOW: [u8; 3] = [1, 1, 1];
/// The sounds (0x8006f26c): cancel, accept, down, up, the effects test.
const CANCEL: u8 = 1;
const ACCEPT: u8 = 2;
const DOWN: u8 = 3;
const UP: u8 = 4;
const TEST: u8 = 5;

/// A menu block (0x8003f444): its position and angles eased toward their
/// targets each frame (0x8003f7f0).
#[derive(Clone, Debug)]
struct Block {
    model: u32,
    pos: [i32; 3],
    target: [i32; 3],
    speed: [i32; 3],
    angles: [i16; 3],
    turn_to: [i16; 3],
    turn: i16,
}

impl Block {
    fn new(model: u32, pos: [i32; 3], target: [i32; 3], speed: [i32; 3]) -> Block {
        Block { model, pos, target, speed, angles: [0; 3], turn_to: [0; 3], turn: 0 }
    }

    fn step(&mut self) {
        for i in 0..3 {
            let (p, t, s) = (self.pos[i], self.target[i], self.speed[i]);
            self.pos[i] = if p < t { (p + s).min(t) } else { (p - s).max(t) };
            let (a, t) = (self.angles[i], self.turn_to[i]);
            self.angles[i] = if a < t { a.saturating_add(self.turn).min(t) } else { a.saturating_sub(self.turn).max(t) };
        }
    }
}

/// A text item and its shadow, which shares its target (the shadow ends
/// short where the text arrives first: it slides only while the text
/// does).
#[derive(Clone, Debug)]
struct Label {
    text: Text,
    shadow: Text,
}

impl Label {
    fn new(fonts: &Fonts, string: String, font: u8, at: (i16, i16), target: (i16, i16), flags: u16, centre: bool) -> Label {
        let mut text = Text { font, spacing: 1, rgb: WHITE, x: at.0, y: at.1, string, ..Default::default() };
        if centre {
            fonts.centre(&mut text, true, false);
        }
        let x = text.x;
        text.target = (if flags & text::SLIDE_X != 0 { target.0 } else { x }, target.1);
        text.rest = text.target;
        text.speed = (12, 12);
        text.count = (4, 4);
        text.flags = flags;
        let mut shadow = text.clone();
        shadow.rgb = SHADOW;
        shadow.x += 2;
        shadow.y += 2;
        Label { text, shadow }
    }

    fn step(&mut self) {
        let moving = self.text.flags & (text::SLIDE_X | text::SLIDE_Y) != 0;
        self.text.step();
        if moving {
            self.shadow.step();
        }
    }
}

pub struct AudioMenu {
    pub settings: Settings,
    entry: Settings,
    cursor: usize,
    frames: u32,
    heading: Label,
    items: [Label; 3],
    stereo_strings: [String; 2],
    boombox: Block,
    meters: [Block; 2],
    needles: [Block; 2],
    arrows: [Block; 2],
    speakers: [Block; 2],
    pub sounds: Vec<u8>,
    /// The song's volume to set this frame (0x8006ffa8), when it changed.
    pub song_volume: Option<i32>,
}

impl AudioMenu {
    /// Its strings: "Audio", "Music", "Sfx", "STEREO", "MONO".
    pub fn strings(disc: &crate::disc::GameDisc) -> [String; 5] {
        [HEADING, MUSIC, SFX, STEREO, MONO].map(|n| crate::front::string(disc, n))
    }

    pub fn open(strings: &[String; 5], fonts: &Fonts, settings: Settings) -> AudioMenu {
        let s = |k: usize| strings[k].clone();
        let stereo_strings = [s(3), s(4)];
        let heading = Label::new(fonts, s(0), 1, (0, -180), (0, -100), text::SLIDE_Y | text::WOBBLE, true);
        let items = [
            Label::new(fonts, s(1), 0, (280, -26), (116, -26), text::SLIDE_X, false),
            Label::new(fonts, s(2), 0, (271, 44), (128, 44), text::SLIDE_X, false),
            // At x 40 whatever the flag (0x80041970).
            Label::new(fonts, stereo_strings[settings.mono as usize].clone(), 0, (40, 140), (40, 77), text::SLIDE_Y, false),
        ];
        let mut boombox = Block::new(BOOMBOX, [-900, 0, 1000], [-400, 0, 900], [50, 50, 10]);
        boombox.turn_to = [2048, 0, 0];
        boombox.turn = 150;
        let meters = [
            Block::new(METER, [900, -100, 500], [174, -125, 500], [50; 3]),
            Block::new(METER, [900, 100, 500], [174, 50, 500], [50; 3]),
        ];
        let needle = |pos, target, v: i32| {
            let mut n = Block::new(NEEDLE, pos, target, [40, 50, 50]);
            n.angles = [0, 0, 1023];
            n.turn_to = [0, 0, needle_angle(v)];
            n.turn = 128;
            n
        };
        let needles = [needle([719, -52, 200], [124, -69, 200], settings.music), needle([719, 93, 200], [124, 53, 200], settings.effects)];
        let arrow = |y: i32, a: i16| {
            let mut b = Block::new(ARROWS, [0, 1000, 500], [0, y, 500], [50; 3]);
            b.angles = [1024, a, 1024];
            b.turn_to = b.angles;
            b
        };
        let arrows = [arrow(220, 0), arrow(150, 2048)];
        // The entry always takes the stereo pair (oracle).
        let speaker = |a: i16| {
            let mut b = Block::new(SPEAKER, [600, 1800, 4000], [600, 740, 4000], [150, 150, 50]);
            b.angles = [a, 0, 0];
            b.turn_to = [a, 0, 0];
            b.turn = 128;
            b
        };
        let speakers = [speaker(STEREO_TURN[0]), speaker(STEREO_TURN[1])];
        AudioMenu {
            settings,
            entry: settings,
            cursor: 0,
            frames: 0,
            heading,
            items,
            stereo_strings,
            boombox,
            meters,
            needles,
            arrows,
            speakers,
            sounds: Vec::new(),
            song_volume: None,
        }
    }

    /// One frame; Some(kept) once Cross, Start (kept) or Triangle
    /// (restored) ends it.
    pub fn frame(&mut self, pads: &Pads) -> Option<bool> {
        self.sounds.clear();
        self.song_volume = None;
        self.frames += 1;
        let done = if self.frames > INTRO { self.input(pads) } else { None };
        self.heading.step();
        for l in &mut self.items {
            l.step();
        }
        let v = [self.settings.music, self.settings.effects];
        for (n, v) in self.needles.iter_mut().zip(v) {
            n.turn_to[2] = needle_angle(v);
        }
        for b in std::iter::once(&mut self.boombox)
            .chain(self.meters.iter_mut())
            .chain(self.needles.iter_mut())
            .chain(self.arrows.iter_mut())
            .chain(self.speakers.iter_mut())
        {
            b.step();
        }
        if done.is_some() {
            self.song_volume = Some(self.settings.music);
        }
        done
    }

    fn input(&mut self, pads: &Pads) -> Option<bool> {
        let pressed = |b| pads.pressed(0, b);
        let held = |b| pads.held(0, b);
        if pressed(input::TRIANGLE) {
            self.sounds.push(CANCEL);
            self.settings = self.entry;
            return Some(false);
        }
        if pressed(input::CROSS) || pressed(input::START) {
            self.sounds.push(ACCEPT);
            return Some(true);
        }
        if pressed(input::UP) {
            self.sounds.push(UP);
            self.cursor = (self.cursor + 2) % 3;
        } else if pressed(input::DOWN) || pressed(input::SELECT) {
            self.sounds.push(DOWN);
            self.cursor = (self.cursor + 1) % 3;
        }
        // The effects test on a release (0x8001bda4 history & 3 == 2).
        if self.cursor == 1 {
            for b in [input::LEFT, input::RIGHT] {
                if pads.released(0, b) {
                    self.sounds.push(TEST);
                }
            }
        }
        if self.cursor == 2 && (pressed(input::LEFT) || pressed(input::RIGHT)) {
            self.flip();
        }
        if self.cursor < 2 {
            let v = if self.cursor == 0 { &mut self.settings.music } else { &mut self.settings.effects };
            let (left, right) = (held(input::LEFT), held(input::RIGHT));
            if left {
                *v = (*v - 1).max(0);
            }
            if right {
                *v = (*v + 1).min(100);
            }
            if self.cursor == 0 && (left || right) {
                self.song_volume = Some(self.settings.music);
            }
        }
        None
    }

    /// 0x800423f4: the flag flips, the label set anew at (40 or 60, 75),
    /// the speakers turned to the flag's pair.
    fn flip(&mut self) {
        self.settings.mono = !self.settings.mono;
        let m = self.settings.mono as usize;
        let l = &mut self.items[2];
        l.text.string = self.stereo_strings[m].clone();
        l.text.x = if self.settings.mono { 60 } else { 40 };
        l.text.y = 75;
        l.text.flags = 0;
        l.shadow = l.text.clone();
        l.shadow.rgb = SHADOW;
        l.shadow.x += 2;
        l.shadow.y = 77;
        let pair = if self.settings.mono { MONO_TURN } else { STEREO_TURN };
        for (s, a) in self.speakers.iter_mut().zip(pair) {
            s.turn_to[0] = a;
        }
    }

    /// Whether ACCEPT shows (not during the intro).
    pub fn accept_shown(&self) -> bool {
        self.frames > INTRO
    }

    /// The models, seen by the second camera (moved by `shift` onto the
    /// main one), and the text.
    pub fn draw(&self, models: &rugrats_data::model::Models, trig: &crate::gte::Trig, shift: Vec3, fonts: &Fonts, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        for b in std::iter::once(&self.boombox)
            .chain(self.meters.iter())
            .chain(self.needles.iter())
            .chain(self.arrows.iter())
            .chain(self.speakers.iter())
        {
            let Some(m) = models.get(b.model) else { continue };
            let e = trig.euler(b.angles);
            let rows = Mat3::from_cols_array(&e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap()).transpose();
            let pos = Vec3::new(b.pos[0] as f32, b.pos[1] as f32, b.pos[2] as f32) + shift;
            rugrats_render::models::model_triangles(m, rows.transpose(), pos, moving);
        }
        // The shadows, the items (the chosen one yellow), then the heading.
        for l in &self.items {
            l.shadow.triangles(fonts, overlay);
        }
        for (k, l) in self.items.iter().enumerate() {
            let mut t = l.text.clone();
            t.rgb = if k == self.cursor { CHOSEN } else { WHITE };
            t.triangles(fonts, overlay);
        }
        self.heading.shadow.triangles(fonts, overlay);
        self.heading.text.triangles(fonts, overlay);
    }
}

/// A needle's turn for a volume (0x80042724): 21 v - 1024, within
/// +-1024.
fn needle_angle(v: i32) -> i16 {
    (21 * v - 1024).clamp(-1024, 1024) as i16
}
