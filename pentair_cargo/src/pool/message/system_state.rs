use log::{debug, error, trace, warn};
use serde::Serialize;
use serial::{self, Error};


/// The decoded package with the system state.

#[derive(Clone, Debug)]
pub struct SystemState {

    // Different switches, usually in the state on/off
    pool_on: bool,
    spa_on: bool,
    aux_circuits: Vec<bool>,
    feature_circuits: Vec<bool>,

    // Block of temperatures
    water_temp: u32,
    air_temp: u32,
    solar_temp: u32,
}


impl SystemState {
    fn new() -> SystemState {
        SystemState {
            pool_on: false,
            spa_on: false,
            aux_circuits: Vec::new(),
            feature_circuits: Vec::new(),
            water_temp: 0,
            air_temp: 0,
            solar_temp: 0,
        }
    }
    pub fn from_packet(packet: &[u8]) -> Result<SystemState, serial::Error> {
        debug!("Processing packet {:?}", packet);
        if packet.len() < 8 {
            return Err(Error::new(
                serial::ErrorKind::InvalidInput,
                "Packet too short",
            ));
        }

        let mut state = Self::new();

        const MASK_IDX: usize = 7;
        const SPA_MASK: u8 = 0x01;
        const AUX1_MASK: u8 = 0x02;
        const AUX2_MASK: u8 = 0x04;
        const AUX3_MASK: u8 = 0x08;
        const POOL_MASK: u8 = 0x20;
        const FEATURE1_MASK: u8 = 0x10;
        const FEATURE2_MASK: u8 = 0x40;
        const FEATURE3_MASK: u8 = 0x80;

        {
            state.pool_on = (packet[MASK_IDX] & POOL_MASK) != 0;
            state.spa_on = (packet[MASK_IDX] & SPA_MASK) != 0;
            state.aux_circuits.push((packet[MASK_IDX] & AUX1_MASK) != 0);
            state.aux_circuits.push((packet[MASK_IDX] & AUX2_MASK) != 0);
            state.aux_circuits.push((packet[MASK_IDX] & AUX3_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE1_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE2_MASK) != 0);
            state
                .feature_circuits
                .push((packet[MASK_IDX] & FEATURE3_MASK) != 0);
        }

        Ok(state)
    }

    // Checks the current state
    pub fn get_controls_state(&self) -> Vec<(String, bool)> {
        vec![
            ("pool".to_string(), self.pool_on),
            ("spa".to_string(), self.spa_on),
        ]
    }

    //
    pub fn get_temperatures(&self) -> Vec<(String, f32)> {
        vec![
            ("water".to_string(), 82.),
            ("air".to_string(), 72.),
            ("solar".to_string(), 83.),
        ]
    }
}

#[cfg(test)]
#[test]
fn test_system_state_from_packet() {
    let packet = vec![
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x53, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5F, 0x00, 0x00, 0x00,
        0x3C, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0xf4, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // \\x00\\xf6\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x02\\x00\\x02\\x00\\xf7\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x06\\x01\\n\\x00\\xf8\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xf9\\x01\\x00\\x00\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfa\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfb\\x01\\x00\\x00\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfc\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xfe\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xff\\x01\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\xff\\x02\\x00\\x00\\x1f\\x03\\x00\\x00\\xff\\xff\\xff\\xff\\x00\\x00\\x00\\x00\\x06\\x00\\x00\\x00\\x07\\x00\\x00\\x00\\x00\\x00\\x00\\x00'"

    let state = SystemState::from_packet(&packet).unwrap();
    assert!(state.pool_on);
    assert!(!state.spa_on);
    assert!(state.aux_circuits[0]);
    assert!(!state.aux_circuits[1]);
    assert!(state.aux_circuits[2]);
    assert!(!state.feature_circuits[0]);
    assert!(!state.feature_circuits[1]);
    assert!(!state.feature_circuits[2]);
}
