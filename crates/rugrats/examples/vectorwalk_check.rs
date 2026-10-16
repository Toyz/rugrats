//! Holds `collide::segment` to the original segment test (0x8002cd7c,
//! "vectorwalk"), called on the same segments with the same COL.
//!
//! `vectorwalk_check [LEVEL..] [--bare N]`
//!
//! For each level (default 3 and 1) it plays the original in the oracle
//! into that level, then calls 0x8002cd7c with `call_nested` on segments
//! around the player and compares the return and all four outputs with the
//! port's on the database's COL from the disc. `--bare N` also checks N
//! segments per database in a bare R3000 with only the COL set up
//! (0x8002a65c), over every database on the disc.

use std::path::Path;
use std::rc::Rc;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, HookResult, PsxExe};
use rugrats::disc::GameDisc;
use rugrats::world::collide::{Segment, query, segment};
use rugrats_data::col::Col;

const COL: usize = 466;
const VECTORWALK: u32 = 0x8002_cd7c;
/// What the outputs hold before a call, to see the ones it leaves.
const UNSET: i32 = 0x5a5a_5a5a;
const PLAYER: u32 = 0x800e_2b48;
/// The game's gp.
const GP: u32 = 0x800c_3e80;

/// A small generator, so a run is the same every time.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u32) as i32
    }
}

/// A segment near `around`: mostly starting in the open, of mixed lengths
/// and directions (some vertical, some along an axis, some through walls).
fn pick(rng: &mut Rng, col: &Col, around: [i32; 3], spread: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let mut from = around;
    for _ in 0..40 {
        from = [0, 1, 2].map(|k| around[k] + rng.range(-spread[k], spread[k]));
        if query(col, from).kind == 0 || rng.next().is_multiple_of(8) {
            break;
        }
    }
    let len = match rng.next() % 4 {
        0 => 300,
        1 => 1500,
        2 => 5000,
        _ => 40000,
    };
    let mut d = [rng.range(-len, len), rng.range(-len / 3, len / 3), rng.range(-len, len)];
    match rng.next() % 10 {
        0 => (d[0], d[2]) = (0, 0),
        1 => d[0] = 0,
        2 => d[2] = 0,
        3 => d[1] = 0,
        4 => d = [d[0], d[1], d[0]],
        _ => {}
    }
    if rng.next().is_multiple_of(40) {
        d = [0; 3];
    }
    (from, [from[0] + d[0], from[1] + d[1], from[2] + d[2]])
}

/// Where the segment's inputs and outputs go in the machine's RAM.
struct Buffers {
    at: u32,
}

impl Buffers {
    fn from(&self) -> u32 {
        self.at
    }
    fn to(&self) -> u32 {
        self.at + 0x10
    }
    fn depth(&self) -> u32 {
        self.at + 0x20
    }
    fn point(&self) -> u32 {
        self.at + 0x30
    }
    fn normal(&self) -> u32 {
        self.at + 0x40
    }
    fn tile(&self) -> u32 {
        self.at + 0x50
    }
    fn args(&self) -> [u32; 6] {
        [self.from(), self.to(), self.depth(), self.point(), self.normal(), self.tile()]
    }

    fn write(&self, bus: &mut impl BusExt, from: [i32; 3], to: [i32; 3]) {
        for k in 0..3u32 {
            bus.write_u32(self.from() + 4 * k, from[k as usize] as u32).unwrap();
            bus.write_u32(self.to() + 4 * k, to[k as usize] as u32).unwrap();
        }
        bus.write_u32(self.from() + 12, 0).unwrap();
        bus.write_u32(self.to() + 12, 0).unwrap();
        for a in (self.depth()..self.tile() + 4).step_by(4) {
            bus.write_u32(a, UNSET as u32).unwrap();
        }
    }

    /// The outputs, the tile as the original's pointer.
    fn read(&self, bus: &mut impl BusExt) -> (i32, [i32; 3], [i32; 3], u32) {
        let w = |bus: &mut _, a: u32| BusExt::read_u32(bus, a).unwrap() as i32;
        let v = |bus: &mut _, a: u32| [w(bus, a), w(bus, a + 4), w(bus, a + 8)];
        (w(bus, self.depth()), v(bus, self.point()), v(bus, self.normal()), w(bus, self.tile()) as u32)
    }
}

/// The original's pointer for a tile of the port's: the column's data
/// (0x800c4a94) past its cells.
fn tile_ptr(bus: &mut impl BusExt, tile: Option<(usize, usize)>) -> u32 {
    tile.map_or(0, |(c, k)| {
        let base = bus.read_u32(0x800c_4a94).unwrap() + 8 * c as u32;
        let data = bus.read_u32(base).unwrap();
        let cells = bus.read_u16(base + 4).unwrap() as u32;
        data + 2 * cells + 8 * k as u32
    })
}

