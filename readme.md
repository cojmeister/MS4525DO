# MS4525DO Airspeed Sensor Driver

[![Crate](https://img.shields.io/crates/v/ms4525do.svg)](https://crates.io/crates/ms4525do)
[![Documentation](https://docs.rs/ms4525do/badge.svg)](https://docs.rs/ms4525do)
[![License](https://img.shields.io/crates/l/ms4525do.svg)](https://github.com/cojmeister/ms4525do)

A platform-agnostic Rust driver for the **MS4525DO** differential pressure sensor, commonly used for airspeed measurement in drones, aircraft, and UAVs.

## Features

- 🚀 **Dual API**: Both blocking and async implementations
- 🔌 **Platform agnostic**: Works on any platform with I2C support (`embedded-hal` / `embedded-hal-async`)
- 📦 **`no_std` compatible**: Perfect for embedded systems
- 🧮 **Zero dynamic allocation**: All operations use stack memory
- ✅ **Validated readings**: Double-read validation ensures data freshness
- 📊 **Built-in airspeed calculation**: Convert pressure to airspeed
- 🔍 **Flexible logging**: Optional `defmt` or `log` support
- 🛡️ **Safe**: `#![forbid(unsafe_code)]`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ms4525do = "0.1.0"
```

### Feature Flags

- `async` (default): Enable async API with embassy-time
- `blocking`: Enable blocking/synchronous API
- `std`: Enable std support (for desktop/server environments)
- `defmt`: Enable defmt logging for embedded debugging
- `log`: Enable log facade for flexible logging

**Examples:**

```toml
# Async only (default)
ms4525do = "0.1.0"

# Blocking only
ms4525do = { version = "0.1.0", default-features = false, features = ["blocking"] }

# Both async and blocking
ms4525do = { version = "0.1.0", features = ["blocking"] }

# With defmt logging
ms4525do = { version = "0.1.0", features = ["defmt"] }
```

## Quick Start

### Blocking API

```rust
use ms4525do::blocking::Ms4525do;
use embedded_hal::delay::DelayNs;

// Create sensor instance
let mut sensor = Ms4525do::new(i2c);
let mut delay = /* your delay implementation */;

// Read sensor data
match sensor.read_data(&mut delay) {
    Ok((pressure_pa, temp_c)) => {
        let airspeed = ms4525do::calculate_airspeed(pressure_pa, temp_c);
        println!("Airspeed: {:.2} m/s", airspeed);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

### Async API (Embassy)

```rust
use ms4525do::async_api::Ms4525do;
use embassy_time::{Duration, Timer};

// Create sensor instance
let mut sensor = Ms4525do::new(i2c);

// Read sensor data
match sensor.read_data().await {
    Ok((pressure_pa, temp_c)) => {
        let airspeed = ms4525do::calculate_airspeed(pressure_pa, temp_c);
        println!("Airspeed: {:.2} m/s", airspeed);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

## Hardware Setup

### Connections

The MS4525DO uses I2C communication:

| MS4525DO Pin | Connection |
|--------------|------------|
| VCC | 3.3V or 5V |
| GND | Ground |
| SDA | I2C Data |
| SCL | I2C Clock |

**Default I2C Address:** `0x28`

### Recommended I2C Speed

- **Standard mode**: 100 kHz
- **Fast mode**: 400 kHz (recommended)

## Sensor Specifications

- **Measurement range**: ±1 PSI (differential pressure)
- **Pressure resolution**: 14-bit
- **Temperature resolution**: 11-bit
- **Operating temperature**: -50°C to +150°C
- **Update rate**: Up to 50 Hz
- **Interface**: I2C (7-bit address: 0x28)

## Examples

See the [`examples/`](./examples) directory for complete examples:

- [`std_mock_example.rs`](./examples/std_mock_example.rs) - **Runnable on your computer!** Mock I2C example with std
- [`blocking_example.rs`](./examples/blocking_example.rs) - Embedded blocking usage template
- [`async_embassy_example.rs`](./examples/async_embassy_example.rs) - Embedded async with Embassy runtime

### Try It Now!

Run the mock example on your computer (no hardware needed):

```bash
cargo run --example std_mock_example --features "blocking,std"
```

This demonstrates the full sensor workflow with simulated I2C data.

### Reading at 50 Hz

```rust
use embassy_time::{Duration, Timer};

loop {
    match sensor.read_data().await {
        Ok((pressure, temp)) => {
            let airspeed = ms4525do::calculate_airspeed(pressure, temp);
            // Process data...
        }
        Err(e) => {
            log::error!("Sensor error: {:?}", e);
            Timer::after(Duration::from_millis(100)).await;
        }
    }
    Timer::after(Duration::from_millis(20)).await; // ~50 Hz
}
```

## How It Works

### Double-Read Validation

The driver implements a robust double-read validation strategy:

1. Sends measurement request command
2. Waits 2ms for fresh data (per datasheet)
3. Reads two consecutive 4-byte packets
4. Validates status progression: `NormalOperation` → `StaleData`
5. Ensures pressure and temperature consistency between reads

This approach ensures you always get fresh, validated data from the sensor.

### Airspeed Calculation

The `calculate_airspeed()` function uses the Bernoulli equation:

```
v = √(2 × ΔP / ρ)
```

Where:
- `v` = airspeed (m/s)
- `ΔP` = differential pressure (Pa)
- `ρ` = air density (calculated from temperature)

## Error Handling

The driver provides detailed error types:

```rust
pub enum Ms4525doError {
    I2cError,              // I2C communication failure
    InvalidStatus(Status), // Unexpected sensor status
    DataOutOfRange,        // Buffer allocation failure
    FaultDetected,         // Sensor fault condition
    StaleDataMismatch,     // Data validation failure
}
```

## Platform Examples

<details>
<summary>ESP32 (ESP-HAL)</summary>

```rust
use esp_hal::i2c::master::I2c;
use ms4525do::async_api::Ms4525do;

let i2c = I2c::new(
    peripherals.I2C0,
    io.pins.gpio21, // SDA
    io.pins.gpio22, // SCL
    100.kHz(),
);

let mut sensor = Ms4525do::new(i2c);
```
</details>

<details>
<summary>STM32 (Embassy)</summary>

```rust
use embassy_stm32::i2c::I2c;
use ms4525do::async_api::Ms4525do;

let i2c = I2c::new(
    p.I2C1,
    p.PB8,  // SCL
    p.PB9,  // SDA
    Hertz(100_000),
);

let mut sensor = Ms4525do::new(i2c);
```
</details>

<details>
<summary>Raspberry Pi Pico (RP2040)</summary>

```rust
use rp2040_hal::i2c::I2C;
use ms4525do::blocking::Ms4525do;

let i2c = I2C::i2c0(
    pac.I2C0,
    sda_pin,
    scl_pin,
    100.kHz(),
);

let mut sensor = Ms4525do::new(i2c);
```
</details>

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Based on the PX4 airspeed driver implementation
- Datasheet: [MS4525DO Digital Pressure Sensor](https://www.te.com/commerce/DocumentDelivery/DDEController?Action=showdoc&DocId=Data+Sheet%7FMS4525%7FB9%7Fpdf%7FEnglish%7FENG_DS_MS4525_B9.pdf%7FCAT-BLPS0041)

## Resources

- [Documentation](https://docs.rs/ms4525do)
- [Repository](https://github.com/cojmeister/ms4525do)
- [Issue Tracker](https://github.com/cojmeister/ms4525do/issues)
