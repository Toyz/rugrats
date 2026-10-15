//! The training messages (0x80097788, docs/levels/level-16.md): a line
//! of help shown once a level start when the player first does a thing.
//! In level 16 one freezes the player and the world until 1500 dt pass
//! or Cross closes it; level 1 shows messages 33 and 35 without a freeze.

use crate::disc::GameDisc;
use crate::input::{self, Pads};

/// Messages 0-35: strings 237-272 (0x800bb89c).
const COUNT: usize = 36;
const FIRST_STRING: u32 = 237;
/// How long one shows, and the cooldown after (0x800974bc).
const TIME: i32 = 1500;
/// Cross closing one (0x80097584).
const CLOSE_SOUND: u8 = 126;
/// The text's colour (0x800c4740) and its shadow's (0x800c4744).
const RGB: [u8; 3] = [106, 185, 228];
const SHADOW: [u8; 3] = [1, 1, 1];

pub struct Training {
    level: usize,
    strings: Vec<String>,
    /// Shows left (0x800e7f08) and cooldowns (0x800e7e90), per message.
    left: [u8; COUNT],
    cooldown: [i32; COUNT],
    /// The message showing (0x800c473c) and its time left (0x800c4738).
    showing: Option<(usize, i32)>,
}

impl Training {
    /// 0x80097664: every message once.
    pub fn load(disc: &GameDisc, level: usize) -> Training {
        let strings = if matches!(level, 1 | 16) {
            (0..COUNT as u32).map(|n| crate::front::string(disc, FIRST_STRING + n)).collect()
        } else {
            Vec::new()
        };
        Training { level, strings, left: [1; COUNT], cooldown: [0; COUNT], showing: None }
    }

    /// 0x80097788: message `n`, in level 16 (level 1: 33 and 35 only),
    /// dropped while another shows.
    pub fn show(&mut self, n: usize) {
        let allowed = self.level == 16 || (self.level == 1 && matches!(n, 33 | 35));
        if !allowed || self.showing.is_some() || n >= COUNT {
            return;
        }
        if self.left[n] == 0 || self.cooldown[n] != 0 {
            return;
        }
        self.left[n] -= 1;
        self.cooldown[n] = TIME;
        self.showing = Some((n, TIME));
        rrt::tracing::info!("training message {n}");
    }

    pub fn showing(&self) -> Option<usize> {
        self.showing.map(|s| s.0)
    }

    /// 0x80097584: a frame. True while the world stays frozen (level 16
    /// only); Cross closes the message with sound 126.
    pub fn frame(&mut self, pads: &Pads, effects: &mut Vec<u8>) -> bool {
        match &mut self.showing {
            Some((_, time)) if *time > 0 => {
                if pads.pressed(0, input::CROSS) {
                    effects.push(CLOSE_SOUND);
                    *time = 0;
                } else {
                    *time -= crate::player::DT;
                }
                self.level != 1
            }
            Some(_) => {
                self.showing = None;
                false
            }
            None => {
                for c in self.cooldown.iter_mut() {
                    *c = (*c - crate::player::DT).max(0);
                }
                false
            }
        }
    }

    /// 0x80097314: the shadow, then the text. Level 16: font 1 centred on
    /// the screen; level 1: font 0 centred across at y -105.
    pub fn draw(&self, fonts: &crate::text::Fonts, out: &mut Vec<rugrats_render::Vtx>) {
        let Some(s) = self.showing.and_then(|(n, time)| (time > 0).then(|| self.strings.get(n)).flatten()) else {
            return;
        };
        let (font, y, down) = if self.level == 1 { (0, -105, false) } else { (1, 0, true) };
        let mut t = crate::text::Text { font, spacing: 1, rgb: RGB, x: 0, y, string: s.clone(), ..Default::default() };
        fonts.centre(&mut t, true, down);
        let mut shadow = t.clone();
        shadow.rgb = SHADOW;
        shadow.x += 2;
        shadow.y += 2;
        shadow.triangles(fonts, out);
        t.triangles(fonts, out);
    }
}
