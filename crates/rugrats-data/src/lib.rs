//! Readers for Rugrats: Search for Reptar's data files.

#![forbid(unsafe_code)]

pub mod files;
pub mod mdec;
pub mod tim;
pub mod vab;

pub use tim::Tim;
