//! Prints the COL answer along a line: `col_probe DB X0 Z0 X1 Z1 Y STEPS`.
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let a: Vec<i32> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let (db, x0, z0, x1, z1, y, n) = (a[0] as usize, a[1], a[2], a[3], a[4], a[5], a[6].max(1));
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let col = rugrats_data::col::Col::parse(&disc.file(466 + db)?)?;
    for k in 0..=n {
        let (x, z) = (x0 + (x1 - x0) * k / n, z0 + (z1 - z0) * k / n);
        let q = rugrats::collide::query(&col, [x, y, z]);
        println!("({x}, {z}): kind {} floor {} ceiling {} flags {}", q.kind, q.floor, q.ceiling, rugrats::collide::floor_flags(&col, &q));
    }
    Ok(())
}
