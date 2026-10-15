//! The original game, run in rrt-emu's R3000A for reference.
//!
//! The executable runs as it is on a model of the console's I/O ([`hw`]):
//! GPU, DMA, SPU registers, interrupt registers. What the BIOS kernel would
//! do is answered on the host by rrt-emu's BIOS table, and a few calls are
//! replaced at their entry points:
//!
//! ```text
//! 0x8009af48  VSync(mode)                  one wait = one frame; the run stops there
//! 0x80019794  read_file(index, dest, len)  the game's loader, served from the disc
//!                                          (unless every read goes through the drive)
//! 0x800b1478  printf                       logged
//! ```
//!
//! Interrupts are delivered by calling libetc's dispatcher (0x8009b3d4) as
//! the BIOS would after an exception: vertical blank once a frame, DMA when
//! a transfer completes. Nothing here is part of the port.

#![forbid(unsafe_code)]

pub mod bus;
pub mod cdrom;
pub mod gpu;
pub mod hw;
pub mod mdec;
pub mod printf;
pub mod state;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rrt::disc::Image;
use rrt::emu::psx::{R3000, bios};
use rrt::emu::{BusExt, Cpu, HookResult, Machine, PsxExe, Stop};

use bus::PsxBus;

/// Addresses in SLUS_006.50.
pub mod addr {
    pub const VSYNC: u32 = 0x8009_af48;
    pub const READ_FILE: u32 = 0x8001_9794;
    pub const PRINTF: u32 = 0x800b_1478;
    /// libetc's interrupt dispatcher, entered through HookEntryInt's jump
    /// buffer when an interrupt arrives.
    pub const INTR_DISPATCH: u32 = 0x8009_b3d4;
    /// libetc: nonzero once ResetCallback has installed the dispatcher.
    pub const INTR_READY: u32 = 0x800b_ba98;
    /// The setjmp buffer ResetCallback gives HookEntryInt: ra, sp, fp,
    /// s0-s7, gp. The BIOS longjmps there on an interrupt, so the handler
    /// runs on libetc's own stack (sp is set to 0x800bcab0) and the C gp.
    pub const INTR_JMPBUF: u32 = 0x800b_bad0;
    /// libetc's vertical blank counter.
    pub const VCOUNT: u32 = 0x800b_cb60;
}

/// Where the main run "returns" to: never reached.
const MAIN_RETURN: u32 = 0xffff_fff0;
/// Where a nested call (an interrupt) returns to.
const NESTED_RETURN: u32 = 0xffff_ff00;
/// Instructions per run slice; devices and interrupts advance between.
const SLICE: u64 = 20_000;
/// A frame without a VSync call this long gets a vertical blank anyway.
const FRAME_STEPS: u64 = 600_000;

/// What the hooks tell the run loop.
#[derive(Default)]
struct Shared {
    /// Frames a VSync call asked to wait.
    vsync: Option<u32>,
    /// The BIOS pad driver's two receive buffers (InitPAD), and whether it
    /// is started.
    pad_buffers: Option<(u32, u32)>,
    pad_started: bool,
}

/// PlayStation pad bits, as the pad sends them (1 = pressed here; the
/// buffer holds them inverted).
pub mod button {
    pub const SELECT: u16 = 1 << 0;
    pub const START: u16 = 1 << 3;
    pub const UP: u16 = 1 << 4;
    pub const RIGHT: u16 = 1 << 5;
    pub const DOWN: u16 = 1 << 6;
    pub const LEFT: u16 = 1 << 7;
    pub const L2: u16 = 1 << 8;
    pub const R2: u16 = 1 << 9;
    pub const L1: u16 = 1 << 10;
    pub const R1: u16 = 1 << 11;
    pub const TRIANGLE: u16 = 1 << 12;
    pub const CIRCLE: u16 = 1 << 13;
    pub const CROSS: u16 = 1 << 14;
    pub const SQUARE: u16 = 1 << 15;

