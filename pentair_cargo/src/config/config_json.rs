use serde::{Deserialize, Serialize};

// Controller parameters

fn default_enabled() -> bool {
    true
}
fn default_baud_rate() -> usize {
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

fn default_timeout_msec() -> u32 {
    1000
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comms {
    pub listen_address: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

pub fn decode_char_size(char_size: u32) -> serial::CharSize {
    match char_size {
        5 => serial::CharSize::Bits5,
        6 => serial::CharSize::Bits6,
        7 => serial::CharSize::Bits7,
        8 => serial::CharSize::Bits8,
        _ => panic!("Invalid char size"),
    }
}


pub fn decode_parity(parity: &str) -> serial::Parity {
    match parity {
        "None" => serial::Parity::ParityNone,
        "Odd" => serial::Parity::ParityOdd,
        "Even" => serial::Parity::ParityEven,
        _ => panic!("Invalid parity"),
    }
}


pub fn decode_stop_bits(stop_bits: u32) -> serial::StopBits {
    match stop_bits {
        1 => serial::StopBits::Stop1,
        2 => serial::StopBits::Stop2,
        _ => panic!("Invalid stop bits"),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PortParameters {
    pub port_name: String,

    #[serde(default = "default_baud_rate")]
    pub baud_rate: usize,
    #[serde(default = "default_char_size")]
    pub char_size: u32,
    #[serde(default = "default_parity")]
    pub parity: String,
    #[serde(default = "default_stop_bits ")]
    pub stop_bits: u32,
    #[serde(default="default_timeout_msec")]
    pub timeout_msec: u32,
}
