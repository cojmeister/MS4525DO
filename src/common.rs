//! Common utilities and shared logic for MS4525DO sensor communication.
//!
//! This module contains data structures, constants, parsing functions, and
//! calculations that are shared between the blocking and async implementations.

#[cfg(feature = "defmt")]
use defmt::Format;

/// 7-bit I2C address for MS4525DO sensor
pub const MS4525DO_ADDR: u8 = 0x28;

/// Size of data packet read from sensor (4 bytes)
pub const DATA_SIZE: usize = 4;

/// Conversion factor from PSI to Pascals
pub const PSI_TO_PA: f32 = 6894.76;

/// Measurement request command
pub const READ_MR: u8 = 0x00;

/// Mask for extracting bridge (pressure) data from first byte
pub const BRIDGE_MASK: u8 = 0b0011_1111;

/// Mask for extracting temperature data from fourth byte
pub const TEMPERATURE_MASK: u8 = 0b1110_0000;

/// Status codes returned by the MS4525DO sensor.
///
/// The status is encoded in the top 2 bits of the first data byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Status {
    /// Normal Operation - Valid data packet
    NormalOperation = 0b00,
    /// Reserved status code
    Reserved = 0b01,
    /// Stale Data - Data has been fetched since last measurement cycle
    StaleData = 0b10,
    /// Fault Detected - Sensor has detected an error condition
    FaultDetected = 0b11,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Status::NormalOperation,
            0b01 => Status::Reserved,
            0b10 => Status::StaleData,
            0b11 => Status::FaultDetected,
            _ => unreachable!(),
        }
    }
}

/// Extracts the 14-bit bridge (pressure) data from a 4-byte sensor reading.
///
/// The pressure data is stored in bits 0-5 of byte 0 (MSB) and all of byte 1 (LSB).
///
/// # Arguments
///
/// * `data` - 4-byte array read from the sensor
///
/// # Returns
///
/// 14-bit pressure value as u16
#[inline]
pub fn extract_bridge_data(data: &[u8]) -> u16 {
    let bridge_msb = data[0] & BRIDGE_MASK;
    let bridge_lsb = data[1];
    ((bridge_msb as u16) << 8) | (bridge_lsb as u16)
}

/// Extracts the 11-bit temperature data from a 4-byte sensor reading.
///
/// The temperature data is stored in all of byte 2 and bits 5-7 of byte 3.
///
/// # Arguments
///
/// * `data` - 4-byte array read from the sensor
///
/// # Returns
///
/// 11-bit temperature value as u16
#[inline]
pub fn read_temperature(data: &[u8]) -> u16 {
    (((data[2] as u16) << 8) | ((data[3] & TEMPERATURE_MASK) as u16)) >> 5
}

/// Converts raw bridge data to differential pressure in Pascals.
///
/// Uses the transfer function specified in the MS4525DO datasheet for the
/// ±1 PSI differential pressure range (001PD variant).
///
/// # Arguments
///
/// * `bridge_data` - 14-bit raw pressure value from sensor
///
/// # Returns
///
/// Differential pressure in Pascals
pub fn calculate_pressure_differential_pa(bridge_data: u16) -> f32 {
    // Transfer function: P = -((bridge / 16383 * 0.1 - 1) / 0.8 * 2) PSI
    let diff_press_psi = -((bridge_data as f32 - 0.1 * 16383.0) * 2.0 / (0.8 * 16383.0) - 1.0);
    diff_press_psi * PSI_TO_PA
}

/// Converts raw temperature data to degrees Celsius.
///
/// Uses the transfer function specified in the MS4525DO datasheet.
/// Temperature range: -50°C to +150°C
///
/// # Arguments
///
/// * `temperature_counts` - 11-bit raw temperature value from sensor
///
/// # Returns
///
/// Temperature in degrees Celsius
pub fn calculate_temperature_deg_c(temperature_counts: u16) -> f32 {
    (200.0 * temperature_counts as f32 / 2047.0) - 50.0
}

/// Calculates airspeed from differential pressure and temperature.
///
/// Uses the Bernoulli equation: v = sqrt(2 * ΔP / ρ)
/// where ρ (air density) is calculated using the ideal gas law assuming
/// standard atmospheric pressure at sea level (101325 Pa).
///
/// # Arguments
///
/// * `pressure_pa` - Differential pressure in Pascals
/// * `temp_c` - Temperature in degrees Celsius
///
/// # Returns
///
/// Airspeed in meters per second (m/s)
///
/// # Example
///
/// ```
/// use ms4525do::calculate_airspeed;
///
/// let pressure = 50.0; // 50 Pa differential pressure
/// let temperature = 20.0; // 20°C
/// let airspeed = calculate_airspeed(pressure, temperature);
/// println!("Airspeed: {:.2} m/s", airspeed);
/// ```
pub fn calculate_airspeed(pressure_pa: f32, temp_c: f32) -> f32 {
    let temp_k = temp_c + 273.15;
    // Calculate air density using ideal gas law: ρ = P / (R * T)
    // P = 101325 Pa (standard pressure), R = 287.05 J/(kg·K) (specific gas constant for air)
    let air_density = 101325.0 / (287.05 * temp_k);
    // Bernoulli equation for airspeed
    libm::sqrtf(2.0 * pressure_pa.abs() / air_density)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_conversion() {
        assert_eq!(Status::from(0b00), Status::NormalOperation);
        assert_eq!(Status::from(0b01), Status::Reserved);
        assert_eq!(Status::from(0b10), Status::StaleData);
        assert_eq!(Status::from(0b11), Status::FaultDetected);
    }

    #[test]
    fn test_extract_bridge_data() {
        let data = [0x3F, 0xFF, 0x80, 0x00];
        let bridge_data = extract_bridge_data(&data);
        assert_eq!(bridge_data, 0x3FFF, "Incorrect bridge data extraction");
    }

    #[test]
    fn test_read_temperature() {
        let data = [0x00, 0x00, 0x80, 0xE0];
        let temp = read_temperature(&data);
        assert_eq!(temp, 0x0407, "Incorrect temperature extraction");
    }

    #[test]
    fn test_calculate_pressure_differential_pa() {
        let bridge_data = 8192; // Mid-range
        let pressure_pa = calculate_pressure_differential_pa(bridge_data);
        assert!(
            (pressure_pa - 0.0).abs() < 1.0,
            "Pressure calculation incorrect: {}",
            pressure_pa
        );
    }

    #[test]
    fn test_calculate_temperature_deg_c() {
        let test_cases = [(0x0000, -50.0), (0x0266, 10.0), (0x03FF, 50.0)];

        for (digital_counts, expected_temp_c) in test_cases {
            let temp_c = calculate_temperature_deg_c(digital_counts);
            assert!(
                (temp_c - expected_temp_c).abs() < 0.05,
                "Temperature calculation incorrect: {} (expected {})",
                temp_c,
                expected_temp_c
            );
        }
    }

    #[test]
    fn test_calculate_airspeed() {
        // Test with reasonable values
        let pressure = 50.0; // 50 Pa
        let temp = 20.0; // 20°C
        let airspeed = calculate_airspeed(pressure, temp);

        // At 20°C, air density ≈ 1.2 kg/m³
        // v = sqrt(2 * 50 / 1.2) ≈ 9.1 m/s
        assert!(
            airspeed > 8.0 && airspeed < 10.0,
            "Airspeed calculation seems incorrect: {}",
            airspeed
        );
    }
}
