#![no_std]
#![deny(missing_docs)]

/*! # DS2484
 *
 *
 *
 */

pub use embedded_onewire::{OneWire, OneWireAsync, OneWireError, OneWireResult};
mod error;
mod onewire;
mod onewire_async;
mod registers;
mod registers_async;
mod traits;
mod traits_async;

pub use error::Ds2484Error;
pub use registers::{
    Ds2484, Ds2484Builder,
    DeviceConfiguration, DeviceStatus, OneWireConfigurationBuilder, OneWirePortConfiguration,
};
pub use registers_async::{Ds2484Async, Ds2484AsyncBuilder};
pub use traits::Interact;
pub use traits_async::InteractAsync;

/// Results of DS2484-specific function calls.
pub type Ds2484Result<T, E> = Result<T, Ds2484Error<E>>;

