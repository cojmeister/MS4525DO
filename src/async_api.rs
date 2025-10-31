//! Asynchronous (non-blocking) API for MS4525DO sensor communication.
//!
//! This module provides an async implementation using `embedded-hal-async` traits,
//! suitable for use with async executors like Embassy on embedded systems.
//!
//! # Example
//!
//! ```ignore
//! use ms4525do::async_api::Ms4525do;
//! use embassy_time::{Duration, Timer};
//!
//! let mut sensor = Ms4525do::new(i2c);
//!
//! loop {
//!     match sensor.read_data().await {
//!         Ok((pressure, temp)) => {
//!             let airspeed = ms4525do::calculate_airspeed(pressure, temp);
//!             println!("Airspeed: {} m/s", airspeed);
//!         }
//!         Err(e) => {
//!             println!("Error: {:?}", e);
//!             Timer::after(Duration::from_millis(100)).await;
//!         }
//!     }
//!     Timer::after(Duration::from_millis(20)).await;
//! }
//! ```

use crate::common::*;
use crate::Ms4525doError;
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

#[cfg(feature = "defmt")]
use defmt::info;

/// MS4525DO sensor driver with async I2C communication.
///
/// This struct is generic over the I2C peripheral type, allowing it to work
/// with any I2C implementation that implements the `embedded_hal_async::i2c::I2c` trait.
///
/// # Type Parameters
///
/// * `I2C` - The I2C peripheral type implementing `embedded_hal_async::i2c::I2c`
pub struct Ms4525do<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Ms4525do<I2C>
where
    I2C: I2c,
{
    /// Creates a new MS4525DO sensor instance with the default I2C address.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C peripheral for communication with the sensor
    ///
    /// # Returns
    ///
    /// A new `Ms4525do` instance configured with the default I2C address (0x28)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sensor = Ms4525do::new(i2c);
    /// ```
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: MS4525DO_ADDR,
        }
    }

    /// Creates a new MS4525DO sensor instance with a custom I2C address.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C peripheral for communication with the sensor
    /// * `address` - Custom 7-bit I2C address
    ///
    /// # Returns
    ///
    /// A new `Ms4525do` instance configured with the specified I2C address
    pub fn new_with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Reads pressure and temperature data from the sensor asynchronously.
    ///
    /// This method implements a double-read validation strategy to ensure data freshness:
    /// 1. Sends a measurement request command
    /// 2. Waits 2ms for fresh data (as per datasheet recommendations)
    /// 3. Reads two consecutive 4-byte packets
    /// 4. Validates status progression: NormalOperation → StaleData
    /// 5. Ensures pressure and temperature consistency between reads
    ///
    /// # Returns
    ///
    /// * `Ok((f32, f32))` - Tuple of (differential_pressure_pa, temperature_c)
    /// * `Err(Ms4525doError)` - Error if communication fails or data is invalid
    ///
    /// # Errors
    ///
    /// * `Ms4525doError::I2cError` - I2C communication failure
    /// * `Ms4525doError::FaultDetected` - Sensor reports fault status
    /// * `Ms4525doError::InvalidStatus` - Unexpected status code
    /// * `Ms4525doError::StaleDataMismatch` - Data inconsistency between reads
    ///
    /// # Example
    ///
    /// ```ignore
    /// match sensor.read_data().await {
    ///     Ok((pressure, temp)) => {
    ///         println!("Pressure: {} Pa, Temperature: {} °C", pressure, temp);
    ///     }
    ///     Err(e) => println!("Read error: {:?}", e),
    /// }
    /// ```
    pub async fn read_data(&mut self) -> Result<(f32, f32), Ms4525doError> {
        // Send measurement request
        let cmd = [READ_MR];
        self.i2c
            .write(self.address, &cmd)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;

        // Wait 2ms for fresh data (per datasheet and PX4 implementation)
        Timer::after(Duration::from_millis(2)).await;

        // Read two consecutive 4-byte packets for validation
        let mut data_1 = [0u8; DATA_SIZE];
        let mut data_2 = [0u8; DATA_SIZE];

        self.i2c
            .read(self.address, &mut data_1)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;

        self.i2c
            .read(self.address, &mut data_2)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;

        // Parse status from both reads
        let status_1 = Status::from(data_1[0] >> 6);
        let status_2 = Status::from(data_2[0] >> 6);

        // Check for sensor fault
        if status_1 == Status::FaultDetected || status_2 == Status::FaultDetected {
            return Err(Ms4525doError::FaultDetected);
        }

        // Validate expected status progression: Normal → Stale
        // This ensures we're getting fresh data followed by the same stale data
        if status_1 != Status::NormalOperation || status_2 != Status::StaleData {
            #[cfg(feature = "defmt")]
            info!("Invalid status sequence: {:?} -> {:?}", status_1, status_2);

            #[cfg(all(not(feature = "defmt"), feature = "log"))]
            log::info!("Invalid status sequence: {:?} -> {:?}", status_1, status_2);

            return Err(Ms4525doError::InvalidStatus(status_1));
        }

        // Extract pressure and temperature from both reads
        let bridge_data_1 = extract_bridge_data(&data_1);
        let bridge_data_2 = extract_bridge_data(&data_2);
        let temperature_1 = read_temperature(&data_1);
        let temperature_2 = read_temperature(&data_2);

        // Validate data consistency between reads
        if bridge_data_1 != bridge_data_2 || temperature_1 != temperature_2 {
            #[cfg(feature = "defmt")]
            info!(
                "Data mismatch: pressure {} != {}, temp {} != {}",
                bridge_data_1, bridge_data_2, temperature_1, temperature_2
            );

            #[cfg(all(not(feature = "defmt"), feature = "log"))]
            log::info!(
                "Data mismatch: pressure {} != {}, temp {} != {}",
                bridge_data_1,
                bridge_data_2,
                temperature_1,
                temperature_2
            );

            return Err(Ms4525doError::StaleDataMismatch);
        }

        // Convert to physical units
        let diff_press_pa = calculate_pressure_differential_pa(bridge_data_1);
        let temp_c = calculate_temperature_deg_c(temperature_1);

        Ok((diff_press_pa, temp_c))
    }

    /// Consumes the sensor driver and returns the underlying I2C peripheral.
    ///
    /// This is useful when you need to reuse the I2C peripheral for other devices.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
