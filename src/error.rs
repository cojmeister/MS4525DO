use defmt::Format;
use crate::ms4525do::Status;

#[derive(Debug, Format)]
pub enum Ms4525doError {
    /// Simplified; in practice, use embedded-hal-async's error type
    /// TODO: improve this
    I2cError, 
    /// Status returned by the sensor
    InvalidStatus(Status),
    ///TODO: Write a better description
    DataOutOfRange,
    FaultDetected,
    StaleDataMismatch,
}