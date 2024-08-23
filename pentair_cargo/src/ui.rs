// Interface implementation

use log::{error, info, trace};
use serde::Deserialize;
use std::sync::{Arc, RwLock};

use axum::{
    http::StatusCode, routing::{get,post, Router},
    response::{Html, IntoResponse},
    extract::{Json, State, Path},

};
use askama::Template;
use crate::protocol;

pub type PoolProtocolRW = Arc<RwLock<protocol::PoolProtocol>>;

// The result structure from the form.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ControlInput {
    control_name: String,
    state: String,
}


pub async fn control_command(State(pool_protocol): State<PoolProtocolRW>, Json(control_input): Json<ControlInput>) {
    trace!("Got client input {:?}", control_input);

    let mut pool_protocol = pool_protocol.write().unwrap();
    let state = if control_input.state == "on" {true} else {false};
    pool_protocol.change_state(&control_input.control_name, state);
}


#[derive(Template)]
#[template(path="index.html")]
struct UITemplate<'a> {
    pub controls: &'a Vec<(String, bool)>,
    pub temperatures: &'a Vec<(String, f32)>,
}

pub async fn serve_status(
    State(pool_protocol): State<PoolProtocolRW>
) -> impl IntoResponse {
    trace!("Calling status state request");
    // Read the current state
    let pool_protocol = pool_protocol.read().unwrap();
    let template = UITemplate{controls: &pool_protocol.get_controls_state(),
    temperatures: &pool_protocol.get_temperatures()};
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Template processing error {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render a template. Error {}", err)
            ).into_response()
        },
    }
}
