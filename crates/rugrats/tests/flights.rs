//! The effect flights (world::flights) and the animated sprites
//! (ui::hud::Sequences): their maths against the original's functions in
//! rrt-emu, and a flame and an explosion of level 7 frame by frame as the
//! oracle showed them (Circle at VSync 3300, a lamp broken at 2931).
//! Skipped without the disc.

use std::path::Path;

use rrt::emu::psx::machine_with_exe;
use rrt::emu::{BusExt, PsxExe};
use rrt::glam::{Mat3, Vec3};
use rugrats::disc::GameDisc;
use rugrats::gte::Trig;
use rugrats::ui::hud::{Sequences, Sprites};
use rugrats::world::flights::{self, Flights, Skin};
use rugrats_data::col::Col;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn disc() -> Option<GameDisc> {
    let cue = rrt::disc::Image::find(&root().join("work/disc")).ok()?;
    GameDisc::open(&cue).ok()
}

fn machine() -> Option<rrt::emu::machine::Machine<rrt::emu::psx::R3000, rrt::emu::Memory>> {
    let bytes = std::fs::read(root().join("work/fs/SLUS_006.50")).ok()?;
    let mut m = machine_with_exe(&PsxExe::parse(&bytes).unwrap()).unwrap();
    m.cpu.r[28] = 0x800c_3e80;
    Some(m)
}

/// Level 7's sprites, collision and flights.
fn level_7(disc: &GameDisc) -> (Sprites, Col, Flights) {
    (Sprites::load(disc, 7).unwrap(), Col::parse(&disc.file(466 + 7).unwrap()).unwrap(), Flights::load(disc).unwrap())
}

const SEED: u32 = 0x800c_445c;

/// 0x80088bac's cone of directions, for several axes and spreads, from
/// several seeds.
#[test]
fn cone_directions_match_the_original() {
    let (Some(disc), Some(mut m)) = (disc(), machine()) else { return };
    let mut f = Flights::load(&disc).unwrap();
    let axes = [
        [0, 0, 4096],
        [0, -4096, 0],
        [4096, 0, 0],
        [0, 0, -4096],
        [2896, 0, -2896],
        [-2896, 0, 2896],
        [300, -3900, 1000],
        [-1200, 2000, -3000],
    ];
    let identity = [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]];
    let (axis_at, out_at) = (0x8010_0000u32, 0x8010_0020u32);
    let mut seed = 12345u32;
    for axis in axes {
        for (min, max) in [(0, 120), (0, 2048), (0, 1024), (300, 600), (0, 0)] {
            for _ in 0..20 {
                seed = seed.wrapping_mul(69069).wrapping_add(1);
                m.bus.write_u32(SEED, seed).unwrap();
                for (k, c) in axis.iter().enumerate() {
                    m.bus.write_u32(axis_at + 4 * k as u32, *c as u32).unwrap();
                }
                let args = [axis_at as u64, 0x800b_4578, min as u64, max as u64, out_at as u64];
                m.call(0x8008_8bac, &args).unwrap();
                let want: Vec<i32> = (0..3).map(|k| m.bus.read_u32(out_at + 4 * k).unwrap() as i32).collect();
                f.seeds.direction = seed;
                let got = f.cone(axis, &identity, min, max);
                assert_eq!(got.to_vec(), want, "axis {axis:?} {min}..{max} seed {seed:#x}");
                assert_eq!(f.seeds.direction, m.bus.read_u32(SEED).unwrap());
            }
        }
    }
}

/// asin, acos and ratan2 (0x80021460, 0x80021500, 0x800a0328).
#[test]
fn angles_match_the_original() {
    let (Some(disc), Some(mut m)) = (disc(), machine()) else { return };
    let a = flights::Angles::load(&disc).unwrap();
    for x in (-4200..4200).step_by(3) {
        let arg = x as i64 as u64;
        assert_eq!(a.asin(x), m.call(0x8002_1460, &[arg]).unwrap() as i32 as i16 as i32, "asin {x}");
        assert_eq!(a.acos(x), m.call(0x8002_1500, &[arg]).unwrap() as i32 as i16 as i32, "acos {x}");
    }
    for y in (-9000..9000).step_by(611) {
        for x in (-9000..9000).step_by(577) {
            let want = m.call(0x800a_0328, &[y as i64 as u64, x as i64 as u64]).unwrap() as i32;
            assert_eq!(a.ratan2(y, x), want, "ratan2({y}, {x})");
        }
    }
    for (y, x) in [(0, 0), (0, 4096), (4096, 0), (-4096, 0), (0, -4096), (2896, -2896), (3_000_000, 7)] {
        let want = m.call(0x800a_0328, &[y as i64 as u64, x as i64 as u64]).unwrap() as i32;
        assert_eq!(a.ratan2(y, x), want, "ratan2({y}, {x})");
    }
}

