use crate::{Ds2484Error, Ds2484Result, traits::Interact};
use bitfield_struct::bitfield;
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};
use embedded_onewire::OneWireStatus;

pub(crate) const READ_PTR_CMD: u8 = 0xe1; // Set the read pointer
pub(crate) const DEVICE_STATUS_PTR: u8 = 0xf0; // Device status register
pub(crate) const DEVICE_RST_CMD: u8 = 0xf0; // Reset the device

/// A DS2484 I2C to 1-Wire bridge device.
///
/// Takes ownership of an I2C bus (implementing [`I2c`](embedded_hal::i2c::I2c) trait)
/// and a timer object implementing the [`DelayNs`](embedded_hal::delay::DelayNs) trait.
pub struct Ds2484<I, D> {
    pub(crate) i2c: I,
    pub(crate) addr: u8,
    pub(crate) delay: D,
    pub(crate) retries: u8,
    pub(crate) reset: bool, // Indicates if the device has been reset
    pub(crate) overdrive: bool,
}

/// Builder for creating a [`Ds2484`] instance with custom configuration.
pub struct Ds2484Builder {
    pub(crate) retries: u8,
    pub(crate) config: DeviceConfiguration,
}

impl Default for Ds2484Builder {
    fn default() -> Self {
        Ds2484Builder {
            retries: 100,
            config: DeviceConfiguration::new(),
        }
    }
}

impl Ds2484Builder {
    /// Sets the retry count for the device.
    ///
    /// The retry count is used to determine how long
    /// the host waits before operations on the 1-Wire
    /// or I2C bus time out.
    pub fn with_retries(mut self, retries: u8) -> Self {
        self.retries = retries;
        self
    }

    /// Sets the device configuration.
    pub fn with_config(mut self, config: DeviceConfiguration) -> Self {
        self.config = config;
        self
    }

    /// Builds a new `Ds2484` instance with the specified configuration.
    pub fn build<I: I2c<SevenBitAddress>, D: DelayNs>(
        mut self,
        i2c: I,
        delay: D,
    ) -> Ds2484Result<Ds2484<I, D>, I::Error> {
        let mut dev = Ds2484 {
            i2c,
            addr: 0x18,
            delay,
            retries: self.retries,
            reset: false,
            overdrive: false,
        };
        dev.bus_reset()?;
        self.config.write(&mut dev)?;
        dev.overdrive = self.config.onewire_speed();
        Ok(dev)
    }
}

impl<I: I2c<SevenBitAddress>, D: DelayNs> Ds2484<I, D> {
    /// Get the status of the device.
    pub fn get_status(&mut self) -> Ds2484Result<DeviceStatus, I::Error> {
        let mut stat = DeviceStatus::default();
        stat.read(self)?;
        Ok(stat)
    }
}

impl<I2C: I2c<SevenBitAddress>, D: DelayNs> Ds2484<I2C, D> {
    /// Reset the device.
    ///
    /// Performs a global reset of device state machine logic. Terminates any ongoing 1-Wire
    /// communication.
    pub fn bus_reset(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        self.i2c.write(self.addr, &[DEVICE_RST_CMD])?;
        self.reset = true;
        let mut tries = 0;
        let status = DeviceStatus::default();
        loop {
            self.i2c.read(self.addr, &mut [status.0])?;
            if status.device_reset() || tries > self.retries {
                break;
            }
            tries += 1;
            self.delay.delay_ms(1);
        }
        if tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }

    pub(crate) fn onewire_wait(&mut self) -> Ds2484Result<DeviceStatus, I2C::Error> {
        let mut tries = 0;
        let status = DeviceStatus::default();
        self.i2c
            .write(self.addr, &[READ_PTR_CMD, DEVICE_STATUS_PTR])?;
        loop {
            self.i2c.read(self.addr, &mut [status.0])?;
            if !status.onewire_busy() || tries > self.retries {
                break;
            }
            tries += 1;
            self.delay.delay_ms(1);
        }

        if status.onewire_busy() && tries > self.retries {
            Err(Ds2484Error::RetriesExceeded)
        } else {
            Ok(status)
        }
    }
}

