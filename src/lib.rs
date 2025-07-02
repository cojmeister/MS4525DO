#![no_std]

mod error;
mod ms4525do;
mod tasks;

pub use error::Ms4525doError;
pub use ms4525do::{calculate_airspeed, Ms4525do, Status};
pub use tasks::airspeed_task;
