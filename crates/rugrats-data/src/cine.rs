//! In-engine cinematics: the stream of frame records in `/DATA/CINEMAT`
//! (OPEN.BIN), one data sector a frame, as `play_cine`'s readers take it
//! (SLUS_006.50:0x80073810 the header, 0x80073b08 the camera, 0x80073c34 an
//! actor, 0x80073f08 a pose).
//!
//! ```text
//! first sector  u32 0xf00dbabe, u16 version (3), u16 0xdead,
//!               u16 frames, u16 setup bytes, setup
//! a frame (data sector; 0xdddddddd first ends the stream)
//!   u16 frame, u16 unknown_02
//!   s32 camera position[3]
//!   u32 a: bits 0-12 q0, 13-25 q1 (signed), 26-31 actors
//!   u32 b: bits 0-12 q2, 13-25 q3 (signed), 26-31 poses
//!   actor[actors], 24 bytes
//!     u32 bits 0-12 q0, 13-25 q1, 26-29 mouth, 30-31 eyes
//!     u32 bits 0-12 q2, 13-25 q3
//!     s32 position[3]
//!     s16 id, u16 unknown_16
//!   pose[poses], 16 bytes and 8 a key
//!     s32 position[3]
//!     u32 bits 0-15 id, 16-23 keys, 24-27 mouth, 28-29 eyes
//!     key[keys]: u32 bits 0-12 q0, 13-25 q1, 26-31 channel;
//!                u32 bits 0-12 q2, 13-25 q3
//! ```
//!
//! Quaternions are (x, y, z, w) = (q0, q1, q2, q3) at any scale. Ids are
//! the object registry's (placed by the database's LP file). A mouth is a
//! frame of the face's mouth block and eyes one more than a frame of its
//! eye block, 0 for neither.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub id: i16,
    pub q: [i16; 4],
    pub pos: [i32; 3],
    pub mouth: u8,
    pub eyes: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    pub id: i16,
    pub pos: [i32; 3],
    pub mouth: u8,
    pub eyes: u8,
    /// (channel, quaternion)
    pub keys: Vec<(u8, [i16; 4])>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub number: u16,
    pub camera_pos: [i32; 3],
    pub camera_q: [i16; 4],
    pub actors: Vec<Placed>,
    pub poses: Vec<Pose>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub frames: u16,
    pub setup: u16,
}

fn sext(v: u32, bits: u32) -> i16 {
    (((v << (32 - bits)) as i32) >> (32 - bits)) as i16
}

fn w(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

fn quat(a: u32, b: u32) -> [i16; 4] {
    [sext(a, 13), sext(a >> 13, 13), sext(b, 13), sext(b >> 13, 13)]
}

/// The stream's header, from its first sector's data.
pub fn header(d: &[u8]) -> Option<Header> {
    let magic = w(d, 0)?;
    if magic != 0xf00d_babe && magic != 0xbabe_d00d {
        return None;
    }
    let h = |at: usize| d.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    (h(6)? == 0xdead).then(|| Some(Header { version: h(4)?, frames: h(8)?, setup: h(10)? })).flatten()
}

/// One frame from a data sector's 2048 bytes; None for the end marker or a
/// record that runs off the sector.
pub fn frame(d: &[u8]) -> Option<Frame> {
    if w(d, 0)? == 0xdddd_dddd {
        return None;
    }
    let number = u16::from_le_bytes([d[0], d[1]]);
    let camera_pos = [w(d, 4)? as i32, w(d, 8)? as i32, w(d, 12)? as i32];
    let (a, b) = (w(d, 0x10)?, w(d, 0x14)?);
    let mut p = 24;
    let actors = (0..a >> 26)
        .map(|_| {
            let (x, y) = (w(d, p)?, w(d, p + 4)?);
            let pos = [w(d, p + 8)? as i32, w(d, p + 12)? as i32, w(d, p + 16)? as i32];
            let id = w(d, p + 20)? as i16;
            p += 24;
            Some(Placed { id, q: quat(x, y), pos, mouth: (x >> 26 & 15) as u8, eyes: (x >> 30) as u8 })
        })
        .collect::<Option<Vec<_>>>()?;
    let poses = (0..b >> 26)
        .map(|_| {
            let pos = [w(d, p)? as i32, w(d, p + 4)? as i32, w(d, p + 8)? as i32];
            let h = w(d, p + 12)?;
            let n = (h >> 16) & 0xff;
            p += 16;
            let keys = (0..n)
                .map(|_| {
                    let (x, y) = (w(d, p)?, w(d, p + 4)?);
                    p += 8;
                    Some(((x >> 26) as u8, quat(x, y)))
                })
                .collect::<Option<Vec<_>>>()?;
            Some(Pose { id: h as u16 as i16, pos, mouth: (h >> 24 & 15) as u8, eyes: (h >> 28 & 3) as u8, keys })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Frame { number, camera_pos, camera_q: quat(a, b), actors, poses })
}
