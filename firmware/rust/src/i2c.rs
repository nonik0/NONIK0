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
static I2C_BUFFER: avr_device::interrupt::Mutex<RefCell<I2cBuffer>> =
    avr_device::interrupt::Mutex::new(RefCell::new(I2cBuffer::new(0)));

struct I2cBuffer {
    data: [u8; I2C_BUFFER_SIZE],
    client_addr: u8,
    bytes_to_process: u8,
    bytes_processed: u8,
}

impl I2cBuffer {
    const fn new(address: u8) -> Self {
        Self {
            data: [0; I2C_BUFFER_SIZE],
            client_addr: address << 1, // byte 0 is rw bit, 0 for write
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

pub struct I2c {
    twi: TWI,
    _sda: Pin<Input<AnyInput>, SDAPIN>,
    _scl: Pin<Input<AnyInput>, SCLPIN>,
}

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
        let mut i2c = Self {
            twi,
            _sda: sda.forget_imode(),
            _scl: scl.forget_imode(),
        };
        i2c.raw_setup(speed);
        i2c
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

    #[inline]
    fn raw_setup(&mut self, speed: u32) {
        let baud = twi_baud(speed, 350) as u8; // hard-coded rise time estimate for now
        self.twi.mbaud().write(|w| w.set(baud));
        self.twi.mctrla().write(|w| w.enable().set_bit());
        self.twi
            .mstatus()
            .write(|w| w.busstate().idle().busstate().idle());
    }

    #[inline]
    fn raw_start(&mut self, address: u8, _direction: Direction) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut buffer = I2C_BUFFER.borrow(cs).borrow_mut();
            *buffer = I2cBuffer::new(address);
            Ok(())
        })
    }

    #[inline]
    fn raw_write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut buffer = I2C_BUFFER.borrow(cs).borrow_mut();
            let buf_start = buffer.bytes_to_process as usize;
            let buf_end = buf_start + bytes.len() as usize;

            if buf_end >= I2C_BUFFER_SIZE {
                return Err(Error::BufferOverflow);
            }

            buffer.data[buf_start..buf_end].copy_from_slice(bytes);
            buffer.bytes_to_process += bytes.len() as u8;
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

    fn host_transmit(&mut self, send_stop: bool) -> Result<(), Error> {
        // if disabled, abort
        if self.twi.mctrla().read().enable().bit_is_clear() {
            return Err(Error::Uninit);
        }

        avr_device::interrupt::free(|cs| {
            let mut buffer = I2C_BUFFER.borrow(cs).borrow_mut();
            let mut result = Ok(());

            // TODO: how to cleanly pack two bools into u8?
            let mut addr_sent = false;
            let mut data_sent = false;
            loop {
                let status = self.twi.mstatus().read();
                let state = status.busstate();

                if state.is_unknown() {
                    return Err(Error::Uninit);
                }

                if status.arblost().bit_is_set() {
                    return Err(Error::ArbitrationLost);
                }

                // wait for bus to be ready
                if state.is_busy() {
                    continue;
                }

                // send address first
                if !addr_sent {
                    self.twi
                        .maddr()
                        .write(|w| w.set(add_write_bit(buffer.client_addr)));
                    addr_sent = true;
                    continue;
                }
                    
                // wait for write to complete
                if status.wif().bit_is_set() {
                    // check if we got a NACK
                    if status.rxack().bit_is_set() {
                        if data_sent {
                            // ignore NACK if all data was sent
                            if buffer.bytes_to_process != 0 {
                                result = Err(Error::DataNack);
                            }
                        } else {
                            result = Err(Error::AddressNack);
                        }
                        break;
                    // else check if more bytes to sen
                    } else if buffer.bytes_to_process > 0 {
                        self.twi.mdata().write(|w| {
                            w.set(buffer.data[buffer.bytes_processed as usize])
                        });
                        buffer.bytes_processed += 1;
                        buffer.bytes_to_process -= 1;
                        data_sent = true;
                    // break when no more bytes to send
                    } else {
                        break;
                    }
                } // bus state busy check
            } // loop

            if send_stop || result.is_err() {
                self.twi.mctrlb().write(|w| w.mcmd().stop());
            }

            result
        })
    }
}
