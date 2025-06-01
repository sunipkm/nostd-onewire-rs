use crate::{error::OneWireError, utils::OneWireCrc, OneWire, OneWireStatus, ONEWIRE_CONDITIONAL_SEARCH_CMD, ONEWIRE_SEARCH_CMD};

/// A structure for searching devices on a 1-Wire bus.
/// This structure implements the search algorithm for discovering devices on the 1-Wire bus.
/// It maintains the state of the search.
pub struct OneWireSearch<'a, T> {
    onewire: &'a mut T,
    cmd: u8,
    last_device: bool,
    last_discrepancy: u8,
    last_family_discrepancy: u8,
    family: u8,
    rom: [u8; 8],
}

#[repr(u8)]
/// Type of search performed using [`OneWireSearch`] or [`OneWireSearchAsync`](crate::OneWireSearchAsync).
pub enum OneWireSearchKind {
    /// Normal search
    Normal = ONEWIRE_SEARCH_CMD,
    /// Search only for devicess with alarm
    Alarmed = ONEWIRE_CONDITIONAL_SEARCH_CMD,
}

impl<'a, T> OneWireSearch<'a, T> {
    /// Creates a new [`OneWireSearch`] instance.
    ///
    /// # Arguments
    /// * `onewire` - A mutable reference to a type that implements the `OneWire` trait.
    /// * `cmd` - The command to use for the search operation (e.g., `0xf0` for normal search, `0xec` for search in alarm state).
    pub fn new(onewire: &'a mut T, cmd: OneWireSearchKind) -> Self {
        Self {
            onewire,
            cmd: cmd as _,
            last_device: false,
            last_discrepancy: 0,
            last_family_discrepancy: 0,
            family: 0, // Initialize family code to 0
            rom: [0; 8],
        }
    }

    /// Creates a new [`OneWireSearch`] instance with a specific family code.
    /// # Arguments
    /// * `onewire` - A mutable reference to a type that implements the `OneWire` trait.
    /// * `cmd` - The command to use for the search operation (e.g., `0xf0` for normal search, `0xec` for search in alarm state).
    /// * `family` - The family code of the devices to search for.
    pub fn with_family(onewire: &'a mut T, cmd: OneWireSearchKind, family: u8) -> Self {
        let rom = [family, 0, 0, 0, 0, 0, 0, 0]; // Initialize the ROM with the family code
        Self {
            onewire,
            cmd: cmd as _,
            last_device: false,
            last_discrepancy: 0,
            last_family_discrepancy: 0,
            family,
            rom,
        }
    }

    /// Resets the search state.
    fn reset(&mut self) {
        self.last_device = false; // Reset the last device flag
        self.last_discrepancy = 0; // Reset the last discrepancy
        self.last_family_discrepancy = 0; // Reset the last family discrepancy
        self.rom = [self.family, 0, 0, 0, 0, 0, 0, 0]; // Reset the ROM array
    }
}

