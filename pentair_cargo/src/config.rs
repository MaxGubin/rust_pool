// Configuration file handling, defining structures  that are read from JSON for compatibility with nodejs-poolController
use io;
use std::path;

mod controller;

// The root configuration structure.
pub struct PoolConfig{
    
}

pub fn read_configuration(config_path: path::Path) -> io::Result<PoolConfig> {

}