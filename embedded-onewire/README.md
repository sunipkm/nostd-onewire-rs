# embedded-onewire
A no-std trait description of the 1-Wire protocol.

This crate provides a trait-based interface for 1-Wire communication, allowing you to implement the protocol on various platforms.
[`OneWire`] trait defines the basic operations required for 1-Wire communication, such as resetting the bus, writing and reading bytes, and writing and reading bits.
It also includes an asynchronous version of the trait, [`OneWireAsync`], for use in async environments.

The crate also provides a search algorithm for discovering devices on the 1-Wire bus, implemented in the [`OneWireSearch`] and [`OneWireSearchAsync`] structs.

# Features
- `crc-table`: Enables the use of a 256-byte lookup table for CRC calculation, which can improve performance at the cost of increased binary size.
- `triplet-read`: Enables the `read_triplet` trait method in [`OneWire`] and [`OneWireAsync`]. 1-Wire bus masters, e.g. the Analog Devices DS2484, implements this function to simplify the device enumeration algorithm. However, the current implementation of the search algorithm with the DS2484 device does not enumerate the devices correctly, so this feature is not recommended for use.