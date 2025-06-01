#![allow(async_fn_in_trait)]
use crate::{Ds2484Async, Ds2484Error};
use embedded_hal_async::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};

/// Trait for interacting with the DS2484 I2C 1-Wire master asynchronously.
pub trait InteractAsync {
    /// The register command to write to the DS2484.
    const WRITE_ADDR: u8;
    /// Read pointer location for the register.
    const READ_PTR: u8;
    /// Read the register value from the DS2484 asynchronously.
    async fn async_read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
    /// Write the register value to the DS2484 asynchronously.
    async fn async_write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
}
