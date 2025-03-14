// TODO: feature gate specific modes (i.e. feather is only vibes)
use crate::{Context, Display, Event, NUM_CHARS};
use static_cell::make_static;

mod game;
mod menu;
mod nametag;
mod random;
mod vibes;

pub use game::*;
pub use menu::*;
pub use nametag::*;
pub use random::*;
pub use vibes::*;

pub const NUM_MODES: usize = 5;

static mut MODES_TAKEN: bool = false;

pub trait Mode {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context);
}

pub fn names(index: usize) -> &'static [u8; NUM_CHARS] {
    [
        b" NONIK0 ",
        b"Nametag ",
        b"   Game ",
        b" Random ",
        b"  Vibes ",
    ][index]
}

pub fn take() -> [&'static mut dyn Mode; NUM_MODES] {
    if unsafe { MODES_TAKEN } {
        panic!("Modes already taken!");
    }
    unsafe {
        MODES_TAKEN = true;
    }

    let menu = make_static!(Menu::new());
    let nametag = make_static!(Nametag::new());
    let game = make_static!(Game::new());
    let random = make_static!(Random::new());
    let vibes = make_static!(Vibes::new());

    [menu, nametag, game, random, vibes]
}
