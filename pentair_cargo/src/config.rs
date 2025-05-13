// Configuration file handling, defining structures  that are read from JSON for compatibility with nodejs-poolController
use std::fs;
use std::io;
use std::path;

use serde::{Deserialize, Serialize};

pub mod config_json;
pub mod mobile_app;

/// Config constants

// The root configuration structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct PoolConfig {
    pub comms: config_json::Comms,
    pub port_parameters: config_json::PortParameters,
    pub system_parameters: config_json::SystemParameters,
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
        assert_eq!(
            config.comms.http_listen_address,
            Some("127.0.0.1:3000".to_string())
        );
    }
}
