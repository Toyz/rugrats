//! `.BS` still pictures: MDEC run-length blocks packed with MPEG-1's AC
//! codes, decoded on the CPU by libpress and then by the MDEC.
//!
//! ```text
//! header, little-endian halfwords
//!   u16  words      run-length data in 32-bit words, once decoded
//!   u16  0x3800     (with `words`, the MDEC decode command)
//!   u16  q_scale
//!   u16  version    2 (3 has differential DC; not on the disc's stills but E3SCR512)
//! bitstream: 16-bit little-endian words, read most significant bit first
//!   per block (Cr, Cb, Y0..Y3 per macroblock, macroblocks down each
//!   16-pixel column, columns left to right):
//!     dc:10          signed
//!     ac codes       MPEG-1 table B.14, "10" ends the block,
//!                    "000001" escapes to run:6 level:10
//! ```

use crate::mdec::{self, Depth, Tables};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub words: u16,
    pub q_scale: u16,
    pub version: u16,
}

#[derive(Debug)]
pub struct BsError(pub String);

impl std::fmt::Display for BsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BS: {}", self.0)
    }
}

impl std::error::Error for BsError {}

pub fn header(bytes: &[u8]) -> Result<Header, BsError> {
    let h = |k: usize| u16::from_le_bytes([bytes[2 * k], bytes[2 * k + 1]]);
    if bytes.len() < 8 || h(1) != 0x3800 {
        return Err(BsError("no 0x3800 magic".into()));
    }
    Ok(Header { words: h(0), q_scale: h(2), version: h(3) })
}

struct Bits<'a> {
    bytes: &'a [u8],
    /// Bits consumed.
    at: usize,
}

impl Bits<'_> {
    fn bit(&mut self) -> Option<u32> {
        let word = self.at / 16;
        let b = u16::from_le_bytes([*self.bytes.get(word * 2)?, *self.bytes.get(word * 2 + 1)?]);
        let v = (b >> (15 - self.at % 16)) & 1;
        self.at += 1;
        Some(v as u32)
    }

    fn take(&mut self, n: u32) -> Option<u32> {
        (0..n).try_fold(0, |v, _| Some(v << 1 | self.bit()?))
    }
}

