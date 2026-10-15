//! Level 7, Reptar 2010 (docs/levels/level-07.md): Tommy as Reptar walks
//! the city to City Hall (room 6, cine 85). Buildings break as he brushes
//! them; Triangle chomps, Circle breathes fire, a jump's landing shakes
//! the camera. Cine 85 is the end of the game: the credits, the progress
//! cleared, the hub.

use crate::actors::player::Pose;
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::world::game::Game;

/// Where cine 79 leaves Reptar.
const START: [i32; 3] = [25162, -703, -9351];
/// How near Reptar's body breaks a building (seen 1670 .. 2240 from his
/// origin).
const BODY: i32 = 1700;
/// Template 1240 is removed outright; its sounds by template.
const GONE: u16 = 1240;
/// Taken by touch (0x8005ab60): templates 401 (sound 88) and 795 (91).
const TAKEN: [(u32, u8); 2] = [(401, 88), (795, 91)];
const TAKE_REACH: i64 = 1400;
/// Chomp (state 29, animation 49) and fire (state 30, animation 51), both
/// with sound 89.
const CHOMP: i16 = 49;
const FIRE: i16 = 51;

pub struct Reptar2010 {
    taken: Vec<usize>,
    was_air: bool,
    broken: u32,
}

impl Reptar2010 {
    /// EpisodeStart (0x8007b748).
    pub fn start(g: &mut Game, _disc: &GameDisc) -> Result<Reptar2010, String> {
        g.player.pos = START;
        g.player.gait = (10, 10);
        g.checkpoint = Some(Pose::new(START, g.player.heading));
        g.flags |= 512;
        Ok(Reptar2010 { taken: Vec::new(), was_air: false, broken: 0 })
    }

    /// The building think (0x8004d7c4): the body against its boxes.
    fn break_near(&mut self, g: &mut Game) {
        let p = g.player.pos;
        let feet = p[1] - g.player.feet();
        for k in 0..g.breakables.len() {
            let b = &g.breakables[k];
            if b.broken || b.gone {
                continue;
            }
            let Some(model) = g.set.models.get(b.shape) else { continue };
            let hit = model.boxes.iter().any(|&(lo, hi)| {
                let corner = |i: usize| {
                    let v = rrt::glam::Vec3::new(
                        if i & 1 == 0 { lo[0] } else { hi[0] } as f32,
                        if i & 2 == 0 { lo[1] } else { hi[1] } as f32,
                        if i & 4 == 0 { lo[2] } else { hi[2] } as f32,
                    );
                    b.rot * v + b.pos
                };
                let cs: Vec<_> = (0..8).map(corner).collect();
                let (mut a, mut z) = (cs[0], cs[0]);
                for c in &cs {
                    a = a.min(*c);
                    z = z.max(*c);
                }
                let inside = |v: f32, lo: f32, hi: f32| v > lo - BODY as f32 && v < hi + BODY as f32;
                inside(p[0] as f32, a.x, z.x)
                    && inside(p[2] as f32, a.z, z.z)
                    && (feet as f32) > a.y - 600.0
                    && (feet as f32 - 4000.0) < z.y
            });
            if hit {
                let t = b.template;
                let b = &mut g.breakables[k];
                if t == GONE {
                    b.gone = true;
                } else {
                    b.broken = true;
                }
                self.broken += 1;
                let sound = match t {
                    914 | 1165 => 96,
                    971 => 95,
                    1135 => 90,
                    _ => 0,
                };
                if sound != 0 {
                    g.play(sound);
                }
                if t == 931 {
                    g.speech.push(1183);
                }
                let sound = if g.rand.next() & 1 == 0 { 91 } else { 92 };
                g.play(sound);
            }
        }
    }
}

impl crate::world::game::Script for Reptar2010 {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        // Raw Triangle and Circle (0x8005ab60), in any state.
        if g.player.action.is_none() {
            if pads.pressed(0, input::TRIANGLE) {
                g.play(89);
                g.player.act(CHOMP, &g.anims);
            } else if pads.pressed(0, input::CIRCLE) {
                g.play(89);
                g.player.act(FIRE, &g.anims);
            }
        }
        // The landing's rumble (0x8005cba8 for char 9): sound 93.
        let air = matches!(g.player.state, crate::actors::player::State::Jump | crate::actors::player::State::Fall);
        if self.was_air && !air {
            g.play(93);
        }
        self.was_air = air;
        self.break_near(g);
        // Templates 401 and 795 taken by touch.
        let p = g.player.pos;
        for (k, q) in g.set.placements.clone().iter().enumerate() {
            let Some(&(_, sound)) = TAKEN.iter().find(|t| t.0 == q.template) else { continue };
            if q.kind != 1 || self.taken.contains(&k) {
                continue;
            }
            let (dx, dz) = ((q.pos[0] - p[0]) as i64, (q.pos[2] - p[2]) as i64);
            if dx * dx + dz * dz < TAKE_REACH * TAKE_REACH {
                self.taken.push(k);
                g.hide_placement(k, true);
                g.play(sound);
            }
        }
    }

    /// ForceCry case 6: no cry; the one try; the hub.
    fn cry_animation(&self) -> bool {
        false
    }

    fn describe(&self) -> String {
        format!("broken {} taken {}", self.broken, self.taken.len())
    }
}
