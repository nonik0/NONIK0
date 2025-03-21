use crate::{NUM_CHARS, Display, Event, Setting, SavedSettings};
use static_cell::make_static;

mod menu;
mod nametag;
mod random;
mod sensors;
mod settings;
mod traffic;
mod tunnel;
mod vibes;

pub use menu::*;
pub use nametag::*;
pub use random::*;
pub use sensors::*;
pub use settings::*;
pub use traffic::*;
pub use tunnel::*;
pub use vibes::*;

pub const NUM_MODES: u8 = 8;

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
            mode_index: settings.read_setting_byte(Setting::LastMode) % NUM_MODES,
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
        self.settings.save_setting_byte(Setting::LastMode, self.mode_index);
    }
}

pub trait Mode {
    //type Setting;

    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display);
}

pub fn names(index: u8) -> &'static [u8; NUM_CHARS] {
    [
        b"  NONIK0", // 0
        b" Nametag", // 1
        b"  Random", // 2
        b" Sensors", // 3
        b"Settings", // 4
        b" Traffic", // 5
        b"  Tunnel", // 6
        b"   Vibes", // 7
    ][index as usize]
}

pub fn take(adc: crate::Adc0, sigrow: crate::Sigrow, vref: crate::Vref, context: &Context, display: &mut Display) -> [&'static mut dyn Mode; NUM_MODES as usize] {
    unsafe {
        if MODES_TAKEN {
            panic!("Modes already taken!");
        }
        MODES_TAKEN = true;
    }

    let menu = make_static!(Menu::new_with_settings(&context.settings));
    let nametag = make_static!(Nametag::new_with_settings(&context.settings));
    let random = make_static!(Random::new_with_settings(&context.settings));
    let sensors = make_static!(Sensors::new_with_settings(&context.settings, adc, sigrow, vref));
    let settings = make_static!(Settings::new_with_settings(&context.settings));
    let traffic = make_static!(Traffic::new());
    let tunnel = make_static!(Tunnel::new());
    let vibes = make_static!(Vibes::new());

    // TODO: improve design of mode initialization
    sensors.seed_rand();
    settings.apply(display);

    [menu, nametag, random, sensors, settings, traffic, tunnel, vibes]
}
