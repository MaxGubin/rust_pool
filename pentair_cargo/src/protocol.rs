
use serial::{self, SerialPort};
use crate::config;




pub fn serial_port(parameters: &config::config_json::PortParameters) -> serial::SystemPort {

    let port_name = &parameters.port_name;
     
    let settings = serial::PortSettings {
        baud_rate: serial::BaudRate::from_speed(parameters.baud_rate),
        char_size: config::config_json::decode_char_size(parameters.char_size),
        parity: config::config_json::decode_parity(parameters.parity.as_str()),
        stop_bits: config::config_json::decode_stop_bits(parameters.stop_bits),
        flow_control: serial::FlowNone,
    };
    
    
     let mut port = serial::open(port_name).unwrap();
     port.configure(&settings);
     port
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
