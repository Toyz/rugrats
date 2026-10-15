//! The I/O ports the game's libraries touch: the GPU's two ports, the DMA
//! channels (2 GPU, 4 SPU, 6 ordering-table clear), the SPU's registers and
//! RAM, the root counters and the interrupt registers. A transfer moves its
//! data the moment it starts; its completion (busy bit, DICR flag, IRQ 3)
//! lands a little later, when the harness calls [`Hw::advance`], so
//! libgpu's queue sees the interrupt it expects.

use crate::cdrom::Cdrom;
use crate::gpu::Gpu;
use crate::mdec::Mdec;
use crate::state::{Reader, Result, Writer};

pub const SPU_RAM: usize = 512 * 1024;

/// Steps a DMA transfer reads as busy before it completes.
pub const DMA_TIME: u64 = 2000;

/// Interrupt request bits (I_STAT, I_MASK).
pub const IRQ_VBLANK: u32 = 1 << 0;
pub const IRQ_CDROM: u32 = 1 << 2;
pub const IRQ_DMA: u32 = 1 << 3;

pub struct Hw {
    pub gpu: Gpu,
    pub cd: Cdrom,
    /// The drive's interrupt line as last seen: IRQ 2 is its rising edge.
    cd_line: bool,
    pub mdec: Mdec,
    /// A DMA 1 (MDEC out) started before its words were there: where and
    /// how many still to move.
    mdec_out: Option<(u32, u32)>,
    /// The harness's step clock, set before each slice it runs.
    pub now: u64,
    /// MADR, BCR, CHCR per channel.
    dma: [[u32; 3]; 7],
    /// When each channel's transfer completes, if one is in flight.
    dma_due: [Option<u64>; 7],
    dpcr: u32,
    dicr: u32,
    pub i_stat: u32,
    pub i_mask: u32,
    pub spu_regs: Vec<u16>,
    pub spu_ram: Vec<u8>,
    spu_addr: usize,
    /// I/O addresses touched that nothing here models, with counts.
    pub unhandled: std::collections::BTreeMap<u32, u64>,
}

impl Default for Hw {
    fn default() -> Self {
        Hw {
            gpu: Gpu::default(),
            cd: Cdrom::default(),
            cd_line: false,
            mdec: Mdec::default(),
            mdec_out: None,
            now: 0,
            dma: [[0; 3]; 7],
            dma_due: [None; 7],
            dpcr: 0x0765_4321,
            dicr: 0,
            i_stat: 0,
            i_mask: 0,
            spu_regs: vec![0; 512],
            spu_ram: vec![0; SPU_RAM],
            spu_addr: 0,
            unhandled: Default::default(),
        }
    }
}

fn word(ram: &[u8], at: u32) -> u32 {
    let a = (at & 0x1f_fffc) as usize;
    u32::from_le_bytes(ram[a..a + 4].try_into().unwrap())
}

fn set_word(ram: &mut [u8], at: u32, v: u32) {
    let a = (at & 0x1f_fffc) as usize;
    ram[a..a + 4].copy_from_slice(&v.to_le_bytes());
}

impl Hw {
    pub fn save(&self, w: &mut Writer) {
        w.bytes(b"hw");
        self.gpu.save(w);
        self.cd.save(w);
        self.mdec.save(w);
        let (a, n) = self.mdec_out.unwrap_or((0, 0));
        w.u32(a);
        w.u32(n);
        for ch in &self.dma {
            for &v in ch {
                w.u32(v);
            }
        }
        for d in &self.dma_due {
            w.u64(d.map_or(u64::MAX, |t| t));
        }
        w.u32(self.dpcr);
        w.u32(self.dicr);
        w.u32(self.i_stat);
        w.u32(self.i_mask);
        w.u16s(&self.spu_regs);
        w.bytes(&self.spu_ram);
        w.u32(self.spu_addr as u32);
        w.u64(self.now);
    }