impl<T: OneWire> OneWireSearch<'_, T> {
    /// Searches for devices on the 1-Wire bus.
    /// This method implements the [1-Wire search algorithm](https://www.analog.com/en/resources/app-notes/1wire-search-algorithm.html) to discover devices connected to the bus.
    /// The [next](OneWireSearch::next) method can be called repeatedly to find all devices on the bus.
    /// At the end of the search, calling this method will return `None` to indicate that no more devices are present.
    /// At that point, the search state becomes unusable and should be dropped.
    /// The search state is reset if the [verify](OneWireSearch::verify) method is called.
    ///
    /// # Returns
    /// A result containing the ROM code of the found device as a `u64` value.
    ///
    /// | Bit | Description |
    /// |-----|-------------|
    /// | 0-7 | Family code (e.g., 0x28 for DS18B20) |
    /// | 8-15 | Serial number (first byte) |
    /// | 16-23 | Serial number (second byte) |
    /// | 24-31 | Serial number (third byte) |
    /// | 32-39 | Serial number (fourth byte) |
    /// | 40-47 | Serial number (fifth byte) |
    /// | 48-55 | Serial number (sixth byte) |
    /// | 56-63 | CRC-8 (`0b1_0001_1001` poly) |
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<u64>, OneWireError<T::BusError>> {
        if self.onewire.get_overdrive_mode()? {
            return Err(OneWireError::BusInvalidSpeed);
        }
        if self.last_device {
            return Ok(None); // If the last device was found, return None
        }
        let status = self.onewire.reset()?;
        if !status.presence() {
            return Err(OneWireError::NoDevicePresent);
        }
        if status.shortcircuit() {
            return Err(OneWireError::ShortCircuit);
        }
        let mut id_bit_num: u8 = 1;
        let mut last_zero: u8 = 0;
        let mut idx: usize = 0; // Index in the ROM array
        let mut rom_mask: u8 = 1; // Mask for the current bit in the ROM byte
        self.onewire.write_byte(self.cmd)?; // Search ROM command
        let res = loop {
            // Determine the direction of the search
            let dir = if id_bit_num < self.last_discrepancy {
                self.rom[idx] & rom_mask > 0
            } else {
                id_bit_num == self.last_discrepancy
            };
            if !dir {
                last_zero = id_bit_num;
                if last_zero < 9 {
                    self.last_family_discrepancy = last_zero;
                }
            }
            // Read the id_bit and the complement_bit using triplet if available
            // If triplet is not implemented, fallback to reading bits, and let
            // the write flag indicate if we need to write the direction bit later.
            let (id_bit, complement_bit, write) = match self.onewire.read_triplet(dir) {
                Ok(triplet) => (triplet.0, triplet.1, false),
                Err(e) => {
                    match e {
                        OneWireError::Unimplemented => {
                            // Fallback to reading bits since triplet is not implemented
                            let id_bit = self.onewire.read_bit()?;
                            let complement_bit = self.onewire.read_bit()?;
                            (id_bit, complement_bit, true)
                        }
                        _ => return Err(e),
                    }
                }
            };
            if id_bit && complement_bit {
                // Both bits are 1, which is an error condition, reset the search
                break false;
            }
            let set = if id_bit != complement_bit {
                // The bits are different, use the id_bit
                id_bit
            } else {
                // Both bits are 0, use the direction from the ROM
                dir
            };
            if set {
                self.rom[idx] |= rom_mask; // Set the bit in the ROM
            } else {
                self.rom[idx] &= !rom_mask; // Clear the bit in the ROM
            }

            if write {
                self.onewire.write_bit(set)?; // Write the direction bit if triplet is not implemented
            }

            id_bit_num += 1;
            rom_mask <<= 1; // Move to the next bit in the ROM byte

            if rom_mask == 0 {
                idx += 1; // Move to the next byte in the ROM
                rom_mask = 1; // Reset the mask for the next byte
            }
            if id_bit_num > 64 {
                self.last_discrepancy = last_zero;
                self.last_device = self.last_discrepancy == 0;
                break true;
            }
        };

        if !res || self.rom[0] == 0 {
            // If no device was found or the first byte is zero, reset the search state
            return Ok(None);
        }
        if !OneWireCrc::validate(&self.rom) {
            // If the CRC is not valid, reset the search state
            return Err(OneWireError::InvalidCrc);
        }
        if self.family != 0 && self.rom[0] != self.family {
            // If a specific family code was set and it does not match the found device
            return Ok(None);
        }
        Ok(Some(u64::from_le_bytes(self.rom)))
    }

    /// Verifies if the device with the given ROM code is present on the 1-Wire bus.
    ///
    /// This function should be called with a search state that has been exhausted (i.e., after calling [next](OneWireSearch::next) until it returns `None`).
    /// This functions resets the search state, and calling [next](OneWireSearch::next) after this call will start a new search.
    pub fn verify(&mut self, rom: u64) -> Result<bool, OneWireError<T::BusError>> {
        self.reset(); // Reset the search state
        self.rom = rom.to_le_bytes(); // Set the ROM to verify
        self.last_discrepancy = 64; // Set the last discrepancy to 64
        let res = self.next()?;
        self.reset(); // Reset the search state after verification
        Ok(res == Some(rom))
    }
}