/// The fade's colour to hue, saturation, value and back (0x8001f238,
/// 0x8001f360).
#[test]
fn hsv_matches_the_original() {
    let Some(mut m) = machine() else { return };
    let (rgb_at, hsv_at) = (0x8010_0000u32, 0x8010_0010u32);
    let levels = [0u8, 1, 7, 40, 80, 127, 128, 200, 254, 255];
    for r in levels {
        for g in levels {
            for b in levels {
                m.bus.write_u32(rgb_at, u32::from_le_bytes([r, g, b, 0])).unwrap();
                m.call(0x8001_f238, &[rgb_at as u64, hsv_at as u64]).unwrap();
                let w = m.bus.read_u32(hsv_at).unwrap();
                let want = ((w & 0xffff) as i32, (w >> 16 & 0xff) as i32, (w >> 24) as i32);
                assert_eq!(flights::rgb_to_hsv([r, g, b]), want, "hsv of {r} {g} {b}");
                // Back, with the value a little less as the fade takes it.
                let (h, s, v) = want;
                let v = (v - 9).max(0);
                m.bus.write_u32(hsv_at, h as u32 | (s as u32) << 16 | (v as u32) << 24).unwrap();
                m.call(0x8001_f360, &[hsv_at as u64, rgb_at as u64]).unwrap();
                let back = m.bus.read_u32(rgb_at).unwrap().to_le_bytes();
                assert_eq!(flights::hsv_to_rgb(h, s, v), [back[0], back[1], back[2]], "rgb of {h} {s} {v}");
            }
        }
    }
}

/// TEXACTRL.DAT: 36 sequences; the explosion's (0x1016) and the flame's
/// (0x1017) times and frames as the oracle's records showed them.
#[test]
fn sequences_load() {
    let Some(disc) = disc() else { return };
    let s = Sequences::load(&disc).unwrap();
    assert!(s.sequence(0x1023).is_some() && s.sequence(0x1024).is_none());
    assert!(s.sequence(0x2000).is_none());
    let (explosion, flame) = (s.sequence(0x1016).unwrap(), s.sequence(0x1017).unwrap());
    assert_eq!((explosion.time, flame.time), (150, 200));
    // Each frame as level 7's TEXPOS has it: page, u0 v0 and CLUT.
    let sprites = Sprites::load(&disc, 7).unwrap();
    let shown = |q: &rugrats::ui::hud::Sequence| -> Vec<(u16, [u8; 2], u16)> {
        q.frames.iter().take(5).map(|&n| sprites.entry(n as u32).map(|t| (t.tpage, t.uv[0], t.clut)).unwrap()).collect()
    };
    assert_eq!(
        shown(explosion),
        [
            (0x17, [0, 128], 0x449c),
            (0x17, [64, 128], 0x4bdc),
            (0x17, [128, 128], 0x4b9c),
            (0x17, [192, 128], 0x4b5c),
            (0x17, [0, 192], 0x4b1c)
        ]
    );
    assert_eq!(
        shown(flame),
        [
            (0x17, [0, 192], 0x4b1c),
            (0x17, [64, 192], 0x4adc),
            (0x17, [128, 192], 0x4a9c),
            (0x17, [192, 192], 0x4a5c),
            (0x18, [0, 0], 0x4a1c)
        ]
    );
}

/// A sequence's records: a shared one is held twice and freed by the
/// second release; the pool holds 32.
#[test]
fn sequence_records_are_held_and_freed() {
    let Some(disc) = disc() else { return };
    let mut s = Sequences::load(&disc).unwrap();
    let a = s.start_shared(0x101b, None).unwrap();
    assert_eq!(s.start_shared(0x101b, None), Some(a));
    s.release(a);
    assert_eq!(s.running(), 1);
    s.release(a);
    assert_eq!(s.running(), 0);
    let all: Vec<_> = (0..40).filter_map(|_| s.start(0x1017, None)).collect();
    assert_eq!(all.len(), 32);
}

/// A peeked record after a step: position, life, half size, and its
/// sequence's frame and time left.
type Peek = ([i32; 3], i32, i16, (usize, i32));

/// The ms of each oracle step from VSync 3346: dt 20 (66 ms) and 15 (50).
const STEPS: [i32; 13] = [20, 20, 15, 20, 20, 20, 15, 20, 20, 20, 15, 20, 20];

