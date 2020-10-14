use std::iter::FromIterator;

use log::{error, info, warn};
use once_cell::sync::Lazy;
use rbatis::rbatis::Rbatis;
use smol::stream::StreamExt;
use util::stream_vec::StreamVec;

use crate::config::Config;

mod config;
pub mod statistic;
pub mod store;

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| smol::block_on(config::init()).expect("读取配置文件失败"));

pub static RB: Lazy<Rbatis> = Lazy::new(Rbatis::new);

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    if std::env::var("SMOL_THREADS").is_err() {
        std::env::set_var("SMOL_THREADS", num_cpus::get().to_string());
    }

    pretty_env_logger::init_timed();

    smol::block_on(run());
    info!("统计结束");
}

async fn run() {
    init().await;
    let mut tasks = Vec::new();

    for id in &CONFIG.git.ids {
        let id = *id;
        tasks.push(smol::spawn(async move {
            let code_statistics = statistic::run(id).await?;
            store::save(id, code_statistics).await
        }));
    }

    let mut tasks = StreamVec::from_iter(tasks);
    while let Some(r) = tasks.next().await {
        if let Err(e) = r {
            error!("{}", e);
        }
    }
}

async fn init() {
    RB.link("sqlite://gitlab.db")
        .await
        .expect("link sqlite fail");

    let drop_sql = "drop table commit_log";
    if let Err(e) = RB.exec("", drop_sql).await {
        warn!("{}", e);
    }

    let create_sql = "create table commit_log (
            id number not null,
            full_id text not null,
            short_id text not null,
            author_name text not null,
            author_email text not null,
            authored_date text not null,
            committer_name text not null,
            committer_email text not null,
            committed_date text not null,
            created_at text not null,
            title text not null,
            message text not null,
            parent_ids text not null,
            additions text not null,
            deletions text not null,
            constraint commit_log_pk primary key (id, full_id)
        )";
    RB.exec("", create_sql).await.expect("crate table fail");
}
