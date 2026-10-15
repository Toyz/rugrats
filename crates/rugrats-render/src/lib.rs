//! The renderer the port and the viewer share: the game's rooms and models
//! from the disc's data, drawn with wgpu, textured through a copy of VRAM
//! with the PlayStation's texture pages and CLUTs. The VRAM sampling and the
//! pipeline are hwtr's.

#![forbid(unsafe_code)]

pub mod mesh;
pub mod renderer;
pub mod rooms;

pub use mesh::{Vram, Vtx};
pub use renderer::Renderer;
