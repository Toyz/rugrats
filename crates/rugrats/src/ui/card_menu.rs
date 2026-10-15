//! Saving and loading on the card (0x800398bc, docs/engine/memory-card.md
//! "Saving and loading from the pause"): the pages, the record list and
//! the card calls behind them. The pause opens it on Save Game? (state 3)
//! or Load Game? (state 9); the title on Load Game (state 8).

use rugrats_render::Vtx;

use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::save::card::{Art, Cards, RECORD, STATE_SIZE, Save};
use crate::ui::front;
use crate::ui::text::{Fonts, Text};

/// The menu's colours (0x800391d4): items light blue, the heading pink,
/// the cursor blinking.
pub const ITEM: [u8; 3] = [106, 185, 228];
pub const HEADING: [u8; 3] = [198, 23, 93];
const CURSOR_FIRST: [u8; 3] = [255, 218, 0];
const CURSOR_BLINK: [[u8; 3]; 2] = [[236, 205, 19], [255, 240, 119]];
/// 26 vertical blanks a blink: 13 game frames.
const BLINK: u32 = 13;
/// Sounds: Left/Up and Triangle 124, Right/Down 125, Cross 126.
pub const BACK: u8 = 124;
pub const ON: u8 = 125;
pub const PICK: u8 = 126;

/// The cursor's colour after `frames` frames of the menu.
pub fn cursor_colour(frames: u32) -> [u8; 3] {
    if frames < BLINK { CURSOR_FIRST } else { CURSOR_BLINK[(frames / BLINK % 2) as usize] }
}

/// Where the menu leaves to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Back to the pause page, the cursor on Save Game? (2) or Load
    /// Game? (4) or Quit Game? (3); the title: the menu ends with 0.
    Back(usize),
    /// Start: the menu ends with 0 from any page.
    Close,
    /// Yes on "Are you sure": record k's state, loaded.
    Loaded(Box<[u8; STATE_SIZE]>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// 4: no card in slot 1.
    NoCard,
    /// 13: create the save file?
    Create,
    /// 22 (save) and 10 (load): the record list.
    List,
    /// 23: overwrite this record?
    Overwrite,
    /// 15: load this record?
    Sure,
    /// 16, 17: saved, or not.
    Saved,
    NotSaved,
    /// 20: the read failed; 21: creating the file failed.
    ReadFailed,
    CreateFailed,
    /// 25 (load): no save file.
    NoFile,
}

/// The strings the pages use, by the game's numbers.
pub struct Strings {
    s: std::collections::HashMap<u32, String>,
    levels: Vec<String>,
}

impl Strings {
    pub fn load(disc: &GameDisc) -> Strings {
        let ids = [11, 12, 13, 14, 15, 16, 17, 20, 21, 47, 48, 49, 50, 51, 52, 54, 55, 225, 226, 206, 207, 208, 209]
            .into_iter()
            .chain(32..=46);
        let s = ids.map(|n| (n, front::string(disc, n))).collect();
        // The levels' names (0x800b7d60, by state +0x00).
        let levels = (0..32).map(|l| disc.exe_string_at(0x800b_7d60 + 4 * l).unwrap_or_default()).collect();
        Strings { s, levels }
    }

    fn get(&self, n: u32) -> String {
        self.s.get(&n).cloned().unwrap_or_default()
    }
}

/// One item: string, font, x (None: centred), y.
#[derive(Clone, Debug)]
struct Item {
    text: String,
    font: u8,
    x: Option<i16>,
    y: i16,
}

pub struct CardMenu {
    saving: bool,
    title: bool,
    state: State,
    save: Save,
    /// The cursor's item (0: none), the chosen record.
    cursor: usize,
    chosen: usize,
    footer: bool,
    frames: u32,
    /// The pause's state block and the hub spot of its level (0x800b8718).
    state_block: [u8; STATE_SIZE],
    spot: [i32; 3],
    art: Art,
    pub sounds: Vec<u8>,
}

