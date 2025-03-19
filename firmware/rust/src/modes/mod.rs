use crate::{NUM_CHARS, Display, Event, SavedSettings};
use static_cell::make_static;

mod game;
mod menu;
mod nametag;
mod random;
mod settings;
mod utils;
mod vibes;

pub use game::*;
pub use menu::*;
pub use nametag::*;
pub use random::*;
pub use settings::*;
pub use utils::*;
pub use vibes::*;

pub const NUM_MODES: u8 = 7;

static mut MODES_TAKEN: bool = false;

// simple context wrapper struct to handle switching modes and tracking state between modes
pub struct Context {
    menu_counter: u16, // overflow issue
    mode_index: u8,
    pub settings: SavedSettings,
}

impl Context {
    pub fn new(settings: SavedSettings) -> Self {
        Context {
            menu_counter: 1,
            mode_index: 1,
            settings,
        }
    }

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

pub trait Mode {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display);
}

pub fn names(index: u8) -> &'static [u8; NUM_CHARS] {
    [
        b"  NONIK0", // 0
        b" Nametag", // 1
        b"    Game", // 2
        b"  Random", // 3
        b"Settings", // 4
        b"   Utils", // 5
        b"   Vibes", // 6
    ][index as usize]
}

pub fn take(adc: crate::Adc0, sigrow: crate::Sigrow, vref: crate::Vref, context: &Context) -> [&'static mut dyn Mode; NUM_MODES as usize] {
    unsafe {
        if MODES_TAKEN {
            panic!("Modes already taken!");
        }
        MODES_TAKEN = true;
    }

    let menu = make_static!(Menu::new(context.mode_index));
    let nametag = make_static!(Nametag::new_with_name(&context.settings.name()));
    let game = make_static!(Game::new());
    let random = make_static!(Random::new());
    let settings = make_static!(Settings::new_with_settings(context.settings.brightness(), context.settings.current()));
    let utils = make_static!(Utils::new_with_adc(adc, sigrow, vref));
    let vibes = make_static!(Vibes::new());

    utils.seed_rand();

    [menu, nametag, game, random, settings, utils, vibes]
}
