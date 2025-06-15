use crate::{OneWireError, OneWireResult};

/// Trait describing the status of a 1-Wire bus.
/// This trait is used to encapsulate the status of the bus after a reset operation.
pub trait OneWireStatus {
    /// Returns true if a device is present on the bus, false otherwise.
    fn presence(&self) -> bool;
    /// Returns true if a short circuit is detected on the bus, false otherwise.
    fn shortcircuit(&self) -> bool;
    /// Returns the direction taken in the [OneWire::read_triplet] operation.
    #[cfg(feature = "triplet-read")]
    #[cfg_attr(docsrs, doc(cfg(feature = "triplet-read")))]
    fn direction(&self) -> Option<bool> {
        None
    }
    /// Returns the logic state of the active 1-Wire line without initiating any 1-Wire communication.
    fn logic_level(&self) -> Option<bool> {
        None
    }
}

/// Trait for 1-Wire communication.
/// This trait defines the basic operations required for 1-Wire communication, such as resetting the bus,
/// writing and reading bytes, and writing and reading bits.
pub trait OneWire {
    /// The status type returned by the reset operation.
    /// This type must implement the [OneWireStatus] trait.
    type Status: OneWireStatus;
    /// The error type returned by the operations of this trait.
    /// This type is used to indicate errors in the underlying hardware or communication.
    type BusError;

    /// Resets the 1-Wire bus and returns the status of the bus.
    ///
    /// # Returns
    /// A result containing the status of the bus after the reset operation.
    ///
    /// # Errors
    /// This method returns an error if the reset operation fails.
    fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError>;

    /// Addresses devices on the 1-Wire bus.
    /// The first [`OneWire::read_byte`], [`OneWire::read_bit`], [`OneWire::write_byte`], [`OneWire::write_bit`] operation should be preceded by this method to address devices on the bus.
    /// Note: A [`OneWire::read_byte`] or [`OneWire::read_bit`] call will return garbage data if this method is called without specifying a ROM address on a bus with multiple devices.
    /// # Arguments
    /// * `rom` - The ROM address of the device to address. Pass [`None`] to skip ROM addressing and address all devices on the bus.
    ///
    /// # Returns
    /// A result indicating the success or failure of the operation.
    /// If the device is successfully addressed, the method returns `Ok(())`.
    fn address(&mut self, rom: Option<u64>) -> OneWireResult<(), Self::BusError> {
        let od = self.get_overdrive_mode()?;
        let cmd = if rom.is_some() {
            if od {
                crate::consts::ONEWIRE_MATCH_ROM_CMD_OD
            } else {
                crate::consts::ONEWIRE_MATCH_ROM_CMD
            }
        } else if od {
            crate::consts::ONEWIRE_SKIP_ROM_CMD_OD
        } else {
            crate::consts::ONEWIRE_SKIP_ROM_CMD
        };
        self.reset()?; // Reset the bus before addressing
        self.write_byte(cmd)?; // Send the match ROM command
        if let Some(rom) = rom {
            for &b in rom.to_le_bytes().iter() {
                self.write_byte(b)?; // Write each byte of the ROM address
            }
        }
        Ok(())
    }

    /// Writes a byte to the device addressed using [`OneWire::address`] on the 1-Wire bus.
    /// Multiple bytes can be written in succession after addressing the device.
    ///
    /// # Arguments
    /// * `byte` - The byte to write to the bus.
    ///
    /// # Errors
    /// This method returns an error if the write operation fails.
    fn write_byte(&mut self, byte: u8) -> OneWireResult<(), Self::BusError>;

    /// Reads a byte from the device addressed using [`OneWire::address`] on the 1-Wire bus.
    /// Multiple bytes can be read in succession after addressing the device.
    ///
    /// # Note
    /// If there are more than one devices on the bus and [`OneWire::address`] was not called
    /// with a specific ROM address, the read operation will return garbage data.
    ///
    /// # Returns
    /// Byte read from the bus.
    ///
    /// # Errors
    /// This method returns an error if the read operation fails.
    fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError>;

    /// Write a single bit to the device addressed using [`OneWire::address`] on the 1-Wire bus.
    /// Multiple bits can be written in succession after addressing the device.
    /// # Arguments
    ///
    /// * `bit` - The byte to write.
    ///
    /// # Errors
    /// This method returns an error if the read operation fails.
    fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError>;

    /// Reads a single bit from the device addressed using [`OneWire::address`] on the 1-Wire bus.
    /// Multiple bits can be read in succession after addressing the device.
    ///
    /// # Note
    /// If there are more than one devices on the bus and [`OneWire::address`] was not called
    /// with a specific ROM address, the read operation will return garbage data.
    ///
    /// # Returns
    /// The bit read from the bus.
    /// # Errors
    /// This method returns an error if the read operation fails.
    fn read_bit(&mut self) -> OneWireResult<bool, Self::BusError>;

    /// # Note: Not intended for public API use.
    /// ## This method is internally used to performa [1-wire search ROM sequence](https://www.analog.com/en/resources/app-notes/1wire-search-algorithm.html). A full sequence requires this command to be executed 64 times to identify and address one device.
    /// ## This method is internally used by the [search algorithm](https://www.analog.com/en/resources/app-notes/1wire-search-algorithm.html).
    ///
    /// Generates three time slots: two read time slots and one write time slot at the 1-Wire line. The
    /// type of write time slot depends on the result of the read time slots and the direction byte. The
    /// direction byte determines the type of write time slot if both read time slots are 0 (a typical
    /// case). In this case, a write-one time slot is generated if V = 1 and a write-zero time
    /// slot if V = 0.
    /// If the read time slots are 0 and 1, they are followed by a write-zero time slot.
    /// If the read time slots are 1 and 0, they are followed by a write-one time slot.
    /// If the read time slots are both 1 (error case), the subsequent write time slot is a write-one.
    ///
    ///
    /// # Arguments
    /// * `direction` - A boolean indicating the direction of the search. If true, the search is in the forward direction; if false, it is in the backward direction.
    ///
    /// # Returns
    /// A result containing a tuple of two booleans:
    /// * The first boolean indicates the id bit read from the bus.
    /// * The second boolean indicates the complement bit read from the bus.
    ///
    /// # Errors
    /// This method returns an error if the triplet read operation is not implemented or if any other error occurs.
    #[cfg(feature = "triplet-read")]
    #[cfg_attr(docsrs, doc(cfg(feature = "triplet-read")))]
    fn read_triplet(&mut self) -> OneWireResult<(bool, bool, bool), Self::BusError>;

    /// Check if the 1-Wire bus is in overdrive mode.
    /// # Returns
    /// A result containing a boolean indicating whether the bus is in overdrive mode.
    fn get_overdrive_mode(&mut self) -> OneWireResult<bool, Self::BusError>;

    /// Set the 1-Wire bus to overdrive mode.
    /// # Arguments
    /// * `enable` - A boolean indicating whether to enable or disable overdrive mode.
    /// # Returns
    /// A result indicating the success or failure of the operation.
    fn set_overdrive_mode(&mut self, _enable: bool) -> OneWireResult<(), Self::BusError> {
        Err(OneWireError::Unimplemented)
    }
}