impl CardMenu {
    /// Save (state 3) or Load (9 -> 8): the card checked, the file looked
    /// for and read, then the first page.
    pub fn open(saving: bool, title: bool, art: Art, state_block: [u8; STATE_SIZE], spot: [i32; 3]) -> CardMenu {
        let mut m = CardMenu {
            saving,
            title,
            state: State::NoCard,
            save: Save::default(),
            cursor: 0,
            chosen: 0,
            footer: false,
            frames: 0,
            state_block,
            spot,
            art,
            sounds: Vec::new(),
        };
        m.check();
        m
    }

    fn cards() -> Cards {
        Cards::default()
    }

    /// 0x80070d80, 0x8007047c, 0x80070598.
    fn check(&mut self) {
        let cards = Self::cards();
        if !cards.present(0) {
            self.enter(State::NoCard);
            return;
        }
        match cards.load(0, &self.art) {
            Some(save) if save.check() => {
                self.save = save;
                self.enter(State::List);
            }
            Some(_) => self.enter(State::ReadFailed),
            None if self.saving => self.enter(State::Create),
            None => self.enter(State::NoFile),
        }
    }

    fn enter(&mut self, s: State) {
        self.state = s;
        // Yes / No pages start on No?; the list on record 1.
        self.cursor = match s {
            State::Create | State::Overwrite | State::Sure => 2,
            State::List => 1,
            _ => 0,
        };
        if matches!(s, State::Create | State::Sure | State::Saved | State::NotSaved | State::Overwrite | State::NoFile)
        {
            self.footer = true;
        }
    }

    fn back(&self, item: usize) -> Outcome {
        if self.title { Outcome::Close } else { Outcome::Back(item) }
    }

    fn used(&self, k: usize) -> bool {
        self.save.words.get(1).is_some_and(|w| w >> k & 1 != 0)
    }

    fn name(&self, k: usize) -> Option<String> {
        if !self.used(k) {
            return None;
        }
        let b = self.save.to_bytes();
        let n = &b[RECORD * k + 0x14..RECORD * k + 0x20];
        let end = n.iter().position(|&c| c == 0).unwrap_or(n.len());
        Some(String::from_utf8_lossy(&n[..end]).into_owned())
    }

    /// 0x80070b14: record k, with the hub's spot of the level saved in
    /// (+0x24..+0x2c), word 2 cleared, the file written.
    fn write(&mut self, k: usize) -> bool {
        let mut state = self.state_block;
        for (i, v) in self.spot.iter().enumerate() {
            state[0x24 + 4 * i..0x28 + 4 * i].copy_from_slice(&v.to_le_bytes());
        }
        self.save.words[2] = 0;
        self.save.put_record(k, &state);
        Self::cards().store(0, &self.art, &self.save).is_ok()
    }

