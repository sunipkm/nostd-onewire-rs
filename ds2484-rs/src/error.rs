#[derive(Debug)]
/// DS2484 Hardware Errors
pub enum Ds2484Error<E> {
    /// I2C bus errors.
    I2c(E),
    /// Busy wait retries exceeded.
    RetriesExceeded,
}

impl<E> From<E> for Ds2484Error<E> {
    fn from(value: E) -> Self {
        Self::I2c(value)
    }
}
