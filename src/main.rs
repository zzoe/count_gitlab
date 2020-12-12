use std::iter::FromIterator;

use anyhow::Result;
use log::{error, info};
use once_cell::sync::Lazy;
use smol::stream::StreamExt;
use util::stream_vec::StreamVec;

use crate::config::Config;

mod config;
pub mod excel;
pub mod gitlab;
pub mod sqlite;

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| smol::block_on(config::init()).expect("读取配置文件失败"));

fn main() {
    if std::env::var("SMOL_THREADS").is_err() {
        std::env::set_var("SMOL_THREADS", num_cpus::get().to_string());
    }

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    info!("统计开始");
    if let Err(e) = smol::block_on(run()) {
        error!("统计失败: {}", e)
    }
}

async fn run() -> Result<()> {
    let mut tasks = Vec::new();

    for id in &CONFIG.git.ids {
        tasks.push(smol::spawn(async move { gitlab::deal_project(*id).await }));
    }

    let mut tasks = StreamVec::from_iter(tasks);
    let conn = sqlite::connect()?;
    let mut stmt = sqlite::prepare_insert(&conn)?;

    let mut fail = 0_usize;
    while let Some(res) = tasks.next().await {
        if let Err(e) = res.and_then(|logs| sqlite::insert(&mut stmt, logs)) {
            error!("{}", e);
            fail += 1;
        }
    }

    info!("统计结束: {}个项目，失败{}个 ", CONFIG.git.ids.len(), fail);

    if fail == 0 {
        excel::create(&conn)
    }

    Ok(())
}
