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
pub mod id;
pub mod let_there_be_light;
pub mod mirrorland;
pub mod mr_friend;
pub mod outer_space;
pub mod pie_toss;
pub mod reptar_2010;
pub mod toy_palace;
pub mod training;

pub use id::Level;

use crate::disc::GameDisc;
use crate::world::game::{Game, Script};

/// The script of level `level` (EpisodeStart, installed by 0x8007871c),
/// started on the opened level; None for a level with no play of its own.
pub fn script(level: Level, g: &mut Game, disc: &GameDisc) -> Result<Option<Box<dyn Script>>, String> {
    fn boxed<S: Script + 'static>(s: S) -> Option<Box<dyn Script>> {
        Some(Box::new(s))
    }
    Ok(match level {
        Level::HUB => boxed(hub::Hub::start(g, disc)?),
        Level::GRANDPAS_TEETH => boxed(grandpas_teeth::GrandpasTeeth::start(g, disc)?),
        Level::AISLE_SEVEN => boxed(aisle_seven::AisleSeven::start(g, disc)?),
        Level::CYNTHIA => boxed(cynthia::Cynthia::start(g, disc)?),
        Level::OUTER_SPACE => boxed(outer_space::OuterSpace::start(g, disc)?),
        Level::TOY_PALACE => boxed(toy_palace::ToyPalace::start(g, disc)?),
        Level::REPTAR_2010 => boxed(reptar_2010::Reptar2010::start(g, disc)?),
        Level::MR_FRIEND => boxed(mr_friend::MrFriend::start(g, disc)?),
        Level::CHUCKIES_GLASSES => boxed(chuckies_glasses::ChuckiesGlasses::start(g, disc)?),
        Level::CIRCUS => boxed(circus::Circus::start(g, disc)?),
        Level::LET_THERE_BE_LIGHT => boxed(let_there_be_light::LetThereBeLight::start(g, disc)?),
        Level::COOKIE_RACE => boxed(cookie_race::CookieRace::start(g, disc)?),
        Level::RETURN_OF_MR_FRIEND => boxed(mr_friend::MrFriend::start14(g, disc)?),
        Level::MIRRORLAND => boxed(mirrorland::Mirrorland::start(g, disc)?),
        Level::TRAINING => boxed(training::Training::start(g, disc)?),
        Level::GOLD_RUSH => boxed(gold_rush::GoldRush::start(g, disc)?),
        Level::EGG_HUNT => boxed(egg_hunt::EggHunt::start(g, disc)?),
        Level::NO_MORE_COOKIES => boxed(box_toss::BoxToss::start(g, disc)?),
        Level::TOUCHDOWN_TOMMY => boxed(box_toss::BoxToss::start21(g, disc)?),
        l if l.is_hole() => boxed(golf::Golf::start(g, disc)?),
        _ => None,
    })
}
