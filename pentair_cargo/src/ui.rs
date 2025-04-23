// Interface implementation

use log::{error, trace};
use serde::{Deserialize, Serialize};

use crate::pool::{protocol::PacketLogElement, PoolProtocolRW};
use askama::Template;
use futures_util::{stream::StreamExt, SinkExt};

use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

// The result structure from the form.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ControlInput {
    control_name: String,
    state: String,
}

pub async fn control_command(
    State(pool_protocol): State<PoolProtocolRW>,
    Json(control_input): Json<ControlInput>,
) {
    trace!("Got client input {:?}", control_input);

    let mut pool_protocol = pool_protocol.write().unwrap();
    let state = control_input.state == "on";
    pool_protocol.change_circuit(&control_input.control_name, state);
}

#[derive(Template)]
#[template(path = "index.html")]
struct UITemplate<'a> {
    pub controls: &'a Vec<(String, bool)>,
    pub temperatures: &'a Vec<(String, f32)>,
}

pub async fn serve_status(State(pool_protocol): State<PoolProtocolRW>) -> impl IntoResponse {
    trace!("Calling status state request");
    // Read the current state
    let pool_state = pool_protocol.read().unwrap().get_state();
    let template = UITemplate {
        controls: &pool_state.get_controls_state(),
        temperatures: &pool_state.get_temperatures(),
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Template processing error {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render a template. Error {}", err),
            )
                .into_response()
        }
    }
}

#[derive(Serialize, Debug)]
struct SystemState {
    /// Version of the system (pool)
    system_version: u32,

    /// version of the application
    application_version: u32,

    /// Switches state.
    switches: Vec<(String, bool)>,

    /// Temperature sensors.
    temperatures: Vec<(String, f32)>,
}

pub async fn state_json(State(pool_protocol): State<PoolProtocolRW>) -> impl IntoResponse {
    trace!("Calling state json request");
    // Read the current state
    let pool_state = pool_protocol.read().unwrap().get_state();
    let state = SystemState {
        system_version: 1,
        application_version: 1,
        switches: pool_state.get_controls_state(),
        temperatures: pool_state.get_temperatures(),
    };
    trace!("Replied with a state {:?}", state);
    Json(state).into_response()
}

#[derive(Template)]
#[template(path = "logs_table.html")]
struct LogsTemplate {
    pub logs: Vec<PacketLogElement>,
}

impl LogsTemplate {
    // This function is used inside the template.
    fn vec_u8_to_hex_string(&self, bytes: &Vec<u8>) -> String {
        bytes
            .iter() // Get an iterator over the bytes (&u8)
            .map(|byte| format!("{:02x}", byte)) // Format each byte to a hex String
            .collect::<String>() // Collect the strings into a single String
    }
}

pub async fn log_json(State(pool_protocol): State<PoolProtocolRW>) -> impl IntoResponse {
    trace!("Calling log");
    let template = LogsTemplate {
        logs: pool_protocol.read().unwrap().get_recent_packets(),
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Template processing error {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render a template. Error {}", err),
            )
                .into_response()
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(pool_protocol): State<PoolProtocolRW>,
) -> impl IntoResponse {
    trace!("Upgrade to Websocket");
    ws.on_upgrade(|socket| async move {
        trace!("Websocket upgraded");
        let (mut tx, mut rx) = socket.split();
        trace!("Websocket split");
        while let Some(Ok(msg)) = rx.next().await {
            match msg {
                Message::Text(text) => {
                    trace!("Got a text message: {}", text);
                    match serde_json::from_str::<ControlInput>(text.as_str()) {
                        Err(e) => error!("Client sent misforemed json {e:?}"),
                        Ok(control_input) => {
                            let mut pool_protocol = pool_protocol.write().unwrap();
                            let state = control_input.state == "on";
                            pool_protocol.change_circuit(&control_input.control_name, state);
                        }
                    }
                    let state = pool_protocol.read().unwrap().get_state();
                    let sstate = SystemState {
                        system_version: 1,
                        application_version: 1,
                        switches: state.get_controls_state(),
                        temperatures: state.get_temperatures(),
                    };
                    let json = serde_json::to_string(&sstate).unwrap();
                    tx.send(Message::Text(json.into())).await.unwrap();
                }
                Message::Binary(_) => {
                    trace!("Got a binary message");
                }
                Message::Ping(_) => {
                    trace!("Got a ping");
                }
                Message::Pong(_) => {
                    trace!("Got a pong");
                }
                Message::Close(_) => {
                    trace!("Got a close");
                }
            }
        }
        trace!("Exit Websocket loop");
    })
}
