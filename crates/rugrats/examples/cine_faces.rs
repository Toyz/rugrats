//! The opening's face records: per id, the frames whose mouth or eyes
//! fields are set; with F, writes VRAM after stream frame F to OUT.
//!
//! cargo run -p rugrats --example cine_faces -- [F OUT.bin]

use std::collections::BTreeMap;

use rugrats::disc::GameDisc;
use rugrats::face::Blinks;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = rugrats::cine::Stream::read(&disc, 0)?;
    let mut seen: BTreeMap<(&str, i16), Vec<(u16, u8, u8)>> = BTreeMap::new();
    for (_, f) in &s.frames {
        for a in f.actors.iter().filter(|a| a.mouth | a.eyes != 0) {
            seen.entry(("actor", a.id)).or_default().push((f.number, a.mouth, a.eyes));
        }
        for p in f.poses.iter().filter(|p| p.mouth | p.eyes != 0) {
            seen.entry(("pose", p.id)).or_default().push((f.number, p.mouth, p.eyes));
        }
    }
    let lp = rugrats_data::placement::parse_lp(&disc.file(402 + 22)?);
    for (k, v) in &seen {
        println!("{k:?}: {} records, first {:?}", v.len(), &v[..v.len().min(8)]);
        println!("  placed: {:?}", lp.iter().filter(|p| p.id as i16 == k.1).collect::<Vec<_>>());
    }
    if std::env::var_os("FLAGS").is_some() {
        return flags_probe(&disc);
    }
    let mut a = std::env::args().skip(1);
    if let (Some(want), Some(out)) = (a.next().and_then(|s| s.parse::<u16>().ok()), a.next()) {
        let set = rugrats::cine::Set::load(&disc, 22)?;
        let mut vram = set.vram.clone();
        let mut blinks = Blinks::new(0);
        for (_, f) in s.frames.iter().take_while(|(_, f)| f.number <= want) {
            set.faces(f, &mut blinks, &mut vram);
            if std::env::var_os("SEEDS").is_some() {
                println!("{} {:08x}", f.number, blinks.seed());
            }
        }
        let bytes: Vec<u8> = vram.words.iter().flat_map(|w| w.to_le_bytes()).collect();
        std::fs::write(&out, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[allow(dead_code)]
fn flags_probe(disc: &GameDisc) -> Result<(), String> {
    let t = rugrats_data::placement::parse_templates(&disc.file(239)?);
    let models = rugrats_data::model::Models::parse(&disc.file(242 + 22)?).map_err(|e| e.to_string())?;
    let raw = disc.file(239)?;
    let lp = rugrats_data::placement::parse_lp(&disc.file(402 + 22)?);
    for p in lp.iter().filter(|p| matches!(p.template, 1149 | 1130)) {
        println!("lp {p:?} model found {}", models.find(t[p.template as usize].shape as u32));
    }
    println!("1130 shape {} found {}, 1149 shape {} found {}", t[1130].shape, models.find(t[1130].shape as u32), t[1149].shape, models.find(t[1149].shape as u32));
    for tid in [626usize, 704] {
        let tm = t[tid];
        println!("raw {:02x?}", &raw.get(4 + 56 * tid..4 + 56 * tid + 56));
        println!("template {tid}: {tm:?} at {:#x}", models.find(tm.shape as u32));
        if let Ok(out) = std::env::var("MOD_OUT") {
            std::fs::write(format!("{out}.{tid}"), &disc.file(242 + 22)?[models.find(tm.shape as u32) as usize..][..256]).map_err(|e| e.to_string())?;
        }
        if let Some(m) = models.get(tm.shape as u32) {
            let mut seen = std::collections::BTreeMap::new();
            for f in &m.faces {
                *seen.entry((f.flags, f.texture.map(|t| (t.1, t.2)))).or_insert(0) += 1;
            }
            for (k, n) in seen {
                println!("  flags {:08x} clut/page {:?} x{n}", k.0, k.1.map(|(c, p)| (format!("{c:04x}"), format!("{p:04x}"))));
            }
        }
    }
    Ok(())
}