    /// One frame; Some once it ends.
    pub fn frame(&mut self, pads: &Pads) -> Option<Outcome> {
        self.sounds.clear();
        self.frames += 1;
        let pressed = |b| pads.pressed(0, b);
        if self.frames > 1 && pressed(input::START) {
            return Some(Outcome::Close);
        }
        // Moves: the list is 5 rows by 3 columns, numbered down the
        // columns, no wrap; Yes / No is one row of two.
        let (rows, cols) = match self.state {
            State::List => (5, 3),
            State::Create | State::Overwrite | State::Sure => (1, 2),
            _ => (0, 0),
        };
        if self.cursor > 0 && rows > 0 {
            let k = self.cursor - 1;
            let (col, row) = (k / rows, k % rows);
            let to = if pressed(input::LEFT) && col > 0 {
                Some((k - rows, BACK))
            } else if pressed(input::RIGHT) && col + 1 < cols {
                Some((k + rows, ON))
            } else if pressed(input::UP) && row > 0 {
                Some((k - 1, BACK))
            } else if pressed(input::DOWN) && row + 1 < rows {
                Some((k + 1, ON))
            } else {
                None
            };
            if let Some((k, s)) = to {
                self.cursor = k + 1;
                self.sounds.push(s);
            }
        }
        let save_item = 2;
        let load_item = 4;
        if pressed(input::CROSS) {
            self.sounds.push(PICK);
            let yes = self.cursor == 1;
            match self.state {
                State::List if self.saving => {
                    self.chosen = self.cursor;
                    if self.used(self.chosen) {
                        self.enter(State::Overwrite);
                    } else {
                        let ok = self.write(self.chosen);
                        self.enter(if ok { State::Saved } else { State::NotSaved });
                    }
                }
                State::List => {
                    self.chosen = self.cursor;
                    if self.name(self.chosen).is_some() {
                        self.enter(State::Sure);
                    }
                }
                State::Overwrite if yes => {
                    let ok = self.write(self.chosen);
                    self.enter(if ok { State::Saved } else { State::NotSaved });
                }
                State::Overwrite => self.enter(State::List),
                State::Sure if yes => {
                    // 0x80070970: the file read again, record k's state.
                    let state = Self::cards().load(0, &self.art).and_then(|s| s.record(self.chosen));
                    return Some(match state {
                        Some(b) => Outcome::Loaded(Box::new(b)),
                        None => self.back(load_item),
                    });
                }
                State::Sure => self.enter(State::List),
                State::Create if yes => {
                    // 0x800707a8: the file with an empty save.
                    self.save = Save::default();
                    self.save.seal();
                    if Self::cards().store(0, &self.art, &self.save).is_ok() {
                        self.enter(State::List);
                    } else {
                        self.enter(State::CreateFailed);
                    }
                }
                State::Create => return Some(self.back(save_item)),
                State::NoCard | State::ReadFailed => {
                    return Some(self.back(if self.saving { save_item } else { load_item }));
                }
                // The load's "No Game File" goes back to Quit Game? (3).
                State::NoFile => return Some(self.back(3)),
                State::Saved | State::NotSaved | State::CreateFailed => return Some(self.back(save_item)),
            }
            return None;
        }
        if pressed(input::TRIANGLE) {
            self.sounds.push(BACK);
            match self.state {
                State::List => return Some(self.back(if self.saving { save_item } else { load_item })),
                State::NoCard => return Some(self.back(load_item)),
                State::Overwrite | State::Sure => self.enter(State::List),
                _ => return Some(self.back(if self.saving { save_item } else { load_item })),
            }
        }
        None
    }

