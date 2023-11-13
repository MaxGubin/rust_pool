
use serde::{Deserialize, Serialize};

// Controller parameters
pub struct Controller {
    
}
#[derive(Serialize, Deserialize)]
pub struct Comms {
    pub message_type: String,
    pub port: u32,
    pub enabled: bool,
    
}
pub struct Config {
    pub comms: Comms,
}