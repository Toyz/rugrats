//! The MDEC's decoding: run-length coded blocks in, pixels out. This is the
//! console's motion decoder as a pure function (dequantise, zigzag, IDCT,
//! YUV to RGB), shared by the reference's hardware model and the port's
//! movie and still-picture players.
//!
//! ```text
//! a block, halfwords:
//!   first   q_scale:6 | dc:10       (0xfe00 before it is padding)
//!   then    run:6 | level:10        run zero coefficients, then level
//!   0xfe00  end of block
//! a colour macroblock: Cr, Cb, Y0 Y1 Y2 Y3 (each 8x8), 16x16 pixels out
//! ```

/// Zigzag order: coefficient k of the stream goes to `ZIGZAG[k]` of the 8x8.
pub const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21,
    28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61,
    54, 47, 55, 62, 63,
];

/// End of block, and the padding before a block.
pub const EOB: u16 = 0xfe00;

/// The tables the decoder holds: luma and chroma quantisation, and the IDCT
/// matrix ("scale table").
#[derive(Clone, Debug)]
pub struct Tables {
    pub luma: [u8; 64],
    pub chroma: [u8; 64],
    pub scale: [i16; 64],
}

impl Default for Tables {
    fn default() -> Self {
        Tables { luma: [0; 64], chroma: [0; 64], scale: [0; 64] }
    }
}

/// How the decoder writes pixels: the command's bits 25-28.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Bits4,
    Bits8,
    Bits24,
    Bits15,
}

impl Depth {
    pub fn from_command(cmd: u32) -> Depth {
        match (cmd >> 27) & 3 {
            0 => Depth::Bits4,
            1 => Depth::Bits8,
            2 => Depth::Bits24,
            _ => Depth::Bits15,
        }
    }
}

fn signed10(v: u16) -> i32 {
    ((v as i32) << 22) >> 22
}

/// Reads one block from `src` at `*at`, returning the 8x8 coefficients in
/// raster order, or None when the stream ran out.
pub fn read_block(src: &[u16], at: &mut usize, qt: &[u8; 64]) -> Option<[i32; 64]> {
    let mut blk = [0i32; 64];
    let mut n = *src.get(*at)?;
    *at += 1;
    while n == EOB {
        n = *src.get(*at)?;
        *at += 1;
    }
    let q = ((n >> 10) & 0x3f) as i32;
    let mut k = 0usize;
    let mut val = signed10(n) * qt[0] as i32;
    loop {
        if q == 0 {
            val = signed10(n) * 2;
        }
        val = val.clamp(-0x400, 0x3ff);
        if q > 0 {
            blk[ZIGZAG[k]] = val;
        } else {
            blk[k] = val;
        }
        n = *src.get(*at)?;
        *at += 1;
        k += ((n >> 10) & 0x3f) as usize + 1;
        if k > 63 {
            break;
        }
        val = (signed10(n) * qt[k] as i32 * q + 4) / 8;
    }
    Some(blk)
}

/// The inverse DCT through the scale table, as the MDEC computes it: two
/// passes of 64-bit sums, rounded, then saturated to signed 8 bits.
pub fn idct(blk: &[i32; 64], scale: &[i16; 64]) -> [i32; 64] {
    let mut temp = [0i64; 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0i64;
            for u in 0..8 {
                sum += blk[u * 8 + x] as i64 * scale[u * 8 + y] as i64;
            }
            temp[x + y * 8] = sum;
        }
    }
    let mut out = [0i32; 64];
    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0i64;
            for u in 0..8 {
                sum += temp[u + y * 8] * scale[u * 8 + x] as i64;
            }
            let v = (sum >> 32) + ((sum >> 31) & 1);
            // Sign-extended from 9 bits, then saturated.
            let v = ((v as i32) << 23) >> 23;
            out[x + y * 8] = v.clamp(-128, 127);
        }
    }
    out
}

/// A decoded colour macroblock: 16x16 RGB, each channel signed (-128..127)
/// or, with `signed` false, 0..255.
pub fn macroblock_rgb(cr: &[i32; 64], cb: &[i32; 64], y: [&[i32; 64]; 4], signed: bool) -> [[u8; 3]; 256] {
    let mut out = [[0u8; 3]; 256];
    for py in 0..16 {
        for px in 0..16 {
            let c = (py / 2) * 8 + px / 2;
            let (r, b) = (cr[c] as f64, cb[c] as f64);
            let lum = y[(py / 8) * 2 + px / 8][(py % 8) * 8 + px % 8];
            // The MDEC's YUV to RGB, in its fixed point.
            let rr = (1.402 * r) as i32;
            let gg = (-0.3437 * b) as i32 + (-0.7143 * r) as i32;
            let bb = (1.772 * b) as i32;
            let ch = |v: i32| {
                let v = (lum + v).clamp(-128, 127);
                (if signed { v } else { v + 128 }) as u8
            };
            out[py * 16 + px] = [ch(rr), ch(gg), ch(bb)];
        }
    }
    out
}