    pub fn load(&mut self, r: &mut Reader) -> Result<()> {
        r.expect(b"hw")?;
        self.gpu.load(r)?;
        self.cd.load(r)?;
        self.mdec.load(r)?;
        let (a, n) = (r.u32()?, r.u32()?);
        self.mdec_out = (n != 0).then_some((a, n));
        for ch in &mut self.dma {
            for v in ch {
                *v = r.u32()?;
            }
        }
        for d in &mut self.dma_due {
            let t = r.u64()?;
            *d = (t != u64::MAX).then_some(t);
        }
        self.dpcr = r.u32()?;
        self.dicr = r.u32()?;
        self.i_stat = r.u32()?;
        self.i_mask = r.u32()?;
        self.spu_regs = r.u16s()?;
        self.spu_ram = r.bytes()?;
        self.spu_addr = r.u32()? as usize;
        self.now = r.u64()?;
        Ok(())
    }

    /// Completes the transfers that are due by `now`: busy bit off, the
    /// channel's DICR flag, and IRQ 3 when DICR lets it through.
    pub fn advance(&mut self, now: u64) {
        self.now = now;
        self.cd.tick(now);
        self.cd_edge();
        for ch in 0..7 {
            if self.dma_due[ch].is_some_and(|t| t <= now) {
                self.dma_due[ch] = None;
                self.dma[ch][2] &= !(1 << 24 | 1 << 28);
                if self.dicr & (1 << (16 + ch)) != 0 {
                    self.dicr |= 1 << (24 + ch);
                }
            }
        }
        self.update_dicr();
    }

    fn cd_edge(&mut self) {
        let line = self.cd.line();
        if line && !self.cd_line {
            self.i_stat |= IRQ_CDROM;
        }
        self.cd_line = line;
    }

    /// DICR bit 31 follows the enabled flags; it raising sets IRQ 3.
    fn update_dicr(&mut self) {
        let flags = (self.dicr >> 24) & 0x7f;
        let on = self.dicr & (1 << 15) != 0 || (self.dicr & (1 << 23) != 0 && flags & (self.dicr >> 16) & 0x7f != 0);
        if on && self.dicr & (1 << 31) == 0 {
            self.i_stat |= IRQ_DMA;
        }
        self.dicr = if on { self.dicr | 1 << 31 } else { self.dicr & !(1 << 31) };
    }

    /// The earliest pending transfer completion, if any.
    pub fn next_dma_due(&self) -> Option<u64> {
        self.dma_due.iter().flatten().copied().min()
    }

