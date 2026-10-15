//! The VRAM image textures live in, and triangles as the shader takes them.

use rrt::kit::Pack;

/// The PlayStation's video memory: 1024 x 512 halfwords.
#[derive(Clone)]
pub struct Vram {
    pub words: Vec<u16>,
}

pub const VRAM_W: usize = 1024;
pub const VRAM_H: usize = 512;

impl Default for Vram {
    fn default() -> Self {
        Vram::new()
    }
}

impl Vram {
    pub fn new() -> Vram {
        Vram { words: vec![0; VRAM_W * VRAM_H] }
    }

    /// LoadImage: a rectangle of halfwords at (x, y), row by row, as far as
    /// `data` goes (DB00A.RAW is shorter than the rectangle it is loaded to).
    pub fn load(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u16]) {
        for row in 0..h as usize {
            for col in 0..w as usize {
                let Some(&v) = data.get(row * w as usize + col) else { return };
                let (vx, vy) = ((x as usize + col) % VRAM_W, (y as usize + row) % VRAM_H);
                self.words[vy * VRAM_W + vx] = v;
            }
        }
    }
}

/// One vertex as the shader takes it.
#[derive(Clone, Copy, Debug)]
pub struct Vtx {
    pub pos: [f32; 3],
    /// r | g << 8 | b << 16
    pub colour: u32,
    /// u | v << 16 (u may run to 256: a sprite's far edge)
    pub uv: u32,
    /// clut | tpage << 16
    pub mode: u32,
    /// The texels the polygon may read: u from, v from, u to, v to (each a
    /// byte, inclusive).
    pub window: u32,
}

impl Vtx {
    /// A texel coordinate pair as `uv` holds it.
    pub const fn uv_of(u: u32, v: u32) -> u32 {
        u | v << 16
    }

    /// The whole texture page.
    pub const WHOLE: u32 = 0xffff_0000;
    /// A mode bit: a semi-transparent polygon.
    pub const SEMI: u32 = 1 << 31;
    /// A mode bit (the page's unused bit 14): an untextured polygon, its
    /// colour as it is (a POLY_F4 or POLY_G4), not modulating a texel.
    pub const FLAT: u32 = 1 << 30;
    /// A mode bit (the page's unused bit 13): `window` is the GPU's texture
    /// window (GP0 0xE2) as masks and offsets in texels, u mask | v mask
    /// << 8 | u offset << 16 | v offset << 24: a texel coordinate keeps its
    /// bits outside the mask and takes the offset's inside it, so the
    /// texture repeats.
    pub const WRAP: u32 = 1 << 29;

    /// The texels of a sprite whose corners have `u` and `v`: the
    /// PlayStation never reaches its far edges, so neither may sampling.
    pub fn window_of(u: [u8; 4], v: [u8; 4]) -> u32 {
        let span = |c: [u8; 4]| {
            let (lo, hi) = (*c.iter().min().unwrap(), *c.iter().max().unwrap());
            (lo as u32, if hi > lo { hi as u32 - 1 } else { hi as u32 })
        };
        let ((u0, u1), (v0, v1)) = (span(u), span(v));
        u0 | v0 << 8 | u1 << 16 | v1 << 24
    }
}

impl Pack for Vtx {
    /// Each field little-endian, in order, matching the layout the pipeline
    /// declares.
    const SIZE: usize = 28;

    fn pack(&self, out: &mut Vec<u8>) {
        self.pos.pack(out);
        [self.colour, self.uv, self.mode, self.window].pack(out);
    }
}

