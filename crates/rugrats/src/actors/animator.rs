//! A character's animation (SLUS_006.50:0x80056834 and the samplers
//! 0x80055170, 0x800552bc): four tracks (object +0xa8, +0xb4, +0xc0,
//! +0xcc), each playing its column of the state's entry, a later track
//! over the parts it keys; each frame blended from the one before
//! (0x800202ec).
//!
//! The tables by character (0x8008a42c, into object +0xe8 and +0xec): a
//! state's entry (s16 a state) and per entry four ANM ids (s16, -1 none).

use rugrats_data::anim::{Anims, Key};
use rugrats_data::hierarchy::Hierarchy;

use crate::disc::GameDisc;
use crate::gte::Trig;

/// dt a frame at 30 fps.
const DT: i32 = 10;
const REMAP_COUNT: usize = 132;

/// (entries, remap) by character (0x8008a42c's cases, character - 2).
fn tables(character: u16, level: u8) -> (u32, u32) {
    match character {
        2 => (0x800b_a9e4, 0x800b_9dbc),
        3 | 37 => (0x800b_ab6c, 0x800b_9e40),
        4 | 5 => (0x800b_a5fc, 0x800b_9d38),
        6 => (0x800b_ac9c, 0x800b_9ec4),
        // Reptar: the golf holes' own table in levels 23-31 (hole 26's
        // Reptar, id 9999).
        9 if (23..32).contains(&level) => (0x800c_4584, 0x800b_a158),
        9 => (0x800b_a754, 0x800b_a0d4),
        43 => (0x800c_458c, 0x800b_a4f4),
        10 | 11 => (0x800b_a7e4, 0x800b_9fcc),
        12 | 21 => (0x800b_ad8c, 0x800b_9f48),
        16 | 47 => (0x800b_a714, 0x800b_a2e4),
        22 => (0x800b_ae3c, 0x800b_a578),
        23 => (0x800b_a80c, 0x800b_a050),
        33 => (0x800b_a6fc, 0x800b_a1dc),
        34 => (0x800b_ae0c, 0x800b_a368),
        35 => (0x800b_add4, 0x800b_a260),
        39 => (0x800b_ae2c, 0x800b_a470),
        41 => (0x800b_ae1c, 0x800b_a3ec),
        _ => (0x800b_a824, 0x800b_9cb4),
    }
}

/// The table state of logical animation `a` (0x80056fe4), which every
/// 0x800572a8 call goes through: the numbers the game's code asks for
/// are not the table's. `climb` is the height a climb rises (+0x144),
/// which picks one of six climbs. None keeps the state playing (17, the
/// idle's "as you were"), or names none (a climb over 1033).
pub fn table_state(a: i16, climb: i32, current: i16, running: bool) -> Option<i16> {
    const MAP: [i16; 54] = [
        0, 1, 3, 5, 8, -1, 32, 27, 30, 31, 33, 20, 22, 15, 16, 7, 60, -1, 26, 25, 19, 17, 34, 36, 37, 38, 39, 40, 41,
        42, 43, 44, 0, 45, 46, 47, 48, 49, 0, 50, 51, 52, 53, 54, 0, 55, 56, 57, 58, 59, 61, 62, 64, 63,
    ];
    match a {
        // 0x80056e44: the rise's band, 128 wide from under 394 to 1033.
        5 => {
            let h = climb.abs();
            (0..6).find(|&k| h < 394 + 128 * k).map(|k| 9 + k as i16)
        }
        17 => Some(current),
        // 0x80056e10: 28 when 0x800590d4 says running (the bump after a
        // run), else 32.
        6 => Some(if running { 28 } else { 32 }),
        0..=53 => Some(MAP[a as usize]),
        _ => Some(0),
    }
}

/// The logical animation table state `s` plays for (for tools giving a
/// body another's state): a second half (2, 4, 6, 18, 35, 46) as its
/// first, a fidget (23, 24) as standing.
pub fn logical_of(s: i16) -> Option<i16> {
    let first = match s {
        23 | 24 => return Some(0),
        2 | 4 | 6 | 18 | 35 | 46 => s - 1,
        _ => s,
    };
    (0..54).find(|&a| !matches!(a, 5 | 6 | 17) && table_state(a, 0, first, false) == Some(first))
}

/// Frame `k`'s root key, scaled by `lo` (4096 = 1, toward zero) if given.
fn root_key(anims: &Anims, k: usize, lo: Option<i32>) -> Option<[i32; 3]> {
    let r = anims.frames.get(k).and_then(|f| {
        anims.keys_of(f).iter().find_map(|key| match *key {
            Key::Lift { dx, dy, dz, .. } => Some([dx as i32, dy as i32, dz as i32]),
            _ => None,
        })
    })?;
    Some(match lo {
        Some(lo) => r.map(|c| {
            let p = c * lo;
            if p < 0 { (p + 4095) >> 12 } else { p >> 12 }
        }),
        None => r,
    })
}

/// What a state does at its last frame (0x800b9c30, by table state; read
/// by 0x8005562c): 4 turns back (ping-pong), 2 loops, 1 completes into the
/// next state (0x80056484), else it holds the last frame (8 or nothing).
/// 32 moves the body across by its root (0x80056834, `RootCurve::across`);
/// 64 takes the tracks over a state on them (0x80056708).
const ENDS: [u16; 66] = [
    1, 1, 1, 1, 1, 33, 40, 8, 1, 33, 33, 33, 33, 33, 33, 33, 97, 1, 1, 2, 33, 2, 33, 1, 1, 1, 1, 1, 33, 33, 33, 34, 33,
    4, 1, 1, 1, 33, 33, 33, 2, 40, 33, 2, 1, 1, 8, 2, 1, 1, 1, 1, 33, 1, 33, 1, 2, 1, 1, 1, 8, 1, 1, 1, 1, 0,
];

