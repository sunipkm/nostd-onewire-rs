#![allow(async_fn_in_trait)]
use crate::{Ds2484, Ds2484Error, traits::Addressing};
use embedded_hal_async::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};

/// Trait for interacting with the DS2484 I2C 1-Wire master asynchronously.
pub trait InteractAsync: Addressing {
    /// Read the register value from the DS2484 asynchronously.
    async fn async_read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
    /// Write the register value to the DS2484 asynchronously.
    async fn async_write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>>;
}
