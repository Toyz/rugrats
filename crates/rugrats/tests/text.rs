//! The text system against the original's functions in rrt-emu: centring
//! (0x8002e144), the glyph sprites (0x8002e67c, read back from its packet
//! buffer) and the slides (0x8002e2b4). Skipped without the disc.

use std::path::Path;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};
use rugrats::disc::GameDisc;
use rugrats::text::{self, Fonts, Text};

type Emu = rrt::emu::Machine<rrt::emu::psx::R3000, rrt::emu::Memory>;

const ITEM: u32 = 0x8010_0000;
const STRING: u32 = 0x8010_0100;
const PACKETS: u32 = 0x8011_0000;

fn setup() -> Option<(Emu, Fonts)> {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let exe = PsxExe::parse(&std::fs::read(root.join("work/fs/SLUS_006.50")).ok()?).ok()?;
    let cue = rrt::disc::Image::find(&root.join("work/disc")).ok()?;
    let fonts = Fonts::load(&GameDisc::open(&cue).ok()?).ok()?;
    let mut m = machine_with_exe(&exe).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    // The fonts' CLUTs, as boot sets them (0x8002ed80).
    for k in 0..3 {
        m.bus.write_u16(0x800c_3f7c + 2 * k, text::CLUT).unwrap();
    }
    Some((m, fonts))
}

fn write_item(m: &mut Emu, t: &Text) {
    for k in 0..44 {
        m.bus.write_u8(ITEM + k, 0).unwrap();
    }
    let h = |m: &mut Emu, at: u32, v: i16| m.bus.write_u16(ITEM + at, v as u16).unwrap();
    h(m, 0x00, t.font as i16);
    h(m, 0x02, t.spacing);
    m.bus.write_u32(ITEM + 4, t.rgb[0] as u32 | (t.rgb[1] as u32) << 8 | (t.rgb[2] as u32) << 16).unwrap();
    h(m, 0x08, t.x);
    h(m, 0x0a, t.y);
    h(m, 0x0c, t.target.0);
    h(m, 0x0e, t.target.1);
    h(m, 0x10, t.rest.0);
    h(m, 0x12, t.rest.1);
    m.bus.write_u32(ITEM + 0x14, STRING).unwrap();
    h(m, 0x18, t.flags as i16);
    h(m, 0x1a, t.toggle);
    h(m, 0x20, t.speed.0);
    h(m, 0x22, t.speed.1);
    h(m, 0x24, t.count.0);
    h(m, 0x26, t.count.1);
    h(m, 0x28, t.wait);
    for (k, b) in t.string.bytes().chain([0]).enumerate() {
        m.bus.write_u8(STRING + k as u32, b).unwrap();
    }
}

fn read_h(m: &mut Emu, at: u32) -> i16 {
    m.bus.read_u16(ITEM + at).unwrap() as i16
}

fn items() -> Vec<Text> {
    let mut out = Vec::new();
    for (font, s) in [(0u8, "Start Game"), (1, "Load Game"), (2, "CANCEL"), (1, "Two\nlines here"), (2, "x: 1, y? [ok]!")] {
        out.push(Text { font, spacing: 1, rgb: [255, 255, 5], x: 10, y: -20, string: s.into(), ..Default::default() });
    }
    out
}

#[test]
fn centring_matches_the_original() {
    let Some((mut m, fonts)) = setup() else {
        eprintln!("skipped: no disc");
        return;
    };
    for t in items() {
        for flags in 0..4u64 {
            let mut ours = t.clone();
            fonts.centre(&mut ours, flags & 1 != 0, flags & 2 != 0);
            write_item(&mut m, &t);
            m.call(0x8002_e144, &[ITEM as u64, flags]).unwrap();
            let theirs = (read_h(&mut m, 0x08), read_h(&mut m, 0x0a));
            assert_eq!((ours.x, ours.y), theirs, "{:?} flags {flags}", t.string);
        }
    }
}

#[test]
fn sprites_match_the_original() {
    let Some((mut m, fonts)) = setup() else {
        eprintln!("skipped: no disc");
        return;
    };
    for t in items() {
        write_item(&mut m, &t);
        for (at, v) in [(0x800d_e3b0u32, PACKETS), (0x800d_e3b4, PACKETS), (0x800d_e3b8, PACKETS + 0x8000)] {
            m.bus.write_u32(at, v).unwrap();
        }
        m.bus.write_u32(0x800c_3f74, 0).unwrap();
        m.bus.write_u32(0x800c_3f70, 0).unwrap();
        m.call(0x8002_e67c, &[ITEM as u64]).unwrap();
        // A draw mode packet (8 bytes), then a sprite packet (20) a glyph.
        let end = m.bus.read_u32(0x800d_e3b0).unwrap();
        let mut theirs = Vec::new();
        let mut p = PACKETS + 8;
        while p < end {
            let w = |m: &mut Emu, k: u32| m.bus.read_u32(p + 4 * k).unwrap();
            let (c, xy, uv, wh) = (w(&mut m, 1), w(&mut m, 2), w(&mut m, 3), w(&mut m, 4));
            assert_eq!(c >> 24, 0x64);
            assert_eq!((uv >> 16) as u16, text::CLUT);
            theirs.push(text::Sprite {
                x: xy as i16,
                y: (xy >> 16) as i16,
                w: wh as i16,
                h: (wh >> 16) as i16,
                u: uv as u8,
                v: (uv >> 8) as u8,
                rgb: [c as u8, (c >> 8) as u8, (c >> 16) as u8],
            });
            p += 20;
        }
        assert_eq!(fonts.sprites(&t), theirs, "{:?}", t.string);
    }
}

#[test]
fn slides_match_the_original() {
    let Some((mut m, _)) = setup() else {
        eprintln!("skipped: no disc");
        return;
    };
    // The menu's items (0x8003d5a0), a bounce, an x slide, a wait.
    let base = Text { font: 1, spacing: 1, string: "Item".into(), ..Default::default() };
    let cases = [
        Text { y: 200, target: (0, 25), rest: (0, 25), speed: (0, 12), count: (0, 4), flags: text::SLIDE_Y | text::WOBBLE, ..base.clone() },
        Text { y: 250, target: (0, 50), rest: (0, 50), speed: (0, 12), count: (0, 4), flags: text::SLIDE_Y | text::WOBBLE, ..base.clone() },
        Text { y: -90, target: (0, 40), rest: (0, 10), speed: (0, 5), count: (0, 81), flags: text::SLIDE_Y | text::BOUNCE, ..base.clone() },
        Text { x: -300, target: (17, 0), rest: (17, 0), speed: (9, 0), flags: text::SLIDE_X, ..base.clone() },
        Text { y: 150, target: (0, 0), rest: (0, 0), speed: (0, 8), wait: 10, flags: text::SLIDE_Y | text::WAIT, ..base.clone() },
    ];
    for t in cases {
        let mut ours = t.clone();
        write_item(&mut m, &t);
        for frame in 0..80 {
            ours.step();
            m.call(0x8002_e2b4, &[ITEM as u64]).unwrap();
            let theirs = (read_h(&mut m, 0x08), read_h(&mut m, 0x0a), m.bus.read_u16(ITEM + 0x18).unwrap(), read_h(&mut m, 0x0e));
            assert_eq!((ours.x, ours.y, ours.flags, ours.target.1), theirs, "frame {frame} of {t:?}");
        }
    }
}
