#![no_std]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub mod consts;
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
