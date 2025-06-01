use crate::{
    DeviceConfiguration, DeviceStatus, Ds2484Error, Ds2484Result, OneWirePortConfiguration,
    registers::{DEVICE_RST_CMD, DEVICE_STATUS_PTR, READ_PTR_CMD},
    traits_async::InteractAsync,
};
use embedded_hal_async::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};

/// A DS2484 I2C to 1-Wire bridge device.
///
/// Takes ownership of an I2C bus (implementing [`I2c`](embedded_hal_async::i2c::I2c) trait)
/// and a timer object implementing the [`DelayNs`](embedded_hal_async::delay::DelayNs) trait.
pub struct Ds2484Async<I, D> {
    pub(crate) i2c: I,
    pub(crate) addr: u8,
    pub(crate) delay: D,
    pub(crate) retries: u8,
}

impl<I, D> Ds2484Async<I, D> {
    /// Creates a new instance of [`Ds2484Async`] with the given I2C interface.
    pub fn new(i2c: I, delay: D) -> Self {
        Self {
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

impl<I: I2c<SevenBitAddress>, D: DelayNs> Ds2484Async<I, D> {
    /// Get the status of the device.
    pub async fn get_status(&mut self) -> Ds2484Result<DeviceStatus, I::Error> {
        let mut stat = DeviceStatus::default();
        stat.async_read(self).await?;
        Ok(stat)
    }
}

impl<I2C: I2c<SevenBitAddress>, D: DelayNs> Ds2484Async<I2C, D> {
    /// Reset the device.
    ///
    /// Performs a global reset of device state machine logic. Terminates any ongoing 1-Wire
    /// communication.
    pub async fn reset(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        self.i2c.write(self.addr, &[DEVICE_RST_CMD]).await?;
        let mut tries = 0;
        let status = 0;
        loop {
            self.i2c.read(self.addr, &mut [status]).await?;
            let status = DeviceStatus::from(status);
            if status.device_reset() || tries > self.retries {
                break;
            }
            tries += 1;
            self.delay.delay_ms(1).await;
        }
        let status: DeviceStatus = status.into();
        if tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }

    pub(crate) async fn onewire_wait(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        let mut tries = 0;
        let status = 0;
        self.i2c
            .write(self.addr, &[READ_PTR_CMD, DEVICE_STATUS_PTR])
            .await?;
        loop {
            self.i2c.read(self.addr, &mut [status]).await?;
            let status = DeviceStatus::from(status);
            if !status.onewire_busy() || tries > self.retries {
                break;
            }
            tries += 1;
            self.delay.delay_ms(1).await;
        }
        let status: DeviceStatus = status.into();
        if status.onewire_busy() && tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }
}

impl InteractAsync for DeviceStatus {
    const WRITE_ADDR: u8 = 0x0;

    const READ_PTR: u8 = 0xf0;

    async fn async_read<I: I2c<SevenBitAddress>, D>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let val = 0;
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut [val])
            .await?;
        *self = DeviceStatus::from(val);
        Ok(())
    }

    async fn async_write<I: I2c<SevenBitAddress>, D>(
        &mut self,
        _dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        Ok(())
    }
}

impl InteractAsync for DeviceConfiguration {
    const WRITE_ADDR: u8 = 0xd2;
    const READ_PTR: u8 = 0xc3;

    async fn async_read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let val = 0;
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut [val])
            .await?;
        *self = DeviceConfiguration::from(val);
        Ok(())
    }

    async fn async_write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait().await?;
        let val = 0;
        dev.i2c
            .write_read(dev.addr, &[Self::WRITE_ADDR, u8::from(*self)], &mut [val])
            .await?;
        *self = val.into();
        Ok(())
    }
}

impl InteractAsync for OneWirePortConfiguration {
    const WRITE_ADDR: u8 = 0xc3;

    const READ_PTR: u8 = 0xb4;

    async fn async_read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let mut buf = [0; 8];
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut buf)
            .await?;
        *self = Self::from_bytes(buf);
        Ok(())
    }

    async fn async_write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484Async<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait().await?;
        dev.i2c.write(dev.addr, &self.to_bytes()).await?;
        self.async_read(dev).await
    }
}
