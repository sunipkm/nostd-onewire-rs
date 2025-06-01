use crate::{
    Ds2484, Ds2484Error,
    registers::{DeviceStatus, READ_PTR_CMD},
};
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};
use embedded_onewire::{OneWire, OneWireError, OneWireResult, OneWireStatus};

const ONEWIRE_RESET_CMD: u8 = 0xb4;
const ONEWIRE_WRITE_BYTE: u8 = 0xa5;
const ONEWIRE_READ_BYTE: u8 = 0x96;
const ONEWIRE_READ_DATA_PTR: u8 = 0xe1;
const ONEWIRE_SINGLE_BIT: u8 = 0x87;
const ONEWIRE_TRIPLET: u8 = 0x78;

impl<I2C: I2c<SevenBitAddress>, D: DelayNs> OneWire for Ds2484<I2C, D> {
    type Status = DeviceStatus;

    type BusError = Ds2484Error<I2C::Error>;

    fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError> {
        self.onewire_wait()?;
        self.i2c
            .write(self.addr, &[ONEWIRE_RESET_CMD])
            .map_err(Ds2484Error::from)?;
        self.onewire_wait().map(|v| {
            if v.short_detect() {
                Err(OneWireError::ShortCircuit)
            } else if !v.presence() {
                Err(OneWireError::NoDevicePresent)
            } else {
                Ok(v)
            }
        })?
    }

    fn write_byte(&mut self, byte: u8) -> OneWireResult<(), Self::BusError> {
        self.onewire_wait()?;
        self.i2c
            .write(self.addr, &[ONEWIRE_WRITE_BYTE, byte])
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError> {
        self.onewire_wait()?;
        self.i2c
            .write(self.addr, &[ONEWIRE_READ_BYTE])
            .map_err(Ds2484Error::from)?;
        self.onewire_wait()?;
        let val = 0;
        self.i2c
            .write_read(
                self.addr,
                &[READ_PTR_CMD, ONEWIRE_READ_DATA_PTR],
                &mut [val],
            )
            .map_err(Ds2484Error::from)?;
        Ok(val)
    }

    fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError> {
        self.onewire_wait()?;
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_SINGLE_BIT, { if bit { 0x80 } else { 0x0 } }],
            )
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    fn read_bit(&mut self) -> OneWireResult<bool, Self::BusError> {
        self.write_bit(true)?;
        Ok(self.onewire_wait()?.single_bit_result())
    }

    fn read_triplet(&mut self, direction: bool) -> OneWireResult<(bool, bool), Self::BusError> {
        self.onewire_wait()?;
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_TRIPLET, { if direction { 0x80 } else { 0x0 } }],
            )
            .map_err(Ds2484Error::from)?;
        Ok(self
            .onewire_wait()
            .map(|v| (v.single_bit_result(), v.triplet_second_bit()))?)
    }
}
