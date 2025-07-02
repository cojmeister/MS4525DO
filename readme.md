# Airspeed Driver

HAL for the PX4AIRSPEEDV1.1 (MS4525DO) sensor, providing async I2C communication for reading 14-bit pressure and 11-bit
temperature data. Designed for a `no_std` environment using Embassy.
Has no dynamic memory allocations due to heapless.

## Usage

- Initialize with `Ms4525do::new(i2c)`
- Use `read_data` to asynchronously fetch pressure and temperature.

## Key Points:

 - Frequency: Reads every 20ms (~50 Hz), matching the sensor’s capability and your project’s requirements.
 - Channel: Sends a (pressure, temp, airspeed) tuple ((f32, f32, f32)) over a Channel with capacity 8, using NoopRawMutex for single-core safety on ESP32.
 - Error Handling: Logs errors with defmt and waits 100ms before retrying to avoid overwhelming the system.
 - Async: Uses embedded-hal-async for non-blocking I2C operations and embassy-time for timing.
 - Modularity: Defined in tasks.rs to keep the HAL (ms4525do.rs) separate from task logic.