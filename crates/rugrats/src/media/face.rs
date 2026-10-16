//! Faces: the characters' mouths and eyes are blocks of their texture
//! that the cinematics swap in VRAM frame by frame, from banks of frames
//! (files 531-550) a database's face mask asks for (loaded by
//! SLUS_006.50:0x80074b9c, the mask at 0x800c4c70).
//!
//! The stream sets a mouth frame (0x80074ec0) and an eye frame
//! (0x80075304); a posed character also blinks on its own (0x80075698).

use std::collections::HashMap;

use rugrats_render::Vram;

use crate::disc::GameDisc;
use crate::rand::Rand;

/// The bank each mask bit loads; a bank's frames start 64 bytes in.
const BANKS: [usize; 20] =
    [535, 531, 536, 532, 537, 545, 538, 546, 539, 540, 541, 542, 543, 544, 533, 534, 547, 548, 549, 550];
const BANK_HEADER: usize = 64;

/// The face masks by database (0x800b91a0, a word each).
const MASKS: u32 = 0x800b_91a0;

/// One character's block: its ids, the mask bit of its bank, where it
/// sits in VRAM (x, y, w, h in words), and the frame count past which a
/// frame wraps back once.
struct Block {
    ids: &'static [i16],
    bit: u8,
    rect: (u16, u16, u16, u16),
    wrap: u8,
}

const fn block(ids: &'static [i16], bit: u8, rect: (u16, u16, u16, u16), wrap: u8) -> Block {
    Block { ids, bit, rect, wrap }
}

/// 0x80074ec0's cases.
const MOUTHS: [Block; 11] = [
    block(&[1492], 1, (704, 256, 8, 32), 10),
    block(&[600], 5, (736, 320, 8, 32), 10),
    block(&[1776], 3, (704, 320, 8, 32), 10),
    block(&[608, 609], 13, (704, 384, 8, 32), 10),
    block(&[75], 17, (808, 384, 8, 32), 10),
    block(&[76], 19, (800, 352, 8, 32), 10),
    block(&[15], 9, (864, 288, 8, 32), 10),
    block(&[16], 11, (784, 480, 8, 32), 10),
    block(&[13], 7, (856, 384, 8, 32), 10),
    block(&[88], 14, (160, 384, 16, 32), 12),
    block(&[112], 15, (256, 256, 32, 96), 13),
];

/// 0x80075304's cases (no wrap).
const EYES: [Block; 11] = [
    block(&[13], 6, (832, 384, 8, 32), 255),
    block(&[15], 8, (776, 352, 8, 32), 255),
    block(&[16], 10, (816, 320, 8, 32), 255),
    block(&[608, 609], 12, (720, 384, 16, 64), 255),
    block(&[1776], 2, (736, 256, 16, 64), 255),
    block(&[600], 4, (720, 320, 16, 64), 255),
    block(&[1492], 0, (720, 256, 16, 64), 255),
    block(&[75], 16, (776, 352, 8, 32), 255),
    block(&[76], 18, (816, 384, 8, 32), 255),
    block(&[88], 14, (160, 384, 16, 32), 255),
    block(&[112], 15, (256, 256, 32, 96), 255),
];

/// The faces as VRAM holds them: every block's words. The PlayStation has
/// one VRAM, so what the last cinematic put there (its last frame's
/// mouths and eyes) is what the characters wear in play; the port keeps
/// each screen's VRAM apart and carries these blocks over.
#[derive(Clone, Debug, Default)]
pub struct FaceBlocks(Vec<((u16, u16, u16, u16), Vec<u16>)>);

impl FaceBlocks {
    /// The blocks as `vram` has them.
    pub fn capture(vram: &Vram) -> FaceBlocks {
        let mut rects: Vec<(u16, u16, u16, u16)> = MOUTHS.iter().chain(EYES.iter()).map(|b| b.rect).collect();
        rects.sort();
        rects.dedup();
        FaceBlocks(
            rects
                .into_iter()
                .map(|(x, y, w, h)| {
                    let words = (y..y + h)
                        .flat_map(|row| {
                            let at = row as usize * 1024 + x as usize;
                            vram.words[at..at + w as usize].iter().copied()
                        })
                        .collect();
                    ((x, y, w, h), words)
                })
                .collect(),
        )
    }

    /// Puts the blocks into `vram`.
    pub fn apply(&self, vram: &mut Vram) {
        for &((x, y, w, h), ref words) in &self.0 {
            vram.load(x, y, w, h, words);
        }
    }
}

/// A database's face banks.
pub struct Faces {
    banks: Vec<Option<Vec<u16>>>,
}

impl Faces {
    /// The banks of mask `mask`.
    pub fn load(disc: &GameDisc, mask: u32) -> Result<Faces, String> {
        let mut banks = Vec::new();
        for (bit, &file) in BANKS.iter().enumerate() {
            banks.push(if mask >> bit & 1 != 0 {
                let raw = disc.file(file)?;
                Some(
                    raw[BANK_HEADER.min(raw.len())..]
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|h| u16::from_le_bytes([h[0], h[1]]))
                        .collect(),
                )
            } else {
                None
            });
        }
        Ok(Faces { banks })
    }

    /// Database `db`'s mask, as a level's setup (0x80017c0c) and the
    /// opening's (0x8008f078) take it.
    pub fn mask(disc: &GameDisc, db: usize) -> Result<u32, String> {
        disc.exe_u32(MASKS + 4 * db as u32)
    }

    fn put(&self, blocks: &[Block], id: i16, frame: u8, vram: &mut Vram) {
        let Some(b) = blocks.iter().find(|b| b.ids.contains(&id)) else { return };
        let Some(Some(bank)) = self.banks.get(b.bit as usize) else { return };
        let frame = if frame >= b.wrap { frame - b.wrap } else { frame };
        let (x, y, w, h) = b.rect;
        let size = w as usize * h as usize;
        // LoadImage reads on past the bank's end; the port leaves VRAM be.
        let Some(words) = bank.get(frame as usize * size..(frame as usize + 1) * size) else { return };
        vram.load(x, y, w, h, words);
    }

    /// Mouth frame `frame` for `id` (0x80074ec0).
    pub fn mouth(&self, id: i16, frame: u8, vram: &mut Vram) {
        self.put(&MOUTHS, id, frame, vram);
    }

    /// Eye frame `frame` for `id` (0x80075304).
    pub fn eyes(&self, id: i16, frame: u8, vram: &mut Vram) {
        self.put(&EYES, id, frame, vram);
    }
}

/// The blinks (0x80075698): each character's blink state (+0xa5) and
/// count (+0xa7), on a rand of their own (0x800c4c48) that play_cine seeds
/// with the cinematic's number.
pub struct Blinks {
    rand: Rand,
    state: HashMap<i16, (u8, u8)>,
}

impl Blinks {
    pub fn new(seed: u32) -> Blinks {
        Blinks { rand: Rand { state: seed }, state: HashMap::new() }
    }

    /// The rand's state (0x800c4c48).
    pub fn seed(&self) -> u32 {
        self.rand.state
    }

    /// One frame of `id`'s blink: the eye frame it shows. Open, a 1 in 32
    /// chance of starting; then frame 1 for three frames and 2 for two.
    pub fn step(&mut self, id: i16) -> u8 {
        let (state, count) = self.state.entry(id).or_default();
        match *state {
            0 => {
                if self.rand.next() & 31 == 0 {
                    (*state, *count) = (1, 1);
                }
            }
            1 if *count == 0 => (*state, *count) = (2, 1),
            2 if *count == 0 => *state = 0,
            _ => *count -= 1,
        }
        *state
    }
}
