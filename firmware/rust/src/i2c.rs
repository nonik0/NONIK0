#![allow(dead_code)]

use avrxmega_hal::{
    clock::Clock,
    port::{mode::*, Pin, PinOps, PB0, PB1},
};
use core::cell::RefCell;

type SDAPIN = PB1;
type SCLPIN = PB0;
type TWI = avrxmega_hal::pac::TWI0;

const fn add_read_bit(address: u8) -> u8 {
    address | 0x01
}
const fn add_write_bit(address: u8) -> u8 {
    address & !0x01
}
const fn twi_baud(freq: u32, t_rise: u32) -> u32 {
    ((crate::CoreClock::FREQ / freq) / 2)
        - (5 + (((crate::CoreClock::FREQ / 1_000_000) * t_rise) / 2000))
}

const I2C_BUFFER_SIZE: usize = 32;
static I2C_STATE: avr_device::interrupt::Mutex<RefCell<Option<I2cState>>> =
    avr_device::interrupt::Mutex::new(RefCell::new(None));

struct I2cState {
    twi: TWI,
    twi_init: bool,
    host_bus_speed: u32,
    client_address: u8,
    _sda: Pin<Input<AnyInput>, SDAPIN>,
    _scl: Pin<Input<AnyInput>, SCLPIN>,
    // buffer that holds data to send and received data
    data: [u8; I2C_BUFFER_SIZE],
    bytes_to_process: u8,
    bytes_processed: u8,
}

impl I2cState {
    fn new(
        twi: TWI,
        sda: Pin<Input<PullUp>, SDAPIN>,
        scl: Pin<Input<PullUp>, SCLPIN>,
        host_bus_speed: u32,
        address: u8,
    ) -> Self {
        Self {
            twi,
            twi_init: false,
            host_bus_speed,
            _sda: sda.forget_imode(),
            _scl: scl.forget_imode(),
            data: [0; I2C_BUFFER_SIZE],
            client_address: address << 1, // byte 0 is rw bit, 0 for write
            bytes_to_process: 0,
            bytes_processed: 0,
        }
    }
}

/// I2C Error
#[derive(ufmt::derive::uDebug, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Error {
    /// Lost arbitration while trying to acquire bus
    ArbitrationLost,
    /// No slave answered for this address or a slave replied NACK
    AddressNack,
    /// Slave replied NACK to sent data
    DataNack,
    /// A bus-error occured
    BusError,
    /// An unknown error occured.  The bus might be in an unknown state.
    Unknown,
    /// The I2C peripheral is not initialized
    Uninit,
    /// Buffer overflow, too many bytes to process
    BufferOverflow,
}

/// I2C Transfer Direction
#[derive(ufmt::derive::uDebug, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Direction {
    /// Write to a slave (LSB is 0)
    Write,
    /// Read from a slave (LSB is 1)
    Read,
}

pub struct I2c {}

