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
}

impl<Pin> Hcms29xx<Pin>
where
    Pin: OutputPin,
{
    /// Creates a new SIPO shift register from clock, latch, and data output pins
    pub fn new(num_chars: usize, data: Pin, rs: Pin, clk: Pin, ce: Pin, blank: Option<Pin>, osc_sel: Option<Pin>) -> Self {
        Hcms29xx {
            num_chars: num_chars as u8,
            data: RefCell::new(data),
            rs: RefCell::new(rs),
            clk: RefCell::new(clk),
            ce: RefCell::new(ce),
            blank: blank.map(RefCell::new),
            osc_sel: osc_sel.map(RefCell::new),
            control_word_0: 0,
            control_word_1: 0,
        }
    }

    pub fn begin(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.clear()?;

        let font_data = font5x7::FONT5X7.load();

        // set dot mode to simultaneous to control             


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

    fn set_dout_mode(&mut self) -> Result<(), Hcms29xxErr<Pin>> {
        self.clk.borrow_mut().set_high()?;
        self.rs.borrow_mut().set_high()?;
        self.ce.borrow_mut().set_low()?;
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

    // /// Get embedded-hal output pins to control the shift register outputs
    // pub fn decompose(&self) -> [ShiftRegisterPin<'_, Pin1, Pin2, Pin3, N>; N] {
    //     core::array::from_fn(|i| ShiftRegisterPin::<'_, Pin1, Pin2, Pin3, N>::new(self, i))
    // }

    // /// Consume the shift register and return the original clock, latch, and data output pins
    // pub fn release(self) -> (Pin1, Pin2, Pin3) {
    //     let Self {
    //         clock,
    //         latch,
    //         data,
    //         output_state: _,
    //     } = self;
    //     (clock.into_inner(), latch.into_inner(), data.into_inner())
    // }

    // fn update(
    //     &self,
    //     index: usize,
    //     command: bool,
    // ) -> Result<
    //     (),
    //     SRErr<Pin1, Pin2, Pin3>,
    // > {

