//! The height-field query against the original's (0x8002b240) in rrt-emu,
//! over a lattice of points in several databases. Skipped without the disc.

use std::path::Path;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};
use rugrats::collide::{Query, query};
use rugrats::disc::GameDisc;
use rugrats_data::col::Col;

const COL_AT: u32 = 0x8010_0000;
const POS: u32 = 0x801f_0000;

#[test]
fn queries_match_the_original() {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let (Ok(bytes), Ok(cue)) =
        (std::fs::read(root.join("work/fs/SLUS_006.50")), rrt::disc::Image::find(&root.join("work/disc")))
    else {
        eprintln!("skipped: no disc");
        return;
    };
    let disc = GameDisc::open(&cue).unwrap();
    let exe = PsxExe::parse(&bytes).unwrap();
    for db in [0usize, 1, 2, 5, 16] {
        let raw = disc.file(466 + db).unwrap();
        let col = Col::parse(&raw).unwrap();
        let mut m = machine_with_exe(&exe).unwrap();
        m.cpu.r[28] = 0x800c_3e80;
        for (k, b) in raw.iter().enumerate() {
            m.bus.write_u8(COL_AT + k as u32, *b).unwrap();
        }
        // load_rooms keeps the file at 0x800c4a88; 0x8002a65c sets it up.
        m.bus.write_u32(0x800c_4a88, COL_AT).unwrap();
        m.call(0x8002_a65c, &[]).unwrap();
        let (lo, hi) = (col.min, col.max);
        let mut checked = 0;
        let mut inside = 0;
        for x in (lo[0] - 700..hi[0] + 700).step_by(613) {
            for z in (lo[2] - 700..hi[2] + 700).step_by(587) {
                for y in (lo[1] - 300..hi[1] + 300).step_by(331) {
                    let pos = [x, y, z];
                    for (k, v) in pos.iter().enumerate() {
                        m.bus.write_u32(POS + 4 * k as u32, *v as u32).unwrap();
                    }
                    let out = POS + 0x20;
                    let r = m.call(0x8002_b240, &[POS as u64, out as u64, (out + 4) as u64, (out + 8) as u64]).unwrap()
                        as i32;
                    let ceiling = m.bus.read_u32(out).unwrap() as i32;
                    let floor = m.bus.read_u32(out + 4).unwrap() as i32;
                    let tile_ptr = m.bus.read_u32(out + 8).unwrap();
                    let ours = query(&col, pos);
                    let theirs_tile = (tile_ptr != 0).then(|| tile_ptr);
                    // The tile as the original's pointer: the column's data
                    // start plus cells and the tile's index.
                    let our_ptr = ours.tile.map(|(c, i)| {
                        let base = m.bus.read_u32(0x800c_4a94).unwrap() + 8 * c as u32;
                        let data = m.bus.read_u32(base).unwrap();
                        let cells = m.bus.read_u16(base + 4).unwrap() as u32;
                        data + 2 * cells + 8 * i as u32
                    });
                    let ours_cmp = Query { tile: None, ..ours };
                    let theirs = Query { kind: r, ceiling, floor, tile: None };
                    assert_eq!((ours_cmp, our_ptr), (theirs, theirs_tile), "db {db} at {pos:?}");
                    checked += 1;
                    inside += usize::from(r == 0);
                }
            }
        }
        eprintln!("db {db}: {checked} points, {inside} between heights");
    }
}
