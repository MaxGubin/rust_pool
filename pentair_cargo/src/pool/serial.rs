use crate::pool::PoolProtocolRW;
use log::{debug, error, trace, warn};
use serial::{self, Error, SerialPort};
use std::io::Read;

fn scan_for_header(port: &mut serial::SystemPort) -> Result<(), serial::Error> {
    trace!("Waiting for a header from the port");
    const HEADER: [u8; 4] = [0xFF, 0x00, 0xFF, 0xA5];
    let mut byte = [0; 1];
    let mut buffer = Vec::with_capacity(HEADER.len());
    loop {
        port.read_exact(&mut byte[..])?;
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
    }
    Ok(())
}
fn read_packet(port: &mut serial::SystemPort) -> Result<Vec<u8>, serial::Error> {
    scan_for_header(port)?;
    const USUAL_PACKET_SIZE: usize = 32;
    let mut buffer: Vec<u8> = Vec::with_capacity(USUAL_PACKET_SIZE);
    let mut byte: [u8; 1] = [0];
    for _ in 0..4 {
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
    for b in buffer.iter() {
        checksum -= *b as u16;
    }
    if checksum != 0 {
        //self.corrupted_packets.fetch_add(1, Ordering::Relaxed);
        return Err(serial::Error::new(
            serial::ErrorKind::InvalidInput,
            "Checksum error",
        ));
    }

    Ok(buffer)
}

pub fn port_read_thread(mut port: serial::SystemPort, pool_protocol: PoolProtocolRW) {
    trace!("Pool monitor thread started");
    loop {
        match read_packet(&mut port) {
            Ok(packet) => {
                trace!("Received a correct packet");
                let mut pool = pool_protocol.write().unwrap();
                pool.log_packet(&packet);
                pool.process_packet(&packet);
            }
            Err(e) => {
                error!("Failed to read packet: {}", e);
            }
        }
        //
    }
}
