//! The follow camera (0x80027304) against the original: the player's path
//! as the oracle recorded it, replayed into the port's camera, which must
//! put the eye (0x800de530) and the look-at (0x800de454) where the
//! original did, frame for frame. Both windows start from R1's reset in
//! the hub's big room (room 23, camera height 1600) and end before the
//! original's view test meets a room polygon (0x80090554), which the
//! port does not have. Skipped without a disc in work/disc.

use std::path::Path;

use rugrats::disc::GameDisc;
use rugrats::input::Pads;
use rugrats::levels::Level;
use rugrats::save::progress::Progress;
use rugrats::world::game::Game;

fn disc() -> Option<GameDisc> {
    let cue = rrt::disc::Image::find(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/disc")).ok()?;
    GameDisc::open(&cue).ok()
}

/// The player at an oracle frame (even, one game frame each two): its
/// position, heading and speed (+0x18, +0x24, +0x184).
type Step = (u32, [i32; 3], i32, i32);

/// Where the camera was at an oracle frame: eye and look-at.
type Seen = (u32, [i32; 3], [i32; 3]);

/// The hub with the player at `start`, the room entered (its camera
/// height) and the camera reset behind it, as R1 does (0x80026ddc).
fn reset_at(disc: &GameDisc, start: Step) -> Game {
    let mut g = Game::open_from(disc, Level::HUB, None, &Progress::default()).unwrap();
    let (_, pos, heading, _) = start;
    g.player.pos = pos;
    g.player.heading = heading;
    g.player.target = heading;
    let mut pads = Pads::default();
    pads.push_bits(0, 0);
    g.frame(&pads);
    g.player.pos = pos;
    g.set_camera_yaw(heading as f32);
    g
}

/// Replays `path` (each step holding until the next) from `start` to
/// frame `end`, checking the camera at every frame in `seen`.
fn replay(g: &mut Game, start: Step, path: &[Step], end: u32, seen: &[Seen]) {
    let mut now = start;
    let mut checked = 0;
    for frame in (start.0 + 2..=end).step_by(2) {
        let prev = now.1;
        if let Some(&s) = path.iter().find(|s| s.0 == frame) {
            now = s;
        }
        g.player.pos = now.1;
        g.player.heading = now.2;
        g.player.speed = now.3;
        g.camera_frame(prev[0] != now.1[0] || prev[2] != now.1[2]);
        if let Some(&(_, eye, look)) = seen.iter().find(|s| s.0 == frame) {
            assert_eq!(g.follow_camera(), (eye, look), "camera at oracle frame {frame}");
            checked += 1;
        }
    }
    assert_eq!(checked, seen.len());
}

/// R1 at oracle frame 3002, then Tommy walks left (camera-relative) for
/// 15 frames, stops, and the camera swings round behind him at 512
/// (0x800284f4) until it passes his back (bits 0x940).
#[test]
fn hub_walk_then_round_behind() {
    let Some(disc) = disc() else { return };
    let start = (3002, [6752, 779, 19058], 1167, 0);
    let mut g = reset_at(&disc, start);
    assert_eq!(g.follow_camera(), ([2754, -1589, 19949], [6752, 11, 19058]), "the reset (oracle frame 3002)");
    let path: [Step; 20] = [
        (3022, [6752, 779, 19058], 1007, 0),
        (3024, [6752, 778, 19058], 847, 17),
        (3026, [6915, 747, 19103], 687, 17),
        (3028, [7062, 739, 19187], 527, 17),
        (3030, [7185, 756, 19304], 367, 17),
        (3032, [7275, 761, 19447], 207, 17),
        (3034, [7328, 769, 19608], 77, 17),
        (3036, [7348, 780, 19777], 51, 17),
        (3038, [7361, 751, 19946], 23, 17),
        (3040, [7366, 731, 20116], 4088, 17),
        (3042, [7364, 720, 20286], 4055, 17),
        (3044, [7354, 753, 20455], 4022, 17),
        (3046, [7335, 773, 20623], 3986, 17),
        (3048, [7307, 778, 20790], 3951, 17),
        (3050, [7270, 747, 20955], 3914, 17),
        (3052, [7270, 739, 20955], 3914, 0),
        (3054, [7270, 749, 20955], 3914, 0),
        (3056, [7270, 759, 20955], 3914, 0),
        (3058, [7270, 770, 20955], 3914, 0),
        (3060, [7270, 779, 20955], 3914, 0),
    ];
    let seen: [Seen; 10] = [
        (3026, [2756, -1589, 19937], [6798, 11, 19071]),
        (3030, [2780, -1589, 19856], [6960, 11, 19171]),
        (3040, [2982, -1589, 19567], [7260, 11, 19855]),
        (3050, [3294, -1589, 19316], [7241, 11, 20791]),
        (3060, [3647, -1589, 19128], [7175, 11, 21223]),
        (3078, [4132, -1589, 18911], [7220, 11, 21116]),
        (3090, [4085, -1589, 18288], [7259, 11, 20989]),
        (3130, [5307, -1589, 17264], [7267, 11, 20958]),
        (3170, [6815, -1589, 16771], [7267, 11, 20958]),
        (3200, [8010, -1589, 16792], [7267, 11, 20958]),
    ];
    replay(&mut g, start, &path, 3200, &seen);
}

/// R1 at oracle frame 2782, then Tommy turns round toward the camera and
/// walks at it: the eye backs off along his axis (0x8002797c) and is
/// pulled after the goal behind him.
#[test]
fn hub_turn_and_walk_at_the_camera() {
    let Some(disc) = disc() else { return };
    let start = (2782, [5239, 779, 18812], 3165, 0);
    let mut g = reset_at(&disc, start);
    assert_eq!(g.follow_camera(), ([9301, -1589, 18229], [5239, 11, 18812]), "the reset (oracle frame 2782)");
    let path: [Step; 17] = [
        (2802, [5239, 779, 18812], 3325, 0),
        (2804, [5239, 772, 18812], 3485, 0),
        (2806, [5239, 776, 18812], 3645, 0),
        (2808, [5239, 793, 18812], 3805, 0),
        (2810, [5239, 794, 18812], 3965, 0),
        (2812, [5239, 789, 18812], 29, 0),
        (2814, [5239, 774, 18812], 189, 0),
        (2816, [5239, 771, 18812], 349, 17),
        (2818, [5325, 745, 18958], 509, 17),
        (2820, [5444, 739, 19078], 669, 17),
        (2822, [5589, 756, 19166], 829, 17),
        (2824, [5751, 761, 19216], 989, 17),
        (2826, [5921, 769, 19225], 1144, 17),
        (2828, [6088, 780, 19194], 1150, 17),
        (2830, [6254, 751, 19162], 1155, 17),
        (2832, [6420, 731, 19129], 1160, 17),
        (2834, [6586, 720, 19094], 1163, 17),
    ];
    let seen: [Seen; 7] = [
        (2814, [9301, -1589, 18229], [5239, 11, 18812]),
        (2816, [9290, -1589, 18224], [5239, 11, 18813]),
        (2818, [9279, -1589, 18224], [5264, 11, 18852]),
        (2820, [9290, -1589, 18231], [5317, 11, 18913]),
        (2824, [9374, -1589, 18263], [5510, 11, 19040]),
        (2828, [9510, -1589, 18273], [5787, 11, 19101]),
        (2834, [9753, -1589, 18242], [6280, 11, 19090]),
    ];
    replay(&mut g, start, &path, 2834, &seen);
}
