//! Runs the original to frame N and lists the object registry: id, object,
//! template (+0x96), model (+0x88), hierarchy copy (+0x154) and its part
//! count and first part's model pointer.
//!
//! cargo run -p rugrats-hle --example objects -- 1700

use std::rc::Rc;

use rrt::emu::BusExt;

fn main() {
    let frames: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1700);
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    while hle.frame < frames {
        hle.run_frame().unwrap();
    }
    let reg = |name: &str| rrt::emu::Cpu::reg_index(&hle.m.cpu, name).map(|i| rrt::emu::Cpu::reg(&hle.m.cpu, i) as u32);
    println!("GTE ofx {:?} ofy {:?} h {:?} dqa {:?} dqb {:?} zsf3 {:?} zsf4 {:?}", reg("ofx"), reg("ofy"), reg("h"), reg("dqa"), reg("dqb"), reg("zsf3"), reg("zsf4"));
    let b = &mut hle.m.bus;
    let n = b.read_u16(0x800c_4144).unwrap() as i16;
    let (ids, objs) = (b.read_u32(0x800c_413c).unwrap(), b.read_u32(0x800c_4140).unwrap());
    println!("{n} objects");
    for k in 0..n.max(0) as u32 {
        let id = b.read_u16(ids + 2 * k).unwrap() as i16;
        let o = b.read_u32(objs + 4 * k).unwrap();
        let template = b.read_u16(o + 0x96).unwrap();
        let model = b.read_u32(o + 0x88).unwrap();
        let hier = b.read_u32(o + 0x154).unwrap();
        let (parts, first) = if hier != 0 {
            (b.read_u16(hier).unwrap() as i16, b.read_u32(hier + 0x58).unwrap())
        } else {
            (0, 0)
        };
        let pos = [o + 0x18, o + 0x1c, o + 0x20].map(|a| b.read_u32(a).unwrap() as i32);
        println!("id {id:5} obj {o:08x} template {template:5} model {model:08x} hier {hier:08x} parts {parts:3} part0 model {first:08x} at {pos:?}");
    }
}
