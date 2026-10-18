//! Level 12, Let There Be Light (docs/levels/level-12.md): Tommy with the
//! flashlight in the dark house. Holding Triangle shines it (1 charge a
//! frame); the light sends a Shadow monster away to another spot. Batteries
//! give 30. The refrigerator's cine 67 drops the light and puts Spike
//! somewhere in the house; finding him plays cine 68: won.

use rugrats_render::Vtx;

use crate::actors::npc::{Event, Mode, Target};
use crate::disc::GameDisc;
use crate::input::{self, Pads};
use crate::world::collide;
use crate::world::fx::{self, Mesh};
use crate::world::game::{self, Effect, Game, Script};

/// The flashlight (template 933): 120 charge at the start, 100 after a
/// pick-up; 1 a frame.
const LIGHT: u16 = 933;
const CHARGE: i32 = 120;
const CHARGE_AGAIN: i32 = 100;
const GAUGE: (i16, i16) = (-172, 73);
/// The light's shot (type 1, table 0x800bb28c entry 1): from hand slot
/// 15 turned (0, 0, 350) (+0x24, 0x80091834), its speed 100 (0x8005f4cc)
/// over the type's 100 (+0x18, 0x8009134c): the facing's unit, 4096, in
/// its one frame.
const SHOT_SLOT: usize = 15;
/// The hand part (hierarchy +0x23, slot 15)'s model: the hand holding the
/// light (cine 66's end, the pick-up 0x800602d8), or the bare hand (the
/// put-down 0x80060910, cine 67 and 68's start, 0x80072ef0).
const HAND_LIGHT: u32 = 1690;
const HAND_BARE: u32 = 1310;
const SHOT_FROM: i32 = 350;
const SHOT_SPEED: i32 = 100;
const SHOT_SCALE: i32 = 100;
/// The light mesh's width (0x80090e70: scale 15000).
const LIGHT_WIDTH: f32 = 15000.0;
/// Shadow monsters (3333-3340, template 934): guard 10000, reach 1024,
/// bite 20; light-struck, mode 10: their box gone, shrinking 250 a frame
/// (+0x36) while it has a size, then a group 10 spot.
const SHADOWS: [u16; 8] = [3333, 3334, 3335, 3336, 3337, 3338, 3339, 3340];
const SHADOW_REACH: i32 = 1024;
const SHRINK: i32 = 250;
/// The hierarchy's box (+0x04..+0x10), zeroed while struck.
const BOX: std::ops::Range<usize> = 4..0x12;
const SPOTS: usize = 10;
/// Batteries (kind 12): taken within 1000 by the trigger pass, +30.
const BATTERY: u16 = 12;
const BATTERY_REACH: f32 = 1000.0;
/// The refrigerator doors (cine 67), Spike (cine 68).
const DOORS: [u16; 2] = [601, 602];
const DOOR_REACH: i64 = 1500;
const SPIKE: u16 = 14;
const SPIKE_REACH: i64 = 1200;
const CINE_FRIDGE: usize = 67;
const CINE_WON: usize = 68;
/// Where cine 66 leaves Tommy.
const START: [i32; 3] = [29611, -5365, 15428];

pub struct LetThereBeLight {
    charge: i32,
    max: i32,
    /// Monsters struck (mode 10) and their hierarchy's box to put back.
    away: Vec<(usize, Vec<u8>)>,
    fridge_done: bool,
    spike_waiting: bool,
    shining: bool,
    /// This frame's shot: the hand's point and where it got to.
    shot: Option<([i32; 3], [i32; 3])>,
    mesh: Option<Mesh>,
}

fn find(g: &Game, id: u16) -> Option<usize> {
    g.npcs.iter().position(|n| n.id == id)
}

impl LetThereBeLight {
    /// Cine 66's end and EpisodeStart (0x8007b7c8).
    pub fn start(g: &mut Game, disc: &GameDisc) -> Result<LetThereBeLight, String> {
        g.hide(SPIKE, true);
        // The refrigerator's use is its cine, not the doors' swing.
        for id in DOORS {
            g.disable_use(id);
        }
        g.player.pos = START;
        g.player.heading = 1024;
        g.player.target = 1024;
        g.set_camera_yaw(1024.0);
        // The flashlight in hand as +0x1f0 alone (0x8007b7c8): no grip, the
        // arms as they were, the hand's model showing it.
        if let Some(k) = g.things.iter().position(|t| t.template == LIGHT) {
            g.hold(k);
        }
        hand(g);
        for id in SHADOWS {
            if let Some(k) = find(g, id) {
                let n = &mut g.npcs[k];
                n.mode = Mode::Guard;
                n.reach = SHADOW_REACH;
                n.bite_wait = 230;
                // On the trigger list (0x8007b7c8: +0x48).
                n.trigger = n.radius;
            }
        }
        let mesh = Mesh::load(disc, &fx::LIGHT).ok();
        Ok(LetThereBeLight {
            charge: CHARGE,
            max: CHARGE,
            away: Vec::new(),
            fridge_done: false,
            spike_waiting: false,
            shining: false,
            shot: None,
            mesh,
        })
    }

