//! # MS4525DO Airspeed Sensor Driver
//!
//! A platform-agnostic Rust driver for the MS4525DO differential pressure sensor,
//! commonly used for airspeed measurement in drones and aircraft.
//!
//! ## Features
//!
//! - **Dual API**: Both blocking and async implementations
//! - **Platform agnostic**: Works on any platform with I2C support
//! - **`no_std` compatible**: Suitable for embedded systems
//! - **Zero dynamic allocation**: All operations use stack memory
//! - **Validated readings**: Double-read validation ensures data freshness
//! - **Flexible logging**: Optional `defmt` or `log` support
//!
//! ## Usage
//!
//! ### Blocking API
//!
//! ```ignore
//! use ms4525do::blocking::Ms4525do;
//! use embedded_hal::delay::DelayNs;
//!
//! let mut sensor = Ms4525do::new(i2c);
//! let mut delay = /* your delay implementation */;
//!
//! match sensor.read_data(&mut delay) {
//!     Ok((pressure_pa, temp_c)) => {
//!         let airspeed = ms4525do::calculate_airspeed(pressure_pa, temp_c);
//!         println!("Airspeed: {:.2} m/s", airspeed);
//!     }
//!     Err(e) => println!("Error: {:?}", e),
//! }
//! ```
//!
//! ### Async API
//!
//! ```ignore
//! use ms4525do::async_api::Ms4525do;
//! use embassy_time::{Duration, Timer};
//!
//! let mut sensor = Ms4525do::new(i2c);
//!
//! match sensor.read_data().await {
//!     Ok((pressure_pa, temp_c)) => {
//!         let airspeed = ms4525do::calculate_airspeed(pressure_pa, temp_c);
//!         println!("Airspeed: {:.2} m/s", airspeed);
//!     }
//!     Err(e) => println!("Error: {:?}", e),
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `async` (default): Enable async API with embassy-time
//! - `blocking`: Enable blocking/synchronous API
//! - `std`: Enable std support (for desktop/server environments)
//! - `defmt`: Enable defmt logging for embedded debugging
//! - `log`: Enable log facade for flexible logging
//!
//! ## Sensor Details
//!
//! The MS4525DO is a digital differential pressure sensor with:
//! - I2C interface (default address: 0x28)
//! - 14-bit pressure resolution
//! - 11-bit temperature resolution
//! - ±1 PSI measurement range (001PD variant)
//! - Operating temperature: -50°C to +150°C

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Module declarations
mod common;
mod error;

#[cfg(feature = "async")]
pub mod async_api;

#[cfg(feature = "blocking")]
pub mod blocking;

// Re-export public types and functions
pub use common::{calculate_airspeed, Status};
pub use error::Ms4525doError;

// For backwards compatibility and convenience, re-export the default API at the root level
#[cfg(all(feature = "async", not(feature = "blocking")))]
pub use async_api::Ms4525do;

#[cfg(all(feature = "blocking", not(feature = "async")))]
pub use blocking::Ms4525do;

// If both features are enabled, don't export at root to avoid ambiguity
// Users must explicitly use async_api::Ms4525do or blocking::Ms4525do