/// Status register for DS2484
/// The read-only Status register is the general means for
/// the DS2484 to report bit-type data from the 1-Wire side,
/// 1-Wire busy status, and its own reset status to the host
/// processor ([Table 3](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2014%3A9051)).
/// All 1-Wire communication commands
/// and the [Device Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2021%3A9058)
/// command position the read pointer
/// at the Status register for the host processor to read with
/// minimal protocol overhead. Status information is updated
/// during the execution of certain commands only. Bit details
/// are given in the following descriptions.
#[bitfield(u8)]
pub struct DeviceStatus {
    /// The 1WB bit reports to the host processor whether the
    /// 1-Wire line is busy. During 1-Wire communication 1WB
    /// is 1; once the command is completed, 1WB returns to
    /// its default 0. Details on when 1WB changes state and
    /// for how long it remains at 1 are found in the
    /// [Function Commands](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2010%3A9047) section.
    pub(crate) onewire_busy: bool,
    /// The PPD bit is updated with every [1-Wire Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2031%3A9139) command.
    /// If the DS2484 detects a logic 0 on the 1-Wire line
    /// at tMSP during the presence-detect cycle, the PPD bit is
    /// set to 1. This bit returns to its default 0 if there is no
    /// presence pulse during a subsequent [1-Wire Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2031%3A9139) command.
    present_pulse_detect: bool,
    /// The SD bit is updated with every [1-Wire Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2031%3A9139) command.
    /// If the DS2484 detects a logic 0 on the 1-Wire line
    /// at tSI during the presence-detect cycle, the SD bit is set
    /// to 1. This bit returns to its default 0 with a subsequent
    /// [1-Wire Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2031%3A9139) command, provided that the short has been
    /// removed. If the 1-Wire line is shorted at t MSP, the PPD
    /// bit is also set. The DS2484 cannot distinguish between a
    /// short and a DS1994 or DS2404 signaling a 1-Wire interrupt.
    /// For this reason, if a DS2404 or DS1994 is used in
    /// the application, the interrupt function must be disabled.
    /// The interrupt signaling is explained in the respective
    /// Analog Devices 1-Wire IC data sheets.
    pub(crate) short_detect: bool,
    /// The LL bit reports the logic state of the active 1-Wire line
    /// without initiating any 1-Wire communication. The 1-Wire
    /// line is sampled for this purpose every time the Status
    /// register is read. The sampling and updating of the LL bit
    /// takes place when the host processor has addressed the
    /// DS2484 in read mode (during the acknowledge cycle),
    /// provided that the read pointer is positioned at the Status
    /// register.
    pub logic_level: bool,
    /// If the RST bit is 1, the DS2484 has performed an internal
    /// reset cycle, either caused by a power-on reset or from
    /// executing the [Device Reset](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2021%3A9058) command. The RST bit is
    /// cleared automatically when the DS2484 executes a [Write Device Configuration](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2015%3A9052) command to restore the selection of
    /// the desired 1-Wire features.
    pub device_reset: bool,
    /// The SBR bit reports the logic state of the active 1-Wire
    /// line sampled at t MSR of a [1-Wire Single Bit](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2016%3A9053) command or
    /// the first bit of a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054) command. The power-on
    /// default of SBR is 0. If the [1-Wire Single Bit](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2016%3A9053) command
    /// sends a 0 bit, SBR should be 0. With a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054)
    /// command, SBR could be 0 as well as 1, depending on
    /// the response of the 1-Wire devices connected. The same
    /// result applies to a [1-Wire Single Bit](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2016%3A9053) command that sends
    /// a 1 bit.
    pub(crate) single_bit_result: bool,
    /// The TSB bit reports the logic state of the active 1-Wire
    /// line sampled at t MSR of the second bit of a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054)
    /// command. The power-on default of TSB is 0. This bit is
    /// updated only with a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054) command and has no
    /// function with other commands.
    pub(crate) triplet_second_bit: bool,
    /// Whenever a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054) command is executed, this bit
    /// reports to the host processor the search direction that was
    /// chosen by the third bit of the triplet. The power-on default
    /// of DIR is 0. This bit is updated only with a [1-Wire Triplet](https://www.analog.com/media/en/technical-documentation/data-sheets/ds2484.pdf#DS2484%20DS.indd%3AAnchor%2017%3A9054)
    /// command and has no function with other commands. For
    /// additional information, see the description of the 1-Wire
    /// Triplet command and the [1-Wire Search Algorithm](https://www.analog.com/en/resources/app-notes/1wire-search-algorithm.html) application note.
    pub(crate) branch_dir_taken: bool,
}

