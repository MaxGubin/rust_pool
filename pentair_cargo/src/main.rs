use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{Request, Response, body::Bytes, service::service_fn};
use log::{info, trace, warn, error};
use std::path::Path;
use std::{convert::Infallible, net::SocketAddr, error::Error};
use tokio::net::TcpListener;

mod config;
mod protocol;

async fn hello(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Pentair Configuration!"))))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    trace!("Starting up");
    println!(
        "User's Name            whoami::realname():    {}",
        whoami::realname(),
    ); 
    println!(
        "User's Username        whoami::username():    {}",
        whoami::username(),
    );
    let config_path = std::env::args().nth(1).unwrap_or_else(|| String::from("config.json"));
    let config = config::read_configuration(Path::new(&config_path))?;
    trace!("Configuration loaded: {:?}", config);

    let _serial_port: serial::SystemPort  = protocol::serial_port(&config.port_parameters);
    trace!("Serial port opened");

    let addr: SocketAddr = config.comms.listen_address.parse().expect("Invalid listen address");



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