/// The state a completed one goes on to (0x80056484): the halves of the
/// walk (1, 2), the run (3, 4) and the idle (17, 18) and others in turn,
/// the jump's rise into its fall (5, 6). None goes back to the state
/// asked for after (+0xe6, logical 0: standing).
fn next(state: i16) -> Option<i16> {
    match state {
        1 => Some(2),
        2 => Some(1),
        3 => Some(4),
        4 => Some(3),
        5 => Some(6),
        17 => Some(18),
        18 => Some(17),
        34 => Some(35),
        35 => Some(34),
        39 | 58 => Some(40),
        45 => Some(46),
        _ => None,
    }
}

/// The sound a body makes entering table state `state` (0x800562bc, at
/// its position): the footsteps of the walk's halves (1, 2) for Spike in
/// levels 9 and 14 (132) and characters 47 (166), 33 (164) and 22 (165);
/// 6 for a jump's fall and the moving jump (6, 41); 11 landing (8).
pub fn entry_sound(state: i16, character: usize, level: u8) -> Option<u8> {
    match state {
        1 | 2 => match character {
            16 if matches!(level, 9 | 14) => Some(132),
            47 => Some(166),
            33 => Some(164),
            22 => Some(165),
            _ => None,
        },
        6 | 41 => Some(6),
        8 => Some(11),
        _ => None,
    }
}

/// The idle's fidgets, in turn, and how long standing still (state 0)
/// comes before one (0x800563bc: 1200 dt, set again in any other state).
const FIDGETS: (i16, i16) = (23, 24);
const FIDGET_AFTER: i32 = 1200;

/// The upper states track 1 keeps over the base (0x80056708: the carry's
/// arms 19, the throw 26, and 50, 51, 53); any other there follows the
/// base.
const UPPERS: [i16; 5] = [19, 26, 50, 51, 53];

/// The logical "as you were" (0x8005678c): the upper track, done, goes
/// back to following the base.
const AS_YOU_WERE: u8 = 17;

/// One track as the original keeps it (12 bytes from +0xa8): its table
/// state (-1 none), the frame before and the frame showing (indices into
/// the frames, -1 none), the time left in the frame showing, its phase
/// (+0xa: 0 to start, 1 playing, 2 done, 3 held on its last frame) and
/// whether a ping-pong runs back (+0xb).
#[derive(Clone, Copy, Debug)]
struct Track {
    state: i16,
    prev: i16,
    cur: i16,
    left: i16,
    phase: u8,
    back: u8,
}

