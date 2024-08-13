use clap::Parser;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Request, Response};
use log::{error, info, trace};
use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::{convert::Infallible, env, error::Error, net::SocketAddr};
use tokio::net::TcpListener;

mod config;
mod protocol;

// Default values for input arguments.
fn default_config() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("config.json");
    path
}

// Command line arguments
#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "default_config()")]
    config: PathBuf,

    ///
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,

    #[arg(short, long)]
    logtostderr: bool,
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from(
        "Pentair Configuration!",
    ))))
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

    let _serial_port: serial::SystemPort =
        protocol::serial_port(&config.port_parameters).expect("Failed to open serial port");
    trace!("Serial port opened");

    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server(&config.comms.listen_address) {
        error!("FAILED: {}", e);
        std::process::exit(1);
    }
}

#[tokio::main]
pub async fn run_server(address: &String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr: SocketAddr = address.parse().expect("Invalid listen address");

    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(hello))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}