/// One monochrome block as 8x8 luma, signed or offset to 0..255.
pub fn block_mono(yb: &[i32; 64], signed: bool) -> [u8; 64] {
    let mut out = [0u8; 64];
    for (o, &v) in out.iter_mut().zip(yb.iter()) {
        *o = (if signed { v } else { v + 128 }) as u8;
    }
    out
}

/// Decodes as many whole macroblocks as `src` holds into the words the MDEC
/// would put out for `depth`: 15-bit and 24-bit colour macroblocks of 16x16,
/// or 8x8 luma for 4- and 8-bit.
pub fn decode(src: &[u16], t: &Tables, depth: Depth, signed: bool, bit15: bool) -> Vec<u32> {
    let mut out_bytes: Vec<u8> = Vec::new();
    let mut at = 0usize;
    match depth {
        Depth::Bits15 | Depth::Bits24 => loop {
            let blocks: Option<Vec<[i32; 64]>> = (0..6)
                .map(|i| read_block(src, &mut at, if i < 2 { &t.chroma } else { &t.luma }).map(|b| idct(&b, &t.scale)))
                .collect();
            let Some(b) = blocks else { break };
            let px = macroblock_rgb(&b[0], &b[1], [&b[2], &b[3], &b[4], &b[5]], signed);
            for p in px {
                if depth == Depth::Bits24 {
                    out_bytes.extend(p);
                } else {
                    let v = (p[0] as u16 >> 3) | (p[1] as u16 >> 3) << 5 | (p[2] as u16 >> 3) << 10 | (bit15 as u16) << 15;
                    out_bytes.extend(v.to_le_bytes());
                }
            }
        },
        Depth::Bits8 | Depth::Bits4 => {
            while let Some(b) = read_block(src, &mut at, &t.luma) {
                let px = block_mono(&idct(&b, &t.scale), signed);
                if depth == Depth::Bits8 {
                    out_bytes.extend(px);
                } else {
                    out_bytes.extend(px.chunks(2).map(|p| (p[0] >> 4) | (p[1] >> 4) << 4));
                }
            }
        }
    }
    out_bytes.resize(out_bytes.len().div_ceil(4) * 4, 0);
    out_bytes.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dc_only_block_reads_back_at_its_zigzag_start() {
        let mut qt = [1u8; 64];
        qt[0] = 2;
        // q_scale 1, dc 5; end of block.
        let src = [EOB, 1 << 10 | 5, EOB];
        let mut at = 0;
        let b = read_block(&src, &mut at, &qt).unwrap();
        assert_eq!(b[0], 10);
        assert!(b[1..].iter().all(|&v| v == 0));
        assert_eq!(at, 3);
    }

    #[test]
    fn a_run_skips_coefficients_and_lands_in_zigzag_order() {
        let qt = [8u8; 64];
        // q_scale 2, dc 0; then run 2 level 3: coefficient k = 3 -> raster 16.
        let src = [2 << 10, 2 << 10 | 3, EOB];
        let mut at = 0;
        let b = read_block(&src, &mut at, &qt).unwrap();
        assert_eq!(b[ZIGZAG[3]], (3 * 8 * 2 + 4) / 8);
        assert_eq!(ZIGZAG[3], 16);
    }

    #[test]
    fn negative_levels_divide_toward_zero() {
        let qt = [3u8; 64];
        // level -1 at k = 1, q_scale 1: (-1 * 3 * 1 + 4) / 8 = 0.
        let src = [1 << 10, 0x3ff, EOB];
        let mut at = 0;
        let b = read_block(&src, &mut at, &qt).unwrap();
        assert_eq!(b[ZIGZAG[1]], 0);
    }

    #[test]
    fn a_flat_block_through_a_flat_matrix_stays_flat() {
        // A scale table whose first row is 2^14 / sqrt(8)-ish makes the DC
        // spread evenly; any constant first row does.
        let mut scale = [0i16; 64];
        for x in 0..8 {
            scale[x] = 0x5a82;
        }
        let mut blk = [0i32; 64];
        blk[0] = 100;
        let out = idct(&blk, &scale);
        assert!(out.iter().all(|&v| v == out[0]));
    }
}