    fn start_dma(&mut self, ram: &mut [u8], ch: usize) {
        let [madr, bcr, chcr] = self.dma[ch];
        let words = match (chcr >> 9) & 3 {
            0 => {
                let n = bcr & 0xffff;
                if n == 0 { 0x10000 } else { n }
            }
            _ => (bcr & 0xffff) * (bcr >> 16).max(1),
        };
        let from_ram = chcr & 1 != 0;
        match ch {
            2 => match (chcr >> 9) & 3 {
                2 => {
                    // Linked list: each node is a header (count << 24 | next)
                    // and that many GP0 words; bit 23 of next ends the list.
                    let mut at = madr & 0x1f_fffc;
                    let mut seen = std::collections::HashSet::new();
                    loop {
                        if !seen.insert(at) {
                            tracing::warn!("GPU DMA list loops at {at:06x} (started at {madr:08x})");
                            break;
                        }
                        let h = word(ram, at);
                        for k in 0..h >> 24 {
                            self.gpu.gp0(word(ram, at + 4 + 4 * k));
                        }
                        if h & 0x80_0000 != 0 {
                            break;
                        }
                        at = h & 0x1f_fffc;
                    }
                }
                _ if from_ram => {
                    for k in 0..words {
                        self.gpu.gp0(word(ram, madr + 4 * k));
                    }
                }
                _ => {
                    for k in 0..words {
                        let v = self.gpu.read();
                        set_word(ram, madr + 4 * k, v);
                    }
                }
            },
            4 if from_ram => {
                for k in 0..words {
                    for b in word(ram, madr + 4 * k).to_le_bytes() {
                        self.spu_ram[self.spu_addr % SPU_RAM] = b;
                        self.spu_addr += 1;
                    }
                }
            }
            4 => {
                for k in 0..words {
                    let mut v = [0u8; 4];
                    for b in &mut v {
                        *b = self.spu_ram[self.spu_addr % SPU_RAM];
                        self.spu_addr += 1;
                    }
                    set_word(ram, madr + 4 * k, u32::from_le_bytes(v));
                }
            }
            0 => {
                for k in 0..words {
                    self.mdec.write(word(ram, madr + 4 * k));
                }
                self.drain_mdec(ram);
            }
            3 => {
                for k in 0..words {
                    let v = u32::from_le_bytes([0; 4].map(|_| self.cd.read_data()));
                    set_word(ram, madr + 4 * k, v);
                }
            }
            1 => {
                self.mdec_out = Some((madr, words));
                self.drain_mdec(ram);
                if self.mdec_out.is_some() {
                    // Completes when the decoder has the words.
                    return;
                }
            }
            6 => {
                // The ordering table, cleared backwards: each entry points at
                // the one before it, the last written ends the list.
                let n = if bcr & 0xffff == 0 { 0x10000 } else { bcr & 0xffff };
                let mut at = madr & 0x1f_fffc;
                for k in 0..n {
                    let v = if k == n - 1 { 0x00ff_ffff } else { at.wrapping_sub(4) & 0x1f_ffff };
                    set_word(ram, at, v);
                    at = at.wrapping_sub(4);
                }
            }
            _ => {
                *self.unhandled.entry(0x1f80_1080 + ch as u32 * 16 + 8).or_default() += 1;
            }
        }
        self.dma_due[ch] = Some(self.now + DMA_TIME);
    }

    /// Moves decoded words to a waiting DMA 1, completing it when all came.
    fn drain_mdec(&mut self, ram: &mut [u8]) {
        let Some((at, left)) = self.mdec_out else { return };
        let n = left.min(self.mdec.output.len() as u32);
        for k in 0..n {
            let v = self.mdec.read();
            set_word(ram, at + 4 * k, v);
        }
        if n == left {
            self.mdec_out = None;
            self.dma_due[1] = Some(self.now + DMA_TIME);
        } else {
            self.mdec_out = Some((at + 4 * n, left - n));
        }
    }

    pub fn read(&mut self, phys: u32, width: usize) -> u32 {
        // The interrupt and DMA registers are words; a narrower access
        // sees its part of one.
        if (0x1f80_1070..0x1f80_1100).contains(&phys) && phys & 3 != 0 {
            return self.read(phys & !3, 4) >> (8 * (phys & 3));
        }
        match phys {
            0x1f80_1070 => self.i_stat,
            0x1f80_1074 => self.i_mask,
            0x1f80_1080..0x1f80_10f0 => {
                let ch = ((phys - 0x1f80_1080) / 16) as usize;
                let reg = ((phys & 0xf) / 4) as usize % 3;
                self.dma[ch][reg]
            }
            0x1f80_10f0 => self.dpcr,
            0x1f80_10f4 => self.dicr,
            0x1f80_1100..0x1f80_1130 => {
                // Root counters: steady within a slice, moving between them.
                ((self.now / 8) & 0xffff) as u32
            }
            0x1f80_1800..0x1f80_1804 => {
                // The data port may be read 16 or 32 bits at a time.
                let mut v = 0u32;
                for k in 0..width.min(4) {
                    let reg = if phys & 3 == 2 { 2 } else { (phys & 3) + k as u32 };
                    v |= (self.cd.read(reg & 3) as u32) << (8 * k);
                }
                v
            }
            0x1f80_1810 => self.gpu.read(),
            0x1f80_1814 => self.gpu.status(),
            0x1f80_1820 => self.mdec.read(),
            0x1f80_1824 => self.mdec.status(),
            0x1f80_1c00..0x1f80_2000 => {
                let r = ((phys - 0x1f80_1c00) / 2) as usize;
                let v = if phys == 0x1f80_1dae {
                    // SPUSTAT mirrors the mode bits of SPUCNT once applied.
                    self.spu_regs[(0x1daa - 0x1c00) / 2] & 0x3f
                } else {
                    self.spu_regs[r]
                };
                if width == 4 { v as u32 | (self.spu_regs[(r + 1) % 512] as u32) << 16 } else { v as u32 }
            }
            _ => {
                *self.unhandled.entry(phys).or_default() += 1;
                0
            }
        }
    }

