#![no_std]

mod error;
mod ms4525do;

pub use error::Ms4525doError;
pub use ms4525do::{calculate_airspeed, Ms4525do, Status};