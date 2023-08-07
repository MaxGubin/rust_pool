iextern crate serial;

pub fn serial_port() -> serial::SystemPort {
    let port = serial::open("/dev/ttyUSB0").unwrap();
    port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::Baud9600));
        try!(settings.set_char_size(serial::Bits8));
        try!(settings.set_parity(serial::ParityNone));
        try!(settings.set_stop_bits(serial::Stop1));
        try!(settings.set_flow_control(serial::FlowNone));
        Ok(())
    })?;
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
