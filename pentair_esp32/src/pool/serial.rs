use crate::config;
use crate::pool::{
    message,
    protocol::{self, PoolProtocolRW},
};
use anyhow::{bail, Result};
use esp_idf_hal::gpio;
use esp_idf_hal::uart::{self, UartConfig, UartDriver};
use log::{debug, error, trace};

pub fn serial_port(
    uart: uart::Uart,
    tx: gpio::Pin,
    rx: gpio::Pin,
    parameters: &config::config_json::PortParameters,
) -> Result<UartDriver<'static>> {
    let config = UartConfig::new()
        .baudrate(parameters.baud_rate as u32)
        .data_bits(config::config_json::decode_char_size(parameters.char_size))
        .parity(config::config_json::decode_parity(
            parameters.parity.as_str(),
        ))
        .stop_bits(config::config_json::decode_stop_bits(parameters.stop_bits));
    let uart_driver = UartDriver::new(
        uart,
        tx,
        rx,
        Option::<gpio::AnyInputPin>::None,
        Option::<gpio::AnyOutputPin>::None,
        &config,
    )?;

    Ok(uart_driver)
}

pub async fn port_read_task(
    mut uart: UartDriver<'static>,
    pool_protocol: PoolProtocolRW,
) -> Result<()> {
    trace!("Pool monitor task started");

    let mut buf = [0_u8; 256];

    loop {
        match uart.read(&mut buf, esp_idf_hal::delay::BLOCK).await {
            Ok(len) => {
                if len > 0 {
                    debug!("Read {} bytes from UART: {:?}", len, &buf[..len]);
                    // Here you would parse the bytes into messages
                    // For now, just print them
                }
            }
            Err(e) => {
                error!("Failed to read from UART: {:?}", e);
                // Optionally, introduce a small delay before retrying
                embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
            }
        }
    }
}
