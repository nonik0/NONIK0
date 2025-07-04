use crate::{adc::Adc, Display, Event, SavedSettings, Setting, tone::Tone, NUM_CHARS};
use enum_dispatch::enum_dispatch;

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
    let mut count = 1;
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

pub const MODE_NAMES: [&[u8; NUM_CHARS]; NUM_MODES] = [
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

//static mut MODES_TAKEN: bool = false;

pub struct Context {
    mode_init: bool,
    mode_index: u8,
    pub tone_enabled: bool,
    pub settings: SavedSettings,
}

impl Context {
    pub fn new(settings: SavedSettings) -> Self {
        let mut saved_index = settings.read_setting_byte(Setting::LastMode);
        if saved_index >= NUM_MODES as u8 {
            saved_index = 1;
        }
        Self {
            mode_init: false,
            mode_index: saved_index,
            tone_enabled: settings.read_setting_bool(Setting::Tone),
            settings,
        }
    }

    #[inline(always)]
    pub fn is_menu(&self) -> bool {
        self.mode_index == 0
    }

    #[inline(always)]
    pub fn need_update(&mut self) -> bool {
        let update = !self.mode_init;
        self.mode_init = true;
        update
    }

    pub fn to_menu(&mut self) {
        self.mode_init = false;
        self.mode_index = 0;
    }

    pub fn mode_index(&self) -> usize {
        self.mode_index as usize
    }

    pub fn to_mode(&mut self, index: usize) {
        self.mode_init = false;
        self.mode_index = index as u8;
        self.settings
            .save_setting_byte(Setting::LastMode, self.mode_index);
    }
}

pub struct Peripherals {
    pub adc: Adc,
    pub buzzer: Tone,
    pub display: Display,
}

impl Peripherals {
    pub fn new(adc: Adc, buzzer: Tone, display: Display) -> Self {
        Self { adc, buzzer, display }
    }
}

#[enum_dispatch]
pub trait ModeHandler {
    fn update(
        &mut self,
        event: &Option<Event>,
        context: &mut Context,
        peripherals: &mut Peripherals,
    );
}

#[enum_dispatch(ModeHandler)]
pub enum Mode {
    Menu(Menu),
    #[cfg(not(feature = "no_nametag"))]
    Nametag(Nametag),
    #[cfg(not(feature = "no_random"))]
    Random(Random),
    #[cfg(not(feature = "no_sensors"))]
    Sensors(Sensors),
    #[cfg(not(feature = "no_settings"))]
    Settings(Settings),
    #[cfg(not(feature = "no_traffic"))]
    Traffic(Traffic),
    #[cfg(not(feature = "no_tunnel"))]
    Tunnel(Tunnel),
    #[cfg(not(feature = "no_vibes"))]
    Vibes(Vibes),
}

impl Mode {
    pub fn from_context(context: &Context) -> Self {
        let index = context.mode_index();
        let mut i = 0;
        if index == i {
            return Mode::Menu(Menu::new_with_settings(&context.settings));
        }
        i += 1;
        #[cfg(not(feature = "no_nametag"))]
        {
            if index == i {
                return Mode::Nametag(Nametag::new_with_settings(&context.settings));
            }
            i += 1;
        }
        #[cfg(not(feature = "no_random"))]
        {
            if index == i {
                return Mode::Random(Random::new_with_settings(&context.settings));
            }
            i += 1;
        }
        #[cfg(not(feature = "no_sensors"))]
        {
            if index == i {
                return Mode::Sensors(Sensors::new_with_settings(&context.settings));
            }
            i += 1;
        }
        #[cfg(not(feature = "no_settings"))]
        {
            if index == i {
                return Mode::Settings(Settings::new_with_settings(&context.settings));
            }
            i += 1;
        }
        #[cfg(not(feature = "no_traffic"))]
        {
            if index == i {
                return Mode::Traffic(Traffic::new());
            }
            i += 1;
        }
        #[cfg(not(feature = "no_tunnel"))]
        {
            if index == i {
                return Mode::Tunnel(Tunnel::new());
            }
            i += 1;
        }
        #[cfg(not(feature = "no_vibes"))]
        {
            if index == i {
                return Mode::Vibes(Vibes::new());
            }
        }
        panic!("Invalid mode index: {}", index);
    }
}