    pub fn by_name(name: &str) -> Option<u16> {
        Some(match name.to_ascii_lowercase().as_str() {
            "select" => SELECT,
            "start" => START,
            "up" => UP,
            "right" => RIGHT,
            "down" => DOWN,
            "left" => LEFT,
            "l2" => L2,
            "r2" => R2,
            "l1" => L1,
            "r1" => R1,
            "triangle" => TRIANGLE,
            "circle" => CIRCLE,
            "cross" => CROSS,
            "square" => SQUARE,
            _ => return None,
        })
    }
}

pub struct Hle {
    pub m: Machine<R3000, PsxBus>,
    /// Port 1's buttons (`button` bits, 1 = held), sent at each vertical blank.
    pub pad: u16,
    pub frame: u64,
    /// Instructions run in total, nested calls included.
    pub steps: u64,
    last_vblank: u64,
    shared: Rc<RefCell<Shared>>,
    irq_enabled: Rc<Cell<bool>>,
    /// Files the game read, in order: (index, bytes).
    pub reads: Rc<RefCell<Vec<(u32, u32)>>>,
    /// Interrupt dispatches made, and those held back (masked, critical
    /// section, dispatcher not installed).
    pub dispatched: u64,
    pub held: [u64; 3],
}

fn stop_text(pc: u32, what: impl Into<String>) -> Stop {
    Stop::Hle { pc, what: what.into() }
}

