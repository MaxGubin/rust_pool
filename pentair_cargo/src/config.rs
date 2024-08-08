// Configuration file handling, defining structures  that are read from JSON for compatibility with nodejs-poolController
use std::fs;
use std::io;
use std::path;

use serde::{Deserialize, Serialize};
use serde_json;

pub mod config_json;
mod controller;


/// Config constants

// The root configuration structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct PoolConfig{
   pub comms: config_json::Comms, 
   pub port_parameters: config_json::PortParameters,
}

pub fn read_configuration(config_path: &path::Path) -> io::Result<PoolConfig> {
    let config_str = fs::read_to_string(config_path)?;
    let config: PoolConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_configuration() {
        let config = read_configuration(path::Path::new("config.json")).unwrap();
        assert_eq!(config.comms.listen_address, "127.0.0.1:3000");
    }
}
