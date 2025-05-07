
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
    decoded: PacketType
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

    pub fn decode_packet(&self) -> Result<ProtocolPacket, serial::Error> {
        let mut output = self.clone();
        output.decoded = PacketType::Unknown;
        if output.packet_content.len() < 4 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Packet is too short (decode_packet)",
            ));
        }
        if self.get_protocol_version() != 0x00 && self.get_protocol_version() != 0x01 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Invalid protocol version",
            ));
        }
        output.decoded = match(self.packet_content[CMD_OFFSET]) {
            0x02 if self.get_source() == 0x10 && self.get_destination() == 0x0f => PacketType::Status(system_state::SystemState::from_packet(&self.packet_content)?),
            0x86 => PacketType::CircuitStatusChange,
            0x01 => PacketType::CircuitStatusResponse,
            0xE1 => PacketType::RemoteLayoutRequest,
            0x21 => PacketType::RemoteLayoutResponse,
            0x05 => PacketType::ClockBroadcast,
            0x07 => PacketType::PumpStatus,
            _ => PacketType::Unknown,

        };
        Ok(output)
    }

}
