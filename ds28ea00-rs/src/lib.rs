#![no_std]
use core::iter::zip;
use embedded_hal::delay::DelayNs;
use embedded_onewire::{
    OneWire, OneWireCrc, OneWireError, OneWireResult, OneWireSearch, OneWireSearchKind,
};
use fixed::types::I12F4;

#[derive(Debug)]
pub struct Ds28ea00Group<const N: usize> {
    devices: usize,
    roms: [u64; N],
    temps: [Temperature; N],
    resolution: ReadoutResolution,
    low: i8,
    high: i8,
    toggle_pio: bool,
}

impl<const N: usize> Default for Ds28ea00Group<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Ds28ea00Group<N> {
    #[inline]
    pub const fn family() -> u8 {
        0x42
    }

    fn new() -> Self {
        Self {
            devices: 0,
            roms: [0; N],
            temps: [Temperature::ZERO; N],
            resolution: ReadoutResolution::default(),
            low: -40,
            high: 85,
            toggle_pio: false,
        }
    }

    pub fn with_resolution(mut self, resolution: ReadoutResolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_t_low(mut self, temp: i8) -> Self {
        self.low = temp;
        self
    }

    pub fn with_t_high(mut self, temp: i8) -> Self {
        self.high = temp;
        self
    }

    pub fn with_toggle_pio(mut self, toggle_pio: bool) -> Self {
        self.toggle_pio = toggle_pio;
        self
    }

    pub fn enumerate<O: OneWire>(&mut self, bus: &mut O) -> OneWireResult<usize, O::BusError> {
        let mut search = OneWireSearch::with_family(bus, OneWireSearchKind::Normal, Self::family());
        // conduct search
        while let Some(rom) = search.next()? {
            self.roms[self.devices] = rom;
            self.devices += 1;
            if self.devices == N {
                break;
            }
        }
        if self.toggle_pio {
            // turn all PIO pins on
            Self::address_any(bus)?;
            bus.write_byte(DS28EA00_TOGGLE_PIO)?;
            bus.write_byte(DS28EA00_TOGGLE_PIO_ON)?;
        }
        // address all devices
        Self::address_any(bus)?;
        // apply configuration
        bus.write_byte(DS28EA00_WRITE_SCRATCH)?;
        bus.write_byte(self.low as _)?;
        bus.write_byte(self.high as _)?;
        bus.write_byte(self.resolution as _)?;
        if self.toggle_pio {
            // turn all PIO pins off
            Self::address_any(bus)?;
            bus.write_byte(DS28EA00_TOGGLE_PIO)?;
            bus.write_byte(DS28EA00_TOGGLE_PIO_OFF)?;
        }
        Ok(self.devices)
    }

    pub(crate) fn address_any<O: OneWire>(bus: &mut O) -> OneWireResult<(), O::BusError> {
        bus.reset()?; // reset 1-Wire bus
        bus.write_byte(DS28EA00_SKIP_ROM_CMD) // match any ROM
    }

    pub fn trigger_temperature_conversion<O: OneWire, D: DelayNs>(
        &self,
        bus: &mut O,
        delay: &mut D,
    ) -> OneWireResult<(), O::BusError> {
        Self::address_any(bus)?; // address all devices
        bus.write_byte(DS28EA00_START_CONV)?; // start temperature conversion
        if self.toggle_pio {
            Self::address_any(bus)?; // address all devices
            bus.write_byte(DS28EA00_TOGGLE_PIO)?;
            bus.write_byte(DS28EA00_TOGGLE_PIO_ON)?; // turn on PIO
        }
        delay.delay_us(self.resolution.delay_us()); // wait till conversion is finished
        Ok(())
    }

    pub fn read_temperatures<O: OneWire>(
        &mut self,
        bus: &mut O,
        crc: bool,
    ) -> OneWireResult<&[Temperature], O::BusError> {
        for (rom, temp) in zip(
            self.roms[..self.devices].iter(),
            self.temps[..self.devices].iter_mut(),
        ) {
            bus.reset()?; // reset 1-Wire bus
            bus.write_byte(DS28EA00_MATCH_ROM_CMD)?; // Match ROM
            for &b in rom.to_le_bytes().iter() {
                // Send ROM address
                bus.write_byte(b)?;
            }
            if !crc {
                for b in temp.to_le_bytes().iter_mut() {
                    *b = bus.read_byte()?;
                }
            } else {
                let mut buf = [0; 9];
                for b in buf.iter_mut() {
                    *b = bus.read_byte()?;
                }
                if OneWireCrc::validate(&buf) {
                    *temp = I12F4::from_le_bytes([buf[0], buf[1]]);
                } else {
                    return Err(OneWireError::InvalidCrc);
                }
            }
            if self.toggle_pio {
                bus.reset()?;
                bus.write_byte(DS28EA00_MATCH_ROM_CMD)?; // Match ROM
                for &b in rom.to_le_bytes().iter() {
                    // Send ROM address
                    bus.write_byte(b)?;
                }
                bus.write_byte(DS28EA00_TOGGLE_PIO_OFF)?;
            }
        }
        Ok(&self.temps[..self.devices])
    }
}

const DS28EA00_MATCH_ROM_CMD: u8 = 0x55;
const DS28EA00_SKIP_ROM_CMD: u8 = 0xcc;
const DS28EA00_READ_SCRATCH: u8 = 0xbe;
const DS28EA00_WRITE_SCRATCH: u8 = 0x4e;
const DS28EA00_COPY_SCRATCH: u8 = 0x48;
const DS28EA00_START_CONV: u8 = 0x44;
const DS28EA00_READ_POWERMODE: u8 = 0xb4;
const DS28EA00_RECALL_EEPROM: u8 = 0xb8;
const DS28EA00_TOGGLE_PIO: u8 = 0xa5;
const DS28EA00_TOGGLE_PIO_ON: u8 = 0b11111101;
const DS28EA00_TOGGLE_PIO_OFF: u8 = !0b11111101;

pub type Temperature = I12F4;

pub struct Config {
    pub low: i8,
    pub high: i8,
    pub res: ReadoutResolution,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ReadoutResolution {
    Resolution9bit = 0x1f,
    Resolution10bit = 0x3f,
    Resolution11bit = 0x5f,
    Resolution12bit = 0x7f,
}

impl Default for ReadoutResolution {
    fn default() -> Self {
        Self::Resolution12bit
    }
}

impl ReadoutResolution {
    pub(crate) fn delay_us(&self) -> u32 {
        use ReadoutResolution::*;
        match self {
            Resolution9bit => 93750,
            Resolution10bit => 187500,
            Resolution11bit => 375000,
            Resolution12bit => 750000,
        }
    }
}

impl TryFrom<u8> for ReadoutResolution {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use ReadoutResolution::*;
        match value {
            0x1f => Ok(Resolution9bit),
            0x3f => Ok(Resolution10bit),
            0x5f => Ok(Resolution11bit),
            0x7f => Ok(Resolution12bit),
            _ => Err("Invalid readout resolution"),
        }
    }
}
