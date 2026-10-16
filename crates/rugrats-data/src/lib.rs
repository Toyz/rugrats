//! Readers for Rugrats: Search for Reptar's data files.

#![forbid(unsafe_code)]

pub mod anim;
pub mod bs;
pub mod cine;
pub mod col;
pub mod files;
pub mod hierarchy;
pub mod level;
pub mod mdec;
pub mod model;
pub mod movie;
pub mod placement;
pub mod sep;
pub mod tim;
pub mod vab;

pub use tim::Tim;
