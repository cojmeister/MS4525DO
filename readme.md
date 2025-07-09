# Airspeed Driver

HAL for the PX4AIRSPEEDV1.1 (MS4525DO) sensor, providing async I2C communication for reading 14-bit pressure and 11-bit
temperature data. Designed for a `no_std` environment using Embassy.
Has no dynamic memory allocations due to heapless.

## Usage

- Initialize with `Ms4525do::new(i2c)`
- Use `read_data` to asynchronously fetch pressure and temperature.

### Embassy Task Usage
I'm trying (miserably failing) to implement this as an embassy async task. Like so:
```rust
#![no_std]
#![no_main]

use crate::{calculate_airspeed, Ms4525do};
use defmt::info;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::I2c;
use esp_hal::{gpio, timer::timg::TimerGroup};

/// Embassy task to periodically read MS4525DO sensor data and calculate airspeed.
///
/// Reads pressure (Pa) and temperature (°C) from the sensor at ~50 Hz (20ms intervals),
/// calculates airspeed (m/s), and sends the results over a channel. Logs data and errors
/// using `defmt` for debugging.
///
/// # Arguments
///
/// * `sensor` - The MS4525DO sensor instance.
/// * `channel` - Channel to send (pressure, temperature, airspeed) tuples.

// Declare async tasks
#[embassy_executor::task]
async fn read_airspeed_task(mut sensor: Ms4525do<I2c<'static, esp_hal::peripherals::I2C0>>,
                            channel: &'static Channel<NoopRawMutex, (f32, f32, f32), 8>,
) {
    loop {
        match sensor.read_data().await {
            Ok((pressure, temp)) => {
                let airspeed = calculate_airspeed(pressure, temp);
                info!(
                    "Airspeed: {} m/s, Pressure: {} Pa, Temp: {} C",
                    airspeed, pressure, temp
                );
                channel.send((pressure, temp, airspeed)).await;
            }
            Err(e) => {
                info!("Airspeed read error: {}", e);
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        Timer::after(Duration::from_millis(20)).await; // ~50 Hz
    }
}
```

Any help is appreciated!

## Key Points:

 - Frequency: Reads every 20ms (~50 Hz), matching the sensor’s capability and your project’s requirements.
 - Channel: Sends a (pressure, temp, airspeed) tuple ((f32, f32, f32)) over a Channel with capacity 8, using NoopRawMutex for single-core safety on ESP32.
 - Error Handling: Logs errors with defmt and waits 100ms before retrying to avoid overwhelming the system.
 - Async: Uses embedded-hal-async for non-blocking I2C operations and embassy-time for timing.
 - Modularity: Defined in tasks.rs to keep the HAL (ms4525do.rs) separate from task logic.