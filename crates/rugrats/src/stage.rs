//! What a 3D screen hands the renderer each frame: VRAM and the fixed
//! scene (rebuilt only when `generation` changes), the triangles that move,
//! the 2D shapes under the world and over it, the camera and the colour
//! the screen clears to.

use rrt::glam::{Mat3, Mat4, Vec3};
use rugrats_render::{Vram, Vtx};

#[derive(Clone)]
pub struct Stage {
    /// Bumped whenever `vram` or `fixed` change.
    pub generation: u64,
    pub vram: Vram,
    /// Bumped when `vram` changes in place (a LoadImage mid-scene).
    pub vram_rev: u64,
    pub fixed: Vec<Vtx>,
    pub moving: Vec<Vtx>,
    /// The semi-transparent triangles of `moving`, blended over it.
    pub semi: Vec<Vtx>,
    /// In screen pixels, drawn in order under the world.
    pub under: Vec<Vtx>,
    /// In screen pixels, drawn over the world.
    pub overlay: Vec<Vtx>,
    pub camera: Mat4,
    pub clear: [u8; 3],
}

impl Stage {
    /// Replaces the moving triangles, picking out the semi-transparent.
    pub fn set_moving(&mut self, tris: Vec<Vtx>) {
        self.semi = tris.as_chunks::<3>().0.iter().filter(|t| t[0].mode & Vtx::SEMI != 0).flatten().copied().collect();
        self.moving = tris;
    }

    /// The semi-transparent triangles of `overlay`, blended over it.
    pub fn overlay_semi(&self) -> Vec<Vtx> {
        self.overlay.as_chunks::<3>().0.iter().filter(|t| t[0].mode & Vtx::SEMI != 0).flatten().copied().collect()
    }

    pub fn new(vram: Vram, fixed: Vec<Vtx>) -> Stage {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Stage {
            generation: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            vram,
            vram_rev: 0,
            fixed,
            moving: Vec::new(),
            semi: Vec::new(),
            under: Vec::new(),
            overlay: Vec::new(),
            camera: Mat4::IDENTITY,
            clear: [0, 0, 0],
        }
    }
}

/// The view and projection the original's cameras give: the camera matrix
/// `rows` (world to camera), its x row scaled by 512 / 320 for the screen
/// (0x8001e0f4, the screen size at 0x800c4a14), a point seen at
/// rows (p - pos) (0x8001eb4c), projected by the GTE at distance `h` onto
/// the 512 x 240 screen with no offset.
pub fn view(pos: Vec3, rows: Mat3, h: f32) -> Mat4 {
    let view =
        Mat4::from_mat3(Mat3::from_diagonal(Vec3::new(512.0 / 320.0, 1.0, 1.0)) * rows) * Mat4::from_translation(-pos);
    let (near, far) = (16.0f32, 200_000.0f32);
    let (a, b) = (far / (far - near), -near * far / (far - near));
    #[rustfmt::skip]
    let proj = Mat4::from_cols_array(&[
        h / 256.0, 0.0, 0.0, 0.0,
        0.0, -h / 120.0, 0.0, 0.0,
        0.0, 0.0, a, 1.0,
        0.0, 0.0, b, 0.0,
    ]);
    proj * view
}
