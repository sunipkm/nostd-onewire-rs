#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

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
    DeviceConfiguration, DeviceStatus, Ds2484, Ds2484Builder, OneWireConfigurationBuilder,
    OneWirePortConfiguration,
};
pub use traits::Interact;
pub use traits_async::InteractAsync;

/// Results of DS2484-specific function calls.
pub type Ds2484Result<T, E> = Result<T, Ds2484Error<E>>;

mod test {

    #[test]
    fn test_ds2484() {
        use crate::registers::{DEVICE_RST_CMD, DEVICE_STATUS_PTR, READ_PTR_CMD};
        extern crate std;
        use super::*;
        use embedded_hal_mock::eh1::delay::NoopDelay as DelayMock;
        use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

        let mut i2c = I2cMock::new(&[
            I2cTransaction::write(0x18, std::vec![DEVICE_RST_CMD]), // write the reset command
            I2cTransaction::write_read(
                0x18,
                std::vec![READ_PTR_CMD, DEVICE_RST_CMD],
                std::vec![0x10],
            ), // set the read pointer to the device status and read the status
            I2cTransaction::write(0x18, std::vec![READ_PTR_CMD, DEVICE_STATUS_PTR]), // write the read pointer command
            I2cTransaction::read(0x18, std::vec![DeviceStatus::default().into_bits()]), // read the device status
            I2cTransaction::write(0x18, std::vec![0xd2, 0xf0]), // default configuration
            I2cTransaction::read(0x18, std::vec![0x00]),        // read the configuration
        ]);

        let delay = DelayMock::new();
        let mut ds2484 = Ds2484Builder::default().build(&mut i2c, delay).unwrap();
        let mut stat = DeviceStatus::default();
        stat.write(&mut ds2484).unwrap();
        i2c.done();
    }
}
