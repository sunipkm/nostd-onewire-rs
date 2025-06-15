use crate::{Ds2484, Ds2484Error};
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};

/// Addresses of registers in the DS2484.
pub trait Addressing {
    /// Register address for writing to the DS2484.
    const WRITE_ADDR: u8;
    /// Pointer address for reading from the DS2484.
    const READ_PTR: u8;
}

/// Trait for interacting with the DS2484 I2C 1-Wire master.
pub trait Interact: Addressing {
    /// Read the register value from the DS2484.
    fn read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
    /// Write the register value to the DS2484.
    fn write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
}
