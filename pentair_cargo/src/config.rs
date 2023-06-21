// Configuration file handling, defining structures  that are read from JSON for compatibility with nodejs-poolController
use io;
use std::path;

use serde::{Deserialize, Serialize};


mod controller;

// The root configuration structure.
pub struct PoolConfig{
    
}

pub fn read_configuration(config_path: path::Path) -> io::Result<PoolConfig> {
    let config_str = io::read_file(config_path)?;
    let config: PoolConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}