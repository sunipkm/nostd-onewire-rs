//! Command constants for 1-Wire communication.

/// Command to match a specific ROM address in 1-Wire communication (non-overdrive mode)
pub(crate) const ONEWIRE_MATCH_ROM_CMD: u8 = 0x55;

/// Command to skip ROM address in 1-Wire communication (non-overdrive mode)
pub(crate) const ONEWIRE_SKIP_ROM_CMD: u8 = 0xcc;

/// The Overdrive-Match ROM command followed by a 64-bit
/// ROM sequence transmitted at overdrive speed allows the
/// bus master to address a specific DS28EA00 on a multidrop
/// bus and to simultaneously set it in over-drive mode.
/// Only the DS28EA00 that exactly matches the 64-bit ROM
/// sequence responds to the subsequent control function
/// command. Slaves already in overdrive mode from a previous
/// Overdrive-Skip ROM or successful Overdrive-Match
/// ROM command remain in overdrive mode. All over-drivecapable
/// slaves return to standard speed at the next reset
/// pulse of minimum 480μs duration. The Overdrive-Match
/// ROM command can be used with a single device or mul-
/// tiple devices on the bus.
pub(crate) const ONEWIRE_MATCH_ROM_CMD_OD: u8 = 0x69;

/// On a single-drop bus this command can save time by
/// allowing the bus master to access the control functions
/// without providing the 64-bit ROM code. Unlike the normal
/// Skip ROM command, the Overdrive-Skip ROM sets the
/// DS28EA00 in the overdrive mode (OD = 1). All communication
/// following this command has to occur at overdrive
/// speed until a reset pulse of minimum 480μs duration
/// resets all devices on the bus to standard speed (OD = 0).
pub(crate) const ONEWIRE_SKIP_ROM_CMD_OD: u8 = 0x3c;

/// Command to search for devices on the 1-Wire bus
pub(crate) const ONEWIRE_SEARCH_CMD: u8 = 0xf0;

/// Command to search for devices in alarm state on the 1-Wire bus
pub(crate) const ONEWIRE_CONDITIONAL_SEARCH_CMD: u8 = 0xec;
