//! The CD-ROM drive behind 0x1f801800-0x1f801803, closely enough for libcd:
//! commands with their parameters, responses that arrive as INT3
//! (acknowledge), INT2 (complete), INT1 (a sector ready) or INT5 (error),
//! one at a time and each waiting for the previous to be acknowledged; reads
//! paced at the drive's speed; the data FIFO loaded on request and drained
//! by the CPU or DMA 3. With XA-ADPCM on (mode bit 6), real-time audio
//! sectors go to the sound side and never reach the CPU. Times are in the
//! harness's steps (instructions).

use std::collections::VecDeque;
use std::rc::Rc;

use rrt::disc::Image;

use crate::state::{Reader, Result, Writer};

/// Steps to a command's acknowledge, to a seek or pause's completion, and
/// between sectors at single speed (about 20 M instructions a second).
pub const ACK_STEPS: u64 = 2_000;
pub const DONE_STEPS: u64 = 60_000;
pub const SECTOR_STEPS: u64 = 266_000;

const MOTOR: u8 = 0x02;
const READING: u8 = 0x20;
const SEEKING: u8 = 0x40;

struct Pending {
    due: u64,
    int: u8,
    bytes: Vec<u8>,
    /// For INT1: the sector it announces, raw 2352 bytes.
    sector: Option<Vec<u8>>,
}

pub struct Cdrom {
    pub disc: Option<Rc<Image>>,
    index: u8,
    params: Vec<u8>,
    response: VecDeque<u8>,
    /// The data FIFO: the sector chosen by the mode, and how far read.
    data: Vec<u8>,
    data_pos: usize,
    /// The last sector read, raw, and whether the data FIFO has taken it
    /// yet (asking again does not rewind a sector being read).
    sector: Vec<u8>,
    sector_fresh: bool,
    int_enable: u8,
    int_flag: u8,
    pending: VecDeque<Pending>,
    pub mode: u8,
    filter: (u8, u8),
    /// Setloc's target, and whether a read or seek has used it yet.
    setloc: u32,
    setloc_fresh: bool,
    pos: u32,
    reading: bool,
    next_read: u64,
    motor: bool,
    /// Commands written, by number, for the run's report.
    pub commands: std::collections::BTreeMap<u8, u64>,
    /// Real-time audio sectors passed to the sound side.
    pub xa_sectors: u64,
}

impl Default for Cdrom {
    /// The drive as the BIOS leaves it for the game: all five interrupts
    /// enabled, motor on.
    fn default() -> Self {
        Cdrom {
            disc: None,
            index: 0,
            params: Vec::new(),
            response: VecDeque::new(),
            data: Vec::new(),
            data_pos: 0,
            sector: Vec::new(),
            sector_fresh: false,
            int_enable: 0x1f,
            int_flag: 0,
            pending: VecDeque::new(),
            mode: 0,
            filter: (0, 0),
            setloc: 0,
            setloc_fresh: false,
            pos: 0,
            reading: false,
            next_read: 0,
            motor: true,
            commands: Default::default(),
            xa_sectors: 0,
        }
    }
}

fn bcd(v: u32) -> u8 {
    ((v / 10) << 4 | (v % 10)) as u8
}

fn from_bcd(b: u8) -> u32 {
    (b >> 4) as u32 * 10 + (b & 15) as u32
}

impl Cdrom {
    pub fn save(&self, w: &mut Writer) {
        w.bytes(b"cdrom");
        w.u8(self.index);
        w.bytes(&self.params);
        w.bytes(&self.response.iter().copied().collect::<Vec<_>>());
        w.bytes(&self.data);
        w.u32(self.data_pos as u32);
        w.bytes(&self.sector);
        w.bool(self.sector_fresh);
        w.u8(self.int_enable);
        w.u8(self.int_flag);
        w.u32(self.pending.len() as u32);
        for p in &self.pending {
            w.u64(p.due);
            w.u8(p.int);
            w.bytes(&p.bytes);
            w.bytes(p.sector.as_deref().unwrap_or(&[]));
        }
        w.u8(self.mode);
        w.u8(self.filter.0);
        w.u8(self.filter.1);
        w.u32(self.setloc);
        w.bool(self.setloc_fresh);
        w.u32(self.pos);
        w.bool(self.reading);
        w.u64(self.next_read);
        w.bool(self.motor);
    }

    pub fn load(&mut self, r: &mut Reader) -> Result<()> {
        r.expect(b"cdrom")?;
        self.index = r.u8()?;
        self.params = r.bytes()?;
        self.response = r.bytes()?.into();
        self.data = r.bytes()?;
        self.data_pos = r.u32()? as usize;
        self.sector = r.bytes()?;
        self.sector_fresh = r.bool()?;
        self.int_enable = r.u8()?;
        self.int_flag = r.u8()?;
        let n = r.u32()?;
        self.pending.clear();
        for _ in 0..n {
            let (due, int, bytes, sector) = (r.u64()?, r.u8()?, r.bytes()?, r.bytes()?);
            self.pending.push_back(Pending { due, int, bytes, sector: (!sector.is_empty()).then_some(sector) });
        }
        self.mode = r.u8()?;
        self.filter = (r.u8()?, r.u8()?);
        self.setloc = r.u32()?;
        self.setloc_fresh = r.bool()?;
        self.pos = r.u32()?;
        self.reading = r.bool()?;
        self.next_read = r.u64()?;
        self.motor = r.bool()?;
        Ok(())
    }

