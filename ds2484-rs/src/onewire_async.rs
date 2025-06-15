#[cfg(feature = "triplet-read")]
use crate::onewire::ONEWIRE_TRIPLET;
use crate::{
    DeviceConfiguration, Ds2484, Ds2484Error, InteractAsync,
    onewire::{
        ONEWIRE_READ_BYTE, ONEWIRE_READ_DATA_PTR, ONEWIRE_RESET_CMD, ONEWIRE_SINGLE_BIT,
        ONEWIRE_WRITE_BYTE,
    },
    registers::{DeviceStatus, READ_PTR_CMD},
};
use embedded_hal_async::{
    delay::DelayNs as DelayNsAsync,
    i2c::{I2c as I2cAsync, SevenBitAddress as SevenBitAddressAsync},
};
use embedded_onewire::{OneWireAsync, OneWireError, OneWireResult, OneWireStatus};

impl<I2C: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync> OneWireAsync for Ds2484<I2C, D> {
    type Status = DeviceStatus;

    type BusError = Ds2484Error<I2C::Error>;

    async fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError> {
        self.onewire_wait_async().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_RESET_CMD])
            .await
            .map_err(Ds2484Error::from)?;
        self.onewire_wait_async().await.map(|v| {
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
        self.onewire_wait_async().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_WRITE_BYTE, byte])
            .await
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    async fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError> {
        self.onewire_wait_async().await?;
        self.i2c
            .write(self.addr, &[ONEWIRE_READ_BYTE])
            .await
            .map_err(Ds2484Error::from)?;
        self.onewire_wait_async().await?;
        let mut val = [0; 1];
        self.i2c
            .write_read(self.addr, &[READ_PTR_CMD, ONEWIRE_READ_DATA_PTR], &mut val)
            .await
            .map_err(Ds2484Error::from)?;
        Ok(val[0])
    }

    async fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError> {
        self.onewire_wait_async().await?;
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_SINGLE_BIT, { if bit { 0x80 } else { 0x0 } }],
            )
            .await
            .map_err(Ds2484Error::from)?;
        Ok(())
    }

    async fn read_bit(&mut self) -> OneWireResult<bool, Self::BusError> {
        self.write_bit(true).await?;
        Ok(self.onewire_wait_async().await?.single_bit_result())
    }

    #[cfg(feature = "triplet-read")]
    async fn read_triplet(&mut self) -> OneWireResult<(bool, bool, bool), Self::BusError> {
        let direction = self.onewire_wait_async().await?.branch_dir_taken();
        self.i2c
            .write(
                self.addr,
                &[ONEWIRE_TRIPLET, { if direction { 0xff } else { 0x0 } }],
            )
            .await
            .map_err(Ds2484Error::from)?;
        Ok(self.onewire_wait_async().await.map(|v| {
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

    async fn set_overdrive_mode(&mut self, enable: bool) -> OneWireResult<(), Self::BusError> {
        let mut config = DeviceConfiguration::new();
        config.async_read(self).await?;
        let cur = config.onewire_speed();
        if enable == cur {
            return Ok(()); // No change needed
        }
        if !cur {
            // not currently in overdrive mode
            self.reset().await?;
            self.address(None).await?;
            config.set_onewire_speed(true);
            config.async_write(self).await?;
            self.overdrive = true;
            self.reset().await?; // reset the bus to apply changes
        } else {
            config.set_onewire_speed(false);
            config.async_write(self).await?;
            self.overdrive = false;
            self.reset().await?; // reset the bus to apply changes
        }
        Ok(())
    }
}
