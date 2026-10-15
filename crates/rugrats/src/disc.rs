//! The player's disc: its files by the game's own numbering, and the few
//! tables the port takes from the executable on it.

use std::path::Path;

use rrt::disc::Image;
use rugrats_data::files::{self, Entry};
use rugrats_data::mdec::Tables;

/// Where the executable keeps libpress's MDEC uploads: the quant command
/// (0x40000001, luma then chroma) and the scale command (0x60000000).
const MDEC_QUANT: u32 = 0x800c_1f20;
const MDEC_SCALE: u32 = 0x800c_1fa4;

pub struct GameDisc {
    image: Image,
    table: Vec<Entry>,
    /// The movies `movie_play` knows, by number.
    pub movies: Vec<crate::movie::Entry>,
    /// The tables libpress gives the MDEC before decoding a picture.
    pub mdec: Tables,
    /// What the memory card code takes from the executable.
    pub card: crate::card::Art,
}

impl GameDisc {
    pub fn open(cue: &Path) -> Result<GameDisc, String> {
        let image = Image::open(cue).map_err(|e| format!("{}: {e}", cue.display()))?;
        let exe = image.iso().and_then(|iso| iso.read_path("/SLUS_006.50")).map_err(|e| e.to_string())?;
        let exe = exe.get(0x800..).ok_or("SLUS_006.50 is too short")?;
        const BASE: u32 = 0x8001_0000;
        let at = |addr: u32, len: usize| exe.get((addr - BASE) as usize..(addr - BASE) as usize + len);
        let table = files::table(exe, BASE).ok_or("no file table in SLUS_006.50")?;
        let quant = at(MDEC_QUANT, 4 + 128).ok_or("no MDEC quant table")?;
        let scale = at(MDEC_SCALE, 4 + 128).ok_or("no MDEC scale table")?;
        if quant[..4] != 0x4000_0001u32.to_le_bytes() || scale[..4] != 0x6000_0000u32.to_le_bytes() {
            return Err("SLUS_006.50 is not the US release this port reads".into());
        }
        let mut mdec = Tables::default();
        mdec.luma.copy_from_slice(&quant[4..68]);
        mdec.chroma.copy_from_slice(&quant[68..132]);
        for (k, s) in mdec.scale.iter_mut().enumerate() {
            *s = i16::from_le_bytes([scale[4 + 2 * k], scale[5 + 2 * k]]);
        }
        let movies = at(crate::movie::TABLE, 8 * crate::movie::COUNT)
            .ok_or("no movie table")?
            .chunks(8)
            .map(|e| crate::movie::Entry {
                frames: u32::from_le_bytes(e[..4].try_into().unwrap()),
                lba: u32::from_le_bytes(e[4..].try_into().unwrap()),
            })
            .collect();
        let card = crate::card::Art::from_exe(exe).ok_or("no memory card tables in SLUS_006.50")?;
        Ok(GameDisc { image, table, movies, mdec, card })
    }

    /// The first sector of file `index` (the streams are read by sector).
    pub fn file_lba(&self, index: usize) -> Result<u32, String> {
        self.table.get(index).filter(|e| e.is_used()).map(|e| e.lba).ok_or(format!("File '{index}' not found!"))
    }

    /// One raw sector, for the streams.
    pub fn sector(&self, lba: u32) -> Result<rrt::disc::Sector, String> {
        self.image.sector(lba).map_err(|e| e.to_string())
    }

    /// File `index` of the game's table, whole.
    pub fn file(&self, index: usize) -> Result<Vec<u8>, String> {
        let e = self.table.get(index).filter(|e| e.is_used()).ok_or(format!("File '{index}' not found!"))?;
        self.image.read_blocks(e.lba, e.size).map_err(|e| e.to_string())
    }
}
