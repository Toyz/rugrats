//! Sony VAB sound banks (`.VH` header, `.VB` body) and PS-ADPCM samples.
//!
//! ```text
//! VH
//!   char[4]  "pBAV"
//!   u32      version
//!   u32      id
//!   u32      size              VH + VB bytes
//!   u16      reserved
//!   u16      programs
//!   u16      tones             total over all programs
//!   u16      vags              samples in the VB
//!   u8       master volume, master pan, attr1, attr2
//!   u32      reserved
//!   program[128]   16 bytes each
//!   tone[16 * programs]  32 bytes each
//!   u16      vag_size[256]     entry 0 unused; entry n = bytes of sample n / 8
//! VB
//!   the samples back to back, 16-byte ADPCM blocks
//! ```

#[derive(Clone, Debug)]
pub struct Program {
    pub tones: u8,
    pub volume: u8,
    pub priority: u8,
    pub mode: u8,
    pub pan: u8,
    pub attr: u16,
}

#[derive(Clone, Debug)]
pub struct Tone {
    pub priority: u8,
    pub mode: u8,
    pub volume: u8,
    pub pan: u8,
    /// The note at which the sample plays at its recorded rate.
    pub center: u8,
    /// Fine tuning of `center`, in 1/128 semitones.
    pub shift: u8,
    pub min_note: u8,
    pub max_note: u8,
    pub vibrato_width: u8,
    pub vibrato_time: u8,
    pub portamento_width: u8,
    pub portamento_time: u8,
    pub pitch_bend_min: u8,
    pub pitch_bend_max: u8,
    pub adsr1: u16,
    pub adsr2: u16,
    pub program: u16,
    /// 1-based sample number in the VB.
    pub vag: u16,
}

#[derive(Clone, Debug)]
pub struct Vab {
    pub id: u32,
    pub version: u32,
    pub master_volume: u8,
    pub master_pan: u8,
    /// The programs in use, by program number.
    pub programs: Vec<(usize, Program)>,
    /// Tones per program, in program order.
    pub tones: Vec<Tone>,
    /// Byte size of each sample in the VB, 1-based numbering at index 0.
    pub vag_sizes: Vec<u32>,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

impl Vab {
    pub fn parse(vh: &[u8]) -> Result<Vab, String> {
        if vh.len() < 32 || &vh[..4] != b"pBAV" {
            return Err("no pBAV magic".into());
        }
        let programs_used = u16_at(vh, 18) as usize;
        let tone_count = u16_at(vh, 20) as usize;
        let vags = u16_at(vh, 22) as usize;
        let tones_at = 32 + 128 * 16;
        let sizes_at = tones_at + programs_used * 16 * 32;
        if vh.len() < sizes_at + 512 {
            return Err(format!("VH is {} bytes, needs {}", vh.len(), sizes_at + 512));
        }
        let mut programs = Vec::new();
        for p in 0..128 {
            let at = 32 + p * 16;
            let r = &vh[at..at + 16];
            if r[0] == 0 {
                continue;
            }
            programs.push((
                p,
                Program { tones: r[0], volume: r[1], priority: r[2], mode: r[3], pan: r[4], attr: u16_at(r, 6) },
            ));
        }
        let mut tones = Vec::new();
        for (slot, (_, prog)) in programs.iter().enumerate() {
            for t in 0..prog.tones as usize {
                let at = tones_at + slot * 16 * 32 + t * 32;
                let r = &vh[at..at + 32];
                tones.push(Tone {
                    priority: r[0],
                    mode: r[1],
                    volume: r[2],
                    pan: r[3],
                    center: r[4],
                    shift: r[5],
                    min_note: r[6],
                    max_note: r[7],
                    vibrato_width: r[8],
                    vibrato_time: r[9],
                    portamento_width: r[10],
                    portamento_time: r[11],
                    pitch_bend_min: r[12],
                    pitch_bend_max: r[13],
                    adsr1: u16_at(r, 16),
                    adsr2: u16_at(r, 18),
                    program: u16_at(r, 20),
                    vag: u16_at(r, 22),
                });
            }
        }
        if tones.len() != tone_count {
            return Err(format!("header says {tone_count} tones, programs hold {}", tones.len()));
        }
        let vag_sizes = (0..=vags).map(|i| u16_at(vh, sizes_at + 2 * i) as u32 * 8).collect();
        Ok(Vab {
            id: u32_at(vh, 8),
            version: u32_at(vh, 4),
            master_volume: vh[24],
            master_pan: vh[25],
            programs,
            tones,
            vag_sizes,
        })
    }

    /// The bytes of sample `n` (1-based) within the VB.
    pub fn vag<'a>(&self, vb: &'a [u8], n: usize) -> Option<&'a [u8]> {
        let start: u32 = self.vag_sizes.get(1..n)?.iter().sum();
        let len = *self.vag_sizes.get(n)?;
        vb.get(start as usize..(start + len) as usize)
    }
}

const FILTERS: [(i32, i32); 5] = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)];

/// Decodes PS-ADPCM to 16-bit PCM, stopping after the block flagged as the
/// end (flag bit 0) when it is not a loop.
pub fn decode_adpcm(data: &[u8]) -> Vec<i16> {
    let mut out = Vec::with_capacity(data.len() / 16 * 28);
    let (mut s1, mut s2) = (0i32, 0i32);
    for block in data.as_chunks::<16>().0 {
        let shift = (block[0] & 15) as i32;
        let (f0, f1) = FILTERS[((block[0] >> 4) as usize).min(4)];
        let shift = if shift > 12 { 9 } else { shift };
        for &b in &block[2..] {
            for nib in [b & 15, b >> 4] {
                let v = ((nib as i32) << 28 >> 28) << (12 - shift);
                let s = (v + ((s1 * f0 + s2 * f1 + 32) >> 6)).clamp(-32768, 32767);
                s2 = s1;
                s1 = s;
                out.push(s as i16);
            }
        }
        if block[1] & 1 != 0 && block[1] & 2 == 0 {
            break;
        }
    }
    out
}

/// A mono 16-bit WAV file.
pub fn wav(samples: &[i16], rate: u32) -> Vec<u8> {
    let len = samples.len() as u32 * 2;
    let mut w = Vec::with_capacity(44 + len as usize);
    w.extend(b"RIFF");
    w.extend((36 + len).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend((rate * 2).to_le_bytes());
    w.extend(2u16.to_le_bytes());
    w.extend(16u16.to_le_bytes());
    w.extend(b"data");
    w.extend(len.to_le_bytes());
    for s in samples {
        w.extend(s.to_le_bytes());
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_block_decodes_to_zero() {
        let mut block = [0u8; 16];
        block[1] = 1; // end
        assert_eq!(decode_adpcm(&block), vec![0; 28]);
    }

    #[test]
    fn nibble_scaling() {
        // shift 12 leaves the nibble value; filter 0.
        let mut block = [0u8; 16];
        block[0] = 12;
        block[2] = 0x71; // samples 1, 7
        let s = decode_adpcm(&block);
        assert_eq!(&s[..2], &[1, 7]);
        block[0] = 0;
        assert_eq!(decode_adpcm(&block)[0], 4096);
    }
}
