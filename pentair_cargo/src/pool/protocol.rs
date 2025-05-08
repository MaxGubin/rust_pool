use crate::{config, pool::message};
use crate::pool::message::system_state::SystemState;
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
        match message::ProtocolPacket::decode_packet(packet) {
            Ok(received_message) => {
                self.log_packet(packet);
                match received_message.decoded {
                    message::PacketType::Status(status) => {
                        self.system_state = status;
                    }
                    message::PacketType::CircuitStatusResponse => {
                        self.waiting_for_circuit_status_response = false;
                    }
                    message::PacketType::Unknown => {
                        self.unrecognized_bytes.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("Error decoding packet: {:?}", e);
                self.corrupted_packets.fetch_add(1, Ordering::Relaxed);
            }
        }

    }

    /// Checks

    // Changes a state of a control. Returns back True if it was changed, false if it was not
    // changed yet.
    pub fn change_circuit(&mut self, control_name: &str, state: bool) -> bool {
        if control_name == "pool" {
          //  self.system_state.pool_on = state;
        }
        if control_name == "spa" {
          //  self.system_state.spa_on = state;
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

