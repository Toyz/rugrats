//! Full-screen stills: a `.BS` picture shown on its own (`show_still`,
//! SLUS_006.50:0x80030010), the legal screen at boot.

use rrt::image::Picture;
use rugrats_data::{bs, mdec::Tables};

/// The stills are 512 x 240, the display mode they are shown in.
pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 240;

/// Vertical blanks a still holds the screen, from the call on.
pub const HOLD: u64 = 250;

/// The colour the still's quads are drawn with (0x800c3f9c): 0x7f on each
/// channel, so every texel is scaled by 127/128.
const SHADE: u32 = 0x7f;

/// The still as the GPU puts it on screen.
pub fn picture(bytes: &[u8], tables: &Tables) -> Result<Picture, bs::BsError> {
    let px = bs::decode(bytes, WIDTH, HEIGHT, tables)?;
    let mut pic = Picture::filled(WIDTH as u32, HEIGHT as u32, [0, 0, 0, 255]);
    for (i, &p) in px.iter().enumerate() {
        let ch = |c: u16| {
            let v = ((c as u32 & 31) * SHADE >> 7).min(31) as u8;
            v << 3 | v >> 2
        };
        pic.set((i % WIDTH) as u32, (i / WIDTH) as u32, [ch(p), ch(p >> 5), ch(p >> 10), 255]);
    }
    Ok(pic)
}
