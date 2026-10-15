//! How far the opening's camera and posed characters move per stream
//! frame, to tell the stream's intended frame rate.

use rugrats::disc::GameDisc;

fn main() -> Result<(), String> {
    let disc = GameDisc::open(&rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?)?;
    let s = rugrats::media::cine::Stream::read(&disc, 0)?;
    println!("frames {}, sound {:.1} s at 44100", s.frames.len(), s.sound.len() as f64 / 44100.0);
    let d = |a: [i32; 3], b: [i32; 3]| {
        (((a[0] - b[0]) as f64).powi(2) + ((a[1] - b[1]) as f64).powi(2) + ((a[2] - b[2]) as f64).powi(2)).sqrt()
    };
    for k in [100usize, 200, 400, 600, 800, 1000, 1200] {
        let (f0, f1) = (&s.frames[k].1, &s.frames[k + 1].1);
        let cam = d(f0.camera_pos, f1.camera_pos);
        let poses: Vec<String> = f0
            .poses
            .iter()
            .filter_map(|p| f1.poses.iter().find(|q| q.id == p.id).map(|q| format!("{}:{:.0}", p.id, d(p.pos, q.pos))))
            .collect();
        println!("frame {k}: camera {cam:.0}, poses {}", poses.join(" "));
    }
    Ok(())
}
