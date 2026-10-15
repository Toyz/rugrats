//! The opening's actor and pose ids, with the frames they appear in, and
//! the length of its sound.

use std::collections::BTreeMap;

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = rugrats::media::cine::Stream::read(&disc, 0)?;
    let mut actors: BTreeMap<i16, (u16, u16, usize)> = BTreeMap::new();
    let mut poses: BTreeMap<i16, (u16, u16, usize)> = BTreeMap::new();
    for (_, f) in &s.frames {
        for a in &f.actors {
            let e = actors.entry(a.id).or_insert((f.number, f.number, 0));
            e.1 = f.number;
            e.2 += 1;
        }
        for p in &f.poses {
            let e = poses.entry(p.id).or_insert((f.number, f.number, 0));
            e.1 = f.number;
            e.2 += 1;
        }
    }
    println!("actors (id: first, last, count): {actors:?}");
    println!("poses: {poses:?}");
    println!("sound: {:.1} s", s.sound.len() as f64 / 44100.0);
    Ok(())
}
