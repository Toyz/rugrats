//! Runs the original until the cinematic frame counter (0x800c4c34)
//! reaches F, and writes the displayed picture.
//!
//! cargo run -p rugrats-hle --example cine_shot -- F OUT.png

use std::rc::Rc;

use rrt::emu::BusExt;

fn main() {
    let mut args = std::env::args().skip(1);
    let want: u16 = args.next().and_then(|s| s.parse().ok()).unwrap_or(200);
    let out = args.next().unwrap_or_else(|| "work/hle/cine.png".into());
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).unwrap();
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(&cue).unwrap()), true).unwrap();
    let mut started = false;
    for _ in 0..6000 {
        hle.run_frame().unwrap();
        let at = hle.m.bus.read_u16(0x800c_4c34).unwrap();
        started |= at > 0 && at < want;
        if started && at >= want {
            // The frame drawn for this counter shows a frame later.
            hle.run_frame().unwrap();
            let (w, h, rgba) = hle.screen();
            std::fs::write(&out, rrt::image::png::encode(w as u32, h as u32, &rgba)).unwrap();
            println!("cine frame {at} at oracle frame {} -> {out}", hle.frame);
            // The camera (0x80073b08 keeps it at 0x800e5648) and the
            // registry's jointed objects, to set against the stream.
            let b = &mut hle.m.bus;
            let cam = [0, 4, 8].map(|k| b.read_u32(0x800e_5648 + k).unwrap() as i32);
            println!("camera at {cam:?}");
            let n = b.read_u16(0x800c_4144).unwrap() as i16;
            let (ids, objs) = (b.read_u32(0x800c_413c).unwrap(), b.read_u32(0x800c_4140).unwrap());
            for k in 0..n.max(0) as u32 {
                let o = b.read_u32(objs + 4 * k).unwrap();
                if b.read_u32(o + 0x154).unwrap() == 0 {
                    continue;
                }
                let id = b.read_u16(ids + 2 * k).unwrap() as i16;
                let pos = [0x18, 0x1c, 0x20].map(|a| b.read_u32(o + a).unwrap() as i32);
                let scale = b.read_u16(o + 0x36).unwrap() as i16;
                println!("id {id:4} at {pos:?} scale {scale}");
                if std::env::var("CINE_PARTS").is_ok_and(|v| v.split(',').any(|s| s == id.to_string())) {
                    // Each part's quaternion as UpdatePartRotations reads it.
                    let h = b.read_u32(o + 0x154).unwrap();
                    let n = b.read_u16(h).unwrap() as i16;
                    for k in 0..n.max(0) as u32 {
                        let p = h + 52 + 40 * k;
                        let q = [4, 8, 12, 16].map(|a| b.read_u32(p + a).unwrap() as i32);
                        println!("  part {k:2} depth {} q {q:?}", b.read_u16(p + 2).unwrap());
                    }
                }
            }
            return;
        }
    }
    println!("never reached cine frame {want}");
}
