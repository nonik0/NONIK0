use super::Mode;
use crate::{
    Adc0, Context, Display, Event, Vref,
};

enum Util {
    I2CDetect,
    Temp,
    Vext,
    Vref,
    // ADC settings
    VrefSet,
    Prescaler,
    Resolution,
}

pub struct Utils {
    cur_util: Util,
    last_update: u16,
    adc: Adc0,
    vref: Vref,
}

impl Utils {
    pub fn new_with_adc(adc: Adc0, vref: Vref) -> Self {
        Utils {
            cur_util: Util::Temp,
            last_update: 0,
            adc,
            vref,
        }
    }

    fn read_temp(&mut self) -> u16 {
        // if Vref is not 1.1V
        // //self.vref.ctrlb.modify(|_, w| w.adc0refen().clear_bit());
        // self.vref.ctrla.modify(|_, w| w.adc0refsel()._1v1());

        // self.adc_settings.ref_voltage = ReferenceVoltage::Internal;
        // self.adc.initialize(self.adc_settings);
        // self.adc
        //     .read_blocking(&avrxmega_hal::adc::channel::Temperature)
        0
    }
}

impl Mode for Utils {
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            update = true;
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    let next_util = match self.cur_util {
                        Util::I2CDetect => Util::Temp,
                        Util::Temp => Util::Vext,
                        Util::Vext => Util::Vref,
                        Util::Vref => Util::VrefSet,
                        Util::VrefSet => Util::Prescaler,
                        Util::Prescaler => Util::Resolution,
                        Util::Resolution => Util::I2CDetect,
                    };
                    self.cur_util = next_util;
                }
                Event::LeftReleased => match self.cur_util {
                    Util::VrefSet => {}
                    _ => {}
                },
                Event::RightReleased => match self.cur_util {
                    Util::VrefSet => {}
                    _ => {}
                },
                _ => {}
            }
        }

        if update {
            match self.cur_util {
                Util::I2CDetect => display.print_ascii_bytes(b"I2C: ???").unwrap(),
                Util::Temp => display.print_ascii_bytes(b"Temp:??\x98F").unwrap(), // HCMS-29xx special char
                Util::Vext => display.print_ascii_bytes(b"Vext:?.?V").unwrap(),
                Util::Vref => display.print_ascii_bytes(b"Vref:?.?V").unwrap(),
                Util::VrefSet => display.print_ascii_bytes(b"VrSet:?.?").unwrap(),
                Util::Prescaler => display.print_ascii_bytes(b"Presc: ???").unwrap(),
                Util::Resolution => display.print_ascii_bytes(b"Resol: ???").unwrap(),
            }
        }
    }
}
