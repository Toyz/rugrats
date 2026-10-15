//! The controller port (SIO0, 0x1f801040): what libpad's direct driver
//! (SLUS_006.50:0x80099e08 installs it) talks to at each vertical blank. A
//! digital pad on port 1, nothing on port 2 or in the card slots.
//!
//! A byte written to the data port is exchanged at once with the selected
//! device; its answer waits in the receive slot. The pad answers 0x01 with
//! Hi-Z, 0x42 with its ID 0x41, then 0x5a and the two button bytes (low
//! active), and pulls /ACK after every byte but the last, which raises IRQ
//! 7 when the control register enables it.
//!
//! The pulse comes a little after the byte, after the driver has cleared
//! I_STAT bit 7, acknowledged the port and taken the answer
//! (0x80098b34, 0x80098bc4); it then polls I_STAT for it. The harness's
//! clock stands still within a slice, so the pulse lands at that poll (an
//! I_STAT read) or at the slice's end.

use crate::state::{Reader, Result, Writer};

/// IRQ 7: the controller and memory card ports.
pub const IRQ_PAD: u32 = 1 << 7;

#[derive(Default)]
pub struct Sio {
    mode: u16,
    ctrl: u16,
    baud: u16,
    /// The receive slot.
    rx: Option<u8>,
    /// Where the selected pad is in its answer; None once it has stopped
    /// answering until deselected.
    step: Option<usize>,
    /// An /ACK pulse on its way.
    ack_due: bool,
    /// STAT bit 7, the /ACK line low: a short pulse, so seen by one read.
    ack: bool,
    /// STAT bit 9, the interrupt request.
    irq: bool,
    /// Port 1's buttons, 1 for pressed, in the pad's bit order.
    pub buttons: u16,
    /// No pad in port 1 either (RUGRATS_NO_PAD): the boot then goes from
    /// the logos straight to the opening.
    pub unplugged: bool,
}

impl Sio {
    pub fn save(&self, w: &mut Writer) {
        w.u32(self.mode as u32);
        w.u32(self.ctrl as u32);
        w.u32(self.baud as u32);
        w.u32(self.rx.map_or(u32::MAX, |b| b as u32));
        w.u32(self.step.map_or(u32::MAX, |s| s as u32));
        w.u32(self.ack_due as u32);
        w.u32(self.ack as u32 | (self.irq as u32) << 1);
        w.u32(self.buttons as u32);
    }

    pub fn load(&mut self, r: &mut Reader) -> Result<()> {
        self.mode = r.u32()? as u16;
        self.ctrl = r.u32()? as u16;
        self.baud = r.u32()? as u16;
        self.rx = Some(r.u32()?).filter(|&v| v != u32::MAX).map(|v| v as u8);
        self.step = Some(r.u32()?).filter(|&v| v != u32::MAX).map(|v| v as usize);
        self.ack_due = r.u32()? != 0;
        let f = r.u32()?;
        (self.ack, self.irq) = (f & 1 != 0, f & 2 != 0);
        self.buttons = r.u32()? as u16;
        Ok(())
    }

    fn selected(&self) -> bool {
        self.ctrl & 0x0002 != 0
    }

    /// The pad's answer to `byte` at `step`, and whether it pulls /ACK.
    fn pad(&self, step: usize, byte: u8) -> Option<(u8, bool)> {
        let held = !self.buttons;
        match (step, byte) {
            (0, 0x01) => Some((0xff, true)),
            (0, _) => None,
            (1, 0x42) => Some((0x41, true)),
            (1, _) => None,
            (2, _) => Some((0x5a, true)),
            (3, _) => Some((held as u8, true)),
            (4, _) => Some(((held >> 8) as u8, false)),
            _ => None,
        }
    }

    fn exchange(&mut self, byte: u8) {
        let port1 = self.ctrl & 0x2000 == 0;
        let answer = match self.step {
            Some(step) if self.selected() && port1 && !self.unplugged => self.pad(step, byte),
            _ => None,
        };
        match answer {
            Some((reply, ack)) => {
                self.rx = Some(reply);
                self.step = self.step.map(|s| s + 1);
                self.ack_due = ack;
            }
            None => {
                // Nothing there: the line floats high and nobody acks.
                self.rx = Some(0xff);
                self.step = None;
                self.ack_due = false;
            }
        }
    }

    /// Lands an /ACK pulse on its way; true when it raises IRQ 7.
    pub fn land(&mut self) -> bool {
        if !std::mem::take(&mut self.ack_due) {
            return false;
        }
        self.ack = true;
        if self.ctrl & 0x1000 != 0 && !self.irq {
            self.irq = true;
            return true;
        }
        false
    }

    pub fn read(&mut self, phys: u32) -> u32 {
        let v = self.read_reg(phys);
        tracing::trace!("sio read {:x} -> {v:04x}", phys & 0xf);
        v
    }

    fn read_reg(&mut self, phys: u32) -> u32 {
        match phys & 0xf {
            0x0 => self.rx.take().map_or(0xff, |b| b as u32),
            0x4 => {
                // TX ready and finished; RX waiting; /ACK; IRQ.
                let mut s = 0x0005;
                if self.rx.is_some() {
                    s |= 0x0002;
                }
                if std::mem::take(&mut self.ack) {
                    s |= 0x0080;
                }
                if self.irq {
                    s |= 0x0200;
                }
                s
            }
            0x8 => self.mode as u32,
            0xa => self.ctrl as u32,
            0xe => self.baud as u32,
            _ => 0,
        }
    }

    pub fn write(&mut self, phys: u32, value: u32) {
        tracing::trace!("sio write {:x} <- {value:04x}", phys & 0xf);
        match phys & 0xf {
            0x0 => self.exchange(value as u8),
            0x8 => self.mode = value as u16,
            0xa => {
                let was = self.selected();
                self.ctrl = value as u16;
                if value & 0x0040 != 0 {
                    // Reset.
                    *self = Sio { buttons: self.buttons, unplugged: self.unplugged, ..Sio::default() };
                    return;
                }
                if value & 0x0010 != 0 {
                    // Acknowledge: the request clears.
                    self.irq = false;
                }
                if !self.selected() {
                    self.step = None;
                    self.ack_due = false;
                } else if !was {
                    self.step = Some(0);
                }
            }
            0xe => self.baud = value as u16,
            _ => {}
        }
    }
}
