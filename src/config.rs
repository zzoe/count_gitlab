use std::path::Path;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub git: Git,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Git {
    pub addr: String,
    pub token: String,
    pub id: usize,
}

pub fn init() -> Result<Config> {
    let cfg_name = "config.toml";
    init_config(cfg_name)?;
    let res = std::fs::read_to_string(cfg_name)?;
    Ok(toml::from_str::<Config>(&*res)?)
}

fn init_config(cfg_name: &str) -> Result<()> {
    if Path::new(cfg_name).exists() {
        return Ok(());
    }

    let cfg = Config {
        git: Git {
            addr: "http://devgit.z-bank.com".to_string(),
            token: "".to_string(),
            id: 0_usize,
        },
    };

    Ok(std::fs::write(cfg_name, toml::to_string_pretty(&cfg)?)?)
}
