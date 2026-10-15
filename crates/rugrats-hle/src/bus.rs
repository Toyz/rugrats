//! The PlayStation's address space as the game sees it: 2 MB of RAM mirrored
//! through the first 8 MB, the 1 KB scratchpad, the I/O ports of [`Hw`],
//! and the BIOS ROM and expansion areas reading as zero (nothing is there:
//! the BIOS is answered on the host).

use rrt::emu::Bus;

use crate::hw::Hw;

pub const RAM_SIZE: usize = 2 << 20;

pub struct PsxBus {
    pub ram: Vec<u8>,
    pub scratch: Vec<u8>,
    pub hw: Hw,
}

impl Default for PsxBus {
    fn default() -> Self {
        PsxBus { ram: vec![0; RAM_SIZE], scratch: vec![0; 1024], hw: Hw::default() }
    }
}

enum Place {
    Ram(usize),
    Scratch(usize),
    Io(u32),
    Zero,
}

fn place(addr: u32) -> Option<Place> {
    if addr >= 0xfffe_0000 {
        return Some(Place::Zero);
    }
    let p = addr & 0x1fff_ffff;
    Some(match p {
        0..0x80_0000 => Place::Ram((p & 0x1f_ffff) as usize),
        0x1f80_0000..0x1f80_0400 => Place::Scratch((p & 0x3ff) as usize),
        0x1f80_1000..0x1f80_3000 => Place::Io(p),
        0x1f00_0000..0x1f80_0000 | 0x1fc0_0000..0x1fc8_0000 => Place::Zero,
        _ => return None,
    })
}

impl PsxBus {
    /// A word of RAM by any alias.
    pub fn ram_u32(&self, addr: u32) -> u32 {
        let a = (addr & 0x1f_fffc) as usize;
        u32::from_le_bytes(self.ram[a..a + 4].try_into().unwrap())
    }

    pub fn set_ram_u32(&mut self, addr: u32, v: u32) {
        let a = (addr & 0x1f_fffc) as usize;
        self.ram[a..a + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// Bytes of RAM from `addr`, wrapping at the mirror.
    pub fn ram_bytes(&self, addr: u32, len: usize) -> Vec<u8> {
        (0..len).map(|k| self.ram[(addr as usize + k) & 0x1f_ffff]).collect()
    }

    pub fn load_ram(&mut self, addr: u32, data: &[u8]) {
        for (k, &b) in data.iter().enumerate() {
            self.ram[(addr as usize + k) & 0x1f_ffff] = b;
        }
    }
}

impl Bus for PsxBus {
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), u32> {
        match place(addr).ok_or(addr)? {
            Place::Ram(_) => {
                for (k, b) in buf.iter_mut().enumerate() {
                    *b = self.ram[(addr as usize + k) & 0x1f_ffff];
                }
            }
            Place::Scratch(a) => {
                for (k, b) in buf.iter_mut().enumerate() {
                    *b = self.scratch[(a + k) & 0x3ff];
                }
            }
            Place::Io(p) => {
                for (k, chunk) in buf.chunks_mut(4).enumerate() {
                    let v = self.hw.read(p + 4 * k as u32, chunk.len());
                    chunk.copy_from_slice(&v.to_le_bytes()[..chunk.len()]);
                }
            }
            Place::Zero => buf.fill(0),
        }
        Ok(())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), u32> {
        match place(addr).ok_or(addr)? {
            Place::Ram(_) => self.load_ram(addr, data),
            Place::Scratch(a) => {
                for (k, &b) in data.iter().enumerate() {
                    self.scratch[(a + k) & 0x3ff] = b;
                }
            }
            Place::Io(p) => {
                for (k, chunk) in data.chunks(4).enumerate() {
                    let mut v = [0u8; 4];
                    v[..chunk.len()].copy_from_slice(chunk);
                    self.hw.write(&mut self.ram, p + 4 * k as u32, chunk.len(), u32::from_le_bytes(v));
                }
            }
            Place::Zero => {}
        }
        Ok(())
    }

    fn canonical(&self, addr: u32) -> u32 {
        match place(addr) {
            Some(Place::Ram(a)) => a as u32,
            _ => addr & 0x1fff_ffff,
        }
    }
}
