//! `.STR` movies: video frames in real-time data sectors, each frame split
//! over several sectors behind a 32-byte header, XA audio interleaved.
//!
//! ```text
//! a video sector's user data (2048 bytes, Form 1)
//!   u16  0x0160
//!   u16  0x8001
//!   u16  chunk      this sector's place in the frame
//!   u16  chunks     sectors in the frame
//!   u32  frame      from 1
//!   u32  size       the frame's bytes (its chunks are padded past them)
//!   u16  width
//!   u16  height
//!   u8   [12]       the frame's BS header, repeated in every chunk
//!   u8   [2016]     the frame's bytes
//! ```

/// One video sector, read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub chunk: u16,
    pub chunks: u16,
    pub frame: u32,
    pub width: u16,
    pub height: u16,
    pub data: &'a [u8],
}

/// The sector's video chunk, if it is one: `raw` is the whole 2352-byte sector.
pub fn chunk(raw: &[u8]) -> Option<Chunk<'_>> {
    let submode = *raw.get(18)?;
    // Data or video, real time, Form 1.
    if submode & 0x20 != 0 || submode & 0x0a == 0 {
        return None;
    }
    let u = raw.get(24..24 + 2048)?;
    let h = |k: usize| u16::from_le_bytes([u[2 * k], u[2 * k + 1]]);
    if h(0) != 0x0160 || h(1) != 0x8001 {
        return None;
    }
    Some(Chunk {
        chunk: h(2),
        chunks: h(3),
        frame: u32::from_le_bytes(u[8..12].try_into().unwrap()),
        width: h(8),
        height: h(9),
        data: &u[32..],
    })
}

/// Frames put together from their chunks, in order.
#[derive(Default)]
pub struct Frames {
    frame: u32,
    parts: Vec<Option<Vec<u8>>>,
}

/// A whole frame: its number, size and BS bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub number: u32,
    pub width: u16,
    pub height: u16,
    pub bs: Vec<u8>,
}

impl Frames {
    /// Takes a chunk; returns the frame it completes.
    pub fn push(&mut self, c: Chunk) -> Option<Frame> {
        if c.frame != self.frame || self.parts.len() != c.chunks as usize {
            self.frame = c.frame;
            self.parts = vec![None; c.chunks as usize];
        }
        *self.parts.get_mut(c.chunk as usize)? = Some(c.data.to_vec());
        if self.parts.iter().any(Option::is_none) {
            return None;
        }
        let bs = self.parts.drain(..).flatten().flatten().collect();
        Some(Frame { number: c.frame, width: c.width, height: c.height, bs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THQ_LOGO.STR's frames decode to the streams libpress handed the MDEC
    /// in the original (oracle dumps 1 to 101, after the legal screen's 0).
    #[test]
    fn thq_logo_frames_decode_to_what_libpress_hands_the_mdec() {
        let work = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work");
        let Ok(image) = rrt_disc_free_read(&work) else {
            eprintln!("skipped: no disc or dumps");
            return;
        };
        let mut frames = Frames::default();
        let mut checked = 0;
        for raw in image.chunks(2352) {
            let Some(c) = chunk(raw) else { continue };
            let Some(f) = frames.push(c) else { continue };
            let Ok(dump) = std::fs::read(work.join(format!("oracle/mdec/{:05}.mdec", f.number))) else { break };
            let ours = crate::bs::rle(&f.bs).unwrap();
            let theirs: Vec<u16> = dump[4..].chunks(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
            let first = ours.iter().zip(&theirs).position(|(a, b)| a != b);
            assert_eq!((ours.len(), first), (theirs.len(), None), "frame {}", f.number);
            checked += 1;
            if f.number == 101 {
                break;
            }
        }
        assert!(checked > 0);
        eprintln!("{checked} frames match");
    }

    /// THQ_LOGO.STR's sectors, read from the raw image (LBA 170320, 1105 sectors).
    fn rrt_disc_free_read(work: &std::path::Path) -> std::io::Result<Vec<u8>> {
        use std::io::{Read, Seek, SeekFrom};
        let bin = std::fs::read_dir(work.join("disc"))?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "bin"))
            .ok_or(std::io::ErrorKind::NotFound)?;
        std::fs::metadata(work.join("oracle/mdec/00001.mdec"))?;
        let mut f = std::fs::File::open(bin)?;
        f.seek(SeekFrom::Start(170320 * 2352))?;
        let mut out = vec![0; 1105 * 2352];
        f.read_exact(&mut out)?;
        Ok(out)
    }
}
