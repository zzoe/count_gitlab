use log::error;
use once_cell::sync::Lazy;

use crate::config::Config;

mod config;
pub mod statistic;
pub mod store;

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| smol::block_on(config::init()).expect("读取配置文件失败"));

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    if std::env::var("SMOL_THREADS").is_err() {
        std::env::set_var("SMOL_THREADS", num_cpus::get().to_string());
    }

    pretty_env_logger::init_timed();

    smol::block_on(run())
}

async fn run() {
    let mut tasks = Vec::new();

    for id in &CONFIG.git.ids {
        let id = *id;
        tasks.push(smol::spawn(async move {
            let code_statistics = statistic::run(id).await?;
            store::save(id, code_statistics).await
        }));
    }

    for task in tasks {
        if let Err(e) = task.await {
            error!("{}", e);
        }
    }
}
