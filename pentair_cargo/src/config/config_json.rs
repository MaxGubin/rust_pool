
use serde::{Deserialize, Serialize};

// Controller parameters
pub struct Controller {
    
}

fn default_enabled() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Comms {
    pub port: u32,
    #[serde(default="default_enabled")]
    pub enabled: bool,
    
}
pub struct Config {
    pub comms: Comms,
}