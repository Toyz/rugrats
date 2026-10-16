//! The game's view of the pads (SLUS_006.50:0x8001acc0 and friends): per
//! port, a history of each of the 16 buttons, one bit a frame, newest in bit
//! 0, so "pressed this frame" is the low two bits reading 01.

use rrt::input::Pad as HostPad;

/// The pad's button bits in the PlayStation's order (bit 0 Select .. bit 15
/// Square): the order the game's histories are kept in.
pub const BUTTONS: usize = 16;
pub const SELECT: usize = 0;
pub const START: usize = 3;
pub const UP: usize = 4;
pub const RIGHT: usize = 5;
pub const DOWN: usize = 6;
pub const LEFT: usize = 7;
pub const L2: usize = 8;
pub const R2: usize = 9;
pub const L1: usize = 10;
pub const R1: usize = 11;
pub const TRIANGLE: usize = 12;
pub const CIRCLE: usize = 13;
pub const CROSS: usize = 14;
pub const SQUARE: usize = 15;

#[derive(Clone, Debug, Default)]
pub struct Pad {
    pub connected: bool,
    /// Button `b`'s last 32 frames, newest in bit 0.
    pub history: [u32; BUTTONS],
    /// The left stick this frame, x right and y up, 4096 full (a
    /// DualShock's in analog mode; 0x8005be24 takes each axis over the
    /// D-pad's where larger). Zero without one.
    pub stick: [i32; 2],
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
            // (The stick's bytes are 0 left and up.)
            pad.stick = if host.analog { [stick_axis(host.left.x), -stick_axis(host.left.y)] } else { [0; 2] };
        }
    }

    /// The left stick of `port` this frame (x right, y up, 4096 full): for
    /// scripted input, after `push_bits`.
    pub fn push_stick(&mut self, port: usize, stick: [i32; 2]) {
        self.port[port].stick = stick.map(|v| v.clamp(-4096, 4096));
    }

    /// One frame's sample of `port` from button bits (bit `b` button
    /// `b`): for scripted input.
    pub fn push_bits(&mut self, port: usize, bits: u16) {
        let pad = &mut self.port[port];
        pad.connected = true;
        for (b, h) in pad.history.iter_mut().enumerate() {
            *h = *h << 1 | (bits >> b) as u32 & 1;
        }
    }

    /// Whether `button` (0 Select .. 15 Square) of `port` went down this
    /// sample, as a direct test of the history word does (play_cine's skip
    /// reads the first port's Triangle at 0x800cda4c).
    pub fn pressed(&self, port: usize, button: usize) -> bool {
        self.port[port].history[button] & 3 == 1
    }

    /// Whether `button` of `port` came up this sample (history & 3 == 2).
    pub fn released(&self, port: usize, button: usize) -> bool {
        self.port[port].history[button] & 3 == 2
    }

    /// Whether `button` of `port` is down this sample.
    pub fn held(&self, port: usize, button: usize) -> bool {
        self.port[port].history[button] & 1 != 0
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
        self.port.iter().filter_map(|p| p.history.iter().position(|&h| hit(h)).map(|b| b + 1)).next_back()
    }
}

/// An analog stick's byte as an axis, 4096 full (0x8001a888's type 2: 127
/// the middle, 24 either side of it dead, the rest scaled back to the
/// whole 127, then to 4096).
fn stick_axis(b: u8) -> i32 {
    let a = b as i32 - 127;
    let a = if a < -24 {
        a + 24
    } else if a > 24 {
        a - 24
    } else {
        0
    };
    let a = ((a * 127) as i64 * 0x13e2_2cbd / (1i64 << 35)) as i32;
    (a * 4096 / 127).clamp(-4096, 4096)
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

    #[test]
    fn the_stick_has_the_originals_dead_zone_and_reach() {
        assert_eq!(stick_axis(127), 0);
        assert_eq!(stick_axis(127 + 24), 0);
        assert_eq!(stick_axis(127 - 24), 0);
        assert_eq!(stick_axis(255), 4096);
        assert_eq!(stick_axis(0), -4096);
        assert!(stick_axis(127 + 25) > 0 && stick_axis(127 + 25) < 64);
    }
}
