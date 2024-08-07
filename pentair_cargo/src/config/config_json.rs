use serde::{Deserialize, Serialize};
use serial::{BaudRate, CharSize, Parity, StopBits};

// Controller parameters
pub struct Controller {}

fn default_enabled() -> bool {
    true
}
fn default_baud_rate() -> u32 {
    9600
}
fn default_char_size() -> u32 {
    8
}
fn default_parity() -> String {
    "None".to_string()
}
fn default_stop_bits() -> u32 {
    1
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comms {
    pub listen_address: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PortParameters {
    pub port_name: String,

    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    #[serde(default = "default_char_size")]
    pub char_size: u32,
    #[serde(default = "default_parity")]
    pub parity: String,
    #[serde(default = "default_stop_bits ")]
    pub stop_bits: u32,
    pub timeout_msec: u32,
}
