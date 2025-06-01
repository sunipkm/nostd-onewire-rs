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
mod traits;
mod utils;
mod traits_async;
mod search_async;
pub use error::OneWireError;
pub use search::{OneWireSearch, OneWireSearchKind};
pub use traits::{OneWire, OneWireStatus};
pub use traits_async::OneWireAsync;
pub use search_async::OneWireSearchAsync;
pub use utils::OneWireCrc;

/// Error type for 1-Wire operations.
pub type OneWireResult<T, E> = Result<T, OneWireError<E>>;