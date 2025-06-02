# One Wire Bus `no-std` Traits
This repository defines `no-std`, synchronous and asynchronous traits to configure and read data from 1-Wire devices, provided in the `embedded-onewire` repository.

## One Wire Bus Implementations
A reference implementation is provided for the [Analog Devices DS2484](https://www.analog.com/en/products/ds2484.html) I2C-to-1-Wire bridge device, in the [`ds2484`] crate. It uses the `I2c` and `DelayNs` traits from [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/) and [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async) crates for synchronous and asynchronous APIs, respectively.

## Example One-Wire Device
A reference driver is provided for the [Analog Devices DS28EA00](https://www.analog.com/en/products/ds28ea00.html) temperature sensor in the `ds28ea00` crate. Currently, only the synchronous API is implemented.

## Example Linux Program
An example Linux program that enumerates DS28EA00 devices connected to a DS2484 bridge devices and reads out temperatures is provided in `ds2484-linux` crate.

## Cross compilation

If you're not working directly on a Raspberry Pi, you'll have to cross-compile your code for the appropriate ARM architecture. Check out [this guide](https://github.com/japaric/rust-cross) for more information, or try the [cross](https://github.com/japaric/cross) project for "zero setup" cross compilation.

### Cargo

For manual cross-compilation without the use of `cross`, you will need to install the appropriate target. Most Raspberry Pi models either need the `armv7-unknown-linux-gnueabihf` target for 32-bit Linux distributions, or `aarch64-unknown-linux-gnu` for 64-bit. For some models, like the Raspberry Pi Zero, a different target triple is required.

Install the relevant target using `rustup`.

```bash
rustup target install armv7-unknown-linux-gnueabihf
```

In the root directory of your project, create a `.cargo` subdirectory, and save the following snippet to `.cargo/config.toml`.

```toml
[build]
target = "armv7-unknown-linux-gnueabihf"
```

### Visual Studio Code

The rust-analyzer extension for Visual Studio Code needs to be made aware of the target platform by setting the `rust-analyzer.cargo.target` configuration option. In the root directory of your project, create a `.vscode` subdirectory, and then save the following snippet to `.vscode/settings.json`.

```json
{
    "rust-analyzer.cargo.target": "armv7-unknown-linux-gnueabihf"
}
```