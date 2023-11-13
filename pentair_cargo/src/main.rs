use std::{convert::Infallible, net::SocketAddr, error::Error};
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes, service::service_fn};
use hyper::server::conn::http1;
use tokio::net::TcpListener;
use std::path::Path;

mod config;

async fn hello(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Pentair Configuration!"))))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_path = std::env::args().nth(1).unwrap_or_else(|| String::from("config.json"));
    let _config = config::read_configuration(Path::new(&config_path))?;
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}