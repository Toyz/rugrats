//! What the PlayStation side of the reverse engineering needs to know: the
//! executable format and the R3000A instruction set with the GTE's commands.

#![forbid(unsafe_code)]

pub mod analysis;
pub mod decomp;
pub mod exe;
pub mod mips;
pub mod program;

pub use exe::Exe;

/// A byte image mapped at a base address.
#[derive(Clone, Copy)]
pub struct Memory<'a> {
    pub base: u32,
    pub bytes: &'a [u8],
}

impl<'a> Memory<'a> {
    pub fn end(&self) -> u32 {
        self.base + self.bytes.len() as u32
    }

    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.end()
    }

    pub fn slice(&self, addr: u32, len: usize) -> Option<&'a [u8]> {
        let at = addr.checked_sub(self.base)? as usize;
        self.bytes.get(at..at + len)
    }

    pub fn u32(&self, addr: u32) -> Option<u32> {
        self.slice(addr, 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
    }

    pub fn u16(&self, addr: u32) -> Option<u16> {
        self.slice(addr, 2).map(|b| u16::from_le_bytes(b.try_into().unwrap()))
    }

    pub fn u8(&self, addr: u32) -> Option<u8> {
        self.slice(addr, 1).map(|b| b[0])
    }

    /// A NUL-terminated string at `addr`, if it is printable ASCII.
    pub fn cstr(&self, addr: u32) -> Option<&'a str> {
        let at = addr.checked_sub(self.base)? as usize;
        let rest = self.bytes.get(at..)?;
        let end = rest.iter().position(|&b| b == 0)?;
        let s = &rest[..end];
        if s.iter().all(|&b| b == b'\n' || b == b'\t' || (0x20..0x7f).contains(&b)) {
            std::str::from_utf8(s).ok()
        } else {
            None
        }
    }
}
