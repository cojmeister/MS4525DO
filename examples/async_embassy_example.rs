//! Example of using the MS4525DO sensor with the async API and Embassy.
//!
//! This example demonstrates how to read pressure and temperature data
//! from the MS4525DO sensor using the async API with the Embassy executor.
//!
//! ## Hardware Requirements
//!
//! - MS4525DO sensor connected via I2C
//! - I2C bus configured at appropriate speed (typically 100-400 kHz)
//! - Sensor connected to default address 0x28
//!
//! ## Usage
//!
//! This is a no_std example designed for Embassy async runtime.
//! You'll need to provide:
//! - An I2C peripheral that implements `embedded_hal_async::i2c::I2c`
//! - Embassy runtime configured for your platform

//! Note: This is a template example that needs to be adapted for your specific platform.
//! The code below is commented out as it requires platform-specific implementations.
//! Uncomment and modify according to your hardware setup.

// Uncomment for no_std embedded platforms:
// #![no_std]
// #![no_main]

// Note: You'll need to adjust these imports based on your platform
// This example shows the general structure for ESP32 with Embassy

// use ms4525do::async_api::Ms4525do;
// use ms4525do::calculate_airspeed;
// use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};
// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
// use embassy_sync::channel::Channel;

// Platform-specific imports (example for ESP32)
// use esp_hal::{
//     gpio,
//     i2c::master::I2c,
//     prelude::*,
//     timer::timg::TimerGroup,
// };
// use esp_backtrace as _;

/// Static channel for sharing sensor data between tasks
// static SENSOR_DATA_CHANNEL: Channel<NoopRawMutex, (f32, f32, f32), 8> = Channel::new();

/// Embassy task to periodically read MS4525DO sensor data.
///
/// Reads pressure and temperature from the sensor at ~50 Hz,
/// calculates airspeed, and sends the results over a channel.
// #[embassy_executor::task]
// async fn read_airspeed_task(
//     mut sensor: Ms4525do<I2c<'static, esp_hal::peripherals::I2C0>>,
// ) {
//     loop {
//         match sensor.read_data().await {
//             Ok((pressure_pa, temp_c)) => {
//                 // Calculate airspeed from pressure and temperature
//                 let airspeed_ms = calculate_airspeed(pressure_pa, temp_c);
//
//                 // Log data using defmt (if enabled)
//                 #[cfg(feature = "defmt")]
//                 defmt::info!(
//                     "Airspeed: {:.2} m/s, Pressure: {:.2} Pa, Temp: {:.2} °C",
//                     airspeed_ms, pressure_pa, temp_c
//                 );
//
//                 // Send data to channel for other tasks to consume
//                 SENSOR_DATA_CHANNEL.send((pressure_pa, temp_c, airspeed_ms)).await;
//             }
//             Err(e) => {
//                 #[cfg(feature = "defmt")]
//                 defmt::warn!("Sensor read error: {:?}", e);
//
//                 // Wait before retrying on error
//                 Timer::after(Duration::from_millis(100)).await;
//             }
//         }
//
//         // Read at ~50 Hz (adjust based on your requirements)
//         Timer::after(Duration::from_millis(20)).await;
//     }
// }

/// Embassy task to consume sensor data.
///
/// This is an example of how another task can receive and process
/// the sensor data from the channel.
///
// #[embassy_executor::task]
// async fn process_data_task() {
//     loop {
//         let (pressure, temp, airspeed) = SENSOR_DATA_CHANNEL.receive().await;
//
//         // Process the data (example: could send over UART, store, etc.)
//         #[cfg(feature = "defmt")]
//         defmt::debug!("Processing: {:.2} m/s", airspeed);
//
//         // Your processing logic here...
//     }
// }

// Main entry point (adjust for your platform)
// #[esp_hal::main]
// async fn main(spawner: Spawner) {
//     // Initialize platform peripherals
//     let peripherals = esp_hal::init(esp_hal::Config::default());
//
//     // Initialize Embassy timer
//     let timg0 = TimerGroup::new(peripherals.TIMG0);
//     esp_hal_embassy::init(timg0.timer0);
//
//     // Configure I2C
//     let i2c = I2c::new(
//         peripherals.I2C0,
//         io.pins.gpio21, // SDA
//         io.pins.gpio22, // SCL
//         100.kHz(),
//     );
//
//     // Create sensor instance
//     let sensor = Ms4525do::new(i2c);
//
//     // Spawn async tasks
//     spawner.spawn(read_airspeed_task(sensor)).ok();
//     spawner.spawn(process_data_task()).ok();
// }

// Dummy main for example compilation
fn main() {
    println!("This is a template example for the MS4525DO sensor with Embassy async runtime.");
    println!("Please refer to the comments above for usage instructions.");
    println!("Adapt the commented code for your specific platform.");
}
