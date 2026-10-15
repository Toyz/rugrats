//! The screens between and inside levels (docs/engine/screens.md): over
//! the front end's backdrop, a burst flies in with a message ("Congra-
//! tulations! You earned a puzzle piece!", the bar award, ...), then the
//! puzzle board where an earned piece is placed. Each runs its own 30 fps
//! loop with the level frozen and not drawn.

use rrt::glam::{Mat3, Vec3};
use rugrats_render::Vtx;

use crate::disc::GameDisc;
use crate::front::{Backdrop, Button};
use crate::hud::Sprites;
use crate::input::{self, Pads};
use crate::text::{Fonts, Text};

/// Colours (0xbbggrr in the executable): yellow 0x13cdec, pink 0x5d17c6,
/// white, the shadow 0x010101.
pub const YELLOW: [u8; 3] = [0xec, 0xcd, 0x13];
pub const PINK: [u8; 3] = [0xc6, 0x17, 0x5d];
pub const WHITE: [u8; 3] = [0xff, 0xff, 0xff];
const SHADOW: [u8; 3] = [1, 1, 1];

/// The burst (BURST2.CMD, file 240) and the front-end camera flying to it
/// (0x80040194, 0x8003f350): from z -17048 to -500 at 600 a frame.
const BURST: usize = 240;
const FAR: i32 = -17048;
const NEAR: i32 = -500;
const FLY: i32 = 600;
const CAMERA_Y: f32 = 30.0;
/// The front end's field of view (0x80040194: 512).
const FOV: i32 = 512;
/// The PRESS X TO CONTINUE line (string 67) and its cube (0x80035798).
const PRESS: u32 = 67;
const BUTTON_MODEL: u32 = 334;
const DB00_MOD: usize = 242;
/// The second camera the cubes are seen by (0x80040010).
const BUTTON_CAMERA: Vec3 = Vec3::new(0.0, 0.0, -500.0);

/// The board's sprites (0x800943bc): the board, the cover tile, piece i
/// in colour, its shadow, greyed.
const BOARD: u32 = 350;
const TILE: u32 = 351;
const PIECE: u32 = 314;
const PIECE_SHADOW: u32 = 326;
const PIECE_GREY: u32 = 338;
/// Tray (0x800bb32c) and slot (0x800bb38c) positions.
const TRAY: [(i16, i16); 12] = [
    (150, -2),
    (-200, -54),
    (-150, -28),
    (150, -54),
    (200, 26),
    (-150, 26),
    (200, 76),
    (200, -28),
    (-200, -2),
    (150, 50),
    (-150, 76),
    (-200, 50),
];
const SLOT: [(i16, i16); 12] = [
    (-52, -44),
    (-1, -44),
    (48, -44),
    (-52, -14),
    (-1, -14),
    (49, -14),
    (-52, 18),
    (-1, 16),
    (48, 18),
    (-52, 46),
    (-1, 46),
    (48, 46),
];
/// The bar icon (sprite 291, the level's own).
const BAR_ICON: u32 = 291;
/// Strings: "Select Piece" (94), "Place Piece" (95), "Terrific!" (96),
/// CANCEL (7), ACCEPT (8).
const TITLES: [u32; 3] = [94, 95, 96];
const CANCEL: u32 = 7;
const ACCEPT: u32 = 8;
/// Sounds: move or wrong (124), back (125), accept (126).
const MOVE: u8 = 124;
const BACK: u8 = 125;
const OK: u8 = 126;

/// A page of the screen.
#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    /// Three lines (string, colour) at y -65, -40, -15, font 1; with the
    /// bar icon and the bars available; with the blinking piece.
    Message { lines: Vec<(String, [u8; 3])>, bars: Option<u32>, blink: bool },
    /// The puzzle board (0x80092c68).
    Board,
    /// Select's "Restart Level?" (0x80043d58): Yes? or No?.
    Restart,
}

/// The messages (0x800930c8 and its siblings), by kind.
pub mod message {
    use super::*;

