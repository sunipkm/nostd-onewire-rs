#[allow(unused_imports)]
use crate::OneWireSearch;

/// One wire communication error type.
#[derive(Debug)]
pub enum OneWireError<E> {
    /// Encapsulates the error type from the underlying hardware.
    Other(E),
    /// Indicates that no device is present on the bus.
    NoDevicePresent,
    /// Indicates that the bus is busy, which may happen if a device is already communicating.
    BusInUse,
    /// Indicates that a short circuit was detected on the bus.
    ShortCircuit,
    /// Indicates that the operation is not implemented, such as reading a triplet when not supported.
    Unimplemented,
    /// Computed CRC of the ROM is invalid.
    InvalidRomCrc,
}

impl<E> From<E> for OneWireError<E> {
    fn from(other: E) -> Self {
        Self::Other(other)
    }
}