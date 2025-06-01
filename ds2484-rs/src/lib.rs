#![no_std]
#![deny(missing_docs)]

/*! # DS2484
 *
 *
 *
 */

pub use embedded_onewire::{OneWire, OneWireAsync, OneWireError, OneWireResult};
mod error;
mod onewire;
mod onewire_async;
mod registers;
mod registers_async;
mod traits;
mod traits_async;

pub use error::Ds2484Error;
pub use registers::{
    DeviceConfiguration, DeviceStatus, OneWireConfigurationBuilder, OneWirePortConfiguration,
};
pub use registers_async::Ds2484Async;
pub use traits::Interact;
pub use traits_async::InteractAsync;

/// Results of DS2484-specific function calls.
pub type Ds2484Result<T, E> = Result<T, Ds2484Error<E>>;

/// A DS2484 I2C to 1-Wire bridge device.
///
/// Takes ownership of an I2C bus (implementing [`I2c`](embedded_hal::i2c::I2c) trait)
/// and a timer object implementing the [`DelayNs`](embedded_hal::delay::DelayNs) trait.
pub struct Ds2484<I, D> {
    pub(crate) i2c: I,
    pub(crate) addr: u8,
    pub(crate) delay: D,
    pub(crate) retries: u8,
}

impl<I, D> Ds2484<I, D> {
    /// Creates a new instance of `Ds2484` with the given I2C interface.
    pub fn new(i2c: I, delay: D) -> Self {
        Ds2484 {
            i2c,
            addr: 0x18,
            delay,
            retries: 100,
        }
    }

    /// Set the retry count.
    ///
    /// The retry count is used to determine how long
    /// the host waits before operations on the 1-Wire
    /// or I2C bus time out.
    pub fn with_retries(mut self, retries: u8) -> Self {
        self.retries = retries;
        self
    }
}
