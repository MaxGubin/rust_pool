use crate::pool::PoolProtocolRW;
use log::{debug, error, trace};
use serial::{self};
use std::io::{ErrorKind, Read, Write, BufWriter};


/// Creates a serial port from the  configuration.
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

    let mut port = serial::open(port_name)?;
    port.configure(&settings)?;
    Ok(port)
}


/// Status from processing serial input.
#[derive(PartialEq)]
enum HeaderScan {
    BusAvailable,
    GoodHeader,
}

fn scan_for_header(port: &mut serial::SystemPort) -> Result<HeaderScan, serial::Error> {
    const HEADER: [u8; 4] = [0xFF, 0x00, 0xFF, 0xA5];
    let mut byte = [0; 1];
    let mut buffer = Vec::with_capacity(HEADER.len());
    if let Err(e) = port.read_exact(&mut byte[..]) {
        return if e.kind() == ErrorKind::TimedOut {
            Ok(HeaderScan::BusAvailable)
        } else {
            Err(e.into())
        };
    }
    trace!("Reading a header from the port");
    loop {
        debug!("Read byte {:?} from the port", byte[0]);
        buffer.push(byte[0]);
        if buffer.len() == HEADER.len() {
            if buffer == HEADER {
                break;
            } else {
                buffer.remove(0);
                //self.unrecognized_bytes.fetch_add(1, Ordering::Relaxed);
            }
        }
        port.read_exact(&mut byte[..])?;
    }
    Ok(HeaderScan::GoodHeader)
}


fn read_packet(port: &mut serial::SystemPort) -> Result<Vec<u8>, serial::Error> {
    const USUAL_PACKET_SIZE: usize = 32;
    let mut buffer: Vec<u8> = Vec::with_capacity(USUAL_PACKET_SIZE);
    let mut byte: [u8; 1] = [0];
    for _ in 0..5 {
        port.read_exact(&mut byte[..])?;
        buffer.push(byte[0]);
    }
    let to_read_len = buffer[4] as usize;
    for _ in 0..to_read_len {
        port.read_exact(&mut byte[..])?;
        buffer.push(byte[0]);
    }

    // Checksum
    port.read_exact(&mut byte[..])?;
    let mut checksum: u16 = 256 * (byte[0] as u16);
    port.read_exact(&mut byte[..])?;
    checksum += byte[0] as u16;
    checksum -= 0xa5; // Because it has the last byte of the header
    debug!("Read checksum {}", checksum);
    for b in buffer.iter() {
        checksum -= *b as u16;
    }
    debug!("Rest of checksum {}", checksum);
    if checksum != 0 {
        //self.corrupted_packets.fetch_add(1, Ordering::Relaxed);
        return Err(serial::Error::new(
            serial::ErrorKind::InvalidInput,
            "Checksum error",
        ));
    }

    Ok(buffer)
}


struct PacketLogger {
    log_file: Option<String>,
    writer: Option<BufWriter<std::fs::File>>,
}

impl PacketLogger {
    fn new(log_file: Option<String>) -> Self {
        let writer = log_file.as_ref().map(|file_path| {
            let file = File::create(file_path).expect("Failed to open log file");
            BufWriter::new(file)
        });
        PacketLogger { log_file, writer }
    }
    fn log_message(&self, message: message::ProtocolPacket) -> Result<(), std::io::Error> {
        if let Some(writer) = &self.writer {
            let message_str = format!("{:?}\n", message);
            writer.write_all(message_str.as_bytes())?;
            writer.flush()?;
        }
        Ok(())
    }
}



pub fn port_read_thread(mut port: serial::SystemPort, pool_protocol: PoolProtocolRW, log_file: Option<String> ) {
    trace!("Pool monitor thread started");

    let message_logger = PacketLogger::new(log_file); 
    loop {
        match scan_for_header(&mut port) {
            Ok(r) => {
                if r == HeaderScan::BusAvailable {
                    // TODO: process waiting messages;
                    continue;
                }
            }
            Err(e) => {
                error!("Failed waiting for a header: {}", e)
            }
        }
        match read_packet(&mut port) {
            Ok(packet) => {
                trace!("Received a correct packet");
                let mut pool = pool_protocol.write().unwrap();
                pool.log_packet(&packet);
                
                match pool.process_packet(&packet) {
                    Ok(message) => {
                        trace!("Received a message: {:?}", message);
                        pool.log_message(message);
                    }
                    Err(e) => {
                        error!("Failed to process packet: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read packet: {}", e);
            }
        }
        //
    }
}
