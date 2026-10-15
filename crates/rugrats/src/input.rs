//! The game's view of the pads (SLUS_006.50:0x8001acc0 and friends): per
//! port, a history of each of the 16 buttons, one bit a frame, newest in bit
//! 0, so "pressed this frame" is the low two bits reading 01.

use rrt::input::Pad as HostPad;

/// The pad's button bits in the PlayStation's order (bit 0 Select .. bit 15
/// Square): the order the game's histories are kept in.
pub const BUTTONS: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct Pad {
    pub connected: bool,
    /// Button `b`'s last 32 frames, newest in bit 0.
    pub history: [u32; BUTTONS],
}

/// What [`Pads::find`] looks for (the query modes of 0x8001baf8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Query {
    /// Down this frame, up the one before.
    Pressed,
    /// Up this frame, down the one before.
    Released,
    Held,
    NotHeld,
}

#[derive(Clone, Debug, Default)]
pub struct Pads {
    pub port: [Pad; 2],
}

impl Pads {
    /// One frame's sample. A port with nothing in it keeps its histories.
    pub fn update(&mut self, pads: [Option<&HostPad>; 2]) {
        for (pad, host) in self.port.iter_mut().zip(pads) {
            pad.connected = host.is_some();
            let Some(host) = host else { continue };
            let bits = host.buttons.bits();
            for (b, h) in pad.history.iter_mut().enumerate() {
                *h = *h << 1 | (bits >> b) as u32 & 1;
            }
        }
    }

    /// The first button (1-based, as the game counts) of a port matching
    /// the query; the second port's answer wins over the first's.
    pub fn find(&self, q: Query) -> Option<usize> {
        let hit = |h: u32| match q {
            Query::Pressed => h & 3 == 1,
            Query::Released => h & 3 == 2,
            Query::Held => h & 1 != 0,
            Query::NotHeld => h & 1 == 0,
        };
        self.port.iter().filter_map(|p| p.history.iter().position(|&h| hit(h)).map(|b| b + 1)).last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_is_seen_for_one_frame() {
        let mut pads = Pads::default();
        let mut cross = HostPad::default();
        cross.buttons = rrt::input::Buttons::CROSS;
        pads.update([Some(&HostPad::default()), None]);
        assert_eq!(pads.find(Query::Pressed), None);
        pads.update([Some(&cross), None]);
        assert_eq!(pads.find(Query::Pressed), Some(15));
        pads.update([Some(&cross), None]);
        assert_eq!(pads.find(Query::Pressed), None);
        assert_eq!(pads.find(Query::Held), Some(15));
        pads.update([Some(&HostPad::default()), None]);
        assert_eq!(pads.find(Query::Released), Some(15));
    }
}