    fn lines(disc: &GameDisc, ids: [u32; 3], colours: [[u8; 3]; 3]) -> Vec<(String, [u8; 3])> {
        ids.iter().zip(colours).map(|(&n, c)| (crate::front::string(disc, n), c)).collect()
    }

    /// "Help Tommy win / twelve puzzle pieces / by completing levels!"
    pub fn help(disc: &GameDisc) -> Page {
        Page::Message { lines: lines(disc, [88, 89, 90], [YELLOW; 3]), bars: None, blink: true }
    }

    /// "Congratulations! / You earned a / puzzle piece!"
    pub fn piece(disc: &GameDisc) -> Page {
        Page::Message { lines: lines(disc, [91, 92, 93], [YELLOW, PINK, PINK]), bars: None, blink: true }
    }

    /// "You did it! / You've found all the / Reptar Puzzle Pieces!"
    pub fn all(disc: &GameDisc) -> Page {
        Page::Message { lines: lines(disc, [228, 230, 231], [YELLOW, PINK, PINK]), bars: None, blink: false }
    }

    /// "Great Job! / You have earned / n Reptar bar(s)!" with the bars
    /// available.
    pub fn bars(disc: &GameDisc, n: u32, available: u32) -> Page {
        let n = n.clamp(1, 4);
        Page::Message { lines: lines(disc, [100, 101, 101 + n], [YELLOW, PINK, PINK]), bars: Some(available), blink: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fly {
    In,
    Hold,
    Out,
}

/// The board's run (0x80092c68): tiles opening, the state machine, the
/// wait, tiles closing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Run {
    Open(usize),
    Wait,
    Close(usize),
}

/// The board's state (0x800c4e08).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Start,
    Select,
    Place,
    Terrific(i32),
    Over,
}

struct Board {
    run: Run,
    state: State,
    tiles: [bool; 35],
    order: [usize; 35],
    cursor: Option<usize>,
    selected: Option<usize>,
    pos: [(i16, i16); 12],
}

pub struct Screen {
    pub stage: crate::stage::Stage,
    pages: Vec<Page>,
    fonts: Fonts,
    sprites: Option<Sprites>,
    trig: crate::gte::Trig,
    focal: f32,
    backdrop: Backdrop,
    frames: u32,
    burst: Option<rugrats_data::model::Model>,
    button_model: Option<rugrats_data::model::Model>,
    cube: Button,
    z: i32,
    fly: Fly,
    board: Option<Board>,
    press: String,
    titles: [String; 3],
    labels: [String; 2],
    all_text: Page,
    /// Pieces earned and placed (+0x44, +0x46): the board writes placed.
    pub earned: u16,
    pub placed: u16,
    /// All twelve placed on this board: Reptar 2010 next.
    pub all_placed: bool,
    /// The restart menu's answer, and its cursor (1 Yes?, 2 No?).
    pub restart: bool,
    cursor: u8,
    restart_text: [String; 3],
    pub sounds: Vec<u8>,
}

fn order(rand: &mut crate::rand::Rand) -> [usize; 35] {
    // 0x80092090: for k, r = (rand & 255) % 35 until free; order[r] = k.
    let mut order = [usize::MAX; 35];
    for k in 0..35 {
        loop {
            let r = (rand.next() & 255) as usize % 35;
            if order[r] == usize::MAX {
                order[r] = k;
                break;
            }
        }
    }
    order
}

impl Screen {
    /// The screen with `pages`; sprites from database `db`'s TEXPOS.
    /// `level` is the VRAM the level left, for the bar icon.
    pub fn open(disc: &GameDisc, pages: Vec<Page>, db: usize, earned: u16, placed: u16, level: Option<&rugrats_render::Vram>) -> Result<Screen, String> {
        let mut vram = crate::front::menu_vram(disc)?;
        let sprites = Sprites::load(disc, db).ok();
        if let (Some(sp), Some(from)) = (&sprites, level) {
            sp.copy(BAR_ICON, from, &mut vram);
        }
        let mut stage = crate::stage::Stage::new(vram, Vec::new());
        stage.clear = [0, 0, 0];
        let trig = crate::gte::Trig::load(disc)?;
        let focal = trig.geom_screen(FOV).unwrap_or(386) as f32;
        let raw = disc.file(BURST)?;
        let burst = rugrats_data::model::Models::parse(&raw)
            .ok()
            .and_then(|m| m.iter().next().map(|(_, m)| m.clone()))
            .or_else(|| rugrats_data::model::Model::parse(&raw).ok());
        if burst.is_none() {
            rrt::tracing::warn!("BURST2.CMD: not read as a model");
        }
        let button_model = rugrats_data::model::Models::parse(&disc.file(DB00_MOD)?).ok().and_then(|m| m.get(BUTTON_MODEL).cloned());
        let mut s = Screen {
            stage,
            pages,
            fonts: Fonts::load(disc)?,
            sprites,
            trig,
            focal,
            backdrop: Backdrop::default(),
            frames: 0,
            burst,
            button_model,
            cube: Button { pos: [-60, 414, 1298], angles: [0; 3], targets: [1024, 0, 0], speed: 128 },
            z: FAR,
            fly: Fly::In,
            board: None,
            press: crate::front::string(disc, PRESS),
            titles: TITLES.map(|n| crate::front::string(disc, n)),
            labels: [crate::front::string(disc, CANCEL), crate::front::string(disc, ACCEPT)],
            all_text: message::all(disc),
            earned,
            placed,
            all_placed: false,
            restart: false,
            cursor: 2,
            restart_text: [229, 11, 12].map(|n| crate::front::string(disc, n)),
            sounds: Vec::new(),
        };
        s.begin();
        s.draw();
        Ok(s)
    }

