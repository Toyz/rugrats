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
pub mod card;
pub mod cdrom;
pub mod gpu;
pub mod hw;
pub mod mdec;
pub mod printf;
pub mod sio;
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
    /// The game's leveled debug print: debug_print(level, fmt, ...).
    pub const DEBUG_PRINT: u32 = 0x8002_f284;
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
    /// The BIOS's interrupt queues (SysEnqIntRP): (priority, entry), each
    /// entry { next, func2, func1 }.
    int_queue: Vec<(u32, u32)>,
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

/// The oracle's card in slot 1: RUGRATS_CARD, or work/oracle/memcard/slot1.
fn card_dir() -> std::path::PathBuf {
    std::env::var_os("RUGRATS_CARD").map(Into::into).unwrap_or_else(|| "work/oracle/memcard/slot1".into())
}

/// The memory card's BIOS calls (see [`card`]): the events (all of them,
/// those of other classes reading as delivered, as the stock BIOS has
/// them), `_card_info`, `_card_load`, the raw sector calls and the file
/// calls on `bu` devices; other files are left to the stock BIOS.
fn install_card(b: &bios::Bios<PsxBus>, slot1: std::path::PathBuf) {
    use card::Card;
    use rrt::emu::BusExt;
    let _ = std::fs::create_dir_all(&slot1);
    let card = Rc::new(RefCell::new(Card::new(slot1)));
    // Event ids, with their class: those of the card classes are the card's.
    let classes: Rc<RefCell<Vec<(u32, u32)>>> = Default::default();
    let next = Rc::new(std::cell::Cell::new(0xf100_0001u32));
    {
        let (card, classes, next) = (card.clone(), classes.clone(), next.clone());
        // B0:08 OpenEvent(class, spec, mode, func)
        b.insert(0xb0, 0x08, move |c| {
            let (class, spec) = (c.arg32(0), c.arg32(1));
            let id = if Card::owns(class) {
                card.borrow_mut().open_event(class, spec)
            } else {
                let id = next.get() | 0x0800_0000;
                next.set(next.get() + 1);
                id
            };
            classes.borrow_mut().push((id, class));
            HookResult::Return(id as u64)
        });
    }
    for f in [0x0a, 0x0b] {
        let card = card.clone();
        // B0:0A WaitEvent(id), B0:0B TestEvent(id)
        b.insert(0xb0, f, move |c| {
            let id = c.arg32(0);
            let mut card = card.borrow_mut();
            let r = if card.is_ours(id) { card.test_event(id) || f == 0x0a } else { true };
            if card.is_ours(id) && r {
                tracing::debug!("card event {id:#x} fired");
            }
            HookResult::Return(r as u64)
        });
    }
    for (f, set) in [(0x07, true), (0x20, false)] {
        let card = card.clone();
        // B0:07 DeliverEvent(class, spec), B0:20 UnDeliverEvent
        b.insert(0xb0, f, move |c| {
            let (class, spec) = (c.arg32(0), c.arg32(1));
            if Card::owns(class) {
                if set {
                    card.borrow_mut().deliver(class, spec);
                } else {
                    card.borrow_mut().undeliver(class, spec);
                }
            }
            HookResult::Return(1)
        });
    }
    {
        let c1 = card.clone();
        b.insert(0xa0, 0xab, move |c| {
            tracing::debug!("card _card_info({:#x})", c.arg32(0));
            c1.borrow_mut().info(c.arg32(0));
            HookResult::Return(1)
        });
        let c2 = card.clone();
        b.insert(0xa0, 0xac, move |c| {
            c2.borrow_mut().load(c.arg32(0));
            HookResult::Return(1)
        });
        // B0:4F _card_read(chan, sector, buf), B0:4E _card_write
        let c3 = card.clone();
        b.insert(0xb0, 0x4f, move |c| {
            let (chan, sector, buf) = (c.arg32(0), c.arg32(1), c.arg32(2));
            let frame = c3.borrow_mut().read_sector(chan, sector);
            tracing::debug!("card _card_read({chan:#x}, {sector})");
            match frame {
                Some(f) => {
                    for (k, v) in f.iter().enumerate() {
                        let _ = c.bus.write_u8(buf + k as u32, *v);
                    }
                    HookResult::Return(1)
                }
                None => {
                    c3.borrow_mut().deliver(card::HW_CARD, card::EV_TIMEOUT);
                    HookResult::Return(1)
                }
            }
        });
        let c4 = card.clone();
        b.insert(0xb0, 0x4e, move |c| {
            c4.borrow_mut().write_sector(c.arg32(0), c.arg32(1));
            HookResult::Return(1)
        });
    }
    // The file calls on bu devices.
    {
        let card = card.clone();
        // B0:32 open(path, flags)
        b.insert(0xb0, 0x32, move |c| {
            let path = c.cstr(0);
            if !path.starts_with("bu") {
                return HookResult::Return(u32::MAX as u64);
            }
            let fd = card.borrow_mut().open(&path, c.arg32(1));
            tracing::debug!("card open({path:?}, {:#x}) = {fd}", c.arg32(1));
            HookResult::Return(fd as u32 as u64)
        });
    }
    {
        let card = card.clone();
        // B0:33 lseek(fd, offset, whence)
        b.insert(0xb0, 0x33, move |c| {
            let fd = c.arg32(0);
            let r = if fd < 2 { 0 } else { card.borrow_mut().seek(fd, c.arg32(1) as i32, c.arg32(2)) };
            HookResult::Return(r as u32 as u64)
        });
    }
    {
        let card = card.clone();
        // B0:34 read(fd, buf, len)
        b.insert(0xb0, 0x34, move |c| {
            let (fd, buf, len) = (c.arg32(0), c.arg32(1), c.arg32(2));
            let Some(bytes) = card.borrow_mut().read(fd, len as usize) else { return HookResult::Return(0) };
            for (k, v) in bytes.iter().enumerate() {
                let _ = c.bus.write_u8(buf + k as u32, *v);
            }
            HookResult::Return(bytes.len() as u64)
        });
    }
    {
        let card = card.clone();
        // B0:35 write(fd, buf, len): fds below 2 are the console.
        b.insert(0xb0, 0x35, move |c| {
            let (fd, buf, len) = (c.arg32(0), c.arg32(1), c.arg32(2));
            if fd < 2 {
                // The console: what the game prints.
                let text: Vec<u8> = (0..len).map(|k| c.bus.read_u8(buf + k).unwrap_or(0)).collect();
                tracing::info!(target: "tty", "{}", String::from_utf8_lossy(&text).trim_end());
                return HookResult::Return(len as u64);
            }
            let data: Vec<u8> = (0..len).map(|k| c.bus.read_u8(buf + k).unwrap_or(0)).collect();
            let r = card.borrow_mut().write(fd, &data);
            HookResult::Return(r as u32 as u64)
        });
    }
    {
        let card = card.clone();
        // B0:36 close(fd)
        b.insert(0xb0, 0x36, move |c| {
            let fd = c.arg32(0);
            if fd < 2 {
                return HookResult::Return(0);
            }
            HookResult::Return(card.borrow_mut().close(fd) as u32 as u64)
        });
    }
    for f in [0x42, 0x43] {
        let card = card.clone();
        // B0:42 firstfile(path, dirent), B0:43 nextfile(dirent): the
        // entry is name[20], attr, size, next, head, system[4].
        b.insert(0xb0, f, move |c| {
            let (path, dirent) = if f == 0x42 { (Some(c.cstr(0)), c.arg32(1)) } else { (None, c.arg32(0)) };
            let found = card.borrow_mut().find(path.as_deref());
            tracing::debug!("card {}({path:?}) = {found:?}", if f == 0x42 { "firstfile" } else { "nextfile" });
            let Some((name, size, head)) = found else { return HookResult::Return(0) };
            let mut e = [0u8; 40];
            e[..name.len().min(19)].copy_from_slice(&name.as_bytes()[..name.len().min(19)]);
            e[20..24].copy_from_slice(&0x51u32.to_le_bytes());
            e[24..28].copy_from_slice(&size.to_le_bytes());
            e[32..36].copy_from_slice(&head.to_le_bytes());
            for (k, v) in e.iter().enumerate() {
                let _ = c.bus.write_u8(dirent + k as u32, *v);
            }
            HookResult::Return(dirent as u64)
        });
    }
    {
        let card = card.clone();
        // B0:45 erase(path)
        b.insert(0xb0, 0x45, move |c| {
            let path = c.cstr(0);
            HookResult::Return(card.borrow_mut().erase(&path) as u64)
        });
    }
    {
        // B0:41 format(path)
        b.insert(0xb0, 0x41, move |c| {
            let path = c.cstr(0);
            HookResult::Return(card.borrow_mut().format(&path) as u64)
        });
    }
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
        // A0:AD set_card_auto_format(flag).
        b.insert(0xa0, 0xad, |_| HookResult::Return(0));
        install_card(&b, card_dir());
        // The kernel's device table (pointer at 0x150, size at 0x154, 80
        // bytes a device): libapi's firstfile (0x8009ac98) looks the device
        // up by name there before calling the BIOS. One device, "bu".
        let (devices, bu) = (0x0000_e000u32, 0x0000_e100u32);
        m.bus.load_ram(0x150, &[devices.to_le_bytes(), 80u32.to_le_bytes()].concat());
        m.bus.load_ram(devices, &bu.to_le_bytes());
        m.bus.load_ram(bu, b"bu\0");
        // B0:5C _card_status(slot), B0:5D _card_wait(slot): ready.
        b.insert(0xb0, 0x5c, |_| HookResult::Return(1));
        b.insert(0xb0, 0x5d, |_| HookResult::Return(1));
        // C0:02 SysEnqIntRP(priority, entry), C0:03 SysDeqIntRP: the
        // BIOS's interrupt queues, which `Hle::deliver` runs (libpad's
        // direct driver queues its vertical-blank handler there).
        {
            let sh = shared.clone();
            b.insert(0xc0, 0x02, move |c| {
                let e = (c.arg32(0), c.arg32(1));
                let mut sh = sh.borrow_mut();
                tracing::debug!("SysEnqIntRP({}, {:#x})", e.0, e.1);
                if !sh.int_queue.contains(&e) {
                    sh.int_queue.push(e);
                    sh.int_queue.sort_by_key(|&(p, _)| p);
                }
                HookResult::Return(0)
            });
            let sh = shared.clone();
            b.insert(0xc0, 0x03, move |c| {
                let e = (c.arg32(0), c.arg32(1));
                sh.borrow_mut().int_queue.retain(|&q| q != e);
                HookResult::Return(0)
            });
        }
        // B0:12 InitPAD(buf1, size1, buf2, size2), B0:13 StartPAD, B0:14
        // StopPAD: the BIOS's pad driver fills the buffers at each vertical
        // blank; `Hle::vblank` does.
        {
            let sh = shared.clone();
            b.insert(0xb0, 0x12, move |c| {
                tracing::debug!("InitPAD({:#x}, {:#x})", c.arg32(0), c.arg32(2));
                sh.borrow_mut().pad_buffers = Some((c.arg32(0), c.arg32(2)));
                HookResult::Return(1)
            });
            let sh = shared.clone();
            b.insert(0xb0, 0x13, move |_| {
                tracing::debug!("StartPAD");
                sh.borrow_mut().pad_started = true;
                HookResult::Return(1)
            });
            let sh = shared.clone();
            b.insert(0xb0, 0x14, move |_| {
                tracing::debug!("StopPAD");
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

        m.hook(addr::DEBUG_PRINT, |c| {
            let level = c.arg32(0);
            let text = printf::format_from(c, 1);
            tracing::debug!(target: "game", "[{level}] {}", text.trim_end());
            HookResult::Return(0)
        });
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
        // libpad's direct driver reads the pad through SIO0.
        self.m.bus.hw.sio.buttons = self.pad;
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
            // The BIOS runs its queues first: each entry's func1, and its
            // func2 with what func1 returned when that is not 0.
            let queue = self.shared.borrow().int_queue.clone();
            for (_, entry) in queue {
                let (func2, func1) = (self.m.bus.ram_u32(entry + 4), self.m.bus.ram_u32(entry + 8));
                if func1 == 0 {
                    continue;
                }
                let v = self.call_nested(func1, &[])?;
                if v != 0 && func2 != 0 {
                    self.call_nested(func2, &[v])?;
                }
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
        // A load the interrupted program has in flight would land on the
        // handler's first instruction; it stays pending in the saved copy.
        self.m.cpu.settle();
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