#[derive(Default)]
struct Tally {
    compared: usize,
    differ: usize,
    by_result: [usize; 3],
    /// Stops by a diagonal, by a floor or ceiling plane, and starts not in
    /// the open whose from - to passes 16 bits (the GTE's IR registers).
    diagonal: usize,
    plane: usize,
    wide: usize,
    /// Calls the original did not finish: VectorNormal's `add` (0x8009f2fc)
    /// raises an overflow exception when the squares pass 2^31.
    trapped: usize,
}

impl Tally {
    fn check(
        &mut self,
        bus: &mut impl BusExt,
        col: &Col,
        label: &str,
        (from, to): ([i32; 3], [i32; 3]),
        r: i32,
        b: &Buffers,
    ) {
        let theirs = b.read(bus);
        let mut out = Segment { depth: UNSET, point: [UNSET; 3], normal: [UNSET; 3], tile: None };
        let ours_r = segment(col, from, to, &mut out);
        let ours = (out.depth, out.point, out.normal, tile_ptr(bus, out.tile));
        self.compared += 1;
        if r == 1 && theirs.2[0].abs() == 2896 {
            self.diagonal += 1;
        }
        if r == 1 && theirs.2[1] != 0 {
            self.plane += 1;
        }
        if r == 2 && (0..3).any(|k| (from[k] - to[k]).abs() > 32767) {
            self.wide += 1;
        }
        if let Some(n) = self.by_result.get_mut(r as usize) {
            *n += 1;
        }
        if ours_r != r || ours != theirs {
            self.differ += 1;
            if self.differ <= 20 {
                println!("{label}: {from:?} -> {to:?}: original {r} {theirs:?}, port {ours_r} {ours:?}");
            }
        }
    }

    fn add(&mut self, t: &Tally) {
        self.compared += t.compared;
        self.differ += t.differ;
        for k in 0..3 {
            self.by_result[k] += t.by_result[k];
        }
        self.diagonal += t.diagonal;
        self.plane += t.plane;
        self.wide += t.wide;
        self.trapped += t.trapped;
    }

    fn trap(&mut self, label: &str, from: [i32; 3], to: [i32; 3], e: impl std::fmt::Debug) {
        self.trapped += 1;
        println!("{label}: {from:?} -> {to:?}: the original stops: {e:?}");
    }

    fn report(&self, label: &str) {
        println!(
            "{label}: {} segments, {} differ (returns: {} through, {} stopped, {} not in the open; \
             {} at a diagonal, {} at a floor or ceiling, {} past 16 bits; {} trapped in the original)",
            self.compared,
            self.differ,
            self.by_result[0],
            self.by_result[1],
            self.by_result[2],
            self.diagonal,
            self.plane,
            self.wide,
            self.trapped
        );
    }
}

/// Plays the original into `level` (the scripted route of the screens
/// probe: main's level argument patched, the menus pressed through).
fn into_level(cue: &Path, level: u32) -> rugrats_hle::Hle {
    let mut hle = rugrats_hle::Hle::new(Rc::new(rrt::disc::Image::open(cue).unwrap()), true).unwrap();
    let done = Rc::new(std::cell::Cell::new(false));
    hle.m.hook(0x8001_7c0c, move |c| {
        if !done.replace(true) {
            c.bus.load_ram(0x800c_5678, &level.to_le_bytes());
        }
        HookResult::Continue
    });
    hle.m.hook(0x8007_4958, |_| HookResult::ret32(0));
    let presses = [(300, "start"), (500, "cross"), (650, "cross"), (800, "cross"), (1650, "cross"), (1800, "cross")];
    for frame in 1..=2700u64 {
        hle.m.bus.load_ram(0x8004_472c, &0x1000_004au32.to_le_bytes());
        match frame {
            1 => hle.m.bus.load_ram(0x8001_8824, &(0x2404_0000 + level).to_le_bytes()),
            2400 => hle.m.bus.load_ram(0x8001_8824, &0x00a0_2021u32.to_le_bytes()),
            _ => {}
        }
        hle.pad = presses
            .iter()
            .filter(|(at, _)| (*at..at + 4).contains(&frame))
            .filter_map(|(_, b)| rugrats_hle::button::by_name(b))
            .fold(0, |a, b| a | b);
        hle.run_frame().unwrap();
    }
    hle
}

/// The database whose COL the original has loaded (0x800c4a88).
fn loaded_col(bus: &mut impl BusExt, disc: &GameDisc) -> (usize, Col) {
    let at = bus.read_u32(0x800c_4a88).unwrap();
    for db in 0..64 {
        let Ok(raw) = disc.file(COL + db) else { break };
        // The column pointers (+0x2c) are set at load; the rest is the file.
        let head = bus.bytes(at, 0x2c).unwrap();
        let tail = bus.bytes(at + 0x102c, raw.len().min(0x4000) - 0x102c).unwrap();
        if raw[..0x2c] == head[..] && raw[0x102c..raw.len().min(0x4000)] == tail[..] {
            return (db, Col::parse(&raw).unwrap());
        }
    }
    panic!("no COL on the disc matches the one in RAM");
}

