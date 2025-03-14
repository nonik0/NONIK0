// TODO: feature gate specific modes (i.e. feather is only vibes)
use crate::{Display, Event, NUM_CHARS};
use static_cell::make_static;

mod game;
mod menu;
mod nametag;
mod random;
mod settings;
mod vibes;

pub use game::*;
pub use menu::*;
pub use nametag::*;
pub use random::*;
pub use settings::*;
pub use vibes::*;

pub const NUM_MODES: u8 = 6;

static mut MODES_TAKEN: bool = false;

// simple context wrapper struct to handle switching modes and tracking state between modes
pub struct Context {
    menu_counter: u16, // overflow issue
    mode_index: u8,
}

impl Context {
    #[inline(always)]
    pub fn is_menu(&mut self) -> bool {
        self.mode_index == 0
    }

    #[inline(always)]
    // TODO: improve clunkiness of tracking updates (detect menu chagnes to draw minimal updates)
    pub fn needs_update(&mut self, last_update: &mut u16) -> bool {
        let update = *last_update < self.menu_counter;
        *last_update = self.menu_counter;
        update
    }

    #[inline(always)]
    pub fn to_menu(&mut self) {
        self.menu_counter += 1;
        self.mode_index = 0;
    }

    #[inline(always)]
    pub fn mode(&mut self) -> usize {
        self.mode_index as usize
    }
    
    #[inline(always)]
    pub fn to_mode(&mut self, index: u8) {
        self.mode_index = index;
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            menu_counter: 1,
            mode_index: 1,
        }
    }
}

pub trait Mode {
    fn update(&mut self, event: &Option<Event>, display: &mut Display, context: &mut Context);
}

pub fn names(index: u8) -> &'static [u8; NUM_CHARS] {
    [
        b"  NONIK0",
        b" Nametag",
        b"    Game",
        b"  Random",
        b"Settings",
        b"   Vibes",
    ][index as usize]
}

pub fn take() -> [&'static mut dyn Mode; NUM_MODES as usize] {
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
    let settings = make_static!(Settings::new());
    let vibes = make_static!(Vibes::new());

    [menu, nametag, game, random, settings, vibes]
}
