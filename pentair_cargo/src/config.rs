// Configuration file handling, defining structures  that are read from JSON for compatibility with nodejs-poolController
use std::fs;
use std::io;
use std::path;

use serde::{Deserialize, Serialize};
use serde_json;

mod config_json;
mod controller;


/// Config constants

// The root configuration structure.
#[derive(Serialize, Deserialize)]
pub struct PoolConfig{
   comms: config_json::Comms, 
}

pub fn read_configuration(config_path: &path::Path) -> io::Result<PoolConfig> {
    let config_str = fs::read_to_string(config_path)?;
    let config: PoolConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}