impl OneWireStatus for DeviceStatus {
    fn presence(&self) -> bool {
        self.present_pulse_detect()
    }

    fn shortcircuit(&self) -> bool {
        self.short_detect()
    }

    fn logic_level(&self) -> Option<bool> {
        Some(self.logic_level())
    }

    fn direction(&self) -> Option<bool> {
        Some(self.branch_dir_taken())
    }
}

impl Interact for DeviceStatus {
    const WRITE_ADDR: u8 = 0x0;

    const READ_PTR: u8 = 0xf0;

    fn read<I: I2c<SevenBitAddress>, D>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut [self.0])?;
        Ok(())
    }

    fn write<I: I2c<SevenBitAddress>, D>(
        &mut self,
        _dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        Ok(())
    }
}

#[bitfield(u8, into=cfg_to_u8)]
/// # Device configuration register
///
/// The DS2484 supports four 1-Wire features that are
/// enabled or selected through the Device Configuration
/// register (Table 1). These features are as follows:
/// - Active Pullup (APU)
/// - 1-Wire Power-Down (PDN)
/// - Strong Pullup (SPU)
/// - 1-Wire Speed (1WS)
/// APU, SPU, and 1WS can be selected in any combination.
/// While APU and 1WS maintain their states, SPU returns to
/// its inactive state as soon as the strong pullup has ended.
///
/// After a device reset (power-up cycle or initiated by the
/// Device Reset command), the Device Configuration register
/// reads 00h.
pub struct DeviceConfiguration {
    /// The APU bit controls whether an active pullup (low impedance
    /// transistor) or a passive pullup (R WPU resistor) is
    /// used to drive a 1-Wire line from low to high. When APU
    /// = 0, active pullup is disabled (resistor mode). Enabling
    /// active pullup is generally recommended for best 1-Wire
    /// bus performance. The active pullup does not apply
    /// to the rising edge of a recovery after a short on the
    /// 1-Wire line. If enabled, a fixed-duration active pullup
    /// (typically 2.5µs standard speed, 0.5µs overdrive speed)
    /// also applies in a reset/presence detect cycle on the rising
    /// edges after t_RSTL and after t_PDL .
    pub active_pullup: bool,
    /// The PDN bit is used to remove power from the 1-Wire
    /// port, e.g., to force a 1-Wire slave to perform a power-on
    /// reset. PDN can as well be used in conjunction with the
    /// sleep mode (see Table 2 for details). While PDN is 1,
    /// no 1-Wire communication is possible. To end the 1-Wire
    /// power-down state, the PDN bit must be changed to 0.
    /// Writing both the PDN bit and the SPU bit to 1 forces the
    /// SPU bit to 0. With the DS2483, both bits can be written
    /// to 1, which can be used to logically distinguish between
    /// both parts.
    pub power_down_1wire: bool,
    /// The SPU bit is used to activate the strong pullup func-
    /// tion prior to a 1-Wire Write Byte or 1-Wire Single Bit
    /// command. Strong pullup is commonly used with 1-Wire
    /// EEPROM devices when copying scratchpad data to the
    /// main memory or when performing a SHA computation
    /// and with parasitically powered temperature sensors or
    /// A/D converters. The respective Analog Devices 1-Wire IC
    /// data sheets specify the location in the communications
    /// protocol after which the strong pullup should be applied.
    /// The SPU bit must be set immediately prior to issuing the
    /// command that puts the 1-Wire device into the state where
    /// it needs the extra power. The strong pullup uses the same
    /// internal pullup transistor as the active pullup feature. See
    /// the R APU parameter in the Electrical Characteristics to
    /// determine whether the voltage drop is low enough to
    /// maintain the required 1-Wire voltage at a given load cur-
    /// rent and 1-Wire supply voltage.
    pub strong_pullup: bool,
    /// The 1WS bit determines the timing of any 1-Wire
    /// communication generated by the DS2484. All 1-Wire slave
    /// devices support standard speed (1WS = 0). Many
    /// 1-Wire devices can also communicate at a higher data
    /// rate, called overdrive speed. To change from standard
    /// to overdrive speed, a 1-Wire device needs to receive
    /// an Overdrive-Skip ROM or Overdrive-Match ROM command,
    /// as explained in the Analog Devices 1-Wire IC data
    /// sheets. The change in speed occurs immediately after
    /// the 1-Wire device has received the speed-changing command
    /// code. The DS2484 must take part in this speed
    /// change to stay synchronized. This is accomplished by
    /// writing to the Device Configuration register with the 1WS
    /// bit as 1 immediately after the 1-Wire Byte command that
    /// changes the speed of a 1-Wire device. Writing to the
    /// Device Configuration register with the 1WS bit as 0, fol-
    /// lowed by a 1-Wire Reset command, changes the DS2484
    /// and any 1-Wire devices on the active 1-Wire line back to
    /// standard speed.
    pub onewire_speed: bool,
    #[bits(4)]
    reserved: u8,
}

