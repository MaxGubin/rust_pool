
use serial::Error;
pub mod system_state;


#[derive(Clone, Debug)]
pub enum PacketType {
    Status(system_state::SystemState),
    CircuitStatusChange, // We'd ignore it.
    CircuitStatusResponse,
    RemoteLayoutRequest,
    RemoteLayoutResponse,
    ClockBroadcast,
    PumpStatus,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ProtocolPacket{
    packet_content: Vec<u8>,  
    pub decoded: PacketType
}

// Block protocol offsets in the packet without any header.

const PROTOCOL_OFFSET: usize = 0;
const DEST_OFFSET: usize = 1;
const SRC_OFFSET: usize = 2;
const CMD_OFFSET: usize = 3;
impl ProtocolPacket {
    pub fn new(packet: &[u8]) -> ProtocolPacket {
        ProtocolPacket {
            packet_content: packet.to_vec(),
            decoded: PacketType::Unknown,
        }
    }
    pub fn get_source(&self) -> u8 {
        if self.packet_content.len() < SRC_OFFSET {
            return 0;
        }
        self.packet_content[SRC_OFFSET]
    }
    pub fn get_destination(&self) -> u8 {
        if self.packet_content.len() < DEST_OFFSET {
            return 0;
        }
        self.packet_content[DEST_OFFSET]
    }

    pub fn get_protocol_version(&self) -> u8 {
        if self.packet_content.len() < 1 {
            return 0;
        }
        self.packet_content[PROTOCOL_OFFSET]
    }

    pub fn decode_packet(packet: &[u8] ) -> Result<ProtocolPacket, serial::Error> {
        if packet.len() < 4 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Packet is too short (decode_packet)",
            ));
        }
        if packet[PROTOCOL_OFFSET] != 0x00 && packet[PROTOCOL_OFFSET] != 0x01 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Invalid protocol version",
            ));
        }
        Ok(ProtocolPacket{
            packet_content: packet.to_vec(),
            decoded: match packet[CMD_OFFSET] {
                0x02 if packet[SRC_OFFSET] == 0x10 && packet[DEST_OFFSET] == 0x0f => PacketType::Status(system_state::SystemState::from_packet(packet)?),
                0x86 => PacketType::CircuitStatusChange,
                0x01 => PacketType::CircuitStatusResponse,
                0xE1 => PacketType::RemoteLayoutRequest,
                0x21 => PacketType::RemoteLayoutResponse,
                0x05 => PacketType::ClockBroadcast,
                0x07 => PacketType::PumpStatus,
                _ => PacketType::Unknown,
            }
        })
    }

}
