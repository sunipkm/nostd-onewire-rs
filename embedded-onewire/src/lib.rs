#![no_std]
#![deny(missing_docs)]
//! # embedded-onewire
//! A no-std implementation of the 1-Wire protocol.
//!
//! This crate provides a trait-based interface for 1-Wire communication, allowing you to implement the protocol on various platforms.
//! [OneWire] trait defines the basic operations required for 1-Wire communication, such as resetting the bus, writing and reading bytes, and writing and reading bits.
//! It also includes an asynchronous version of the trait, [OneWireAsync], for use in async environments.
//!
//! The crate also provides a search algorithm for discovering devices on the 1-Wire bus, implemented in the [OneWireSearch] and [OneWireSearchAsync] structs.

mod error;
mod search;
mod search_async;
mod traits;
mod traits_async;
mod utils;
pub use error::OneWireError;
pub use search::{OneWireSearch, OneWireSearchKind};
pub use search_async::OneWireSearchAsync;
pub use traits::{OneWire, OneWireStatus};
pub use traits_async::OneWireAsync;
pub use utils::OneWireCrc;

/// Error type for 1-Wire operations.
pub type OneWireResult<T, E> = Result<T, OneWireError<E>>;

/// Command to match a specific ROM address in 1-Wire communication (non-overdrive mode)
pub const ONEWIRE_MATCH_ROM_CMD: u8 = 0x55;

/// Command to skip ROM address in 1-Wire communication (non-overdrive mode)
pub const ONEWIRE_SKIP_ROM_CMD: u8 = 0xcc;

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
pub const ONEWIRE_MATCH_ROM_CMD_OD: u8 = 0x69;

/// On a single-drop bus this command can save time by
/// allowing the bus master to access the control functions
/// without providing the 64-bit ROM code. Unlike the normal
/// Skip ROM command, the Overdrive-Skip ROM sets the
/// DS28EA00 in the overdrive mode (OD = 1). All communication
/// following this command has to occur at overdrive
/// speed until a reset pulse of minimum 480μs duration
/// resets all devices on the bus to standard speed (OD = 0).
pub const ONEWIRE_SKIP_ROM_CMD_OD: u8 = 0x3c;

/// Command to search for devices on the 1-Wire bus
pub const ONEWIRE_SEARCH_CMD: u8 = 0xf0;

/// Command to search for devices in alarm state on the 1-Wire bus
pub const ONEWIRE_CONDITIONAL_SEARCH_CMD: u8 = 0xec;