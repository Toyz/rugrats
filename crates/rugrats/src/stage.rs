//! What a 3D screen hands the renderer each frame: VRAM and the fixed
//! scene (rebuilt only when `generation` changes), the triangles that move,
//! the 2D shapes under the world and over it, the camera and the colour
//! the screen clears to.

use rrt::glam::Mat4;
use rugrats_render::{Vram, Vtx};

#[derive(Clone)]
pub struct Stage {
    /// Bumped whenever `vram` or `fixed` change.
    pub generation: u64,
    pub vram: Vram,
    pub fixed: Vec<Vtx>,
    pub moving: Vec<Vtx>,
    /// In screen pixels, drawn in order under the world.
    pub under: Vec<Vtx>,
    /// In screen pixels, drawn over the world.
    pub overlay: Vec<Vtx>,
    pub camera: Mat4,
    pub clear: [u8; 3],
}

impl Stage {
    pub fn new(vram: Vram, fixed: Vec<Vtx>) -> Stage {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Stage {
            generation: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            vram,
            fixed,
            moving: Vec::new(),
            under: Vec::new(),
            overlay: Vec::new(),
            camera: Mat4::IDENTITY,
            clear: [0, 0, 0],
        }
    }
}