fn main() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let cue = rrt::disc::Image::find(&root.join("work/disc")).unwrap();
    let disc = GameDisc::open(&cue).unwrap();
    let mut levels = Vec::new();
    let mut bare = 0usize;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--bare" => bare = args.next().and_then(|s| s.parse().ok()).unwrap_or(2000),
            n => levels.extend(n.parse::<u32>().ok()),
        }
    }
    if levels.is_empty() && bare == 0 {
        levels = vec![3, 1];
    }
    let mut total = Tally::default();
    for level in levels {
        let mut hle = into_level(&cue, level);
        let (db, col) = loaded_col(&mut hle.m.bus, &disc);
        let w = |hle: &mut rugrats_hle::Hle, a: u32| hle.m.bus.read_u32(a).unwrap() as i32;
        let player = [w(&mut hle, PLAYER + 0x18), w(&mut hle, PLAYER + 0x1c), w(&mut hle, PLAYER + 0x20)];
        println!("level {level}: database {db}, player at {player:?}");
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ level as u64);
        let mut tally = Tally::default();
        for n in 0..600 {
            let (from, to) = if n == 0 {
                (player, [player[0] + 3000, player[1], player[2] + 1000])
            } else {
                pick(&mut rng, &col, player, [4000, 1200, 4000])
            };
            // Below the game's stack: the buffers, and call_nested's frame
            // (sp - 0x400) taking the two stack arguments.
            let sp = hle.m.cpu.r[29];
            let b = Buffers { at: sp.wrapping_sub(0x1000) & !15 };
            b.write(&mut hle.m.bus, from, to);
            let call_sp = sp.wrapping_sub(0x400) & !7;
            hle.m.bus.write_u32(call_sp + 0x10, b.normal()).unwrap();
            hle.m.bus.write_u32(call_sp + 0x14, b.tile()).unwrap();
            let a = b.args();
            // The collision code finds the COL through gp, which the code
            // the frame stopped in need not hold: the game's, for the call.
            let gp = std::mem::replace(&mut hle.m.cpu.r[28], GP);
            let r = hle.call_nested(VECTORWALK, &a[..4]);
            hle.m.cpu.r[28] = gp;
            let r = match r {
                Ok(r) => r as i32,
                Err(e) => {
                    tally.trap(&format!("level {level}"), from, to, e);
                    continue;
                }
            };
            tally.check(&mut hle.m.bus, &col, &format!("level {level}"), (from, to), r, &b);
        }
        tally.report(&format!("level {level} (database {db})"));
        total.add(&tally);
    }
    if bare > 0 {
        let bytes = std::fs::read(root.join("work/fs/SLUS_006.50")).unwrap();
        let exe = PsxExe::parse(&bytes).unwrap();
        for db in 0..64 {
            let Ok(raw) = disc.file(COL + db) else { break };
            let col = match Col::parse(&raw) {
                Ok(col) => col,
                Err(e) => {
                    println!("database {db}: {e}");
                    continue;
                }
            };
            let mut m = machine_with_exe(&exe).unwrap();
            m.cpu.r[28] = GP;
            const COL_AT: u32 = 0x8010_0000;
            for (k, v) in raw.iter().enumerate() {
                m.bus.write_u8(COL_AT + k as u32, *v).unwrap();
            }
            m.bus.write_u32(0x800c_4a88, COL_AT).unwrap();
            m.call(0x8002_a65c, &[]).unwrap();
            let b = Buffers { at: 0x801f_0000 };
            let mut rng = Rng(0x2545_f491_4f6c_dd1d ^ db as u64);
            let mut tally = Tally::default();
            let mid = [0, 1, 2].map(|k| (col.min[k] + col.max[k]) / 2);
            let spread = [0, 1, 2].map(|k| (col.max[k] - col.min[k]) / 2 + 500);
            for _ in 0..bare {
                let (from, to) = pick(&mut rng, &col, mid, spread);
                b.write(&mut m.bus, from, to);
                let r = match m.call(VECTORWALK, &b.args().map(u64::from)) {
                    Ok(r) => r as i32,
                    Err(e) => {
                        tally.trap(&format!("database {db}"), from, to, e);
                        continue;
                    }
                };
                tally.check(&mut m.bus, &col, &format!("database {db}"), (from, to), r, &b);
            }
            tally.report(&format!("database {db} (bare)"));
            total.add(&tally);
        }
    }
    total.report("all");
}
