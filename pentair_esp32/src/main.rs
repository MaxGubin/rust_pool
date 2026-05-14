use esp_idf_sys as _;

use anyhow::{bail, Result};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{EspHttpServer, Request};
use esp_idf_svc::nvs::EspNvs;
use esp_idf_svc::nvs::NvsCustom;
use esp_idf_svc::nvs::Partition;
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};

use std::sync::{Arc, RwLock};

mod config;
mod pool;
mod ui;

#[embassy_executor::main]
pub async fn main() -> Result<()> {
    esp_idf_svc::log::EspLogger::initialize();

    log::info!("Hello from ESP32!");

    let peripherals =
        Peripherals::new().map_err(|e| anyhow::anyhow!("Failed to get peripherals: {:?}", e))?;
    let sys_loop = EspSystemEventLoop::take()
        .map_err(|e| anyhow::anyhow!("Failed to get system event loop: {:?}", e))?;
    let nvs_default_partition =
        Partition::take().map_err(|e| anyhow::anyhow!("Failed to get NVS partition: {:?}", e))?;
    let mut nvs = EspNvs::new(nvs_default_partition.clone(), "config", true)
        .map_err(|e| anyhow::anyhow!("Failed to create NVS handle: {:?}", e))?;

    let pool_config = match config::load_configuration(&mut nvs) {
        Ok(cfg) => cfg,
        Err(e) => {
            log::warn!(
                "Failed to load configuration from NVS: {}. Using default configuration.",
                e
            );
            let default_config = config::PoolConfig::default();
            config::save_configuration(&mut nvs, &default_config)?;
            default_config
        }
    };

    log::info!("Loaded configuration: {:?}", pool_config);

    // Initialize Wi-Fi
    let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs.clone()))?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
    log::info!("Wifi configuration set, about to connect...");

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    log::info!("Wifi connected!");

    let pool_protocol = Arc::new(RwLock::new(pool::protocol::PoolProtocol::new()));

    // TODO: Use actual pins from peripherals for UART
    // For now, let's assume UART1 and some dummy pins
    let uart1 = peripherals.uart1;
    let tx_pin = peripherals.pins.gpio4;
    let rx_pin = peripherals.pins.gpio5;

    let uart_driver =
        pool::serial::serial_port(uart1, tx_pin, rx_pin, &pool_config.port_parameters)?;

    embassy_executor::spawn(
        "port_read_task",
        pool::serial::port_read_task(uart_driver, pool_protocol.clone()),
    )?;

    let mut server = EspHttpServer::new(&Default::default())?;

    let pool_protocol_clone = pool_protocol.clone();
    server.fn_handler("/", embedded_svc::http::Method::Get, move |req| {
        ui::http_server::handle_serve_status(&pool_protocol_clone, req)
    })?;

    let pool_protocol_clone = pool_protocol.clone();
    server.fn_handler("/control", embedded_svc::http::Method::Post, move |req| {
        ui::http_server::handle_control_command(&pool_protocol_clone, req)
    })?;

    let pool_protocol_clone = pool_protocol.clone();
    server.fn_handler("/state", embedded_svc::http::Method::Get, move |req| {
        ui::http_server::handle_state_json(&pool_protocol_clone, req)
    })?;

    let pool_protocol_clone = pool_protocol.clone();
    server.fn_handler("/log", embedded_svc::http::Method::Get, move |req| {
        ui::http_server::handle_log_json(&pool_protocol_clone, req)
    })?;

    let pool_protocol_clone = pool_protocol.clone();
    server.fn_handler("/ws", embedded_svc::http::Method::Get, move |req| {
        ui::http_server::handle_websocket(&pool_protocol_clone, req)
    })?;

    // Serve static assets
    server.fn_handler("/assets/style.css", embedded_svc::http::Method::Get, |req| {
        req.into_ok_response(include_bytes!("../../pentair_cargo/assets/style.css"))
    })?;

    server.fn_handler("/assets/script.js", embedded_svc::http::Method::Get, |req| {
        req.into_ok_response(include_bytes!("../../pentair_cargo/assets/script.js"))
    })?;

    log::info!("HTTP server started");

    // Keep the main task alive
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
    }