/// The first flame of the oracle's Circle at 3300 (record 0x801855f4):
/// launched at (25448, -3687, -8423) along (2118, -446, 3485), then after
/// each step its position, life, half size and its sequence's frame and
/// time left, as peeked.
#[test]
fn a_flame_flies_as_in_the_original() {
    let Some(disc) = disc() else { return };
    let (sprites, col, mut f) = level_7(&disc);
    assert!(f.flame([25448, -3687, -8423], [2118, -446, 3485], Some(&sprites)));
    let fl = f.iter().next().unwrap().clone();
    assert_eq!(
        (fl.kind(), fl.v, fl.size, fl.face, fl.colour),
        (11, [395, -84, 650], [750, 750], 0xd2a0_0000, [128; 3])
    );
    let Skin::Sequence(slot) = fl.skin else { panic!("no sequence") };
    let want: [Peek; 13] = [
        ([25571, -3714, -8220], 1434, 830, (0, 134)),
        ([25694, -3741, -8017], 1368, 910, (0, 68)),
        ([25786, -3761, -7865], 1318, 970, (0, 18)),
        ([25909, -3788, -7662], 1252, 1050, (1, 152)),
        ([26032, -3815, -7459], 1186, 1130, (1, 86)),
        ([26155, -3842, -7256], 1120, 1210, (1, 20)),
        ([26247, -3862, -7104], 1070, 1270, (2, 170)),
        ([26370, -3889, -6901], 1004, 1350, (2, 104)),
        ([26493, -3916, -6698], 938, 1430, (2, 38)),
        ([26616, -3943, -6495], 872, 1510, (3, 172)),
        ([26708, -3963, -6343], 822, 1570, (3, 122)),
        ([26831, -3990, -6140], 756, 1650, (3, 56)),
        ([26954, -4017, -5937], 690, 1730, (4, 190)),
    ];
    for (k, (&dt, w)) in STEPS.iter().zip(want).enumerate() {
        f.step(&col, Some(&sprites), dt);
        let fl = f.iter().next().unwrap();
        assert_eq!((fl.pos, fl.life, fl.size[0], fl.size[1]), (w.0, w.1, w.2, w.2), "step {k}");
        assert_eq!(fl.radius, w.2 as i32);
        assert_eq!(f.sequences.frame(slot), Some(w.3), "step {k}");
    }
    // The fifth frame is page 0x18's first sprite.
    assert_eq!(f.sequences.texture(slot).map(|t| (t.tpage, t.uv[0])), Some((0x18, [0, 0])));
    // At dt 10 (33 ms a frame) it lives 690 / 33 more frames, then lets its
    // sequence go.
    for _ in 0..20 {
        f.step(&col, Some(&sprites), 10);
    }
    assert_eq!(f.len(), 1);
    f.step(&col, Some(&sprites), 10);
    assert!(f.is_empty());
    assert_eq!(f.sequences.running(), 0);
}

/// The lamp broken at 2931 (record 0x80185558): an explosion standing at
/// the lamp, 3000 across, white, sequence 0x1016 from 150 ms.
#[test]
fn an_explosion_stands_as_in_the_original() {
    let Some(disc) = disc() else { return };
    let (sprites, col, mut f) = level_7(&disc);
    assert!(f.explosion([26004, -1701, -7911], Some(&sprites)));
    let fl = f.iter().next().unwrap().clone();
    assert_eq!(
        (fl.kind(), fl.v, fl.size, fl.face, fl.colour, fl.life),
        (7, [0; 3], [3000; 2], 0xd2a0_0000, [255; 3], 5000)
    );
    let Skin::Sequence(slot) = fl.skin else { panic!("no sequence") };
    for (dt, life, frame) in [(15, 4950, (0, 100)), (20, 4884, (0, 34)), (20, 4818, (1, 118)), (15, 4768, (1, 68))] {
        f.step(&col, Some(&sprites), dt);
        assert_eq!(f.iter().next().unwrap().life, life);
        assert_eq!(f.sequences.frame(slot), Some(frame));
    }
}

/// Reptar's head at the oracle's first flame (part 2 on slot 16: its
/// matrix rows and joint): the start and direction 0x8005ce74 gave.
#[test]
fn the_flame_leaves_the_head_as_in_the_original() {
    let Some(disc) = disc() else { return };
    let f = Flights::load(&disc).unwrap();
    let trig = Trig::load(&disc).unwrap();
    let m = [[3514.0, 128.0, -2120.0], [121.0, 4079.0, 447.0], [2119.0, -448.0, 3487.0]];
    let row = |r: usize| Vec3::new(m[r][0], m[r][1], m[r][2]) / 4096.0;
    // The game keeps the part's matrix world to part: the part's rotation
    // is its transpose, whose columns are those rows.
    let rot = Mat3::from_cols(row(0), row(1), row(2));
    let (start, dir) = flights::flame_from(&f.angles, &trig, rot, Vec3::new(25225.0, -2236.0, -8706.0));
    assert_eq!(start, [25448, -3687, -8423]);
    assert_eq!(dir, [2118, -446, 3485]);
}

