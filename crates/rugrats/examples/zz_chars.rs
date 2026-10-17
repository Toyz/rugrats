use std::rc::Rc;
fn main() -> Result<(), String> {
    let cue = rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?;
    let mut p = rugrats_hle::play::Play::boot(Rc::new(rrt::disc::Image::open(&cue).map_err(|e| e.to_string())?), rugrats_hle::play::Boot { level: 1, ..Default::default() })?;
    let rooms = p.read_u32(0x800c_4a8c);
    let count = p.read_i32(0x800c_4a9c);
    for r in 0..count.max(0) as u32 {
        for list in [0x48u32, 0x4c] {
            let mut node = p.read_u32(rooms + 112 * r + list);
            while node != 0 {
                let o = p.read_u32(node + 8);
                if p.read_u32(o + 0x88) == 0 {
                    let h = p.read_u32(o + 0x154);
                    let hdr: Vec<i16> = (0..26).map(|k| p.read_i16(h + 2 * k)).collect();
                    let bound: Vec<i16> = (0..8).map(|k| p.read_i16(o + 0x78 + 2 * k)).collect();
                    println!("{o:08x} type {} pos {:?} angles {:?} bound {bound:?} hdr {hdr:?}", p.read_i16(o + 0x96), [0, 4, 8].map(|k| p.read_i32(o + 0x18 + k)), [0, 2, 4].map(|k| p.read_i16(o + 0x24 + k)));
                }
                node = p.read_u32(node + 4);
            }
        }
    }
    Ok(())
}
