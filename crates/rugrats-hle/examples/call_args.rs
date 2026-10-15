//! Runs the original FRAMES frames with PRESSES (as rugrats-hle's
//! --press: frame:buttons:len, comma-separated) and prints each call of
//! ADDR from frame FROM on: the frame, a0-a3 and the caller.
//!
//! cargo run -p rugrats-hle --example call_args -- ADDR FROM FRAMES [PRESSES]

use std::cell::Cell;
use std::rc::Rc;

use rrt::emu::{Cpu, HookResult};

fn main() {
    let mut a = std::env::args().skip(1);
    let num = |s: String| if let Some(h) = s.strip_prefix("0x") { u64::from_str_radix(h, 16).unwrap() } else { s.parse().unwrap() };
    let addr = a.next().map(num).unwrap_or(0x8006_eac4) as u32;
    let from = a.next().map(num).unwrap_or(0);
    let frames = a.next().map(num).unwrap_or(7000);
    let presses: Vec<(u64, u16, u64)> = a
        .next()
        .unwrap_or_default()
        .split(',')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let f: Vec<&str> = p.split(':').collect();
            let buttons = f.get(1).map_or(0, |n| n.split('+').filter_map(rugrats_hle::button::by_name).fold(0, |a, b| a | b));
            (f[0].parse().unwrap(), buttons, f.get(2).and_then(|s| s.parse().ok()).unwrap_or(4))
        })
        .collect();
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    let frame = Rc::new(Cell::new(0u64));
    let f = frame.clone();
    hle.m.hook(addr, move |c| {
        if f.get() >= from {
            let r = |k| c.cpu.reg(k) as u32;
            println!("frame {}: {addr:08x}({:#x}, {:#x}, {:#x}, {:#x}) from {:08x}", f.get(), r(4), r(5), r(6), r(7), r(31));
        }
        HookResult::Continue
    });
    while hle.frame < frames {
        let held = presses.iter().filter(|(at, _, len)| (*at..at + len).contains(&hle.frame)).fold(0, |a, p| a | p.1);
        hle.pad = held;
        hle.run_frame().unwrap();
        frame.set(hle.frame);
    }
}