const fn cfg_to_u8(cfg: u8) -> u8 {
    (cfg & 0x0f) | ((!cfg & 0x0f) << 4)
}

impl Interact for DeviceConfiguration {
    const WRITE_ADDR: u8 = 0xd2;
    const READ_PTR: u8 = 0xc3;

    fn read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut [self.0])?;
        Ok(())
    }

    fn write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait()?;
        dev.i2c
            .write_read(dev.addr, &[Self::WRITE_ADDR, self.0], &mut [self.0])?;
        dev.reset = false; // Reset the device state after writing configuration
        Ok(())
    }
}

/// 1-Wire port parameters.
///
/// Affects the 1-Wire timing or pull-up resistors.
///
/// # Note: Upon a power-on reset or after a
/// Device Reset command, the parameter default values apply.
#[derive(Debug, PartialEq)]
pub struct OneWirePortConfiguration {
    t_rstl: u8,    // 0b0000
    t_rstl_od: u8, // 0b0001
    t_msp: u8,     // 0b0010
    t_msp_od: u8,  // 0b0011
    t_w0l: u8,     // 0b0100
    t_w0l_od: u8,  // 0b0101
    t_rec0: u8,    // 0b0110
    r_wpu: u8,     // 0b1000
}

impl Interact for OneWirePortConfiguration {
    const WRITE_ADDR: u8 = 0xc3;

    const READ_PTR: u8 = 0xb4;

    fn read<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        let mut buf = [0; 8];
        dev.i2c
            .write_read(dev.addr, &[READ_PTR_CMD, Self::READ_PTR], &mut buf)?;
        *self = Self::from_bytes(buf);
        Ok(())
    }

    fn write<I: I2c<SevenBitAddress>, D: DelayNs>(
        &mut self,
        dev: &mut Ds2484<I, D>,
    ) -> Result<(), Ds2484Error<I::Error>> {
        dev.onewire_wait()?;
        dev.i2c.write(dev.addr, &self.to_bytes())?;
        self.read(dev)
    }
}

impl OneWirePortConfiguration {
    /// Reset low time in ns (tRSTL).
    pub fn reset_time(&self) -> u32 {
        (self.t_rstl & 0x0f) as u32 * 20000 + 440000
    }

    /// Reset low time in OverDrive mode, in ns (tRSTL).
    pub fn reset_time_overdrive(&self) -> u32 {
        (self.t_rstl_od & 0x0f) as u32 * 2000 + 44000
    }