impl Hle {
    /// Loads SLUS_006.50 from the disc and installs the hardware and hooks.
    /// `fast_load` serves the game's `read_file` straight from the disc;
    /// without it every load goes through libcd and the drive model.
    pub fn new(disc: Rc<Image>, fast_load: bool) -> Result<Hle, String> {
        let iso = disc.iso().map_err(|e| e.to_string())?;
        let bytes = iso.find("/SLUS_006.50").and_then(|e| iso.read(&e)).map_err(|e| e.to_string())?;
        let exe = PsxExe::parse(&bytes).map_err(|e| e.to_string())?;
        let table = rugrats_data::files::table(&exe.image, exe.t_addr).ok_or("no file table in the executable")?;

        let shared: Rc<RefCell<Shared>> = Default::default();
        let mut m = Machine::new(R3000::new(), PsxBus::default());
        let b = bios::install(&mut m);
        // A0:71 _96_init (the BIOS's CD reset): nothing to reset here.
        b.insert(0xa0, 0x71, |_| HookResult::Return(0));
        // A0:AB _card_info(port), A0:AC _card_load(port): started; the
        // events they end with all read as delivered. A0:AD
        // set_card_auto_format(flag).
        b.insert(0xa0, 0xab, |_| HookResult::Return(1));
        b.insert(0xa0, 0xac, |_| HookResult::Return(1));
        b.insert(0xa0, 0xad, |_| HookResult::Return(0));
        // B0:5C _card_status(slot), B0:5D _card_wait(slot): ready.
        b.insert(0xb0, 0x5c, |_| HookResult::Return(1));
        b.insert(0xb0, 0x5d, |_| HookResult::Return(1));
        // B0:12 InitPAD(buf1, size1, buf2, size2), B0:13 StartPAD, B0:14
        // StopPAD: the BIOS's pad driver fills the buffers at each vertical
        // blank; `Hle::vblank` does.
        {
            let sh = shared.clone();
            b.insert(0xb0, 0x12, move |c| {
                sh.borrow_mut().pad_buffers = Some((c.arg32(0), c.arg32(2)));
                HookResult::Return(1)
            });
            let sh = shared.clone();
            b.insert(0xb0, 0x13, move |_| {
                sh.borrow_mut().pad_started = true;
                HookResult::Return(1)
            });
            let sh = shared.clone();
            b.insert(0xb0, 0x14, move |_| {
                sh.borrow_mut().pad_started = false;
                HookResult::Return(1)
            });
        }
        m.ext.insert(b);
        let irq_enabled = bios::critical_sections(&mut m);
        exe.load(&mut m.bus)?;
        m.bus.hw.cd.disc = Some(disc.clone());
        m.cpu.set_pc(exe.pc0);
        m.cpu.r[29] = 0x801f_fff0;
        m.return_to = MAIN_RETURN;

        let reads: Rc<RefCell<Vec<(u32, u32)>>> = Default::default();

        // VSync(mode): mode < 0 the counter, 1 the time since the last, else
        // wait that many vertical blanks: the frame ends here.
        {
            let shared = shared.clone();
            m.hook(addr::VSYNC, move |c| {
                let mode = c.arg32(0) as i32;
                let ra = c.cpu.r[31];
                let v = match mode {
                    m if m < 0 => c.bus.ram_u32(addr::VCOUNT),
                    1 => 0,
                    m => {
                        shared.borrow_mut().vsync = Some(m.max(1) as u32);
                        0
                    }
                };
                c.cpu.r[2] = v;
                c.cpu.set_pc(ra);
                match shared.borrow().vsync {
                    Some(_) => HookResult::Stop(Stop::Halt { pc: ra }),
                    None => HookResult::Return(u64::from(v)),
                }
            });
        }

        // read_file(index, dest, len): `len` bytes of the file, 0 for an
        // unused entry. The original seeks and reads with libcd.
        if fast_load {
            let reads = reads.clone();
            let disc = disc.clone();
            m.hook(addr::READ_FILE, move |c| {
                let (index, dest, len) = (c.arg32(0), c.arg32(1), c.arg32(2));
                let Some(e) = table.get(index as usize) else {
                    return HookResult::Stop(stop_text(addr::READ_FILE, format!("read_file({index}): no such entry")));
                };
                if !e.is_used() {
                    return HookResult::Return(0);
                }
                let data = match disc.read_blocks(e.lba, len) {
                    Ok(d) => d,
                    Err(err) => return HookResult::Stop(stop_text(addr::READ_FILE, err.to_string())),
                };
                c.bus.load_ram(dest, &data);
                reads.borrow_mut().push((index, len));
                tracing::debug!("read_file({index}) -> {dest:08x}, {len} bytes");
                HookResult::Return(u64::from(len))
            });
        }

        m.hook(addr::PRINTF, |c| {
            let text = printf::format(c);
            tracing::info!(target: "game", "{}", text.trim_end());
            HookResult::Return(text.len() as u64)
        });

        Ok(Hle { m, pad: 0, frame: 0, steps: 0, last_vblank: 0, shared, irq_enabled, reads, dispatched: 0, held: [0; 3] })
    }

    /// Runs to the end of the next frame: the next VSync wait, or a frame's
    /// worth of instructions without one.
    pub fn run_frame(&mut self) -> Result<(), Stop> {
        loop {
            self.m.bus.hw.advance(self.steps);
            self.deliver()?;
            let before = self.m.steps;
            self.m.step_limit = before + SLICE;
            let r = self.m.run();
            self.steps += self.m.steps - before;
            match r {
                Err(Stop::StepLimit { .. }) => {
                    if self.steps - self.last_vblank >= FRAME_STEPS {
                        self.vblank()?;
                        return Ok(());
                    }
                }
                Err(Stop::Halt { .. }) if self.shared.borrow().vsync.is_some() => {
                    let n = self.shared.borrow_mut().vsync.take().unwrap_or(1);
                    for _ in 0..n {
                        self.vblank()?;
                    }
                    return Ok(());
                }
                Err(e) => return Err(e),
                Ok(()) => return Err(stop_text(self.m.cpu.pc(), "the program returned")),
            }
        }
    }

