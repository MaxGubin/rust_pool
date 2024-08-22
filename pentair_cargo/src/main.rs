use clap::Parser;
use log::{error, info, trace};
use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{convert::Infallible, error::Error, net::SocketAddr};
use axum::{
    http::StatusCode, routing::{get, Router},
    response::{Html, IntoResponse},
    extract::{Form, State, Path},

};
use tower_http::services::ServeDir;
use serde::Deserialize;


// A thread/
type PoolProtocolRW = Arc<RwLock<protocol::PoolProtocol>>;

mod config;
mod protocol;

// Command line arguments
#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    ///
    #[arg(short, long, default_value = "5")]
    verbosity: u8,

    #[arg(short, long, default_value = "true")]
    logtostderr: bool,

}

// The result structure from the form.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ControlInput {
    control_name: String,
    state: String,
}


async fn update_command(Form(ControlInput): Form<ControlInput>) {
    trace!("Got client input {:?}", ControlInput);
}

async fn serve_status(
    State(pool_protocol): State<PoolProtocolRW>
) -> Html<&'static str> {
    trace!("Calling status state request");
    // Read the current state
    let _pool_state = pool_protocol.write().unwrap().get_status();
    Html(
        r#"
        <!doctype html>
        <html>
            <head>
            <link href="/assets/style.css" rel="stylesheet" type="text/css">
            </head>
            <body>
                    <button type="submit" class="button"> Pool </button>
                    <button type="submit" class="buttonon"> Spa </button>
            </body>
        </html>
        "#,
    )
}

fn init_logging(verbosity: u8, logtostderr: bool) {
    let log_level = match verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![WriteLogger::new(
        log_level,
        Config::default(),
        File::create("pool.log").unwrap(),
    )];
    if logtostderr {
        loggers.push(SimpleLogger::new(log_level, Config::default()));
    }
    CombinedLogger::init(loggers).unwrap();
}

fn main() {
    let args = Cli::parse();
    init_logging(args.verbosity, args.logtostderr);
    trace!("Starting up");
    info!(
        "User's Name            whoami::realname():    {}",
        whoami::realname(),
    );
    info!(
        "User's Username        whoami::username():    {}",
        whoami::username(),
    );
    let config = config::read_configuration(&args.config).expect("Failed to read configuration");
    trace!("Configuration loaded: {:?}", config);

    let pool_protocol = PoolProtocolRW::new(RwLock::new(protocol::PoolProtocol::new(
        protocol::serial_port(&config.port_parameters).expect("Failed to open serial port"),
    )));
    trace!("Serial port opened");
    match run_server(&config.comms.listen_address, pool_protocol) {
        Ok(()) => info!("Successfully stopping"),
        Err(e) => error!("Failed {}", e)
    }
}

#[tokio::main]
pub async fn run_server(
    address: &String,
    pool_protocol: PoolProtocolRW)->Result<(), std::io::Error> {
    let addr: SocketAddr = address.parse().expect("Invalid listen address");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Opened address {:?} for listening", addr);


    let app = Router::new()
    .route("/", get(serve_status).post(update_command))
    .with_state(pool_protocol)
    .nest_service("/assets", ServeDir::new("assets"));

    axum::serve(listener, app).await

}