/// Debris: ten bits of rubble, white, at speed 100 at most, gone after
/// their 1000 ms (31 frames of 33).
#[test]
fn debris_flies_and_ends() {
    let Some(disc) = disc() else { return };
    let (sprites, col, mut f) = level_7(&disc);
    f.debris([25162, -2000, -9351], 0, Some(&sprites));
    assert_eq!(f.len(), 10);
    for fl in f.iter() {
        assert_eq!((fl.kind(), fl.colour, fl.size, fl.face, fl.life), (5, [255; 3], [200, 200], 0xc080_0000, 1000));
        let Skin::Sprite(n) = fl.skin else { panic!("no sprite") };
        assert!([392, 393, 394, 395, 396, 397, 399, 401, 403, 405].contains(&n));
        assert!(fl.v.iter().all(|c| c.abs() <= 100 * 65536 / 6000), "{:?}", fl.v);
    }
    for _ in 0..30 {
        f.step(&col, Some(&sprites), 10);
    }
    assert_eq!(f.len(), 10);
    f.step(&col, Some(&sprites), 10);
    assert!(f.is_empty());
}

/// Circle in level 7: no flame until the fire's animation event, then
/// one a frame while the animation plays, none after.
#[test]
fn reptar_breathes_fire() {
    use rugrats::input::{self, Pads};
    let Some(disc) = disc() else { return };
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(7)).unwrap();
    let mut pads = Pads::default();
    let (mut first, mut last, mut most) = (None, None, 0);
    for frame in 0..140 {
        pads.push_bits(0, if (20..22).contains(&frame) { 1 << input::CIRCLE } else { 0 });
        g.frame(&pads);
        let flames = g.flights.iter().filter(|f| f.kind() == 11).count();
        if flames > most {
            assert_eq!(flames, most + 1, "one a frame");
            assert_eq!(g.player.action, Some(51));
            first = first.or(Some(frame));
            last = Some(frame);
            most = flames;
        }
    }
    let (first, last) = (first.expect("no flame"), last.unwrap());
    assert!(first > 22, "before the event: {first}");
    assert_eq!(most, last - first + 1);
    // 1500 ms each: all gone by frame 140 (the fire at Reptar's half
    // speed, 0x8005a068, lasts about 35 frames as in the oracle).
    assert_eq!(g.flights.iter().filter(|f| f.kind() == 11).count(), 0);
}

/// A pie's splat: five white bits of sprite 359, opaque, at speed 50 at
/// most about the surface's normal, bouncing by 1000 / 4096.
#[test]
fn a_splat_throws_five_bits() {
    let Some(disc) = disc() else { return };
    let (sprites, _, mut f) = level_7(&disc);
    f.splat([0, -4096, 0], [0, 0, 0], Some(&sprites));
    assert_eq!(f.len(), 5);
    for fl in f.iter() {
        assert_eq!(
            (fl.kind(), fl.colour, fl.skin, fl.face, fl.bounce, fl.life),
            (5, [255; 3], Skin::Sprite(359), 0xd080_0000, 1000, 2000)
        );
        assert!(fl.v.iter().all(|c| c.abs() <= 50 * 65536 / 6000), "{:?}", fl.v);
        // 3391 * 64 / 300.
        assert_eq!(fl.spin, 723);
    }
}

/// A collectible taken (0x800514b0): three bits of its sprite (a bar's
/// 295 here), 500 ms, bounce 2048, speed under 100.
#[test]
fn a_collectible_sparkles() {
    let Some(disc) = disc() else { return };
    let (sprites, _, mut f) = level_7(&disc);
    f.sparkle(999, [0, 0, 0], 0, Some(&sprites));
    assert_eq!(f.len(), 3);
    for fl in f.iter() {
        assert_eq!((fl.kind(), fl.skin, fl.face, fl.bounce, fl.life), (5, Skin::Sprite(295), 0xd280_0000, 2048, 500));
        assert!(fl.v.iter().all(|c| c.abs() < 100 * 65536 / 6000), "{:?}", fl.v);
    }
}

