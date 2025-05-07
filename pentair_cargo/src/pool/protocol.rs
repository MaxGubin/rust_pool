use crate::config;
use axum::extract::ws::close_code::PROTOCOL;
use chrono::{DateTime, Local};
use log::{debug, error, trace, warn};
use serde::Serialize;
use serial::{self, Error, SerialPort};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Serialize)]
pub struct PacketLogElement {
    pub packet_content: Vec<u8>,
    pub timestamp: DateTime<Local>,
}

pub struct PoolProtocol {
    // This is the only one thread that reads/writes the port.
    // communication_thread: std::thread::JoinHandle,
    system_state: SystemState,

    // The version of the system
    version: u32,

    // We just sent a circuit change request and is waitng to CiercuitStatusREsponse
    waiting_for_circuit_status_response: bool,

    /// Keep a few recent packets for debugging/logging.
    recent_packets: Vec<PacketLogElement>,

    /// Different counters with protocol errors.
    unrecognized_bytes: AtomicU32,
    corrupted_packets: AtomicU32,
    short_packets: AtomicU32,
    unknown_protocol: AtomicU32,

    /// A queue of outgoing packets.
    outgoing: Vec<String>,
}

impl PoolProtocol {
    pub fn new() -> PoolProtocol {
        PoolProtocol {
            system_state: SystemState::new(),
            version: 0,
            waiting_for_circuit_status_response: false,
            recent_packets: Vec::new(),
            unrecognized_bytes: AtomicU32::new(0),
            corrupted_packets: AtomicU32::new(0),
            short_packets: AtomicU32::new(0),
            unknown_protocol: AtomicU32::new(0),
            outgoing: vec![],
        }
    }

    /// Returns the current state of the system.
    pub fn get_state(&self) -> SystemState {
        self.system_state.clone()
    }

    pub fn get_recent_packets(&self) -> Vec<PacketLogElement> {
        self.recent_packets.clone()
    }

    pub fn process_packet(&mut self, packet: &[u8]) {
        debug!("Processing packet {:?}", packet);
        const MINIMUM_PACKET_SIZE: usize = 4;
        const PROTOCOL_OFFSET: usize = 0;
        const COMMAND_OFFSET: usize = 3;
        if packet.len() < MINIMUM_PACKET_SIZE {
            warn!("Got a short packet");
            self.short_packets.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if packet[PROTOCOL_OFFSET] != 0x00 && packet[PROTOCOL_OFFSET] != 0x01 {
            warn!("Got a packet with invalid protocol");
            self.unknown_protocol.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let received_message = message::ProtocolPacket::new(packet);
        let decoded_message = received_message.decode_packet();


        match packet[COMMAND_OFFSET] {
            0x02 => match SystemState::from_packet(packet) {
                Ok(state) => {
                    self.system_state = state;
                }
                Err(e) => {
                    error!("Failed to process packet: {}", e);
                }
            },
            _ => {
                warn!("Got a packet with unknown command");
            }
        }
    }

    pub fn pocket_type(&self, packet: &[u8]) -> PacketType {
        const MINIMUM_PACKET_SIZE: usize = 4;
        const PROTOCOL_OFFSET: usize = 0;
        const COMMAND_OFFSET: usize = 3;
        const SOURCE_OFFSET: usize = 2;
        const DEST_OFFSET: usize = 1;
        if packet.len() < MINIMUM_PACKET_SIZE {
            return PacketType::Unknown;
        }
        if packet[PROTOCOL_OFFSET] != 0x00 && packet[PROTOCOL_OFFSET] != 0x01 {
            return PacketType::Unknown;
        }

        match packet[COMMAND_OFFSET] {
            0x02 => PacketType::Status,
            0x86 => PacketType::CircuitStatusChange,
            0x01 => PacketType::CircuitStatusResponse,
            0xE1 => PacketType::RemoteLayoutRequest,
            0x21 => PacketType::RemoteLayoutResponse,
            0x05 => PacketType::ClockBroadcast,
            0x07 => PacketType::PumpStatus,
            _ => PacketType::Unknown,
        }
    }

    /// Checks

    // Changes a state of a control. Returns back True if it was changed, false if it was not
    // changed yet.
    pub fn change_circuit(&mut self, control_name: &str, state: bool) -> bool {
        if control_name == "pool" {
            self.system_state.pool_on = state;
        }
        if control_name == "spa" {
            self.system_state.spa_on = state;
        }
        let mut _packet = vec![0x0];
        true
    }

    pub fn log_packet(&mut self, pckt: &[u8]) {
        self.recent_packets.push(PacketLogElement {
            packet_content: pckt.to_vec(),
            timestamp: Local::now(),
        });

        if self.recent_packets.len() > 10 {
            self.recent_packets.remove(0);
        }
    }
}