impl I2c
where
    SDAPIN: PinOps,
    SCLPIN: PinOps,
{
    pub fn new(
        twi: TWI,
        sda: Pin<Input<PullUp>, SDAPIN>,
        scl: Pin<Input<PullUp>, SCLPIN>,
        speed: u32,
    ) -> Self {
        let state = I2cState::new(twi, sda, scl, speed, 0);

        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            *state_opt = Some(state);
        });

        Self {}
    }

    pub fn setup_host(&mut self, speed: u32) {
        self.raw_setup(speed);
    }

    pub fn setup_client(&mut self, address: u8) {
        self.raw_setup_client(address);
    }

    pub fn ping_device(&mut self, address: u8, direction: Direction) -> Result<bool, Error> {
        self.raw_start(address, direction)?;
        match self.raw_stop() {
            Ok(_) => Ok(true),
            Err(Error::AddressNack) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        self.raw_start(address, Direction::Write)?;
        self.raw_write(bytes)?;
        self.raw_stop()?;
        Ok(())
    }

    pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.raw_start(address, Direction::Read)?;
        self.raw_read(buffer, true)?;
        self.raw_stop()?;
        Ok(())
    }

    pub fn read_client(&mut self) -> Option<u8> {
        self.raw_read_client()
    }

    pub fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.raw_start(address, Direction::Write)?;
        self.raw_write(bytes)?;
        self.raw_start(address, Direction::Read)?;
        self.raw_read(buffer, true)?;
        self.raw_stop()?;
        Ok(())
    }

    pub fn end(&mut self) {
        self.raw_end();
        self.raw_end_client();
    }

    //
    // HOST
    //
    #[inline]
    fn raw_setup(&mut self, speed: u32) {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            let baud = twi_baud(speed, 350) as u8; // hard-coded rise time estimate for now
            state.twi.mbaud().write(|w| w.set(baud));
            state.twi.mctrla().write(|w| w.enable().set_bit());
            state
                .twi
                .mstatus()
                .write(|w| w.busstate().idle().busstate().idle());
        });
    }

    #[inline]
    fn raw_start(&mut self, address: u8, _direction: Direction) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.client_address = address << 1; // byte 0 is rw bit, 0 for write
            state.bytes_to_process = 0;
            state.bytes_processed = 0;

            Ok(())
        })
    }

    #[inline]
    fn raw_write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            let buf_start = state.bytes_to_process as usize;
            let buf_end = buf_start + bytes.len() as usize;
            if buf_end >= I2C_BUFFER_SIZE {
                return Err(Error::BufferOverflow);
            }

            state.data[buf_start..buf_end].copy_from_slice(bytes);
            state.bytes_to_process += bytes.len() as u8;

            Ok(())
        })
    }

    #[inline]
    fn raw_read(&mut self, _buffer: &mut [u8], _last_read: bool) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn raw_stop(&mut self) -> Result<(), Error> {
        self.host_transmit(true)
    }

    #[inline]
    fn raw_end(&mut self) {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.twi.mctrla().write(|w| w.enable().clear_bit());
            state.twi.mbaud().write(|w| w.set(0));
        });
    }

    //
    // CLIENT
    //
    #[inline]
    fn raw_setup_client(&mut self, address: u8) {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.twi.saddr().write(|w| w.set(address << 1));
            state.twi.sctrla().write(|w| {
                w.dien()
                    .set_bit()
                    .apien()
                    .set_bit()
                    .pien()
                    .set_bit()
                    .enable()
                    .set_bit()
            });
        });
    }

    fn raw_read_client(&mut self) -> Option<u8> {
        avr_device::interrupt::free(|cs| -> Option<u8> {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            if state.bytes_processed < state.bytes_to_process {
                let data = state.data[state.bytes_processed as usize];
                state.bytes_processed += 1;
                Some(data)
            } else {
                None
            }
        })
    }

    fn raw_end_client(&mut self) {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.twi.saddr().write(|w| w.set(0));
            state.twi.sctrla().write(|w| w.enable().clear_bit());
        });
    }

    fn host_transmit(&mut self, send_stop: bool) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            // if disabled, abort
            if state.twi.mctrla().read().enable().bit_is_clear() {
                return Err(Error::Uninit);
            }

            let mut result = Ok(());

            // TODO: how to cleanly pack two bools into u8?
            let mut addr_sent = false;
            let mut data_sent = false;
            loop {
                let status = state.twi.mstatus().read();
                let bus_state = status.busstate();

                if bus_state.is_unknown() {
                    return Err(Error::Uninit);
                }

                if status.arblost().bit_is_set() {
                    return Err(Error::ArbitrationLost);
                }

                // wait for bus to be ready
                if bus_state.is_busy() {
                    continue;
                }

                // send address first
                if !addr_sent {
                    state
                        .twi
                        .maddr()
                        .write(|w| w.set(add_write_bit(state.client_address)));
                    addr_sent = true;
                    continue;
                }

                // wait for write to complete
                if status.wif().bit_is_set() {
                    // check if we got a NACK
                    if status.rxack().bit_is_set() {
                        if data_sent {
                            // ignore NACK if all data was sent
                            if state.bytes_to_process != 0 {
                                result = Err(Error::DataNack);
                            }
                        } else {
                            result = Err(Error::AddressNack);
                        }
                        break;
                    // else check if more bytes to send
                    } else if state.bytes_to_process > 0 {
                        state
                            .twi
                            .mdata()
                            .write(|w| w.set(state.data[state.bytes_processed as usize]));
                        state.bytes_processed += 1;
                        state.bytes_to_process -= 1;
                        data_sent = true;
                    // break when no more bytes to send
                    } else {
                        break;
                    }
                } // bus state busy check
            } // loop

            if send_stop || result.is_err() {
                state.twi.mctrlb().write(|w| w.mcmd().stop());
            }

            result
        })
    }
}

// #[avr_device::interrupt(attiny1604)]
// fn TWI0_TWIS_vect() {
//     avr_device::interrupt::free(|cs| {
//         let mut buffer = I2C_BUFFER.borrow(cs).borrow_mut();

//         // head => bytes_to_process
//         // tail => bytes_processed

//         let status = self.twi.mstatus().read();
//     });
// }
