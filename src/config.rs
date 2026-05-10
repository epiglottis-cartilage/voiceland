use crate::Result;
use serde::Deserialize;
use std::{net::IpAddr, path::Path};

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    pub name: String,
    pub peers: Vec<IpAddr>,
    pub buffer_len: u8,
    pub denoise: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
