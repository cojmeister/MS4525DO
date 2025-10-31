//! Example of using the MS4525DO sensor on a standard computer (std environment).
//!
//! This example uses a mock I2C implementation to simulate the MS4525DO sensor,
//! allowing you to test the library on a standard computer without actual hardware.
//!
//! Run with: cargo run --example std_mock_example --features "blocking,std"

use ms4525do::blocking::Ms4525do;
use ms4525do::calculate_airspeed;
use std::thread;
use std::time::Duration;

/// Mock I2C implementation that simulates MS4525DO sensor responses
struct MockI2c {
    measurement_count: u32,
    read_count: u8,
}

impl MockI2c {
    fn new() -> Self {
        Self {
            measurement_count: 0,
            read_count: 0,
        }
    }

    /// Simulates sensor data based on a sine wave pattern
    /// This creates varying pressure readings that simulate airspeed changes
    fn generate_sensor_data(&self, is_fresh: bool) -> [u8; 4] {
        // Simulate varying pressure using a sine wave
        let angle = (self.measurement_count as f32 * 0.1).sin();

        // Simulate pressure around midpoint with some variation
        // Range: roughly 8000-8400 counts (around 0 differential pressure)
        let pressure_counts = (8192.0 + angle * 200.0) as u16;

        // Simulate temperature around 25°C (counts around 767)
        let temp_counts = 767u16 + ((angle * 50.0) as u16);

        // Status: 0b00 for fresh data, 0b10 for stale
        let status = if is_fresh { 0b00 } else { 0b10 };

        // Pack into 4-byte format according to MS4525DO specification
        let byte0 = ((status << 6) | ((pressure_counts >> 8) & 0x3F)) as u8;
        let byte1 = (pressure_counts & 0xFF) as u8;
        let byte2 = (temp_counts >> 3) as u8;
        let byte3 = ((temp_counts & 0x07) << 5) as u8;

        [byte0, byte1, byte2, byte3]
    }
}

/// Mock delay implementation
struct MockDelay;

impl embedded_hal::delay::DelayNs for MockDelay {
    fn delay_ns(&mut self, ns: u32) {
        thread::sleep(Duration::from_nanos(ns as u64));
    }
}

// Implement the embedded_hal I2C traits for our mock
impl embedded_hal::i2c::ErrorType for MockI2c {
    type Error = MockI2cError;
}

#[derive(Debug)]
struct MockI2cError;

impl embedded_hal::i2c::Error for MockI2cError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

impl embedded_hal::i2c::I2c for MockI2c {
    fn transaction(
        &mut self,
        _address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                embedded_hal::i2c::Operation::Write(_data) => {
                    // Measurement request received
                    self.measurement_count += 1;
                    self.read_count = 0;
                }
                embedded_hal::i2c::Operation::Read(buffer) => {
                    // First read returns fresh data, second read returns stale
                    let is_fresh = self.read_count == 0;
                    let data = self.generate_sensor_data(is_fresh);
                    buffer.copy_from_slice(&data);
                    self.read_count += 1;
                }
            }
        }
        Ok(())
    }
}

fn main() {
    println!("MS4525DO Mock Example");
    println!("======================\n");
    println!("This example simulates reading from an MS4525DO sensor using mock I2C.");
    println!(
        "The simulated sensor produces varying airspeed readings using a sine wave pattern.\n"
    );

    // Create mock I2C and delay
    let mock_i2c = MockI2c::new();
    let mut delay = MockDelay;

    // Create sensor instance
    let mut sensor = Ms4525do::new(mock_i2c);

    println!("Reading sensor data at 10 Hz for 5 seconds...\n");
    println!(
        "{:<10} {:<15} {:<15} {:<15}",
        "Reading", "Pressure (Pa)", "Temp (°C)", "Airspeed (m/s)"
    );
    println!("{:-<60}", "");

    // Read sensor data at ~10 Hz for 5 seconds
    for i in 1..=50 {
        match sensor.read_data(&mut delay) {
            Ok((pressure_pa, temp_c)) => {
                let airspeed = calculate_airspeed(pressure_pa, temp_c);

                println!(
                    "{:<10} {:<15.2} {:<15.2} {:<15.2}",
                    i, pressure_pa, temp_c, airspeed
                );

                // Demonstrate error handling every 20 readings
                if i % 20 == 0 {
                    println!("\n✓ Successfully completed {} readings\n", i);
                }
            }
            Err(e) => {
                println!("Error reading sensor: {:?}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Sleep for ~100ms (10 Hz)
        thread::sleep(Duration::from_millis(100));
    }

    println!("\n{:-<60}", "");
    println!("\n✓ Example completed successfully!");
    println!("\nKey observations:");
    println!("  • Pressure values vary in a sine wave pattern");
    println!("  • Temperature fluctuates slightly with the pattern");
    println!("  • Airspeed is calculated from pressure and temperature");
    println!("  • All data is validated using double-read verification");
}
