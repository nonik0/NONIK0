use constants::{ControlWord0, ControlWord1};
use core::cell::RefCell;
use embedded_hal::digital::{self, ErrorType, OutputPin};

type Hcms29xxErr<Pin> = Hcms29xxError<<Pin as ErrorType>::Error>;

mod constants;
mod font5x7;

#[derive(Clone, Copy, Debug)]
pub enum Hcms29xxError<E> {
    OutputPinError(E),
}

impl<PinError> From<PinError> for Hcms29xxError<PinError> {
    fn from(error: PinError) -> Self {
        Hcms29xxError::OutputPinError(error)
    }
}

// impl<E> digital::Error for Hcms29xxError<E>
// where
//     E: core::fmt::Debug,
// {
//     fn kind(&self) -> digital::ErrorKind {
//         digital::ErrorKind::Other
//     }
// }

pub struct Hcms29xx<Pin>
where
    Pin: OutputPin,
{
    num_chars: u8,
    data: RefCell<Pin>,
    rs: RefCell<Pin>,
    clk: RefCell<Pin>,
    ce: RefCell<Pin>,
    blank: Option<RefCell<Pin>>,
    osc_sel: Option<RefCell<Pin>>,
    control_word_0: u8,
    control_word_1: u8,
    font_ascii_start_index: u8,
}

impl<Pin> Hcms29xx<Pin>
where
    Pin: OutputPin,
{
    pub fn new(
        num_chars: usize,
        data: Pin,
        rs: Pin,
        clk: Pin,
        ce: Pin,
        blank: Option<Pin>,
        osc_sel: Option<Pin>,
    ) -> Result<Self, Hcms29xxErr<Pin>> {
        // TODO
        // data.set_low().unwrap();
        // ce.set_high().unwrap();
        // if let Some(ref blank) = blank {
        //     blank.set_high().unwrap();
        // }

        let font_data = font5x7::FONT5X7.load();
        let font_ascii_start_index = font_data[0] - 1;

        let new_hcms = Hcms29xx {
            num_chars: num_chars as u8,
            data: RefCell::new(data),
            rs: RefCell::new(rs),
            clk: RefCell::new(clk),
            ce: RefCell::new(ce),
            blank: blank.map(RefCell::new),
            osc_sel: osc_sel.map(RefCell::new),
            control_word_0: 0,
            control_word_1: 0,
            font_ascii_start_index: font_ascii_start_index,
        };

        new_hcms.data.borrow_mut().set_low()?;
        new_hcms.ce.borrow_mut().set_high()?;
        if let Some(ref blank) = new_hcms.blank {
            blank.borrow_mut().set_high()?;
        }

        Ok(new_hcms)
    }

    pub fn begin(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.clear()?;

        self.update_control_word(
            ControlWord0::SELECT.bits()
                | ControlWord0::NORMAL_OPERATION.bits()
                | constants::DEFAULT_CURRENT
                | constants::DEFAULT_BRIGHTNESS,
        )?;
        self.update_control_word(ControlWord1::SELECT.bits() | constants::DEFAULT_DATA_OUT_MODE)?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.set_dot_data()?;
        for i in 0..self.num_chars * constants::CHAR_WIDTH {
            self.send_byte(0x00)?;
        }
        self.end_transfer()?;
        Ok(())
    }

    pub fn print_c_string(&mut self, c_str: &[u8]) -> Result<(), Hcms29xxErr<Pin>> {
        self.set_dot_data()?;
        let font_data = font5x7::FONT5X7.load(); // TODO: does load cost anything?
        for i in 0..self.num_chars {
            if i >= c_str.len() as u8 || c_str[i as usize] < self.font_ascii_start_index {
                break;
            }
            let char_start_index =
                (c_str[i as usize] - self.font_ascii_start_index) * constants::CHAR_WIDTH;
            for j in 0..constants::CHAR_WIDTH {
                self.send_byte(font_data[(char_start_index + j) as usize])?;
            }
        }
        self.end_transfer()?;
        Ok(())
    }

    pub fn display_blank(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        if let Some(ref blank) = self.blank {
            blank.borrow_mut().set_high()?;
        }
        Ok(())
    }

    pub fn display_unblank(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        if let Some(ref blank) = self.blank {
            blank.borrow_mut().set_low()?;
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), Hcms29xxErr<Pin>> {
        self.update_control_word(
            self.control_word_0 & !ControlWord0::BRIGHTNESS_MASK.bits()
                | (brightness & ControlWord0::BRIGHTNESS_MASK.bits()),
        )?;
        Ok(())
    }

    pub fn set_ext_osc(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        if let Some(ref osc_sel) = self.osc_sel {
            osc_sel.borrow_mut().set_high()?;
        }
        Ok(())
    }

    pub fn set_int_osc(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        if let Some(ref osc_sel) = self.osc_sel {
            osc_sel.borrow_mut().set_low()?;
        }
        Ok(())
    }

    // fn set_data_out_serial_mode(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
    //     self.update_control_word(
    //         self.control_word_1 & !ControlWord1::DATA_OUT_SIMULTANEOUS.bits(),
    //     )?;
    //     Ok(())
    // }

    // fn set_data_out_simultaneous_mode(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
    //     self.update_control_word(self.control_word_1 | ControlWord1::DATA_OUT_SIMULTANEOUS.bits())?;
    //     Ok(())
    // }

    fn update_control_word(&mut self, control_word: u8) -> Result<(), Hcms29xxErr<Pin>> {
        // read current data out mode before potentially changing it
        let times_to_send =
            if (self.control_word_1 & ControlWord1::DATA_OUT_SIMULTANEOUS.bits()) != 0 {
                1
            } else {
                self.num_chars / constants::DEVICE_CHARS
            };

        self.set_control_data()?;
        for _ in 0..times_to_send {
            self.send_byte(control_word)?;
        }
        self.end_transfer()?;

        if control_word & ControlWord1::SELECT.bits() != 0 {
            self.control_word_1 = control_word;
        } else {
            self.control_word_0 = control_word;
        }

        Ok(())
    }

    fn set_dot_data(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.clk.borrow_mut().set_high()?;
        self.rs.borrow_mut().set_low()?;
        self.ce.borrow_mut().set_low()?;
        Ok(())
    }

    fn set_control_data(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.clk.borrow_mut().set_high()?;
        self.rs.borrow_mut().set_high()?;
        self.ce.borrow_mut().set_low()?;
        Ok(())
    }

    fn send_byte(&mut self, byte: u8) -> Result<(), Hcms29xxErr<Pin>> {
        for i in 0..8 {
            self.clk.borrow_mut().set_low()?;
            if (byte & (1 << (7 - i))) != 0 {
                self.data.borrow_mut().set_high()?;
            } else {
                self.data.borrow_mut().set_low()?;
            }
            self.clk.borrow_mut().set_high()?;
        }
        Ok(())
    }

    fn end_transfer(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.ce.borrow_mut().set_high()?;
        self.clk.borrow_mut().set_low()?;
        Ok(())
    }
}
