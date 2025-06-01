use crate::{
    Ds2484Async, Ds2484Error,
    registers::{DeviceStatus, READ_PTR_CMD},
};
use embedded_hal_async::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};
use embedded_onewire::{OneWireAsync, OneWireError, OneWireResult, OneWireStatus};

const ONEWIRE_RESET_CMD: u8 = 0xb4;
const ONEWIRE_WRITE_BYTE: u8 = 0xa5;
const ONEWIRE_READ_BYTE: u8 = 0x96;
const ONEWIRE_READ_DATA_PTR: u8 = 0xe1;
const ONEWIRE_SINGLE_BIT: u8 = 0x87;
const ONEWIRE_TRIPLET: u8 = 0x78;

impl<I2C: I2c<SevenBitAddress>, D: DelayNs> OneWireAsync for Ds2484Async<I2C, D> {
    type Status = DeviceStatus;

    type BusError = Ds2484Error<I2C::Error>;

    async fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError> {
        self.onewire_wait().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_RESET_CMD]).await
            .map_err(Ds2484Error::from)?;
        self.onewire_wait().await.map(|v| {
            if v.short_detect() {
                Err(OneWireError::ShortCircuit)
            } else if !v.presence() {
                Err(OneWireError::NoDevicePresent)
            } else {
                Ok(v)
            }
        })?
    }

    async fn write_byte(&mut self, byte: u8) -> OneWireResult<(), Self::BusError> {
        self.onewire_wait().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_WRITE_BYTE, byte]).await
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    async fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError> {
        self.onewire_wait().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_READ_BYTE]).await
            .map_err(Ds2484Error::from)?;
        self.onewire_wait().await?;
        let val = 0;
        self.i2c
            .write_read(
                self.addr,
                &[READ_PTR_CMD, ONEWIRE_READ_DATA_PTR],
                &mut [val],
            ).await
            .map_err(Ds2484Error::from)?;
        Ok(val)
    }

    async fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError> {
        self.onewire_wait().await?;
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_SINGLE_BIT, { if bit { 0x80 } else { 0x0 } }],
            ).await
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    async fn read_bit(&mut self) -> OneWireResult<bool, Self::BusError> {
        self.write_bit(true).await?;
        Ok(self.onewire_wait().await?.single_bit_result())
    }

    async fn read_triplet(&mut self, direction: bool) -> OneWireResult<(bool, bool), Self::BusError> {
        self.onewire_wait().await?;
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_TRIPLET, { if direction { 0x80 } else { 0x0 } }],
            ).await
            .map_err(Ds2484Error::from)?;
        Ok(self
            .onewire_wait().await
            .map(|v| (v.single_bit_result(), v.triplet_second_bit()))?)
    }
}
