# ds2484

Implementation of the [`embedded-onewire`](https://crates.io/crates/embedded-onewire) traits for the [Analog Devices DS2484](https://www.analog.com/en/products/ds2484.html) I2C-to-1-Wire bridge device.

# Usage

Add the following to your `Cargo.toml`:

```toml
ds2484 = "0.0.4"
```

# Synchronous Operations

```rust,no_compile
use ds2484::Ds2484Builder;

let mut i2c = todo!();
let delay = todo!();
let mut ds2484 = Ds2484Builder::default()
                    .build(&mut i2c, delay)
                    .expect("Could not create a DS2484 instance");
```

# Asynchronous Operations
```rust,no_compile
use ds2484::Ds2484Builder;

let mut i2c = todo!();
let delay = todo!();
let mut ds2484 = Ds2484Builder::default()
                    .build_async(&mut i2c, delay)
                    .await
                    .expect("Could not create a DS2484 instance");
```