    /// One vertical blank: IRQ 0, and the frame counter.
    fn vblank(&mut self) -> Result<(), Stop> {
        self.last_vblank = self.steps;
        self.frame += 1;
        let (buffers, started) = {
            let sh = self.shared.borrow();
            (sh.pad_buffers, sh.pad_started)
        };
        if let (Some((one, two)), true) = (buffers, started) {
            // Port 1 a digital pad (0x41), buttons active low; port 2 empty.
            let b = !self.pad;
            self.m.bus.load_ram(one, &[0x00, 0x41, b as u8, (b >> 8) as u8]);
            self.m.bus.load_ram(two, &[0xff, 0xff, 0xff, 0xff]);
        }
        self.m.bus.hw.i_stat |= hw::IRQ_VBLANK;
        self.deliver()
    }

    /// Calls libetc's dispatcher while an unmasked interrupt is pending,
    /// outside critical sections, once the dispatcher is installed.
    fn deliver(&mut self) -> Result<(), Stop> {
        for _ in 0..8 {
            let hw = &self.m.bus.hw;
            if hw.i_stat == 0 {
                return Ok(());
            }
            if hw.i_stat & hw.i_mask == 0 {
                self.held[0] += 1;
                return Ok(());
            }
            if !self.irq_enabled.get() {
                self.held[1] += 1;
                return Ok(());
            }
            if self.m.bus.read_u16(addr::INTR_READY).unwrap_or(0) == 0 {
                self.held[2] += 1;
                return Ok(());
            }
            self.dispatched += 1;
            let buf: Vec<u32> = (0..12).map(|k| self.m.bus.ram_u32(addr::INTR_JMPBUF + 4 * k)).collect();
            self.run_nested(addr::INTR_DISPATCH, |cpu| {
                cpu.r[29] = buf[1];
                cpu.r[30] = buf[2];
                cpu.r[16..24].copy_from_slice(&buf[3..11]);
                cpu.r[28] = buf[11];
            })?;
            self.m.bus.hw.advance(self.steps);
        }
        Ok(())
    }

    /// Calls `func(args)` on top of the running program, on a stack below
    /// its own, and puts everything back.
    pub fn call_nested(&mut self, func: u32, args: &[u32]) -> Result<u32, Stop> {
        let args = args.to_vec();
        self.run_nested(func, move |cpu| {
            cpu.r[29] = cpu.r[29].wrapping_sub(0x400) & !7;
            for (k, &a) in args.iter().take(4).enumerate() {
                cpu.r[4 + k] = a;
            }
        })
    }

    /// Runs `func` with the registers `prepare` sets, returning to the
    /// interrupted program afterwards with every register as it was, as an
    /// exception handler does. The devices keep time while it runs (a
    /// handler may wait for a transfer), but no interrupt nests.
    fn run_nested(&mut self, func: u32, prepare: impl FnOnce(&mut R3000)) -> Result<u32, Stop> {
        let cpu = self.m.cpu.clone();
        let (steps, limit, ret) = (self.m.steps, self.m.step_limit, self.m.return_to);
        prepare(&mut self.m.cpu);
        self.m.cpu.r[31] = NESTED_RETURN;
        self.m.cpu.set_pc(func);
        self.m.return_to = NESTED_RETURN;
        let start = self.m.steps;
        let result = loop {
            let before = self.m.steps;
            self.m.step_limit = before + SLICE;
            let r = self.m.run();
            self.steps += self.m.steps - before;
            self.m.bus.hw.advance(self.steps);
            match r {
                Ok(()) => break Ok(self.m.cpu.r[2]),
                Err(Stop::StepLimit { .. }) if self.m.steps - start < 20_000_000 => {}
                // A VSync inside a handler does not end the frame.
                Err(Stop::Halt { .. }) if self.shared.borrow_mut().vsync.take().is_some() => {}
                Err(e) => break Err(e),
            }
        };
        self.m.cpu = cpu;
        (self.m.steps, self.m.step_limit, self.m.return_to) = (steps, limit, ret);
        result
    }

    /// The picture on the TV now, RGBA8.
    pub fn screen(&self) -> (usize, usize, Vec<u8>) {
        self.m.bus.hw.gpu.screen()
    }
}
