//! Lists database DB's models with a textured face on page PAGE and CLUT
//! CLUT (hex), to find what the original drew with them.
//!
//! cargo run -p rugrats --example find_texture -- DB PAGE CLUT

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let mut a = std::env::args().skip(1);
    let db: usize = a.next().and_then(|s| s.parse().ok()).ok_or("DB")?;
    let hex = |s: Option<String>| s.and_then(|s| u16::from_str_radix(&s, 16).ok()).ok_or("PAGE CLUT");
    let (page, clut) = (hex(a.next())?, hex(a.next())?);
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let models = rugrats_data::model::Models::parse(&disc.file(242 + db)?).map_err(|e| e.to_string())?;
    for (id, m) in models.iter() {
        let hits = m
            .faces
            .iter()
            .filter(|f| f.texture.is_some_and(|(_, c, p)| p & 0x1ff == page & 0x1ff && c == clut))
            .count();
        if hits > 0 {
            println!("model {id}: {hits} of {} faces", m.faces.len());
        }
    }
    Ok(())
}