    /// Presence-detect sampling time in ns (tMSP).
    pub fn presence_detect_time(&self) -> u32 {
        (match (self.t_msp & 0x0f) as u32 {
            0..=1 => 58,
            13..=u32::MAX => 76,
            val => 60 + (val - 2) * 2,
        }) * 100
    }

    /// Presence-detect sampling time in OverDrive mode, in ns (tMSP).
    pub fn presence_detect_time_overdrive(&self) -> u32 {
        (match (self.t_msp_od & 0x0f) as u32 {
            0..=1 => 55,
            13..=u32::MAX => 110,
            val => 60 + (val - 2) * 5,
        }) * 100
    }

    /// Write zero low time in ns (tW0L).
    pub fn write_zero_low_time(&self) -> u32 {
        match (self.t_w0l & 0x0f) as u32 {
            10..=u32::MAX => 70000,
            val => 52000 + val * 2000,
        }
    }

    /// Write zero low time in OverDrive mode, in ns (tW0L).
    pub fn write_zero_low_time_overdrive(&self) -> u32 {
        match (self.t_w0l_od & 0x0f) as u32 {
            10..=u32::MAX => 10000,
            val => 5000 + val * 500,
        }
    }

    /// Write zero recovery time in ns (tREC0).
    pub fn write_zero_recovery_time(&self) -> u32 {
        match (self.t_rec0 & 0x0f) as u32 {
            0..=4 => 2750,
            15..=u32::MAX => 25250,
            val => 2750 + (val - 5) * 2500,
        }
    }

    /// Weak pull-up resistor value in Ohms (R_WPU).
    pub fn weak_pullup_resistor(&self) -> u16 {
        if self.r_wpu & 0x0f < 0b0110 {
            500 // 500 Ohm
        } else {
            1000 // 1000 Ohm
        }
    }

    pub(crate) fn to_bytes(&self) -> [u8; 9] {
        [
            0xc3,
            self.t_rstl,
            self.t_rstl_od,
            self.t_msp,
            self.t_msp_od,
            self.t_w0l,
            self.t_w0l_od,
            self.t_rec0,
            self.r_wpu,
        ]
    }

    pub(crate) fn from_bytes(bytes: [u8; 8]) -> Self {
        OneWirePortConfiguration {
            t_rstl: (bytes[0] & 0x0f),
            t_rstl_od: (bytes[1] & 0x0f) | 0b0001_0000,
            t_msp: (bytes[2] & 0x0f) | 0b0010_0000,
            t_msp_od: (bytes[3] & 0x0f) | 0b0011_0000,
            t_w0l: (bytes[4] & 0x0f) | 0b0100_0000,
            t_w0l_od: (bytes[5] & 0x0f) | 0b0101_0000,
            t_rec0: (bytes[6] & 0x0f) | 0b0110_0000,
            r_wpu: (bytes[7] & 0x0f) | 0b1000_0000,
        }
    }
}

impl Default for OneWirePortConfiguration {
    fn default() -> Self {
        OneWirePortConfiguration {
            t_rstl: 0b0000_0110,
            t_rstl_od: 0b0001_0110,
            t_msp: 0b0010_0110,
            t_msp_od: 0b0011_0110,
            t_w0l: 0b0100_0110,
            t_w0l_od: 0b0101_0110,
            t_rec0: 0b0110_0110,
            r_wpu: 0b1000_0110,
        }
    }
}

/// Builder for configuring the 1-Wire port parameters.
#[derive(Debug, Default)]
pub struct OneWireConfigurationBuilder {
    cfg: OneWirePortConfiguration,
}

#[allow(clippy::from_over_into)]
impl Into<OneWireConfigurationBuilder> for OneWirePortConfiguration {
    fn into(self) -> OneWireConfigurationBuilder {
        OneWireConfigurationBuilder { cfg: self }
    }
}

