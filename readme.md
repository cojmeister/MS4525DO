# Airspeed Driver

HAL for the PX4AIRSPEEDV1.1 (MS4525DO) sensor, providing async I2C communication for reading 14-bit pressure and 11-bit
temperature data. Designed for a `no_std` environment using Embassy.
Has no dynamic memory allocations due to heapless.

## Usage

- Initialize with `Ms4525do::new(i2c)`
- Use `read_data` to asynchronously fetch pressure and temperature.
