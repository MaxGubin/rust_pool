use reqwest;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::str::FromStr;

pub async fn get_external_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    const KEY_NAME: &str = "origin";
    match resp.get(KEY_NAME) {
        Some(value) => Ok(IpAddr::from_str(value)?),
        None => Err(Box::new(Error::new(
            ErrorKind::InvalidInput,
            "Input string cannot be empty",
        ))),
    }
}
