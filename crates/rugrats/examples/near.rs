//! Placements (kind 1) near a point, with their template, flags, class
//! and model boxes: `near DB X Z RADIUS`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let a: Vec<i32> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let set = rugrats::cine::Set::load(&disc, a[0] as usize)?;
    for (k, p) in set.placements.iter().enumerate().filter(|(_, p)| p.kind == 1) {
        let (dx, dz) = ((p.pos[0] - a[1]) as i64, (p.pos[2] - a[2]) as i64);
        if dx * dx + dz * dz > (a[3] as i64).pow(2) {
            continue;
        }
        let t = set.templates.get(p.template as usize);
        let boxes = t.and_then(|t| set.models.get(t.shape as u32)).map(|m| m.boxes.clone());
        let rot = rugrats_render::models::quat_matrix(p.rotation.map(|v| v as i16)).transpose();
        let corners: Vec<(i32, i32)> = boxes.iter().flatten().take(1).flat_map(|(lo, hi)| {
            [(lo[0], lo[2]), (hi[0], lo[2]), (hi[0], hi[2]), (lo[0], hi[2])].map(|(x, z)| {
                let w = rot * rrt::glam::Vec3::new(x as f32, 0.0, z as f32);
                (p.pos[0] + w.x as i32, p.pos[2] + w.z as i32)
            })
        }).collect();
        println!("  corners {corners:?}");
        println!(
            "LP {k}: template {} id {} at {:?} class {:?} flags {:x?} boxes {:?}",
            p.template,
            p.id,
            p.pos,
            t.map(|t| t.raw[4]),
            t.map(|t| t.flags),
            boxes
        );
    }
    Ok(())
}
