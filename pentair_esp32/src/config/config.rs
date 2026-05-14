use anyhow::Result;
use embedded_svc::nvs::Nvs;
use serde::{Deserialize, Serialize};

pub mod config_json;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PoolConfig {
    pub comms: config_json::Comms,
    pub port_parameters: config_json::PortParameters,
    pub system_parameters: config_json::SystemParameters,
}

pub fn load_configuration(nvs: &mut impl Nvs) -> Result<PoolConfig> {
    let config_str = nvs
        .get_str("pool_config")?
        .unwrap_or_else(|| "{}".to_string()); // Default to empty JSON if not found

    let config: PoolConfig = serde_json::from_str(&config_str)?;
    Ok(config)
}

pub fn save_configuration(nvs: &mut impl Nvs, config: &PoolConfig) -> Result<()> {
    let config_str = serde_json::to_string(config)?;
    nvs.set_str("pool_config", &config_str)?;
    Ok(())
}
