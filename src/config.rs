use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub git: Git,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Git {
    pub addr: String,
    pub token: String,
    pub ids: Vec<usize>,
}

pub async fn init() -> Result<Config> {
    let cfg_name = "config.toml";
    init_config(cfg_name).await?;
    let res = smol::fs::read_to_string(cfg_name).await?;
    Ok(toml::from_str::<Config>(&*res)?)
}

async fn init_config(cfg_name: &str) -> Result<()> {
    if Path::new(cfg_name).exists() {
        return Ok(());
    }

    let cfg = Config {
        git: Git {
            addr: "http://devgit.z-bank.com".to_string(),
            token: "".to_string(),
            ids: vec![0_usize],
        },
    };

    Ok(smol::fs::write(cfg_name, toml::to_string_pretty(&cfg)?).await?)
}
