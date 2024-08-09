
use serial::{self, SerialPort};
use crate::config;




pub fn serial_port(parameters: &config::config_json::PortParameters) -> Result<serial::SystemPort, serial::Error> {

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
     port
}

fn read_to_header(port: &mut serial::SystemPort)_{
    const HEADER: [u8; 4] = [0xFF, 0x00, 0xFF, 0xA5];
    let mut byte: u8 = 0;
    let mut buffer = Vec::with_capacity(HEADER.len());
    let buf_offset = 0;
    loop {
        port.read(&mut byte)?;
        buffer.push(byte);
        if buffer.len() == HEADER.len() {
            if buffer == HEADER {
                break;
            } else {
                buffer.remove(0);
            }
        }

    }
}

fn read_packet(port: &mut serial::SystemPort) -> Vec<u8> {
    read_to_header(port);
    let mut buffer: Vec<u8> = Vec::new();
    buffer.push(0xA5);
    for _ in 0..4 {
        port.read(&mut byte)?;
        buffer.push(byte);
    }
    let to_read_len  = buffer[4] as usize;
    for _ in 0..to_read_len {
        port.read(&mut byte)?;
        buffer.push(byte);
    }

    // Checksum
    port.read(&mut byte)?;
    let mut checksum = 256*(byte as usize);
    port.read(&mut byte)?;
    checksum += byte as u32;
    for b in buffer.iter() {
        checksum -= b as u32;
    }
    if checksum != 0 {
        panic!("Checksum error");
    }

    buffer
}


pub struct SystemState {

    // True if the packet was read successfully
    pub valid: bool,

    // 
    pub last_error: String,
    pub pool_on: bool,
    pub spa_on: bool,
    pub feature_circuits: Vec<bool>,


    // Block of temperatures
    pub water_temp: u32,
    pub air_temp: u32,
    pub solar_temp: u32,

}

impl SystemState {
    pub fn new() -> SystemState {
        SystemState {
            valid: false,
            last_error: String::new(),
            pool_on: false,
            spa_on: false,
            feature_circuits: Vec::new(),
            water_temp: 0,
            air_temp: 0,
            solar_temp: 0,
        }
    }
    pub fn from_error(err: serial::Error) -> SystemState {
        SystemState {
            valid: false,
            last_error: err.to_string(),
            pool_on: false,
            spa_on: false,
            feature_circuits: Vec::new(),
            water_temp: 0,
            air_temp: 0,
            solar_temp: 0,
        }
    }
    pub fn from_packet(packet: Vec<u8>) -> SystemState {
        
        if packet.len() < 7 {
            from_error(serial::Error::new(serial::ErrorKind::InvalidData, "Packet too short"))
        }
        const PROTOCOL_OFFSET:usize = 1;
        if packet[PROTOCOL_OFFSET] != 0x00 || packet[PROTOCOL_OFFSET] != 0x01 {
            from_error(serial::Error::new(serial::ErrorKind::InvalidData, "Invalid protocol"))
        }

        const DEST_OFFSET:usize = 2;
        const SRC_OFFSET:usize = 3;
        if packet[DEST_OFFSET] != 0x0f || packet[SRC_OFFSET] != 0x10 {
            from_error(serial::Error::new(serial::ErrorKind::InvalidData, "Invalid destination or source"))
        }

        const CMD_OFFSET:usize = 4;
        const SYSTEM_STATUS_CMD: usize = 0x02;
        if packet[CMD_OFFSET] != SYSTEM_STATUS_CMD {
            from_error(serial::Error::new(serial::ErrorKind::InvalidData, "Invalid command"))
        }

        let mut state = new();

        const MASK_IDX: usize = 8;
        const SPA_MASK: u8 = 0x01;
        const AUX1_MASK: u8 = 0x02;
        const AUX2_MASK: u8 = 0x04;
        const AUX3_MASK: u8 = 0x08;
        const POOL_MASK: u8 = 0x20;
        const FEATURE1_MASK: u8 = 0x10;
        const FEATURE2_MASK: u8 = 0x40;
        const FEATURE3_MASK: u8 = 0x80;

        {
            state.pool_on = packet[MASK_IDX] & POOL_MASK;
            state.spa_on = packet[MASK_IDX] & SPA_MASK;
            

        }

        state
    }
}

pub fn get_status(port: &mut serial::SystemPort) -> Vec<u8> {
    match read_packet(port) {
        Ok(packet) => {
            let mut state = SystemState {
                last_error: String::new(),
                pool_on: false,
                spa_on: false,
                feature_circuits: Vec::new(),
                water_temp: 0,
                air_temp: 0,
                solar_temp: 0,
            };
            let mut i = 5;
            state.pool_on = packet[i] & 0x01;
            state.spa_on = packet[i] & 0x02;
            i += 1;
            for _ in 0..8 {
                state.feature_circuits.push(packet[i] & 0x01);
                i += 1;
            }
            state.water_temp = packet[i] as u32;
            i += 1;
            state.air_temp = packet[i] as u32;
            i += 1;
            state.solar_temp = packet[i] as u32;
            i += 1;
            state
        },
        Err(e) => {
            SystemState::from_error(e)
        }
    }

    let mut state = SystemState {
        last_error: String::new(),
        pool_on: false,
        spa_on: false,
        feature_circuits: Vec::new(),
        water_temp: 0,
        air_temp: 0,
        solar_temp: 0,
    };
    let packet = read_packet(port);


}
enum PentairDevice {
    Controller,
    Pump,
    Heater,
    Solar,
}

/* 
enum MessageCode {
    MSG_CODE_1 = 0,
    ERROR_LOGIN_REJECTED = 13,
    CHALLENGE_QUERY = 14,
    PING_QUERY = 16,
    LOCALLOGIN_QUERY = 27,
    ERROR_INVALID_REQUEST = 30,
    ERROR_BAD_PARAMETER = 31,  // Actually bad parameter?
    FIRMWARE_QUERY = 8058,
    GET_DATETIME_QUERY = 8110,
    SET_DATETIME_QUERY = 8112,
    VERSION_QUERY = 8120,
    WEATHER_FORECAST_CHANGED = 9806,
    WEATHER_FORECAST_QUERY = 9807,
    STATUS_CHANGED = 12500,
    COLOR_UPDATE = 12504,
    CHEMISTRY_CHANGED = 12505,
    ADD_CLIENT_QUERY = 12522,
    REMOVE_CLIENT_QUERY = 12524,
    POOLSTATUS_QUERY = 12526,
    SETHEATTEMP_QUERY = 12528,
    BUTTONPRESS_QUERY = 12530,
    CTRLCONFIG_QUERY = 12532,
    SETHEATMODE_QUERY = 12538,
    LIGHTCOMMAND_QUERY = 12556,
    SETCHEMDATA_QUERY = 12594,
    EQUIPMENT_QUERY = 12566,
    SCGCONFIG_QUERY = 12572,
    SETSCG_QUERY = 12576,
    PUMPSTATUS_QUERY = 12584,
    SETCOOLTEMP_QUERY = 12590,
    CHEMISTRY_QUERY = 12592,
    GATEWAYDATA_QUERY = 18003
}
*/

struct PentairMessage {
    destination: u8,
    source: u8,
    command: u8,
    data: u8,
    checksum: u8,
}
