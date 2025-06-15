use crate::{
    DeviceConfiguration, DeviceStatus, Ds2484, Ds2484Error, Ds2484Result, OneWirePortConfiguration,
    registers::{DEVICE_RST_CMD, DEVICE_STATUS_PTR, READ_PTR_CMD},
    traits::Addressing,
    traits_async::InteractAsync,
};
use embedded_hal_async::{
    delay::DelayNs as DelayNsAsync,
    i2c::{I2c as I2cAsync, SevenBitAddress as SevenBitAddressAsync},
};

impl<I: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync> Ds2484<I, D> {
    /// Get the status of the device.
    pub async fn get_status_async(&mut self) -> Ds2484Result<DeviceStatus, I::Error> {
        let mut stat = DeviceStatus::default();
        stat.async_read(self).await?;
        Ok(stat)
    }
}

impl<I2C: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync> Ds2484<I2C, D> {
    /// Reset the device.
    ///
    /// Performs a global reset of device state machine logic. Terminates any ongoing 1-Wire
    /// communication.
    pub async fn bus_reset_async(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        self.i2c.write(self.addr, &[DEVICE_RST_CMD]).await?;
        self.reset = true;
        let mut tries = 0;
        let mut status = [0; 1];
        loop {
            self.i2c.read(self.addr, &mut status).await?;
            let status = DeviceStatus::from(status[0]);
            if status.device_reset() || tries > self.retries {
                break;
            }
            tries += 1;
            self.delay.delay_ms(1).await;
        }
        let status: DeviceStatus = status[0].into();
        if tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }

    pub(crate) async fn onewire_wait_async(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        let mut tries = 0;
        let mut status = [0; 1];
        self.i2c
            .write(self.addr, &[READ_PTR_CMD, DEVICE_STATUS_PTR])
            .await?;
        loop {
            self.i2c.read(self.addr, &mut status).await?;
            let status = DeviceStatus::from(status[0]);
            if !status.onewire_busy() || tries > self.retries {
                break;
            }
            tries += 1;
            if !self.overdrive {
                self.delay.delay_ms(1).await;
            } else {
                self.delay.delay_us(100).await;
            }
        }
        let status: DeviceStatus = status[0].into();
        if status.onewire_busy() && tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }
}

impl InteractAsync for DeviceStatus {
    async fn async_read<I: I2cAsync<SevenBitAddressAsync>, D>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let mut val = [0; 1];
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut val)
            .await?;
        *self = DeviceStatus::from(val[0]);
        Ok(())
    }

    async fn async_write<I: I2cAsync<SevenBitAddressAsync>, D>(
        &mut self,
        _dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        Ok(())
    }
}

impl InteractAsync for DeviceConfiguration {
    async fn async_read<I: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let mut val = [0; 1];
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut val)
            .await?;
        *self = DeviceConfiguration::from(val[0]);
        Ok(())
    }

    async fn async_write<I: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait_async().await?;
        let out = u8::from(*self);
        let out = (out & 0x0f) | ((!out & 0x0f) << 4);
        let mut val = [0; 1];
        dev.i2c
            .write_read(dev.addr, &[Self::WRITE_ADDR, out], &mut val)
            .await?;
        *self = val[0].into();
        dev.reset = false; // Clear the reset flag after writing configuration
        Ok(())
    }
}

impl InteractAsync for OneWirePortConfiguration {
    async fn async_read<I: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let mut buf = [0; 8];
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut buf)
            .await?;
        *self = Self::from_bytes(buf);
        Ok(())
    }

    async fn async_write<I: I2cAsync<SevenBitAddressAsync>, D: DelayNsAsync>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait_async().await?;
        dev.i2c.write(dev.addr, &self.to_bytes()).await?;
        self.async_read(dev).await
    }
}
