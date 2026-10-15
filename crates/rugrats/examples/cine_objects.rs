//! Lists where each placed object of database 22 lands on screen at
//! frame F of the opening, to find one in a picture.
//!
//! cargo run -p rugrats --example cine_objects -- F

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let want: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let stream = rugrats::cine::Stream::read(&disc, 0)?;
    let (_, f) = &stream.frames[want - 1];
    let m = rugrats::cine::camera(f);
    let lp = rugrats_data::placement::parse_lp(&disc.file(402 + 22)?);
    let templates = rugrats_data::placement::parse_templates(&disc.file(239)?);
    for (k, p) in lp.iter().enumerate().filter(|(_, p)| p.kind == 1) {
        let c = m * rrt::glam::Vec4::new(p.pos[0] as f32, p.pos[1] as f32, p.pos[2] as f32, 1.0);
        if c.w <= 0.0 {
            continue;
        }
        let (x, y) = ((c.x / c.w + 1.0) * 256.0, (1.0 - c.y / c.w) * 120.0);
        if (-50.0..562.0).contains(&x) && (-50.0..290.0).contains(&y) {
            let t = templates[p.template as usize];
            println!("lp {k:2} id {:4} template {:4} shape {:4} hier {} at ({x:.0}, {y:.0}) depth {:.0}", p.id, p.template, t.shape, t.flags & 1, c.w);
        }
    }
    Ok(())
}