    fn stat(&self) -> u8 {
        let mut s = if self.motor { MOTOR } else { 0 };
        if self.reading {
            s |= READING;
        }
        s
    }

    fn sector_steps(&self) -> u64 {
        if self.mode & 0x80 != 0 { SECTOR_STEPS / 2 } else { SECTOR_STEPS }
    }

    fn queue(&mut self, due: u64, int: u8, bytes: Vec<u8>) {
        self.pending.push_back(Pending { due, int, bytes, sector: None });
    }

    fn command(&mut self, now: u64, cmd: u8) {
        *self.commands.entry(cmd).or_default() += 1;
        tracing::debug!("cd {now}: command {cmd:#04x} params {:02x?} (flag {} enable {:#x})", self.params, self.int_flag, self.int_enable);
        let p = std::mem::take(&mut self.params);
        let ack = now + ACK_STEPS;
        let stat = self.stat();
        match cmd {
            0x01 => self.queue(ack, 3, vec![stat]),
            0x02 => {
                if p.len() >= 3 {
                    let lba = (from_bcd(p[0]) * 60 + from_bcd(p[1])) * 75 + from_bcd(p[2]);
                    self.setloc = lba.saturating_sub(150);
                    self.setloc_fresh = true;
                }
                self.queue(ack, 3, vec![stat]);
            }
            // Play (CD audio): there is none on this disc.
            0x03 => {
                self.motor = true;
                self.queue(ack, 3, vec![self.stat()]);
            }
            0x06 | 0x1b => {
                self.motor = true;
                if self.setloc_fresh || !self.reading {
                    self.pos = self.setloc;
                    self.setloc_fresh = false;
                }
                // Responses for sectors of an earlier read are dropped.
                self.pending.retain(|q| q.int != 1);
                self.reading = true;
                self.next_read = ack + DONE_STEPS + self.sector_steps();
                self.queue(ack, 3, vec![self.stat()]);
            }
            0x07 => {
                self.motor = true;
                self.queue(ack, 3, vec![self.stat()]);
                self.queue(ack + DONE_STEPS, 2, vec![self.stat()]);
            }
            0x08 => {
                self.reading = false;
                self.pending.retain(|q| q.int != 1);
                self.queue(ack, 3, vec![stat]);
                self.motor = false;
                self.queue(ack + DONE_STEPS, 2, vec![0]);
            }
            0x09 => {
                self.reading = false;
                self.pending.retain(|q| q.int != 1);
                self.queue(ack, 3, vec![stat]);
                self.queue(ack + DONE_STEPS, 2, vec![self.stat()]);
            }
            0x0a => {
                self.reading = false;
                self.pending.retain(|q| q.int != 1);
                self.mode = 0x20;
                self.motor = true;
                self.queue(ack, 3, vec![self.stat()]);
                self.queue(ack + DONE_STEPS, 2, vec![self.stat()]);
            }
            0x0b | 0x0c => self.queue(ack, 3, vec![stat]),
            0x0d => {
                if p.len() >= 2 {
                    self.filter = (p[0], p[1]);
                }
                self.queue(ack, 3, vec![stat]);
            }
            0x0e => {
                if let Some(&m) = p.first() {
                    self.mode = m;
                }
                self.queue(ack, 3, vec![stat]);
            }
            0x0f => self.queue(ack, 3, vec![stat, self.mode, 0, self.filter.0, self.filter.1]),
            0x10 => {
                let h = self.sector.get(12..20).map(<[u8]>::to_vec).unwrap_or(vec![0; 8]);
                self.queue(ack, 3, h);
            }
            0x11 => {
                let abs = self.pos + 150;
                let rel = self.pos;
                let msf = |l: u32| [bcd(l / 75 / 60), bcd(l / 75 % 60), bcd(l % 75)];
                let (r, a) = (msf(rel), msf(abs));
                self.queue(ack, 3, vec![1, 1, r[0], r[1], r[2], a[0], a[1], a[2]]);
            }
            0x13 => self.queue(ack, 3, vec![stat, 1, 1]),
            0x14 => {
                let l = match p.first().copied().unwrap_or(0) {
                    0 => self.disc.as_ref().map_or(0, |d| d.sectors()) + 150,
                    _ => 150,
                };
                self.queue(ack, 3, vec![stat, bcd(l / 75 / 60), bcd(l / 75 % 60)]);
            }
            0x15 | 0x16 => {
                self.reading = false;
                self.pending.retain(|q| q.int != 1);
                self.pos = self.setloc;
                self.setloc_fresh = false;
                self.motor = true;
                self.queue(ack, 3, vec![self.stat() | SEEKING]);
                self.queue(ack + DONE_STEPS, 2, vec![self.stat()]);
            }
            0x19 => match p.first() {
                Some(0x20) => self.queue(ack, 3, vec![0x94, 0x09, 0x19, 0xc0]),
                _ => self.queue(ack, 5, vec![stat | 1, 0x10]),
            },
            0x1a => {
                self.queue(ack, 3, vec![stat]);
                self.queue(ack + DONE_STEPS, 2, vec![0x02, 0x00, 0x20, 0x00, b'S', b'C', b'E', b'A']);
            }
            0x1e => {
                self.queue(ack, 3, vec![stat]);
                self.queue(ack + DONE_STEPS, 2, vec![stat]);
            }
            _ => {
                tracing::warn!("CD-ROM command {cmd:#04x} not modelled");
                self.queue(ack, 5, vec![stat | 1, 0x40]);
            }
        }
    }

