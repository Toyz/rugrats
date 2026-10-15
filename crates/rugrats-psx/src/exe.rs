//! The PS-X EXE format: a 2048-byte header, then the text image loaded at
//! `t_addr`.

use std::fmt;

pub const HEADER_SIZE: usize = 0x800;

#[derive(Clone, Debug)]
pub struct Exe {
    /// Entry point.
    pub pc0: u32,
    /// Initial $gp.
    pub gp0: u32,
    /// Load address of the image that follows the header.
    pub t_addr: u32,
    pub t_size: u32,
    pub d_addr: u32,
    pub d_size: u32,
    /// The region the BIOS clears to zero before the jump.
    pub b_addr: u32,
    pub b_size: u32,
    /// Initial $sp base and offset ($sp = s_addr + s_size when s_addr is set).
    pub s_addr: u32,
    pub s_size: u32,
    /// The marker string at 0x4c, e.g. "Sony Computer Entertainment Inc. for North America area".
    pub marker: String,
    /// The image, `t_size` bytes.
    pub image: Vec<u8>,
}

#[derive(Debug)]
pub struct ExeError(pub String);

impl fmt::Display for ExeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PS-X EXE: {}", self.0)
    }
}

impl std::error::Error for ExeError {}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

impl Exe {
    pub fn parse(bytes: &[u8]) -> Result<Exe, ExeError> {
        if bytes.len() < HEADER_SIZE || &bytes[..8] != b"PS-X EXE" {
            return Err(ExeError("no PS-X EXE magic".into()));
        }
        let t_size = le32(bytes, 0x1c);
        let image = bytes
            .get(HEADER_SIZE..HEADER_SIZE + t_size as usize)
            .ok_or_else(|| ExeError(format!("t_size {t_size:#x} runs past the file ({:#x})", bytes.len())))?
            .to_vec();
        let marker_end = bytes[0x4c..HEADER_SIZE].iter().position(|&b| b == 0).unwrap_or(HEADER_SIZE - 0x4c);
        Ok(Exe {
            pc0: le32(bytes, 0x10),
            gp0: le32(bytes, 0x14),
            t_addr: le32(bytes, 0x18),
            t_size,
            d_addr: le32(bytes, 0x20),
            d_size: le32(bytes, 0x24),
            b_addr: le32(bytes, 0x28),
            b_size: le32(bytes, 0x2c),
            s_addr: le32(bytes, 0x30),
            s_size: le32(bytes, 0x34),
            marker: String::from_utf8_lossy(&bytes[0x4c..0x4c + marker_end]).to_string(),
            image,
        })
    }

    /// The image as a memory view starting at `t_addr`.
    pub fn view(&self) -> crate::Memory<'_> {
        crate::Memory { base: self.t_addr, bytes: &self.image }
    }
}
