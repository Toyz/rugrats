//! The MDEC as libpress drives it: commands and their parameters through
//! 0x1f801820 or DMA channel 0, pixels back through DMA channel 1. A decode
//! command runs the moment its last parameter arrives
//! (`rugrats_data::mdec::decode`); the words wait in `output` until read.

use std::collections::VecDeque;

use rugrats_data::mdec::{self, Depth, Tables};

use crate::state::{Reader, Result, Writer};

#[derive(Default)]
pub struct Mdec {
    tables: Tables,
    /// The command being fed and the parameter words it still wants.
    cmd: u32,
    want: usize,
    params: Vec<u32>,
    pub output: VecDeque<u32>,
}

impl Mdec {
    pub fn save(&self, w: &mut Writer) {
        w.bytes(b"mdec");
        w.bytes(&self.tables.luma);
        w.bytes(&self.tables.chroma);
        w.u16s(&self.tables.scale.map(|v| v as u16));
        w.u32(self.cmd);
        w.u32(self.want as u32);
        w.u32s(&self.params);
        w.u32s(&self.output.iter().copied().collect::<Vec<_>>());
    }

    pub fn load(&mut self, r: &mut Reader) -> Result<()> {
        r.expect(b"mdec")?;
        self.tables.luma.copy_from_slice(&r.bytes()?);
        self.tables.chroma.copy_from_slice(&r.bytes()?);
        let s = r.u16s()?;
        for (d, v) in self.tables.scale.iter_mut().zip(s) {
            *d = v as i16;
        }
        self.cmd = r.u32()?;
        self.want = r.u32()? as usize;
        self.params = r.u32s()?;
        self.output = r.u32s()?.into();
        Ok(())
    }

    /// A word to 0x1f801820: a new command, or the next parameter.
    pub fn write(&mut self, w: u32) {
        if self.want == 0 {
            self.cmd = w;
            self.params.clear();
            self.want = match w >> 29 {
                1 => (w & 0xffff) as usize,
                2 => {
                    if w & 1 != 0 {
                        32
                    } else {
                        16
                    }
                }
                3 => 32,
                _ => 0,
            };
            return;
        }
        self.params.push(w);
        self.want -= 1;
        if self.want == 0 {
            self.execute();
        }
    }

    fn execute(&mut self) {
        let bytes: Vec<u8> = self.params.iter().flat_map(|w| w.to_le_bytes()).collect();
        match self.cmd >> 29 {
            1 => {
                // RUGRATS_MDEC_DUMP=DIR keeps every decode command's input,
                // for checking a port's VLC decoder against libpress.
                if let Ok(dir) = std::env::var("RUGRATS_MDEC_DUMP") {
                    let n = std::fs::read_dir(&dir).map_or(0, |d| d.count());
                    let mut out = self.cmd.to_le_bytes().to_vec();
                    out.extend(&bytes);
                    let _ = std::fs::write(format!("{dir}/{n:05}.mdec"), out);
                }
                let halves: Vec<u16> = bytes.chunks(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
                let depth = Depth::from_command(self.cmd);
                let signed = self.cmd & (1 << 26) != 0;
                let bit15 = self.cmd & (1 << 25) != 0;
                self.output.extend(mdec::decode(&halves, &self.tables, depth, signed, bit15));
            }
            2 => {
                self.tables.luma.copy_from_slice(&bytes[..64]);
                if self.cmd & 1 != 0 {
                    self.tables.chroma.copy_from_slice(&bytes[64..128]);
                }
            }
            3 => {
                for (k, h) in bytes.chunks(2).take(64).enumerate() {
                    self.tables.scale[k] = i16::from_le_bytes([h[0], h[1]]);
                }
            }
            _ => {}
        }
        self.params.clear();
    }

    /// 0x1f801824 written: bit 31 resets.
    pub fn control(&mut self, v: u32) {
        if v & (1 << 31) != 0 {
            self.want = 0;
            self.params.clear();
            self.output.clear();
        }
    }

    /// 0x1f801824 read: out FIFO empty (31), busy (29), data-in and
    /// data-out requests (28, 27), and the parameter words left - 1.
    pub fn status(&self) -> u32 {
        let mut s = 0u32;
        if self.output.is_empty() {
            s |= 1 << 31;
        } else {
            s |= 1 << 27;
        }
        if self.want > 0 {
            s |= 1 << 29 | 1 << 28;
        }
        s | (self.want.wrapping_sub(1) as u32 & 0xffff)
    }

    pub fn read(&mut self) -> u32 {
        self.output.pop_front().unwrap_or(0)
    }
}
