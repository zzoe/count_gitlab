use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::CONFIG;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub threads: usize,
    pub sqlite: String,
    pub gitlab: Gitlab,
    pub account: HashMap<String, String>,
    pub project: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gitlab {
    pub addr: String,
    pub token: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectId(pub u32);

impl Display for ProjectId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = self.0.to_string();
        let value = CONFIG.project.get(&id).unwrap_or(&id);
        write!(f, "{}", value)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Author(pub String);

impl Display for Author {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CONFIG.account.get(&self.0).unwrap_or(&self.0))
    }
}
pub fn init() -> Result<Config> {
    let cfg_name = "config.toml";
    init_config(cfg_name)?;
    let res = fs::read_to_string(cfg_name)?;
    Ok(toml::from_str::<Config>(&*res)?)
}

fn init_config(cfg_name: &str) -> Result<()> {
    if Path::new(cfg_name).exists() {
        return Ok(());
    }

    let mut account = HashMap::new();
    account.insert("zs".to_string(), "张三".to_string());
    let mut project = HashMap::new();
    project.insert("705".to_string(), "综合理财".to_string());

    let cfg = Config {
        threads: num_cpus::get(),
        sqlite: "gitlab.db".to_string(),
        gitlab: Gitlab {
            addr: "http://devgit.z-bank.com/".to_string(),
            token: "".to_string(),
        },
        account,
        project,
    };

    Ok(fs::write(cfg_name, toml::to_string_pretty(&cfg)?)?)
}
