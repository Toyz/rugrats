//! Runs the original until the cinematic frame counter (0x800c4c34)
//! reaches F, and writes the displayed picture (stream frame F - 1). Then
//! lists the camera and the registry's jointed objects (all with
//! CINE_ALL; parts' quaternions for the ids in CINE_PARTS), writes VRAM
//! with CINE_VRAM=PATH, and traces polygons with CINE_RECT.
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
            // The picture shown for this counter is the stream's frame
            // before it, a tick later. CINE_RECT=X0,Y0,X1,Y1 then traces
            // the polygons drawn there (VRAM coordinates, both buffers)
            // over the next two ticks, the next picture's drawing.
            hle.run_frame().unwrap();
            let (w, h, rgba) = hle.screen();
            std::fs::write(&out, rrt::image::png::encode(w as u32, h as u32, &rgba)).unwrap();
            println!("cine frame {at} at oracle frame {} -> {out}", hle.frame);
            hle.m.bus.hw.gpu.trace = std::env::var("CINE_RECT").ok().and_then(|s| {
                let v: Vec<i32> = s.split(',').filter_map(|v| v.parse().ok()).collect();
                v.try_into().ok()
            });
            if hle.m.bus.hw.gpu.trace.is_some() {
                hle.run_frame().unwrap();
                hle.run_frame().unwrap();
                hle.m.bus.hw.gpu.trace = None;
            }
            // CINE_VRAM=PATH: VRAM as raw little-endian halfwords, 1024 x 512.
            if let Some(path) = std::env::var_os("CINE_VRAM") {
                let bytes: Vec<u8> = hle.m.bus.hw.gpu.vram.iter().flat_map(|h| h.to_le_bytes()).collect();
                std::fs::write(path, bytes).unwrap();
            }
            // The camera (0x80073b08 keeps it at 0x800e5648) and the
            // registry's jointed objects, to set against the stream.
            let b = &mut hle.m.bus;
            let cam = [0, 4, 8].map(|k| b.read_u32(0x800e_5648 + k).unwrap() as i32);
            println!("camera at {cam:?}");
            let n = b.read_u16(0x800c_4144).unwrap() as i16;
            let (ids, objs) = (b.read_u32(0x800c_413c).unwrap(), b.read_u32(0x800c_4140).unwrap());
            for k in 0..n.max(0) as u32 {
                let o = b.read_u32(objs + 4 * k).unwrap();
                if b.read_u32(o + 0x154).unwrap() == 0 && std::env::var_os("CINE_ALL").is_none() {
                    continue;
                }
                let id = b.read_u16(ids + 2 * k).unwrap() as i16;
                let pos = [0x18, 0x1c, 0x20].map(|a| b.read_u32(o + a).unwrap() as i32);
                let scale = b.read_u16(o + 0x36).unwrap() as i16;
                let (template, model) = (b.read_u16(o + 0x96).unwrap(), b.read_u32(o + 0x88).unwrap());
                println!("id {id:4} at {pos:?} scale {scale} template {template} model {model:08x} flags {:08x}", b.read_u32(o).unwrap());
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