    pub fn write(&mut self, ram: &mut [u8], phys: u32, width: usize, value: u32) {
        if (0x1f80_1070..0x1f80_1100).contains(&phys) && (phys & 3 != 0 || width < 4) {
            let (at, shift) = (phys & !3, 8 * (phys & 3));
            let mask = if width >= 4 { u32::MAX } else { ((1u32 << (8 * width)) - 1) << shift };
            // A partial write to I_STAT or DICR's flags must not
            // acknowledge the bytes it does not cover.
            let keep = match at {
                0x1f80_1070 => u32::MAX,
                0x1f80_10f4 => !0x7f00_0000 & self.dicr,
                _ => self.read(at, 4),
            };
            return self.write(ram, at, 4, (keep & !mask) | ((value << shift) & mask));
        }
        match phys {
            // Writing 0 to a bit of I_STAT acknowledges it.
            0x1f80_1070 => self.i_stat &= value,
            0x1f80_1074 => self.i_mask = value,
            0x1f80_1080..0x1f80_10f0 => {
                let ch = ((phys - 0x1f80_1080) / 16) as usize;
                let reg = ((phys & 0xf) / 4) as usize % 3;
                self.dma[ch][reg] = value;
                if reg == 2 && value & (1 << 24) != 0 {
                    self.start_dma(ram, ch);
                }
            }
            0x1f80_10f0 => self.dpcr = value,
            0x1f80_10f4 => {
                // Writing a 1 to a flag bit acknowledges it.
                let flags = self.dicr & !value & 0x7f00_0000;
                self.dicr = (value & 0x00ff_ffff) | flags;
                self.update_dicr();
            }
            0x1f80_1100..0x1f80_1130 => {}
            // Memory control (bus delays) and RAM size: nothing to model.
            0x1f80_1000..0x1f80_1024 | 0x1f80_1060 => {}
            0x1f80_1800..0x1f80_1804 => {
                for k in 0..width.min(4) {
                    let reg = (phys & 3) + k as u32;
                    if reg < 4 {
                        let now = self.now;
                        self.cd.write(now, reg, (value >> (8 * k)) as u8);
                    }
                }
                self.cd.tick(self.now);
                self.cd_edge();
            }
            0x1f80_1810 => self.gpu.gp0(value),
            0x1f80_1814 => self.gpu.gp1(value),
            0x1f80_1820 => {
                self.mdec.write(value);
                self.drain_mdec(ram);
            }
            0x1f80_1824 => self.mdec.control(value),
            0x1f80_1c00..0x1f80_2000 => {
                let r = ((phys - 0x1f80_1c00) / 2) as usize;
                self.spu_regs[r] = value as u16;
                if width == 4 {
                    self.spu_regs[(r + 1) % 512] = (value >> 16) as u16;
                }
                match phys {
                    // The transfer address, in units of 8 bytes.
                    0x1f80_1da6 => self.spu_addr = (value as u16 as usize) * 8,
                    // A halfword through the data port.
                    0x1f80_1da8 => {
                        let a = self.spu_addr % SPU_RAM;
                        self.spu_ram[a] = value as u8;
                        self.spu_ram[(a + 1) % SPU_RAM] = (value >> 8) as u8;
                        self.spu_addr += 2;
                    }
                    _ => {}
                }
            }
            _ => {
                *self.unhandled.entry(phys).or_default() += 1;
            }
        }
    }
}
