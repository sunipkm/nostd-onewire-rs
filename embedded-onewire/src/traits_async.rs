#![allow(async_fn_in_trait)]
use crate::{OneWireError, OneWireResult, OneWireStatus};

/// Trait for 1-Wire communication.
/// This trait defines the basic operations required for 1-Wire communication, such as resetting the bus,
/// writing and reading bytes, and writing and reading bits.
pub trait OneWireAsync {
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
    async fn reset(&mut self) -> OneWireResult<Self::Status, Self::BusError>;

    /// Writes a byte to the 1-Wire bus.
    /// # Arguments
    /// * `byte` - The byte to write to the bus.
    ///
    /// # Errors
    /// This method returns an error if the write operation fails.
    async fn write_byte(&mut self, byte: u8) -> OneWireResult<(), Self::BusError>;

    /// Reads a byte from the 1-Wire bus.
    /// # Returns
    /// Byte read from the bus.
    ///
    /// # Errors
    /// This method returns an error if the read operation fails.
    async fn read_byte(&mut self) -> OneWireResult<u8, Self::BusError>;

    /// Reads a byte from the 1-Wire bus, with an option to write a byte before reading.
    /// # Arguments
    ///
    /// * `bit` - The byte to write.
    ///
    /// # Errors
    /// This method returns an error if the read operation fails.
    async fn write_bit(&mut self, bit: bool) -> OneWireResult<(), Self::BusError>;

    /// Reads a single bit from the 1-Wire bus.
    /// # Returns
    /// The bit read from the bus.
    /// # Errors
    /// This method returns an error if the read operation fails.
    async fn read_bit(&mut self) -> OneWireResult<bool, Self::BusError>;

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
    async fn read_triplet(&mut self) -> OneWireResult<(bool, bool, bool), Self::BusError> {
        Err(OneWireError::Unimplemented)
    }

    /// Check if the 1-Wire bus is in overdrive mode.
    /// # Returns
    /// A result containing a boolean indicating whether the bus is in overdrive mode.
    async fn get_overdrive_mode(&mut self) -> OneWireResult<bool, Self::BusError>;

    /// Set the 1-Wire bus to overdrive mode.
    /// # Arguments
    /// * `enable` - A boolean indicating whether to enable or disable overdrive mode.
    /// # Returns
    /// A result indicating the success or failure of the operation.
    async fn set_overdrive_mode(&mut self, _enable: bool) -> OneWireResult<(), Self::BusError> {
        Err(OneWireError::Unimplemented)
    }

    /// Addresses devices on the 1-Wire bus.
    /// The first [`OneWire::read_byte`], [`OneWire::read_bit`], [`OneWire::write_byte`], [`OneWire::write_bit`] operation should be preceded by this method to address devices on the bus.
    /// Note: A [`OneWire::read_byte`] or [`OneWire::read_bit`] call will return garbage data if this method is called without specifying a ROM address on a bus with multiple devices.
    /// # Arguments
    /// * `rom` - The ROM address of the device to address. Pass [`None`] to skip ROM addressing and address all devices on the bus.
    ///
    /// # Returns
    /// A result indicating the success or failure of the operation.
    /// If the device is successfully addressed, the method returns `Ok(())`.
    async fn address(&mut self, rom: Option<u64>) -> OneWireResult<(), Self::BusError> {
        let od = self.get_overdrive_mode().await?;
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
        self.reset().await?; // Reset the bus before addressing
        self.write_byte(cmd).await?; // Send the match ROM command
        if let Some(rom) = rom {
            for &b in rom.to_le_bytes().iter() {
                self.write_byte(b).await?; // Write each byte of the ROM address
            }
        }
        Ok(())
    }
}
