//! The game's rand (SLUS_006.50:0x8002fb7c): x = x * 214013 + 2531011,
//! answering bits 16-23. Its state lives at 0x800c3f94, seeded 0x2dcf1a5 in
//! the executable; everything that draws on it shares it.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rand {
    pub state: u32,
}

impl Default for Rand {
    fn default() -> Rand {
        Rand { state: 0x02dc_f1a5 }
    }
}

impl Rand {
    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.state >> 16) & 255
    }
}
