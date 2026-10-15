//! The pause menu (0x8003e05c, 0x800398bc mode 0).

use super::*;

/// How the pause menu ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PauseEnd {
    Continue,
    /// Load Game confirmed (state 15): the pause returns 1, to the hub.
    Loaded(Box<[u8; crate::save::card::STATE_SIZE]>),
    QuitGame,
    QuitLevel,
}

/// The pause menu (0x8003e05c, 0x800398bc mode 0, laid out by
/// 0x80035fec): "Game Paused" over five items, the cursor 1-5; a quit
/// asks again (0x80036368) with "No?" first.
pub(super) struct Pause {
    pub(super) strings: [String; 6],
    pub(super) yes: String,
    pub(super) no: String,
    pub(super) cursor: usize,
    /// Asking about item 3 or 5 (0x80036368): its cursor, 1 Yes?, 2
    /// Save Game?, 3 No? (the default), 4 Load Game?.
    pub(super) confirm: Option<(usize, usize)>,
    pub(super) sounds: Vec<u8>,
    /// The card pages (Save Game?, Load Game?), their strings, the
    /// footer's texts, the card's art, the hub's spot of this level.
    pub(super) card: Option<crate::ui::card_menu::CardMenu>,
    pub(super) card_strings: crate::ui::card_menu::Strings,
    pub(super) press: String,
    pub(super) labels: [String; 2],
    pub(super) art: crate::save::card::Art,
    pub(super) spot: [i32; 3],
    /// Frames since the pause opened (the cursor's blink).
    pub(super) frames: u32,
}

/// Where the items sit (0x80035fec), from the screen's centre.
pub(super) const PAUSE_AT: [(i16, i16); 5] = [(-185, -30), (-145, -10), (-60, -30), (30, -10), (80, -30)];

/// The menu's sounds (0x800398bc): Left/Up and Triangle 124,
/// Right/Down 125, Cross 126.
pub(super) const MOVE: u8 = crate::ui::card_menu::BACK;

pub(super) const MOVE_ON: u8 = crate::ui::card_menu::ON;

pub(super) const PICK: u8 = crate::ui::card_menu::PICK;

impl Pause {
    pub(super) fn load(disc: &GameDisc, level: Level) -> Result<Pause, String> {
        let s = |n: u32| crate::ui::front::string(disc, n);
        // The hub's spot for this level (0x800b8718), written into a save.
        let b = disc.exe_bytes(0x800b_8718 + 16 * level.index() as u32, 12)?;
        let spot = [0, 4, 8].map(|k| i32::from_le_bytes(b[k..k + 4].try_into().unwrap()));
        let spot = if spot == [0; 3] { [5775, 779, 19403] } else { spot };
        Ok(Pause {
            strings: [s(1), s(2), s(3), s(4), s(5), s(6)],
            yes: disc.exe_string_at_addr(0x800c_48f0)?,
            no: disc.exe_string_at_addr(0x800c_48ec)?,
            cursor: 1,
            confirm: None,
            sounds: Vec::new(),
            card: None,
            card_strings: crate::ui::card_menu::Strings::load(disc),
            press: crate::ui::front::string(disc, crate::ui::screens::PRESS),
            labels: [
                disc.exe_string_at(0x800b_b504).unwrap_or_default(),
                disc.exe_string_at(0x800b_b508).unwrap_or_default(),
            ],
            art: disc.card.clone(),
            spot,
            frames: 0,
        })
    }

    pub(super) fn open(&mut self) {
        self.cursor = 1;
        self.confirm = None;
        self.card = None;
        self.frames = 0;
    }

