//! `DBnnMOD.BIN`: a database's models, as the loader (SLUS_006.50:0x80044d00)
//! and the hand-written model renderer (0x80010000) read them.
//!
//! ```text
//! u32  0xc0de0001
//! u16  count, u16 pad
//! u32  at[count]          low 21 bits the model's offset from the table's
//!                         end, high 11 its id (ascending); 0xffffffff for
//!                         none
//! model
//!   u32  link
//!   u32  unknown_04       (the loader writes the index here)
//!   u16  radius           the culling test's (0x80031834)
//!   u16  unknown_0a
//!   u32  bounds_at        16 bytes: s16 centre[3], pad, s16 extent[3]?, pad
//!   u16  vertices
//!   u16  colours
//!   u32  vertices_at      s16 x, y, z, w each
//!   u32  colours_at       r, g, b, pad each (when colours is not 0)
//!   u32  faces_at
//! faces: sections u16 kind, u16 count, records; the kinds are the GPU's
//!   0x20 F3  0x28 F4    u32 flags, u8 v[4], u8 r g b code         12 bytes
//!   0x30 G3  0x38 G4    u32 flags, u8 v[4], u8 colour index[4]    12 bytes
//!   0x24 FT3 0x2c FT4   u32 flags, u8 v[4], u8 r g b pad,
//!   0x34 GT3 0x3c GT4     u8 u0 v0, u16 clut, u8 u1 v1, u16 tpage,
//!                         u8 u2 v2 u3 v3                           24 bytes
//!   0xff the end
//! ```

use crate::level::LevelError as Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shading {
    Flat,
    Gouraud,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face {
    pub v: [u8; 4],
    pub quad: bool,
    pub shading: Shading,
    /// The corners' colours, resolved through the palette for gouraud faces.
    pub rgb: [[u8; 3]; 4],
    /// For a textured face: its corners' texels, CLUT and texture page.
    pub texture: Option<([[u8; 2]; 4], u16, u16)>,
    pub flags: u32,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub radius: u16,
    pub vertices: Vec<[i16; 4]>,
    pub palette: Vec<[u8; 3]>,
    pub faces: Vec<Face>,
}

fn u16_at(b: &[u8], at: usize) -> Result<u16, Error> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or_else(|| Error(format!("model: short at {at:#x}")))
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, Error> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap())).ok_or_else(|| Error(format!("model: short at {at:#x}")))
}

impl Model {
    /// One model, `b` from its start (to the end of the file or further).
    pub fn parse(b: &[u8]) -> Result<Model, Error> {
        let radius = u16_at(b, 8)?;
        let (nv, nc) = (u16_at(b, 0x10)? as usize, u16_at(b, 0x12)? as usize);
        let (va, ca, fa) = (u32_at(b, 0x14)? as usize, u32_at(b, 0x18)? as usize, u32_at(b, 0x1c)? as usize);
        let vertices = (0..nv)
            .map(|k| {
                let at = va + 8 * k;
                Ok([u16_at(b, at)? as i16, u16_at(b, at + 2)? as i16, u16_at(b, at + 4)? as i16, u16_at(b, at + 6)? as i16])
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let palette = (0..nc)
            .map(|k| b.get(ca + 4 * k..ca + 4 * k + 3).map(|c| [c[0], c[1], c[2]]))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error("model: palette runs past the model".into()))?;
        let mut faces = Vec::new();
        let mut p = fa;
        loop {
            let (kind, n) = (u16_at(b, p)?, u16_at(b, p + 2)? as usize);
            p += 4;
            if kind == 0xff {
                break;
            }
            let (quad, shading, textured) = match kind {
                0x20 => (false, Shading::Flat, false),
                0x28 => (true, Shading::Flat, false),
                0x30 => (false, Shading::Gouraud, false),
                0x38 => (true, Shading::Gouraud, false),
                0x24 => (false, Shading::Flat, true),
                0x2c => (true, Shading::Flat, true),
                0x34 => (false, Shading::Gouraud, true),
                0x3c => (true, Shading::Gouraud, true),
                k => return Err(Error(format!("model: face kind {k:#x} at {p:#x}"))),
            };
            let size = if textured { 24 } else { 12 };
            for _ in 0..n {
                let r = b.get(p..p + size).ok_or_else(|| Error("model: faces run past the model".into()))?;
                let v = [r[4], r[5], r[6], if quad { r[7] } else { r[6] }];
                if v.iter().any(|&i| i as usize >= nv) {
                    return Err(Error(format!("model: a face names vertex {:?} of {nv}", v)));
                }
                let colour = |i: u8| palette.get(i as usize).copied().unwrap_or([128, 128, 128]);
                let rgb = match shading {
                    Shading::Flat => [[r[8], r[9], r[10]]; 4],
                    Shading::Gouraud => [colour(r[8]), colour(r[9]), colour(r[10]), colour(if quad { r[11] } else { r[10] })],
                };
                let texture = textured.then(|| {
                    (
                        [[r[12], r[13]], [r[16], r[17]], [r[20], r[21]], [r[22], r[23]]],
                        u16::from_le_bytes([r[14], r[15]]),
                        u16::from_le_bytes([r[18], r[19]]),
                    )
                });
                faces.push(Face { v, quad, shading, rgb, texture, flags: u32::from_le_bytes(r[..4].try_into().unwrap()) });
                p += size;
            }
        }
        Ok(Model { radius, vertices, palette, faces })
    }
}

/// A whole MOD file: the models by id (None where there is none), as the
/// game's lookup (0x80044b74) finds them, by binary search on the table
/// entries' high 11 bits.
pub fn parse_file(b: &[u8]) -> Result<Vec<Option<Model>>, Error> {
    if u32_at(b, 0)? != 0xc0de_0001 {
        return Err(Error("MOD: no 0xc0de0001".into()));
    }
    let n = u16_at(b, 4)? as usize;
    let base = 8 + 4 * n;
    let mut models = Vec::new();
    for k in 0..n {
        let at = u32_at(b, 8 + 4 * k)?;
        if at == u32::MAX {
            continue;
        }
        let id = (at >> 21) as usize;
        let start = base + (at & 0x1f_ffff) as usize;
        let body = b.get(start..).ok_or_else(|| Error(format!("MOD: model {id} past the end")))?;
        let m = Model::parse(body).map_err(|e| Error(format!("MOD model {id}: {}", e.0)))?;
        if models.len() <= id {
            models.resize(id + 1, None);
        }
        models[id] = Some(m);
    }
    Ok(models)
}

#[cfg(test)]
mod tests {
    /// Every model of every MOD file but DB08's parses, its faces naming
    /// its own vertices.
    #[test]
    fn every_model_parses() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/fs/DATA");
        let Ok(dbs) = std::fs::read_dir(&dir) else {
            eprintln!("skipped: no work/fs");
            return;
        };
        let mut models = 0;
        for db in dbs.flatten() {
            let name = db.file_name().to_string_lossy().to_string();
            let Some(n) = name.strip_prefix("DB") else { continue };
            if n == "08" {
                continue;
            }
            let Ok(bytes) = std::fs::read(db.path().join(format!("DB{n}MOD.BIN"))) else { continue };
            let all = super::parse_file(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            models += all.iter().flatten().count();
        }
        assert_eq!(models, 6286);
    }
}
