use std::path::Path;

use anyhow::Result;
use serde::export::Formatter;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub store_type: StoreType,
    pub sqlite: String,
    pub gitlab: Gitlab,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoreType {
    SQLite,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gitlab {
    pub addr: String,
    pub token: String,
    pub ids: Vec<ProjectID>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectID(pub u32);

impl Display for ProjectID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            705 => write!(f, "综合理财后管"),
            706 => write!(f, "综合理财"),
            715 => write!(f, "综合理财底层"),
            other => write!(f, "未识别的项目代码[{}]", other),
        }
    }
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
        store_type: StoreType::SQLite,
        sqlite: "gitlab.db".to_string(),
        gitlab: Gitlab {
            addr: "http://devgit.z-bank.com/".to_string(),
            token: "".to_string(),
            ids: vec![ProjectID(705), ProjectID(706), ProjectID(715)],
        },
    };

    Ok(smol::fs::write(cfg_name, toml::to_string_pretty(&cfg)?).await?)
}
