use crate::{
    DeviceConfiguration, Ds2484, Ds2484Error, Interact,
    registers::{DeviceStatus, READ_PTR_CMD},
};
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};
use embedded_onewire::{
    OneWire, OneWireError, OneWireResult, OneWireStatus, consts::ONEWIRE_SKIP_ROM_CMD_OD,
};

pub(crate) const ONEWIRE_RESET_CMD: u8 = 0xb4;
pub(crate) const ONEWIRE_WRITE_BYTE: u8 = 0xa5;
pub(crate) const ONEWIRE_READ_BYTE: u8 = 0x96;
pub(crate) const ONEWIRE_READ_DATA_PTR: u8 = 0xe1;
pub(crate) const ONEWIRE_SINGLE_BIT: u8 = 0x87;
#[cfg(feature = "triplet-read")]
pub(crate) const ONEWIRE_TRIPLET: u8 = 0x78;

impl<I2C: I2c<SevenBitAddress>, D: DelayNs> OneWire for Ds2484<I2C, D> {
    type Status = DeviceStatus;

    type BusError = Ds2484Error<I2C::Error>;

    fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError> {
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
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
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
        self.onewire_wait()?;
        self.i2c
            .write(self.addr, &[ONEWIRE_WRITE_BYTE, byte])
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError> {
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
        self.onewire_wait()?;
        self.i2c
            .write(self.addr, &[ONEWIRE_READ_BYTE])
            .map_err(Ds2484Error::from)?;
        self.onewire_wait()?;
        let mut val = [0; 1];
        self.i2c
            .write_read(self.addr, &[READ_PTR_CMD, ONEWIRE_READ_DATA_PTR], &mut val)
            .map_err(Ds2484Error::from)?;
        Ok(val[0])
    }

    fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError> {
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
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
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
        self.write_bit(true)?;
        Ok(self.onewire_wait()?.single_bit_result())
    }

    #[cfg(feature = "triplet-read")]
    fn read_triplet(&mut self) -> OneWireResult<(bool, bool, bool), Self::BusError> {
        if self.reset {
            return Err(OneWireError::BusUninitialized);
        }
        let direction = self.onewire_wait()?.branch_dir_taken();
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_TRIPLET, { if direction { 0xff } else { 0x0 } }],
            )
            .map_err(Ds2484Error::from)?;
        Ok(self.onewire_wait().map(|v| {
            (
                v.single_bit_result(),
                v.triplet_second_bit(),
                v.branch_dir_taken(),
            )
        })?)
    }

    fn get_overdrive_mode(&mut self) -> bool {
        self.overdrive
    }

    fn set_overdrive_mode(&mut self, enable: bool) -> OneWireResult<(), Self::BusError> {
        let mut config = DeviceConfiguration::new();
        config.read(self)?;
        let cur = config.onewire_speed();
        if enable == cur {
            return Ok(()); // No change needed
        }
        if !cur {
            // not currently in overdrive mode
            self.reset()?;
            self.write_byte(ONEWIRE_SKIP_ROM_CMD_OD)?;
            config.set_onewire_speed(true);
            config.write(self)?;
            self.overdrive = true;
            self.reset()?; // reset the bus to apply changes
        } else {
            config.set_onewire_speed(false);
            config.write(self)?;
            self.overdrive = false;
            self.reset()?; // reset the bus to apply changes
        }
        Ok(())
    }
}
