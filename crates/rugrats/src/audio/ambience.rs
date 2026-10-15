//! The levels' ambience (0x800818e0, table 0x800b9618): a countdown
//! (0x800c4cc0) that says a speech line from the level's tables when it
//! runs out. Speech only; nothing that changes play.

use crate::actors::npc::Mode;
use crate::actors::player::DT;
use crate::disc::GameDisc;
use crate::levels::Level;
use crate::world::game::Game;

/// The line tables (s16 each) the callbacks pick from.
pub struct Tables {
    words: Vec<(u32, Vec<u16>)>,
}

/// (address, entries) of every table the callbacks read.
const TABLES: [(u32, usize); 22] = [
    (0x800b_92dc, 8),
    (0x800b_92ec, 6),
    (0x800b_92f8, 31),
    (0x800b_937c, 14),
    (0x800b_9398, 11),
    (0x800b_93c4, 24),
    (0x800b_9400, 6),
    (0x800b_940c, 5),
    (0x800b_9418, 8),
    (0x800b_9438, 4),
    (0x800b_9440, 7),
    (0x800b_9450, 7),
    (0x800b_9460, 4),
    (0x800b_9468, 10),
    (0x800b_9480, 16),
    (0x800b_94a0, 3),
    (0x800b_94a8, 7),
    (0x800b_94b8, 4),
    (0x800b_94c0, 3),
    (0x800b_94c8, 2),
    (0x800b_94cc, 5),
    (0x800b_950c, 14),
];
const GOLF_TABLES: [(u32, usize); 4] = [(0x800b_9528, 19), (0x800b_9550, 11), (0x800b_9568, 11), (0x800b_9580, 12)];

impl Tables {
    pub fn load(disc: &GameDisc) -> Tables {
        let words = TABLES
            .iter()
            .chain(GOLF_TABLES.iter())
            .map(|&(at, n)| {
                let b = disc.exe_bytes(at, 2 * n).unwrap_or(&[]);
                (at, b.chunks_exact(2).map(|w| u16::from_le_bytes([w[0], w[1]])).collect())
            })
            .collect();
        Tables { words }
    }

    fn get(&self, at: u32, k: u32) -> u16 {
        self.words.iter().find(|t| t.0 == at).and_then(|t| t.1.get(k as usize)).copied().unwrap_or(0)
    }
}

/// The setup's countdown (0x800b9598: 0x80080308 900, 0x80080318 300
/// and the first-line flag).
pub fn setup(level: Level) -> (i32, bool) {
    match level.0 {
        7 | 17 => (300, true),
        2..=6 | 9..=13 | 15 | 16 | 23..=31 => (900, false),
        _ => (0, false),
    }
}

fn r(g: &mut Game) -> u32 {
    g.rand.next() & 255
}