impl Default for Track {
    fn default() -> Track {
        Track { state: -1, prev: -1, cur: -1, left: 0, phase: 0, back: 0 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Animator {
    tracks: [Track; 4],
    remap: Vec<i16>,
    entries: Vec<[i16; 4]>,
    /// The logical animation last asked for, and the climb's rise for
    /// the next climb (logical 5).
    asked: i16,
    /// The logical animation last asked for on the base track and on track
    /// 1 (an upper state's ask leaves the base's as it was).
    asked_on: [i16; 2],
    pub climb: i32,
    /// Running as 0x800590d4 judges it (above 3933/4096 of the top speed)
    /// when the body asks: the bump after a run is state 28, else 32.
    pub running: bool,
    /// The table states entered on the base track since last taken (for
    /// their sounds, 0x800562bc).
    entered: Vec<i16>,
    /// The idle's fidget to come and the time standing still before it
    /// (+0xf0, +0xf2).
    fidget: (i16, i32),
    /// The logical states asked for after the base completes (+0xe6) and
    /// after an upper state completes (+0xe7, 17 "as you were": back to
    /// the base), 0x80057244.
    after_base: u8,
    after_upper: u8,
    /// The parts' rotations as the samplers leave them (each part's +0x38
    /// in the original), by channel: kept from frame to frame, a part a
    /// frame does not key keeping its last.
    pose: Vec<Option<[i32; 4]>>,
    /// The flags of this step (+0xe0, cleared each step): the event bits
    /// of the frames that ended (FRM +0x00, 0x80055d40: 16 a throw's
    /// release, 2048 a grab, 0x2000 a hurt state's wait, 1 an animation's
    /// last frame), 0x8000000 the base completed, 0x10000000 an upper.
    pub events: u32,
    /// The heights a set up owes the body, by channel (0x80055170: each
    /// column's first frame's lift key, as it is, added to +0x1c), until
    /// taken.
    lift: Vec<(u8, i32)>,
    /// The speed its tracks run at (+0xe4, 4096 as written): each
    /// 0x800572a8 call sets it from its third argument, clamped to 2048 ..
    /// 16384 (0x80057214); a frame's time runs down by dt times it >> 12
    /// (0x80056834).
    speed: i32,
    /// The speed a state's entry asks for (0x8005a068: 4096, or 2048 for
    /// a body of template 8, Reptar).
    pub written_speed: i32,
    /// The character whose tables these are (the body's +0x15c).
    pub character: u16,
    /// Each track's column's frames (first, count) as of the last step.
    spans: [Option<(i16, i16)>; 4],
    /// Whether the base track began a state in the last step (asked, or
    /// gone on to its next).
    began: bool,
}

impl Animator {
    /// Character `character`'s tables.
    pub fn load(disc: &GameDisc, character: u16) -> Result<Animator, String> {
        Self::load_in(disc, character, 0)
    }

    /// [`Self::load`] for a body in level `level` (0x8008a42c picks some
    /// characters' tables by the level).
    pub fn load_in(disc: &GameDisc, character: u16, level: u8) -> Result<Animator, String> {
        let (entries_at, remap_at) = tables(character, level);
        let s16s = |addr: u32, n: usize| -> Result<Vec<i16>, String> {
            Ok(disc
                .exe_bytes(addr, 2 * n)?
                .as_chunks::<2>()
                .0
                .iter()
                .map(|h| i16::from_le_bytes([h[0], h[1]]))
                .collect())
        };
        let remap = s16s(remap_at, REMAP_COUNT)?;
        let count = remap.iter().copied().filter(|&e| (0..256).contains(&e)).max().unwrap_or(0) as usize + 1;
        let entries =
            s16s(entries_at, 4 * count)?.as_chunks::<4>().0.iter().map(|e| [e[0], e[1], e[2], e[3]]).collect();
        Ok(Animator {
            tracks: [Track::default(); 4],
            remap,
            entries,
            asked: 0,
            asked_on: [-1; 2],
            climb: 0,
            running: false,
            entered: Vec::new(),
            fidget: (FIDGETS.0, FIDGET_AFTER),
            after_base: 0,
            after_upper: AS_YOU_WERE,
            pose: vec![None; 32],
            events: 0,
            lift: Vec::new(),
            speed: 4096,
            written_speed: 4096,
            character,
            spans: [None; 4],
            began: false,
        })
    }

    /// State `a` plays `ids` on its four tracks (0x8008a668).
    pub fn set_state(&mut self, a: i16, ids: [i16; 4]) {
        if let Some(r) = self.remap.get_mut(a as usize) {
            *r = self.entries.len() as i16;
            self.entries.push(ids);
        }
    }

    /// The ANM animation of table state `s`'s column `k` (-1 none).
    fn column(&self, s: i16, k: usize) -> Option<usize> {
        let e = *self.remap.get(usize::try_from(s).ok()?)?;
        let id = self.entries.get(usize::try_from(e).ok()?)?[k];
        (id >= 0).then_some(id as usize)
    }

    /// The frames of table state `s`'s column `k`: first and count
    /// (D_800c4594).
    fn span(&self, s: i16, k: usize, anims: &Anims) -> Option<(i16, i16)> {
        let (first, count) = anims.index.get(self.column(s, k)?).copied().flatten()?;
        Some((first as i16, count as i16))
    }

    /// The base track: (state, first frame, frames, frame showing, time
    /// left).
    pub fn track(&self) -> (i16, usize, usize, usize, i32) {
        self.track_parts(0)
    }

    fn track_parts(&self, k: usize) -> (i16, usize, usize, usize, i32) {
        let t = &self.tracks[k];
        let (first, count) = self.spans.get(k).copied().flatten().unwrap_or((0, 0));
        (t.state, first as usize, count as usize, t.cur.max(0) as usize, t.left as i32)
    }

    /// The tracks as the original lays them out (+0xa8, 12 bytes each):
    /// state, frame before, frame showing, time left, 0, and the phase in
    /// the low byte and the way back in the high.
    pub fn record(&self) -> [[i16; 6]; 4] {
        self.tracks.map(|t| [t.state, t.prev, t.cur, t.left, 0, (t.phase as u16 | (t.back as u16) << 8) as i16])
    }

    /// Takes on another body's state as the original keeps it (for the
    /// oracle's checks): the track records (+0xa8, as [`Self::record`]),
    /// the speed (+0xe4), the states after (+0xe6, +0xe7), the fidget
    /// (+0xf0, +0xf2) and each channel's kept quaternion.
    #[allow(clippy::too_many_arguments)]
    pub fn load_record(
        &mut self,
        tracks: [[i16; 6]; 4],
        speed: i32,
        after: (u8, u8),
        fidget: (i16, i32),
        pose: Vec<Option<[i32; 4]>>,
        anims: &Anims,
    ) {
        self.tracks = tracks.map(|t| Track { state: t[0], prev: t[1], cur: t[2], left: t[3], phase: t[5] as u8, back: (t[5] as u16 >> 8) as u8 });
        self.speed = speed;
        (self.after_base, self.after_upper) = after;
        self.fidget = fidget;
        self.pose = pose;
        // (Standing: the idle asked last on the base.)
        self.asked = 0;
        self.asked_on = [0, -1];
        self.spans = std::array::from_fn(|k| self.span(self.tracks[k].state, k, anims));
    }

    /// The height the last set up owes the body (0x80055170), taken: the
    /// lift keys on channels `hierarchy` gives a part (+0x14, not 0xff).
    pub fn take_lift(&mut self, hierarchy: &Hierarchy) -> i32 {
        std::mem::take(&mut self.lift)
            .into_iter()
            .filter(|&(c, _)| hierarchy.header.get(0x14 + c as usize).is_some_and(|&p| p != 0xff))
            .map(|(_, dy)| dy)
            .sum()
    }

    /// Marks logical animation `a` as asked last on its track, without
    /// asking (for a body whose state was taken on, `load_record`).
    pub fn mark_asked(&mut self, a: i16) {
        if let Some(s) = table_state(a, self.climb, self.tracks[0].state, self.running) {
            self.asked = a;
            self.asked_on[self.track_of(s)] = a;
        }
    }

    /// The tracks in words, for tools.
    pub fn describe(&self) -> String {
        let t: Vec<String> =
            self.tracks.iter().map(|t| format!("{}:{}<{} {} p{}{}", t.state, t.cur, t.prev, t.left, t.phase, if t.back != 0 { " back" } else { "" })).collect();
        format!("[{}]", t.join(" | "))
    }

    /// Whether the base state's root keys move the body across (its word
    /// has 32: 0x80056834 adds x and z into +0xd8 and +0xdc only then).
    pub fn moves_across(&self) -> bool {
        ENDS.get(self.tracks[0].state as usize).is_some_and(|e| e & 32 != 0)
    }

    /// Whether the base track began a state in the last step: asked for
    /// (again, it starts again) or gone on to its next.
    pub fn began(&self) -> bool {
        self.began
    }

    /// The logical animation last asked for.
    pub fn asked(&self) -> i16 {
        self.asked
    }

    /// The track logical animation `a` plays on (0x800571d8): the base
    /// when its state has a base column, else track 1 over it.
    fn track_of(&self, s: i16) -> usize {
        if self.column(s, 0).is_some() { 0 } else { 1 }
    }

    /// Whether logical animation `a` plays and has not finished: asked
    /// last, on its track, not held on its last frame.
    pub fn playing(&self, a: i16) -> bool {
        let Some(s) = table_state(a, self.climb, self.tracks[0].state, self.running) else { return false };
        let k = self.track_of(s);
        let t = &self.tracks[k];
        self.asked_on[k] == a && t.state == s && t.phase != 3
    }

    /// The ANM animation of logical animation `a` on the base track
    /// (0x80055f7c: -1 none).
    pub fn anm(&self, a: i16) -> Option<usize> {
        self.column(table_state(a, self.climb, self.tracks[0].state, self.running)?, 0)
    }

    /// The logical state asked for after the base (`base`) or an upper
    /// state completes (0x80057244); asking for 0 resets the fidget.
    pub fn set_after(&mut self, logical: u8, base: bool) {
        if base {
            self.after_base = logical;
        } else {
            self.after_upper = logical;
        }
        if logical == 0 {
            self.fidget = (FIDGETS.0, FIDGET_AFTER);
        }
    }

    /// Asks for logical animation `a` (0x800572a8), unless it is the one
    /// last asked for: the game asks on entering a state, and what it asked
    /// for then runs on through its states.
    pub fn set(&mut self, a: i16, anims: &Anims) {
        if self.tracks[0].state == -1 {
            self.start(a, anims);
            return;
        }
        let Some(s) = table_state(a, self.climb, self.tracks[0].state, self.running) else { return };
        let k = self.track_of(s);
        if self.asked_on[k] == a {
            return;
        }
        // An upper state its track has gone on to by itself (+0xe7's,
        // 0x8005678c) is not asked again; a base state new to the base
        // track always is (a state's entry asks, 0x8005b2d8, and its first
        // frame begins anew).
        if k == 1 && self.tracks[1].state == s && self.tracks[1].phase == 1 {
            self.asked = a;
            self.asked_on[1] = a;
            return;
        }
        self.start(a, anims);
    }

    /// Standing, not turning (0x8005d11c): a base on any state but the
    /// idle, a fidget (23, 24) or 60 asks the state after the base (+0xe6)
    /// again from its first frame (0x80057274), each frame.
    pub fn stand(&mut self, anims: &Anims) {
        if !matches!(self.tracks[0].state, 0 | 23 | 24 | 60) {
            self.ask(self.after_base as i16, 4096, anims);
        }
    }

    /// Starts logical animation `a` from its first frame at speed
    /// `speed` (0x800572a8's third argument, 4096 as written).
    pub fn start_at(&mut self, a: i16, speed: i32, anims: &Anims) {
        self.ask(a, speed, anims);
    }

    /// Starts logical animation `a` from its first frame, even if it is
    /// playing, at the written speed.
    pub fn start(&mut self, a: i16, anims: &Anims) {
        let speed = if self.written_speed > 0 { self.written_speed } else { 4096 };
        self.ask(a, speed, anims);
    }

    /// 0x800572a8: the state's track takes the state, to start at the
    /// next step; the speed is set. A body never asked before has its
    /// tracks set up first (0x80055f7c).
    fn ask(&mut self, a: i16, speed: i32, anims: &Anims) {
        self.speed = speed.clamp(2048, 0x4000);
        let Some(s) = table_state(a, self.climb, self.tracks[0].state, self.running) else { return };
        if self.tracks[0].state == -1 {
            self.init(s, anims);
        }
        self.asked = a;
        let k = self.track_of(s);
        self.asked_on[k] = a;
        let t = &mut self.tracks[k];
        t.state = s;
        t.phase = 0;
        t.back = 0;
    }

    /// Its tracks set up on table state `s` without asking (0x80055f7c):
    /// what was asked last stays asked.
    pub fn set_up(&mut self, s: i16, anims: &Anims) {
        self.init(s, anims);
    }

    /// The tracks set up on state `s` (0x80055f7c, 0x80055ed8): each
    /// column's track at the second frame from the first, the first's
    /// rotations taken as they are and its lift owed to the body
    /// (0x80055170); a column without an animation leaves its track empty.
    fn init(&mut self, s: i16, anims: &Anims) {
        for k in 0..4 {
            let Some((first, _)) = self.span(s, k, anims) else {
                self.tracks[k].state = -1;
                continue;
            };
            let cur = first + 1;
            self.tracks[k] = Track { state: s, prev: first, cur, left: duration(anims, cur), phase: 1, back: 0 };
            if let Some(f) = anims.frames.get(first as usize) {
                for key in anims.keys_of(f) {
                    match *key {
                        Key::Rotation { channel, q } => {
                            if let Some(slot) = self.pose.get_mut(channel as usize) {
                                *slot = Some(q.map(|v| v as i32));
                            }
                        }
                        Key::Lift { channel, dy, .. } => self.lift.push((channel, dy as i32)),
                    }
                }
            }
        }
        self.spans = std::array::from_fn(|k| self.span(self.tracks[k].state, k, anims));
    }

    /// The table states entered since the last call.
    pub fn take_entered(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.entered)
    }

    /// A track starts its state (0x80055560): a state past 64 is 0; the
    /// frame showing is the column's first (its last running back), the
    /// frame before kept, so the first blends from what showed.
    fn begin(&mut self, k: usize, anims: &Anims) {
        if self.tracks[k].state as u16 >= 65 {
            self.tracks[k].state = 0;
        }
        let t = self.tracks[k];
        let Some((first, count)) = self.span(t.state, k, anims) else { return };
        let cur = if t.back == 0 { first } else { first + count - 1 };
        self.tracks[k].cur = cur;
        self.tracks[k].left = duration(anims, cur);
    }

    /// The base track's start (0x800562bc): playing, its state entered.
    fn begin_base(&mut self, anims: &Anims) {
        self.began = true;
        self.tracks[0].phase = 1;
        if self.entered.len() < 8 {
            self.entered.push(self.tracks[0].state);
        }
        self.begin(0, anims);
    }

    fn begin_upper(&mut self, anims: &Anims) {
        self.tracks[1].phase = 1;
        self.begin(1, anims);
    }

    /// Track `k` spends `time` (0x80055d40): held, it spends it all and
    /// gives the frame before's events; the first frame of its animation
    /// after a shorter one first loses the ratio of their lengths; the
    /// time left in the frame runs down, and a frame run out gives its
    /// events.
    fn consume(&mut self, k: usize, time: &mut i16, anims: &Anims) {
        let t = self.tracks[k];
        let frame = |i: i16| anims.frames.get(i as usize);
        if t.phase == 3 {
            *time = 0;
            self.events |= frame(t.prev).map_or(0, |f| f.events as u32);
            return;
        }
        let first = self.span(t.state, k, anims).map_or(0, |s| s.0);
        let cur_dur = frame(t.cur).map_or(0, |f| f.duration as i16).max(1);
        let prev_dur = frame(t.prev).map_or(0, |f| f.duration as i16);
        let mut left = t.left;
        if prev_dur != 0 && t.cur == first && prev_dur < cur_dur {
            left = (left - cur_dur / prev_dur).max(0);
        }
        if left < *time {
            *time = (*time - left).max(0);
            left = 0;
        } else {
            left = (left - *time).max(0);
            *time = 0;
        }
        self.tracks[k].left = left;
        if left == 0 {
            self.events |= frame(t.cur).map_or(0, |f| f.events as u32);
        }
    }

    /// Track `k` goes on to its next frame (0x8005562c): the frame showing
    /// becomes the one before; at the column's end its state's word says
    /// what follows: 4 turns back (ping-pong), 2 loops, 1 completes (the
    /// phase 2, the frame none), 8 holds (phase 3).
    fn advance(&mut self, k: usize, anims: &Anims) {
        let t = self.tracks[k];
        let Some((first, count)) = self.span(t.state, k, anims) else { return };
        let last = first + count - 1;
        let ends = ENDS.get(t.state as usize).copied().unwrap_or(1);
        let mut n = self.tracks[k];
        n.prev = t.cur;
        if t.back == 0 {
            if t.cur < last {
                n.cur = t.cur + 1;
            } else if ends & 4 != 0 {
                n.back = 1;
                n.cur = if count == 1 { first } else { first + count - 2 };
            } else if ends & 2 != 0 {
                n.cur = first;
            } else if ends & 1 != 0 {
                n.phase = 2;
                n.cur = -1;
            } else if ends & 8 != 0 {
                n.phase = 3;
            }
        } else if first < t.cur {
            n.cur = t.cur - 1;
        } else if ends & 4 != 0 {
            n.back = 0;
            n.cur = if count == 1 { first } else { first + 1 };
        } else if ends & 2 != 0 {
            n.cur = last;
        } else if ends & 1 != 0 {
            n.phase = 2;
            n.cur = -1;
        } else if ends & 8 != 0 {
            n.phase = 3;
        }
        n.left = duration(anims, n.cur);
        self.tracks[k] = n;
    }

    /// The base completes (0x80056484): the walk's and run's halves, the
    /// idle's and others go on in turn, the jump's rise into its fall;
    /// the rest go to the state asked for after (+0xe6). A held state
    /// (8) does not complete.
    fn complete(&mut self) {
        self.events |= 0x800_0000;
        let s = self.tracks[0].state;
        let ends = ENDS.get(s as usize).copied().unwrap_or(1);
        if ends & 6 == 0 && ends & 8 != 0 {
            return;
        }
        let to = match next(s) {
            Some(n) => n,
            None => match table_state(self.after_base as i16, self.climb, s, self.running) {
                Some(n) => n,
                None => return,
            },
        };
        let t = &mut self.tracks[0];
        t.state = to;
        t.phase = 0;
        t.back = 0;
    }

    /// An upper state completes (0x8005678c): track 1 takes the state
    /// asked for after (+0xe7), or, as you were, follows the base again
    /// (true).
    fn complete_upper(&mut self) -> bool {
        self.events |= 0x1000_0000;
        if self.after_upper == AS_YOU_WERE {
            return true;
        }
        if let Some(s) = table_state(self.after_upper as i16, self.climb, self.tracks[0].state, self.running) {
            let t = &mut self.tracks[1];
            t.state = s;
            t.phase = 0;
            t.back = 0;
        }
        false
    }

    /// Whether track 1 follows the base (0x80056708): always under a base
    /// state of flag 64, else unless it holds an upper state (which keeps
    /// the fidget away).
    fn follows(&mut self) -> bool {
        if ENDS.get(self.tracks[0].state as usize).is_some_and(|e| e & 64 != 0) {
            return true;
        }
        if UPPERS.contains(&self.tracks[1].state) {
            self.fidget.1 = FIDGET_AFTER;
            return false;
        }
        true
    }

    /// Track `k` (1-3) takes up the place of the track it follows
    /// (0x800561b0, 0x80056088): track 1 the base; 2 and 3 track 1 where
    /// its state has their column, else the base. It takes that track's
    /// state, the same frame of its own column, its time and phase (a
    /// held one's last frame, playing).
    fn sync(&mut self, k: usize, anims: &Anims) {
        if self.column(self.tracks[k].state, k).is_none() {
            return;
        }
        let from = if k >= 2 && self.column(self.tracks[1].state, k).is_some() { 1 } else { 0 };
        let src = self.tracks[from];
        self.tracks[k].state = src.state;
        let Some((src_first, src_count)) = self.span(src.state, from, anims) else { return };
        let at = if src.phase == 3 { src_count - 1 } else { src.cur - src_first };
        self.tracks[k].back = src.back;
        let first = self.span(src.state, k, anims).map_or(0, |s| s.0);
        let cur = first + at;
        self.tracks[k].cur = cur;
        if src.phase == 3 {
            self.tracks[k].left = duration(anims, cur);
            self.tracks[k].phase = 1;
        } else {
            self.tracks[k].left = src.left;
            self.tracks[k].phase = src.phase;
        }
    }

    /// Standing still (state 0) runs down to a fidget, 23 and 24 in turn
    /// (0x800563bc); any other state puts it off again.
    fn fidget(&mut self, anims: &Anims) {
        let s = self.tracks[0].state;
        if s != 0 {
            if (23..25).contains(&s) {
                self.events |= 0x2000_0000;
            }
            self.fidget.1 = FIDGET_AFTER;
            return;
        }
        self.events |= 0x2000_0000;
        self.fidget.1 -= DT;
        if self.fidget.1 > 0 {
            return;
        }
        self.fidget.1 = 0;
        let f = self.fidget.0;
        if f == FIDGETS.0 || f == FIDGETS.1 {
            self.fidget.0 = if f == FIDGETS.0 { FIDGETS.1 } else { FIDGETS.0 };
            // A body without the fidget keeps standing (the port's guard:
            // the original would play nothing).
            if self.span(f, 0, anims).is_some() {
                let t = &mut self.tracks[0];
                t.state = f;
                t.phase = 0;
                t.back = 0;
            }
        }
    }

    /// One frame (dt) of the tracks (0x80056834): each runs down its time
    /// (dt times the speed), going on frame by frame as the base, track 1
    /// and those following them say, then each samples its frame into the
    /// pose.
    pub fn step(&mut self, anims: &Anims, trig: &Trig) {
        self.events = 0;
        self.began = false;
        let speed = self.speed;
        let time = |t: &Track| if t.state == -1 { 0 } else { (DT * speed / 4096) as i16 };
        let mut times = self.tracks.map(|t| time(&t));
        // (A guard the original does without: a track whose column is
        // missing would never spend its time.)
        let mut guard = 0;
        while times.iter().any(|&t| t != 0) {
            guard += 1;
            if guard > 64 {
                break;
            }
            // The base.
            if self.tracks[0].phase == 0 {
                self.begin_base(anims);
            }
            if times[0] != 0 {
                self.consume(0, &mut times[0], anims);
                if self.tracks[0].phase == 1 && self.tracks[0].left == 0 {
                    self.advance(0, anims);
                    if self.tracks[0].phase == 2 {
                        self.complete();
                        if self.tracks[0].phase == 0 {
                            self.begin_base(anims);
                        }
                    }
                }
                self.fidget(anims);
                if self.tracks[0].phase == 0 {
                    self.begin_base(anims);
                }
            }
            // Track 1: its own upper state, or the base's.
            if self.tracks[1].phase == 0 {
                self.begin_upper(anims);
            }
            if times[1] != 0 {
                self.consume(1, &mut times[1], anims);
                if self.tracks[1].phase == 1 && self.tracks[1].left == 0 {
                    self.advance(1, anims);
                    if self.tracks[1].phase == 2 {
                        if self.complete_upper() {
                            if self.tracks[1].state == self.tracks[0].state && self.tracks[0].phase == 3 {
                                self.tracks[1].phase = 3;
                            }
                            self.sync(1, anims);
                        }
                        if self.tracks[1].phase == 0 {
                            self.begin_upper(anims);
                        }
                    }
                }
                if self.follows() && self.tracks[1].state != self.tracks[0].state {
                    self.sync(1, anims);
                }
                if self.tracks[1].phase == 0 {
                    self.begin_upper(anims);
                }
            }
            // Tracks 2 and 3 follow.
            for k in 2..4 {
                if times[k] != 0 {
                    self.consume(k, &mut times[k], anims);
                    if self.tracks[k].phase == 1 && self.tracks[k].left == 0 {
                        self.tracks[k].prev = self.tracks[k].cur;
                    }
                    self.sync(k, anims);
                }
            }
        }
        for k in 0..4 {
            if self.tracks[k].state != -1 {
                self.sample(k, anims, trig);
            }
        }
        self.spans = std::array::from_fn(|k| self.span(self.tracks[k].state, k, anims));
    }

    /// Track `k`'s frame into the pose (0x800552bc): each part the frame
    /// showing turns, from its turn in the frame before (a part the frame
    /// before does not key is left as it was), by the time gone in the
    /// frame (0x800202ec).
    fn sample(&mut self, k: usize, anims: &Anims, trig: &Trig) {
        let t = self.tracks[k];
        if t.phase == 3 || t.cur == -1 {
            return;
        }
        let (Some(cur), Some(prev)) = (anims.frames.get(t.cur as usize), anims.frames.get(t.prev as usize)) else { return };
        let dur = cur.duration as i32;
        if dur == 0 {
            return;
        }
        let along = ((dur - t.left as i32) << 12) / dur;
        for key in anims.keys_of(cur) {
            let Key::Rotation { channel, q } = *key else { continue };
            let from = anims.keys_of(prev).iter().find_map(|k| match *k {
                Key::Rotation { channel: c, q } if c == channel => Some(q),
                _ => None,
            });
            if let (Some(p), Some(slot)) = (from, self.pose.get_mut(channel as usize)) {
                *slot = Some(slerp(trig, p.map(|v| v as i32), q.map(|v| v as i32), along));
            }
        }
    }

    /// The base track's root motion (its frames' root keys, 0x800557ec):
    /// what it has moved so far, the frame showing taken in proportion to
    /// its time gone (0x80055904), and its whole move. None without root
    /// keys.
    pub fn root_progress(&self, anims: &Anims) -> Option<([i32; 3], [i32; 3])> {
        self.root_progress_scaled(anims, None)
    }

    /// [`Self::root_progress`] with each key first scaled by `lo` (4096 =
    /// 1, toward zero): another body than the player's moves by its root
    /// offset (hierarchy +0x4c << 12 over D_800c41dc's -561, 0x80055904).
    pub fn root_progress_scaled(&self, anims: &Anims, lo: Option<i32>) -> Option<([i32; 3], [i32; 3])> {
        let parts = self.root_parts_scaled(anims, lo)?;
        let (mut done, mut total) = ([0; 3], [0; 3]);
        for (d, t) in parts {
            for i in 0..3 {
                done[i] += d[i];
                total[i] += t[i];
            }
        }
        Some((done, total))
    }

    /// [`Self::root_progress_scaled`] by frame of the base state: each
    /// frame's part had so far (its key times its time gone, rounded toward
    /// zero, as 0x80055904's deltas are) and its whole key. None without
    /// root keys.
    pub fn root_parts_scaled(&self, anims: &Anims, lo: Option<i32>) -> Option<Vec<([i32; 3], [i32; 3])>> {
        let t = &self.tracks[0];
        let (first, count) = self.span(t.state, 0, anims)?;
        // Asked and not yet begun: at its start.
        let (cur, left) = if t.phase == 0 { (first, duration(anims, first)) } else { (t.cur, t.left) };
        let (first, count, cur, left) = (first as usize, count as usize, cur.max(0) as usize, left as i32);
        if count == 0 {
            return None;
        }
        let mut parts = Vec::with_capacity(count);
        let mut any = false;
        for k in first..first + count {
            let Some(r) = root_key(anims, k, lo) else {
                parts.push(([0; 3], [0; 3]));
                continue;
            };
            any = true;
            let part = if k < cur {
                4096
            } else if k == cur {
                let dur = anims.frames.get(k).map_or(1, |f| f.duration.max(1) as i32);
                ((dur - left).clamp(0, dur) << 12) / dur
            } else {
                0
            };
            let done = r.map(|v| {
                let p = v * part;
                if p < 0 { (p + 4095) >> 12 } else { p >> 12 }
            });
            parts.push((done, r));
        }
        any.then_some(parts)
    }

    /// Each part's kept quaternion (the part's +4 in the original), by
    /// part (None: never turned, the disc's zeros).
    pub fn turns(&self, hierarchy: &Hierarchy) -> Vec<Option<[i32; 4]>> {
        let mut out = vec![None; hierarchy.parts.len()];
        for (channel, q) in self.pose.iter().enumerate() {
            let part = hierarchy.header.get(0x14 + channel).copied().unwrap_or(0xff);
            if let (Some(q), Some(slot)) = (q, out.get_mut(part as usize)) {
                *slot = Some(*q);
            }
        }
        out
    }

}

/// A frame's length (its +2, at least 1), as a track's time left starts.
fn duration(anims: &Anims, frame: i16) -> i16 {
    anims.frames.get(frame as usize).map_or(0, |f| f.duration as i16).max(1)
}

/// The turn `t` (4096 = all) of the way from quaternion `a` to `b`
/// (0x800202ec): by the arc between them (acos, 0x80021500; rsin) unless
/// nearly the same (cos at least 3841), then straight; the shorter way.
fn slerp(trig: &Trig, a: [i32; 4], b: [i32; 4], t: i32) -> [i32; 4] {
    let toward = |v: i32| if v < 0 { (v + 4095) >> 12 } else { v >> 12 };
    let dot = toward((0..4).map(|k| a[k].wrapping_mul(b[k])).fold(0i32, i32::wrapping_add));
    let (dot, flip) = if dot < 0 { (-dot, true) } else { (dot, false) };
    let (mut wa, mut wb) = (4096 - t, t);
    if 4096 - dot >= 256 {
        let omega = trig.acos(dot as i16 as i32) as i16 as i32;
        let sin = trig.sin(omega);
        let part = toward(t * omega);
        wa = (trig.sin(omega - part) << 12) / sin;
        wb = (trig.sin(part) << 12) / sin;
    }
    if flip {
        wb = -wb;
    }
    [0, 1, 2, 3].map(|k| toward(wa * a[k] + wb * b[k]))
}

/// The height over the floor an animation puts a body at on the ground
/// (0x80055904 into +0xda / +0xde): the base track's root curve so far,
/// plus where the body was as the animation began, blended away over its
/// first frame (worklog 147).
#[derive(Clone, Debug, Default)]
pub struct RootCurve {
    state: i16,
    first: usize,
    count: usize,
    cur: usize,
    done: i32,
    total: i32,
    base: i32,
    /// Its state begun here (its first frame's gap taken, the keys from
    /// there), not one set up and gone on from where it was.
    begun: bool,
    /// What a state set up moves the body by across its first frame
    /// (+0xde: 0 from the settling, 0x800452f8), in place of that frame's key.
    gap: i32,
    /// The height: drawn under the body's position.
    pub y: i32,
    /// What the keys and the gap make the height (the original adds each
    /// step's change of it to +0x1c), and the height before this step.
    model: i32,
    before: i32,
    /// The across part of the base state's root taken so far, and what a
    /// state run out this step still had (for a state whose word has 32).
    across_done: Vec<[i32; 2]>,
    across_rest: Option<Vec<[i32; 2]>>,
    across_cur: usize,
}

impl RootCurve {
    /// This step's move across by the base state's root (a state whose
    /// word has 32, 0x80055904's +0xd8/+0xdc), in the body's own frame:
    /// the keys scaled by `lo`; a state begun anew first gives what the
    /// one before still had.
    pub fn across(&mut self, a: &Animator, anims: &Anims, lo: i32) -> Vec<[i32; 2]> {
        let mut out = Vec::new();
        // (A loop's next round starts it again as well.)
        let (_, _, _, now, _) = a.track();
        let round = now < self.across_cur;
        self.across_cur = now;
        if a.began() || round {
            // What the state run out still had, frame by frame.
            if let Some(rest) = self.across_rest.take() {
                for (k, r) in rest.iter().enumerate() {
                    let had = self.across_done.get(k).copied().unwrap_or([0; 2]);
                    out.push([r[0] - had[0], r[1] - had[1]]);
                }
            }
            self.across_done.clear();
        }
        self.across_rest = None;
        if a.moves_across()
            && let Some(parts) = a.root_parts_scaled(anims, Some(lo))
        {
            self.across_done.resize(parts.len(), [0; 2]);
            for (k, (done, _)) in parts.iter().enumerate() {
                let had = self.across_done[k];
                out.push([done[0] - had[0], done[2] - had[1]]);
                self.across_done[k] = [done[0], done[2]];
            }
            let (_, first, count, cur, _) = a.track();
            if cur + 1 == first + count {
                self.across_rest = Some(parts.iter().map(|(_, t)| [t[0], t[2]]).collect());
            }
        }
        out.retain(|d| *d != [0; 2]);
        out
    }

    /// One step, after the animator's: the keys scaled by `lo` (4096 = 1:
    /// a body other than the player's moves by its root offset, hierarchy
    /// +0x4c << 12 over 0x800c41dc's -561, 0x80055904). A new state, or a
    /// loop's next round, starts from where the last left the body (with
    /// what its end still had); its first frame takes the body from there
    /// to the key, base + (key - base) frac rounded toward zero. `off` is
    /// where the keys start under the body's place: 0 for a body settled on
    /// its floor, else the floor and the feet less its place (the original
    /// takes its first frame's gap to the floor (+0x148), the feet and the
    /// key: an unsettled hover is brought down to it).
    pub fn step(&mut self, a: &Animator, anims: &Anims, lo: i32, off: i32) {
        let (state, first, count, cur, left) = a.track();
        let (done, total) = a.root_progress_scaled(anims, Some(lo)).map_or((0, 0), |r| (r.0[1], r.1[1]));
        self.before = self.y;
        // (A first frame begun again, as a state asked anew each frame is,
        // takes its gap from where the body is now: 0x80055904's +0xde.)
        if (self.state, self.first) != (state, first) || cur < self.cur || (a.began() && cur == first) {
            let rest = if self.cur + 1 == self.first + self.count { self.total - self.done } else { 0 };
            self.base = self.y + rest;
            self.model = self.y;
            self.begun = true;
        }
        // A state set up (0x80055ed8) moves the body by its keys from where
        // it is, each step the frame's key times the time gone.
        if !self.begun {
            let done = self.gap_for_key(anims, lo, first, cur, left, done);
            let total = self.gap_for_key(anims, lo, first, first + 1, 0, total);
            self.y += done - self.done;
            (self.cur, self.done, self.total) = (cur, done, total);
            return;
        }
        let model = off + if cur > first {
            done
        } else {
            let dur = anims.frames.get(first).map_or(1, |f| f.duration.max(1) as i32);
            let frac = ((dur - left).clamp(0, dur) << 12) / dur;
            let key = root_key(anims, first, Some(lo)).map_or(0, |k| k[1]);
            let base = self.base - off;
            let p = (key - base) * frac;
            base + if p < 0 { (p + 4095) >> 12 } else { p >> 12 }
        };
        // (Each step's change, as 0x80055904 adds it: a place put back
        // keeps what was put back.)
        self.y += model - self.model;
        self.model = model;
        (self.state, self.first, self.count, self.cur, self.done, self.total) = (state, first, count, cur, done, total);
    }

    /// The place put back where the frame began (0x800627a0): the height
    /// this step added goes with it.
    pub fn put_back(&mut self) {
        self.y = self.before;
    }

    /// Takes on the base track as it stands, set up and not begun
    /// (0x80055f7c): the body goes on by its keys from where it is, and
    /// across the first frame by `gap` (+0xde: 0x80055904 takes it there
    /// whatever the time, the key only on the frames after).
    pub fn set_up(&mut self, a: &Animator, anims: &Anims, lo: i32, gap: i32) {
        let (state, first, count, cur, left) = a.track();
        let (done, total) = a.root_progress_scaled(anims, Some(lo)).map_or((0, 0), |r| (r.0[1], r.1[1]));
        self.gap = gap;
        let done = self.gap_for_key(anims, lo, first, cur, left, done);
        let total = self.gap_for_key(anims, lo, first, first + 1, 0, total);
        (self.state, self.first, self.count, self.cur, self.done, self.total) = (state, first, count, cur, done, total);
        self.begun = false;
        // On its first frame with its whole time still to go, its next step
        // begins it (0x80055904 takes the gap anew there).
        let dur = anims.frames.get(first).map_or(1, |f| f.duration.max(1) as i32);
        if cur == first && left >= dur {
            self.state = -2;
        }
    }

    /// The keys' progress `done` with the first frame's part taken by the
    /// gap instead of its key.
    fn gap_for_key(&self, anims: &Anims, lo: i32, first: usize, cur: usize, left: i32, done: i32) -> i32 {
        let part = if cur > first {
            4096
        } else {
            let dur = anims.frames.get(first).map_or(1, |f| f.duration.max(1) as i32);
            ((dur - left).clamp(0, dur) << 12) / dur
        };
        let toward_zero = |p: i32| if p < 0 { (p + 4095) >> 12 } else { p >> 12 };
        let key = root_key(anims, first, Some(lo)).map_or(0, |k| k[1]);
        done - toward_zero(key * part) + toward_zero(self.gap * part)
    }
}
