use crate::config;
use serial::{self, Error, SerialPort};
use std::io::Read;

pub fn serial_port(
    parameters: &config::config_json::PortParameters,
) -> Result<serial::SystemPort, serial::Error> {
    let port_name = &parameters.port_name;

    let settings = serial::PortSettings {
        baud_rate: serial::BaudRate::from_speed(parameters.baud_rate),
        char_size: config::config_json::decode_char_size(parameters.char_size),
        parity: config::config_json::decode_parity(parameters.parity.as_str()),
        stop_bits: config::config_json::decode_stop_bits(parameters.stop_bits),
        flow_control: serial::FlowNone,
    };

    let mut port = serial::open(port_name).unwrap();
    port.configure(&settings)?;
    Ok(port)
}

fn read_to_header(port: &mut serial::SystemPort) -> Result<(), serial::Error> {
    const HEADER: [u8; 4] = [0xFF, 0x00, 0xFF, 0xA5];
    let mut byte = [0; 1];
    let mut buffer = Vec::with_capacity(HEADER.len());
    loop {
        port.read(&mut byte[..])?;
        buffer.push(byte[0]);
        if buffer.len() == HEADER.len() {
            if buffer == HEADER {
                break;
            } else {
                buffer.remove(0);
            }
        }
    }
    Ok(())
}

fn read_packet(port: &mut serial::SystemPort) -> Result<Vec<u8>, serial::Error> {
    read_to_header(port)?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut byte: [u8; 1] = [0];
    for _ in 0..4 {
        port.read(&mut byte[..])?;
        buffer.push(byte[0]);
    }
    let to_read_len = buffer[4] as usize;
    for _ in 0..to_read_len {
        port.read(&mut byte[..])?;
        buffer.push(byte[0]);
    }

    // Checksum
    port.read(&mut byte[..])?;
    let mut checksum: u16 = 256 * (byte[0] as u16);
    port.read(&mut byte[..])?;
    checksum += byte[0] as u16;
    for b in buffer.iter() {
        checksum -= *b as u16;
    }
    if checksum != 0 {
        return Err(serial::Error::new(
            serial::ErrorKind::InvalidInput,
            "Checksum error",
        ));
    }

    Ok(buffer)
}

#[derive(Clone, Debug)]
pub struct SystemState {
    // Version of the system's configururation, if changed, the circuits can be renamed.
    version: u32,

    // Different switches, usually in the state on/off
    pool_on: bool,
    spa_on: bool,
    aux_circuits: Vec<bool>,
    feature_circuits: Vec<bool>,

    // Block of temperatures
    water_temp: u32,
    air_temp: u32,
    solar_temp: u32,
}

impl SystemState {
    fn new() -> SystemState {
        SystemState {
            version: 0,
            pool_on: false,
            spa_on: false,
            aux_circuits: Vec::new(),
            feature_circuits: Vec::new(),
            water_temp: 0,
            air_temp: 0,
            solar_temp: 0,
        }
    }
    fn from_packet(packet: Vec<u8>) -> Result<SystemState, serial::Error> {
        if packet.len() < 7 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Packet too short",
            ));
        }
        const PROTOCOL_OFFSET: usize = 0;
        if packet[PROTOCOL_OFFSET] != 0x00 || packet[PROTOCOL_OFFSET] != 0x01 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Invalid protocol",
            ));
        }

        const DEST_OFFSET: usize = 1;
        const SRC_OFFSET: usize = 3;
        if packet[DEST_OFFSET] != 0x0f || packet[SRC_OFFSET] != 0x10 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Invalid destination or source",
            ));
        }

        const CMD_OFFSET: usize = 3;
        const SYSTEM_STATUS_CMD: u8 = 0x02;
        if packet[CMD_OFFSET] != SYSTEM_STATUS_CMD {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Invalid command",
            ));
        }

        let mut state = Self::new();

        const MASK_IDX: usize = 7;
        const SPA_MASK: u8 = 0x01;
        const AUX1_MASK: u8 = 0x02;
        const AUX2_MASK: u8 = 0x04;
        const AUX3_MASK: u8 = 0x08;
        const POOL_MASK: u8 = 0x20;
        const FEATURE1_MASK: u8 = 0x10;
        const FEATURE2_MASK: u8 = 0x40;
        const FEATURE3_MASK: u8 = 0x80;

        {
            state.pool_on = (packet[MASK_IDX] & POOL_MASK) != 0;
            state.spa_on = (packet[MASK_IDX] & SPA_MASK) != 0;
            state.aux_circuits.push((packet[MASK_IDX] & AUX1_MASK) != 0);
            state.aux_circuits.push((packet[MASK_IDX] & AUX2_MASK) != 0);
            state.aux_circuits.push((packet[MASK_IDX] & AUX3_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE1_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE2_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE3_MASK) != 0);
        }

        Ok(state)
    }
    pub fn get_version(&self) -> u32 {
        self.version
    }

    // Checks the current state
    pub fn get_controls_state(&self) -> Vec<(String, bool)> {
        vec![
            ("pool".to_string(), self.pool_on),
            ("spa".to_string(), self.spa_on),
        ]
    }

    //
    pub fn get_temperatures(&self) -> Vec<(String, f32)> {
        vec![
            ("water".to_string(), 82.),
            ("air".to_string(), 72.),
            ("solar".to_string(), 83.),
        ]
    }
}

pub struct PoolProtocol {
    port: serial::SystemPort,
    // This is the only one thread that reads/writes the port.
    // communication_thread: std::thread::JoinHandle,
    system_state: SystemState,

    // The version of the system
    version: u32,
}

impl PoolProtocol {
    pub fn new(port: serial::SystemPort) -> PoolProtocol {
        PoolProtocol {
            port,
            system_state: SystemState::new(),
            version: 0,
        }
    }

    /// Returns the current state of the system.
    pub fn get_state(&self) -> SystemState {
        self.system_state.clone()
    }

    // Changes a state of a control. Returns back True if it was changed, false if it was not
    // changed yet.
    pub fn change_circuit(&mut self, control_name: &str, state: bool) -> bool {
        if control_name == "pool" {
            self.system_state.pool_on = state;
        }
        if control_name == "spa" {
            self.system_state.spa_on = state;
        }
        true
    }
}

#[cfg(test)]
#[test]
fn test_system_state_from_packet() {
    let packet = vec![
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x53, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5F, 0x00, 0x00, 0x00,
        0x3C, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0xf4, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // \\x00\\xf6\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x02\\x00\\x02\\x00\\xf7\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x06\\x01\\n\\x00\\xf8\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xf9\\x01\\x00\\x00\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfa\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfb\\x01\\x00\\x00\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfc\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfe\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xff\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xff\\x02\\x00\\x00\\x1f\\x03\\x00\\x00\\xff\\xff\\xff\\xff\\x00\\x00\\x00\\x00\\x06\\x00\\x00\\x00\\x07\\x00\\x00\\x00\\x00\\x00\\x00\\x00'"

    let state = SystemState::from_packet(packet).unwrap();
    assert!(state.pool_on);
    assert!(!state.spa_on);
    assert!(state.aux_circuits[0]);
    assert!(!state.aux_circuits[1]);
    assert!(state.aux_circuits[2]);
    assert!(!state.feature_circuits[0]);
    assert!(!state.feature_circuits[1]);
    assert!(!state.feature_circuits[2]);
}
