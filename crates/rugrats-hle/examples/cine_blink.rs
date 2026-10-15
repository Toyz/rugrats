//! Runs the original through the opening, printing the cinematic frame
//! counter (0x800c4c34) and the blink rand (0x800c4c48) each tick it
//! changes, to set the port's blinks against. With BLINK_CALLS, lists
//! each blink (0x80075698) instead: counter, id, state and count before.
//!
//! cargo run -p rugrats-hle --example cine_blink -- [LAST]

use std::rc::Rc;

use rrt::emu::{BusExt, Cpu, HookResult};

fn main() {
    let last: u16 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(900);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    let calls = std::env::var_os("BLINK_CALLS").is_some();
    if calls {
        hle.m.hook(0x8007_5698, |c| {
            let at = c.bus.read_u16(0x800c_4c34).unwrap_or(0);
            let id = c.cpu.reg(4) as u16 as i16;
            let seed = c.bus.read_u32(0x800c_4c48).unwrap_or(0);
            println!("call {at} {id} {seed:08x}");
            HookResult::Continue
        });
    }
    let mut seen = (0u16, 0u32);
    for _ in 0..8000 {
        hle.run_frame().unwrap();
        let at = hle.m.bus.read_u16(0x800c_4c34).unwrap();
        let seed = hle.m.bus.read_u32(0x800c_4c48).unwrap();
        if (at, seed) != seen && at > 0 && !calls {
            println!("{at} {seed:08x}");
            seen = (at, seed);
        }
        if at >= last {
            break;
        }
    }
}