/// MPEG-1 table B.14 without the sign bit: (code, length, run, level).
const AC: &[(u16, u8, u8, u8)] = &[
    (0b11, 2, 0, 1),
    (0b011, 3, 1, 1),
    (0b0100, 4, 0, 2),
    (0b0101, 4, 2, 1),
    (0b00101, 5, 0, 3),
    (0b00111, 5, 3, 1),
    (0b00110, 5, 4, 1),
    (0b000110, 6, 1, 2),
    (0b000111, 6, 5, 1),
    (0b000101, 6, 6, 1),
    (0b000100, 6, 7, 1),
    (0b0000110, 7, 0, 4),
    (0b0000100, 7, 2, 2),
    (0b0000111, 7, 8, 1),
    (0b0000101, 7, 9, 1),
    (0b00100110, 8, 0, 5),
    (0b00100001, 8, 0, 6),
    (0b00100101, 8, 1, 3),
    (0b00100100, 8, 3, 2),
    (0b00100111, 8, 10, 1),
    (0b00100011, 8, 11, 1),
    (0b00100010, 8, 12, 1),
    (0b00100000, 8, 13, 1),
    (0b0000001010, 10, 0, 7),
    (0b0000001100, 10, 1, 4),
    (0b0000001011, 10, 2, 3),
    (0b0000001111, 10, 4, 2),
    (0b0000001001, 10, 5, 2),
    (0b0000001110, 10, 14, 1),
    (0b0000001101, 10, 15, 1),
    (0b0000001000, 10, 16, 1),
    (0b000000011101, 12, 0, 8),
    (0b000000011000, 12, 0, 9),
    (0b000000010011, 12, 0, 10),
    (0b000000010000, 12, 0, 11),
    (0b000000011011, 12, 1, 5),
    (0b000000010100, 12, 2, 4),
    (0b000000011100, 12, 3, 3),
    (0b000000010010, 12, 4, 3),
    (0b000000011110, 12, 6, 2),
    (0b000000010101, 12, 7, 2),
    (0b000000010001, 12, 8, 2),
    (0b000000011111, 12, 17, 1),
    (0b000000011010, 12, 18, 1),
    (0b000000011001, 12, 19, 1),
    (0b000000010111, 12, 20, 1),
    (0b000000010110, 12, 21, 1),
    (0b0000000011010, 13, 0, 12),
    (0b0000000011001, 13, 0, 13),
    (0b0000000011000, 13, 0, 14),
    (0b0000000010111, 13, 0, 15),
    (0b0000000010110, 13, 1, 6),
    (0b0000000010101, 13, 1, 7),
    (0b0000000010100, 13, 2, 5),
    (0b0000000010011, 13, 3, 4),
    (0b0000000010010, 13, 5, 3),
    (0b0000000010001, 13, 9, 2),
    (0b0000000010000, 13, 10, 2),
    (0b0000000011111, 13, 22, 1),
    (0b0000000011110, 13, 23, 1),
    (0b0000000011101, 13, 24, 1),
    (0b0000000011100, 13, 25, 1),
    (0b0000000011011, 13, 26, 1),
    (0b00000000011111, 14, 0, 16),
    (0b00000000011110, 14, 0, 17),
    (0b00000000011101, 14, 0, 18),
    (0b00000000011100, 14, 0, 19),
    (0b00000000011011, 14, 0, 20),
    (0b00000000011010, 14, 0, 21),
    (0b00000000011001, 14, 0, 22),
    (0b00000000011000, 14, 0, 23),
    (0b00000000010111, 14, 0, 24),
    (0b00000000010110, 14, 0, 25),
    (0b00000000010101, 14, 0, 26),
    (0b00000000010100, 14, 0, 27),
    (0b00000000010011, 14, 0, 28),
    (0b00000000010010, 14, 0, 29),
    (0b00000000010001, 14, 0, 30),
    (0b00000000010000, 14, 0, 31),
    (0b000000000011000, 15, 0, 32),
    (0b000000000010111, 15, 0, 33),
    (0b000000000010110, 15, 0, 34),
    (0b000000000010101, 15, 0, 35),
    (0b000000000010100, 15, 0, 36),
    (0b000000000010011, 15, 0, 37),
    (0b000000000010010, 15, 0, 38),
    (0b000000000010001, 15, 0, 39),
    (0b000000000010000, 15, 0, 40),
    (0b000000000011111, 15, 1, 8),
    (0b000000000011110, 15, 1, 9),
    (0b000000000011101, 15, 1, 10),
    (0b000000000011100, 15, 1, 11),
    (0b000000000011011, 15, 1, 12),
    (0b000000000011010, 15, 1, 13),
    (0b000000000011001, 15, 1, 14),
    (0b0000000000010011, 16, 1, 15),
    (0b0000000000010010, 16, 1, 16),
    (0b0000000000010001, 16, 1, 17),
    (0b0000000000010000, 16, 1, 18),
    (0b0000000000010100, 16, 6, 3),
    (0b0000000000011010, 16, 11, 2),
    (0b0000000000011001, 16, 12, 2),
    (0b0000000000011000, 16, 13, 2),
    (0b0000000000010111, 16, 14, 2),
    (0b0000000000010110, 16, 15, 2),
    (0b0000000000010101, 16, 16, 2),
    (0b0000000000011111, 16, 27, 1),
    (0b0000000000011110, 16, 28, 1),
    (0b0000000000011101, 16, 29, 1),
    (0b0000000000011100, 16, 30, 1),
    (0b0000000000011011, 16, 31, 1),
];

/// One AC code: Some((run, level)) or None at end of block.
fn ac(bits: &mut Bits) -> Result<Option<(u16, i32)>, BsError> {
    let eof = || BsError("the bitstream ends inside a block".into());
    let mut code = 0u16;
    for len in 1..=16u8 {
        code = code << 1 | bits.bit().ok_or_else(eof)? as u16;
        if len == 2 && code == 0b10 {
            return Ok(None);
        }
        if len == 6 && code == 0b000001 {
            let run = bits.take(6).ok_or_else(eof)? as u16;
            let level = bits.take(10).ok_or_else(eof)? as i32;
            return Ok(Some((run, (level << 22) >> 22)));
        }
        if let Some(&(_, _, run, level)) = AC.iter().find(|e| e.1 == len && e.0 == code) {
            let sign = bits.bit().ok_or_else(eof)?;
            let level = if sign == 1 { -(level as i32) } else { level as i32 };
            return Ok(Some((run as u16, level)));
        }
    }
    Err(BsError(format!("no AC code matches at bit {}", bits.at)))
}

