use axum::routing::{any, get, post, Router};
use clap::Parser;
use log::{error, info, trace};
use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::RwLock;
use std::thread;
use tower_http::services::ServeDir;

// A thread/

mod config;
mod pool;
mod ui;

// Command line arguments
#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    #[arg(short, long, default_value = "5")]
    verbosity: u8,

    #[arg(short, long, default_value = "true")]
    logtostderr: bool,
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

    let pool_protocol = pool::PoolProtocolRW::new(RwLock::new(pool::protocol::PoolProtocol::new()));
    let port =
        pool::protocol::serial_port(&config.port_parameters).expect("Failed to open serial port");
    {
        let p1 = pool_protocol.clone();
        thread::spawn(move || pool::serial::port_read_thread(port, p1));
    }

    trace!("Serial port opened");
    match run_server(&config.comms, pool_protocol) {
        Ok(()) => info!("Successfully stopping"),
        Err(e) => error!("Failed {}", e),
    }
}

#[tokio::main]
pub async fn run_server(
    config: &config::config_json::Comms,
    pool_protocol: pool::PoolProtocolRW,
) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", get(ui::serve_status))
        .route("/control", post(ui::control_command))
        .route("/state", get(ui::state_json))
        .route("/log", get(ui::log_json))
        .route("/ws", any(ui::ws_handler))
        .with_state(pool_protocol)
        .nest_service("/assets", ServeDir::new("assets"));
    if config.https_listen_address.is_some() {
        if config.cert_path.is_none() || config.key_path.is_none() {
            panic!("Missing cert_path or key_path");
        }
        let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            config.cert_path.as_ref().unwrap(),
            config.key_path.as_ref().unwrap(),
        )
        .await?;
        let addr = config
            .https_listen_address
            .as_ref()
            .unwrap()
            .parse()
            .expect("Invalid https address");
        axum_server::tls_rustls::bind_rustls(addr, rustls_config)
            .serve(app.into_make_service())
            .await
    } else {
        if config.http_listen_address.is_none() {
            panic!("Missing both http_listen_address");
        }
        axum_server::bind(
            config
                .http_listen_address
                .as_ref()
                .unwrap()
                .parse()
                .expect("Invalid http address"),
        )
        .serve(app.into_make_service())
        .await
    }
}
