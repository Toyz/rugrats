//! How the opening's XA sectors (file 1, channel 1) hang together: for
//! strides 1-8, decodes every stride-th sector as one stream and measures
//! the jump at each sector boundary against the steps inside sectors.
//! A real stream joins smoothly.

use rrt::kit::adpcm::xa;
use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let start = disc.file_lba(0)?;
    let mut sectors = Vec::new();
    for lba in start..start + 7060 {
        let s = disc.sector(lba)?;
        let r = &s.raw;
        if r[18] & 0x64 == 0x64 && (r[16], r[17]) == (1, 1) {
            sectors.push((lba - start, r[19], r[24..24 + xa::SECTOR_AUDIO].to_vec()));
        }
    }
    println!("{} sectors; first at {:?}", sectors.len(), sectors.iter().take(12).map(|s| s.0).collect::<Vec<_>>());
    for stride in 1..=8usize {
        let mut best = f64::MAX;
        for phase in 0..stride {
            let mut dec = xa::Decoder::new();
            let (mut inner, mut n_inner, mut edge, mut n_edge) = (0f64, 0u64, 0f64, 0u64);
            let mut last: Option<i16> = None;
            for (_, coding, data) in sectors.iter().skip(phase).step_by(stride).take(300) {
                let mut out = Vec::new();
                dec.decode(xa::Coding(*coding), data, &mut out);
                if let (Some(l), Some(f)) = (last, out.first()) {
                    edge += (f.0 as f64 - l as f64).abs();
                    n_edge += 1;
                }
                for w in out.windows(2) {
                    inner += (w[1].0 as f64 - w[0].0 as f64).abs();
                    n_inner += 1;
                }
                last = out.last().map(|s| s.0);
            }
            let ratio = (edge / n_edge.max(1) as f64) / (inner / n_inner.max(1) as f64).max(1.0);
            best = best.min(ratio);
            println!("stride {stride} phase {phase}: edge/inner {ratio:.2}");
        }
        println!("stride {stride} best {best:.2}");
    }
    Ok(())
}
