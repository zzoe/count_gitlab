use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::CONFIG;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub threads: usize,
    pub concurrent: usize,
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
pub fn init() -> Config {
    let cfg_name = "config.toml";
    init_config(cfg_name);
    let res = fs::read_to_string(cfg_name).expect("读取配置文件失败");
    toml::from_str::<Config>(&*res).expect("解析toml配置失败")
}

fn init_config(cfg_name: &str) {
    if Path::new(cfg_name).exists() {
        return;
    }

    let mut account = HashMap::new();
    account.insert("zs".to_string(), "张三".to_string());
    let mut project = HashMap::new();
    project.insert("705".to_string(), "综合理财".to_string());

    let cfg = Config {
        threads: num_cpus::get(),
        concurrent: 32,
        sqlite: "gitlab.db".to_string(),
        gitlab: Gitlab {
            addr: "http://devgit.z-bank.com/".to_string(),
            token: "".to_string(),
        },
        account,
        project,
    };

    fs::write(
        cfg_name,
        toml::to_string_pretty(&cfg).expect("初始化toml格式失败"),
    )
    .expect("初始化toml配置文件失败")
}