    fn items(&self, s: &Strings) -> Vec<Item> {
        let heading = s.get(if self.saving { 14 } else { 13 });
        let c = |n: u32, y: i16| Item { text: s.get(n), font: 0, x: None, y };
        let yes_no = |v: &mut Vec<Item>| {
            v.push(Item { text: s.get(11), font: 0, x: Some(-125), y: 0 });
            v.push(Item { text: s.get(12), font: 0, x: Some(60), y: 0 });
        };
        let mut v = Vec::new();
        match self.state {
            State::NoCard => {
                v.push(Item { text: heading, font: 1, x: None, y: -60 });
                v.push(c(16, -30));
                v.push(c(17, -10));
            }
            State::NoFile => {
                v.push(Item { text: heading, font: 1, x: None, y: -60 });
                v.push(c(15, -30));
            }
            State::Create => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                yes_no(&mut v);
                v.push(c(20, -40));
                v.push(c(21, -20));
            }
            State::Overwrite => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                yes_no(&mut v);
                v.push(c(55, -40));
                v.push(c(54, -20));
            }
            State::Sure => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                yes_no(&mut v);
                v.push(c(225, -40));
                v.push(c(226, -20));
            }
            State::Saved => {
                v.push(Item { text: heading, font: 1, x: None, y: -60 });
                v.push(c(48, -30));
            }
            State::NotSaved => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                v.push(c(49, -40));
                v.push(c(50, -20));
            }
            State::ReadFailed => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                v.push(c(49, -40));
                v.push(c(51, -20));
            }
            State::CreateFailed => {
                v.push(Item { text: heading, font: 1, x: None, y: -70 });
                v.push(c(49, -40));
                v.push(c(52, -20));
            }
            State::List => {
                v.push(Item { text: heading, font: 1, x: None, y: -90 });
                const NUMBER_X: [i16; 3] = [-190, -45, 110];
                const NAME_X: [i16; 3] = [-160, -15, 140];
                for k in 1..=15usize {
                    let (col, row) = ((k - 1) / 5, (k - 1) % 5);
                    v.push(Item {
                        text: s.get(31 + k as u32),
                        font: 0,
                        x: Some(NUMBER_X[col]),
                        y: -60 + 25 * row as i16,
                    });
                }
                for k in 1..=15usize {
                    let (col, row) = ((k - 1) / 5, (k - 1) % 5);
                    let name = self.name(k).unwrap_or_else(|| s.get(47));
                    v.push(Item { text: name, font: 0, x: Some(NAME_X[col]), y: -60 + 25 * row as i16 });
                }
            }
        }
        v
    }

    /// The load list's line about the cursor's record (string 206-209):
    /// its level, pieces and bars (mod 15).
    fn detail(&self, s: &Strings) -> Option<String> {
        if self.saving || self.state != State::List {
            return None;
        }
        let state = self.save.record(self.cursor)?;
        let w = |at: usize| u32::from_le_bytes(state[at..at + 4].try_into().unwrap());
        let level = w(0) as usize;
        let pieces = (w(0x44) & 0xffff).count_ones();
        let bars = (0..32).map(|l| w(0x48 + 4 * l).count_ones()).sum::<u32>() % 15;
        let form = s.get(match (pieces == 1, bars == 1) {
            (true, true) => 206,
            (true, false) => 207,
            (false, true) => 208,
            (false, false) => 209,
        });
        let name = s.levels.get(level).cloned().unwrap_or_default();
        let mut out = String::new();
        let mut args = [name, pieces.to_string(), bars.to_string()].into_iter();
        let mut chars = form.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '%' && matches!(chars.peek(), Some('s' | 'd')) {
                chars.next();
                out.push_str(&args.next().unwrap_or_default());
            } else {
                out.push(ch);
            }
        }
        Some(out)
    }

    /// The page: shadows, then the items coloured (0x800391d4), the
    /// detail line, and the footer.
    pub fn triangles(&self, s: &Strings, fonts: &Fonts, press: &str, labels: &[String; 2], out: &mut Vec<Vtx>) {
        let items = self.items(s);
        let cursor = cursor_colour(self.frames);
        let list = self.state == State::List;
        let text = |it: &Item, rgb: [u8; 3]| {
            let mut t = Text {
                font: it.font,
                spacing: 1,
                rgb,
                x: it.x.unwrap_or(0),
                y: it.y,
                string: it.text.clone(),
                ..Default::default()
            };
            if it.x.is_none() {
                fonts.centre(&mut t, true, false);
            }
            t
        };
        let mut drawn: Vec<Text> = Vec::new();
        for (k, it) in items.iter().enumerate() {
            // The cursor's item, and on the list its name with its number.
            let lit = self.cursor > 0 && (k == self.cursor || (list && k == self.cursor + 15));
            let rgb = if k == 0 {
                HEADING
            } else if lit {
                cursor
            } else {
                ITEM
            };
            drawn.push(text(it, rgb));
        }
        for t in &drawn {
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(fonts, out);
        }
        for t in &drawn {
            t.triangles(fonts, out);
        }
        if let Some(line) = self.detail(s) {
            let it = Item { text: line, font: 0, x: None, y: 64 };
            let mut shadow = text(&it, [1, 1, 1]);
            shadow.y += 2;
            shadow.triangles(fonts, out);
            text(&it, HEADING).triangles(fonts, out);
        }
        // PRESS X TO CONTINUE while the footer flag is set, else CANCEL
        // and ACCEPT.
        let label = |string: &str, x: i16, y: i16, font: u8, out: &mut Vec<Vtx>| {
            let t =
                Text { font, spacing: 1, rgb: [255, 255, 255], x, y, string: string.to_string(), ..Default::default() };
            let mut shadow = t.clone();
            shadow.rgb = [1, 1, 1];
            shadow.x += 2;
            shadow.y += 2;
            shadow.triangles(fonts, out);
            t.triangles(fonts, out);
        };
        if self.footer {
            label(press, -96, 80, 2, out);
        } else {
            label(&labels[0], -192, -93, 2, out);
            label(&labels[1], 115, -93, 2, out);
        }
    }
}
