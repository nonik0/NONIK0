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

// TODO: investigate issues with chunking data and consecutive writes from host
// for now, specific feature enables large buffer for use as I2C client
#[cfg(feature = "i2c_client")]
pub const I2C_BUFFER_SIZE: usize = 256; 
#[cfg(not(feature = "i2c_client"))]
pub const I2C_BUFFER_SIZE: usize = 32;
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
    bytes_transmitted: u8, // client response
    client_check_nak: bool,
    host_data_sent: bool,
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
            bytes_transmitted: 0,
            client_check_nak: false,
            host_data_sent: false,
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

    pub fn end(&mut self) {
        self.raw_end();
        self.raw_end_client();
    }

    //
    // HOST PUBLIC
    //
    pub fn host_setup(&mut self, speed: u32) {
        self.raw_setup(speed);
    }

    pub fn host_ping_device(&mut self, address: u8, direction: Direction) -> Result<bool, Error> {
        self.raw_start(address, direction)?;
        match self.raw_stop() {
            Ok(_) => Ok(true),
            Err(Error::AddressNack) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn host_write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        self.raw_start(address, Direction::Write)?;
        self.raw_write(bytes)?;
        self.raw_stop()?;
        Ok(())
    }

    pub fn host_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.raw_start(address, Direction::Read)?;
        self.raw_read(buffer, true)?;
        self.raw_stop()?;
        Ok(())
    }

    pub fn host_write_read(
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

    //
    // CLIENT PUBLIC
    //
    pub fn client_setup(&mut self, address: u8) {
        self.raw_setup_client(address);
    }

    pub fn client_available(&self) -> u8 {
        self.raw_available_client()
    }

    pub fn client_read(&mut self) -> Option<u8> {
        self.raw_read_client()
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
    fn raw_read(&mut self, buffer: &mut [u8], _last_read: bool) -> Result<(), Error> {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.bytes_processed = 0;
            state.bytes_to_process = buffer.len() as u8;
            if state.bytes_to_process > I2C_BUFFER_SIZE as u8 {
                state.bytes_to_process = I2C_BUFFER_SIZE as u8;
            }

            // TODO: receive bytes blocking
            Ok(())
        })
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

    fn raw_available_client(&self) -> u8 {
        avr_device::interrupt::free(|cs| {
            let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
            let state = state_opt.as_mut().unwrap();

            state.bytes_to_process - state.bytes_processed
        })
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
                // clear buffer once all data is read (TODO: not needed with callback)
                if state.bytes_processed > 0 {
                    state.bytes_to_process = 0;
                    state.bytes_processed = 0;
                }
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

            let mut addr_sent = false;
            let mut data_sent = false;
            
            let result = loop {
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
                                break Err(Error::DataNack);
                            } else {
                                break Ok(());
                            }
                        } else {
                            break Err(Error::AddressNack);
                        }
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
                        break Ok(());
                    }
                }
            };

            if send_stop || result.is_err() {
                state.twi.mctrlb().write(|w| w.mcmd().stop());
            }

            result
        })
    }
}

#[avr_device::interrupt(attiny1604)]
fn TWI0_TWIS() {
    avr_device::interrupt::free(|cs| {
        let mut state_opt = I2C_STATE.borrow(cs).borrow_mut();
        let state = state_opt.as_mut().unwrap();

        enum Response {
            None,
            AckContinue,
            AckComplete,
            NakComplete,
        }
        let mut response = Response::None;
        let client_status = state.twi.sstatus().read();

        // address or stop condition detected
        if client_status.apif().bit_is_set() {
            // host is done sending data
            if state.host_data_sent {
                state.host_data_sent = false;

                // TODO: callback
                // if state.on_receive {
                //     on_receive(state.bytes_to_process);
                // }
            }

            // address detected (START/RESTART condition)
            if client_status.ap().bit_is_set() {
                state.client_address = state.twi.sdata().read().bits();
                state.bytes_to_process = 0;
                state.bytes_processed = 0;

                // host is reading
                if client_status.dir().bit_is_set() {
                    // TODO: callback
                    // if state.on_request {
                    //     state.on_request();
                    // }

                    // response based on whether there is data to send
                    response = if state.bytes_to_process == 0 {
                        Response::NakComplete
                    } else {
                        Response::AckContinue
                    };
                }
                // host is writing
                else {
                    state.host_data_sent = true;
                    response = Response::AckContinue;
                }
            }
            // STOP condition detected
            else {
                // TODO: callback
                // state.bytes_to_process = 0;
                // state.bytes_processed = 0;
                response = Response::AckComplete;
            }
        }
        // data received
        else if client_status.dif().bit_is_set() {
            // host is reading
            if client_status.dir().bit_is_set() {
                // collision detected
                let nak = state.client_check_nak && client_status.rxack().bit_is_set();
                let collision = client_status.coll().bit_is_set();
                if nak || collision {
                    state.client_check_nak = false;
                    state.bytes_to_process = 0;
                    response = Response::AckComplete;
                }
                // data ACKed, continue sending
                else {
                    state.bytes_transmitted += 1;
                    state.client_check_nak = true;

                    // send more data
                    if state.bytes_processed < state.bytes_to_process {
                        let data = state.data[state.bytes_processed as usize];
                        state.twi.sdata().write(|w| w.set(data));
                        state.bytes_processed += 1;
                        response = Response::AckContinue;
                    }
                    // no more data to send
                    else {
                        state.bytes_to_process = 0;
                        response = Response::AckComplete;
                    }
                }
            }
            // host is writing
            else {
                let data = state.twi.sdata().read().bits();

                // check if buffer has space
                if state.bytes_to_process < I2C_BUFFER_SIZE as u8 {
                    state.data[state.bytes_to_process as usize] = data;
                    state.bytes_to_process += 1;

                    // response based on whether buffer is full
                    response = if state.bytes_to_process == I2C_BUFFER_SIZE as u8 {
                        Response::NakComplete
                    } else {
                        Response::AckContinue
                    };
                }
            }
        }

        match response {
            Response::None => {}
            Response::AckContinue => {
                state.twi.sctrlb().write(|w| w.scmd().response());
            }
            Response::AckComplete => {
                state.twi.sctrlb().write(|w| w.scmd().comptrans());
            }
            Response::NakComplete => {
                state
                    .twi
                    .sctrlb()
                    .write(|w| w.ackact().set_bit().scmd().comptrans());
            }
        }
    });
}