//! Lists who calls the game's rand (0x8002fb7c) from frame FROM to TO: the
//! frame, the caller's return address and the state before.
//!
//! cargo run -p rugrats-hle --example rand_calls -- FROM TO

use std::cell::Cell;
use std::rc::Rc;

use rrt::emu::{BusExt, Cpu, HookResult};

fn main() {
    let mut a = std::env::args().skip(1).map(|s| s.parse::<u64>().unwrap());
    let (from, to) = (a.next().unwrap_or(1500), a.next().unwrap_or(1520));
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    while hle.frame < from {
        hle.run_frame().unwrap();
    }
    let frame = Rc::new(Cell::new(hle.frame));
    let f = frame.clone();
    hle.m.hook(0x8002_fb7c, move |c| {
        let ra = c.cpu.reg(31) as u32;
        let state = c.bus.read_u32(0x800c_3f94).unwrap_or(0);
        println!("frame {} rand from {ra:08x} state {state:08x}", f.get());
        HookResult::Continue
    });
    while hle.frame < to {
        hle.run_frame().unwrap();
        frame.set(hle.frame);
    }
}
