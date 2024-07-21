
use serialport::prelude::*;


pub fn serial_port() -> serial::SystemPort {

    let port_name = "/dev/ttyUSB0";
    let settings = serial::PortSettings {
        baud_rate: serial::Baud9600,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };
    
    match serial::open_with_settings(port_name, &settings) {
        Ok(port) => port,
        Err(e) => eprintln!("{:?}", e),
    }
    port
}


pub fn serial_write(port: serial::SystemPort, data: &[u8]) -> Result<(), serial::Error> {
    port.write(data)
}

enum PentairDevice {
    Controller,
    Pump,
    Heater,
    Solar,
}

enum MessageCode {
    MSG_CODE_1 = 0,
    ERROR_LOGIN_REJECTED = 13,
    CHALLENGE_QUERY = 14,
    PING_QUERY = 16,
    LOCALLOGIN_QUERY = 27,
    ERROR_INVALID_REQUEST = 30
    ERROR_BAD_PARAMETER = 31  # Actually bad parameter?
    FIRMWARE_QUERY = 8058
    GET_DATETIME_QUERY = 8110
    SET_DATETIME_QUERY = 8112
    VERSION_QUERY = 8120
    WEATHER_FORECAST_CHANGED = 9806
    WEATHER_FORECAST_QUERY = 9807
    STATUS_CHANGED = 12500
    COLOR_UPDATE = 12504
    CHEMISTRY_CHANGED = 12505
    ADD_CLIENT_QUERY = 12522
    REMOVE_CLIENT_QUERY = 12524
    POOLSTATUS_QUERY = 12526
    SETHEATTEMP_QUERY = 12528
    BUTTONPRESS_QUERY = 12530
    CTRLCONFIG_QUERY = 12532
    SETHEATMODE_QUERY = 12538
    LIGHTCOMMAND_QUERY = 12556
    SETCHEMDATA_QUERY = 12594
    EQUIPMENT_QUERY = 12566
    SCGCONFIG_QUERY = 12572
    SETSCG_QUERY = 12576
    PUMPSTATUS_QUERY = 12584
    SETCOOLTEMP_QUERY = 12590
    CHEMISTRY_QUERY = 12592
    GATEWAYDATA_QUERY = 18003
}

struct PentairMessage {
    destination: u8,
    source: u8,
    command: u8,
    data: u8,
    checksum: u8,
}

pub fn decode_serial(data: &[u8]) -> Result<(), serial::Error> {
    const preambl: [u8;4] = {0x00, 0xFF, 0xA5};
    if data[0..3] != preamblu {
        return Err(serial::Error::new(serial::ErrorKind::InvalidInput, "Invalid preamble"));
    }
    if data[4] != 0x00 || data[4] != 0x01{
        return Err(serial::Error::new(serial::ErrorKind::InvalidInput, "Invalid version"));
    }
    let destination = decode_device(data[5]);

}

pub fn serial_read(port: serial::SystemPort, buf: &mut [u8]) -> Result<usize, serial::Error> {
    let mut serial_buf: Vec<u8> = vec![0; 1000];
    match port.read(serial_buf) {
        Ok(count) => Ok(count),
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),   
    }
}