    fn begin(&mut self) {
        self.z = FAR;
        self.fly = Fly::In;
        self.board = None;
    }

    fn start_board(&mut self, rand: &mut crate::rand::Rand) {
        let pos = std::array::from_fn(|i| if self.placed >> i & 1 != 0 { SLOT[i] } else { TRAY[i] });
        self.board = Some(Board {
            run: Run::Open(0),
            state: State::Start,
            tiles: [true; 35],
            order: order(rand),
            cursor: None,
            selected: None,
            pos,
        });
    }

    /// One frame; false when the last page is over.
    pub fn frame(&mut self, pads: &Pads, rand: &mut crate::rand::Rand) -> bool {
        self.sounds.clear();
        self.frames += 1;
        if self.frames.is_multiple_of(30) {
            self.backdrop.fading = true;
        }
        if self.backdrop.fading {
            self.backdrop.fading = self.backdrop.step();
        }
        let Some(page) = self.pages.first().cloned() else { return false };
        let done = match page {
            Page::Message { .. } => self.message_frame(pads),
            Page::Restart => self.restart_frame(pads),
            Page::Board => {
                if self.board.is_none() {
                    self.z = NEAR;
                    self.start_board(rand);
                }
                self.board_frame(pads, rand)
            }
        };
        if done {
            self.pages.remove(0);
            if self.pages.is_empty() && self.all_placed && !matches!(page, Page::Message { blink: false, bars: None, .. }) {
                // All twelve placed: "You did it!" (0x80094978, speech 2046).
                self.pages.push(self.all_text.clone());
            }
            self.begin();
            if self.pages.is_empty() {
                return false;
            }
        }
        self.draw();
        true
    }

    /// The message loop (0x80093048): fly in, hold until Cross, fly out.
    fn message_frame(&mut self, pads: &Pads) -> bool {
        self.cube.step();
        if self.fly != Fly::Out && pads.pressed(0, input::CROSS) {
            self.sounds.push(OK);
            self.fly = Fly::Out;
            self.z = NEAR;
            return false;
        }
        match self.fly {
            Fly::In => {
                self.z += FLY;
                if self.z >= NEAR - 1 {
                    self.z = NEAR;
                    self.fly = Fly::Hold;
                }
                false
            }
            Fly::Hold => false,
            Fly::Out => {
                self.z -= FLY;
                if self.z <= FAR {
                    self.z = FAR;
                    return true;
                }
                false
            }
        }
    }

