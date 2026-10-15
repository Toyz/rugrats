use rugrats::levels::Level;
use rugrats::rand::Rand;
use rugrats::save::progress::{Finish, Progress};

#[test]
fn main_levels_give_pieces_and_open_bonus_levels() {
    let mut p = Progress::default();
    let mut r = Rand::default();
    let f = p.finish(Level(2), true, &mut r);
    assert_eq!(f, Finish { bars: 0, piece: true, next: None });
    assert_eq!(p.pieces(), 1);
    assert!(p.won(Level(2)));
    // The second main level opens No More Cookies, the third Mirrorland.
    assert_eq!(p.finish(Level(3), true, &mut r).next, Some(Level(20)));
    assert_eq!(p.finish(Level(4), true, &mut r).next, Some(Level(15)));
    assert_eq!(p.pieces(), 3);
    // Won again: no new piece, no count.
    assert_eq!(p.finish(Level(4), true, &mut r), Finish::default());
    // Mirrorland goes back to the hub and opens its doors.
    assert_eq!(p.finish(Level(15), true, &mut r).next, Some(Level(1)));
    assert_eq!(p.flags() & 16, 16);
}

#[test]
fn bonus_levels_give_bars_and_lost_levels_nothing() {
    let mut p = Progress::default();
    let mut r = Rand::default();
    assert_eq!(p.finish(Level(5), false, &mut r), Finish::default());
    assert!(!p.won(Level(5)));
    assert_eq!(p.finish(Level(17), true, &mut r).bars, 3);
    assert_eq!(p.bars_available(), 3);
    assert_eq!(p.finish(Level(13), true, &mut r).bars, 2);
    // Cookie Race gives its bars once.
    assert_eq!(p.finish(Level(13), true, &mut r).bars, 0);
}

#[test]
fn fifteen_bars_buy_a_piece() {
    let mut p = Progress::default();
    let mut r = Rand::default();
    for n in 0..15 {
        p.find_bar(Level(9), n);
    }
    assert!(p.buy_piece(&mut r));
    assert_eq!(p.bars_available(), 0);
    assert_eq!(p.pieces(), 1);
    assert!(!p.buy_piece(&mut r));
}

#[test]
fn activity_play_keeps_nothing() {
    let mut p = Progress::default();
    p.set_activity(true);
    assert_eq!(p.finish(Level(17), true, &mut Rand::default()), Finish::default());
    assert!(!p.won(Level(17)));
}