    pub(super) fn frame(&mut self, pads: &Pads, state: &[u8; crate::save::card::STATE_SIZE]) -> Option<PauseEnd> {
        self.frames += 1;
        if let Some(card) = &mut self.card {
            let out = card.frame(pads);
            self.sounds.extend(card.sounds.drain(..));
            match out {
                Some(crate::ui::card_menu::Outcome::Back(item)) => {
                    self.card = None;
                    self.cursor = item;
                }
                Some(crate::ui::card_menu::Outcome::Close) => return Some(PauseEnd::Continue),
                Some(crate::ui::card_menu::Outcome::Loaded(b)) => return Some(PauseEnd::Loaded(b)),
                None => {}
            }
            return None;
        }
        let pressed = |b: usize| pads.pressed(0, b);
        if let Some((item, c)) = &mut self.confirm {
            // Two columns: Yes? over Save Game?, No? over Load Game?.
            if pressed(input::LEFT) && *c > 2 {
                *c -= 2;
                self.sounds.push(MOVE);
            } else if pressed(input::RIGHT) && *c < 3 {
                *c += 2;
                self.sounds.push(MOVE_ON);
            } else if pressed(input::UP) && *c % 2 == 0 {
                *c -= 1;
                self.sounds.push(MOVE);
            } else if pressed(input::DOWN) && *c % 2 == 1 {
                *c += 1;
                self.sounds.push(MOVE_ON);
            }
            if pressed(input::TRIANGLE) {
                self.sounds.push(MOVE);
                self.confirm = None;
                return None;
            }
            if pressed(input::CROSS) {
                self.sounds.push(PICK);
                let (item, c) = (*item, *c);
                self.confirm = None;
                match c {
                    1 => return Some(if item == 3 { PauseEnd::QuitGame } else { PauseEnd::QuitLevel }),
                    2 | 4 => {
                        self.card = Some(crate::ui::card_menu::CardMenu::open(
                            c == 2,
                            false,
                            self.art.clone(),
                            *state,
                            self.spot,
                        ))
                    }
                    _ => self.cursor = item,
                }
            }
            return None;
        }
        if pressed(input::LEFT) && self.cursor > 2 {
            self.cursor -= 2;
            self.sounds.push(MOVE);
        }
        if pressed(input::RIGHT) && self.cursor < 4 {
            self.cursor += 2;
            self.sounds.push(MOVE_ON);
        }
        if pressed(input::UP) && self.cursor % 2 == 0 {
            self.cursor -= 1;
            self.sounds.push(MOVE);
        } else if pressed(input::DOWN) && self.cursor % 2 == 1 && self.cursor < 5 {
            self.cursor += 1;
            self.sounds.push(MOVE_ON);
        }
        if pressed(input::START) {
            return Some(PauseEnd::Continue);
        }
        if pressed(input::TRIANGLE) {
            self.sounds.push(MOVE);
            return Some(PauseEnd::Continue);
        }
        if pressed(input::CROSS) {
            self.sounds.push(PICK);
            match self.cursor {
                1 => return Some(PauseEnd::Continue),
                3 | 5 => self.confirm = Some((self.cursor, 3)),
                // Save Game? (state 3) or Load Game? (9): the card pages.
                c => {
                    self.card =
                        Some(crate::ui::card_menu::CardMenu::open(c == 2, false, self.art.clone(), *state, self.spot))
                }
            }
        }
        None
    }

    pub(super) fn triangles(&self, fonts: &crate::ui::text::Fonts, out: &mut Vec<rugrats_render::Vtx>) {
        use crate::ui::text::Text;
        if let Some(card) = &self.card {
            card.triangles(&self.card_strings, fonts, &self.press, &self.labels, out);
            return;
        }
        // The heading in font 1 and pink, the items in font 0 and light
        // blue, the cursor's blinking (0x80035fec, 0x800391d4).
        let cursor = crate::ui::card_menu::cursor_colour(self.frames);
        let mut item = |s: &str, x: i16, y: i16, lit: bool, centre: bool| {
            let rgb = if centre {
                crate::ui::card_menu::HEADING
            } else if lit {
                cursor
            } else {
                crate::ui::card_menu::ITEM
            };
            let mut t = Text {
                font: if centre { 1 } else { 0 },
                spacing: 1,
                string: s.to_string(),
                x,
                y,
                rgb,
                ..Default::default()
            };
            if centre {
                fonts.centre(&mut t, true, false);
            }
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(fonts, out);
            t.triangles(fonts, out);
        };
        match self.confirm {
            Some((q, c)) => {
                item(&self.strings[q], 0, -60, false, true);
                item(&self.yes, -125, -30, c == 1, false);
                item(&self.strings[2], -145, -10, c == 2, false);
                item(&self.no, 35, -30, c == 3, false);
                item(&self.strings[4], 35, -10, c == 4, false);
            }
            None => {
                item(&self.strings[0], 0, -60, false, true);
                for (k, &(x, y)) in PAUSE_AT.iter().enumerate() {
                    item(&self.strings[k + 1], x, y, self.cursor == k + 1, false);
                }
            }
        }
    }
}