    /// The restart menu (0x80043d58): the burst already near; Left/Up
    /// and Right/Down move between Yes? and No?; Cross answers, Start or
    /// Triangle is No.
    fn restart_frame(&mut self, pads: &Pads) -> bool {
        self.z = NEAR;
        self.fly = Fly::Hold;
        if pads.pressed(0, input::LEFT) && self.cursor > 1 {
            self.cursor -= 1;
            self.sounds.push(MOVE);
        } else if pads.pressed(0, input::RIGHT) && self.cursor < 2 {
            self.cursor += 1;
            self.sounds.push(BACK);
        } else if pads.pressed(0, input::UP) && self.cursor < 2 {
            self.cursor += 1;
            self.sounds.push(MOVE);
        } else if pads.pressed(0, input::DOWN) && self.cursor > 1 {
            self.cursor -= 1;
            self.sounds.push(BACK);
        }
        if pads.pressed(0, input::START) || pads.pressed(0, input::TRIANGLE) {
            self.sounds.push(MOVE);
            self.restart = false;
            return true;
        }
        if pads.pressed(0, input::CROSS) {
            self.sounds.push(OK);
            self.restart = self.cursor == 1;
            return true;
        }
        false
    }

    /// The board (0x80092c68, 0x800929f8).
    fn board_frame(&mut self, pads: &Pads, rand: &mut crate::rand::Rand) -> bool {
        let earned = self.earned;
        let Some(b) = &mut self.board else { return true };
        match b.run {
            Run::Open(n) => {
                if n < 35 {
                    b.tiles[b.order[n]] = false;
                    b.run = Run::Open(n + 1);
                }
            }
            Run::Wait => {
                if pads.pressed(0, input::CROSS) {
                    self.sounds.push(OK);
                    b.run = Run::Close(0);
                    b.order = order(rand);
                } else if pads.pressed(0, input::TRIANGLE) {
                    b.run = Run::Close(0);
                    b.order = order(rand);
                }
                return false;
            }
            Run::Close(n) => {
                if n >= 35 {
                    if self.placed.count_ones() >= 12 {
                        self.all_placed = true;
                    }
                    return true;
                }
                b.tiles[b.order[n]] = true;
                b.run = Run::Close(n + 1);
                return false;
            }
        }
        let unplaced = |placed: u16| (0..12).filter(move |&i| earned >> i & 1 != 0 && placed >> i & 1 == 0);
        // The next earned, unplaced piece from `from` (0x8009254c).
        let next = |placed: u16, from: usize| (0..12).map(|k| (from + k) % 12).find(|&i| earned >> i & 1 != 0 && placed >> i & 1 == 0);
        let mut end = false;
        match b.state {
            State::Start => {
                b.selected = None;
                b.cursor = None;
                b.state = State::Select;
            }
            State::Select => {
                if unplaced(self.placed).count() == 1 {
                    b.selected = next(self.placed, 0);
                    b.state = State::Place;
                } else if [input::UP, input::DOWN, input::LEFT, input::RIGHT].iter().any(|&k| pads.pressed(0, k)) {
                    self.sounds.push(MOVE);
                    let from = b.cursor.map_or(0, |c| c + 1);
                    b.cursor = next(self.placed, from);
                } else if pads.pressed(0, input::CROSS) && b.cursor.is_some() {
                    self.sounds.push(OK);
                    b.selected = b.cursor.take();
                    b.state = State::Place;
                } else if pads.pressed(0, input::TRIANGLE) {
                    self.sounds.push(BACK);
                    end = true;
                }
            }
            State::Place => {
                let Some(i) = b.selected else {
                    b.state = State::Start;
                    return false;
                };
                let p = &mut b.pos[i];
                for (k, d) in [(input::UP, (0, -3)), (input::DOWN, (0, 3)), (input::LEFT, (-7, 0)), (input::RIGHT, (7, 0))] {
                    if pads.held(0, k) {
                        p.0 = (p.0 + d.0).clamp(-200, 200);
                        p.1 = (p.1 + d.1).clamp(-76, 76);
                    }
                    if pads.pressed(0, k) {
                        self.sounds.push(MOVE);
                    }
                }
                let s = SLOT[i];
                let dx = (s.0 as i32 - p.0 as i32) * 320 / 512;
                let dy = s.1 as i32 - p.1 as i32;
                if pads.pressed(0, input::TRIANGLE) {
                    self.sounds.push(BACK);
                    *p = TRAY[i];
                    if unplaced(self.placed).count() == 1 {
                        end = true;
                    } else {
                        b.state = State::Start;
                    }
                } else if pads.pressed(0, input::CROSS) {
                    if dx * dx + dy * dy <= 100 {
                        self.sounds.push(OK);
                        *p = s;
                        self.placed |= 1 << i;
                        b.selected = None;
                        b.state = State::Start;
                    } else {
                        self.sounds.push(MOVE);
                        *p = TRAY[i];
                    }
                }
            }
            State::Terrific(left) => {
                if left < crate::player::DT {
                    end = true;
                } else {
                    b.state = State::Terrific(left - crate::player::DT);
                }
            }
            State::Over => {}
        }
        if !matches!(b.state, State::Terrific(_) | State::Over) && unplaced(self.placed).count() == 0 && !end {
            b.state = State::Terrific(150);
        }
        if end {
            b.state = State::Over;
        }
        if b.state == State::Over && matches!(b.run, Run::Open(35)) {
            b.run = Run::Wait;
        }
        false
    }

