//! Sony TIM images: the standard PlayStation texture file.
//!
//! ```text
//! u32   magic      0x00000010
//! u32   flags      bits 0-2 pixel mode (0 4bpp, 1 8bpp, 2 15bpp, 3 24bpp), bit 3 has CLUT
//! clut block (when bit 3)
//!   u32 length     bytes, including these 12
//!   u16 x, y       VRAM position
//!   u16 w, h       colours per palette, palettes
//!   u16 colours[w*h]
//! image block
//!   u32 length
//!   u16 x, y       VRAM position, in 16-bit units
//!   u16 w, h       width in 16-bit units, height in lines
//!   u16 data[w*h]
//! ```

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Bpp4,
    Bpp8,
    Bpp15,
    Bpp24,
}

#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Clone, Debug)]
pub struct Tim {
    pub mode: Mode,
    /// Palette block: VRAM rectangle and the colours, row after row.
    pub clut: Option<(Rect, Vec<u16>)>,
    /// Image block: VRAM rectangle in 16-bit units and the raw halfwords.
    pub rect: Rect,
    pub data: Vec<u16>,
}

#[derive(Debug)]
pub struct TimError(pub String);

impl fmt::Display for TimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TIM: {}", self.0)
    }
}

impl std::error::Error for TimError {}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

/// A block, sized by its rectangle. The length field is not trusted: the 492
/// car skins on this disc store the pixel bytes alone there (32768), without
/// the 12-byte block header that the standard counts.
fn block(b: &[u8], at: usize) -> Option<(Rect, Vec<u16>, usize)> {
    let len = u32_at(b, at)? as usize;
    let rect = Rect { x: u16_at(b, at + 4)?, y: u16_at(b, at + 6)?, w: u16_at(b, at + 8)?, h: u16_at(b, at + 10)? };
    let n = rect.w as usize * rect.h as usize;
    if len != 12 + n * 2 && len != n * 2 {
        return None;
    }
    let data = (0..n).map(|i| u16_at(b, at + 12 + 2 * i)).collect::<Option<Vec<_>>>()?;
    Some((rect, data, at + 12 + n * 2))
}

/// The 15-bit colour as RGBA8. Black with the STP bit clear is transparent,
/// as the GPU draws textures.
pub fn rgba(c: u16) -> [u8; 4] {
    let ex = |v: u16| ((v & 31) << 3 | (v & 31) >> 2) as u8;
    [ex(c), ex(c >> 5), ex(c >> 10), if c == 0 { 0 } else { 255 }]
}

impl Tim {
    pub fn parse(b: &[u8]) -> Result<Tim, TimError> {
        let err = |m: &str| TimError(m.to_string());
        if u32_at(b, 0) != Some(0x10) {
            return Err(err("no TIM magic"));
        }
        let flags = u32_at(b, 4).ok_or(err("short header"))?;
        let mode = match flags & 7 {
            0 => Mode::Bpp4,
            1 => Mode::Bpp8,
            2 => Mode::Bpp15,
            3 => Mode::Bpp24,
            m => return Err(TimError(format!("pixel mode {m}"))),
        };
        let mut at = 8;
        let clut = if flags & 8 != 0 {
            let (rect, data, next) = block(b, at).ok_or(err("bad CLUT block"))?;
            at = next;
            Some((rect, data))
        } else {
            None
        };
        let (rect, data, _) = block(b, at).ok_or(err("bad image block"))?;
        Ok(Tim { mode, clut, rect, data })
    }

    /// Width in pixels.
    pub fn width(&self) -> usize {
        let w = self.rect.w as usize;
        match self.mode {
            Mode::Bpp4 => w * 4,
            Mode::Bpp8 => w * 2,
            Mode::Bpp15 => w,
            Mode::Bpp24 => w * 2 / 3,
        }
    }

    pub fn height(&self) -> usize {
        self.rect.h as usize
    }

    /// Number of palettes in the CLUT block.
    pub fn palettes(&self) -> usize {
        self.clut.as_ref().map_or(0, |(r, _)| r.h as usize)
    }

    /// The palette index of each pixel, for the indexed modes.
    pub fn indices(&self) -> Option<Vec<u8>> {
        let bytes: Vec<u8> = self.data.iter().flat_map(|h| h.to_le_bytes()).collect();
        match self.mode {
            Mode::Bpp4 => Some(bytes.iter().flat_map(|&b| [b & 15, b >> 4]).collect()),
            Mode::Bpp8 => Some(bytes),
            _ => None,
        }
    }

    /// The image as RGBA8, the indexed modes through palette `palette`.
    pub fn to_rgba(&self, palette: usize) -> Vec<u8> {
        let (w, h) = (self.width(), self.height());
        let mut out = Vec::with_capacity(w * h * 4);
        match self.mode {
            Mode::Bpp4 | Mode::Bpp8 => {
                let (rect, colours) = self.clut.as_ref().expect("indexed TIM without CLUT");
                let row = palette.min(rect.h.saturating_sub(1) as usize) * rect.w as usize;
                for i in self.indices().unwrap() {
                    out.extend(rgba(colours.get(row + i as usize).copied().unwrap_or(0)));
                }
            }
            Mode::Bpp15 => {
                for &c in &self.data {
                    out.extend(rgba(c));
                }
            }
            Mode::Bpp24 => {
                let bytes: Vec<u8> = self.data.iter().flat_map(|h| h.to_le_bytes()).collect();
                let stride = self.rect.w as usize * 2;
                for y in 0..h {
                    for x in 0..w {
                        let p = &bytes[y * stride + x * 3..y * stride + x * 3 + 3];
                        out.extend([p[0], p[1], p[2], 255]);
                    }
                }
            }
        }
        out
    }
}
