// Interface implementation

use log::{error, trace};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use crate::protocol;
use askama::Template;

use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use futures::{sink::SinkExt, stream::StreamExt};

pub type PoolProtocolRW = Arc<RwLock<protocol::PoolProtocol>>;

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

pub async fn ws_handler(ws: WebSocketUpgrade, State(pool_protocol): State<PoolProtocolRW>) {
    let ws = ws.on_upgrade(move |socket| async {
        let (mut tx, mut rx) = socket.split();
        while let Some(Ok(msg)) = rx.next().await {
            let mut pool_protocol = pool_protocol.write().unwrap();
            match msg {
                Message::Text(text) => {
                    trace!("Got a text message: {}", text);
                    let state = pool_protocol.get_state();
                    let state = SystemState {
                        system_version: 1,
                        application_version: 1,
                        switches: state.get_controls_state(),
                        temperatures: state.get_temperatures(),
                    };
                    let json = serde_json::to_string(&state).unwrap();
                    tx.send(Message::Text(json)).await.unwrap();
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
        while let Some(Ok(msg)) = rx.next().await {
            let mut pool_protocol = pool_protocol.write().unwrap();
            match msg {
                Message::Text(text) => {
                    trace!("Got a text message: {}", text);
                    let state = pool_protocol.get_state();
                    let state = SystemState {
                        system_version: 1,
                        application_version: 1,
                        switches: state.get_controls_state(),
                        temperatures: state.get_temperatures(),
                    };
                    let json = serde_json::to_string(&state).unwrap();
                    tx.send(Message::Text(json)).await.unwrap();
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
    });
}