    /// Text centred across, with its shadow 2 right and down.
    fn text(&self, s: &str, x: Option<i16>, y: i16, font: u8, rgb: [u8; 3], out: &mut Vec<Vtx>) {
        let mut t = Text { font, spacing: 1, rgb, x: x.unwrap_or(0), y, string: s.to_string(), ..Default::default() };
        if x.is_none() {
            self.fonts.centre(&mut t, true, false);
        }
        let mut shadow = t.clone();
        shadow.rgb = SHADOW;
        shadow.x += 2;
        shadow.y += 2;
        shadow.triangles(&self.fonts, out);
        t.triangles(&self.fonts, out);
    }

    fn draw(&mut self) {
        self.stage.under.clear();
        self.backdrop.triangles(&mut self.stage.under);
        let eye = Vec3::new(0.0, CAMERA_Y, self.z as f32);
        self.stage.camera = crate::stage::view(eye, Mat3::IDENTITY, self.focal);
        let mut moving = Vec::new();
        let mut overlay = Vec::new();
        let page = self.pages.first().cloned();
        match (&page, &self.board) {
            (Some(Page::Board), Some(b)) => self.draw_board(b, &mut overlay),
            (Some(Page::Restart), _) => {
                if let Some(m) = &self.burst {
                    rugrats_render::models::model_triangles(m, Mat3::IDENTITY, Vec3::ZERO, &mut moving);
                }
                // The chosen item yellow (0x13cdeb), the other white.
                let colour = |k: u8| if self.cursor == k { YELLOW } else { WHITE };
                self.text(&self.restart_text[0], None, -55, 1, WHITE, &mut overlay);
                self.text(&self.restart_text[1], Some(-90), -20, 0, colour(1), &mut overlay);
                self.text(&self.restart_text[2], Some(45), -20, 0, colour(2), &mut overlay);
                self.text(&self.labels[0], Some(-192), -93, 2, WHITE, &mut overlay);
                self.text(&self.labels[1], Some(115), -93, 2, WHITE, &mut overlay);
            }
            (Some(Page::Message { lines, bars, blink }), _) => {
                if let Some(m) = &self.burst {
                    rugrats_render::models::model_triangles(m, Mat3::IDENTITY, Vec3::ZERO, &mut moving);
                }
                if self.fly == Fly::Hold {
                    for (k, (s, c)) in lines.iter().enumerate() {
                        self.text(s, None, -65 + 25 * k as i16, 1, *c, &mut overlay);
                    }
                }
                if let (Some(n), Some(sp), true) = (bars, &self.sprites, self.fly != Fly::Out) {
                    sp.draw(BAR_ICON, (192, 76), 52.0, 64.0, 1024.0, 0.0, self.focal, &mut overlay);
                    self.text(&n.to_string(), Some(120), 76, 1, WHITE, &mut overlay);
                }
                // The lowest earned, unplaced piece blinks at (0, 45)
                // (0x80092d9c).
                if *blink && self.frames % 30 < 15 {
                    if let (Some(i), Some(sp)) = ((0..12).find(|&i| self.earned >> i & 1 != 0 && self.placed >> i & 1 == 0), &self.sprites) {
                        sp.draw(PIECE_SHADOW + i as u32, (2, 47), 64.0, 64.0, 1024.0, 0.0, self.focal, &mut overlay);
                        sp.draw(PIECE + i as u32, (0, 45), 64.0, 64.0, 1024.0, 0.0, self.focal, &mut overlay);
                    }
                }
                // PRESS X TO CONTINUE (0x80035798) and its cube.
                self.text(&self.press, Some(-96), 80, 2, WHITE, &mut overlay);
                if let Some(m) = &self.button_model {
                    let e = self.trig.euler(self.cube.angles);
                    let rows = Mat3::from_cols_array(&e.concat().into_iter().map(|v| v as f32 / 4096.0).collect::<Vec<_>>().try_into().unwrap()).transpose();
                    let shift = eye - BUTTON_CAMERA;
                    let pos = Vec3::new(self.cube.pos[0] as f32, self.cube.pos[1] as f32, self.cube.pos[2] as f32) + shift;
                    rugrats_render::models::model_triangles(m, rows.transpose(), pos, &mut moving);
                }
            }
            _ => {}
        }
        self.stage.set_moving(moving);
        self.stage.overlay = overlay;
    }

