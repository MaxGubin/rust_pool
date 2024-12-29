// A simple application to send debug sequence to the port.

use clap::Parser;
use serial::{self, SerialPort};
use std::io::Write;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "/dev/ttyUSB1")]
    port: String,

    #[arg(long, default_value = "4142")]
    hex_string: String,
}

fn parse_hex_string(in_str: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    let mut bytes = Vec::with_capacity(in_str.len() / 2);
    for i in (0..in_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&in_str[i..i + 2], 16)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn send_to_port(port_name: &str, bytes: &[u8]) {
    let settings = serial::PortSettings {
        baud_rate: serial::BaudRate::from_speed(9600),
        char_size: serial::CharSize::Bits8,
        parity: serial::Parity::ParityNone,
        stop_bits: serial::StopBits::Stop1,
        flow_control: serial::FlowNone,
    };

    let mut port = serial::open(port_name).unwrap();
    port.configure(&settings).unwrap();
    port.write_all(bytes).unwrap();
}

fn main() {
    let args = Cli::parse();
    match parse_hex_string(args.hex_string.as_str()) {
        Ok(bytes) => send_to_port(args.port.as_str(), &bytes),
        Err(e) => println!("Error {} parsing {}", e, args.hex_string),
    }
}
