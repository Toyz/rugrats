//! At cinematic frame F, lists the original's face emitters (the leaves
//! 0x8002242c..0x80022bc4 and 0x800236d4..0x800238ac, and the dispatchers
//! 0x8002390c, 0x800237e0) that
//! draw semi-transparent faces: the leaf, its flags and its caller.
//!
//! cargo run -p rugrats-hle --example face_calls -- F

use std::rc::Rc;

use rrt::emu::{BusExt, Cpu, HookResult};

const LEAVES: [u32; 15] = [
    0x8002242c, 0x800224c8, 0x80022598, 0x80022664, 0x80022764, 0x80022880, 0x80022964, 0x80022a7c, 0x8002390c, 0x800237e0,
    0x800236d4, 0x80023714, 0x80023774, 0x80023840, 0x800238ac,
];

fn main() {
    let want: u16 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(840);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    for _ in 0..8000 {
        hle.run_frame().unwrap();
        if hle.m.bus.read_u16(0x800c_4c34).unwrap() >= want {
            break;
        }
    }
    // RAM=PATH: main RAM at that point, 2 MB.
    if let Some(path) = std::env::var_os("RAM") {
        let ram: Vec<u8> = (0..0x20_0000u32).map(|a| hle.m.bus.read_u8(0x8000_0000 + a).unwrap_or(0)).collect();
        std::fs::write(path, ram).unwrap();
    }
    for at in LEAVES {
        hle.m.hook(at, move |c| {
            let flags = c.cpu.reg(4) as u32;
            if flags & 0x0200_0000 != 0 || std::env::var_os("ALL").is_some() {
                println!("{at:08x} flags {flags:08x} from {:08x}", c.cpu.reg(31) as u32);
            }
            HookResult::Continue
        });
    }
    // AddPrim (0x8002214c, ot, packet): a semi FT3 (0x26) and who adds it.
    hle.m.hook(0x8002_214c, |c| {
        let p = c.cpu.reg(5) as u32;
        let code = c.bus.read_u32(p + 4).unwrap_or(0) >> 24;
        if code & 0xfe == 0x26 || code & 0xfe == 0x2e {
            let clut = c.bus.read_u32(p + 12).unwrap_or(0) >> 16;
            println!("addprim {code:02x} clut {clut:04x} from {:08x}", c.cpu.reg(31) as u32);
        }
        HookResult::Continue
    });
    if std::env::var_os("SANITY").is_some() {
        hle.m.hook(0x8007_3c34, |_| {
            println!("actor reader");
            HookResult::Continue
        });
    }
    hle.run_frame().unwrap();
    hle.run_frame().unwrap();
}
