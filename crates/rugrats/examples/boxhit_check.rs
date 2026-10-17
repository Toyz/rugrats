//! Holds `world::boxhit::slab` to the original's slab test (0x80068e68)
//! and `world::boxhit::object` to its object box test (0x80069c40), called
//! on random segments, boxes and objects in a fresh oracle (no level
//! needed). The object is laid out as the original reads it: +0x18 its
//! position, +0x24 its angles, +0x88 its model (+10 the box count, +12 the
//! boxes, 16 bytes each: lo x, y, z halfwords, then hi at +8).
//!
//! `boxhit_check [--n N]`
use std::path::Path;
use std::rc::Rc;

use rrt::emu::BusExt;
use rugrats::world::boxhit;
use rugrats_hle::Hle;

const SLAB: u32 = 0x8006_8e68;
const OBJECT: u32 = 0x8006_9c40;
const GP: u32 = 0x800c_3e80;
/// Scratch RAM for the arguments, and a stack under it.
const BUF: u32 = 0x801c_0000;
const STACK: u32 = 0x801f_f000;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u32) as i32
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut n = 20000;
    while let Some(a) = args.next() {
        if a == "--n" {
            n = args.next().and_then(|s| s.parse().ok()).unwrap_or(n);
        }
    }
    let cue = rrt::disc::Image::find(Path::new("work/disc")).map_err(|e| e.to_string())?;
    let mut hle = Hle::new(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), true)?;
    hle.m.cpu.r[29] = STACK;
    hle.m.cpu.r[28] = GP;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut same, mut bad) = (0, 0);
    for i in 0..n {
        let size = [0; 3].map(|_| if rng.next().is_multiple_of(16) { 0 } else { rng.range(1, 2000) });
        let pt = |rng: &mut Rng, k: usize| {
            if rng.next().is_multiple_of(8) {
                // On a face.
                if rng.next().is_multiple_of(2) { 0 } else { size[k] }
            } else {
                rng.range(-1500, size[k] + 1500)
            }
        };
        let a = [0, 1, 2].map(|k| pt(&mut rng, k));
        let b = [0, 1, 2].map(|k| pt(&mut rng, k));
        let w = |hle: &mut Hle, at: u32, v: i32| hle.m.bus.write_u32(at, v as u32).ok();
        for k in 0..3 {
            w(&mut hle, BUF + 4 * k, a[k as usize]);
            w(&mut hle, BUF + 16 + 4 * k, b[k as usize]);
            w(&mut hle, BUF + 32 + 4 * k, size[k as usize]);
            w(&mut hle, BUF + 48 + 4 * k, 0x5555);
            w(&mut hle, BUF + 64 + 4 * k, 0x5555);
        }
        let r = hle.call_nested(SLAB, &[BUF, BUF + 32, BUF + 48, BUF + 64]).map_err(|e| format!("{e:?}"))? as i32;
        let rd = |hle: &mut Hle, at: u32| hle.m.bus.read_u32(at).unwrap_or(0) as i32;
        let point = [0, 1, 2].map(|k| rd(&mut hle, BUF + 48 + 4 * k));
        let normal = [0, 1, 2].map(|k| rd(&mut hle, BUF + 64 + 4 * k));
        let port = boxhit::slab(a, b, size);
        let ok = match port {
            None => r < 0,
            Some(h) if h.t == 0 && h.normal == [0; 3] && point == a && r == 0 && normal == [0x5555; 3] => true,
            Some(h) => r == h.t && point == h.point && normal == h.normal,
        };
        if ok {
            same += 1;
        } else {
            bad += 1;
            if bad <= 20 {
                println!("{i}: a {a:?} b {b:?} size {size:?}: original {r} {point:?} {normal:?}; port {port:?}");
            }
        }
    }
    println!("slab: {same} the same, {bad} different");
    let trig = rugrats::gte::Trig::load(&rugrats::disc::GameDisc::open(&cue)?)?;
    let (mut same2, mut bad2) = (0, 0);
    let (obj, model, boxes_at) = (BUF + 0x100, BUF + 0x200, BUF + 0x300);
    for i in 0..n {
        let pos = [0; 3].map(|_| rng.range(-30000, 30000));
        let angles = [0; 3].map(|_| if rng.next().is_multiple_of(3) { 0 } else { rng.range(-4096, 4096) as i16 });
        let count = rng.range(1, 4) as usize;
        let boxes: Vec<([i32; 3], [i32; 3])> = (0..count)
            .map(|_| {
                let lo = [0; 3].map(|_| rng.range(-900, 100));
                let hi = lo.map(|l| l + rng.range(0, 1200));
                (lo, hi)
            })
            .collect();
        let near = |rng: &mut Rng| [0, 1, 2].map(|k| pos[k] + rng.range(-1500, 1500));
        let (a, b) = (near(&mut rng), near(&mut rng));
        let w = |hle: &mut Hle, at: u32, v: i32| hle.m.bus.write_u32(at, v as u32).ok();
        let h = |hle: &mut Hle, at: u32, v: i32| hle.m.bus.write_u16(at, v as u16).ok();
        for k in 0..3u32 {
            w(&mut hle, obj + 0x18 + 4 * k, pos[k as usize]);
            h(&mut hle, obj + 0x24 + 2 * k, angles[k as usize] as i32);
            w(&mut hle, BUF + 4 * k, a[k as usize]);
            w(&mut hle, BUF + 16 + 4 * k, b[k as usize]);
            w(&mut hle, BUF + 48 + 4 * k, 0x5555);
            w(&mut hle, BUF + 64 + 4 * k, 0);
        }
        w(&mut hle, obj + 0x88, model as i32);
        h(&mut hle, model + 10, count as i32);
        w(&mut hle, model + 12, boxes_at as i32);
        for (j, (lo, hi)) in boxes.iter().enumerate() {
            for k in 0..3u32 {
                h(&mut hle, boxes_at + 16 * j as u32 + 2 * k, lo[k as usize]);
                h(&mut hle, boxes_at + 16 * j as u32 + 8 + 2 * k, hi[k as usize]);
            }
        }
        // The fifth argument (the margin, 0 from every caller) on the stack.
        let sp = hle.m.cpu.r[29];
        let margin_at = (sp.wrapping_sub(0x400) & !7) + 16;
        w(&mut hle, margin_at, 0);
        let r = hle.call_nested(OBJECT, &[BUF, obj, BUF + 48, BUF + 64]).map_err(|e| format!("{e:?}"))? as i32;
        let rd = |hle: &mut Hle, at: u32| hle.m.bus.read_u32(at).unwrap_or(0) as i32;
        let point = [0, 1, 2].map(|k| rd(&mut hle, BUF + 48 + 4 * k));
        let index = rd(&mut hle, BUF + 60);
        let normal = [0, 1, 2].map(|k| rd(&mut hle, BUF + 64 + 4 * k));
        let port = boxhit::object(&trig, a, b, pos, angles, &boxes);
        let ok = match port {
            None => r < 0,
            Some((k, h)) => r == h.t && point == h.point && index == k as i32 && normal == h.normal,
        };
        if ok {
            same2 += 1;
        } else {
            bad2 += 1;
            if bad2 <= 20 {
                println!("{i}: a {a:?} b {b:?} pos {pos:?} angles {angles:?} boxes {boxes:?}: original {r} {point:?} #{index} {normal:?}; port {port:?}");
            }
        }
    }
    println!("object: {same2} the same, {bad2} different");
    bad += bad2;
    if bad > 0 {
        return Err(format!("{bad} different"));
    }
    Ok(())
}
