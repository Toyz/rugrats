//! The levels' own play (docs/levels): one script a level, run by
//! world::game through the Script trait.

pub mod aisle_seven;
pub mod box_toss;
pub mod chuckies_glasses;
pub mod circus;
pub mod cookie_race;
pub mod cynthia;
pub mod egg_hunt;
pub mod gold_rush;
pub mod golf;
pub mod grandpas_teeth;
pub mod hub;
pub mod let_there_be_light;
pub mod mirrorland;
pub mod mr_friend;
pub mod outer_space;
pub mod pie_toss;
pub mod reptar_2010;
pub mod toy_palace;
pub mod training;

use crate::disc::GameDisc;
use crate::world::game::{Game, Script};

/// The script of level `level` (EpisodeStart, installed by 0x8007871c),
/// started on the opened level; None for a level with no play of its own.
pub fn script(level: usize, g: &mut Game, disc: &GameDisc) -> Result<Option<Box<dyn Script>>, String> {
    fn boxed<S: Script + 'static>(s: S) -> Option<Box<dyn Script>> {
        Some(Box::new(s))
    }
    Ok(match level {
        1 => boxed(hub::Hub::start(g, disc)?),
        2 => boxed(grandpas_teeth::GrandpasTeeth::start(g, disc)?),
        3 => boxed(aisle_seven::AisleSeven::start(g, disc)?),
        4 => boxed(cynthia::Cynthia::start(g, disc)?),
        5 => boxed(outer_space::OuterSpace::start(g, disc)?),
        6 => boxed(toy_palace::ToyPalace::start(g, disc)?),
        7 => boxed(reptar_2010::Reptar2010::start(g, disc)?),
        9 => boxed(mr_friend::MrFriend::start(g, disc)?),
        10 => boxed(chuckies_glasses::ChuckiesGlasses::start(g, disc)?),
        11 => boxed(circus::Circus::start(g, disc)?),
        12 => boxed(let_there_be_light::LetThereBeLight::start(g, disc)?),
        13 => boxed(cookie_race::CookieRace::start(g, disc)?),
        14 => boxed(mr_friend::MrFriend::start14(g, disc)?),
        15 => boxed(mirrorland::Mirrorland::start(g, disc)?),
        16 => boxed(training::Training::start(g, disc)?),
        17 => boxed(gold_rush::GoldRush::start(g, disc)?),
        18 => boxed(egg_hunt::EggHunt::start(g, disc)?),
        20 => boxed(box_toss::BoxToss::start(g, disc)?),
        21 => boxed(box_toss::BoxToss::start21(g, disc)?),
        23..=31 => boxed(golf::Golf::start(g, disc)?),
        _ => None,
    })
}
