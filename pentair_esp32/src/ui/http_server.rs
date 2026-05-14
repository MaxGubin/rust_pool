use crate::pool::{message::PacketLogElement, PoolProtocolRW};
use anyhow::Result;
use embedded_svc::http::server::{Request, Response, WebSocketResponseBody};
use embedded_svc::http::WebsocketService;
use embedded_svc::io::Write;
use embedded_svc::ws::FrameType;
use embedded_svc::ws::WebSocket;
use embedded_svc::ws::WebSocketStack;
use esp_idf_svc::ws::EspWebSocket;
use futures::StreamExt;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// The result structure from the form.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ControlInput {
    control_name: String,
    state: String,
}

#[derive(Serialize, Debug)]
pub struct SystemState {
    /// Version of the system (pool)
    system_version: u32,

    /// version of the application
    application_version: u32,

    /// Switches state.
    switches: Vec<(String, bool)>,

    /// Temperature sensors.
    temperatures: Vec<(String, f32)>,
}

pub fn handle_control_command(
    pool_protocol: &PoolProtocolRW,
    mut req: Request,
) -> Result<Response> {
    trace!("Handling control command");
    let len = req.content_len().unwrap_or(0);
    let mut buf = vec![0; len as usize];
    req.read_exact(&mut buf)?;
    let control_input: ControlInput = serde_json::from_slice(&buf)?;

    let mut pool_protocol = pool_protocol.write().unwrap();
    let state = control_input.state == "on";
    pool_protocol.change_circuit(&control_input.control_name, state);

    req.into_ok_response("Control command received".as_bytes())
}

pub fn handle_serve_status(pool_protocol: &PoolProtocolRW, req: Request) -> Result<Response> {
    trace!("Serving status page");
    let pool_state = pool_protocol.read().unwrap().get_state();

    let controls_html = pool_state
        .get_controls_state()
        .into_iter()
        .map(|(name, state)| format!("<p>{}: {}</p>", name, if state { "ON" } else { "OFF" }))
        .collect::<String>();

    let temperatures_html = pool_state
        .get_temperatures()
        .into_iter()
        .map(|(name, temp)| format!("<p>{}: {} F</p>", name, temp))
        .collect::<String>();

    let html = include_str!("index.html")
        .replace("{{ controls }}", &controls_html)
        .replace("{{ temperatures }}", &temperatures_html);

    req.into_ok_response(html.as_bytes())
}

pub fn handle_state_json(pool_protocol: &PoolProtocolRW, req: Request) -> Result<Response> {
    trace!("Serving state JSON");
    let pool_state = pool_protocol.read().unwrap().get_state();
    let state = SystemState {
        system_version: 1,
        application_version: 1,
        switches: pool_state.get_controls_state(),
        temperatures: pool_state.get_temperatures(),
    };
    req.into_json(&state)
}

pub fn handle_log_json(pool_protocol: &PoolProtocolRW, req: Request) -> Result<Response> {
    trace!("Serving log page");
    let logs = pool_protocol.read().unwrap().get_recent_packets();

    let log_rows_html = logs
        .into_iter()
        .map(|log_element| {
            let packet_hex = log_element
                .packet_content
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>();
            format!(
                "<tr><td>{}</td><td>{}</td></tr>",
                log_element.timestamp.format("%Y-%m-%d %H:%M:%S"),
                packet_hex
            )
        })
        .collect::<String>();

    let html = include_str!("logs_table.html").replace("{{ logs_rows }}", &log_rows_html);

    req.into_ok_response(html.as_bytes())
}

pub async fn websocket_task(
    mut websocket: EspWebSocket<'static>,
    pool_protocol: PoolProtocolRW,
) -> Result<()> {
    loop {
        match websocket.recv().await {
            Ok(FrameType::Text(msg)) => {
                let msg_str = String::from_utf8_lossy(&msg);
                trace!("Received WebSocket message: {}", msg_str);

                match serde_json::from_str::<ControlInput>(&msg_str) {
                    Ok(control_input) => {
                        let mut pool_protocol_lock = pool_protocol.write().unwrap();
                        let state = control_input.state == "on";
                        pool_protocol_lock.change_circuit(&control_input.control_name, state);
                    }
                    Err(e) => {
                        error!("Failed to deserialize control input: {:?}", e);
                    }
                }

                // Send current system state back to the client
                let state = pool_protocol.read().unwrap().get_state();
                let sstate = SystemState {
                    system_version: 1,
                    application_version: 1,
                    switches: state.get_controls_state(),
                    temperatures: state.get_temperatures(),
                };
                let json = serde_json::to_string(&sstate)?;
                websocket.send_text(&json).await?;
            }
            Ok(FrameType::Close(_)) => {
                trace!("WebSocket closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {:?}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_websocket(pool_protocol: &PoolProtocolRW, req: Request) -> Result<Response> {
    trace!("Handling WebSocket upgrade");
    let (response, websocket) = req.into_websocket(EspWebSocket::wrap)?;

    embassy_executor::spawn(
        "websocket_task",
        websocket_task(websocket, pool_protocol.clone()),
    )?;

    Ok(response)
}
