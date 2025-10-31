//! Error types for MS4525DO sensor operations.

use crate::common::Status;

#[cfg(feature = "defmt")]
use defmt::Format;

/// Errors that can occur during MS4525DO sensor operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Ms4525doError {
    /// I2C communication error occurred.
    ///
    /// This error indicates a failure in the I2C bus communication,
    /// which could be caused by:
    /// - Bus contention
    /// - Electrical noise
    /// - Incorrect wiring
    /// - Sensor not responding
    I2cError,

    /// Sensor returned an unexpected status code.
    ///
    /// The sensor's status should progress from `NormalOperation` to `StaleData`
    /// when performing a double-read validation. This error indicates an
    /// unexpected status code was received.
    ///
    /// Contains the unexpected `Status` value that was received.
    InvalidStatus(Status),

    /// Internal buffer allocation failed.
    ///
    /// This error should not occur in normal operation as the buffer size
    /// is fixed at 4 bytes. If this occurs, it may indicate memory corruption.
    DataOutOfRange,

    /// Sensor detected a fault condition.
    ///
    /// The sensor has detected an internal fault. Common causes:
    /// - Sensor power supply issues
    /// - Internal sensor malfunction
    /// - Out-of-range pressure applied to sensor
    ///
    /// Consider power cycling the sensor or checking for physical damage.
    FaultDetected,

    /// Data validation failed between consecutive reads.
    ///
    /// When performing double-read validation, the pressure or temperature
    /// values did not match between the first and second read. This indicates:
    /// - Sensor was read during a measurement cycle
    /// - Timing issues with sensor communication
    /// - Potential sensor malfunction
    ///
    /// Try reading again after a short delay.
    StaleDataMismatch,
}

impl core::fmt::Display for Ms4525doError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ms4525doError::I2cError => {
                write!(f, "I2C communication error")
            }
            Ms4525doError::InvalidStatus(status) => {
                write!(f, "Invalid sensor status: {:?}", status)
            }
            Ms4525doError::DataOutOfRange => {
                write!(f, "Internal buffer allocation failed")
            }
            Ms4525doError::FaultDetected => {
                write!(f, "Sensor fault detected")
            }
            Ms4525doError::StaleDataMismatch => {
                write!(f, "Data validation failed between consecutive reads")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Ms4525doError {}