impl OneWireConfigurationBuilder {
    /// Set the reset low time in nanoseconds (tRSTL).
    pub fn reset_pulse(mut self, normal: u32, overdrive: u32) -> Self {
        const NORMAL: [u32; 16] = [
            440000, 460000, 480000, 500000, 520000, 540000, 560000, 580000, 600000, 620000, 640000,
            660000, 680000, 700000, 720000, 740000,
        ];
        let index = NORMAL.iter().position(|&v| v >= normal).unwrap_or(0);
        self.cfg.t_rstl = (self.cfg.t_rstl & 0xf0) | (index as u8);
        const OVERDRIVE: [u32; 16] = [
            44000, 46000, 48000, 50000, 52000, 54000, 56000, 58000, 60000, 62000, 64000, 66000,
            68000, 70000, 72000, 74000,
        ];
        let index = OVERDRIVE.iter().position(|&v| v >= overdrive).unwrap_or(0);
        self.cfg.t_rstl_od = (self.cfg.t_rstl_od & 0xf0) | (index as u8);
        self
    }

    /// Set the presence-detect sampling time in nanoseconds (tMSP).
    pub fn presence_detect_time(mut self, normal: u32, overdrive: u32) -> Self {
        const NORMAL: [u32; 16] = [
            58000, 58000, 60000, 62000, 64000, 66000, 68000, 70000, 72000, 74000, 76000, 76000,
            76000, 76000, 76000, 76000,
        ];
        let index = NORMAL.iter().position(|&v| v >= normal).unwrap_or(0);
        self.cfg.t_msp = (self.cfg.t_msp & 0xf0) | (index as u8);
        const OVERDRIVE: [u32; 16] = [
            5500, 5500, 6000, 6500, 7000, 7500, 8000, 8500, 9000, 9500, 10000, 10500, 11000, 11000,
            11000, 11000,
        ];
        let index = OVERDRIVE.iter().position(|&v| v >= overdrive).unwrap_or(0);
        self.cfg.t_msp_od = (self.cfg.t_msp_od & 0xf0) | (index as u8);
        self
    }

    /// Set the write zero low time in nanoseconds (tW0L).
    pub fn write_zero_low_time(mut self, normal: u32, overdrive: u32) -> Self {
        const NORMAL: [u32; 16] = [
            52000, 54000, 56000, 58000, 60000, 62000, 64000, 66000, 68000, 70000, 70000, 70000,
            70000, 70000, 70000, 70000,
        ];
        let index = NORMAL.iter().position(|&v| v >= normal).unwrap_or(0);
        self.cfg.t_w0l = (self.cfg.t_w0l & 0xf0) | (index as u8);
        const OVERDRIVE: [u32; 16] = [
            5000, 5500, 6000, 6500, 7000, 7500, 8000, 8500, 9000, 9500, 10000, 10000, 10000, 10000,
            10000, 10000,
        ];
        let index = OVERDRIVE.iter().position(|&v| v >= overdrive).unwrap_or(0);
        self.cfg.t_w0l_od = (self.cfg.t_w0l_od & 0xf0) | (index as u8);
        self
    }

    /// Set the write zero recovery time in nanoseconds (tREC0).
    pub fn write_zero_recovery_time(mut self, value: u16) -> Self {
        const VALUES: [u16; 16] = [
            275, 275, 275, 275, 275, 275, 525, 775, 1025, 1275, 1525, 1775, 2025, 2275, 2525, 2525,
        ];
        let index = VALUES.iter().position(|&v| v >= value).unwrap_or(0);
        self.cfg.t_rec0 = (self.cfg.t_rec0 & 0xf0) | (index as u8);
        self
    }

    /// Set the weak pull-up resistor value in Ohms (R_WPU).
    pub fn weak_pullup_resistor(mut self, value: u16) -> Self {
        if value < 1000 {
            self.cfg.r_wpu &= 0xf0; // 500 Ohm
        } else {
            self.cfg.r_wpu &= 0xff; // 1000 Ohm
        }
        self
    }

    /// Build the configuration.
    pub fn build(self) -> OneWirePortConfiguration {
        self.cfg
    }
}