    /// The interrupt line: a flagged response the CPU has enabled.
    pub fn line(&self) -> bool {
        self.int_flag & self.int_enable != 0
    }

    /// Reads due sectors and delivers due responses.
    pub fn tick(&mut self, now: u64) {
        while self.reading && now >= self.next_read {
            let lba = self.pos;
            self.pos += 1;
            self.next_read += self.sector_steps();
            let Some(raw) = self.disc.as_ref().and_then(|d| d.sector(lba).ok()).map(|s| s.raw.to_vec()) else {
                self.reading = false;
                self.queue(now, 5, vec![self.stat() | 1, 0x04]);
                break;
            };
            let (file, chan, submode) = (raw[16], raw[17], raw[18]);
            let audio = submode & 0x04 != 0 && submode & 0x40 != 0;
            if self.mode & 0x40 != 0 && audio {
                if self.mode & 0x08 == 0 || (file, chan) == self.filter {
                    self.xa_sectors += 1;
                }
                continue;
            }
            // A sector not yet taken is overwritten by the next.
            self.pending.retain(|q| q.int != 1);
            let stat = self.stat();
            self.pending.push_back(Pending { due: now, int: 1, bytes: vec![stat], sector: Some(raw) });
        }
        if self.int_flag != 0 {
            return;
        }
        let Some(front) = self.pending.front() else { return };
        if front.due > now {
            return;
        }
        let p = self.pending.pop_front().unwrap();
        if let Some(s) = p.sector {
            self.sector = s;
            self.sector_fresh = true;
        }
        tracing::debug!("cd {now}: INT{} {:02x?} (enable {:#x})", p.int, p.bytes, self.int_enable);
        self.response = p.bytes.into();
        self.int_flag = p.int;
    }

    /// The bytes the data FIFO serves for the last sector, by mode bit 5.
    fn load_data(&mut self) {
        self.data = match (self.mode & 0x20 != 0, self.sector.len() >= 2352) {
            (_, false) => Vec::new(),
            (true, true) => self.sector[12..12 + 0x924].to_vec(),
            (false, true) => self.sector[24..24 + 0x800].to_vec(),
        };
        self.data_pos = 0;
    }

    pub fn read_data(&mut self) -> u8 {
        let b = self.data.get(self.data_pos).copied().unwrap_or(0);
        self.data_pos += 1;
        b
    }

    pub fn read(&mut self, reg: u32) -> u8 {
        match reg {
            0 => {
                let mut s = self.index;
                s |= (self.params.is_empty() as u8) << 3;
                s |= ((self.params.len() < 16) as u8) << 4;
                s |= (!self.response.is_empty() as u8) << 5;
                s |= ((self.data_pos < self.data.len()) as u8) << 6;
                s |= ((self.int_flag == 0 && self.pending.front().is_some_and(|p| p.int == 3)) as u8) << 7;
                s
            }
            1 => self.response.pop_front().unwrap_or(0),
            2 => self.read_data(),
            _ => {
                if self.index & 1 == 0 {
                    self.int_enable | 0xe0
                } else {
                    self.int_flag | 0xe0
                }
            }
        }
    }

    pub fn write(&mut self, now: u64, reg: u32, v: u8) {
        match (reg, self.index) {
            (0, _) => self.index = v & 3,
            (1, 0) => self.command(now, v),
            (2, 0) => {
                if self.params.len() < 16 {
                    self.params.push(v);
                }
            }
            (2, 1) => self.int_enable = v & 0x1f,
            (3, 0) => {
                if v & 0x80 != 0 {
                    if self.sector_fresh {
                        self.load_data();
                        self.sector_fresh = false;
                    }
                } else {
                    self.data.clear();
                    self.data_pos = 0;
                }
            }
            (3, 1) => {
                tracing::debug!("cd {now}: ack {v:#04x}");
                self.int_flag &= !(v & 0x1f);
                if v & 0x40 != 0 {
                    self.params.clear();
                }
            }
            // Volumes and the sound map: nothing to model.
            _ => {}
        }
    }
}
