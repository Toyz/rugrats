//! The air handler's push (0x80061330) read off its stack during a running
//! jump from level 1's start, Up and Left held: before the camera's turn
//! (0x800616ac, the angle in a0) and after the body's (0x800616d4).
use std::rc::Rc;

use rrt::emu::{BusExt, Cpu, HookResult};
use rugrats_hle::button::{CROSS, LEFT, UP};
use rugrats_hle::play::{Boot, Play};

fn main() -> Result<(), String> {
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?;
    let mut p = Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), Boot { level: 1, ..Default::default() })?;
    for (at, what) in [(0x8006_16ac_u32, "before"), (0x8006_16d4, "after")] {
        p.hle.m.hook(at, move |c| {
            let (sp, a0) = (c.cpu.reg(29) as u32, c.cpu.reg(4) as u32 as i32);
            let v: Vec<i32> = [32, 36, 40].iter().map(|&k| c.bus.read_u32(sp + k).unwrap_or(0) as i32).collect();
            println!("{what}: a0 {a0} push {v:?}");
            HookResult::Continue
        });
    }
    let steps: [(u16, usize); 4] = [(0, 3), (UP | LEFT, 12), (UP | LEFT | CROSS, 1), (UP | LEFT, 14)];
    let mut k = 0;
    for (pad, n) in steps {
        for _ in 0..n {
            p.frame(pad)?;
            let pl = p.player();
            println!("frame {k}: pos {:?} heading {} vel {:?} cam {}", pl.pos, pl.heading, pl.velocity, p.camera_heading());
            k += 1;
        }
    }
    Ok(())
}
