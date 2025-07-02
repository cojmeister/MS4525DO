//! Sensor Specific Logic

use crate::Ms4525doError;
use defmt::{info, Format};
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;
use heapless::Vec;

/// 7-bit I2C address
const MS4525DO_ADDR: u8 = 0x28;
/// 4-byte data packet
const DATA_SIZE: usize = 4;
const PSI_TO_PA: f32 = 6894.76; // Conversion factor
const READ_MR: u8 = 0x00; // Measurement request command

#[derive(Debug, Format)]
pub enum Status {
    /// 0: Normal Operation. Good Data Packet
    NormalOperation = 0b00,
    /// 1: Reserved
    Reserved = 0b01,
    /// 2: Stale Data. Data has been fetched since last measurement cycle.
    StaleData = 0b10,
    /// 3: Fault Detected
    FaultDetected = 0b11,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Status::NormalOperation,
            0b01 => Status::Reserved,
            0b10 => Status::StaleData,
            0b11 => Status::FaultDetected,
            _ => unreachable!(),
        }
    }
}

pub struct Ms4525do<I2C> {
    i2c: I2C,
    address: u8,
}

const BRIDGE_MASK: u8 = 0b0011_1111;
const TEMPERATURE_MASK: u8 = 0b1110_0000;

impl<I2C> Ms4525do<I2C>
where
    I2C: I2c,
{
    /// Creates a new MS4525DO sensor instance.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C peripheral for communication with the sensor.
    ///
    /// # Returns
    ///
    /// A new `Ms4525do` instance configured with the default I2C address (0x28).
    pub fn new(i2c: I2C) -> Self {
        Ms4525do { i2c, address: MS4525DO_ADDR }
    }

    /// Reads data with double-read validation
    pub async fn read_data(&mut self) -> Result<(f32, f32), Ms4525doError> {
        // Send measurement request
        let cmd = [READ_MR];
        self.i2c
            .write(self.address, &cmd)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;

        // Wait 2ms for fresh data (per PX4)
        Timer::after(Duration::from_millis(2)).await;

        // Read two 4-byte packets
        let mut data_1: Vec<u8, DATA_SIZE> = Vec::new();
        data_1.resize_default(DATA_SIZE).map_err(|_| Ms4525doError::DataOutOfRange)?;
        let mut data_2: Vec<u8, DATA_SIZE> = Vec::new();
        data_2.resize_default(DATA_SIZE).map_err(|_| Ms4525doError::DataOutOfRange)?;

        self.i2c
            .read(self.address, &mut data_1)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;
        self.i2c
            .read(self.address, &mut data_2)
            .await
            .map_err(|_| Ms4525doError::I2cError)?;

        // Parse status
        let status_1 = (data_1[0] >> 6) & 0b11;
        let status_2 = (data_2[0] >> 6) & 0b11;

        // Check for fault
        if status_1 == Status::FaultDetected as u8 || status_2 == Status::FaultDetected as u8 {
            return Err(Ms4525doError::FaultDetected);
        }

        // Validate Normal Operation -> Stale Data
        if status_1 != Status::NormalOperation as u8 || status_2 != Status::StaleData as u8 {
            info!("Invalid status: {} -> {}", status_1, status_2);
            return Err(Ms4525doError::InvalidStatus(Status::from(status_1)));
        }

        // Parse pressure (14-bit)
        let bridge_data_1 = extract_bridge_data(&data_1);

        let bridge_data_2 = extract_bridge_data(&data_2);

        // Parse temperature (11-bit)
        let temperature_digital_counts = read_temperature(&data_1);
        let temp_2 = read_temperature(&data_2);

        // Validate data consistency
        if bridge_data_1 != bridge_data_2 || temperature_digital_counts != temp_2 {
            info!("Data mismatch: pressure {} != {}, temp {} != {}", bridge_data_1, bridge_data_2, temperature_digital_counts, temp_2);
            return Err(Ms4525doError::StaleDataMismatch);
        }

        // Convert pressure to Pascals
        let diff_press_pa = calculate_pressure_differential_pa(bridge_data_1);

        // Convert temperature to Celsius
        let temp_c = calculate_temperature_deg_c(temperature_digital_counts);

        Ok((diff_press_pa, temp_c))
    }
}

pub fn calculate_airspeed(pressure_pa: f32, temp_c: f32) -> f32 {
    let temp_k = temp_c + 273.15;
    let air_density = 101325.0 / (287.05 * temp_k); // Standard pressure at sea level
    libm::sqrtf(2.0 * pressure_pa.abs() / air_density)
}

fn calculate_temperature_deg_c(temperature_counts: u16) -> f32 {
    let temp_c = (200.0 * temperature_counts as f32 / 2047.0) - 50.0;
    temp_c
}

fn calculate_pressure_differential_pa(bridge_data_1: u16) -> f32 {
    let diff_press_psi = -((bridge_data_1 as f32 - 0.1 * 16383.0) * 2.0 / (0.8 * 16383.0) - 1.0);
    let diff_press_pa = diff_press_psi * PSI_TO_PA;
    diff_press_pa
}

fn read_temperature(data_1: &Vec<u8, 4>) -> u16 {
    (((data_1[2] as u16) << 8) | ((data_1[3] & TEMPERATURE_MASK) as u16)) >> 5
}

fn extract_bridge_data(data_1: &Vec<u8, 4>) -> u16 {
    let bridge_msb_1 = data_1[0] & BRIDGE_MASK;
    let bridge_lsb_1 = data_1[1];
    let bridge_data_1 = ((bridge_msb_1 as u16) << 8) | (bridge_lsb_1 as u16);
    bridge_data_1
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;
    use parameterized::parameterized;

    #[test]
    fn test_extract_bridge_data() {
        let data = Vec::from_slice(&[0x3F, 0xFF, 0x80, 0x00]).unwrap();
        let bridge_data = extract_bridge_data(&data);
        assert_eq!(bridge_data, 0x3FFF, "Incorrect bridge data extraction");
    }

    #[test]
    fn test_read_temperature() {
        let data = Vec::from_slice(&[0x00, 0x00, 0x80, 0xE0]).unwrap();
        let temp = read_temperature(&data);
        assert_eq!(temp, 0x0407, "Incorrect temperature extraction"); // (0x80 << 3) | (0xE0 >> 5) = 0x0400 | 0x07
    }

    #[test]
    fn test_calculate_pressure_differential_pa() {
        let bridge_data = 8192; // Mid-range
        let pressure_pa = calculate_pressure_differential_pa(bridge_data);
        assert!((pressure_pa - 0.0).abs() < 1.0, "Pressure calculation incorrect: {}", pressure_pa);
    }

    #[parameterized(digital_counts = {0x0000, 0x0266, 0x03FF},
    expected_temp_c = { -50.0,  10.0, 50.0})]
    fn test_calculate_temperature_deg_c(digital_counts: u16, expected_temp_c: f32) {
        let temp_c = calculate_temperature_deg_c(digital_counts);
        assert!((temp_c - expected_temp_c).abs() < 0.05, "Temperature calculation incorrect: {}", temp_c);
    }
}