/// The run-length halfwords libpress's DecDCTvlc hands the MDEC: `words`
/// 32-bit words, padded with end-of-block codes.
pub fn rle(bytes: &[u8]) -> Result<Vec<u16>, BsError> {
    let h = header(bytes)?;
    if h.version != 2 {
        return Err(BsError(format!("version {} is not decoded", h.version)));
    }
    let want = h.words as usize * 2;
    let mut out = Vec::with_capacity(want);
    let mut bits = Bits { bytes: &bytes[8..], at: 0 };
    let total_bits = (bytes.len() - 8) * 8;
    // The stream's tail holds less than a block; what is left of the
    // output is end-of-block padding.
    'blocks: while out.len() < want && bits.at + 10 <= total_bits {
        let mut block = vec![h.q_scale << 10 | bits.take(10).unwrap() as u16];
        loop {
            match ac(&mut bits) {
                Ok(Some((run, level))) => block.push(run << 10 | (level as u16 & 0x3ff)),
                Ok(None) => break,
                Err(_) if bits.at + 16 >= total_bits => break 'blocks,
                Err(e) => return Err(e),
            }
        }
        block.push(mdec::EOB);
        out.extend(block);
    }
    out.resize(want, mdec::EOB);
    Ok(out)
}

/// The picture, `width` x `height` (multiples of 16) RGB555 with bit 15
/// clear, as the MDEC turns the stream out in 15-bit mode.
pub fn decode(bytes: &[u8], width: usize, height: usize, tables: &Tables) -> Result<Vec<u16>, BsError> {
    let words = mdec::decode(&rle(bytes)?, tables, Depth::Bits15, false, false);
    let mut px = vec![0u16; width * height];
    let per_col = height / 16;
    for (mb, chunk) in words.chunks(128).enumerate() {
        let (mx, my) = (mb / per_col * 16, mb % per_col * 16);
        if mx >= width {
            break;
        }
        for (k, w) in chunk.iter().enumerate() {
            for (j, v) in [*w as u16, (*w >> 16) as u16].into_iter().enumerate() {
                let p = 2 * k + j;
                px[(my + p / 16) * width + mx + p % 16] = v;
            }
        }
    }
    Ok(px)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ac_table_is_prefix_free() {
        for (i, a) in AC.iter().enumerate() {
            for b in &AC[i + 1..] {
                let (short, long) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                assert_ne!(long.0 >> (long.1 - short.1), short.0, "{a:?} prefixes {b:?}");
            }
            assert_ne!(a.0 >> (a.1.saturating_sub(2)), 0b10, "{a:?} starts with EOB");
            if a.1 >= 6 {
                assert_ne!(a.0 >> (a.1 - 6), 0b000001, "{a:?} starts with the escape");
            }
        }
    }

    /// The stream libpress hands the MDEC for LEGAL512.BS, dumped from the
    /// original running in the oracle (RUGRATS_MDEC_DUMP), word for word.
    #[test]
    fn legal512_decodes_to_what_libpress_hands_the_mdec() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work");
        let (Ok(bs), Ok(dump)) = (
            std::fs::read(root.join("fs/DATA/MENU/TIMS/LEGAL512.BS")),
            std::fs::read(root.join("oracle/mdec/00000.mdec")),
        ) else {
            eprintln!("skipped: no work/fs or oracle dump");
            return;
        };
        let ours = rle(&bs).unwrap();
        let theirs: Vec<u16> = dump[4..].chunks(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
        assert_eq!(&dump[..4], &bs[..4]);
        assert_eq!(ours.len(), theirs.len());
        let first = ours.iter().zip(&theirs).position(|(a, b)| a != b);
        assert_eq!(first, None, "first difference at halfword {first:?}");
    }
}