    fn holding(&self, g: &Game) -> bool {
        g.carried().is_some_and(|k| g.things[k].template == LIGHT)
    }

    /// The light (0x80059d3c -> 0x80091834, shot type 1): a shot from the
    /// hand toward the aim target or along the facing (`shot`), stepped
    /// once (0x8009134c) and freed after its draw. A Shadow
    /// monster its step meets first is struck (its +0x58, 0x800511a8 case
    /// 41): mode 10, its hierarchy's box zeroed, sound 110 at it.
    fn shine(&mut self, g: &mut Game) {
        self.shot = shot(g);
        let Some((from, to)) = self.shot else { return };
        if let Some(k) = g.shot_meets(from, to)
            && SHADOWS.contains(&g.npcs[k].id)
        {
            let n = &mut g.npcs[k];
            n.mode = Mode::Idle;
            let saved = n.hierarchy.header.get(BOX).map(<[u8]>::to_vec).unwrap_or_default();
            if let Some(b) = n.hierarchy.header.get_mut(BOX) {
                b.fill(0);
            }
            let at = n.pos;
            g.play_at(110, at);
            self.away.push((k, saved));
        }
    }
}

/// The hand part's model and the flashlight's own draw (+0x40 0 while
/// in hand: the hand's model shows it).
fn hand(g: &mut Game) {
    let held = g.carried().filter(|&k| g.things[k].template == LIGHT);
    for t in g.things.iter_mut().filter(|t| t.template == LIGHT) {
        t.hidden = held.is_some();
    }
    if let Some(k) = g.player.slot_part(SHOT_SLOT)
        && let Some(p) = g.player.hierarchy.parts.get_mut(k)
    {
        p.model = if held.is_some() { HAND_LIGHT } else { HAND_BARE };
    }
}

/// The light's shot: its start, hand slot 15's part as last drawn
/// (+0x158, 0x80044fec) turning (0, 0, 350) (0x8009fbe8 by the part's
/// matrix, transposed by 0x80010b54); its way (+0x200, 0x80059cdc), the
/// unit toward the aim target (+0x1f4, 0x80075e4c) while it has a size
/// (+0x36), else the facing's (0x80076040), made a unit again
/// (0x8009f278); its end that unit times (speed << 10) / 100, >> 10.
pub fn shot(g: &Game) -> Option<([i32; 3], [i32; 3])> {
    let (posed, _) = g.drawn_pose();
    let m = posed.get(g.player.slot_part(SHOT_SLOT)?)?;
    let from = [0, 1, 2].map(|r| m.joint[r] + ((m.rot[2][r] * SHOT_FROM) >> 12));
    let way = match g.aim_target.map(|t| &g.npcs[t]).filter(|n| !n.hidden && n.scale > 0) {
        Some(n) => [0, 1, 2].map(|k| n.pos[k] - g.player.pos[k]),
        None => [g.trig.sin(g.player.heading), 0, g.trig.cos(g.player.heading)],
    };
    let dir = collide::normalize(collide::unit(way));
    let step = (SHOT_SPEED << 10) / SHOT_SCALE;
    Some((from, [0, 1, 2].map(|k| from[k] + ((dir[k] * step) >> 10))))
}