    /// The board's draw (0x80092450, 0x800921bc).
    fn draw_board(&self, b: &Board, out: &mut Vec<Vtx>) {
        let Some(sp) = &self.sprites else { return };
        let f = self.focal;
        sp.draw(BOARD, (-1, 2), 128.0, 169.0, 1024.0, 0.0, f, out);
        // The titles at y -90.
        let title = match b.state {
            State::Select | State::Start => Some(0),
            State::Place => Some(1),
            State::Terrific(_) => Some(2),
            State::Over => None,
        };
        if let Some(t) = title {
            self.text(&self.titles[t], None, -90, 1, PINK, out);
        }
        // The pieces: the others, then the selected or blinking cursor.
        for i in 0..12 {
            if Some(i) == b.selected || Some(i) == b.cursor {
                continue;
            }
            let p = b.pos[i];
            if self.earned >> i & 1 != 0 {
                let z = if self.placed >> i & 1 != 0 { 1024.0 } else { 1536.0 };
                // Added later in the ordering table, drawn first: the
                // shadow goes under.
                sp.draw(PIECE_SHADOW + i as u32, (p.0 + 2, p.1 + 2), 64.0, 64.0, z, 0.0, f, out);
                sp.draw(PIECE + i as u32, p, 64.0, 64.0, z, 0.0, f, out);
            } else {
                sp.draw(PIECE_GREY + i as u32, p, 64.0, 64.0, 1536.0, 0.0, f, out);
            }
        }
        let shown = b.selected.or(b.cursor.filter(|_| self.frames % 30 < 15));
        if let Some(i) = shown {
            let p = b.pos[i];
            sp.draw(PIECE_SHADOW + i as u32, (p.0 + 2, p.1 + 2), 64.0, 64.0, 1024.0, 0.0, f, out);
            sp.draw(PIECE + i as u32, p, 64.0, 64.0, 1024.0, 0.0, f, out);
        }
        // The cover tiles (0x80091f7c): a 7 x 5 grid.
        for (k, &on) in b.tiles.iter().enumerate() {
            if on {
                let (c, r) = ((k % 7) as i16, (k / 7) as i16);
                sp.draw(TILE, (-267 + 89 * c, -104 + 52 * r), 96.0, 92.0, 1024.0, 0.0, f, out);
            }
        }
        // CANCEL and ACCEPT (0x80035550, 0x80035674).
        self.text(&self.labels[0], Some(-192), -93, 2, WHITE, out);
        self.text(&self.labels[1], Some(115), -93, 2, WHITE, out);
    }
}
