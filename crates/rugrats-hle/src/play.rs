//! The original booted into a level for play, the way the oracle probes do
//! it, and the player read back: what the port's play is proved against.
//!
//! The front end is pressed through as a player would (Start, Cross), with
//! the level choice patched: the instruction at 0x80018824 that loads the
//! level number into a0 becomes `li a0, LEVEL` for the boot and is put back
//! after, and the level global (0x800c5678) is set when the level loader
//! (0x80017c0c) first runs. 0x8004472c is turned into a branch past the
//! front end's attract wait, and cinematics are skipped by returning 0 from
//! `play_cine` (0x80074958) unless asked for. A frame here is a vertical
//! blank; the game steps every second one.

use std::cell::Cell;
use std::rc::Rc;

use rrt::disc::Image;
use rrt::emu::{BusExt, HookResult};

use crate::{Hle, button};

/// The player object (0x800e2b48), and its fields read here.
pub const PLAYER: u32 = 0x800e_2b48;
/// The level number (0x800c5678), the player's character (0x800c5680).
pub const LEVEL: u32 = 0x800c_5678;
pub const CHARACTER: u32 = 0x800c_5680;
/// The vertical blank by which every level is in play after the boot.
pub const IN_PLAY: u64 = 4000;

/// The player as the original holds it (0x800e2b48).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Player {
    /// +0x18: the position.
    pub pos: [i32; 3],
    /// +0x24: the heading (y angle), +0x2c the heading turned toward.
    pub heading: i16,
    pub target: i16,
    /// +0x18c: the state (0 stand, 1 walk, 2 run, 5 jump, 9 bump ...).
    pub state: u8,
    /// +0x184: the speed.
    pub speed: i32,
    /// +0x188: the flags.
    pub flags: u32,
    /// +0x170: the air velocity (body frame).
    pub velocity: [i16; 3],
    /// +0x178: the way the ground step goes.
    pub way: [i32; 3],
}

/// What a boot asks for.
#[derive(Clone, Copy, Debug, Default)]
pub struct Boot {
    pub level: u8,
    /// Play the level's cinematics instead of skipping them.
    pub cinematics: bool,
    /// Run as this character (0x800c5680), as level 4 (6, Spike), 5 (3), 6
    /// and 10 (2, Chuckie), 7 (9), 11 (6), 17 (5) do when entered from the
    /// hub.
    pub character: Option<u32>,
}

/// The original in play.
pub struct Play {
    pub hle: Hle,
}

impl Play {
    /// Boots the disc into `boot.level` and runs to [`IN_PLAY`].
    pub fn boot(disc: Rc<Image>, boot: Boot) -> Result<Play, String> {
        let mut hle = Hle::new(disc, true)?;
        let level = boot.level as u32;
        let set_level = Rc::new(Cell::new(false));
        {
            let done = set_level.clone();
            hle.m.hook(0x8001_7c0c, move |c| {
                if !done.get() {
                    done.set(true);
                    c.bus.load_ram(LEVEL, &level.to_le_bytes());
                }
                HookResult::Continue
            });
        }
        // The character, at each level load (0x800454dc) of the boot: with
        // the cinematics the loader runs before play mode's activation sets
        // the character from the hub's level (0x80018784), and again after.
        let booting = Rc::new(Cell::new(true));
        if let Some(ch) = boot.character {
            let booting = booting.clone();
            hle.m.hook(0x8004_54dc, move |c| {
                if booting.get() {
                    c.bus.load_ram(CHARACTER, &ch.to_le_bytes());
                }
                HookResult::Continue
            });
        }
        if !boot.cinematics {
            hle.m.hook(0x8007_4958, |_| HookResult::ret32(0));
        }
        let presses: [(u64, u16); 6] = [
            (300, button::START),
            (500, button::CROSS),
            (650, button::CROSS),
            (800, button::CROSS),
            (1650, button::CROSS),
            (1800, button::CROSS),
        ];
        let mut play = Play { hle };
        for f in 1..=IN_PLAY {
            if f == 1 {
                play.write_u32(0x8001_8824, 0x2404_0000 + level);
            }
            if f == 2400 {
                play.write_u32(0x8001_8824, 0x00a0_2021);
            }
            play.write_u32(0x8004_472c, 0x1000_004a);
            let pad = presses.iter().filter(|(at, _)| (*at..at + 4).contains(&f)).fold(0, |a, p| a | p.1);
            play.vblank(pad)?;
        }
        booting.set(false);
        Ok(play)
    }

    /// One vertical blank with `pad` held (`button` bits).
    pub fn vblank(&mut self, pad: u16) -> Result<(), String> {
        self.write_u32(0x8004_472c, 0x1000_004a);
        self.hle.pad = pad;
        self.hle.run_frame().map_err(|e| format!("vblank {}: {e:?}", self.hle.frame))
    }

    /// One game frame (two vertical blanks) with `pad` held.
    pub fn frame(&mut self, pad: u16) -> Result<(), String> {
        self.vblank(pad)?;
        self.vblank(pad)
    }

    pub fn read_u32(&mut self, a: u32) -> u32 {
        self.hle.m.bus.read_u32(a).unwrap_or(0)
    }
    pub fn read_i32(&mut self, a: u32) -> i32 {
        self.read_u32(a) as i32
    }
    pub fn read_i16(&mut self, a: u32) -> i16 {
        self.hle.m.bus.read_u16(a).unwrap_or(0) as i16
    }
    pub fn read_u8(&mut self, a: u32) -> u8 {
        self.hle.m.bus.read_u8(a).unwrap_or(0)
    }
    pub fn write_u32(&mut self, a: u32, v: u32) {
        self.hle.m.bus.load_ram(a, &v.to_le_bytes());
    }
    pub fn write_i16(&mut self, a: u32, v: i16) {
        self.hle.m.bus.load_ram(a, &v.to_le_bytes());
    }

    /// The player now.
    pub fn player(&mut self) -> Player {
        let p = PLAYER;
        Player {
            pos: [self.read_i32(p + 0x18), self.read_i32(p + 0x1c), self.read_i32(p + 0x20)],
            heading: self.read_i16(p + 0x24),
            target: self.read_i16(p + 0x2c),
            state: self.read_u8(p + 0x18c),
            speed: self.read_i32(p + 0x184),
            flags: self.read_u32(p + 0x188),
            velocity: [self.read_i16(p + 0x170), self.read_i16(p + 0x172), self.read_i16(p + 0x174)],
            way: [self.read_i32(p + 0x178), self.read_i32(p + 0x17c), self.read_i32(p + 0x180)],
        }
    }

    /// The camera's heading as the stick reads it (0x800a0328 on the
    /// player's camera matrix, +0x224: atan2(-m02, m22)), 4096 a turn.
    pub fn camera_heading(&mut self) -> i32 {
        let m = self.read_u32(PLAYER + 0x224);
        if m == 0 {
            return 0;
        }
        let (m02, m22) = (self.read_i16(m + 4) as f64, self.read_i16(m + 0x10) as f64);
        ((-m02).atan2(m22) * 4096.0 / std::f64::consts::TAU).round() as i32 & 4095
    }

    /// Puts the player at `pos` facing `heading` (the turn's target too).
    pub fn put(&mut self, pos: [i32; 3], heading: i16) {
        let p = PLAYER;
        for (k, v) in pos.iter().enumerate() {
            self.write_u32(p + 0x18 + 4 * k as u32, *v as u32);
        }
        self.write_i16(p + 0x24, heading);
        self.write_i16(p + 0x2c, heading);
    }
}