impl Script for LetThereBeLight {
    fn frame(&mut self, g: &mut Game, pads: &Pads) {
        for e in std::mem::take(&mut g.npc_events) {
            if let Event::Bite { target: Target::Player, amount, .. } = e
                && amount > 0
            {
                g.post_damage(amount);
            }
        }
        // Shining (0x8005a6f4): Triangle held, 1 a frame, sound 161 on the
        // press.
        let holding = self.holding(g);
        self.shining = holding && pads.held(0, input::TRIANGLE) && self.charge > 0;
        if holding && pads.pressed(0, input::TRIANGLE) {
            g.play(161);
        }
        // Mode 10 (0x80084314, +0x688 bit 0x8000): the size down 250 a
        // frame while above 0; then speech 143 or 21 (rand & 1) and back
        // (0x80083be0 case 43): size 4096, the box put back, guarding from
        // a group 10 spot, not marked used.
        let mut back = Vec::new();
        self.away.retain(|(k, saved)| {
            let n = &mut g.npcs[*k];
            if n.scale > 0 {
                n.scale -= SHRINK;
                return true;
            }
            back.push((*k, saved.clone()));
            false
        });
        for (k, saved) in back {
            g.speech.push(if g.rand.next() & 1 == 0 { 143 } else { 21 });
            let n = &mut g.npcs[k];
            n.scale = 4096;
            if let Some(b) = n.hierarchy.header.get_mut(BOX) {
                b.copy_from_slice(&saved);
            }
            if let Some(i) = game::spawn(&mut g.spots, SPOTS, &mut g.rand) {
                g.spots[i].used = false;
                let at = g.spots[i].pos;
                let n = &mut g.npcs[k];
                n.place(at, &g.col);
                n.home = at;
            }
            g.npcs[k].mode = Mode::Guard;
        }
        if self.shining {
            self.charge -= 1;
            self.shine(g);
        }
        // Batteries within 1000 (0x8004cf58): +30, clamped to the gauge.
        let me = rrt::glam::Vec3::new(g.player.pos[0] as f32, g.player.pos[1] as f32, g.player.pos[2] as f32);
        for b in g.goodies.iter_mut().filter(|b| b.kind == BATTERY && !b.taken) {
            if (b.pos - me).length() < BATTERY_REACH {
                b.taken = true;
                if holding {
                    self.charge = (self.charge + 30).min(self.max);
                }
                g.effects.push(Effect::new(109));
            }
        }
        // Picked up again (0x800602d8): the gauge's max is 100.
        if holding && self.max == CHARGE && self.fridge_done {
            self.max = CHARGE_AGAIN;
        }
        // Triangle by the refrigerator (cine 67) or Spike (cine 68).
        if pads.pressed(0, input::TRIANGLE) {
            let p = g.player.pos;
            let near = |at: [i32; 3], r: i64| {
                let (dx, dz) = ((at[0] - p[0]) as i64, (at[2] - p[2]) as i64);
                dx * dx + dz * dz < r * r
            };
            if !self.fridge_done
                && g.set.placements.iter().any(|q| q.kind == 1 && DOORS.contains(&q.id) && near(q.pos, DOOR_REACH))
            {
                self.fridge_done = true;
                g.drop_carried();
                g.cine = Some(CINE_FRIDGE);
            } else if self.spike_waiting && find(g, SPIKE).is_some_and(|k| near(g.npcs[k].pos, SPIKE_REACH)) {
                self.spike_waiting = false;
                g.drop_carried();
                g.cine = Some(CINE_WON);
            }
        }
        hand(g);
    }

    fn cine_finished(&mut self, g: &mut Game, n: usize) {
        if n == CINE_FRIDGE {
            // 0x8007bb84 after card 705: Spike idle at a group 0 spot.
            if let Some(k) = find(g, SPIKE) {
                if let Some(i) = game::spawn(&mut g.spots, 0, &mut g.rand) {
                    let at = g.spots[i].pos;
                    g.npcs[k].place(at, &g.col);
                }
                g.hide(SPIKE, false);
                g.npcs[k].mode = Mode::Idle;
                self.spike_waiting = true;
            }
        }
    }

    /// The refrigerator opened (602's use gone).
    fn cue(&self) -> i32 {
        self.fridge_done as i32
    }

    fn draw(&self, g: &Game, moving: &mut Vec<Vtx>, overlay: &mut Vec<Vtx>) {
        // The light (0x80090e70): mesh 12/13 at the shot's midpoint, 15000
        // across and its length along, turned by 0x8008e6c8 of the way
        // (end less start), which sets its +z back at the hand.
        if let (true, Some(mesh), Some((from, to))) = (self.shining, &self.mesh, self.shot) {
            let v = |p: [i32; 3]| rrt::glam::Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32);
            mesh.along(v(to), v(from), LIGHT_WIDTH, moving);
        }
        // The gauge (0x8008c7f0: 360 over 361) while the light is held.
        if self.holding(g)
            && let Some(s) = &g.sprites
        {
            let top = 2.0 * 54.0 * (self.max - self.charge.min(self.max)).max(0) as f32 / self.max as f32;
            s.draw(361, GAUGE, 36.0, 54.0, 1024.0, top, g.focal(), overlay);
            s.draw(360, GAUGE, 36.0, 64.0, 1024.0, 0.0, g.focal(), overlay);
        }
    }

    fn describe(&self) -> String {
        format!(
            "charge {} away {} fridge {} spike {} shining {}",
            self.charge,
            self.away.len(),
            self.fridge_done,
            self.spike_waiting,
            self.shining
        )
    }
}
