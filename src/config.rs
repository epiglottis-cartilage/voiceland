use crate::Result;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    pub local_name: String,
    pub peers: Vec<IpAddr>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_str = std::fs::read_to_string("voiceland.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
