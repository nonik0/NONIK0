use crate::{Display, Event, SavedSettings, Setting, NUM_CHARS};
use static_cell::make_static;

mod menu;
#[cfg(not(feature = "no_nametag"))] 
mod nametag;
#[cfg(not(feature = "no_random"))]
mod random;
#[cfg(not(feature = "no_sensors"))]
mod sensors;
#[cfg(not(feature = "no_settings"))]
mod settings;
#[cfg(not(feature = "no_traffic"))]
mod traffic;
#[cfg(not(feature = "no_tunnel"))]
mod tunnel;
#[cfg(not(feature = "no_vibes"))]
mod vibes;

pub use menu::*;
#[cfg(not(feature = "no_nametag"))]
pub use nametag::*;
#[cfg(not(feature = "no_random"))]
pub use random::*;
#[cfg(not(feature = "no_sensors"))]
pub use sensors::*;
#[cfg(not(feature = "no_settings"))]
pub use settings::*;
#[cfg(not(feature = "no_traffic"))]
pub use traffic::*;
#[cfg(not(feature = "no_tunnel"))]
pub use tunnel::*;
#[cfg(not(feature = "no_vibes"))]
pub use vibes::*;

pub const NUM_MODES: usize = {
    let mut count = 1; // menu is always included
    #[cfg(not(feature = "no_nametag"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_random"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_sensors"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_settings"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_traffic"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_tunnel"))]
    {
        count += 1;
    }
    #[cfg(not(feature = "no_vibes"))]
    {
        count += 1;
    }
    count
};
pub const MODE_NAMES: [&[u8; NUM_CHARS]; NUM_MODES as usize] = [
    b"  NONIK0",
    #[cfg(not(feature = "no_nametag"))]
    b" Nametag",
    #[cfg(not(feature = "no_random"))]
    b"  Random",
    #[cfg(not(feature = "no_sensors"))]
    b" Sensors",
    #[cfg(not(feature = "no_settings"))]
    b"Settings",
    #[cfg(not(feature = "no_traffic"))]
    b" Traffic",
    #[cfg(not(feature = "no_tunnel"))]
    b"  Tunnel",
    #[cfg(not(feature = "no_vibes"))]
    b"   Vibes",
];

static mut MODES_TAKEN: bool = false;

pub struct Context {
    menu_counter: u16,
    mode_index: u8,
    pub settings: SavedSettings,
}

impl Context {
    pub fn new(settings: SavedSettings) -> Self {
        let mut saved_index = settings.read_setting_byte(Setting::LastMode);
        if saved_index >= NUM_MODES as u8 {
            saved_index = 1;
        }
        Context {
            menu_counter: 1,
            mode_index: saved_index,
            settings,
        }
    }

    #[inline(always)]
    pub fn is_menu(&mut self) -> bool {
        self.mode_index == 0
    }

    #[inline(always)]
    // TODO: improve clunkiness of tracking updates (detect menu changes to draw minimal updates)
    pub fn needs_update(&mut self, last_update: &mut u16) -> bool {
        let update = *last_update < self.menu_counter;
        *last_update = self.menu_counter;
        update
    }

    pub fn to_menu(&mut self) {
        self.menu_counter += 1;
        self.mode_index = 0;
    }

    pub fn mode(&mut self) -> usize {
        self.mode_index as usize
    }

    pub fn to_mode(&mut self, index: usize) {
        self.mode_index = index as u8;
        self.settings
            .save_setting_byte(Setting::LastMode, self.mode_index);
    }
}

pub trait Mode {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display);
}

pub fn take(
    adc: crate::Adc0,
    sigrow: crate::Sigrow,
    vref: crate::Vref,
    context: &Context,
    display: &mut Display,
) -> [&'static mut dyn Mode; NUM_MODES as usize] {
    unsafe {
        if MODES_TAKEN {
            panic!("Modes already taken!");
        }
        MODES_TAKEN = true;
    }

    let menu = make_static!(Menu::new_with_settings(&context.settings));

    #[cfg(not(feature = "no_nametag"))]
    let nametag = make_static!(Nametag::new_with_settings(&context.settings));

    #[cfg(not(feature = "no_random"))]
    let random = make_static!(Random::new_with_settings(&context.settings));

    #[cfg(not(feature = "no_sensors"))]
    let sensors = make_static!(Sensors::new_with_settings(
        &context.settings,
        adc,
        sigrow,
        vref
    ));

    #[cfg(not(feature = "no_settings"))]
    let settings = make_static!(Settings::new_with_settings(&context.settings));

    #[cfg(not(feature = "no_traffic"))]
    let traffic = make_static!(Traffic::new());

    #[cfg(not(feature = "no_tunnel"))]
    let tunnel = make_static!(Tunnel::new());

    #[cfg(not(feature = "no_vibes"))]
    let vibes = make_static!(Vibes::new());

    // TODO: improve design of mode initialization
    #[cfg(not(feature = "no_sensors"))]
    sensors.seed_rand();
    #[cfg(not(feature = "no_settings"))]
    settings.apply(display);

    [
        menu,
        #[cfg(not(feature = "no_nametag"))]
        nametag,
        #[cfg(not(feature = "no_random"))]
        random,
        #[cfg(not(feature = "no_sensors"))]
        sensors,
        #[cfg(not(feature = "no_settings"))]
        settings,
        #[cfg(not(feature = "no_traffic"))]
        traffic,
        #[cfg(not(feature = "no_tunnel"))]
        tunnel,
        #[cfg(not(feature = "no_vibes"))]
        vibes,
    ]
}