/// Walking in level 4's water (COL byte 4 bit 0) splashes each frame
/// (0x80050770).
#[test]
fn walking_in_water_splashes() {
    use rugrats::input::{self, Pads};
    let Some(disc) = disc() else { return };
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(4)).unwrap();
    let at = water_spot(&g);
    let feet = g.player.feet();
    g.player.pos = [at[0], at[1] + feet, at[2]];
    let mut pads = Pads::default();
    let mut most = 0;
    for _ in 0..20 {
        pads.push_bits(0, 1 << input::UP);
        g.frame(&pads);
        most = most.max(g.flights.iter().filter(|f| f.kind() == 5).count());
    }
    assert!(most > 0, "no splash at {at:?}");
}

/// The water floor nearest level 4's start with room to stand, in a room.
fn water_spot(g: &rugrats::world::game::Game) -> [i32; 3] {
    use rugrats::world::collide;
    // The water floor nearest the start with room to stand in a room.
    let (lo, start) = (g.col.min[1], g.player.pos);
    let mut spots: Vec<(i64, [i32; 3])> = Vec::new();
    for x in (start[0] - 40000..start[0] + 40000).step_by(256) {
        for z in (start[2] - 40000..start[2] + 40000).step_by(256) {
            let q = collide::query(&g.col, [x, lo, z]);
            let q = if q.kind == 0 { q } else { collide::query(&g.col, [x, q.floor - 1, z]) };
            let open = q.kind == 0 && q.floor - q.ceiling > 3000;
            if open && collide::floor_flags(&g.col, &q) & 1 != 0 && g.room_of([x, q.floor - 100, z]).is_some() {
                let d = ((x - start[0]) as i64).pow(2) + ((z - start[2]) as i64).pow(2);
                spots.push((d, [x, q.floor, z]));
            }
        }
    }
    spots.into_iter().min_by_key(|s| s.0).map(|s| s.1).expect("no water in level 4")
}

/// Landing on level 4's water plays sound 57 (0x8005b624).
#[test]
fn landing_in_water_splashes_aloud() {
    use rugrats::input::Pads;
    let Some(disc) = disc() else { return };
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(4)).unwrap();
    let at = water_spot(&g);
    let feet = g.player.feet();
    g.player.pos = [at[0], at[1] + feet - 1500, at[2]];
    let pads = Pads::default();
    let mut heard = Vec::new();
    for _ in 0..90 {
        g.frame(&pads);
        heard.extend(g.effects.iter().map(|e| e.sound));
    }
    assert!(heard.contains(&57), "heard {heard:?}");
}

/// Walking on the hub's yard (floor flag 16) throws up grass, model 1008
/// as a kind 2 flight (0x80050b5c, from 0x8005e1f4), as the original does
/// there about every other frame.
#[test]
fn the_yard_throws_up_grass() {
    use rugrats::input::{self, Pads};
    let Some(disc) = disc() else { return };
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(1)).unwrap();
    assert!(g.set.models.get(flights::DUST).is_some());
    // Where the original was put for the check: (-1536, 708, 30000).
    g.player.pos = [-1536, 708, 30000];
    let mut pads = Pads::default();
    let mut most = 0;
    for _ in 0..40 {
        pads.push_bits(0, 1 << input::UP);
        g.frame(&pads);
        let n = g.flights.iter().filter(|f| f.kind() == 2 && f.skin == Skin::Model(flights::DUST)).count();
        most = most.max(n);
    }
    assert!(most >= 5, "{most} tufts");
    assert!(g.flights.models().all(|(m, ..)| m == flights::DUST));
}

/// Beside a door sign in the hub, the sign sparkles every frame
/// (0x80049eec's 0x80050fd0): three kind 5 sprite 296 a frame, 500 ms.
#[test]
fn a_door_sign_sparkles() {
    use rugrats::input::Pads;
    use rugrats::world::collide;
    let Some(disc) = disc() else { return };
    let mut g = rugrats::world::game::Game::open(&disc, rugrats::levels::Level(1)).unwrap();
    // Level 10's sign (template 498) in room 23, near the start.
    let sign = g.set.placements.iter().find(|p| p.kind == 1 && p.template == 498).expect("the sign").pos;
    let (x, z) = (sign[0], sign[2] - 1200);
    let q = collide::query(&g.col, [x, sign[1] - 1500, z]);
    assert_eq!(q.kind, 0);
    g.player.pos = [x, q.floor + g.player.feet(), z];
    let pads = Pads::default();
    let mut most = 0;
    for _ in 0..10 {
        g.frame(&pads);
        most = most.max(g.flights.iter().filter(|f| f.kind() == 5 && f.skin == Skin::Sprite(296)).count());
    }
    assert!(most >= 3, "{most} sparks; Tommy {:?} room {:?}", g.player.pos, g.current_room());
}
