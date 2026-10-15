//! Sony SEP sequence packs (libsnd's `SsSepOpen`, SLUS_006.50:0x800a5ec8):
//! several SEQ songs back to back.
//!
//! ```text
//! char[4]  "pQES"
//! u16      version (0)
//! song, repeated
//!   u16    id                 big-endian, as every field here
//!   u16    resolution         ticks a quarter note
//!   u24    tempo              microseconds a quarter note
//!   u16    rhythm             time signature (numerator, log2 denominator)
//!   u32    length             bytes of the events that follow
//!   events                    standard MIDI track events with running
//!                             status, delta times first; FF 2F 00 ends
//! ```

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub id: u16,
    pub resolution: u16,
    pub tempo: u32,
    /// The MIDI events, from the first delta time.
    pub events: Vec<u8>,
}

/// The songs of a SEP file.
pub fn parse(b: &[u8]) -> Result<Vec<Song>, String> {
    if b.get(..4) != Some(b"pQES") {
        return Err("no pQES magic".into());
    }
    let be = |at: usize, n: usize| -> Result<u32, String> {
        let s = b.get(at..at + n).ok_or_else(|| format!("SEP: short at {at:#x}"))?;
        Ok(s.iter().fold(0, |v, &x| v << 8 | x as u32))
    };
    let mut songs = Vec::new();
    let mut p = 6;
    while p + 13 <= b.len() {
        let len = be(p + 9, 4)? as usize;
        let events = b.get(p + 13..p + 13 + len).ok_or_else(|| format!("SEP: song at {p:#x} runs past the file"))?.to_vec();
        songs.push(Song { id: be(p, 2)? as u16, resolution: be(p + 2, 2)? as u16, tempo: be(p + 4, 3)?, events });
        p += 13 + len;
    }
    Ok(songs)
}