/// One frame of the level's callback. `cue` is the script's state the
/// callback reads (see `Script::cue`).
pub fn frame(g: &mut Game, t: &Tables, cue: i32) {
    // Activity play picks the callback by the character (0x800818e0).
    let level = if g.activity {
        match g.character {
            1 => 25,
            2 => 26,
            3 => 31,
            4 => 23,
            5 => 29,
            _ => return,
        }
    } else {
        g.level.0
    };
    let dt = DT;
    let say = |g: &mut Game, n: u16| g.speech.push(n);
    match level {
        // 0x80080648: with +0x30 bit 8 sounds 32-35 or lines; the goose
        // walking its posts honks or says; else a line.
        2 => {
            g.ambience -= dt;
            if g.ambience > 0 {
                return;
            }
            let s = r(g);
            let goose = g.npcs.iter().position(|n| n.id == 11 && !n.hidden);
            if cue != 0 {
                if s & 1 != 0 {
                    let k = r(g);
                    say(g, t.get(0x800b_937c, k % 14));
                    g.ambience = (r(g) * 8) as i32;
                } else {
                    let e = 32 + (r(g) & 3) as u8;
                    g.play(e);
                }
            } else if let Some(k) = goose.filter(|&k| matches!(g.npcs[k].mode, Mode::Posts { .. } | Mode::Wait { .. }))
            {
                let _ = k;
                if s & 1 != 0 {
                    let k = r(g);
                    say(g, t.get(0x800b_93c4, k % 24));
                    g.ambience = (r(g) * 12 + 900) as i32;
                } else {
                    let e = 32 + (r(g) & 3) as u8;
                    g.play(e);
                }
            } else {
                say(g, t.get(0x800b_9398, s % 11));
                g.ambience = (r(g) * 16 + 1800) as i32;
            }
        }
        // 0x80080330: the pie toss, rooms 26 and 35, else 1087 + r % 22.
        3 => {
            g.ambience -= dt;
            if g.ambience > 0 {
                return;
            }
            let room = g.current_room();
            if cue != 0 {
                let k = g.rand.next() & 7;
                say(g, t.get(0x800b_92dc, k));
                g.ambience = 2000;
            } else if matches!(room, Some(26 | 35)) {
                let k = r(g);
                say(g, t.get(0x800b_92ec, k % 6));
                g.ambience = 2000;
            } else {
                let k = r(g);
                say(g, 1087 + (k % 22) as u16);
                g.ambience = 5000;
            }
        }
        // 0x80080934.
        4 => {
            g.ambience -= dt;
            if g.ambience < 0 {
                let k = r(g);
                say(g, t.get(0x800b_9400, k % 6));
                g.ambience += 1000;
            }
        }
        // 0x80080ba8: a robot attacking within 5000 (guess: one guarding
        // and within its range), floating, or else.
        5 => {
            g.ambience -= dt;
            let p = g.player.pos;
            let attacking = g.npcs.iter().any(|n| {
                n.character == 35 && !n.hidden && matches!(n.mode, Mode::Guard) && {
                    let (dx, dz) = ((n.pos[0] - p[0]) as i64, (n.pos[2] - p[2]) as i64);
                    dx * dx + dz * dz < 5000 * 5000 && dx * dx + dz * dz < (n.range as i64).pow(2)
                }
            });
            if g.ambience > 0 {
                return;
            }
            if attacking {
                let k = g.rand.next() & 3;
                say(g, t.get(0x800b_9438, k));
                g.ambience = (r(g) * 8) as i32;
            } else {
                let s = r(g);
                let line = if g.player.float.is_some() { t.get(0x800b_940c, s % 5) } else { t.get(0x800b_9418, s & 7) };
                say(g, line);
                g.ambience = (r(g) * 16 + 900) as i32;
            }
        }
        // 0x80080a20: Tommy near a guard, Tommy, or Chuckie.
        6 => {
            g.ambience -= dt;
            if g.ambience > 0 {
                return;
            }
            let p = g.player.pos;
            let guard = g.npcs.iter().any(|n| {
                !n.hidden && matches!(n.mode, Mode::Guard) && {
                    let (dx, dz) = ((n.pos[0] - p[0]) as i64, (n.pos[2] - p[2]) as i64);
                    let d = dx * dx + dz * dz;
                    d > 0 && d < 6000 * 6000
                }
            });
            match g.character {
                1 if guard => {
                    let k = g.rand.next() & 3;
                    say(g, t.get(0x800b_9460, k));
                    g.ambience = (r(g) * 8) as i32;
                }
                1 => {
                    let k = r(g);
                    say(g, t.get(0x800b_9440, k % 7));
                    g.ambience = (r(g) * 16 + 1800) as i32;
                }
                2 => {
                    let k = r(g);
                    say(g, t.get(0x800b_9450, k % 7));
                    g.ambience = (r(g) * 16 + 1800) as i32;
                }
                _ => {}
            }
        }
        // 0x80080cd8: 1181 first, then 1123.. or 1182.. lines.
        7 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let line = if std::mem::take(&mut g.ambience_first) {
                1181
            } else if g.rand.next() & 1 == 0 {
                let n = 1123 + (r(g) % 10) as u16;
                if n == 1134 { 1123 } else { n }
            } else {
                let v = (r(g) % 13) as u16;
                if matches!(v + 1182, 1183 | 1185) { v + 1183 } else { v + 1182 }
            };
            say(g, line);
            g.ambience += 1000;
        }
        // 0x80080e08: while deaths are to go and Mr. Friend's parts are
        // about.
        9 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let about = |id: u16| g.set.placements.iter().any(|p| p.kind == 1 && p.id == id) && !g.is_hidden(id);
            if cue > 0 && [46u16, 800, 91].into_iter().any(about) {
                let k = r(g);
                say(g, t.get(0x800b_9468, k % 10));
            }
            g.ambience = (r(g) * 16) as i32;
        }
        // 0x80080fe0: while 1492, 608 and 609 are about and none walks
        // posts.
        10 => {
            let ok = [1492u16, 608, 609]
                .iter()
                .all(|&id| g.npcs.iter().any(|n| n.id == id && !matches!(n.mode, Mode::Posts { .. })));
            if !ok {
                return;
            }
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let k = g.rand.next() & 15;
            let s = r(g);
            say(g, t.get(0x800b_9480, k));
            g.ambience = (s * 16 + 600) as i32;
        }
        // 0x800810c8: ring steps 9 and 5.
        11 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let line = match cue {
                9 => {
                    let k = r(g);
                    t.get(0x800b_94a0, k % 3)
                }
                5 => {
                    let k = r(g);
                    t.get(0x800b_94a8, k % 7)
                }
                _ => return,
            };
            say(g, line);
            g.ambience = (r(g) * 8) as i32;
        }
        // 0x800812c8: before and after the refrigerator (cine 67).
        12 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let line = if cue == 0 {
                let k = g.rand.next() & 1;
                t.get(0x800b_94c8, k)
            } else {
                let k = r(g);
                t.get(0x800b_94cc, k % 5)
            };
            g.ambience = (r(g) * 32) as i32;
            say(g, line);
        }
        // 0x800811c0.
        15 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let k = g.rand.next() & 3;
            say(g, t.get(0x800b_94b8, k));
            g.ambience = (r(g) * 8) as i32;
        }
        // 0x80081230.
        16 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let k = r(g);
            say(g, t.get(0x800b_94c0, k % 3));
            g.ambience = (r(g) * 8 + 1200) as i32;
        }
        // 0x800813a0: lines 0-4 of DB17's own table.
        17 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let a = r(g);
            let line = if std::mem::take(&mut g.ambience_first) {
                if g.character == 5 { a % 5 } else { a & 3 }
            } else if a & 1 != 0 {
                a & 3
            } else {
                a % 3
            };
            say(g, line as u16);
            g.ambience = (r(g) * 8 + 900) as i32;
        }
        // 0x8008046c (the catch is the script's).
        20 | 21 => {
            g.ambience -= dt;
            if g.ambience > 0 {
                return;
            }
            let k = r(g);
            say(g, t.get(0x800b_92f8, k % 31));
            g.ambience = (r(g) * 8) as i32;
        }
        // Golf (0x80081474, 0x80081524, 0x800815e0, 0x8008168c,
        // 0x80081758): a fixed line in the stroke (state 15), else one of
        // the baby's.
        23..=31 => {
            g.ambience -= dt;
            if g.ambience >= 0 {
                return;
            }
            let stroke = cue == 15;
            let (line, reload) = match level {
                25 | 27 | 30 => {
                    let k = r(g);
                    (if stroke { 270 } else { t.get(0x800b_950c, k % 14) }, 1800)
                }
                26 | 28 => {
                    let k = r(g);
                    (if stroke { 347 } else { t.get(0x800b_9528, k % 19) }, 1800)
                }
                24 | 31 => {
                    let k = r(g);
                    (if stroke { 598 } else { t.get(0x800b_9580, k % 12) }, 1800)
                }
                29 => {
                    let k = r(g);
                    let line = if stroke { if k & 1 != 0 { 820 } else { 868 } } else { t.get(0x800b_9568, k % 11) };
                    (line, 2700)
                }
                _ => {
                    let k = r(g);
                    let line = if stroke { if k & 1 != 0 { 949 } else { 995 } } else { t.get(0x800b_9550, k % 11) };
                    (line, 1800)
                }
            };
            say(g, line);
            g.ambience = (r(g) * 16) as i32 + reload;
        }
        _ => {}
    }
}
