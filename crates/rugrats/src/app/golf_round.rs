//! An activity mini-golf round (0x80096a30): players take turns at each
//! hole, the score card between holes.

use crate::disc::GameDisc;
use crate::levels::golf::FACES;
use crate::ui::front;
use crate::ui::screens::{PINK, Page};

/// Each player's face slot, strokes by hole, the hole and player in play.
pub(super) struct GolfRound {
    pub slots: Vec<usize>,
    pub strokes: Vec<[u32; 9]>,
    pub hole: usize,
    pub player: usize,
}

impl GolfRound {
    pub fn new(slots: Vec<usize>) -> GolfRound {
        let n = slots.len();
        GolfRound { slots, strokes: vec![[0; 9]; n], hole: 0, player: 0 }
    }

    /// The body of the player in turn.
    pub fn character(&self) -> usize {
        FACES[self.slots[self.player] % 6]
    }

    /// The turn banner (0x800965e8): "Player %d" / "Hole %d" in pink.
    pub fn banner(&self, disc: &GameDisc) -> Page {
        let s = |n: u32, v: usize| front::string(disc, n).replace("%d", &v.to_string());
        Page::Message {
            lines: vec![(s(203, self.player + 1), PINK), (s(204, self.hole + 1), PINK)],
            bars: None,
            blink: false,
        }
    }

    /// The score card (0x80095768): a row a player with his face.
    pub fn card(&self, press: bool) -> Page {
        Page::Scores { rows: self.slots.iter().copied().zip(self.strokes.iter().copied()).collect(), press }
    }
}
