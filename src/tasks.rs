use crate::{calculate_airspeed, Ms4525do, Ms4525doError};
use defmt::info;
use embassy_executor;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

const FIFTY_HERTZ: u64 = 1000 / 50;

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
#[embassy_executor::task]
pub async fn airspeed_task(
    mut sensor: Ms4525do<I2c<'static, Async>>,
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
        Timer::after(Duration::from_millis(FIFTY_HERTZ)).await;
    }
}

