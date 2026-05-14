use anyhow::Result;
use log::{debug, error, trace, warn};
use serde::Serialize;

/// The decoded package with the system state.

#[derive(Clone, Debug, Default, Serialize)]
pub struct SystemState {
    // Different switches, usually in the state on/off
    pub pool_on: bool,
    pub spa_on: bool,
    pub aux_circuits: Vec<bool>,
    pub feature_circuits: Vec<bool>,

    // Block of temperatures
    pub water_temp: u32,
    pub air_temp: u32,
    pub solar_temp: u32,
}

impl SystemState {
    pub fn new() -> SystemState {
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
    pub fn from_packet(packet: &[u8]) -> Result<SystemState> {
        debug!("Processing packet {:?}", packet);
        if packet.len() < 8 {
            anyhow::bail!("Packet too short");
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
        let mut controls = vec![
            ("pool".to_string(), self.pool_on),
            ("spa".to_string(), self.spa_on),
        ];
        // Append aux circuits dynamically
        for (i, &state) in self.aux_circuits.iter().enumerate() {
            controls.push((format!("AUX{}", i + 1), state));
        }
        for (i, &state) in self.feature_circuits.iter().enumerate() {
            controls.push((format!("FEATURE{}", i + 1), state));
        }
        controls
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
